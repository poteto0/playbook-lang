---
sidebar_position: 1.5
---

# Defenders

defenders defines the defenders appearing in the playbook, in addition to `players`. They are denoted by a number following `d`, such as d1-d5.

An identifier cannot be used in both `players` and `defenders` at the same time.

```playbook
players = {p1, p2}
defenders = {d1, d2}
```

Defenders are drawn as a plain `x` mark, without the circle used for players, and never carry the ball.
