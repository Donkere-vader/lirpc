use std::{env, str::FromStr};

use lirpc::{ServerBuilder, extractors::Message, handlers, types};
use lirpc_macros::LiRpcType;
use serde::{Deserialize, Serialize};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(LiRpcType, Serialize, Deserialize)]
struct GreetingRequest {
    name: String,
}

#[derive(LiRpcType, Serialize, Deserialize)]
struct GreetingResponse {
    msg: String,
}

async fn greet(Message(msg): Message<GreetingRequest>) -> GreetingResponse {
    GreetingResponse {
        msg: format!("Hello {}!", msg.name),
    }
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .with_handlers(handlers!(greet))
        .with_types(types!(GreetingRequest, GreetingResponse))
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

    info!("Serving on 127.0.0.1:5000");

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving server");
}
