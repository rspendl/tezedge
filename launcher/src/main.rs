use std::sync::{Arc, RwLock};

use slog::{crit, debug, error, info, Drain, Level, Logger};

mod filters;
mod handlers;
mod node;


#[tokio::main]
async fn main() {
    let log = create_logger();

    let shared_state = Arc::new(RwLock::new(node::LightNodeState {
        process: None
    }));

    let api = filters::launcher(log.clone(), shared_state);

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
    .filter_level(Level::Info)
    .fuse();
    Logger::root(drain, slog::o!())
}
