use std::{env, str::FromStr};

use lirpc::{
    ServerBuilder,
    error::LiRpcError,
    extractors::{Message, Output},
};
use serde::{Deserialize, Serialize};
use tracing::{Level, info};

#[derive(Deserialize)]
struct GreetingRequest {
    name: String,
}

#[derive(Serialize)]
struct GreetingResponse {
    msg: String,
}

async fn greet(
    Message(msg): Message<GreetingRequest>,
    output: Output<GreetingResponse>,
) -> Result<(), LiRpcError> {
    output
        .send(GreetingResponse {
            msg: format!("Hello {}!", msg.name),
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(
            Level::from_str(&env::var("LOG_LEVEL").unwrap_or_default()).unwrap_or(Level::INFO),
        )
        .init();

    let server = ServerBuilder::new()
        .register_handler("greet".to_string(), greet)
        .build();

    info!("Serving on 127.0.0.1:5000");

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving server");
}
