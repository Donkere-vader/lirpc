use tokio::sync::{mpsc, watch};

use crate::lirpc_message::{LiRpcStreamOutput, RawLiRpcMessagePayload};

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
    #[error("mpsc SendError: {0}")]
    MpscSendError(#[from] mpsc::error::SendError<LiRpcStreamOutput>),
    #[error("watch SendError: {0}")]
    WatchSendError(#[from] watch::error::SendError<bool>),
    #[error("Extractor error: {0}")]
    ExtractorError(String),
    #[error("Output stream was closed")]
    OutputStreamClosed,
    #[error("Error in handler: {0:?}")]
    ErrorInHandler(RawLiRpcMessagePayload),
}
