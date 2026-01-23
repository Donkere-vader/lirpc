use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcFunctionCall, LiRpcStreamOutput},
    stream_manager::StreamManager,
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
        _message: &LiRpcFunctionCall,
        _state: &S,
        _output: &Sender<LiRpcStreamOutput>,
        _stream_manager: &StreamManager,
    ) -> Result<Self, Self::Error> {
        Ok(Self(connection.connection_state.clone()))
    }
}
