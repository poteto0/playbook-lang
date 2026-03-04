---
sidebar_position: 3
---

# Formatter

The Playbook formatter is a tool designed to automatically format Playbook files, ensuring consistent style and readability across projects.

## Usage

### CLI

You can use the formatter via the Playbook CLI:

```bash
playbook-cli fmt input.playbook
```

This will output the formatted version of `input.playbook` to stdout.

### Library (Rust)

The formatter is also available as a Rust library.

```rust
use playbook_lang_formatter::format;

let input = "players={p1,p2}state={baller=p1}";
let formatted = format(input);
println!("{}", formatted);
```

## Formatting Rules

The formatter applies the following rules:

- **Indentation**: 2 spaces.
- **Players Block**: One line with spaces around braces.
- **State Block**: Key-value pairs with each property on a new line.
- **Actions Block**: Actions are grouped and properly indented.
- **Comments**: Preservation of comments in the appropriate sections.

## Example

### Before

```playbook
players={p1,p2}state={baller=p1,position={p1=(0,0),p2=(10,10)}}
```

### After

```playbook
players = { p1, p2 }

state = {
  baller = p1,
  position = {
    p1 = (0, 0),
    p2 = (10, 10),
  },
}
```
