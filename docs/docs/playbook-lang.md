---
sidebar_position: 1
slug: /
---

# Playbook-Lang

Try it easily.

[web-playground](https://playbook-lang-app.poteto-mahiro.com/)

## DSL for BasketBall Playbook.

### :rocket: write playbook by markup language

You can save the playbook by markup language.
Lightweight than png or jpeg.

```playbook
players = { p1, p2 }

position = {
  p1 = (0, 0),
  p2 = (50, 50),
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

### :zap: rust-base fast compile

Quickstart compile time(5 players 1 play);

```
real    0m0.006s
user    0m0.004s
sys     0m0.002s
```

### :goal_net: Someday, Be De Facto Standard

We will develop lang and web-app for compile.
