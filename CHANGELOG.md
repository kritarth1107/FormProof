# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-09-09

Initial public release of FormProof.

### Core Library

- **Schema parsing**: JSON Schema v0 subset (objects ≤8 props, integer/enum/string/bytes32)
- **Circuit compilation**: Schema → Groth16 (BN254) R1CS circuit
- **Proving**: Generate zkSNARK proofs from private witness data
- **Verification**: Verify proofs against public commitment
- **Portable proof packages**: `ProofPackage` bundles proof + commitment + schema fingerprint
  - JSON serialization for easy transport between hosts
  - Schema fingerprint (SHA-256) for verifier schema binding
  - `Proof::to_package()` convenience method
- Re-export schema limit constants (`MAX_PROPERTIES`, `MAX_ENUM_VARIANTS`, `MAX_STRING_LENGTH`) in public API

### CLI

- `formproof compile` — compile schema to proving/verifying keys
- `formproof prove` — generate proof from witness
- `formproof verify` — verify proof against commitment
- `formproof info` — show schema constraints and fingerprint
- `formproof package-build` — build portable proof package
- `formproof package-verify` — verify package with fingerprint validation
- `--compact` flag for minified JSON output

### MCP Policy Schemas

Ready-to-use policy schemas in `schemas/`:

- `refund.json` — refund amount ≤$50, valid currency
- `age_gate.json` — age ≥18 with region
- `access_country.json` — country allowlist + tier
- `spend_cap.json` — spend limit ≤$100, multi-currency
- `session_ttl.json` — session TTL with tier
- `rate_limit.json` — requests_per_window, window_secs, tier
- `tool_allowlist.json` — tool_name enum, max_args, scope
- `quota_budget.json` — budget_units, period, soft_cap
- `model_route.json` — model_id, max_tokens, priority, temperature_class
- `data_residency.json` — region, storage_class, retention_days, cross_border
- `purpose_bind.json` — purpose, legal_basis, max_secondary_uses
- `human_gate.json` — action_class, approval_tier, max_auto_approve_secs

### Documentation

- `docs/SCHEMA_V0.md` — Frozen schema specification
- `docs/THREAT_MODEL.md` — Security model and trust assumptions
- `docs/HOST_INTEGRATION.md` — MCP/tool host verify-only integration
- `docs/PROOF_PACKAGE.md` — Portable proof package format
- `docs/DATA_RESIDENCY.md` — Data residency / retention policy guide
- `docs/PURPOSE_BIND.md` — Purpose limitation / legal basis policy guide
- `docs/HUMAN_GATE.md` — Human-in-the-loop approval policy guide
- `docs/WASM.md` — WebAssembly verification path and caveats
- `docs/FUZZING.md` — Property testing and fuzzing approach
- `docs/RELEASE.md` — Release preparation checklist
- `SECURITY.md` — Supported versions and vulnerability reporting

### Examples

- `mcp_tool_host.rs` — MCP integration example
- `verify_only.rs` — Host-side verify-only workflow
- `proof_package_demo.rs` — Proof package workflow
- `spend_cap_demo.rs`, `session_ttl_demo.rs`, `quota_budget_demo.rs`
- `model_route_demo.rs`, `rate_limit_demo.rs`, `tool_allowlist_demo.rs`
- `age_gate_demo.rs`, `data_residency_demo.rs`, `purpose_bind_demo.rs`
- `human_gate_demo.rs` — Human-in-the-loop approval demonstration

### Testing

- Golden proof tests (3 schemas)
- Property-based tests using proptest
- Fuzz-like integration tests for schema parser
- Boundary tests for all 12 policy schemas
- CLI end-to-end tests for all commands
- Schema fixture validation tests

### Benchmarks

Criterion benchmarks for all 13 policy schemas:
- Core: refund, user, token
- MCP: rate_limit, session_ttl, age_gate, tool_allowlist, quota_budget,
  model_route, data_residency, purpose_bind, human_gate

### CI

- Format check (`cargo fmt --check`)
- Clippy lint (`cargo clippy --all-targets -- -D warnings`)
- Test suite (`cargo test --all-features`)
- Documentation build with warnings
- Schema JSON syntax validation
- Schema parsing validation
- Security audit (cargo-audit)
- Examples smoke test
- CLI end-to-end tests

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
- WASM browser verification not yet validated (deferred)
- Manual MCP host integration testing deferred
