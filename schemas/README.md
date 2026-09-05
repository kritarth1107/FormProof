# Example Schemas

Frozen FormProof v0 schema fixtures for CLI demos, docs, and host integration tests.

These files match the subset documented in [docs/SCHEMA_V0.md](../docs/SCHEMA_V0.md).

| File | Policy | Fields |
|------|--------|--------|
| [`refund.json`](refund.json) | Refund ≤ $50, allowed currency | `amount` (0–50), `currency` enum |
| [`age_gate.json`](age_gate.json) | Age ≥ 18 with region | `age` (18–120), `region` enum |
| [`access_country.json`](access_country.json) | Country allowlist + tier | `country` enum, `tier` enum, optional `token_id` bytes32 |
| [`spend_cap.json`](spend_cap.json) | Spend limit ≤ $100, multi-currency | `cents` (0–10000), `currency` enum (USD/EUR/GBP/INR) |

## Quick CLI Usage

```bash
# Inspect constraints
formproof info --schema schemas/refund.json

# Compile keys once
formproof compile --schema schemas/refund.json --output ./keys
```

## Sample Witnesses

**refund** (valid):

```json
{ "amount": 25, "currency": "USD" }
```

**age_gate** (valid):

```json
{ "age": 21, "region": "IN" }
```

**access_country** (valid, optional bytes32 omitted):

```json
{ "country": "US", "tier": "pro" }
```

Invalid examples (should fail to prove / fail verify):

- refund with `"amount": 99`
- age_gate with `"age": 16`
- access_country with `"country": "XX"` (not in enum)

## Notes

- Keep fixtures within v0 limits (≤8 properties, ≤8 enum variants, string `maxLength` ≤64).
- Prefer editing these files over embedding one-off JSON in docs when the policy is reusable.
