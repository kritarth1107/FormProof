# Host Integration Guide

How an MCP/tool host should integrate FormProof verification without seeing private tool arguments.

## Roles

| Role | Holds | Sees |
|------|-------|------|
| **Schema author / host** | Policy schema, verifying key | Commitment + proof |
| **Prover (agent)** | Witness values, proving key | Full private JSON |
| **Network** | Serialized proof bytes | Nothing about field values |

The schema is **public**. Only the payload (witness) is private.

## Recommended Host Flow

1. **Define policy once** — freeze a v0 schema (see [SCHEMA_V0.md](SCHEMA_V0.md)).
2. **Compile once** — call `CompiledSchema::compile`, persist the verifying key (and proving key for agents that prove locally).
3. **Distribute** — give agents the schema + proving key (or a prove API you operate).
4. **Verify on every request** — accept `(proof_bytes, commitment)`, load VK, call `verify`.
5. **Act on the boolean** — proceed only if verification returns `Ok(true)`.

```text
Agent                         Host
  |                             |
  |-- prove(witness) ---------> |
  |   proof + commitment       |
  |                             |-- verify(vk, proof, commitment)
  |                             |-- accept / reject
```

## Minimal Verify Path (Rust)

```rust
use formproof::{verify, CompiledSchema, FormProofSchema, Proof};

fn host_verify(
    schema_json: &str,
    vk_bytes: &[u8],
    pk_bytes: &[u8], // only needed if you reconstruct CompiledSchema this way
    proof_bytes: &[u8],
    commitment: [u8; 32],
) -> Result<bool, Box<dyn std::error::Error>> {
    let schema = FormProofSchema::from_json(schema_json)?;
    let compiled = CompiledSchema::deserialize_keys(schema, pk_bytes, vk_bytes)?;
    let proof = Proof::deserialize(proof_bytes, commitment)?;
    Ok(verify(&compiled, &proof)?)
}
```

For a full walkthrough, run:

```bash
cargo run --example verify_only
cargo run --example mcp_tool_host
```

## What the Host Must Treat as Public

- The JSON Schema / policy document
- The verifying key
- The 32-byte commitment
- The proof bytes

## What the Host Must Never Require

- Raw field values (`amount`, `currency`, etc.)
- The proving key (unless the host also runs proving for clients)

Asking for the witness alongside the proof defeats the point of FormProof.

## Failure Modes Hosts Should Handle

| Outcome | Meaning | Host action |
|---------|---------|-------------|
| `Ok(true)` | Constraints satisfied | Allow the tool call |
| `Ok(false)` | Proof does not satisfy circuit | Reject |
| `Err(...)` | Malformed proof / key / schema mismatch | Reject and log |

Never soft-fail into "trust the agent anyway."

## Commitment Binding

The commitment is a hash of the witness. If an attacker swaps the commitment while keeping proof bytes, verification fails (see the tamper demo in `examples/verify_only.rs`). Hosts should bind `(tool_name, schema_id, commitment, proof)` together in their audit log.

## Replay Considerations

FormProof does **not** include a nonce or timestamp in v0. If replay matters for your product:

- Bind an application-level nonce into a schema field the agent must prove, or
- Track seen commitments server-side and reject duplicates, or
- Wrap FormProof inside a larger authenticated request protocol

See [THREAT_MODEL.md](THREAT_MODEL.md) for the honest security boundary.

## CLI Equivalent

```bash
formproof compile --schema schemas/refund.json --output ./keys
formproof verify --schema schemas/refund.json \
  --verifying-key keys/verifying_key.bin \
  --proof proof.bin \
  --commitment <hex>
```

Example schemas live in [`schemas/`](../schemas/).

## WASM / Browser Hosts

Browser verification is **not** production-ready yet. Prefer server-side verify until WASM support lands. Details: [WASM.md](WASM.md).

## Checklist for Shipping a Host Integration

- [ ] Schema frozen and versioned (`schema_id`)
- [ ] Verifying key stored offline from proving key
- [ ] Verify path covered by integration tests
- [ ] Reject path tested (out-of-range, bad enum, missing required)
- [ ] Audit log stores commitment + verify result, not witness
- [ ] Replay policy decided for your threat model
