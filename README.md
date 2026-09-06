# FormProof

**Tool hosts shouldn't have to see the refund amount to know it's ≤ $50 and a valid currency.**

FormProof is a Rust library that compiles a frozen JSON Schema subset into a Groth16 (BN254) circuit. An MCP/tool host can verify that private JSON data satisfies schema constraints—required keys present, enums in set, integers in range, string lengths bounded—**without learning the actual field values**.

## What It Is

A compiler + prover/verifier pipeline:

1. **Schema** → Define constraints (max amounts, valid enums, required fields)
2. **Compile** → Generate Groth16 proving/verifying keys for that schema
3. **Prove** → Prover creates a proof from private JSON data
4. **Verify** → Verifier checks proof against public commitment (never sees values)

## 30-Second Example

```rust
use formproof::{FormProofSchema, CompiledSchema, Witness, Proof, verify};

// Schema: refund amount ≤50, valid currency
let schema = FormProofSchema::from_json(r#"{
    "type": "object",
    "properties": {
        "amount": { "type": "integer", "minimum": 0, "maximum": 50 },
        "currency": { "enum": ["USD", "EUR", "GBP"] }
    },
    "required": ["amount", "currency"]
}"#).unwrap();

// Compile schema to circuit
let compiled = CompiledSchema::compile(schema).unwrap();

// Prover: create witness with actual values (PRIVATE)
let mut witness = Witness::new();
witness.set_u64("amount", 25);  // Actual amount - verifier never sees this
witness.set_enum("currency", "USD");

// Generate proof
let proof = Proof::create(&compiled, &witness).unwrap();

// Verifier: only sees commitment hash, proves amount ≤50 + valid currency
assert!(verify(&compiled, &proof).unwrap());
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
formproof = { git = "https://github.com/kritarth1107/FormProof" }
```

Or clone and build:

```bash
git clone https://github.com/kritarth1107/FormProof
cd FormProof
cargo build --release
```

## CLI Usage

```bash
# Show schema constraints and fingerprint
formproof info --schema schemas/refund.json

# Compile schema to proving/verifying keys
formproof compile --schema schemas/refund.json --output ./keys

# Generate proof from private witness
formproof prove --schema schemas/refund.json --proving-key keys/proving_key.bin \
    --witness witness.json --output proof.bin

# Verify proof (only needs commitment, never sees actual values)
formproof verify --schema schemas/refund.json --verifying-key keys/verifying_key.bin \
    --proof proof.bin --commitment <hex-commitment>
```

### Proof Packages

For easier transport, bundle proof + commitment + schema fingerprint into a single JSON file:

```bash
# Build a portable proof package
formproof package-build --schema schemas/refund.json \
    --proving-key keys/proving_key.bin \
    --witness witness.json --output proof_package.json

# Verify a package (checks fingerprint + proof)
formproof package-verify --schema schemas/refund.json \
    --verifying-key keys/verifying_key.bin \
    --package proof_package.json
```

See [docs/PROOF_PACKAGE.md](docs/PROOF_PACKAGE.md) for format details.

Ready-made policies live in [`schemas/`](schemas/) (`refund`, `age_gate`, `access_country`, `spend_cap`, `session_ttl`, `rate_limit`, `tool_allowlist`).

### Example Files

**refund.json** (schema):
```json
{
    "type": "object",
    "properties": {
        "amount": { "type": "integer", "minimum": 0, "maximum": 50 },
        "currency": { "enum": ["USD", "EUR", "GBP"] }
    },
    "required": ["amount", "currency"]
}
```

**witness.json** (private data):
```json
{
    "amount": 25,
    "currency": "USD"
}
```

## v0 Scope (Honest Limitations)

This is an early version with intentionally limited scope.

- [docs/SCHEMA_V0.md](docs/SCHEMA_V0.md) — Complete frozen schema specification
- [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) — Security model and trust assumptions
- [docs/HOST_INTEGRATION.md](docs/HOST_INTEGRATION.md) — MCP/tool host verify-only integration
- [docs/WASM.md](docs/WASM.md) — WebAssembly verification path and caveats
- [docs/RELEASE.md](docs/RELEASE.md) — Release preparation checklist
- [SECURITY.md](SECURITY.md) — Supported versions and vulnerability reporting

### Supported
- Objects with **≤8 properties**
- Types:
  - `integer` (u64) with optional `minimum`/`maximum`
  - `enum` with **≤8 string variants**
  - `string` with `maxLength` **≤64**
  - `bytes32` (32-byte hex strings, via `format: "bytes32"`)
- `required` field validation
- Payload hashes to public commitment

### NOT Supported (Don't Expect These)
- ❌ Arbitrary JSON Schema (only the subset above)
- ❌ Regex patterns
- ❌ Nested objects or arrays
- ❌ Recursive schemas
- ❌ zkML or machine learning integration
- ❌ Hiding the schema/policy (schema is public; only payload is private)
- ❌ Large schemas (>8 properties)

## How It Works

1. **Schema Parsing**: JSON Schema subset → internal constraint representation
2. **Circuit Compilation**: Constraints → R1CS circuit → Groth16 keys (BN254 curve)
3. **Witness Commitment**: Private data → SHA256 hash (public commitment)
4. **Proving**: Witness + proving key → zkSNARK proof
5. **Verification**: Proof + commitment + verifying key → accept/reject

The verifier learns nothing about the actual values—only that they satisfy the schema constraints.

## Why This Exists

MCP tool hosts often need to validate that agent requests meet policies without seeing sensitive data:

- **Refunds**: Verify amount ≤ $50 without seeing the exact amount
- **Access control**: Verify user is in allowed country list without revealing location
- **Age verification**: Prove age ≥ 18 without revealing exact age

This **complements** (does not replace) OAuth and standard authorization. OAuth proves *who* is making a request; FormProof proves the request *content* satisfies constraints.

## Architecture

```
formproof/           # Core library
├── src/
│   ├── schema.rs    # JSON Schema subset parser
│   ├── circuit.rs   # R1CS circuit generation
│   ├── prove.rs     # Groth16 proving
│   ├── verify.rs    # Groth16 verification
│   ├── package.rs   # Portable proof packages
│   └── lib.rs       # Public API
├── benches/
│   └── proof_bench.rs  # Criterion benchmarks
├── examples/
│   ├── mcp_tool_host.rs     # MCP integration example
│   ├── verify_only.rs       # Host-side verify-only workflow
│   ├── spend_cap_demo.rs    # Spend cap policy demonstration
│   └── proof_package_demo.rs # Proof package workflow
└── tests/
    └── golden.rs    # Golden proofs + rejection corpus

formproof-cli/       # CLI binary (compile, prove, verify, package-build, package-verify)
└── src/main.rs

schemas/             # Reusable v0 policy fixtures
├── refund.json
├── age_gate.json
├── access_country.json
├── spend_cap.json
├── session_ttl.json
├── rate_limit.json      # MCP rate-limit policy
└── tool_allowlist.json  # MCP tool access policy

docs/
├── SCHEMA_V0.md          # Frozen schema specification
├── THREAT_MODEL.md       # Security model and trust assumptions
├── HOST_INTEGRATION.md   # Host verify-only integration
├── PROOF_PACKAGE.md      # Portable proof package format
├── WASM.md               # WebAssembly verification notes
└── RELEASE.md            # Release preparation checklist
```

## Testing

```bash
# Run all tests
cargo test

# Run golden proof tests specifically
cargo test --test golden

# Run property tests
cargo test --test proptest_schema

# Run fuzz-like tests
cargo test --test fuzz_schema

# Run with output
cargo test -- --nocapture
```

Tests include:
- 3 golden proofs that verify
- Rejection corpus: wrong enum, out of range, missing required, invalid values
- Edge cases: boundary values, optional fields, bytes32
- Property tests: random valid/invalid schemas, witness/circuit satisfaction
- Fuzz tests: arbitrary JSON input, malformed schemas, edge cases

See [docs/FUZZING.md](docs/FUZZING.md) for property testing and fuzzing details.

## Benchmarks

Measured on the 3 golden schemas using criterion (release build):

| Schema | Prove Time | Verify Time |
|--------|------------|-------------|
| Refund (2 props: amount, currency) | 2.35 ms | 1.13 ms |
| User (3 props: age, country, name) | 2.28 ms | 1.15 ms |
| Token (2 props: token_id, balance) | 5.82 ms | 1.14 ms |

Run benchmarks yourself:

```bash
cargo bench --bench proof_bench
```

## Performance Notes

- Circuit compilation (key generation) is slow (~seconds for small schemas)
- Proving time scales with schema complexity
- Verification is fast (constant time)
- For CI, tests use small circuits to keep runtime reasonable

## License

MIT License - see [LICENSE](LICENSE)

## CI

GitHub Actions runs on every PR:

- **Format**: `cargo fmt --check`
- **Clippy**: `cargo clippy --all-targets -- -D warnings`
- **Test**: `cargo test`
- **Docs**: `cargo doc --no-deps -p formproof` with `-D warnings`
- **Build**: Release build and CLI check

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

Quick checklist:
1. Keep changes within v0 scope
2. Add tests for new functionality
3. Run `cargo fmt` and `cargo clippy` before submitting
4. No AI co-author trailers in commits
