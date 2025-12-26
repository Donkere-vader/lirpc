use std::pin::Pin;

use tokio::sync::mpsc::Sender;

use crate::{
    connection::Connection,
    error::LiRpcError,
    handler::Handler,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub(crate) trait Service<S>
where
    S: Send + Sync + 'static,
    Self: Send + Sync,
{
    fn call(
        &self,
        connection: Connection,
        message: LiRpcMessage,
        state: S,
        output: Sender<LiRpcResponse>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>>;
}

pub(crate) struct HandlerService<F, T, S>(pub Box<dyn Handler<F, T, S>>);

impl<F, T, S> Service<S> for HandlerService<F, T, S>
where
    S: Send + Sync + 'static,
    F: 'static,
    T: 'static,
{
    fn call(
        &self,
        connection: Connection,
        message: LiRpcMessage,
        state: S,
        output: Sender<LiRpcResponse>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>> {
        self.0.call(connection, message, state, output)
    }
}
