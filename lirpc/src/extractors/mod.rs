mod connection_state;
pub(super) mod error;
mod message;
mod state;

pub use connection_state::ConnectionState;
pub use message::Message;
pub use state::State;

use crate::{
    connection_details::ConnectionDetails,
    lirpc_message::LiRpcRequest,
    translatable::{Translatable, Type},
};

pub trait FromConnectionMessage<S, C>
where
    Self: Sized + Send + 'static,
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error: Translatable + Send + Sync + 'static;

    fn from_connection_message(
        connection: &ConnectionDetails<C>,
        message: &LiRpcRequest,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;

    /// Extractors that extend the method signature
    /// with extra accepted messages should
    /// implement this method meaningfully
    fn extends_signature_with() -> Option<Type> {
        None
    }
}
