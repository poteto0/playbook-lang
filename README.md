# Playbook Lang

A domain-specific language (DSL) for describing basketball playbooks and generating SVG previews.

## Features

- **Simple DSL**: Human-readable syntax to describe player positions and actions.
- **SVG Generation**: Generates clean, scalable vector graphics for your playbooks.
- **Fast**: Built with Rust for high performance and safety.
- **Wasm Ready**: The core logic is designed to run in both CLI and Web environments.

## Quick Start

### 1. Installation

clone repository

### 2. Write Playbook Code

```playbook
players = { p1, p2, p3, p4, p5 }

state = {
  baller = p1,
  position = {
    p1 = (0, 0)
    p2 = (50, 50)
    p3 = (-50, 50)
    p4 = (50, -50)
    p5 = (-50, -50)
  },
}

action = {
  move = {
    p2 -> (0, 80)
  },
  pass = {
    p1 -> p2:after
  },
}

```

### 3 Run compile

#### 3.1 Run with pre-built binary

A pre-built CLI is available in the `build/` directory.

```bash
# Convert a sample playbook to SVG
./build/playbook-cli fixtures/input.playbook --output fixtures/output.svg
```

#### 3.2 Build from source

If you have [Rust](https://www.rust-lang.org/) and [just](https://github.com/casey/just) installed:

```bash
# Run all tests
just test

# Convert using cargo
just convert fixtures/input.playbook
```

### 4. Output svg

<img src="./fixtures/output.svg" alt="Output" width="500" height="500">

```svg
<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-100" y="-100" width="200" height="200" fill="none" stroke="#ccc" stroke-width="1" /><line x1="50" y1="50" x2="0" y2="80" stroke="black" stroke-width="2" marker-end="url(#arrowhead)" /><line x1="0" y1="0" x2="0" y2="80" stroke="black" stroke-width="2" stroke-dasharray="4" marker-end="url(#arrowhead)" /><circle cx="0" cy="0" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><circle cx="0" cy="0" r="4" fill="orange" stroke="black" stroke-width="1" transform="translate(10, -10)" /><circle cx="50" cy="50" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="80" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="80" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">2</text><circle cx="-50" cy="50" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="-50" cy="50" r="10" fill="white" stroke="black" stroke-width="2" /><text x="-50" y="50" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">3</text><circle cx="50" cy="-50" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="50" cy="-50" r="10" fill="white" stroke="black" stroke-width="2" /><text x="50" y="-50" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">4</text><circle cx="-50" cy="-50" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="-50" cy="-50" r="10" fill="white" stroke="black" stroke-width="2" /><text x="-50" y="-50" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">5</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
