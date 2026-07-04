use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

use crate::error::LiRpcError;

#[derive(Debug, Deserialize)]
pub struct LiRpcRequest {
    pub headers: LiRpcRequestHeaders,
    pub payload: Option<LiRpcPayload>,
}

#[derive(Debug, Deserialize)]
pub struct LiRpcRequestHeaders {
    pub id: u32,
    pub function: String,
}

#[derive(Debug, Serialize)]
pub struct LiRpcResponse {
    pub headers: LiRpcResponseHeaders,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<LiRpcPayload>,
}

impl LiRpcResponse {
    pub fn new(headers: LiRpcResponseHeaders, payload: Option<LiRpcPayload>) -> Self {
        Self { headers, payload }
    }
}

#[derive(Debug, Serialize)]
pub struct LiRpcResponseHeaders {
    pub id: u32,
    #[serde(skip_serializing_if = "LiRpcResponseResultHeader::is_ok")]
    pub res: LiRpcResponseResultHeader,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LiRpcResponseResultHeader {
    Ok,
    Err,
}

impl LiRpcResponseResultHeader {
    pub fn is_ok(&self) -> bool {
        matches!(self, LiRpcResponseResultHeader::Ok)
    }
}

impl LiRpcResponseHeaders {
    pub fn new(id: u32, res: LiRpcResponseResultHeader) -> Self {
        Self { id, res }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiRpcPayload(pub Value);

impl LiRpcPayload {
    pub fn new(value: Value) -> Self {
        Self(value)
    }
}

impl TryFrom<Message> for LiRpcRequest {
    type Error = LiRpcError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(raw_txt) => Ok(serde_json::from_str(&raw_txt)?),
            _ => Err(LiRpcError::UnableToParseWebsocketMessage),
        }
    }
}

impl TryFrom<LiRpcResponse> for Message {
    type Error = LiRpcError;

    fn try_from(so: LiRpcResponse) -> Result<Self, Self::Error> {
        let response = serde_json::to_string(&so)?;

        Ok(Message::Text(Utf8Bytes::from(response)))
    }
}
