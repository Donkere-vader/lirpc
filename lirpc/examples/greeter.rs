use std::{env, str::FromStr};

use lirpc::{
    ServerBuilder,
    error::LiRpcError,
    extractors::{Message, Output},
    lirpc_message::{IntoRawLiRpcResponsePayload, RawLiRpcMessagePayload},
};
use lirpc_macros::{lirpc_method, lirpc_type};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[lirpc_type]
#[derive(Deserialize)]
struct GreetingRequest {
    name: String,
}

#[lirpc_type]
#[derive(Serialize)]
struct GreetingResponse {
    msg: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
#[lirpc_type]
pub enum MyError {
    ServerError,
}

impl IntoRawLiRpcResponsePayload for MyError {
    fn into(&self) -> RawLiRpcMessagePayload {
        match self {
            MyError::ServerError => RawLiRpcMessagePayload::Json(json!({"error": "server_error"})),
        }
    }
}

impl From<LiRpcError> for MyError {
    fn from(_: LiRpcError) -> Self {
        Self::ServerError
    }
}

#[lirpc_method]
async fn greet(
    Message(msg): Message<GreetingRequest>,
    output: Output<GreetingResponse>,
) -> Result<(), MyError> {
    output
        .send(GreetingResponse {
            msg: format!("Hello {}!", msg.name),
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .register_handler("greet".to_string(), greet)
        .build();

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
        .expect("Error serving server");
}
