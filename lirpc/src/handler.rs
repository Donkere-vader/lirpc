use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::json;
use tracing::error;

use crate::{
    api_spec::LiRpcMethodSpec,
    connection_details::ConnectionDetails,
    extractors::FromConnectionMessage,
    lirpc_message::{
        LiRpcPayload, LiRpcRequest, LiRpcResponse, LiRpcResponseHeaders, LiRpcResponseResultHeader,
    },
    translatable::{Translatable, Type},
};

pub trait Handler<F, T, S, C, R>
where
    Self: Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    fn call(
        &self,
        connection: Arc<ConnectionDetails<C>>,
        message: LiRpcRequest,
        state: S,
    ) -> Pin<Box<dyn Future<Output = LiRpcResponse> + Send>>;

    fn get_spec(&self) -> LiRpcMethodSpec;
}

fn build_lirpc_response(message_id: u32, is_ok: bool, payload: impl Translatable) -> LiRpcResponse {
    let serialized_payload = serde_json::to_value(payload);

    let headers = LiRpcResponseHeaders::new(
        message_id,
        match (is_ok, &serialized_payload) {
            // An error occurred in the handler
            (false, _) => LiRpcResponseResultHeader::Err,
            // An error occurred when serializing the response
            (true, Err(_)) => LiRpcResponseResultHeader::Err,
            // both things went Ok
            (true, Ok(_)) => LiRpcResponseResultHeader::Ok,
        },
    );

    LiRpcResponse::new(
        headers,
        Some(LiRpcPayload::new(match serialized_payload {
            Ok(p) => p,
            Err(e) => {
                error!("error serializing response: {e}");
                json!({
                    "error": "server_error",
                    "detail": "an error occurred on the server which it was unable to recover from"
                })
            }
        })),
    )
}

macro_rules! try_extract {
    ($Ty:ty, $connection:expr, $message:expr, $state:expr) => {{
        match <$Ty as FromConnectionMessage<_, _>>::from_connection_message(
            &$connection,
            &$message,
            &$state,
        )
        .await
        {
            Ok(value) => value,
            Err(e) => {
                // Format the error into a Send-safe String outside the async block,
                // then early-return a pinned async error future.
                // return Err(LiRpcError::ExtractorError(e));
                return build_lirpc_response($message.headers.id, false, e);
            }
        }
    }};
}

macro_rules! impl_handler {
    (( $($Ti:ident),* )) => {
        impl<F, $($Ti,)* S, Fut, C, R> Handler<F, ( $($Ti,)* ), S, C, R> for F
        where
            Self: Clone,
            F: Fn( $($Ti),* ) -> Fut + Send + Sync + 'static,
            C: Clone + Send + Sync + 'static,
            Fut: Future<Output = R> + Send + 'static,
            R: Translatable,
            S: Clone + Send + Sync + 'static,
            $( $Ti: FromConnectionMessage<S, C>, )*
        {
            fn call(
                &self,
                connection: std::sync::Arc<ConnectionDetails<C>>,
                message: LiRpcRequest,
                state: S,
            ) -> Pin<Box<dyn Future<Output = LiRpcResponse> + Send>> {
                // Touch parameters to suppress unused warnings in the 0-argument case.
                let _ = (&connection, &message, &state);

                let slf = self.clone();

                Box::pin(async move {
                    build_lirpc_response(message.headers.id, true, slf($(
                        try_extract!($Ti, connection, message, state)
                    ),*).await)
                })
            }

            fn get_spec(&self) -> LiRpcMethodSpec {
                let signature_extensions: Vec<Option<Type>> = vec![$($Ti::extends_signature_with(),)*];

                LiRpcMethodSpec {
                    messages: signature_extensions.into_iter().flatten().collect(),
                    returns: R::get_type(),
                }
            }

        }
    };
}

// Generate implementations for 0..16 arguments
impl_handler!(());
impl_handler!((T1));
impl_handler!((T1, T2));
impl_handler!((T1, T2, T3));
impl_handler!((T1, T2, T3, T4));
impl_handler!((T1, T2, T3, T4, T5));
impl_handler!((T1, T2, T3, T4, T5, T6));
impl_handler!((T1, T2, T3, T4, T5, T6, T7));
impl_handler!((T1, T2, T3, T4, T5, T6, T7, T8));
impl_handler!((T1, T2, T3, T4, T5, T6, T7, T8, T9));
impl_handler!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10));
impl_handler!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11));
impl_handler!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12));
impl_handler!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13));
impl_handler!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14));
impl_handler!((
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
));
impl_handler!((
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
));
