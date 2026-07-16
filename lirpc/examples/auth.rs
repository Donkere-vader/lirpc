use std::{env, str::FromStr, sync::Arc};

use lirpc::{
    ServerBuilder,
    connection_details::ConnectionDetails,
    extractors::{self, FromConnectionMessage},
    handlers,
    lirpc_message::LiRpcRequest,
    types,
};
use lirpc_macros::LiRpcType;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Clone)]
struct User(String);

#[derive(Default, Clone)]
struct ConnectionState {
    username: Arc<Mutex<Option<User>>>,
}

#[derive(LiRpcType, Serialize, Deserialize)]
struct AuthMessage {
    username: String,
    password: String,
}

#[derive(LiRpcType, Serialize, Deserialize)]
struct SecretMessage {
    secret: String,
}

struct AuthRequired(pub User);

impl FromConnectionMessage<(), ConnectionState> for AuthRequired {
    type Error = MyError;

    async fn from_connection_message(
        connection: &ConnectionDetails<ConnectionState>,
        _message: &LiRpcRequest,
        _state: &(),
    ) -> Result<Self, Self::Error> {
        let username_lock = connection.connection_state.username.lock().await;

        match username_lock.as_ref() {
            Some(user) => Ok(Self(user.clone())),
            None => Err(MyError::Unauthenticated),
        }
    }
}

#[derive(LiRpcType, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MyError {
    AuthFailure,
    Unauthenticated,
}

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

async fn protected_function(AuthRequired(User(username)): AuthRequired) -> SecretMessage {
    info!("user '{username}' has requested the secret");

    SecretMessage {
        secret: "my-secret-123".to_string(),
    }
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .with_handlers(handlers!(login, protected_function))
        .with_types(types!(AuthMessage, SecretMessage, MyError))
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

    let api_spec = server
        .compile_json_api_spec(
            "auth_lib".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
        .unwrap();
    fs::write("./client_examples/auth/api_spec.json", api_spec)
        .await
        .unwrap();

    info!("Serving on 127.0.0.1:5000");

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Failed to serve server");
}
