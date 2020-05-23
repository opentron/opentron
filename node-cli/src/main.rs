#![recursion_limit = "1024"]

use futures::channel::oneshot;
use futures::future::FutureExt;
use futures::join;
use log::info;
use slog::{o, Drain};
use slog_scope_futures::FutureExt as SlogFutureExt;
use std::error::Error;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

use node_cli::channel::server::channel_server;
use node_cli::context::AppContext;
use node_cli::graphql::server::graphql_server;
use node_cli::discovery::server::discovery_server;
use node_cli::util::get_my_ip;

pub enum PeerStatus {
    HandshakeFinished,
    BeforeSyncing,
    Syncing,
    BackingUp,
}

pub enum Direction {
    Inbound,
    Outbound,
}

/*
pub struct PeerConnectionContext {
    peer_addr: SocketAddr,
    done: oneshot::Receiver<()>,
    syncing: RwLock<bool>,
}
*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ! init app command line arguments
    let yaml = clap::load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    // ! init loggers
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    // let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, slog::Level::Info).fuse();

    let logger = slog::Logger::root(drain, o!());

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    let config_file = matches.value_of("config").expect("has default in cli.yml; qed");

    // ! #[tokio::main] runner
    let mut rt = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .core_threads(num_cpus::get_physical())
        .thread_name("tokio-pool")
        .enable_all()
        .build()?;

    match matches.subcommand() {
        ("check", Some(arg_matches)) => {
            let fut = node_cli::commands::check::main(config_file, arg_matches);
            rt.block_on(fut)
        }
        ("fix", Some(arg_matches)) => {
            let fut = node_cli::commands::fix::main(config_file, arg_matches);
            rt.block_on(fut)
        }
        _ => {
            let fut = run(config_file);
            rt.block_on(fut)
        }
    }
}

// NOTE: #[tokio::main] conflicts with slog_scope, cause data race in global static resource release.
async fn run<P: AsRef<Path>>(config_file: P) -> Result<(), Box<dyn Error>> {
    let mut ctx = AppContext::from_config(config_file)?;
    ctx.outbound_ip = get_my_ip().await?;
    info!("outbound ip address: {}", ctx.outbound_ip);
    let ctx = Arc::new(ctx);

    // Fix: account state root First appares in 8222293

    /*
    for num in 1102553..1135973 {
        if let Some(blk) = ctx.db.get_block_by_number(num) {
            println!("delete {} => {:?}", num, ctx.db.delete_block(&blk));
        } else {
            println!("done");
            return Ok(());
        }
    }
    */

    let (termination_tx, termination_done) = oneshot::channel::<()>();
    let (channel_tx, channel_done) = oneshot::channel::<()>();
    let (graphql_tx, graphql_done) = oneshot::channel::<()>();
    let (discovery_tx, discovery_done) = oneshot::channel::<()>();
    let termination_handler = {
        let ctx = ctx.clone();
        move || {
            let _ = channel_tx.send(());
            let _ = graphql_tx.send(());
            let _ = discovery_tx.send(());
            while let Some(done) = ctx.peers.write().unwrap().pop() {
                let _ = done.send(());
            }
            ctx.running.store(false, Ordering::SeqCst);
            ctx.db.report_status();
            unsafe {
                ctx.db.prepare_close();
            }
            let _ = termination_tx.send(());
        }
    };

    let f = Mutex::new(Some(termination_handler));
    ctrlc::set_handler(move || {
        eprintln!("\nCtrl-C pressed...");
        if let Ok(mut guard) = f.lock() {
            let f = guard.take().expect("f can only be taken once");
            f();
        }
    })
    .expect("Error setting Ctrl-C handler");

    let graphql_service = {
        let ctx = ctx.clone();
        let logger = slog_scope::logger().new(o!("service" => "graphql"));
        graphql_server(ctx, graphql_done.map(|_| ())).with_logger(logger)
    };

    let channel_service = {
        let ctx = ctx.clone();
        let logger = slog_scope::logger().new(o!("service" => "channel"));
        channel_server(ctx, channel_done.map(|_| ())).with_logger(logger)
    };

    let discovery_service = {
        let ctx = ctx.clone();
        let logger = slog_scope::logger().new(o!("service" => "discovery"));
        discovery_server(ctx, discovery_done.map(|_| ()).with_logger(logger))
    };
    let _ = join!(graphql_service, channel_service, discovery_service);

    Ok(termination_done.await?)
}
