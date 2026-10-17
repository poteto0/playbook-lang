---
sidebar_position: 1
---

# Players

players defines the players appearing in the playbook. Currently, only 1-5 are permitted, and they are denoted by a number following `p`, such as p1-p5.

```playbook
// 3人
players = {p1, p2, p3}

// 5人
players = {p1, p2, p3, p4, p5}
```

Every player listed here must have a `state.position` entry; otherwise generation fails with a `Player '...' has no position in state` error instead of silently drawing the player at the court origin.
