use auth_lib::{AuthMessage, login, protected_function};
use lirpc_rs_client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new_tcp_plain("127.0.0.1:5000").await.unwrap();

    let response = login(
        &mut client,
        AuthMessage {
            username: "some-user".to_string(),
            password: "password".to_string(),
        },
    )
    .await
    .unwrap();

    match response {
        Ok(_) => println!("Authenticated"),
        Err(e) => {
            eprintln!("Failed to authenticate: {e:?}");
            return;
        }
    }

    let protected_response = protected_function(&mut client).await.unwrap();

    println!("received secret from server: {}", protected_response.secret);
}
