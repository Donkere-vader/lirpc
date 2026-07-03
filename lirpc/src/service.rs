use std::{pin::Pin, sync::Arc};

use crate::{
    connection_details::ConnectionDetails,
    handler::Handler,
    lirpc_message::{LiRpcPayload, LiRpcRequest, LiRpcResponse, LiRpcResponseHeaders},
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
        let message_id = message.headers.id;
        let future_call_result = self.0.call(connection, message, state);

        Box::pin(async move {
            let call_result = future_call_result.await;

            // TODO do conversion from LiRpcType to LiRpcResponse here?
            LiRpcResponse::new(
                LiRpcResponseHeaders::new(message_id),
                Some(LiRpcPayload::new(
                    serde_json::to_value(call_result).expect("TODO"),
                )),
            )
        })
    }
}
