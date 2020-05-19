use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Response, Server, StatusCode,
};
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use log::{info, warn};
use std::future::Future;
use std::sync::Arc;

use super::schema::{Context, Query, Schema};
use crate::context::AppContext;

pub async fn graphql_server<F>(ctx: Arc<AppContext>, shutdown_signal: F)
where
    F: Future<Output = ()>,
{
    let config = &ctx.config.graphql;

    if !config.enable {
        warn!("graphql server disabled");
        return;
    }

    let addr = config.endpoint.parse().expect("malformed endpoint address");

    let root_node: Arc<Schema> = Arc::new(RootNode::new(Query, EmptyMutation::new(), EmptySubscription::new()));
    let ctx = Arc::new(Context { app: ctx });

    let graphql_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        let ctx = ctx.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let ctx = ctx.clone();

                info!(
                    "{:?} {} {:?} {:?}",
                    req.method(),
                    req.uri(),
                    req.headers().get("user-agent").unwrap(),
                    req.headers().get("x-forwarded-for"),
                );
                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => juniper_hyper::graphiql("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            juniper_hyper::graphql(root_node, ctx, req).await
                        }
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            Ok(response)
                        }
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(graphql_service);
    info!("listening on http://{}", addr);

    let _ = server.with_graceful_shutdown(shutdown_signal).await;
}
