---
sidebar_position: 4
---

# CLI Tool

The Playbook CLI is a powerful command-line interface for working with Playbook files.

## Installation

You can build and install the CLI from the source code.

```bash
cd cli
cargo build --release
```

The binary will be available at `target/release/playbook-cli`.

## Commands

### `render`

The `render` command converts a Playbook file into an SVG.

```bash
playbook-cli render input.playbook --output output.svg
```

By default, if you don't provide the `--output` option, it will save the file with the same name and a `.svg` extension.

### `fmt`

The `fmt` command formats a Playbook file and prints it to the console.

```bash
playbook-cli fmt input.playbook
```

You can use it to reformat your Playbook code automatically.

## Integration

The CLI is written in Rust and provides a fast, reliable way to automate Playbook-to-SVG conversion in your workflows, CI/CD pipelines, or local development.
