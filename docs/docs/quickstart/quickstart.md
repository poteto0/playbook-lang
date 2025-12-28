---
sidebar_position: 1
---

# Quick Start

write a playbook `input.playbook`.

```playbook
players = { p1, p2, p3, p4, p5 }

state = {
  baller = p1,
  position = {
    p1 = (0, 60),
    p2 = (90, -80),
    p3 = (-90, -80),
    p4 = (50, -10),
    p5 = (-50, -10),
  },
}

action = {
  move = {
    p2 -> (70, 20),
    p3 -> (0, -80),
  },
  screen = {
    p4 -> p2:middle,
    p5 -> p3:middle,
  },
  pass = {
    p1 -> p2:after,
  },
}
```

run cli

```bash
./build/playbook-cli input.playbook --output output.svg
```

output

<svg width="500" height="500" viewBox="-105 -105 210 210" xmlns="http://www.w3.org/2000/svg"><rect x="-105" y="-105" width="210" height="210" fill="white" /><rect x="-100" y="-90" width="200" height="180" fill="white" stroke="black" stroke-width="2" /><rect x="-20" y="-90" width="40" height="65" fill="none" stroke="black" stroke-width="1" /><circle cx="0" cy="-25" r="20" fill="none" stroke="black" stroke-width="1" /><path d="M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90" fill="none" stroke="black" stroke-width="1" /><path d="M -20 90 A 20 20 0 0 1 20 90" fill="none" stroke="black" stroke-width="1" /><line x1="-12" y1="-88" x2="12" y2="-88" stroke="black" stroke-width="1" /><circle cx="0" cy="-84" r="5" stroke="red" stroke-width="1" fill="none" /><line x1="90" y1="-80" x2="70" y2="20" stroke="black" stroke-width="2" marker-end="url(#arrowhead)" /><line x1="-90" y1="-80" x2="0" y2="-80" stroke="black" stroke-width="2" marker-end="url(#arrowhead)" /><line x1="0" y1="60" x2="70" y2="20" stroke="black" stroke-width="2" stroke-dasharray="4" marker-end="url(#arrowhead)" /><line x1="50" y1="-10" x2="75.83974852831078" y2="-27.226499018873852" stroke="black" stroke-width="2" /><line x1="71.67949705662156" y1="-33.46687622640768" x2="80" y2="-20.986121811340023" stroke="black" stroke-width="2" /><line x1="-50" y1="-10" x2="-45.35623524993955" y2="-75.01270650084632" stroke="black" stroke-width="2" /><line x1="-52.837175498670064" y1="-75.54705937575564" x2="-37.87529500120904" y2="-74.478353625937" stroke="black" stroke-width="2" /><circle cx="0" cy="60" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="60" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="60" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">1</text><circle cx="0" cy="60" r="4" fill="orange" stroke="black" stroke-width="1" transform="translate(10, -10)" /><circle cx="90" cy="-80" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="70" cy="20" r="10" fill="white" stroke="black" stroke-width="2" /><text x="70" y="20" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">2</text><circle cx="-90" cy="-80" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="0" cy="-80" r="10" fill="white" stroke="black" stroke-width="2" /><text x="0" y="-80" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">3</text><circle cx="50" cy="-10" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="50" cy="-10" r="10" fill="white" stroke="black" stroke-width="2" /><text x="50" y="-10" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">4</text><circle cx="-50" cy="-10" r="8" fill="white" stroke="gray" stroke-width="1" opacity="0.3" /><circle cx="-50" cy="-10" r="10" fill="white" stroke="black" stroke-width="2" /><text x="-50" y="-10" font-size="12" text-anchor="middle" dominant-baseline="central" font-family="Arial">5</text><defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs></svg>
