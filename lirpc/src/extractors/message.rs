use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::{
    connection::Connection,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcMessage, LiRpcResponse, RawLiRpcMessagePayload},
};

pub struct Message<M>(pub M)
where
    M: for<'a> Deserialize<'a>;

impl<S, M> FromConnectionMessage<S> for Message<M>
where
    M: for<'a> Deserialize<'a>,
{
    type Error = LiRpcError;

    fn from_connection_message(
        _connection: &Connection,
        message: &LiRpcMessage,
        _state: &S,
        _output: &Sender<LiRpcResponse>,
    ) -> Result<Self, Self::Error> {
        match &message.payload {
            RawLiRpcMessagePayload::JsonString(json_string) => {
                Ok(Self(serde_json::from_str(json_string)?))
            }
        }
    }
}
