format:
    cargo fmt
    cargo clippy --fix --allow-dirty

lint:
    cargo fmt --check
    cargo clippy -- -D warnings