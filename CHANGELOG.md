# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Property-based tests using proptest for schema parser and circuit validation
- Fuzz-like integration tests for schema parser edge cases
- `docs/FUZZING.md` documenting property testing and fuzzing approach
- `docs/WASM.md` documenting WebAssembly verification path and current limitations
- `examples/verify_only.rs` demonstrating host-side verify-only workflow

### Changed

- Expanded test coverage with random valid/invalid schema generation

## [0.1.0] - 2026-09-03

### Added

- Initial release of FormProof
- **Schema parsing**: JSON Schema v0 subset (objects ≤8 props, integer/enum/string/bytes32)
- **Circuit compilation**: Schema to Groth16 (BN254) R1CS circuit
- **Proving**: Generate zkSNARK proofs from private witness data
- **Verification**: Verify proofs against public commitment
- **CLI**: `formproof compile`, `prove`, `verify`, `info` commands
- **Documentation**: README with examples, `docs/SCHEMA_V0.md` specification
- **Tests**: 32 tests including 3 golden proofs and rejection corpus
- **Benchmarks**: Criterion benchmarks for prove/verify times
- **Example**: MCP tool host integration example

### Supported Schema Types

- `integer` with optional `minimum`/`maximum` (u64)
- `enum` with up to 8 string variants
- `string` with `maxLength` up to 64
- `bytes32` for 32-byte binary data

### Constraints

- Maximum 8 properties per object
- Required field validation
- All values in witness committed via SHA-256

### Known Limitations

- No nested objects or arrays
- No regex patterns
- Schema is public (only payload is private)
- No `$ref` or schema composition
