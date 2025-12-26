use tokio::sync::mpsc;

use crate::lirpc_message::LiRpcResponse;

#[derive(Debug, thiserror::Error)]
pub enum LiRpcError {
    #[error("Error deserializing: {0}")]
    DeserializeError(#[from] serde_json::Error),
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Unable to parse websocket message type that is not `Text`")]
    UnableToParseWebsocketMessage,
    #[error("Raw message could not be split into headers and payload")]
    RawMessageCouldNotBeSplitOnHeaderAndPayload,
    #[error("Method {0} not found")]
    HandlerNotFound(String),
    #[error("SendError: {0}")]
    SendError(#[from] mpsc::error::SendError<LiRpcResponse>),
}
