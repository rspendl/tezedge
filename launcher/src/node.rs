use std::process::{Child, Command};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::sync::{Arc, RwLock};

use getset::Getters;
use serde::{Deserialize, Serialize};
use wait_timeout::ChildExt;

#[derive(Debug, Deserialize, Serialize, Clone, Getters)]
#[getset(get_copy = "pub")]
pub struct LightNodeConfiguration {
    tezos_data_dir: Option<String>,
    identity_file: Option<String>,
    identity_expected_pow: Option<u16>,
    bootstrap_db_path: Option<String>,
    db_cfg_max_threads: Option<i32>,
    db_cfg_max_open_files: Option<i32>,
    bootstrap_lookup_address: Option<String>,
    disable_bootstrap_lookup: Option<bool>,
    log_file: Option<String>,
    log_format: Option<String>,
    log_level: Option<String>,
    ocaml_log_enabled: Option<bool>,
    network: Option<String>,
    p2p_port: Option<usize>,
    rpc_port: Option<usize>,
    websocket_address: Option<String>,
    monitor_port: Option<usize>,
    peers: Option<String>,
    peer_thresh_low: Option<usize>,
    peer_thresh_high: Option<usize>,
    protocol_runner: Option<String>,
    ffi_calls_gc_threshold: Option<usize>,
    ffi_pool_max_connections: Option<usize>,
    ffi_pool_connection_timeout_in_secs: Option<usize>,
    ffi_pool_max_lifetime_in_secs: Option<usize>,
    ffi_pool_idle_timeout_in_secs: Option<usize>,
    store_context_actions: Option<bool>,
    tokio_threads: Option<usize>,
    enable_testchain: Option<bool>,
    sandbox_patch_context_json_file: Option<String>,
    disable_mempool: Option<bool>,
    private_node: Option<bool>,
    config_file: Option<String>,
}
/// Thread safe reference to a shared RPC state
pub type LightNodeStateRef = Arc<RwLock<LightNodeState>>;

pub struct LightNodeState {
    // TODO: more than one?
    pub process: Option<Child>,
}

pub struct LightNodeRunner {
    config: LightNodeConfiguration,
    executable_path: PathBuf,
    name: String,
    // TODO: anything else?
}

// TODO: maybe implement (and possible rename to just Runner?) the trait ProtocolRunner found in tezos/wrapper/src/service.rs
impl LightNodeRunner {
    const PROCESS_WAIT_TIMEOUT: Duration = Duration::from_secs(4);

    pub fn new(name: &str, executable_path: PathBuf, cfg: LightNodeConfiguration) -> Self {
        Self {
            config: cfg,
            executable_path,
            name: name.to_string(),
        }
    }

    pub fn spawn(&self) -> Result<Child, failure::Error> {
        let process = Command::new(&self.executable_path)
            .args(&self.construct_args())
            .spawn()?;
        Ok(process)
    }

    pub fn terminate(mut process: Child) {
        match process.wait_timeout(Self::PROCESS_WAIT_TIMEOUT).unwrap() {
            Some(_) => (),
            None => {
                // child hasn't exited yet
                let _ = process.kill();
            }
        };
    }

    pub fn terminate_ref(process: &mut Child) {
        match process.wait_timeout(Self::PROCESS_WAIT_TIMEOUT).unwrap() {
            Some(_) => (),
            None => {
                // child hasn't exited yet
                let _ = process.kill();
            }
        };
    }

    pub fn is_running(process: &mut Child) -> bool {
        match process.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }

    fn construct_args(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        let cfg = &self.config;
        if let Some(tezos_data_dir) = &cfg.tezos_data_dir {
            args.push("--tezos-data-dir".to_string());
            args.push(tezos_data_dir.to_string());
        }
        if let Some(identity_file) = &cfg.identity_file {
            args.push("--identity-file".to_string());
            args.push(identity_file.to_string());
        }
        if let Some(identity_expected_pow) = &cfg.identity_expected_pow {
            args.push("--identity-expected-pow".to_string());
            args.push(identity_expected_pow.to_string());
        }
        if let Some(bootstrap_db_path) = &cfg.bootstrap_db_path {
            args.push("--bootstrap-db-path".to_string());
            args.push(bootstrap_db_path.to_string());
        }
        if let Some(db_cfg_max_threads) = &cfg.db_cfg_max_threads {
            args.push("--db-cfg-max-threads".to_string());
            args.push(db_cfg_max_threads.to_string());
        }
        if let Some(db_cfg_max_open_files) = &cfg.db_cfg_max_open_files {
            args.push("--db-cfg-max-open-files".to_string());
            args.push(db_cfg_max_open_files.to_string());
        }
        if let Some(bootstrap_lookup_address) = &cfg.bootstrap_lookup_address {
            args.push("--bootstrap-lookup-address".to_string());
            args.push(bootstrap_lookup_address.to_string());
        }
        if let Some(_) = &cfg.disable_bootstrap_lookup {
            args.push("--disable-bootstrap-lookup".to_string());
            // args.push(disable_bootstrap_lookup.to_string());
        }
        if let Some(log_file) = &cfg.log_file {
            args.push("--log-file".to_string());
            args.push(log_file.to_string());
        }
        if let Some(log_format) = &cfg.log_format {
            args.push("--log-format".to_string());
            args.push(log_format.to_string());
        }
        if let Some(log_level) = &cfg.log_level {
            args.push("--log-level".to_string());
            args.push(log_level.to_string());
        }
        if let Some(ocaml_log_enabled) = &cfg.ocaml_log_enabled {
            args.push("--ocaml-log-enabled".to_string());
            args.push(ocaml_log_enabled.to_string());
        }
        if let Some(network) = &cfg.network {
            args.push("--network".to_string());
            args.push(network.to_string());
        }
        if let Some(p2p_port) = &cfg.p2p_port {
            args.push("--p2p-port".to_string());
            args.push(p2p_port.to_string());
        }
        if let Some(rpc_port) = &cfg.rpc_port {
            args.push("--rpc-port".to_string());
            args.push(rpc_port.to_string());
        }
        if let Some(websocket_address) = &cfg.websocket_address {
            args.push("--websocket-address".to_string());
            args.push(websocket_address.to_string());
        }
        if let Some(monitor_port) = &cfg.monitor_port {
            args.push("--monitor-port".to_string());
            args.push(monitor_port.to_string());
        }
        if let Some(peers) = &cfg.peers {
            args.push("--peers".to_string());
            args.push(peers.to_string());
        }
        if let Some(peer_thresh_low) = &cfg.peer_thresh_low {
            args.push("--peer-thresh-low".to_string());
            args.push(peer_thresh_low.to_string());
        }
        if let Some(peer_thresh_high) = &cfg.peer_thresh_high {
            args.push("--peer-thresh-high".to_string());
            args.push(peer_thresh_high.to_string());
        }
        if let Some(protocol_runner) = &cfg.protocol_runner {
            args.push("--protocol-runner".to_string());
            args.push(protocol_runner.to_string());
        }
        if let Some(ffi_calls_gc_threshold) = &cfg.ffi_calls_gc_threshold {
            args.push("--ffi-calls-gc-threshold".to_string());
            args.push(ffi_calls_gc_threshold.to_string());
        }
        if let Some(ffi_pool_max_connections) = &cfg.ffi_pool_max_connections {
            args.push("--ffi-pool-max-connections".to_string());
            args.push(ffi_pool_max_connections.to_string());
        }
        if let Some(ffi_pool_connection_timeout_in_secs) = &cfg.ffi_pool_connection_timeout_in_secs {
            args.push("--ffi-pool-connection-timeout-in-secs".to_string());
            args.push(ffi_pool_connection_timeout_in_secs.to_string());
        }
        if let Some(ffi_pool_max_lifetime_in_secs) = &cfg.ffi_pool_max_lifetime_in_secs {
            args.push("--ffi-pool-max-lifetime-in-secs".to_string());
            args.push(ffi_pool_max_lifetime_in_secs.to_string());
        }
        if let Some(ffi_pool_idle_timeout_in_secs) = &cfg.ffi_pool_idle_timeout_in_secs {
            args.push("--ffi-pool-idle-timeout-in-secs".to_string());
            args.push(ffi_pool_idle_timeout_in_secs.to_string());
        }
        if let Some(store_context_actions) = &cfg.store_context_actions {
            args.push("--store-context-actions".to_string());
            args.push(store_context_actions.to_string());
        }
        if let Some(tokio_threads) = &cfg.tokio_threads {
            args.push("--tokio-threads".to_string());
            args.push(tokio_threads.to_string());
        }
        if let Some(enable_testchain) = &cfg.enable_testchain {
            args.push("--enable-testchain".to_string());
            args.push(enable_testchain.to_string());
        }
        if let Some(sandbox_patch_context_json_file) = &cfg.sandbox_patch_context_json_file {
            args.push("--sandbox-patch-context-json-file".to_string());
            args.push(sandbox_patch_context_json_file.to_string());
        }
        if let Some(disable_mempool) = &cfg.disable_mempool {
            args.push("--disable-mempool".to_string());
            args.push(disable_mempool.to_string());
        }
        if let Some(private_node) = &cfg.private_node {
            args.push("--private-node".to_string());
            args.push(private_node.to_string());
        }
        if let Some(config_file) = &cfg.config_file {
            args.push("--config-file".to_string());
            args.push(config_file.to_string());
        }
        
        args
    }
}