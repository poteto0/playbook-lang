# デフォルト: ヘルプを表示
default:
    @just --list

# コードのフォーマットを整える
fmt:
    cargo fmt --all

# 静的解析を実行してコードの質をチェックする
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# テストを実行する
test:
    cargo test --workspace

# プロジェクト全体をビルドする
build:
    cargo build --workspace

# 全てのチェック（fmt, lint, test）を実行する
check: fmt lint test

# CLIを実行して変換を試す
convert input_path="fixtures/canvas/input.playbook":
    cargo run -p playbook-cli -- {{input_path}}

# リリースバイナリをビルドして build/ に配置する
release:
    @cargo build --release -p playbook-cli
    @mkdir -p build
    @cp ./target/release/playbook-cli build/