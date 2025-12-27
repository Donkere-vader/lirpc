use std::{env, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

use lirpc::{
    ServerBuilder,
    error::LiRpcError,
    extractors::{Message, Output, OutputStream, State},
};
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
    State(app_state): State<AppState>,
    Message(msg): Message<HelloMessage>,
    output: Output<HelloResponse>,
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

async fn do_something_twice(
    State(app_state): State<AppState>,
    Message(msg): Message<HelloMessage>,
    output: OutputStream<HelloResponse>,
) -> Result<(), LiRpcError> {
    let mut counter_lock = app_state.counter.lock().await;
    *counter_lock += 1;
    let counter_value = *counter_lock;
    drop(counter_lock);

    let f1 = output.send(HelloResponse {
        msg: format!("Hello {}!", msg.name),
        count: counter_value,
    });

    let f2 = output.send(HelloResponse {
        msg: format!("Hello {}!", msg.name),
        count: counter_value,
    });

    let (r1, r2) = tokio::join!(f1, f2);
    r1?;
    r2?;

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
        .register_handler("do_something_twice".to_string(), do_something_twice)
        .build_with_state(AppState {
            counter: Arc::new(Mutex::new(0)),
        });

    info!("Serving on 127.0.0.1:5000");

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving server");
}
