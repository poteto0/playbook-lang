# Security Review: `linter` Crate

## Overview
The `linter` crate provides a WASM-compatible interface for analyzing `.playbook` code and returning diagnostics. It is primarily used in web-based editors (like CodeMirror).

## Audit Results

### 1. Memory Safety & WASM Boundary
- **Findings**:
    - **`unsafe`**: None found.
    - **WASM Interop**: Uses `serde-wasm-bindgen` to serialize Rust structs (`LintDiagnostic`) into JavaScript objects.
        - `lint_playbook` returns `Result<JsValue, JsValue>`, which handles serialization errors gracefully.
    - **Panic Safety**: The `lint_playbook` function does not contain `unwrap()` or `expect()`, making it robust against crashes in the browser.
- **Status**: ✅ **Pass**
- **Recommendation**: Ensure `playbook_lang_core` dependencies (parser/lexer) do not panic, as a panic in WASM is harder to debug.

### 2. Logic & Correctness
- **Findings**:
    - Maps `ParseError` variants to `LintDiagnostic` structures.
    - `ParseError::UnexpectedEOF` is handled with a placeholder location (`line: 0, column: 0`).
        - *Improvement*: Could calculate the last valid position or the end of the input string for better UX.
    - **Severity**: Hardcoded to "error".
        - *Future*: Consider adding "warning" or "info" levels.
- **Status**: ✅ **Pass** (with minor improvement suggestions)

### 3. Performance & DoS
- **Findings**:
    - **Input Processing**: The linter runs synchronously on the main thread (in JS).
        - *Risk*: Large inputs could freeze the UI.
    - **Recursion**: Inherits potential recursion risks from `core`'s parser.
- **Status**: ⚠️ **Performance Warning**
- **Recommendation**:
    - For large files, consider running the linter in a Web Worker to avoid blocking the UI thread.
    - Implement a timeout or input size check if dealing with arbitrary user input.

## Conclusion
The `linter` crate is well-structured for WASM use, with safe error handling and no critical vulnerabilities found.
