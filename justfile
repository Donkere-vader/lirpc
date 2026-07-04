ci:
    cargo build
    cargo build --examples
    cargo clippy -- -D warnings
    cargo clippy --examples -- -D warnings
    cargo test
