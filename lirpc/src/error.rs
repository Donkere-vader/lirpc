use serde_json::json;
use tokio::sync::{mpsc, watch};

use crate::lirpc_message::{
    IntoRawLiRpcResponsePayload, LiRpcStreamOutput, RawLiRpcMessagePayload,
};

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
    #[error("Extractor error: {0:?}")]
    ExtractorError(RawLiRpcMessagePayload),
    #[error("Output stream was closed")]
    OutputStreamClosed,
    #[error("Error in handler: {0:?}")]
    ErrorInHandler(RawLiRpcMessagePayload),
    #[error("Error turning handler error into raw LiRpc response payload: {0}")]
    ErrorTurningHandlerErrorIntoRawLiRpcResponsePayload(String),
    #[error("Error turning extractor error into raw LiRpc response payload: {0}")]
    ErrorTurningExtractorErrorIntoRawLiRpcResponsePayload(String),
}

impl IntoRawLiRpcResponsePayload for LiRpcError {
    fn into(&self) -> RawLiRpcMessagePayload {
        RawLiRpcMessagePayload::Json(match self {
            LiRpcError::DeserializeError(_) => json!({"error": "deserialize_error"}),
            LiRpcError::IoError(_) => json!({"error": "io_error"}),
            LiRpcError::UnableToParseWebsocketMessage => {
                json!({"error": "unable_to_parse_websocket_message"})
            }
            LiRpcError::RawMessageCouldNotBeSplitOnHeaderAndPayload => {
                json!({"error": "raw_message_could_not_be_split_on_header_and_payload"})
            }
            LiRpcError::HandlerNotFound(_) => json!({"error": "handler_not_found"}),
            LiRpcError::MpscSendError(_) => json!({"error": "mpsc_send_error"}),
            LiRpcError::WatchSendError(_) => json!({"error": "watch_send_error"}),
            LiRpcError::ExtractorError(_) => json!({"error": "extractor_error"}),
            LiRpcError::OutputStreamClosed => json!({"error": "output_stream_closed"}),
            LiRpcError::ErrorInHandler(_) => json!({"error": "error_in_handler"}),
            LiRpcError::ErrorTurningHandlerErrorIntoRawLiRpcResponsePayload(_) => {
                json!({"error": "error_turning_handler_error_into_raw_lirpc_response_payload"})
            }
            LiRpcError::ErrorTurningExtractorErrorIntoRawLiRpcResponsePayload(_) => {
                json!({"error": "error_turning_extractor_error_into_raw_lirpc_response_payload"})
            }
        })
    }
}
