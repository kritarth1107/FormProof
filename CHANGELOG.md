# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `schemas/model_route.json`: MCP model-routing policy with model_id enum (8 models), max_tokens (1–128000), priority, optional temperature_class
- `formproof/tests/model_route_bounds.rs` for model routing boundary validation
- `formproof/examples/quota_budget_demo.rs` demonstrating quota/budget prove/verify workflow
- `formproof/examples/model_route_demo.rs` demonstrating model routing prove/verify workflow
- Criterion benches for `quota_budget` and `model_route` prove/verify
- CI: `schema-parse` job verifying all schemas compile (not just valid JSON)
- CI: `security-audit` job using rustsec/audit-check for dependency vulnerabilities
- `docs/RELEASE.md`: detailed v0.1.0 pre-release checklist

### Previously Added

- `schemas/quota_budget.json`: MCP quota/budget policy with budget_units, period, optional soft_cap
- `formproof/tests/access_country_bounds.rs` for country/tier boundary validation
- `formproof/tests/quota_budget_bounds.rs` for budget units and period validation
- `formproof/examples/session_ttl_demo.rs` demonstrating session TTL prove/verify workflow
- Criterion benches for `age_gate` and `tool_allowlist` prove/verify
- `formproof/tests/session_ttl_bounds.rs` for session TTL boundary validation
- `formproof/tests/age_gate_bounds.rs` for age gate boundary validation
- Criterion benches for `rate_limit` and `session_ttl` prove/verify
- **Portable proof packages**: `ProofPackage` bundles proof + commitment + schema fingerprint
  - JSON serialization for easy transport between hosts
  - Schema fingerprint (SHA-256) for verifier schema binding
  - `Proof::to_package()` convenience method
  - `docs/PROOF_PACKAGE.md` documenting format, usage, and threat notes
- **CLI package commands**: `package-build` and `package-verify`
  - Build portable packages from witness files
  - Verify packages with fingerprint and proof validation
  - `--compact` flag for minified JSON output
  - `info` command now shows schema fingerprint
- **New MCP policy schemas**:
  - `schemas/rate_limit.json`: requests_per_window, window_secs, tier
  - `schemas/tool_allowlist.json`: tool_name enum, max_args, scope
- `examples/proof_package_demo.rs` demonstrating package workflow
- `formproof/tests/package_roundtrip.rs` for package create/verify tests
- `formproof/tests/policy_bounds.rs` for rate_limit and tool_allowlist bounds
- Property-based tests using proptest for schema parser and circuit validation
- Fuzz-like integration tests for schema parser edge cases
- `docs/FUZZING.md` documenting property testing and fuzzing approach
- `docs/WASM.md` documenting WebAssembly verification path and current limitations
- `docs/HOST_INTEGRATION.md` for MCP/tool host verify-only integration
- `docs/RELEASE.md` with release preparation checklist
- `examples/verify_only.rs` demonstrating host-side verify-only workflow
- `schemas/` fixtures: `refund.json`, `age_gate.json`, `access_country.json`, `spend_cap.json`
- `formproof/tests/schema_fixtures.rs` validating all schema fixtures parse and compile
- CI job to validate `schemas/*.json` syntax with jq
- `SECURITY.md` with supported versions and private disclosure contact
- `examples/spend_cap_demo.rs` demonstrating spend_cap policy verification
- `schemas/session_ttl.json` fixture for session TTL with tier policy
- `formproof/tests/spend_cap_bounds.rs` for boundary constraint validation

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
