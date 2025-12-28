use lirpc::{
    ServerBuilder,
    error::LiRpcError,
    extractors::{Message, Output},
};
use lirpc_macros::{lirpc_method, lirpc_type};
use serde::{Deserialize, Serialize};

#[lirpc_type]
#[derive(Deserialize)]
struct GreetingRequest {
    name: String,
}

#[derive(Serialize)]
struct GreetingResponse {
    msg: String,
}

#[lirpc_method]
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
    let server = ServerBuilder::new()
        .register_handler("greet".to_string(), greet)
        .build();

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving server");
}
