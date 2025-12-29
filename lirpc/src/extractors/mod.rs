mod connection_state;
mod message;
mod output;
mod output_stream;
mod state;

use std::fmt::Debug;

pub use connection_state::ConnectionState;
pub use message::Message;
pub use output::Output;
pub use output_stream::OutputStream;
pub use state::State;

use tokio::sync::mpsc::Sender;

use crate::{
    connection_details::ConnectionDetails,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub trait FromConnectionMessage<S, C>
where
    Self: Sized,
    C: Clone + Send + Sync + 'static,
{
    type Error: Debug;

    fn from_connection_message(
        connection: &ConnectionDetails<C>,
        message: &LiRpcMessage,
        state: &S,
        output: &Sender<LiRpcResponse>,
    ) -> Result<Self, Self::Error>;
}
