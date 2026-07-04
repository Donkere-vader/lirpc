use tokio::sync::watch;

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
    #[error("watch SendError: {0}")]
    WatchSendError(#[from] watch::error::SendError<bool>),
    #[error("Extractor error: {0:?}")]
    ExtractorError(String),
    #[error("Output stream was closed")]
    OutputStreamClosed,
    #[error("Error turning handler error into raw LiRpc response payload: {0}")]
    ErrorTurningHandlerErrorIntoRawLiRpcResponsePayload(String),
    #[error("Error turning extractor error into raw LiRpc response payload: {0}")]
    ErrorTurningExtractorErrorIntoRawLiRpcResponsePayload(String),
}
