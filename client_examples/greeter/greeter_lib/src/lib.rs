use lirpc_rs_client::{Client, transport::Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingResponse {
    pub msg: String,
}

pub async fn greet<T, F>(
    client: &mut Client<T, F>,
    request: GreetingRequest,
) -> Result<GreetingResponse, lirpc_rs_client::error::Error>
where
    T: Transport<F>,
{
    client
        .call::<GreetingRequest, GreetingResponse>("greet".to_string(), Some(request))
        .await?
        .resolve()
        .await
}
