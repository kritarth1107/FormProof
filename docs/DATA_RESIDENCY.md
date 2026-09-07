# Data Residency Policy

How hosts use FormProof's `data_residency` schema to verify placement and retention constraints without learning the exact placement tuple.

## Why This Policy Exists

MCP tool hosts and agent runtimes often need to enforce:

- Data must live in an allowed region set
- Storage class must match the retention tier
- Retention windows must stay inside legal / contractual bounds
- Cross-border transfer rules (forbidden, allowed, EU-only)

FormProof lets the agent prove those constraints while keeping the concrete region, class, and retention days private.

## Schema Summary

Fixture: [`schemas/data_residency.json`](../schemas/data_residency.json)

| Field | Type | Constraints | Required |
|-------|------|-------------|----------|
| `region` | enum | `us-east`, `us-west`, `eu-west`, `eu-central`, `ap-south`, `ap-northeast`, `sa-east`, `ca-central` | yes |
| `storage_class` | enum | `hot`, `warm`, `cold`, `archive` | yes |
| `retention_days` | integer | 1–3650 | yes |
| `cross_border` | enum | `forbidden`, `allowed`, `eu-only` | no |

## Host Integration Notes

1. **Freeze the allowlist** — treat the region enum as the public policy surface. Expanding it is a schema version bump.
2. **Verify, then route** — only after `verify` succeeds should the host accept a store/move tool call.
3. **Log commitments, not placements** — audit `(commitment, verified, schema_fingerprint)`; never require the witness.
4. **Cross-border is optional** — hosts that do not care about transfer policy can omit the field from agent witnesses; hosts that require it should reject missing proofs by using a stricter schema fork.

## Example Valid Witnesses

```json
{ "region": "us-east", "storage_class": "hot", "retention_days": 30, "cross_border": "forbidden" }
```

```json
{ "region": "eu-west", "storage_class": "cold", "retention_days": 730, "cross_border": "eu-only" }
```

```json
{ "region": "ap-south", "storage_class": "warm", "retention_days": 90 }
```

## Example Invalid Witnesses

- `retention_days: 0` (below minimum)
- `retention_days: 3651` (above maximum)
- `region: "mars-1"` (not in enum)
- `storage_class: "plasma"` (not in enum)
- `cross_border: "asia-only"` (not in enum)

## Demo and Tests

```bash
cargo run -p formproof --example data_residency_demo
cargo test -p formproof --test data_residency_bounds
```

See also [HOST_INTEGRATION.md](HOST_INTEGRATION.md) for the generic verify-only host flow.
