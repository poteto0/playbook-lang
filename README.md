# Playbook Lang

A domain-specific language (DSL) for describing basketball playbooks and generating SVG previews.

## Features

- **Simple DSL**: Human-readable syntax to describe player positions and actions.
- **SVG Generation**: Generates clean, scalable vector graphics for your playbooks.
- **Fast**: Built with Rust for high performance and safety.
- **Wasm Ready**: The core logic is designed to run in both CLI and Web environments.

## Quick Start

### 1. Installation

clon repository

### 2. Run with pre-built binary

A pre-built CLI is available in the `build/` directory.

```bash
# Convert a sample playbook to SVG
./build/playbook-cli fixtures/canvas/input.playbook --output output.svg
```

### 3. Build from source

If you have [Rust](https://www.rust-lang.org/) and [just](https://github.com/casey/just) installed:

```bash
# Run all tests
just test

# Convert using cargo
just convert fixtures/canvas/input.playbook
```

## Language Syntax

```playbook
players = { p1, p2 }

state = {
  baller = p1,
  position = {
    p1 = (0, 0)
    p2 = (50, 50)
  }
}

action {
  move = {
    p2 -> (0, 50)
  }
  pass = {
    p1 -> p2:after
  }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
