# Arc Line Support Todo

- [x] Analyze codebase for AST, Parser, and Renderer changes
- [x] Update AST to support Arc paths (Curve direction)
- [x] Update Parser to handle `~>`, `~[left]>`, `~[right]>` syntax
- [x] Update IR to propagate curve information
- [x] Update Renderer to draw curves (Bezier/Arc)
- [x] Create test/fixture for Arc lines
- [x] Verify output

## Curve Factor Support
- [x] Update Lexer to read content inside `~[...]` more flexibly (allow `:0.1`)
- [x] Update AST `CurveDirection` to hold factor value
- [x] Update Parser to parse `dir:factor` string
- [x] Update IR and Renderer to use dynamic factor
- [x] Test new syntax
