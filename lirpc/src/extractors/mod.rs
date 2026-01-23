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
    lirpc_message::{LiRpcFunctionCall, LiRpcStreamOutput},
    stream_manager::StreamManager,
};

pub trait FromConnectionMessage<S, C>
where
    Self: Sized + Send + 'static,
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error: Debug + Send + Sync + 'static;

    fn from_connection_message(
        connection: &ConnectionDetails<C>,
        message: &LiRpcFunctionCall,
        state: &S,
        output: &Sender<LiRpcStreamOutput>,
        stream_manager: &StreamManager,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}
