// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::channel::{bounded, Receiver, RecvError, Sender, SendError};
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;
use std::cmp::Ordering::Equal;

static CHANNEL_ENABLED: AtomicBool = AtomicBool::new(false);
const CHANNEL_BUFFER_LEN: usize = 1_048_576;

lazy_static! {
    /// This channel is shared by both OCaml and Rust
    static ref CHANNEL: (Sender<ContextAction>, Receiver<ContextAction>) = bounded(CHANNEL_BUFFER_LEN);
}

/// Send message into the shared channel.
pub fn context_send(action: ContextAction) -> Result<(), SendError<ContextAction>> {
    if CHANNEL_ENABLED.load(Ordering::Acquire) {
        CHANNEL.0.send(action)
    } else {
        Ok(())
    }
}

/// Receive message from the shared channel.
pub fn context_receive() -> Result<ContextAction, RecvError> {
    CHANNEL.1.recv()
}

/// By default channel is disabled.
///
/// This is needed to prevent unit tests from overflowing the shared channel.
pub fn enable_context_channel() {
    CHANNEL_ENABLED.store(true, Ordering::Release)
}

type Hash = Vec<u8>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContextAction {
    Set {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        key: Vec<String>,
        value: Vec<u8>,
        value_as_json: Option<String>,
        start_time: f64,
        end_time: f64,
    },
    Delete {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        key: Vec<String>,
        start_time: f64,
        end_time: f64,
    },
    RemoveRecursively {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        key: Vec<String>,
        start_time: f64,
        end_time: f64,
    },
    Copy {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        from_key: Vec<String>,
        to_key: Vec<String>,
        start_time: f64,
        end_time: f64,
    },
    Checkout {
        context_hash: Hash,
        start_time: f64,
        end_time: f64,
    },
    Commit {
        parent_context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        new_context_hash: Hash,
        start_time: f64,
        end_time: f64,
    },
    Mem {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        key: Vec<String>,
        start_time: f64,
        end_time: f64,
    },
    DirMem {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        key: Vec<String>,
        start_time: f64,
        end_time: f64,
    },
    Get {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        key: Vec<String>,
        start_time: f64,
        end_time: f64,
    },
    Fold {
        context_hash: Option<Hash>,
        block_hash: Option<Hash>,
        operation_hash: Option<Hash>,
        key: Vec<String>,
        start_time: f64,
        end_time: f64,
    },
    /// This is a control event used to shutdown IPC channel
    Shutdown,
}

fn get_time(action: &ContextAction) -> f64 {
    match action {
        ContextAction::Set { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Delete { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::RemoveRecursively { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Copy { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Checkout { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Commit { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Mem { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::DirMem { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Get { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Fold { start_time, end_time, .. } => *end_time - *start_time,
        ContextAction::Shutdown => 0f64,
    }
}

impl Ord for ContextAction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        get_time(&self).partial_cmp(&get_time(&other)).unwrap_or(Equal)
    }
}

impl PartialOrd for ContextAction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ContextAction {
    fn eq(&self, other: &Self) -> bool {
        get_time(&self) == get_time(&other)
    }
}

impl Eq for ContextAction {}
