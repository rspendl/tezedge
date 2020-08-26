// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

/// Rust implementation of messages required for Rust <-> OCaml FFI communication.

use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::mem::size_of;

use derive_builder::Builder;
use failure::Fail;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use znfe::OCamlError;

use crypto::hash::{BlockHash, ChainId, ContextHash, HashType, OperationHash, ProtocolHash};
use tezos_encoding::{binary_writer, ser};
use tezos_encoding::binary_reader::{BinaryReader, BinaryReaderError};
use tezos_encoding::de::from_value as deserialize_from_value;
use tezos_encoding::encoding::{Encoding, Field, FieldName, HasEncoding, Tag, TagMap, TagVariant};
use tezos_messages::p2p::encoding::prelude::{BlockHeader, Operation, OperationsForBlocksMessage, Path};
use tezos_messages::p2p::encoding::operations_for_blocks::path_encoding;

pub type RustBytes = Vec<u8>;

/// Trait for binary encoding messages for ffi.
pub trait FfiMessage: DeserializeOwned + Serialize + Sized + Send + PartialEq + Debug {
    #[inline]
    fn as_rust_bytes(&self) -> Result<RustBytes, ser::Error> {
        binary_writer::write(&self, Self::encoding())
    }

    /// Create new struct from bytes.
    #[inline]
    fn from_rust_bytes(buf: RustBytes) -> Result<Self, BinaryReaderError> {
        let value = BinaryReader::new().read(buf, Self::encoding())?;
        let value: Self = deserialize_from_value(&value)?;
        Ok(value)
    }

    fn encoding() -> &'static Encoding;
}

/// Genesis block information structure
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GenesisChain {
    pub time: String,
    pub block: String,
    pub protocol: String,
}

/// Voted protocol overrides
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProtocolOverrides {
    pub forced_protocol_upgrades: Vec<(i32, String)>,
    pub voted_protocol_overrides: Vec<(String, String)>,
}

/// Patch_context key json
#[derive(Clone, Serialize, Deserialize)]
pub struct PatchContext {
    pub key: String,
    pub json: String,
}

impl fmt::Debug for PatchContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "key: {}, json: {:?}", &self.key, &self.json)
    }
}

/// Test chain information
#[derive(Debug, Serialize, Deserialize)]
pub struct TestChain {
    pub chain_id: RustBytes,
    pub protocol_hash: RustBytes,
    pub expiration_date: String,
}

/// Holds configuration for ocaml runtime - e.g. arguments which are passed to ocaml and can be change in runtime
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TezosRuntimeConfiguration {
    pub log_enabled: bool,
    pub no_of_ffi_calls_treshold_for_gc: i32,
    pub debug_mode: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq)]
pub struct ApplyBlockRequest {
    pub chain_id: ChainId,
    pub block_header: BlockHeader,
    pub pred_header: BlockHeader,
    pub max_operations_ttl: i32,
    pub operations: Vec<Vec<Operation>>,
}

impl ApplyBlockRequest {
    pub fn convert_operations(block_operations: Vec<OperationsForBlocksMessage>) -> Vec<Vec<Operation>> {
        block_operations
            .into_iter()
            .map(|ops| ops.operations)
            .collect()
    }
}

lazy_static! {
    pub static ref APPLY_BLOCK_REQUEST_ENCODING: Encoding = Encoding::Obj(vec![
        Field::new(FieldName::ChainID, Encoding::Hash(HashType::ChainId)),
        Field::new(FieldName::BlockHeader, Encoding::dynamic(BlockHeader::encoding().clone())),
        Field::new(FieldName::PredHeader, Encoding::dynamic(BlockHeader::encoding().clone())),
        Field::new(FieldName::MaxOperationsTTL, Encoding::Int31),
        Field::new(FieldName::Operations, Encoding::dynamic(Encoding::list(Encoding::dynamic(Encoding::list(Encoding::dynamic(Operation::encoding().clone())))))),
    ]);
}

impl FfiMessage for ApplyBlockRequest {
    fn encoding() -> &'static Encoding {
        &APPLY_BLOCK_REQUEST_ENCODING
    }
}

/// Application block result
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ApplyBlockResponse {
    pub validation_result_message: String,
    pub context_hash: ContextHash,
    pub block_header_proto_json: String,
    pub block_header_proto_metadata_json: String,
    pub operations_proto_metadata_json: String,
    pub max_operations_ttl: i32,
    pub last_allowed_fork_level: i32,
    pub forking_testchain: bool,
    pub forking_testchain_data: Option<ForkingTestchainData>,
}

lazy_static! {
    pub static ref FORKING_TESTCHAIN_DATA_ENCODING: Encoding = Encoding::Obj(vec![
        Field::new(FieldName::ForkingBlockHash, Encoding::Hash(HashType::BlockHash)),
        Field::new(FieldName::TestChainID, Encoding::Hash(HashType::ChainId)),
    ]);

    pub static ref APPLY_BLOCK_RESPONSE_ENCODING: Encoding = Encoding::Obj(vec![
        Field::new(FieldName::ValidationResultMessage, Encoding::String),
        Field::new(FieldName::ContextHash, Encoding::Hash(HashType::ContextHash)),
        Field::new(FieldName::BlockHeaderProtoJson, Encoding::String),
        Field::new(FieldName::BlockHeaderProtoMetadataJson, Encoding::String),
        Field::new(FieldName::OperationsProtoMetadataJson, Encoding::String),
        Field::new(FieldName::MaxOperationsTTL, Encoding::Int31),
        Field::new(FieldName::LastAllowedForkLevel, Encoding::Int32),
        Field::new(FieldName::ForkingTestchain, Encoding::Bool),
        Field::new(FieldName::ForkingTestchainData, Encoding::option(FORKING_TESTCHAIN_DATA_ENCODING.clone())),
    ]);
}

impl FfiMessage for ApplyBlockResponse {
    fn encoding() -> &'static Encoding {
        &APPLY_BLOCK_RESPONSE_ENCODING
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct PrevalidatorWrapper {
    pub chain_id: ChainId,
    pub protocol: ProtocolHash,
}

impl fmt::Debug for PrevalidatorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chain_hash_encoding = HashType::ChainId;
        let protocol_hash_encoding = HashType::ProtocolHash;
        write!(f, "PrevalidatorWrapper[chain_id: {}, protocol: {}]",
               chain_hash_encoding.bytes_to_string(&self.chain_id),
               protocol_hash_encoding.bytes_to_string(&self.protocol)
        )
    }
}

lazy_static! {
    pub static ref PREVALIDATOR_WRAPPER_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::ChainID, Encoding::Hash(HashType::ChainId)),
            Field::new(FieldName::Protocol, Encoding::Hash(HashType::ProtocolHash)),
    ]);
}

impl FfiMessage for PrevalidatorWrapper {
    fn encoding() -> &'static Encoding {
        &PREVALIDATOR_WRAPPER_ENCODING
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq)]
pub struct BeginConstructionRequest {
    pub chain_id: ChainId,
    pub predecessor: BlockHeader,
    pub protocol_data: Option<Vec<u8>>,
}

lazy_static! {
    pub static ref BEGIN_CONSTRUCTION_REQUEST_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::ChainID, Encoding::Hash(HashType::ChainId)),
            Field::new(FieldName::Predecessor, Encoding::dynamic(BlockHeader::encoding().clone())),
            Field::new(FieldName::ProtocolData, Encoding::option(Encoding::list(Encoding::Uint8))),
    ]);
}

impl FfiMessage for BeginConstructionRequest {
    fn encoding() -> &'static Encoding {
        &BEGIN_CONSTRUCTION_REQUEST_ENCODING
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq)]
pub struct ValidateOperationRequest {
    pub prevalidator: PrevalidatorWrapper,
    pub operation: Operation,
}

lazy_static! {
    pub static ref VALIDATE_OPERATION_REQUEST_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::Prevalidator, PREVALIDATOR_WRAPPER_ENCODING.clone()),
            Field::new(FieldName::Operation, Encoding::dynamic(Operation::encoding().clone())),
    ]);
}

impl FfiMessage for ValidateOperationRequest {
    fn encoding() -> &'static Encoding {
        &VALIDATE_OPERATION_REQUEST_ENCODING
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq)]
pub struct ValidateOperationResponse {
    pub prevalidator: PrevalidatorWrapper,
    pub result: ValidateOperationResult,
}

lazy_static! {
    pub static ref VALIDATE_OPERATION_RESPONSE_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::Prevalidator, PREVALIDATOR_WRAPPER_ENCODING.clone()),
            Field::new(FieldName::Result, VALIDATE_OPERATION_RESULT_ENCODING.clone()),
    ]);
}

impl FfiMessage for ValidateOperationResponse {
    fn encoding() -> &'static Encoding {
        &VALIDATE_OPERATION_RESPONSE_ENCODING
    }
}

pub type OperationProtocolDataJson = String;
pub type ErrorListJson = String;

#[derive(Serialize, Deserialize, Clone, Builder, PartialEq)]
pub struct OperationProtocolDataJsonWithErrorListJson {
    pub protocol_data_json: OperationProtocolDataJson,
    pub error_json: ErrorListJson,
}

impl fmt::Debug for OperationProtocolDataJsonWithErrorListJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[error_json: {}, protocol_data_json: {}]",
               format_json_single_line(&self.error_json),
               format_json_single_line(&self.protocol_data_json)
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Builder, PartialEq)]
pub struct Applied {
    pub hash: OperationHash,
    pub protocol_data_json: OperationProtocolDataJson,
}

#[inline]
fn format_json_single_line(origin: &str) -> String {
    let json = serde_json::json!(origin);
    serde_json::to_string(&json).unwrap_or_else(|_| origin.to_string())
}

impl fmt::Debug for Applied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operation_hash_encoding = HashType::OperationHash;
        write!(f, "[hash: {}, protocol_data_json: {}]",
               operation_hash_encoding.bytes_to_string(&self.hash),
               format_json_single_line(&self.protocol_data_json)
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Builder, PartialEq)]
pub struct Errored {
    pub hash: OperationHash,
    pub is_endorsement: Option<bool>,
    pub protocol_data_json_with_error_json: OperationProtocolDataJsonWithErrorListJson,
}

impl fmt::Debug for Errored {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let operation_hash_encoding = HashType::OperationHash;
        write!(f, "[hash: {}, protocol_data_json_with_error_json: {:?}]",
               operation_hash_encoding.bytes_to_string(&self.hash),
               &self.protocol_data_json_with_error_json
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq, Default)]
pub struct ValidateOperationResult {
    pub applied: Vec<Applied>,
    pub refused: Vec<Errored>,
    pub branch_refused: Vec<Errored>,
    pub branch_delayed: Vec<Errored>,
    // TODO: outedate?
}

impl ValidateOperationResult {
    /// Merges result with new one, and returns `true/false` if something was changed
    pub fn merge(&mut self, new_result: &ValidateOperationResult) -> bool {
        let mut changed = self.merge_applied(&new_result.applied);
        changed |= self.merge_refused(&new_result.refused);
        changed |= self.merge_branch_refused(&new_result.branch_refused);
        changed |= self.merge_branch_delayed(&new_result.branch_delayed);
        changed
    }

    fn merge_applied(&mut self, new_items: &[Applied]) -> bool {
        let mut changed = false;
        let mut added = false;
        let mut m = HashMap::new();

        for a in &self.applied {
            m.insert(a.hash.clone(), a.clone());
        }
        for na in new_items {
            match m.insert(na.hash.clone(), na.clone()) {
                Some(_) => changed |= true,
                None => added |= true,
            };
        }

        if added || changed {
            self.applied = m.values().cloned().collect();
        }
        added || changed
    }

    fn merge_refused(&mut self, new_items: &[Errored]) -> bool {
        Self::merge_errored(&mut self.refused, new_items)
    }

    fn merge_branch_refused(&mut self, new_items: &[Errored]) -> bool {
        Self::merge_errored(&mut self.branch_refused, new_items)
    }

    fn merge_branch_delayed(&mut self, new_items: &[Errored]) -> bool {
        Self::merge_errored(&mut self.branch_delayed, new_items)
    }

    fn merge_errored(old_items: &mut Vec<Errored>, new_items: &[Errored]) -> bool {
        let mut changed = false;
        let mut added = false;
        let mut m = HashMap::new();

        for a in old_items.iter_mut() {
            m.insert(a.hash.clone(), (*a).clone());
        }
        for na in new_items {
            match m.insert(na.hash.clone(), na.clone()) {
                Some(_) => changed |= true,
                None => added |= true,
            };
        }

        if added || changed {
            *old_items = m.values().cloned().collect();
        }
        added || changed
    }
}

lazy_static! {
    static ref OPERATION_DATA_ERROR_JSON_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::ProtocolDataJson, Encoding::String),
            Field::new(FieldName::ErrorJson, Encoding::String),
    ]);

    pub static ref VALIDATE_OPERATION_RESULT_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::Applied, Encoding::dynamic(Encoding::list(
                    Encoding::Obj(
                        vec![
                            Field::new(FieldName::Hash, Encoding::Hash(HashType::OperationHash)),
                            Field::new(FieldName::ProtocolDataJson, Encoding::String),
                        ]
                    )
                ))
            ),
            Field::new(FieldName::Refused, Encoding::dynamic(Encoding::list(
                    Encoding::Obj(
                        vec![
                            Field::new(FieldName::Hash, Encoding::Hash(HashType::OperationHash)),
                            Field::new(FieldName::IsEndorsement, Encoding::option(Encoding::Bool)),
                            Field::new(FieldName::ProtocolDataJsonWithErrorJson, OPERATION_DATA_ERROR_JSON_ENCODING.clone()),
                        ]
                    )
                ))
            ),
            Field::new(FieldName::BranchRefused, Encoding::dynamic(Encoding::list(
                    Encoding::Obj(
                        vec![
                            Field::new(FieldName::Hash, Encoding::Hash(HashType::OperationHash)),
                            Field::new(FieldName::IsEndorsement, Encoding::option(Encoding::Bool)),
                            Field::new(FieldName::ProtocolDataJsonWithErrorJson, OPERATION_DATA_ERROR_JSON_ENCODING.clone()),
                        ]
                    )
                ))
            ),
            Field::new(FieldName::BranchDelayed, Encoding::dynamic(Encoding::list(
                    Encoding::Obj(
                        vec![
                            Field::new(FieldName::Hash, Encoding::Hash(HashType::OperationHash)),
                            Field::new(FieldName::IsEndorsement, Encoding::option(Encoding::Bool)),
                            Field::new(FieldName::ProtocolDataJsonWithErrorJson, OPERATION_DATA_ERROR_JSON_ENCODING.clone()),
                        ]
                    )
                ))
            ),
    ]);
}

/// Init protocol context result
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct InitProtocolContextResult {
    pub supported_protocol_hashes: Vec<ProtocolHash>,
    /// Presents only if was genesis commited to context
    pub genesis_commit_hash: Option<ContextHash>,
}

impl fmt::Debug for InitProtocolContextResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let genesis_commit_hash = match &self.genesis_commit_hash {
            Some(hash) => HashType::ContextHash.bytes_to_string(hash),
            None => "-none-".to_string()
        };
        let supported_protocol_hashes = self.supported_protocol_hashes
            .iter()
            .map(|ph| HashType::ProtocolHash.bytes_to_string(ph))
            .collect::<Vec<String>>();
        write!(f, "genesis_commit_hash: {}, supported_protocol_hashes: {:?}", &genesis_commit_hash, &supported_protocol_hashes)
    }
}

/// Commit genesis result
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CommitGenesisResult {
    pub block_header_proto_json: String,
    pub block_header_proto_metadata_json: String,
    pub operations_proto_metadata_json: String,
}

/// Forking test chain data
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ForkingTestchainData {
    pub forking_block_hash: BlockHash,
    pub test_chain_id: ChainId,
}

#[derive(Serialize, Deserialize, Debug, Fail, PartialEq)]
pub enum CallError {
    #[fail(display = "Failed to call - message: {:?}!", parsed_error_message)]
    FailedToCall {
        parsed_error_message: Option<String>,
    },
    #[fail(display = "Invalid request data - message: {}!", message)]
    InvalidRequestData {
        message: String,
    },
    #[fail(display = "Invalid response data - message: {}!", message)]
    InvalidResponseData {
        message: String,
    },
}

impl From<OCamlError> for CallError {
    fn from(error: OCamlError) -> Self {
        match error {
            OCamlError::Exception(exception) => {
                match exception.message() {
                    None => CallError::FailedToCall {
                        parsed_error_message: None
                    },
                    Some(message) => {
                        CallError::FailedToCall {
                            parsed_error_message: Some(message)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Fail)]
pub enum TezosRuntimeConfigurationError {
    #[fail(display = "Change ocaml settings failed, message: {}!", message)]
    ChangeConfigurationError {
        message: String
    }
}

impl From<OCamlError> for TezosRuntimeConfigurationError {
    fn from(error: OCamlError) -> Self {
        match error {
            OCamlError::Exception(exception) => {
                TezosRuntimeConfigurationError::ChangeConfigurationError {
                    message: exception.message().unwrap_or_else(|| "unknown".to_string())
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Fail)]
pub enum TezosGenerateIdentityError {
    #[fail(display = "Generate identity failed, message: {}!", message)]
    GenerationError {
        message: String
    },
    #[fail(display = "Generated identity is invalid json! message: {}!", message)]
    InvalidJsonError {
        message: String
    },
}

impl From<OCamlError> for TezosGenerateIdentityError {
    fn from(error: OCamlError) -> Self {
        match error {
            OCamlError::Exception(exception) => {
                TezosGenerateIdentityError::GenerationError {
                    message: exception.message().unwrap_or_else(|| "unknown".to_string())
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Fail)]
pub enum TezosStorageInitError {
    #[fail(display = "Ocaml storage init failed, message: {}!", message)]
    InitializeError {
        message: String
    }
}

impl From<OCamlError> for TezosStorageInitError {
    fn from(error: OCamlError) -> Self {
        match error {
            OCamlError::Exception(exception) => {
                TezosStorageInitError::InitializeError {
                    message: exception.message().unwrap_or_else(|| "unknown".to_string())
                }
            }
        }
    }
}

impl slog::Value for TezosStorageInitError {
    fn serialize(&self, _record: &slog::Record, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        serializer.emit_arguments(key, &format_args!("{}", self))
    }
}

#[derive(Serialize, Deserialize, Debug, Fail)]
pub enum GetDataError {
    #[fail(display = "Ocaml failed to get data, message: {}!", message)]
    ReadError {
        message: String
    }
}

impl From<OCamlError> for GetDataError {
    fn from(error: OCamlError) -> Self {
        match error {
            OCamlError::Exception(exception) => {
                GetDataError::ReadError {
                    message: exception.message().unwrap_or_else(|| "unknown".to_string())
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Fail, PartialEq)]
pub enum ApplyBlockError {
    #[fail(display = "Incomplete operations, exptected: {}, has actual: {}!", expected, actual)]
    IncompleteOperations {
        expected: usize,
        actual: usize,
    },
    #[fail(display = "Failed to apply block - message: {}!", message)]
    FailedToApplyBlock {
        message: String,
    },
    #[fail(display = "Unknown predecessor context - try to apply predecessor at first message: {}!", message)]
    UnknownPredecessorContext {
        message: String,
    },
    #[fail(display = "Predecessor does not match - message: {}!", message)]
    PredecessorMismatch {
        message: String,
    },
    #[fail(display = "Invalid request/response data - message: {}!", message)]
    InvalidRequestResponseData {
        message: String,
    },
}

impl From<CallError> for ApplyBlockError {
    fn from(error: CallError) -> Self {
        match error {
            CallError::FailedToCall { parsed_error_message } => {
                match parsed_error_message {
                    None => ApplyBlockError::FailedToApplyBlock {
                        message: "unknown".to_string()
                    },
                    Some(message) => {
                        match message.as_str() {
                            e if e.starts_with("Unknown_predecessor_context") => ApplyBlockError::UnknownPredecessorContext {
                                message: message.to_string()
                            },
                            e if e.starts_with("Predecessor_mismatch") => ApplyBlockError::PredecessorMismatch {
                                message: message.to_string()
                            },
                            message => ApplyBlockError::FailedToApplyBlock {
                                message: message.to_string()
                            }
                        }
                    }
                }
            }
            CallError::InvalidRequestData { message } => ApplyBlockError::InvalidRequestResponseData {
                message
            },
            CallError::InvalidResponseData { message } => ApplyBlockError::InvalidRequestResponseData {
                message
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Fail, PartialEq)]
pub enum BeginConstructionError {
    #[fail(display = "Failed to begin construction - message: {}!", message)]
    FailedToBeginConstruction {
        message: String,
    },
    #[fail(display = "Unknown predecessor context - try to apply predecessor at first message: {}!", message)]
    UnknownPredecessorContext {
        message: String,
    },
    #[fail(display = "Invalid request/response data - message: {}!", message)]
    InvalidRequestResponseData {
        message: String,
    },
}

impl From<CallError> for BeginConstructionError {
    fn from(error: CallError) -> Self {
        match error {
            CallError::FailedToCall { parsed_error_message } => {
                match parsed_error_message {
                    None => BeginConstructionError::FailedToBeginConstruction {
                        message: "unknown".to_string()
                    },
                    Some(message) => {
                        match message.as_str() {
                            e if e.starts_with("Unknown_predecessor_context") => BeginConstructionError::UnknownPredecessorContext {
                                message: message.to_string()
                            },
                            message => BeginConstructionError::FailedToBeginConstruction {
                                message: message.to_string()
                            }
                        }
                    }
                }
            }
            CallError::InvalidRequestData { message } => BeginConstructionError::InvalidRequestResponseData {
                message
            },
            CallError::InvalidResponseData { message } => BeginConstructionError::InvalidRequestResponseData {
                message
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Fail, PartialEq)]
pub enum ValidateOperationError {
    #[fail(display = "Failed to validate operation - message: {}!", message)]
    FailedToValidateOperation {
        message: String,
    },
    #[fail(display = "Invalid request/response data - message: {}!", message)]
    InvalidRequestResponseData {
        message: String,
    },
}

impl From<CallError> for ValidateOperationError {
    fn from(error: CallError) -> Self {
        match error {
            CallError::FailedToCall { parsed_error_message } => {
                match parsed_error_message {
                    None => ValidateOperationError::FailedToValidateOperation {
                        message: "unknown".to_string()
                    },
                    Some(message) => {
                        ValidateOperationError::FailedToValidateOperation {
                            message
                        }
                    }
                }
            }
            CallError::InvalidRequestData { message } => ValidateOperationError::InvalidRequestResponseData {
                message
            },
            CallError::InvalidResponseData { message } => ValidateOperationError::InvalidRequestResponseData {
                message
            },
        }
    }
}

#[derive(Debug, Fail)]
pub enum BlockHeaderError {
    #[fail(display = "BlockHeader cannot be read from storage: {}!", message)]
    ReadError {
        message: String
    },
    #[fail(display = "BlockHeader was expected, but was not found!")]
    ExpectedButNotFound,
}

impl From<OCamlError> for BlockHeaderError {
    fn from(error: OCamlError) -> Self {
        match error {
            OCamlError::Exception(exception) => {
                BlockHeaderError::ReadError {
                    message: exception.message().unwrap_or_else(|| "unknown".to_string())
                }
            }
        }
    }
}

#[derive(Debug, Fail)]
pub enum ContextDataError {
    #[fail(display = "Resolve/decode context data failed to decode: {}!", message)]
    DecodeError {
        message: String
    },
}

impl From<OCamlError> for ContextDataError {
    fn from(error: OCamlError) -> Self {
        match error {
            OCamlError::Exception(exception) => {
                ContextDataError::DecodeError {
                    message: exception.message().unwrap_or_else(|| "unknown".to_string())
                }
            }
        }
    }
}

pub type Json = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JsonRpcRequest {
    pub body: Json,
    pub context_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JsonRpcResponse {
    pub body: Json
}

lazy_static! {
    pub static ref JSON_RPC_REQUEST_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::Body, Encoding::String),
            Field::new(FieldName::ContextPath, Encoding::String),
    ]);

    pub static ref JSON_RPC_RESPONSE_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::Body, Encoding::String),
    ]);
}

impl FfiMessage for JsonRpcResponse {
    fn encoding() -> &'static Encoding {
        &JSON_RPC_RESPONSE_ENCODING
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq)]
pub struct ProtocolJsonRpcRequest {
    pub block_header: BlockHeader,
    pub chain_arg: String,
    pub chain_id: ChainId,

    pub request: JsonRpcRequest,

    // TODO: TE-140 - will be removed, when router is done
    pub ffi_service: FfiRpcService,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FfiRpcService {
    HelpersRunOperation,
    HelpersPreapplyOperations,
    HelpersPreapplyBlock,
    HelpersCurrentLevel,
    DelegatesMinimalValidTime,
    LiveBlocks,
}

lazy_static! {
    pub static ref PROTOCOL_JSON_RPC_REQUEST_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::BlockHeader, Encoding::dynamic(BlockHeader::encoding().clone())),
            Field::new(FieldName::ChainArg, Encoding::String),
            Field::new(FieldName::ChainID, Encoding::Hash(HashType::ChainId)),
            Field::new(FieldName::Request, JSON_RPC_REQUEST_ENCODING.clone()),
            Field::new(FieldName::FFIService, Encoding::Tags(
                    size_of::<u16>(),
                    TagMap::new(&[
                        Tag::new(0, TagVariant::HelpersRunOperation, Encoding::Unit),
                        Tag::new(1, TagVariant::HelpersPreapplyOperations, Encoding::Unit),
                        Tag::new(2, TagVariant::HelpersPreapplyBlock, Encoding::Unit),
                        Tag::new(3, TagVariant::HelpersCurrentLevel, Encoding::Unit),
                        Tag::new(4, TagVariant::DelegatesMinimalValidTime, Encoding::Unit),
                        Tag::new(5, TagVariant::LiveBlocks, Encoding::Unit),
                    ]),
                )
            ),
    ]);
}

impl FfiMessage for ProtocolJsonRpcRequest {
    fn encoding() -> &'static Encoding {
        &PROTOCOL_JSON_RPC_REQUEST_ENCODING
    }
}

#[derive(Serialize, Deserialize, Debug, Fail, PartialEq)]
pub enum ProtocolRpcError {
    #[fail(display = "Failed to call protocol rpc - message: {}!", message)]
    FailedToCallProtocolRpc {
        message: String,
    },
    #[fail(display = "Invalid request data - message: {}!", message)]
    InvalidRequestData {
        message: String,
    },
    #[fail(display = "Invalid response data - message: {}!", message)]
    InvalidResponseData {
        message: String,
    },
}

impl From<CallError> for ProtocolRpcError {
    fn from(error: CallError) -> Self {
        match error {
            CallError::FailedToCall { parsed_error_message } => {
                match parsed_error_message {
                    None => ProtocolRpcError::FailedToCallProtocolRpc {
                        message: "unknown".to_string()
                    },
                    Some(message) => {
                        ProtocolRpcError::FailedToCallProtocolRpc {
                            message
                        }
                    }
                }
            }
            CallError::InvalidRequestData { message } => ProtocolRpcError::InvalidRequestData {
                message
            },
            CallError::InvalidResponseData { message } => ProtocolRpcError::InvalidResponseData {
                message
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ComputePathRequest {
    pub operations: Vec<Vec<OperationHash>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ComputePathResponse {
    pub operations_hashes_path: Vec<Path>,
}

lazy_static! {
    pub static ref COMPUTE_PATH_REQUEST_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::Operations, Encoding::dynamic(Encoding::list(Encoding::dynamic(Encoding::list(Encoding::Hash(HashType::OperationHash)))))),
    ]);
}

lazy_static! {
    pub static ref COMPUTE_PATH_RESPONSE_ENCODING: Encoding = Encoding::Obj(vec![
            Field::new(FieldName::OperationHashesPath, Encoding::dynamic(Encoding::list(path_encoding()))),
    ]);
}

impl FfiMessage for ComputePathRequest {
    fn encoding() -> &'static Encoding {
        &COMPUTE_PATH_REQUEST_ENCODING
    }
}

impl FfiMessage for ComputePathResponse {
    fn encoding() -> &'static Encoding {
        &COMPUTE_PATH_RESPONSE_ENCODING
    }
}

#[derive(Serialize, Deserialize, Debug, Fail)]
pub enum ComputePathError {
    #[fail(display = "Path computation failed, message: {}!", message)]
    PathError {
        message: String
    },
    #[fail(display = "Path computation failed, message: {}!", message)]
    InvalidRequestResponseData {
        message: String
    },
}

impl From<CallError> for ComputePathError {
    fn from(error: CallError) -> Self {
        match error {
            CallError::FailedToCall { parsed_error_message } => {
                match parsed_error_message {
                    None => ComputePathError::PathError {
                        message: "unknown".to_string()
                    },
                    Some(message) => {
                        ComputePathError::PathError {
                            message: message.to_string()
                        }
                    }
                }
            }
            CallError::InvalidRequestData { message } => ComputePathError::InvalidRequestResponseData {
                message
            },
            CallError::InvalidResponseData { message } => ComputePathError::InvalidRequestResponseData {
                message
            },
        }
    }
}