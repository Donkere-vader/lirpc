use std::{pin::Pin, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    handler::Handler,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub(crate) trait Service<S, C>
where
    S: Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    Self: Send + Sync,
{
    fn call(
        &self,
        connection: Arc<ConnectionDetails<C>>,
        message: LiRpcMessage,
        state: S,
        output: Sender<LiRpcResponse>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>>;
}

pub(crate) struct HandlerService<F, T, S, C>(pub Box<dyn Handler<F, T, S, C>>);

impl<F, T, S, C> Service<S, C> for HandlerService<F, T, S, C>
where
    S: Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    F: 'static,
    T: 'static,
{
    fn call(
        &self,
        connection: Arc<ConnectionDetails<C>>,
        message: LiRpcMessage,
        state: S,
        output: Sender<LiRpcResponse>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>> {
        self.0.call(connection, message, state, output)
    }
}
