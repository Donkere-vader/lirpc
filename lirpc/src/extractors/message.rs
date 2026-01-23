use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcFunctionCall, LiRpcStreamOutput, RawLiRpcMessagePayload},
    stream_manager::StreamManager,
};

pub struct Message<M>(pub M)
where
    M: for<'a> Deserialize<'a>;

impl<S, M, C> FromConnectionMessage<S, C> for Message<M>
where
    M: for<'a> Deserialize<'a> + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error = LiRpcError;

    async fn from_connection_message(
        _connection: &ConnectionDetails<C>,
        message: &LiRpcFunctionCall,
        _state: &S,
        _output: &Sender<LiRpcStreamOutput>,
        _stream_manager: &StreamManager,
    ) -> Result<Self, Self::Error> {
        match &message.payload {
            Some(RawLiRpcMessagePayload::Json(json_value)) => {
                Ok(Self(serde_json::from_value(json_value.clone())?))
            }
            // TODO: probably not very clean to just parse an empty string here
            None => Ok(Self(serde_json::from_str("")?)),
        }
    }
}
