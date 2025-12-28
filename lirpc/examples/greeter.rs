use lirpc::{
    ServerBuilder,
    error::LiRpcError,
    extractors::{Message, Output},
};
use serde::{Deserialize, Serialize};

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
    let server = ServerBuilder::new()
        .register_handler("greet".to_string(), greet)
        .build();

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving server");
}
