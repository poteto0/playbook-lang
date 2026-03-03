# Security Review: `core` Crate

## Overview
The `core` crate contains the Lexer, Parser, IR Generator, and Renderer for the Playbook Language. It is the most critical component for security as it handles untrusted input.

## Audit Results

### 1. Memory Safety & `unsafe`
- **Findings**: No `unsafe` blocks were found in the codebase.
- **Status**: ✅ **Pass**
- **Recommendation**: Continue to avoid `unsafe`.

### 2. Error Handling & Panics
- **Findings**:
    - `lexer/mod.rs`: Uses `parse().unwrap_or(0.0)` for number parsing. This is safe as it provides a fallback.
    - `ir/generator.rs`: Uses `unwrap_or` for retrieving player positions.
        - *Risk*: If logic is incorrect, it silently falls back to `(0.0, 0.0)` or previous positions, which might lead to confusing visual output rather than an explicit error.
    - **Test Code**: Extensive use of `unwrap()` and `expect()` in tests, which is acceptable.
- **Status**: ⚠️ **Review Needed** (Logic Safety)
- **Recommendation**:
    - Review `ir/generator.rs` to ensure `unwrap_or` usage doesn't mask logic bugs. Consider returning `Result` if a missing position is a critical inconsistency.

### 3. DSL Security (Parser Robustness)
- **Findings**:
    - **Recursion**: The parser (`parser/mod.rs`) uses a recursive descent approach for sections (e.g., `parse_actions_section`).
        - *Risk*: While the current grammar is shallow, deep nesting of future structures could lead to stack overflow.
    - **Input Size**: No explicit limit on input size.
        - *Risk*: Large inputs could cause memory exhaustion (DoS), especially in WASM environments.
- **Status**: ⚠️ **Improvement Needed**
- **Recommendation**:
    - Implement a maximum recursion depth check if the grammar becomes more complex.
    - Consider adding a "max input size" check in `Lexer::new` or `Parser::new` to reject excessively large files early.

### 4. Integer & Arithmetic Safety
- **Findings**: Standard arithmetic is used.
- **Status**: ✅ **Pass**
- **Recommendation**: Keep monitoring for potential overflows if complex calculations (like curve geometry) are added.

## Conclusion
The `core` crate is generally safe from memory corruption but has potential logic risks in IR generation and DoS risks from unbounded input processing.
