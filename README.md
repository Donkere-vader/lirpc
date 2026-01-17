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
/// lirpc/examples/greeter.rs
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
```

Client example:

_Waiting for codegen implementation_

The current client examples are hardcoded, just to be able to actually test the example server implementations.

## Contributing

Feel free to open issues or PRs. Although I cannot guarantee that this project will ever go anywhere.

### Vibe coding & AI-assisted development

Vibe coded PRs will not be accepted. AI-assisted coding is certainly allowed, but you should be able to reason about/ defend the changes that you (or an AI model under your supervision) made.

## Crates

- `lirpc`: core framework (server, handler trait, extractors, message types)
- `lirpc_macros`: proc-macros to annotate types and methods, generating contract files into `OUT_DIR` for tooling and codegen
