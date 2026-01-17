use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub struct ConnectionState<C>(pub C);

impl<S, C> FromConnectionMessage<S, C> for ConnectionState<C>
where
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error = LiRpcError;

    async fn from_connection_message(
        connection: &ConnectionDetails<C>,
        _message: &LiRpcMessage,
        _state: &S,
        _output: &Sender<LiRpcResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self(connection.connection_state.clone()))
    }
}
