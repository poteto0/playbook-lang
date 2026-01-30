---
sidebar_position: 2
---

# Best Practices

## Sanitize the generated SVG

playbook-lang is strictly a compiler and is not intended for web use. Maintainers will address security incidents to the extent possible, but make no guarantees. Sanitize generated SVGs in case the compiled output contains vulnerabilities.

Recommended options below

```ts title="sanitize_option.ts"
export const svgSanitizeOptions = {
  allowedTags: [
    "svg",
    "path",
    "defs",
    "g",
    "circle",
    "rect",
    "line",
    "polygon",
    "polyline",
    "ellipse",
    "text",
    "marker",
  ],
  allowedAttributes: {
    svg: ["viewBox", "width", "height", "xmlns"],
    path: ["d", "fill", "stroke", "stroke-width", "marker-end"],
    g: ["transform"],
    circle: ["cx", "cy", "r", "fill", "stroke", "stroke-width", "transform"],
    rect: ["x", "y", "width", "height", "fill", "stroke", "stroke-width"],
    line: [
      "x1",
      "y1",
      "x2",
      "y2",
      "fill",
      "stroke",
      "stroke-width",
      "stroke-dasharray",
      "marker-end",
    ],
    polygon: ["points", "fill"],
    polyline: ["points"],
    ellipse: ["cx", "cy", "rx", "ry"],
    text: [
      "x",
      "y",
      "font-size",
      "text-anchor",
      "dominant-baseline",
      "font-family",
    ],
    marker: ["id", "refX", "refY", "markerWidth", "markerHeight", "orient"],
    /* disable others */
    "*": [],
  },
  parser: {
    lowerCaseAttributeNames: false,
  },
};
```

Renders the string with sanitization options.

```tsx title="App.tsx"
import sanitize from "sanitize-html";

function App() {
  /* ... */

  return (
    <div className="app-container">
      /* ... */
      <div
        id="output"
        dangerouslySetInnerHTML={{
          __html: sanitize(renderedOutput, svgSanitizeOptions),
        }}
      ></div>
    </div>
  );
}
```
