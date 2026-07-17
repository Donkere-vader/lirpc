# LiRPC

LiRPC is a lightweight, strictly typed RPC framework for Rust Servers and Clients written in any[^1] language over WebSockets. It provides a simple way to define typed request/response contracts and wire up handlers with minimal boilerplate. The companion `lirpc_macros` crate generates contract artifacts at build time for tooling and codegen.

[^1]: Currently no official client support exists yet, as codegen is not yet implemented.

## Features

- Typed RPC handlers with ergonomic extractors (`Message<T>`, `Output<T>`, `OutputStream<T>`, `State<S>`, `ConnectionState<C>`). (Should feel very familiar to people that have used [tokio's axum](https://github.com/tokio-rs/axum) before).
- WebSocket server with concurrent request handling
- Optional global app state and per-connection state
- Build-time contract generation via `#[lirpc_type]` and `#[lirpc_method]` macros
- Simple wire format: JSON headers + JSON payload. Ideally also with support for binary formats in the future to safe on bandwidth and serialization time.

## Quick Start

Define a request/response type and a handler, then register it on the server:

```rust
/// similar to lirpc/examples/greeter.rs
use std::{env, str::FromStr};

use lirpc::{ServerBuilder, compile_json_api_spec, extractors::Message, handlers, types};
use lirpc_macros::LiRpcType;
use serde::{Deserialize, Serialize};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(LiRpcType, Serialize, Deserialize)]
struct GreetingRequest {
    name: String,
}

#[derive(LiRpcType, Serialize, Deserialize)]
struct GreetingResponse {
    msg: String,
}

async fn greet(Message(msg): Message<GreetingRequest>) -> GreetingResponse {
    GreetingResponse {
        msg: format!("Hello {}!", msg.name),
    }
}

#[tokio::main]
async fn main() {
    let server = ServerBuilder::new()
        .with_handlers(handlers!(greet))
        // Registering the types is necessary for generating the correct api spec
        .with_types(types!(GreetingRequest, GreetingResponse))
        .build();

    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_max_level(
                env::var("LOG_LEVEL")
                    .ok()
                    .and_then(|l| Level::from_str(&l).ok())
                    .unwrap_or(Level::INFO),
            )
            .finish(),
    )
    .expect("Failed to set global tracing subscriber");

    let api_spec = compile_json_api_spec!(server).unwrap();
    fs::write("api_spec.json", api_spec)
        .await
        .unwrap();

    info!("Serving on 127.0.0.1:5000");

    server
        .serve("127.0.0.1:5000")
        .await
        .expect("Error serving server");
}
```

Use the api spec in `api_spec.json` to generate a client library:

```sh
lirpc codegen rust api_spec.json greeter
```

Then on the client side we can simply do the following:

```rust
/// Similar to client_examples/greeter/greeter_client/src/main.rs
use greeter::{GreetingRequest, greet};
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
```

## Contributing

Feel free to open issues or PRs. Although I cannot guarantee that this project will ever go anywhere.

### Vibe coding & AI-assisted development

Vibe coded PRs will not be accepted. AI-assisted coding is certainly allowed, but you should be able to reason about/ defend the changes that you (or an AI model under your supervision) made.

## Crates

- `lirpc`: core framework (server, handler trait, extractors, message types)
- `lirpc_macros`: proc-macros to annotate types and methods, generating contract files into `OUT_DIR` for tooling and codegen
