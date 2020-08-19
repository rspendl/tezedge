use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use slog::{info, Drain, Level, Logger};

mod filters;
mod handlers;
mod node_runner;

#[tokio::main]
async fn main() {
    let log = create_logger();

    // TODO: should add an argument?
    let path = PathBuf::from(r"./target/release/light-node");

    let runner = Arc::new(RwLock::new(node_runner::LightNodeRunner::new(
        "light-node-0",
        path,
    )));

    let api = filters::launcher(log.clone(), runner);

    // TODO: add argument handling (clap)
    // TODO: enable custom port definition
    info!(log, "Starting the launcher RPC server");
    warp::serve(api).run(([0, 0, 0, 0], 3030)).await;
}

/// Creates a slog Logger
fn create_logger() -> Logger {
    // TODO: should we enable different log levels?

    let drain = slog_async::Async::new(
        slog_term::FullFormat::new(slog_term::TermDecorator::new().build())
            .build()
            .fuse(),
    )
    .chan_size(32768)
    .overflow_strategy(slog_async::OverflowStrategy::Block)
    .build()
    .filter_level(Level::Trace)
    .fuse();
    Logger::root(drain, slog::o!())
}
