use lirpc_rs_client::{Client, transport::Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMessage {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MyError {
    AuthFailure,
    Unauthenticated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMessage {
    pub secret: String,
}

pub async fn login<T, F>(
    client: &mut Client<T, F>,
    request: AuthMessage,
) -> Result<Result<(), MyError>, lirpc_rs_client::error::Error>
where
    T: Transport<F>,
{
    client
        .call::<AuthMessage, Result<(), MyError>>("login".to_string(), Some(request))
        .await?
        .resolve()
        .await
}

pub async fn protected_function<T, F>(
    client: &mut Client<T, F>,
) -> Result<SecretMessage, lirpc_rs_client::error::Error>
where
    T: Transport<F>,
{
    client
        .call::<(), SecretMessage>("protected_function".to_string(), None)
        .await?
        .resolve()
        .await
}
