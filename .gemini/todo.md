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

## Validation Enhancement
- [x] Lexer: Add `TokenKind::Error` and improve `read_inside_brackets` robustness (check unclosed bracket/newline)
- [x] Parser: Add length check for curve factor (max 3 chars)
- [x] Parser: Handle `TokenKind::Error` in `expect_arrow`
- [x] Parser: Support multiple errors

## Multiple Action Support
- [x] Update Lexer to support `actions` keyword, `[` and `]` symbols
- [x] Update AST to support `Vec<Action>` in `Playbook`
- [x] Update Parser to handle `actions = [ action = { ... }, ... ]` and single `action = { ... }`
- [x] Enforce max 3 actions in Parser
- [x] Update IR Generator to process actions sequentially, tracking positions and baller state across phases
- [x] Add unit tests for multiple action parsing and IR generation
- [x] Verify full pipeline with multiple actions

## Linter Implementation (New Crate)

- [x] **Create `linter` crate**: Initialize a new Rust library crate (`linter`) in the workspace.
- [x] **Configure Dependencies**: Add `playbook_lang_core`, `serde`, `serde-wasm-bindgen`, and `wasm-bindgen` to `linter/Cargo.toml`.
- [x] **Implement `LintDiagnostic`**: Define a struct for error messages (line, column, message, severity).
- [x] **Implement `lint` function**: Create a function that parses code using `core` and maps `ParseError` to `LintDiagnostic`.
- [x] **Expose to WASM**: Annotate the `lint` function with `#[wasm_bindgen]` for JS usage.
- [x] **Build WASM**: Compile the `linter` crate to WASM.
- [x] **Unit Testing**: Add tests for `lint` logic.

## CodeMirror Integration

- [x] **Install Dependency**: Add `pkg-linter` (local path) dependency to `examples/codemirror-demo`.
- [x] **Implement Linter Extension**: Create `src/linter.ts` in `examples/codemirror-demo` that calls `lint_playbook`.
- [x] **Register Linter**: Add the linter extension to the editor configuration in `examples/codemirror-demo/src/main.ts`.
- [x] **Verify**: Run the demo and check error reporting.

## Play Command Support (Final Positions)
- [x] Update `core` crate to support JSON serialization for final positions
- [x] Implement `Renderer::play` method in `core`
- [x] Export `play_playbook` in `core/src/lib.rs`
- [x] Add `play` subcommand to `cli`
- [x] Verify `play` command output with fixtures

