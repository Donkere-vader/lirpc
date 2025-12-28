use std::sync::Arc;

use lirpc::{
    ServerBuilder,
    error::LiRpcError,
    extractors::{Output, State},
};
use serde::Serialize;
use tokio::sync::Mutex;

#[derive(Default, Clone)] // State used in the server must implement Clone
struct AppState {
    count: Arc<Mutex<u64>>,
}

#[derive(Serialize)]
struct CountResponse {
    count: u64,
}

async fn count(
    State(app_state): State<AppState>,
    output: Output<CountResponse>,
) -> Result<(), LiRpcError> {
    let mut counter_lock = app_state.count.lock().await;
    *counter_lock += 1;
    let value = *counter_lock;
    drop(counter_lock);

    output.send(CountResponse { count: value }).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .register_handler("count".to_string(), count)
        .build_with_state(AppState::default());

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving LiRpc server");
}
