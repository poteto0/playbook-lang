# help
default:
    @just --list

fmt:
    @cargo fmt --all

lint:
    @cargo clippy --all-targets --all-features -- -D warnings

test:
    @cargo test --workspace

build:
    @cargo build --workspace

# check ci
ci: fmt lint test

# run cli by cargo
convert input_path="fixtures/canvas/input.playbook":
    @cargo run -p playbook-cli -- {{input_path}}

# build cli
release-cli:
    @cargo build --release -p playbook-cli
    @mkdir -p build
    @cp ./target/release/playbook-cli build/