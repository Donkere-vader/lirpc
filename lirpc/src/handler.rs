use crate::lirpc_message::IntoRawLiRpcResponsePayload;
use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcFunctionCall, LiRpcStreamOutput},
    stream_manager::StreamManager,
};

pub trait Handler<F, T, S, C, E>
where
    Self: Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
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

macro_rules! try_extract {
    ($Ty:ty, $connection:expr, $message:expr, $state:expr, $output:expr, $stream_manager:expr) => {{
        match <$Ty as FromConnectionMessage<_, _>>::from_connection_message(
            &$connection,
            &$message,
            &$state,
            &$output,
            &$stream_manager,
        )
        .await
        {
            Ok(value) => value,
            Err(e) => {
                // Format the error into a Send-safe String outside the async block,
                // then early-return a pinned async error future.
                return Err(LiRpcError::ExtractorError(
                    IntoRawLiRpcResponsePayload::into(&e),
                ));
            }
        }
    }};
}

macro_rules! impl_handler {
    (( $($Ti:ident),* )) => {
        impl<F, $($Ti,)* S, Fut, C, E> Handler<F, ( $($Ti,)* ), S, C, Result<(), E>> for F
        where
            Self: Clone,
            F: Fn( $($Ti),* ) -> Fut + Send + Sync + 'static,
            C: Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<(), E>> + Send + 'static,
            E: IntoRawLiRpcResponsePayload,
            S: Clone + Send + Sync + 'static,
            $( $Ti: FromConnectionMessage<S, C>, )*
        {
            fn call(
                &self,
                connection: std::sync::Arc<ConnectionDetails<C>>,
                message: LiRpcFunctionCall,
                state: S,
                output: Sender<LiRpcStreamOutput>,
                stream_manager: StreamManager,
            ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>> {
                // Touch parameters to suppress unused warnings in the 0-argument case.
                let _ = (&connection, &message, &state, &output, &stream_manager);

                let slf = self.clone();

                Box::pin(async move {
                    let invoke_result = slf($(
                        try_extract!($Ti, connection, message, state, output, stream_manager)
                    ),*).await;

                    match invoke_result {
                        Ok(r) => Ok(r),
                        Err(e) => Err(LiRpcError::ErrorInHandler(
                            IntoRawLiRpcResponsePayload::into(&e)
                        )),
                    }
                })
            }
        }

        impl<F, $($Ti,)* S, Fut, C> Handler<F, ( $($Ti,)* ), S, C, ()> for F
        where
            Self: Clone,
            F: Fn( $($Ti),* ) -> Fut + Send + Sync + 'static,
            C: Clone + Send + Sync + 'static,
            Fut: Future<Output = ()> + Send + 'static,
            S: Clone + Send + Sync + 'static,
            $( $Ti: FromConnectionMessage<S, C>, )*
        {
            fn call(
                &self,
                connection: std::sync::Arc<ConnectionDetails<C>>,
                message: LiRpcFunctionCall,
                state: S,
                output: Sender<LiRpcStreamOutput>,
                stream_manager: StreamManager,
            ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>> {
                // Touch parameters to suppress unused warnings in the 0-argument case.
                let _ = (&connection, &message, &state, &output, &stream_manager);

                let slf = self.clone();

                Box::pin(async move {
                    slf($(
                        try_extract!($Ti, connection, message, state, output, stream_manager)
                    ),*).await;

                    Ok(())
                })
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
