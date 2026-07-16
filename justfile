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
    cargo run --bin lirpc_cli -- codegen rust client_examples/auth/api_spec.json client_examples/auth/auth_lib --overwrite
    cargo run --bin lirpc_cli -- codegen rust client_examples/greeter/api_spec.json client_examples/greeter/greeter_lib --overwrite
    cargo run --bin lirpc_cli -- codegen rust client_examples/with_app_state/api_spec.json client_examples/with_app_state/with_app_state_lib --overwrite
