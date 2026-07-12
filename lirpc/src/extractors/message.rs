use serde::Deserialize;

use crate::{
    connection_details::ConnectionDetails,
    extractors::{FromConnectionMessage, error::LiRpcExtractorError},
    lirpc_message::{LiRpcPayload, LiRpcRequest},
    lirpc_type::LiRpcType,
    translatable::Type,
};

pub struct Message<M>(pub M)
where
    M: for<'a> Deserialize<'a>;

impl<S, M, C> FromConnectionMessage<S, C> for Message<M>
where
    M: LiRpcType + for<'a> Deserialize<'a> + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    type Error = LiRpcExtractorError;

    async fn from_connection_message(
        _connection: &ConnectionDetails<C>,
        message: &LiRpcRequest,
        _state: &S,
    ) -> Result<Self, Self::Error> {
        match &message.payload {
            Some(LiRpcPayload(json_value)) => Ok(Self(serde_json::from_value(json_value.clone())?)),
            // TODO: probably not very clean to just parse an empty string here
            None => Ok(Self(serde_json::from_str("")?)),
        }
    }

    fn extends_signature_with() -> Option<Type> {
        Some(M::get_type())
    }
}
