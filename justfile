default: check

check:
    cargo check --all-targets

build:
    cargo build

test:
    cargo test --all-targets

fmt:
    cargo fmt --all

lint:
    cargo clippy --all-targets -- -D warnings

ci: fmt lint test
