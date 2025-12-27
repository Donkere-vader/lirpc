mod message;
mod output;
mod output_stream;
mod state;

use std::fmt::Debug;

pub use message::Message;
pub use output::Output;
pub use output_stream::OutputStream;
pub use state::State;

use tokio::sync::mpsc::Sender;

use crate::{
    connection::Connection,
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};

pub trait FromConnectionMessage<S>
where
    Self: Sized,
{
    type Error: Debug;

    fn from_connection_message(
        connection: &Connection,
        message: &LiRpcMessage,
        state: &S,
        output: &Sender<LiRpcResponse>,
    ) -> Result<Self, Self::Error>;
}
