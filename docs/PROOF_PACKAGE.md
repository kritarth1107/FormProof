# Proof Packages

A proof package bundles everything needed to verify a FormProof proof into a single portable JSON artifact:

- **Proof bytes** (hex-encoded Groth16 proof)
- **Commitment** (hex-encoded 32-byte SHA-256 commitment)
- **Schema fingerprint** (hex-encoded SHA-256 of normalized schema JSON)

## Why Packages?

Without packages, verifying a proof requires:
1. The proof file (binary)
2. The commitment (hex string)
3. Knowledge of which schema was used

With packages, hosts can pass around a single JSON file. The fingerprint lets the verifier confirm the proof was generated for the expected schema.

## Format

```json
{
  "version": 1,
  "proof_hex": "6a7a4ee5...05050b01",
  "commitment_hex": "9dbbb4f301ad302a...c6ebb20153e5c94f",
  "schema_fingerprint": "6ff69fec...53e5c94f"
}
```

| Field | Description |
|-------|-------------|
| `version` | Package format version (currently `1`) |
| `proof_hex` | Hex-encoded serialized Groth16 proof (~192 bytes decoded) |
| `commitment_hex` | Hex-encoded 32-byte commitment to witness data |
| `schema_fingerprint` | SHA-256 of the schema's normalized JSON (via `to_json()`) |

## CLI Usage

### Create a package

```bash
# Compile schema first
formproof compile --schema schemas/rate_limit.json --output ./keys

# Build package from witness
formproof package-build \
    --schema schemas/rate_limit.json \
    --proving-key keys/proving_key.bin \
    --witness witness.json \
    --output proof_package.json

# Optional: compact JSON (no whitespace)
formproof package-build ... --compact
```

### Verify a package

```bash
formproof package-verify \
    --schema schemas/rate_limit.json \
    --verifying-key keys/verifying_key.bin \
    --package proof_package.json
```

### View schema fingerprint

```bash
formproof info --schema schemas/rate_limit.json
# Output includes: Fingerprint: 6ff69fec...
```

## Library API

### Create a package

```rust
use formproof::{CompiledSchema, FormProofSchema, ProofPackage, Witness};

let schema = FormProofSchema::from_json(schema_json)?;
let compiled = CompiledSchema::compile(schema)?;

let mut witness = Witness::new();
witness.set_u64("amount", 25);
witness.set_enum("currency", "USD");

// Create package
let package = ProofPackage::create(&compiled, &witness)?;

// Serialize to JSON
let json = package.to_json()?;        // Pretty
let json = package.to_json_compact()?; // Minified
```

### Verify a package

```rust
let package = ProofPackage::from_json(&json)?;

// Verify against compiled schema
package.verify(&compiled)?;
// Returns Ok(()) on success, Err on failure
```

### From existing proof

```rust
use formproof::{Proof, ProofPackage};

let proof = Proof::create(&compiled, &witness)?;
let package = proof.to_package(&compiled.schema)?;
// or
let package = ProofPackage::from_proof(&proof, &compiled.schema)?;
```

### Schema fingerprint

```rust
use formproof::schema_fingerprint;

let fp = schema_fingerprint(&schema);
println!("Fingerprint: {}", hex::encode(fp));
```

## Threat Model Notes

Proof packages provide **transport convenience**, not additional security:

| Concern | Status |
|---------|--------|
| **Schema still public** | The fingerprint is a hash of the schema JSON. Verifiers need the full schema to compile keys. The schema describes the policy, not the private data. |
| **No encryption** | Packages are plaintext JSON. Use TLS or other transport encryption if needed. |
| **Fingerprint binding** | Prevents presenting a proof against the wrong schema. A proof generated for schema A cannot verify against schema B (fingerprint mismatch). |
| **Tamper detection** | Modified proof or commitment bytes will fail Groth16 verification. |
| **Version field** | Allows future format changes without breaking old verifiers. |

## Verification Flow

1. Parse package JSON
2. Check `version == 1` (or supported version)
3. Compute expected fingerprint from verifier's schema
4. Compare to package's `schema_fingerprint`
5. If mismatch → reject with "schema fingerprint mismatch"
6. Deserialize proof bytes with package commitment
7. Run Groth16 verification
8. If invalid → reject with "proof is invalid"
9. Accept proof

## Example Package

For a rate-limit policy with `requests_per_window=100`, `window_secs=3600`, `tier=pro`:

```json
{
  "version": 1,
  "proof_hex": "6a7a4ee553269b5820d808412f62f0788b0e43c7ca942f5d52f9708784e817282f23adcb4a6ecedceafa4f3861fe68e5dd3fc623a7a93f2515421ed39705050b01895952962aff91ef6e582beb9f8a526a1de5...",
  "commitment_hex": "9dbbb4f301ad302a...",
  "schema_fingerprint": "6ff69fecd1c18c5e55ab66b9369b2504901433da00e311d8c6ebb20153e5c94f"
}
```

## See Also

- [SCHEMA_V0.md](SCHEMA_V0.md) — Schema specification
- [HOST_INTEGRATION.md](HOST_INTEGRATION.md) — Verify-only host integration
- [THREAT_MODEL.md](THREAT_MODEL.md) — Security assumptions
- [`examples/proof_package_demo.rs`](../formproof/examples/proof_package_demo.rs) — Working example
