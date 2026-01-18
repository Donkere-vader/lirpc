use std::sync::Arc;

use lirpc::{
    ServerBuilder,
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::{self, FromConnectionMessage, Output},
    lirpc_message::{LiRpcMessage, LiRpcResponse},
};
use lirpc_macros::{lirpc_method, lirpc_type};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc::Sender};

#[derive(Clone)]
struct User(String);

#[derive(Default, Clone)]
struct ConnectionState {
    username: Arc<Mutex<Option<User>>>,
}

#[lirpc_type]
#[derive(Deserialize)]
struct AuthMessage {
    username: String,
    password: String,
}

#[lirpc_type]
#[derive(Serialize)]
struct SecretMessage {
    secret: String,
}

struct AuthRequired(pub User);

impl FromConnectionMessage<(), ConnectionState> for AuthRequired {
    type Error = String;

    async fn from_connection_message(
        connection: &ConnectionDetails<ConnectionState>,
        _message: &LiRpcMessage,
        _state: &(),
        _output: &Sender<LiRpcResponse>,
    ) -> Result<Self, Self::Error> {
        let username_lock = connection.connection_state.username.lock().await;

        match username_lock.as_ref() {
            Some(user) => Ok(Self(user.clone())),
            None => Err("Not authenticated in".to_string()),
        }
    }
}

#[lirpc_method]
async fn login(
    extractors::ConnectionState(connection_state): extractors::ConnectionState<ConnectionState>,
    extractors::Message(message): extractors::Message<AuthMessage>,
) -> Result<(), LiRpcError> {
    if &message.password == "password" {
        let mut username_lock = connection_state.username.lock().await;
        *username_lock = Some(User(message.username));
        drop(username_lock);
    }

    Ok(())
}

#[lirpc_method]
async fn protected_function(
    AuthRequired(User(username)): AuthRequired,
    output: Output<SecretMessage>,
) -> Result<(), LiRpcError> {
    println!("[INFO] user '{username}' has requested the secret");

    output
        .send(SecretMessage {
            secret: "my-secret-123".to_string(),
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .register_handler("login".to_string(), login)
        .register_handler("protected_function".to_string(), protected_function)
        .build_with_connection_state(ConnectionState::default);

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Failed to serve server");
}
