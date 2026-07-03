use crate::{
    connection_details::ConnectionDetails, extractors::FromConnectionMessage,
    lirpc_message::LiRpcRequest,
};

pub struct ConnectionState<C>(pub C);

impl<S, C> FromConnectionMessage<S, C> for ConnectionState<C>
where
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error = ();

    async fn from_connection_message(
        connection: &ConnectionDetails<C>,
        _message: &LiRpcRequest,
        _state: &S,
    ) -> Result<Self, Self::Error> {
        Ok(Self(connection.connection_state.clone()))
    }
}
