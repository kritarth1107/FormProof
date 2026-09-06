# Example Schemas

Frozen FormProof v0 schema fixtures for CLI demos, docs, and host integration tests.

These files match the subset documented in [docs/SCHEMA_V0.md](../docs/SCHEMA_V0.md).

| File | Policy | Fields |
|------|--------|--------|
| [`refund.json`](refund.json) | Refund ≤ $50, allowed currency | `amount` (0–50), `currency` enum |
| [`age_gate.json`](age_gate.json) | Age ≥ 18 with region | `age` (18–120), `region` enum |
| [`access_country.json`](access_country.json) | Country allowlist + tier | `country` enum, `tier` enum, optional `token_id` bytes32 |
| [`spend_cap.json`](spend_cap.json) | Spend limit ≤ $100, multi-currency | `cents` (0–10000), `currency` enum (USD/EUR/GBP/INR) |
| [`session_ttl.json`](session_ttl.json) | Session TTL with tier | `ttl_seconds` (1–86400), `tier` enum (free/pro/enterprise) |
| [`rate_limit.json`](rate_limit.json) | MCP rate-limit policy | `requests_per_window` (1–10000), `window_secs` (1–86400), `tier` enum |
| [`tool_allowlist.json`](tool_allowlist.json) | MCP tool access policy | `tool_name` enum, optional `max_args` (0–64), `scope` enum |
| [`quota_budget.json`](quota_budget.json) | MCP quota/budget policy | `budget_units` (1–1000000), `period` enum, optional `soft_cap` (0–1000000) |
| [`model_route.json`](model_route.json) | MCP model-routing policy | `model_id` enum, `max_tokens` (1–128000), `priority` enum, optional `temperature_class` enum |

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

**rate_limit** (valid):

```json
{ "requests_per_window": 100, "window_secs": 3600, "tier": "pro" }
```

**tool_allowlist** (valid, optional max_args omitted):

```json
{ "tool_name": "read", "scope": "local" }
```

**quota_budget** (valid, optional soft_cap omitted):

```json
{ "budget_units": 5000, "period": "daily" }
```

**quota_budget** (valid, with soft_cap):

```json
{ "budget_units": 100000, "period": "monthly", "soft_cap": 80000 }
```

**model_route** (valid, required fields only):

```json
{ "model_id": "claude-3-sonnet", "max_tokens": 4096, "priority": "normal" }
```

**model_route** (valid, with optional temperature_class):

```json
{ "model_id": "gpt-4o", "max_tokens": 8192, "priority": "high", "temperature_class": "balanced" }
```

Invalid examples (should fail to prove / fail verify):

- refund with `"amount": 99`
- age_gate with `"age": 16`
- access_country with `"country": "XX"` (not in enum)
- rate_limit with `"requests_per_window": 0` (below minimum)
- tool_allowlist with `"tool_name": "admin"` (not in enum)
- quota_budget with `"budget_units": 0` (below minimum)
- quota_budget with `"period": "yearly"` (not in enum)
- model_route with `"model_id": "unknown-model"` (not in enum)
- model_route with `"max_tokens": 0` (below minimum)
- model_route with `"max_tokens": 200000` (above maximum)

## Notes

- Keep fixtures within v0 limits (≤8 properties, ≤8 enum variants, string `maxLength` ≤64).
- Prefer editing these files over embedding one-off JSON in docs when the policy is reusable.
