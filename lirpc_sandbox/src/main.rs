use std::{env, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use lirpc::{ServerBuilder, error::LiRpcError, extractors};
use serde::{Deserialize, Serialize};
use tracing::{Level, info};

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<u64>>,
}

#[derive(Deserialize)]
struct HelloMessage {
    name: String,
}

#[derive(Serialize)]
struct HelloResponse {
    msg: String,
    count: u64,
}

async fn do_something(
    extractors::AppState(app_state): extractors::AppState<AppState>,
    extractors::Message(msg): extractors::Message<HelloMessage>,
    output: extractors::Output<HelloResponse>,
) -> Result<(), LiRpcError> {
    let mut counter_lock = app_state.counter.lock().await;
    *counter_lock += 1;
    let counter_value = *counter_lock;
    drop(counter_lock);

    output
        .send(HelloResponse {
            msg: format!("Hello {}!", msg.name),
            count: counter_value,
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
        .register_handler("do_something".to_string(), do_something)
        .build_with_state(AppState {
            counter: Arc::new(Mutex::new(0)),
        });

    info!("Serving on 127.0.0.1:5000");

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving server");
}
