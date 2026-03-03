---
name: rust-safety
description: Guidance for writing secure and robust Rust code. Use when performing security audits, implementing sensitive logic, or reviewing Rust code for common vulnerabilities like memory safety, integer overflows, and dependency risks.
---

# Rust Safety

## Overview
This skill provides a checklist and best practices for ensuring Rust code is secure, robust, and free from common vulnerabilities. While Rust provides strong safety guarantees, logic errors, `unsafe` blocks, and dependency risks still require careful attention.

## Security Checklist

### 1. Memory Safety & `unsafe`
- **Minimize `unsafe`**: Always look for safe alternatives first.
- **Audit `unsafe` blocks**: Ensure they uphold safety invariants. Use `miri` for testing.
- **Avoid raw pointers**: Prefer references or smart pointers (`Box`, `Rc`, `Arc`).

### 2. Integer & Arithmetic Safety
- **Overflows**: Be cautious with arithmetic operations. Use `checked_`, `saturating_`, or `wrapping_` methods where appropriate.
- **Casting**: Use `try_from` instead of `as` when lossy conversions could lead to security issues.

### 3. Error Handling
- **No `unwrap()`/`expect()`**: Use proper error handling (`Result`, `Option`) in production code. Avoid panicking in libraries.
- **Sensitive Data**: Ensure error messages do not leak sensitive information (e.g., file paths, internal state).

### 4. Dependency Management
- **Audit dependencies**: Use `cargo audit` to check for known vulnerabilities.
- **Pin versions**: Be careful with `*` or wide version ranges in `Cargo.toml`.
- **Minimal dependencies**: Only include what is strictly necessary.

### 5. Concurrency
- **Deadlocks**: Ensure lock ordering is consistent.
- **Shared State**: Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` correctly, and keep critical sections small.

## Tools
- `cargo audit`: Audit dependencies for crates.io vulnerabilities.
- `cargo clippy`: Linter for catching common mistakes and non-idiomatic code.
- `miri`: An interpreter for Rust's mid-level intermediate representation, capable of detecting many types of undefined behavior.
