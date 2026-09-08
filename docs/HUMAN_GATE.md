# Human Gate Policy

How hosts use FormProof's `human_gate` schema to verify human-in-the-loop approval
constraints without learning the exact action class, approval tier, or auto-approve TTL.

## Why This Policy Exists

MCP tool hosts and agent runtimes often need to enforce:

- High-risk tool actions require a declared human approval tier
- Action risk class must be within an allowlisted set
- Cached auto-approval windows must stay within a declared bound

FormProof lets the agent prove those constraints while keeping the concrete
action class, approval tier, and auto-approve TTL private.

## Schema Summary

Fixture: [`schemas/human_gate.json`](../schemas/human_gate.json)

| Field | Type | Constraints | Required |
|-------|------|-------------|----------|
| `action_class` | enum | `read`, `write`, `delete`, `export`, `admin`, `payment` | yes |
| `approval_tier` | enum | `self`, `peer`, `manager`, `security` | yes |
| `max_auto_approve_secs` | integer | 0–86400 | no |

## Host Integration Notes

1. **Action allowlist is the policy** — treat the `action_class` enum as the public policy surface. Expanding it is a schema version bump.
2. **Verify before mutate** — only after `verify` succeeds should the host accept a tool call that performs the gated action.
3. **Log commitments, not tiers** — audit `(commitment, verified, schema_fingerprint)`; never require the witness.
4. **Auto-approve optional** — hosts that do not cache approvals can omit the field; hosts that require it should fork a schema with `max_auto_approve_secs` required.

## Example Valid Witnesses

```json
{ "action_class": "read", "approval_tier": "self" }
```

```json
{ "action_class": "write", "approval_tier": "peer", "max_auto_approve_secs": 300 }
```

```json
{ "action_class": "delete", "approval_tier": "security", "max_auto_approve_secs": 0 }
```

```json
{ "action_class": "payment", "approval_tier": "manager", "max_auto_approve_secs": 3600 }
```

## Example Invalid Witnesses

- `action_class: "exfiltrate"` (not in enum)
- `approval_tier: "anonymous"` (not in enum)
- `max_auto_approve_secs: 86401` (above maximum)
- `max_auto_approve_secs: -1` (below minimum)

## Demo and Tests

```bash
cargo run -p formproof --example human_gate_demo
cargo test -p formproof --test human_gate_bounds
```

See also [HOST_INTEGRATION.md](HOST_INTEGRATION.md) for the generic verify-only host flow.
