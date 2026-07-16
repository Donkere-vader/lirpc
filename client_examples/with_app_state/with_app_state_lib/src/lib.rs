use lirpc_rs_client::{Client, transport::Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountResponse {
    pub count: u64,
}

pub async fn count<T, F>(
    client: &mut Client<T, F>,
) -> Result<CountResponse, lirpc_rs_client::error::Error>
where
    T: Transport<F>,
{
    client.call::<(), CountResponse>("count".to_string(), None).await?.resolve().await
}
