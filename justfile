
compile-contracts:
    cargo run --bin lirpc-cli -- compile lirpc auth example-contracts/auth.json 0.1.0
    cargo run --bin lirpc-cli -- compile lirpc greeter example-contracts/greeter.json 0.1.0
    cargo run --bin lirpc-cli -- compile lirpc with_app_state example-contracts/with_app_state.json 0.1.0
    cargo run --bin lirpc-cli -- compile lirpc greeter_stream example-contracts/greeter_stream.json 0.1.0

ci:
    cargo build
    cargo build --examples
    cargo clippy -- -D warnings
    cargo clippy --examples -- -D warnings
    cargo test
