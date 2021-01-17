use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

use async_graphql::{EmptySubscription, Schema};
use async_graphql_warp::BadRequest;
use http::StatusCode;
use log::{info, trace, warn};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use warp::{Filter, Rejection};

use super::schema::{MutationRoot, QueryRoot};
use crate::context::AppContext;

pub async fn graphql_server(ctx: Arc<AppContext>, mut shutdown_signal: broadcast::Receiver<()>) {
    let config = &ctx.config.graphql;

    if !config.enable {
        warn!("graphql server disabled");
        return;
    }

    let addr: SocketAddr = config
        .endpoint
        .parse()
        .expect("malformed endpoint address for graphql server");

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(ctx)
        .finish();

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (Schema<_, _, _>, async_graphql::Request)| async move {
            trace!("req: {:?}", request.query);
            Ok::<_, Infallible>(async_graphql_warp::Response::from(schema.execute(request).await))
        },
    );
    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        warp::http::Response::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    });

    let routes = graphql_playground
        .or(graphql_post)
        .recover(|err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(err.to_string(), StatusCode::BAD_REQUEST));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        });

    let (listening_addr, fut) = warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
        shutdown_signal.recv().await.ok();
    });

    info!("listening on http://{}", listening_addr);

    fut.await;
}
