use std::{env, str::FromStr, sync::Arc};

use lirpc::{
    ServerBuilder,
    connection_details::ConnectionDetails,
    error::LiRpcError,
    extractors::{self, FromConnectionMessage, Output},
    lirpc_message::{
        IntoRawLiRpcResponsePayload, LiRpcFunctionCall, LiRpcStreamOutput, RawLiRpcMessagePayload,
    },
    stream_manager::StreamManager,
};
use lirpc_macros::{lirpc_method, lirpc_type};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{Mutex, mpsc::Sender};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

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
    type Error = MyError;

    async fn from_connection_message(
        connection: &ConnectionDetails<ConnectionState>,
        _message: &LiRpcFunctionCall,
        _state: &(),
        _output: &Sender<LiRpcStreamOutput>,
        _stream_manager: &StreamManager,
    ) -> Result<Self, Self::Error> {
        let username_lock = connection.connection_state.username.lock().await;

        match username_lock.as_ref() {
            Some(user) => Ok(Self(user.clone())),
            None => Err(MyError::Unauthenticated),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
#[lirpc_type]
pub enum MyError {
    ServerError,
    AuthFailure,
    Unauthenticated,
}

impl From<LiRpcError> for MyError {
    fn from(_: LiRpcError) -> Self {
        Self::ServerError
    }
}

impl IntoRawLiRpcResponsePayload for MyError {
    fn into(&self) -> RawLiRpcMessagePayload {
        match self {
            MyError::ServerError => RawLiRpcMessagePayload::Json(json!({"error": "server_error"})),
            MyError::AuthFailure => RawLiRpcMessagePayload::Json(json!({"error": "auth_failure"})),
            MyError::Unauthenticated => {
                RawLiRpcMessagePayload::Json(json!({"error": "unauthenticated"}))
            }
        }
    }
}

#[lirpc_method]
async fn login(
    extractors::ConnectionState(connection_state): extractors::ConnectionState<ConnectionState>,
    extractors::Message(message): extractors::Message<AuthMessage>,
) -> Result<(), MyError> {
    if &message.password == "password" {
        let mut username_lock = connection_state.username.lock().await;
        *username_lock = Some(User(message.username.to_string()));
        drop(username_lock);

        info!("user '{}' has authenticated", message.username);

        Ok(())
    } else {
        Err(MyError::AuthFailure)
    }
}

#[lirpc_method]
async fn protected_function(
    AuthRequired(User(username)): AuthRequired,
    output: Output<SecretMessage>,
) -> Result<(), MyError> {
    info!("user '{username}' has requested the secret");

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

    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(
                env::var("LOG_LEVEL")
                    .ok()
                    .and_then(|l| Level::from_str(&l).ok())
                    .unwrap_or(Level::INFO),
            )
            .finish(),
    )
    .expect("Failed to set global tracing subscriber");

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Failed to serve server");
}
