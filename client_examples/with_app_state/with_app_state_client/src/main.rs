use lirpc_rs_client::Client;
use with_app_state_lib::count;

#[tokio::main]
async fn main() {
    let mut client = Client::new_tcp_plain("127.0.0.1:5000").await.unwrap();

    let response = count(&mut client).await.unwrap();

    println!("Server count: {}", response.count);
}
