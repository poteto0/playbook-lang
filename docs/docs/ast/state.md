---
sidebar_position: 2
---

# State

The state section specifies the initial state. It sets the player's position and designates the ball carrier.

```playbook
players = {p1, p2}

state = {
  baller = p1,
  position = {
    p1 = (0, 60),
    p2 = (90, -80),
  },
}
```

<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-105" y="-105" width="210" height="210" fill="white" /><rect x="-100" y="-90" width="200" height="180" fill="white" stroke="black" stroke-width="2" /><rect x="-20" y="-90" width="40" height="65" fill="none" stroke="black" stroke-width="1" /><circle cx="0" cy="-25" r="20" fill="none" stroke="black" stroke-width="1" /><path d="M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90" fill="none" stroke="black" stroke-width="1" /><path d="M -20 90 A 20 20 0 0 1 20 90" fill="none" stroke="black" stroke-width="1" /><line x1="-12" y1="-88" x2="12" y2="-88" stroke="black" stroke-width="1" /><circle cx="0" cy="-84" r="5" stroke="red" stroke-width="1" fill="none" /><circle cx="0" cy="60" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="60" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="60" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><circle cx="0" cy="60" r="4" fill="orange" stroke="black" stroke-width="1" transform="translate(10, -10)" /><circle cx="90" cy="-80" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="90" cy="-80" r="10" fill="white" stroke="black" stroke-width="2" /><text x="90" y="-80" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">2</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>
