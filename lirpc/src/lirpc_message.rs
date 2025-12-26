use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

use crate::error::LiRpcError;

pub struct LiRpcMessage {
    pub headers: LiRpcMessageHeaders,
    pub payload: RawLiRpcMessagePayload,
}

pub struct LiRpcResponse {
    pub headers: LiRpcResponseHeaders,
    pub payload: RawLiRpcMessagePayload,
}

pub enum RawLiRpcMessagePayload {
    JsonString(String),
}

#[derive(Deserialize)]
pub struct LiRpcMessageHeaders {
    pub id: u32,
    pub method: String,
}

#[derive(Serialize)]
pub struct LiRpcResponseHeaders {
    pub id: u32,
}

impl TryFrom<Message> for LiRpcMessage {
    type Error = LiRpcError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(raw_txt) => {
                let (headers, payload) = raw_txt
                    .split_once("\n\n")
                    .ok_or(LiRpcError::RawMessageCouldNotBeSplitOnHeaderAndPayload)?;

                Ok(Self {
                    headers: serde_json::from_str(headers)?,
                    payload: RawLiRpcMessagePayload::JsonString(payload.to_string()),
                })
            }
            _ => Err(LiRpcError::UnableToParseWebsocketMessage),
        }
    }
}

impl TryFrom<LiRpcResponse> for Message {
    type Error = LiRpcError;

    fn try_from(value: LiRpcResponse) -> Result<Self, Self::Error> {
        let headers = serde_json::to_string(&value.headers)?;

        match value.payload {
            RawLiRpcMessagePayload::JsonString(raw_json) => Ok(Message::Text(Utf8Bytes::from(
                format!("{headers}\n\n{}", raw_json),
            ))),
        }
    }
}
