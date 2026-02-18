# help
[group("misc")]
default:
    @just --list

[group("rust")]
fmt-rust:
    @cargo fmt --all

[group("ci")]
fmt: fmt-rust

[group("rust")]
lint-rust:
    @cargo clippy --all-targets --all-features -- -D warnings

[group("ci")]
lint: lint-rust

[group("rust")]
ut-rust:
    @cargo test --workspace

[group("node")]
[working-directory("code-mirror")]
ut-node:
    @npm run test

[group("ci")]
ut: ut-rust ut-node

[group("ci")]
ut-cov:
    @cargo llvm-cov

[group("node")]
[working-directory("code-mirror")]
build-node:
    @npm ci
    @npm run build

[group("ci")]
build: build-node

# check ci
[group("ci")]
ci: fmt lint ut

# run cli by cargo
convert input_path="fixtures/canvas/input.playbook":
    @cargo run -p playbook-cli -- {{input_path}}

# build cli
release-cli:
    @cargo build --release -p playbook-cli
    @mkdir -p build
    @cp ./target/release/playbook-cli build/

[working-directory("core")]
release-wasm:
    @wasm-pack build --target web

[group("node")]
[working-directory("code-mirror")]
build-code-mirror:
    @npm install
    @npm run build

[group("node")]
[working-directory("code-mirror")]
publish-code-mirror: build-code-mirror
    @npm publish
