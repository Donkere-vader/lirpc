use std::{pin::Pin, sync::Arc};

use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub trait Handler<F, T, S, C>
where
    Self: Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    fn call(
        &self,
        connection: Arc<ConnectionDetails<C>>,
        message: LiRpcMessage,
        state: S,
        output: Sender<LiRpcResponse>,
    ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>>;
}

macro_rules! try_extract {
    ($Ty:ty, $connection:expr, $message:expr, $state:expr, $output:expr) => {{
        match <$Ty as FromConnectionMessage<_, _>>::from_connection_message(
            &$connection,
            &$message,
            &$state,
            &$output,
        ) {
            Ok(value) => value,
            Err(e) => {
                // Format the error into a Send-safe String outside the async block,
                // then early-return a pinned async error future.
                let err_string = format!("{:?}", e);
                return Box::pin(async move { Err(LiRpcError::ExtractorError(err_string)) });
            }
        }
    }};
}

macro_rules! impl_handler {
    (( $($Ti:ident),* )) => {
        impl<F, $($Ti,)* S, Fut, C> Handler<F, ( $($Ti,)* ), S, C> for F
        where
            F: Fn( $($Ti),* ) -> Fut + Send + Sync + 'static,
            C: Clone + Send + Sync + 'static,
            Fut: Future<Output = Result<(), LiRpcError>> + Send + 'static,
            $( $Ti: FromConnectionMessage<S, C>, )*
        {
            fn call(
                &self,
                connection: std::sync::Arc<ConnectionDetails<C>>,
                message: LiRpcMessage,
                state: S,
                output: Sender<LiRpcResponse>,
            ) -> Pin<Box<dyn Future<Output = Result<(), LiRpcError>> + Send>> {
                // Touch parameters to suppress unused warnings in the 0-argument case.
                let _ = (&connection, &message, &state, &output);

                Box::pin(self($(
                    try_extract!($Ti, connection, message, state, output)
                ),*))
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
