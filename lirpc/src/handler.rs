use std::pin::Pin;

use tokio::sync::mpsc::Sender;

use crate::{
    connection::Connection,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub trait Handler<F, T, S>
where
    Self: Send + Sync + 'static,
{
    fn call(
        &self,
        connection: Connection,
        message: LiRpcMessage,
        state: S,
        output: Sender<LiRpcResponse>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>>;
}

impl<F, T1, T2, T3, S, Fut> Handler<F, (T1, T2, T3), S> for F
where
    F: Fn(T1, T2, T3) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), LiRpcError>> + Send + 'static,
    T1: FromConnectionMessage<S>,
    T2: FromConnectionMessage<S>,
    T3: FromConnectionMessage<S>,
{
    fn call(
        &self,
        connection: Connection,
        message: LiRpcMessage,
        state: S,
        output: Sender<LiRpcResponse>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>> {
        let t1 = T1::from_connection_message(&connection, &message, &state, &output).expect("TODO");
        let t2 = T2::from_connection_message(&connection, &message, &state, &output).expect("TODO");
        let t3 = T3::from_connection_message(&connection, &message, &state, &output).expect("TODO");

        Box::pin(self(t1, t2, t3))
    }
}
