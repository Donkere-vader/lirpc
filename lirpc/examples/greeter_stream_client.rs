use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    let (socket, _response) = connect_async("ws://127.0.0.1:5000").await.unwrap();

    let (mut tx, mut rx) = socket.split();

    tx
        .send(Message::text(
            r#"{"type":"function_call", "headers": {"id": 1, "method": "greet_stream"}, "payload": {"name": "Cas"}}"#
        ))
        .await
        .unwrap();

    tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            match msg {
                Ok(m) => println!("Received: {m}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    });

    sleep(Duration::from_secs(5)).await;

    tx.send(Message::text(r#"{"type":"close_stream", "stream_id": 1}"#))
        .await
        .unwrap();

    tx.send(Message::Close(None))
        .await
        .expect("Failed to send close message");
    tx.close()
        .await
        .expect("Failed to close sink of websocket connection");
}
