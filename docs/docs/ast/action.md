---
sidebar_position: 3
---

# Action

The action section describes the movements in that play.

```playbook
action = {
  move = {
    p1 -> (50,50),
  },
}
```

## Allowed Syntax

### Move Action

Move represents the player's movement. Specify the player's movement coordinates as follows.

```playbook
players = {p1}

state = {
  position = {
    p1 = (0, 60),
  }
}

action = {
  move = {
    p1 -> (0, 0),
  }
}
```

<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-105" y="-105" width="210" height="210" fill="white" /><rect x="-100" y="-90" width="200" height="180" fill="white" stroke="black" stroke-width="2" /><rect x="-20" y="-90" width="40" height="65" fill="none" stroke="black" stroke-width="1" /><circle cx="0" cy="-25" r="20" fill="none" stroke="black" stroke-width="1" /><path d="M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90" fill="none" stroke="black" stroke-width="1" /><path d="M -20 90 A 20 20 0 0 1 20 90" fill="none" stroke="black" stroke-width="1" /><line x1="-12" y1="-88" x2="12" y2="-88" stroke="black" stroke-width="1" /><circle cx="0" cy="-84" r="5" stroke="red" stroke-width="1" fill="none" /><line x1="0" y1="60" x2="0" y2="0" stroke="black" stroke-width="2" marker-end="url(#arrowhead)" /><circle cx="0" cy="60" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>

#### Arc Move

Using `~>` allows you to draw an arc.

You can specify the direction after `~`. `l | left | r | right` are permitted.

To adjust the arc, you can add a minor up to three characters after the direction specification.

:::warning

Adjusting the default settings is not recommended, as it may unnecessarily complicate things. For most use cases, the default values are sufficient.

:::

```playbook
players = {p1, p2, p3}

state = {
  position = {
    p1 = (0, 60),
    p2 = (90, -80),
    p3 = (-90, -80),
  }
}

action = {
  move = {
    p1 -> (0, 0),
    p2 ~[l]> (60, 30),
    p3 ~[r:0.2]> (-60, 30),
  }
}
```

<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-105" y="-105" width="210" height="210" fill="white" /><rect x="-100" y="-90" width="200" height="180" fill="white" stroke="black" stroke-width="2" /><rect x="-20" y="-90" width="40" height="65" fill="none" stroke="black" stroke-width="1" /><circle cx="0" cy="-25" r="20" fill="none" stroke="black" stroke-width="1" /><path d="M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90" fill="none" stroke="black" stroke-width="1" /><path d="M -20 90 A 20 20 0 0 1 20 90" fill="none" stroke="black" stroke-width="1" /><line x1="-12" y1="-88" x2="12" y2="-88" stroke="black" stroke-width="1" /><circle cx="0" cy="-84" r="5" stroke="red" stroke-width="1" fill="none" /><line x1="0" y1="60" x2="0" y2="0" stroke="black" stroke-width="2" marker-end="url(#arrowhead)" /><path d="M 90 -80 Q 108 -16 60 30" stroke="black" stroke-width="2" fill="none" marker-end="url(#arrowhead)" /><path d="M -90 -80 Q -97 -19 -60 30" stroke="black" stroke-width="2" fill="none" marker-end="url(#arrowhead)" /><circle cx="0" cy="60" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><circle cx="90" cy="-80" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="60" cy="30" r="10" fill="white" stroke="black" stroke-width="2" /><text x="60" y="30" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">2</text><circle cx="-90" cy="-80" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="-60" cy="30" r="10" fill="white" stroke="black" stroke-width="2" /><text x="-60" y="30" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">3</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>

### Pass Action

A pass represents the movement of the ball from the player. Specify the receiving player as follows.

```playbook
players = {p1, p2}

state = {
  baller = p1,
  position = {
    p1 = (0, 60),
    p2 = (90, -80),
  },
}

action = {
  pass = {
    p1 -> p2,
  }
}
```

<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-105" y="-105" width="210" height="210" fill="white" /><rect x="-100" y="-90" width="200" height="180" fill="white" stroke="black" stroke-width="2" /><rect x="-20" y="-90" width="40" height="65" fill="none" stroke="black" stroke-width="1" /><circle cx="0" cy="-25" r="20" fill="none" stroke="black" stroke-width="1" /><path d="M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90" fill="none" stroke="black" stroke-width="1" /><path d="M -20 90 A 20 20 0 0 1 20 90" fill="none" stroke="black" stroke-width="1" /><line x1="-12" y1="-88" x2="12" y2="-88" stroke="black" stroke-width="1" /><circle cx="0" cy="-84" r="5" stroke="red" stroke-width="1" fill="none" /><line x1="0" y1="60" x2="90" y2="-80" stroke="black" stroke-width="2" stroke-dasharray="4" marker-end="url(#arrowhead)" /><circle cx="0" cy="60" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="60" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="60" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><circle cx="0" cy="60" r="4" fill="orange" stroke="black" stroke-width="1" transform="translate(10, -10)" /><circle cx="90" cy="-80" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="90" cy="-80" r="10" fill="white" stroke="black" stroke-width="2" /><text x="90" y="-80" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">2</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>

### Screen Action

screen represents the player's screen. Specify the receiving player as follows.

```playbook
players = {p1, p2}

state = {
  baller = p1,
  position = {
    p1 = (0, 60),
    p2 = (50, 0),
  },
}

action = {
  move = {
    p1 -> (0, 0),
  }

  screen = {
    p2 -> p1:middle,
  }
}
```

<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-105" y="-105" width="210" height="210" fill="white" /><rect x="-100" y="-90" width="200" height="180" fill="white" stroke="black" stroke-width="2" /><rect x="-20" y="-90" width="40" height="65" fill="none" stroke="black" stroke-width="1" /><circle cx="0" cy="-25" r="20" fill="none" stroke="black" stroke-width="1" /><path d="M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90" fill="none" stroke="black" stroke-width="1" /><path d="M -20 90 A 20 20 0 0 1 20 90" fill="none" stroke="black" stroke-width="1" /><line x1="-12" y1="-88" x2="12" y2="-88" stroke="black" stroke-width="1" /><circle cx="0" cy="-84" r="5" stroke="red" stroke-width="1" fill="none" /><line x1="0" y1="60" x2="0" y2="0" stroke="black" stroke-width="2" marker-end="url(#arrowhead)" /><line x1="50" y1="0" x2="4.287464628562721" y2="27.427521222862367" stroke="black" stroke-width="2" /><line x1="8.14618279426917" y1="33.85871816570645" x2="0.4287464628562718" y2="20.996324280018285" stroke="black" stroke-width="2" /><circle cx="0" cy="60" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><circle cx="0" cy="0" r="4" fill="orange" stroke="black" stroke-width="1" transform="translate(10, -10)" /><circle cx="50" cy="0" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="50" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="50" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">2</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>

#### Arc Screen

This support Arc Line, too.

```playbook
players = {p1, p2, p3}

state = {
  baller = p1,
  position = {
    p1 = (0, 60),
    p2 = (50, 0),
    p3 = (-50, 0),
  },
}

action = {
  move = {
    p1 -> (0, 0),
  }

  screen = {
    p2 -> p1:middle,
    p3 ~[r]> (0, 50),
  }
}
```

<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-105" y="-105" width="210" height="210" fill="white" /><rect x="-100" y="-90" width="200" height="180" fill="white" stroke="black" stroke-width="2" /><rect x="-20" y="-90" width="40" height="65" fill="none" stroke="black" stroke-width="1" /><circle cx="0" cy="-25" r="20" fill="none" stroke="black" stroke-width="1" /><path d="M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90" fill="none" stroke="black" stroke-width="1" /><path d="M -20 90 A 20 20 0 0 1 20 90" fill="none" stroke="black" stroke-width="1" /><line x1="-12" y1="-88" x2="12" y2="-88" stroke="black" stroke-width="1" /><circle cx="0" cy="-84" r="5" stroke="red" stroke-width="1" fill="none" /><line x1="0" y1="60" x2="0" y2="0" stroke="black" stroke-width="2" marker-end="url(#arrowhead)" /><line x1="50" y1="0" x2="4.287464628562721" y2="27.427521222862367" stroke="black" stroke-width="2" /><line x1="8.14618279426917" y1="33.85871816570645" x2="0.4287464628562718" y2="20.996324280018285" stroke="black" stroke-width="2" /><path d="M -50 0 Q -40.707106781186546 37.17157287525381 -3.5355339059327373 46.46446609406726" stroke="black" stroke-width="2" fill="none" /><line x1="1.7677669529663684" y1="41.161165235168156" x2="-8.838834764831843" y2="51.76776695296637" stroke="black" stroke-width="2" /><circle cx="0" cy="60" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><circle cx="0" cy="0" r="4" fill="orange" stroke="black" stroke-width="1" transform="translate(10, -10)" /><circle cx="50" cy="0" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="50" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="50" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">2</text><circle cx="-50" cy="0" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="-50" cy="0" r="10" fill="white" stroke="black" stroke-width="2" /><text x="-50" y="0" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">3</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>
