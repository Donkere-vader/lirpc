use std::{pin::Pin, sync::Arc};

use crate::{
    connection_details::ConnectionDetails,
    handler::Handler,
    lirpc_message::{LiRpcRequest, LiRpcResponse},
    translatable::Translatable,
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
        message: LiRpcRequest,
        state: S,
    ) -> Pin<Box<dyn Future<Output = LiRpcResponse> + Send>>;
}

pub(crate) struct HandlerService<F, T, S, C, E>(pub Box<dyn Handler<F, T, S, C, E>>);

impl<F, T, S, C, R> Service<S, C> for HandlerService<F, T, S, C, R>
where
    S: Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    F: 'static,
    T: 'static,
    R: Translatable + 'static,
{
    fn call(
        &self,
        connection: Arc<ConnectionDetails<C>>,
        message: LiRpcRequest,
        state: S,
    ) -> Pin<Box<dyn Future<Output = LiRpcResponse> + Send>> {
        Box::pin(self.0.call(connection, message, state))
    }
}
