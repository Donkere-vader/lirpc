use std::{pin::Pin, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    handler::Handler,
    lirpc_message::{LiRpcFunctionCall, LiRpcStreamOutput},
    stream_manager::StreamManager,
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
        message: LiRpcFunctionCall,
        state: S,
        output: Sender<LiRpcStreamOutput>,
        stream_manager: StreamManager,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>>;
}

pub(crate) struct HandlerService<F, T, S, C, E>(pub Box<dyn Handler<F, T, S, C, E>>);

impl<F, T, S, C, E> Service<S, C> for HandlerService<F, T, S, C, E>
where
    S: Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    F: 'static,
    T: 'static,
    E: 'static,
{
    fn call(
        &self,
        connection: Arc<ConnectionDetails<C>>,
        message: LiRpcFunctionCall,
        state: S,
        output: Sender<LiRpcStreamOutput>,
        stream_manager: StreamManager,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>> {
        self.0
            .call(connection, message, state, output, stream_manager)
    }
}
