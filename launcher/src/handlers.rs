use std::convert::Infallible;
use std::path::PathBuf;

use slog::{crit, info, Logger};
use warp::http::StatusCode;

use crate::node::{LightNodeConfiguration, LightNodeStateRef, LightNodeRunner};

pub async fn start_node_with_config(
    cfg: LightNodeConfiguration,
    log: Logger,
    state: LightNodeStateRef
) -> Result<impl warp::Reply, Infallible> {

    info!(
        log,
        "Received request to start the light node with config: {:?}", cfg
    );

    // TODO: should add into the request
    let path = PathBuf::from(r"./target/release/light-node");

    let mut state = state.write().unwrap();
    let process = state.process.as_mut();

    // No process started yet
    if process.is_none() || !LightNodeRunner::is_running(process.unwrap()) {
        // TODO better error handling (unwrap...)
        let runner = LightNodeRunner::new("light-node", path, cfg).spawn().unwrap();

        state.process = Some(runner);
    } else {
        crit!(log, "Light node is allready running");
        return Ok(StatusCode::FORBIDDEN)
    }


    Ok(StatusCode::OK)
}

pub async fn stop_node(
    log: Logger,
    state: LightNodeStateRef
) -> Result<impl warp::Reply, Infallible> {
    // println!("SUPPLIED CONFIG: {:?}", cfg);

    let mut state = state.write().unwrap();
    let process = state.process.as_mut().unwrap();

    if LightNodeRunner::is_running(process) {
        info!(log, "Stopping the node");
        LightNodeRunner::terminate_ref(process);
        
    }

    Ok(StatusCode::OK)
}