use std::marker::PhantomData;

use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{
        LiRpcFunctionCall, LiRpcResponseHeaders, LiRpcResponseResult, LiRpcStreamOutput,
        RawLiRpcMessagePayload,
    },
    stream_manager::StreamManager,
};

pub struct Output<M>
where
    M: Serialize,
{
    id: u32,
    tx: Sender<LiRpcStreamOutput>,
    _marker: PhantomData<M>,
}

impl<M> Output<M>
where
    M: Serialize,
{
    pub fn new(id: u32, tx: Sender<LiRpcStreamOutput>) -> Self {
        Self {
            id,
            tx,
            _marker: PhantomData,
        }
    }

    pub async fn send(self, message: M) -> Result<(), LiRpcError> {
        let serialized_message = RawLiRpcMessagePayload::Json(serde_json::to_value(&message)?);

        self.tx
            .send(LiRpcStreamOutput {
                headers: LiRpcResponseHeaders {
                    id: self.id,
                    result: LiRpcResponseResult::Ok,
                },
                payload: Some(serialized_message),
            })
            .await?;

        Ok(())
    }
}

impl<M, S, C> FromConnectionMessage<S, C> for Output<M>
where
    M: Serialize + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error = LiRpcError;

    async fn from_connection_message(
        _connection: &ConnectionDetails<C>,
        message: &LiRpcFunctionCall,
        _state: &S,
        output: &Sender<LiRpcStreamOutput>,
        _stream_manager: &StreamManager,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(message.headers.id, output.clone()))
    }
}
