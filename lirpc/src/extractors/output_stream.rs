use std::marker::PhantomData;

use serde::Serialize;
use tokio::sync::{mpsc::Sender, watch::Receiver};

use crate::{
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{
        LiRpcFunctionCall, LiRpcResponseHeaders, LiRpcResponseResult, LiRpcStreamOutput,
        RawLiRpcMessagePayload,
    },
    stream_manager::StreamManager,
};

pub struct OutputStream<M>
where
    M: Serialize,
{
    id: u32,
    tx: Sender<LiRpcStreamOutput>,
    open_stream: Receiver<bool>,
    _marker: PhantomData<M>,
}

impl<M> OutputStream<M>
where
    M: Serialize,
{
    pub fn new(id: u32, open_stream: Receiver<bool>, tx: Sender<LiRpcStreamOutput>) -> Self {
        Self {
            id,
            tx,
            open_stream,
            _marker: PhantomData,
        }
    }

    pub async fn send(&self, message: M) -> Result<(), LiRpcError> {
        let serialized_message = RawLiRpcMessagePayload::Json(serde_json::to_value(&message)?);

        if !*self.open_stream.borrow() {
            return Err(LiRpcError::OutputStreamClosed);
        }

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

impl<M, S, C> FromConnectionMessage<S, C> for OutputStream<M>
where
    M: Serialize + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error = LiRpcError;

    async fn from_connection_message(
        _connection: &crate::connection_details::ConnectionDetails<C>,
        message: &LiRpcFunctionCall,
        _state: &S,
        output: &Sender<LiRpcStreamOutput>,
        stream_manager: &StreamManager,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(
            message.headers.id,
            stream_manager.register_stream(message.headers.id).await,
            output.clone(),
        ))
    }
}
