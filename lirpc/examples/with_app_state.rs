use std::{env, str::FromStr, sync::Arc};

use lirpc::{ServerBuilder, extractors::State};
use lirpc_macros::LiRpcType;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Default, Clone)] // State used in the server must implement Clone
struct AppState {
    count: Arc<Mutex<u64>>,
}

#[derive(LiRpcType, Serialize, Deserialize)]
struct CountResponse {
    count: u64,
}

#[derive(LiRpcType, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MyError {
    ServerError,
}

async fn count(State(app_state): State<AppState>) -> CountResponse {
    let mut counter_lock = app_state.count.lock().await;
    *counter_lock += 1;
    let value = *counter_lock;
    drop(counter_lock);

    CountResponse { count: value }
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .register_handler("count".to_string(), count)
        .build_with_state(AppState::default());

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
        .expect("Error serving LiRpc server");
}
