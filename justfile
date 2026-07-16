ci:
    cargo build
    cargo build --examples
    cargo clippy -- -D warnings
    cargo clippy --examples -- -D warnings
    cargo test

push flags='':
    git push -u github $(git branch --show-current) {{flags}}
    git push -u origin $(git branch --show-current) {{flags}}

codegen-example-libs:
    cd client_examples/auth && cargo run --bin lirpc_cli -- codegen rust api_spec.json auth_lib --overwrite
    cd client_examples/greeter && cargo run --bin lirpc_cli -- codegen rust api_spec.json greeter_lib --overwrite
    cd client_examples/with_app_state && cargo run --bin lirpc_cli -- codegen rust api_spec.json with_app_state_lib --overwrite
