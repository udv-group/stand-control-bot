fmt:
    cargo clippy --fix --allow-dirty
    cargo fmt

check:
    cargo check
    cargo fmt --check
    cargo clippy -- -D warnings