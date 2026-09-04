# Contributing to FormProof

Thank you for your interest in contributing to FormProof! This document provides guidelines for contributing.

## Development Setup

### Prerequisites

- Rust 1.70+ (stable)
- Cargo

### Building

```bash
git clone https://github.com/kritarth1107/FormProof
cd FormProof
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Running Benchmarks

```bash
cargo bench --bench proof_bench
```

This benchmarks prove/verify times for the 3 golden schemas (refund, user, token).

### Running the Example

```bash
cargo run --example mcp_tool_host
```

This demonstrates a tool host verifying a refund request without seeing the actual amount.

### Building Documentation

```bash
cargo doc --no-deps --open
```

## Code Style

### Formatting

Always run `cargo fmt` before committing:

```bash
cargo fmt --all
```

### Linting

Ensure clippy passes with no warnings:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

All public items must have rustdoc comments. The library has `#![warn(missing_docs)]` enabled.

## Commit Guidelines

### Small Logical Commits

Prefer many small, logical commits over one large commit. Each commit should:

- Do one thing
- Have a clear, descriptive message
- Pass all tests

### Commit Messages

- Use imperative mood ("Add feature" not "Added feature")
- First line: 50 chars or less, summarize the change
- Body: explain what and why, not how

Good:
```
Add bytes32 support to schema parser

Support fixed 32-byte binary data for hashes and identifiers.
Uses format: "bytes32" in JSON Schema string type.
```

Bad:
```
updated stuff
```

### No AI Co-Author Trailers

Do not add `Co-authored-by` trailers for AI assistants (Cursor, Copilot, etc.) or any other co-authors. Commits should be authored solely by the contributor.

## Pull Requests

### Before Submitting

1. Run `cargo fmt --all`
2. Run `cargo clippy --all-targets --all-features -- -D warnings`
3. Run `cargo test`
4. Run `cargo doc --no-deps` (should have no warnings)

### PR Guidelines

- Keep PRs focused on a single concern
- Reference any related issues
- Update documentation if needed
- Add tests for new functionality

## Scope

FormProof v0 has intentionally limited scope. Please keep contributions within these bounds:

### In Scope

- Bug fixes
- Documentation improvements
- Test coverage
- Performance optimizations
- Better error messages

### Out of Scope (for v0)

- Nested objects or arrays
- Regex patterns
- New property types beyond u64/enum/string/bytes32
- Schema privacy features
- Alternative proving systems

If you want to work on out-of-scope features, please open an issue first to discuss.

## Questions?

Open an issue for questions or discussion.
