---
name: dsl-oss-security
description: Security analysis for Domain Specific Languages (DSLs) and Open Source Software (OSS). Use when auditing DSL implementations (parsers, interpreters, compilers) for sandbox escapes, DoS vulnerabilities, or supply chain risks in OSS projects.
---

# DSL & OSS Security

## Overview
This skill provides guidance for securing DSL implementations and maintaining high security standards in OSS projects. DSLs often introduce unique risks related to parsing and execution environments.

## DSL Security Patterns

### 1. Parser Robustness (DoS Prevention)
- **Recursion Limits**: Ensure parsers have a maximum depth to prevent stack overflow from deeply nested structures.
- **Input Size Limits**: Enforce maximum input sizes early in the pipeline.
- **Timeouts**: If parsing or interpretation can be slow, use timeouts.
- **Regular Expressions**: Avoid "Evil Regexes" that can lead to ReDoS (Regular Expression Denial of Service).

### 2. Sandbox & Execution Safety
- **No Arbitrary Execution**: Ensure the DSL cannot execute arbitrary system commands or access the filesystem unless explicitly intended and sandboxed.
- **Resource Accounting**: If the DSL is interpreted, track resource usage (CPU, memory) to prevent exhaustion.
- **Input Validation**: Strictly validate all external inputs before they reach the interpreter.

## OSS Security Best Practices

### 1. Supply Chain Security
- **Lockfiles**: Always commit `Cargo.lock` (for applications) or ensure reproducible builds.
- **CI/CD Security**: Use GitHub Actions with minimal permissions (`contents: read`). Use OIDC for cloud providers.
- **Dependency Review**: Review PRs that add or update dependencies.

### 2. Vulnerability Disclosure
- **Security Policy**: Include a `SECURITY.md` file with instructions on how to report vulnerabilities.
- **Private Reporting**: Use GitHub's private vulnerability reporting feature.

### 3. Integrity
- **Signed Commits**: Encourage developers to sign their commits.
- **Release Integrity**: Provide checksums (SHA-256) and signatures for binary releases.
