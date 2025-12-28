---
sidebar_position: 4
---

# Timing

Timing determines which position of the player is specified when the player is designated as the destination. Write `:` after `player`, followed by the position.

For example, in the following cases, extend the path endpoint to the destination of p2.

```playbook
action = {
  pass = {
    p1 -> p2:after
  },
}
```

## Allowed Syntax

### Screen Action

The following three actions are allowed in Screen Action:

- `before`
- `middle`
- `after`

### Pass Action

The following three actions are allowed in Pass Action:

- `before`
- `after`
