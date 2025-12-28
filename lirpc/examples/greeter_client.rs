use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (mut socket, _response) = connect_async("ws://127.0.0.1:5000").await.unwrap();

    socket
        .send(Message::text(format!(
            "{}\n\n{}",
            r#"{"id": 0, "method": "greet"}"#, r#"{"name": "Cas"}"#
        )))
        .await
        .unwrap();

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(m) => println!("Received: {m}"),
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}
