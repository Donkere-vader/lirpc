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
}

impl LiRpcResponseHeaders {
    pub fn new(id: u32) -> Self {
        Self { id }
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

// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
//
// use crate::error::LiRpcError;

// #[derive(Deserialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case")]
// pub enum LiRpcRequest {
//     FunctionCall(LiRpcFunctionCall),
//     CloseStream(LiRpcCloseStream),
// }
//
// #[derive(Deserialize, Debug)]
// pub struct LiRpcFunctionCall {
//     pub headers: LiRpcFunctionCallHeaders,
//     pub payload: Option<RawLiRpcMessagePayload>,
// }
//
// #[derive(Deserialize, Debug)]
// pub struct LiRpcCloseStream {
//     pub stream_id: u32,
// }
//
// #[derive(Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case")]
// pub enum LiRpcResponse {
//     StreamOutput(LiRpcStreamOutput),
// }
//
// #[derive(Serialize, Debug)]
// pub struct LiRpcStreamOutput {
//     pub headers: LiRpcResponseHeaders,
//     pub payload: Option<RawLiRpcMessagePayload>,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// #[serde(untagged)]
// pub enum RawLiRpcMessagePayload {
//     Json(Value),
// }
//
// #[derive(Deserialize, Debug)]
// pub struct LiRpcFunctionCallHeaders {
//     pub id: u32,
//     pub method: String,
// }
//
// #[derive(Serialize, Debug)]
// pub enum LiRpcResponseResult {
//     Ok,
//     Err,
// }
//
// #[derive(Serialize, Debug)]
// pub struct LiRpcResponseHeaders {
//     pub id: u32,
//     pub result: LiRpcResponseResult,
// }
//
// impl TryFrom<Message> for LiRpcRequest {
//     type Error = LiRpcError;
//
//     fn try_from(value: Message) -> Result<Self, Self::Error> {
//         match value {
//             Message::Text(raw_txt) => Ok(serde_json::from_str(&raw_txt)?),
//             _ => Err(LiRpcError::UnableToParseWebsocketMessage),
//         }
//     }
// }
//
// impl TryFrom<LiRpcStreamOutput> for Message {
//     type Error = LiRpcError;
//
//     fn try_from(so: LiRpcStreamOutput) -> Result<Self, Self::Error> {
//         let response = serde_json::to_string(&so)?;
//
//         Ok(Message::Text(Utf8Bytes::from(response)))
//     }
// }
//
