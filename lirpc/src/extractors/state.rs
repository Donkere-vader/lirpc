use crate::{
    connection_details::ConnectionDetails, error::LiRpcError, extractors::FromConnectionMessage,
    lirpc_message::LiRpcRequest,
};

pub struct State<S>(pub S);

impl<S, C> FromConnectionMessage<S, C> for State<S>
where
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error = LiRpcError;

    async fn from_connection_message(
        _connection: &ConnectionDetails<C>,
        _message: &LiRpcRequest,
        state: &S,
    ) -> Result<Self, Self::Error> {
        Ok(Self(state.clone()))
    }
}
