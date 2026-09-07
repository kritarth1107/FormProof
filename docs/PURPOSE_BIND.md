# Purpose Bind Policy

How hosts use FormProof's `purpose_bind` schema to verify processing purpose and legal basis constraints without learning the exact witness tuple.

## Why This Policy Exists

MCP tool hosts and agent runtimes often need to enforce:

- Data processing must be for an allowed purpose
- The legal basis must be GDPR-compliant or equivalent
- Secondary uses must stay within a declared bound

FormProof lets the agent prove those constraints while keeping the concrete purpose, legal basis, and secondary-use limit private.

## Schema Summary

Fixture: [`schemas/purpose_bind.json`](../schemas/purpose_bind.json)

| Field | Type | Constraints | Required |
|-------|------|-------------|----------|
| `purpose` | enum | `inference`, `analytics`, `training`, `support`, `billing`, `debugging` | yes |
| `legal_basis` | enum | `consent`, `contract`, `legitimate_interest`, `legal_obligation` | yes |
| `max_secondary_uses` | integer | 0–8 | no |

## Host Integration Notes

1. **Purpose allowlist is the policy** — treat the purpose enum as the public policy surface. Expanding it is a schema version bump.
2. **Verify before mutate** — only after `verify` succeeds should the host accept a tool call that processes user data.
3. **Log commitments, not purposes** — audit `(commitment, verified, schema_fingerprint)`; never require the witness.
4. **Secondary uses optional** — hosts that do not track secondary-use bounds can omit the field from agent witnesses; hosts that require it should fork a schema with `max_secondary_uses` required.

## Example Valid Witnesses

```json
{ "purpose": "inference", "legal_basis": "consent" }
```

```json
{ "purpose": "analytics", "legal_basis": "legitimate_interest", "max_secondary_uses": 2 }
```

```json
{ "purpose": "training", "legal_basis": "contract", "max_secondary_uses": 0 }
```

```json
{ "purpose": "billing", "legal_basis": "legal_obligation", "max_secondary_uses": 4 }
```

## Example Invalid Witnesses

- `purpose: "marketing"` (not in enum)
- `legal_basis: "verbal_agreement"` (not in enum)
- `max_secondary_uses: 9` (above maximum)
- `max_secondary_uses: -1` (below minimum)

## Demo and Tests

```bash
cargo run -p formproof --example purpose_bind_demo
cargo test -p formproof --test purpose_bind_bounds
```

See also [HOST_INTEGRATION.md](HOST_INTEGRATION.md) for the generic verify-only host flow.
