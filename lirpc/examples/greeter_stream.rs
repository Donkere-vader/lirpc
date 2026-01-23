use std::{env, str::FromStr, time::Duration};

use lirpc::{
    ServerBuilder,
    error::LiRpcError,
    extractors::{Message, OutputStream},
};
use lirpc_macros::{lirpc_method, lirpc_type};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
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

#[lirpc_method]
async fn greet_stream(
    Message(msg): Message<GreetingRequest>,
    output: OutputStream<GreetingResponse>,
) -> Result<(), LiRpcError> {
    loop {
        output
            .send(GreetingResponse {
                msg: format!("Hello {}!", msg.name),
            })
            .await?;

        sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .register_handler("greet_stream".to_string(), greet_stream)
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
