use greeter_lib::{GreetingRequest, greet};
use lirpc_rs_client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new_tcp_plain("127.0.0.1:5000").await.unwrap();

    let response = greet(
        &mut client,
        GreetingRequest {
            name: "Cas".to_string(),
        },
    )
    .await
    .unwrap();

    println!("Server said: {}", response.msg);
}
