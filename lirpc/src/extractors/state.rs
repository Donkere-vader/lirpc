use tokio::sync::mpsc::Sender;

use crate::{
    connection::Connection,
    error::LiRpcError,
    extractors::FromConnectionMessage,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub struct State<S>(pub S);

impl<S: Clone> FromConnectionMessage<S> for State<S> {
    type Error = LiRpcError;

    fn from_connection_message(
        _connection: &Connection,
        _message: &LiRpcMessage,
        state: &S,
        _output: &Sender<LiRpcResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self(state.clone()))
    }
}
