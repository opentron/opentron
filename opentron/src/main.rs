// NOTE: Embedding slog macros and select! requires increasing recursion_limit.
#![recursion_limit = "1024"]
use std::error::Error;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use futures::channel::oneshot;
use futures::join;
use log::info;
use slog::{o, slog_debug, slog_info, Drain};
use slog_scope_futures::FutureExt as SlogFutureExt;
use tokio_compat_02::FutureExt as Compat02FutureExt;

use channel_service::server::channel_server;
use context::AppContext;
use discovery_service::server::discovery_server;
use graphql_service::server::graphql_server;
use opentron::util::get_my_ip;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ! init app command line arguments
    let yaml = clap::load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    // ! init loggers
    let decorator = slog_term::TermDecorator::new().build();
    // let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = if matches.is_present("debug") {
        slog::LevelFilter(drain, slog::Level::Debug).fuse()
    } else {
        slog::LevelFilter(drain, slog::Level::Info).fuse()
    };

    let logger = slog::Logger::root(drain, o!());

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    let config_file = matches.value_of("config").expect("has default in cli.yml; qed");

    // ! #[tokio::main] runner
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get_physical())
        .thread_name("tokio-pool")
        .enable_all()
        .build()?;

    slog_info!(slog_scope::logger(), "use config file"; "path" => config_file);
    let mut ctx = AppContext::from_config(config_file)?;
    let outbound_ip = get_my_ip().unwrap_or("127.0.0.1".into());
    info!("outbound ip address: {}", outbound_ip);
    ctx.outbound_ip = outbound_ip;

    slog_debug!(slog_scope::logger(), "loaded config"; "config" => format!("{:#?}", ctx.config));

    match matches.subcommand() {
        ("check", Some(arg_matches)) => {
            let fut = opentron::commands::check::main(ctx, arg_matches);
            rt.block_on(fut)
        }
        ("fix", Some(arg_matches)) => {
            let fut = opentron::commands::fix::main(ctx, arg_matches);
            rt.block_on(fut)
        }
        ("dev", Some(_)) => {
            let fut = opentron::commands::dev::main(ctx);
            rt.block_on(fut)
        }
        _ => {
            let fut = run(ctx).compat();
            rt.block_on(fut)
        }
    }
}

// NOTE: #[tokio::main] conflicts with slog_scope, cause data race in global static resource release.
async fn run(ctx: AppContext) -> Result<(), Box<dyn Error>> {
    let ctx = Arc::new(ctx);

    let (termination_tx, termination_done) = oneshot::channel::<()>();
    let termination_handler = {
        let ctx = ctx.clone();
        move || {
            ctx.running.store(false, Ordering::SeqCst);
            ctx.chain_db.report_status();
            let _ = ctx.termination_signal.send(());
            unsafe {
                ctx.chain_db.prepare_close();
            }
            let _ = termination_tx.send(());
        }
    };

    let f = Mutex::new(Some(termination_handler));
    ctrlc::set_handler(move || {
        eprintln!("\nCtrl-C pressed. Now shuting down gracefully... ");
        if let Ok(mut guard) = f.lock() {
            if let Some(f) = guard.take() {
                f();
            } else {
                eprintln!("\nCtrl-C pressed again, be patient!");
            }
        }
    })
    .expect("Error setting Ctrl-C handler");

    let graphql_service = {
        let ctx = ctx.clone();
        let done_signal = ctx.termination_signal.subscribe();
        let logger = slog_scope::logger().new(o!("service" => "graphql"));
        graphql_server(ctx, done_signal).with_logger(logger)
    };

    let channel_service = {
        let ctx = ctx.clone();
        let done_signal = ctx.termination_signal.subscribe();
        channel_server(ctx, done_signal)
    };

    let discovery_service = {
        let ctx = ctx.clone();
        let done_signal = ctx.termination_signal.subscribe();
        let logger = slog_scope::logger().new(o!("service" => "discovery"));
        discovery_server(ctx, done_signal).with_logger(logger)
    };
    let _ = join!(graphql_service, channel_service, discovery_service);

    Ok(termination_done.await?)
}
