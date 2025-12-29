use std::marker::PhantomData;

use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcMessage, LiRpcResponse, LiRpcResponseHeaders, RawLiRpcMessagePayload},
};

pub struct OutputStream<M>
where
    M: Serialize,
{
    id: u32,
    tx: Sender<LiRpcResponse>,
    _marker: PhantomData<M>,
}

impl<M> OutputStream<M>
where
    M: Serialize,
{
    pub fn new(id: u32, tx: Sender<LiRpcResponse>) -> Self {
        Self {
            id,
            tx,
            _marker: PhantomData,
        }
    }

    pub async fn send(&self, message: M) -> Result<(), LiRpcError> {
        let serialized_message =
            RawLiRpcMessagePayload::JsonString(serde_json::to_string(&message)?);

        self.tx
            .send(LiRpcResponse {
                headers: LiRpcResponseHeaders { id: self.id },
                payload: serialized_message,
            })
            .await?;

        Ok(())
    }
}

impl<M, S, C> FromConnectionMessage<S, C> for OutputStream<M>
where
    M: Serialize,
    C: Clone + Send + Sync + 'static,
{
    type Error = LiRpcError;

    fn from_connection_message(
        _connection: &ConnectionDetails<C>,
        message: &LiRpcMessage,
        _state: &S,
        output: &Sender<LiRpcResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(message.headers.id, output.clone()))
    }
}
