use criterion::{black_box, criterion_group, criterion_main, Criterion};
use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};

fn refund_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "amount": { "type": "integer", "minimum": 0, "maximum": 50 },
            "currency": { "enum": ["USD", "EUR", "GBP"] }
        },
        "required": ["amount", "currency"]
    }"#,
    )
    .unwrap()
}

fn user_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "age": { "type": "integer", "minimum": 18, "maximum": 120 },
            "country": { "enum": ["US", "UK", "DE", "FR", "JP"] },
            "name": { "type": "string", "maxLength": 32 }
        },
        "required": ["age", "country"]
    }"#,
    )
    .unwrap()
}

fn token_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "token_id": { "type": "string", "format": "bytes32" },
            "balance": { "type": "integer", "minimum": 0, "maximum": 1000000 }
        },
        "required": ["token_id", "balance"]
    }"#,
    )
    .unwrap()
}

fn rate_limit_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "requests_per_window": { "type": "integer", "minimum": 1, "maximum": 10000 },
            "window_secs": { "type": "integer", "minimum": 1, "maximum": 86400 },
            "tier": { "enum": ["free", "basic", "pro", "enterprise"] }
        },
        "required": ["requests_per_window", "window_secs", "tier"]
    }"#,
    )
    .unwrap()
}

fn session_ttl_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "ttl_seconds": { "type": "integer", "minimum": 1, "maximum": 86400 },
            "tier": { "enum": ["free", "pro", "enterprise"] }
        },
        "required": ["ttl_seconds", "tier"]
    }"#,
    )
    .unwrap()
}

fn age_gate_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "age": { "type": "integer", "minimum": 18, "maximum": 120 },
            "region": { "enum": ["US", "EU", "UK", "IN"] }
        },
        "required": ["age", "region"]
    }"#,
    )
    .unwrap()
}

fn tool_allowlist_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "tool_name": { "enum": ["read", "write", "execute", "list", "search", "delete"] },
            "max_args": { "type": "integer", "minimum": 0, "maximum": 64 },
            "scope": { "enum": ["local", "remote", "any"] }
        },
        "required": ["tool_name", "scope"]
    }"#,
    )
    .unwrap()
}

fn bench_prove_refund(c: &mut Criterion) {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 25);
    witness.set_enum("currency", "USD");

    c.bench_function("prove_refund", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_refund(c: &mut Criterion) {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 25);
    witness.set_enum("currency", "USD");

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_refund", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn bench_prove_user(c: &mut Criterion) {
    let schema = user_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("age", 30);
    witness.set_enum("country", "US");
    witness.set_string("name", "Alice");

    c.bench_function("prove_user", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_user(c: &mut Criterion) {
    let schema = user_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("age", 30);
    witness.set_enum("country", "US");
    witness.set_string("name", "Alice");

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_user", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn bench_prove_token(c: &mut Criterion) {
    let schema = token_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_bytes32("token_id", [0xAB; 32]);
    witness.set_u64("balance", 1000);

    c.bench_function("prove_token", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_token(c: &mut Criterion) {
    let schema = token_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_bytes32("token_id", [0xAB; 32]);
    witness.set_u64("balance", 1000);

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_token", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn bench_prove_rate_limit(c: &mut Criterion) {
    let schema = rate_limit_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("requests_per_window", 100);
    witness.set_u64("window_secs", 3600);
    witness.set_enum("tier", "pro");

    c.bench_function("prove_rate_limit", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_rate_limit(c: &mut Criterion) {
    let schema = rate_limit_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("requests_per_window", 100);
    witness.set_u64("window_secs", 3600);
    witness.set_enum("tier", "pro");

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_rate_limit", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn bench_prove_session_ttl(c: &mut Criterion) {
    let schema = session_ttl_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("ttl_seconds", 3600);
    witness.set_enum("tier", "pro");

    c.bench_function("prove_session_ttl", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_session_ttl(c: &mut Criterion) {
    let schema = session_ttl_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("ttl_seconds", 3600);
    witness.set_enum("tier", "pro");

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_session_ttl", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn bench_prove_age_gate(c: &mut Criterion) {
    let schema = age_gate_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("age", 25);
    witness.set_enum("region", "US");

    c.bench_function("prove_age_gate", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_age_gate(c: &mut Criterion) {
    let schema = age_gate_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("age", 25);
    witness.set_enum("region", "US");

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_age_gate", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn bench_prove_tool_allowlist(c: &mut Criterion) {
    let schema = tool_allowlist_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_enum("tool_name", "read");
    witness.set_u64("max_args", 8);
    witness.set_enum("scope", "local");

    c.bench_function("prove_tool_allowlist", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_tool_allowlist(c: &mut Criterion) {
    let schema = tool_allowlist_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_enum("tool_name", "read");
    witness.set_u64("max_args", 8);
    witness.set_enum("scope", "local");

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_tool_allowlist", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn quota_budget_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "budget_units": { "type": "integer", "minimum": 1, "maximum": 1000000 },
            "period": { "enum": ["daily", "weekly", "monthly"] },
            "soft_cap": { "type": "integer", "minimum": 0, "maximum": 1000000 }
        },
        "required": ["budget_units", "period"]
    }"#,
    )
    .unwrap()
}

fn model_route_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "model_id": { "enum": ["gpt-4o", "gpt-4o-mini", "claude-3-opus", "claude-3-sonnet", "claude-3-haiku", "gemini-pro", "llama-3", "mixtral"] },
            "max_tokens": { "type": "integer", "minimum": 1, "maximum": 128000 },
            "priority": { "enum": ["low", "normal", "high", "critical"] },
            "temperature_class": { "enum": ["deterministic", "balanced", "creative"] }
        },
        "required": ["model_id", "max_tokens", "priority"]
    }"#,
    )
    .unwrap()
}

fn bench_prove_quota_budget(c: &mut Criterion) {
    let schema = quota_budget_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 50000);
    witness.set_enum("period", "monthly");
    witness.set_u64("soft_cap", 40000);

    c.bench_function("prove_quota_budget", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_quota_budget(c: &mut Criterion) {
    let schema = quota_budget_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 50000);
    witness.set_enum("period", "monthly");
    witness.set_u64("soft_cap", 40000);

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_quota_budget", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

fn bench_prove_model_route(c: &mut Criterion) {
    let schema = model_route_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_enum("model_id", "claude-3-sonnet");
    witness.set_u64("max_tokens", 8192);
    witness.set_enum("priority", "high");
    witness.set_enum("temperature_class", "balanced");

    c.bench_function("prove_model_route", |b| {
        b.iter(|| Proof::create(black_box(&compiled), black_box(&witness)).unwrap())
    });
}

fn bench_verify_model_route(c: &mut Criterion) {
    let schema = model_route_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_enum("model_id", "claude-3-sonnet");
    witness.set_u64("max_tokens", 8192);
    witness.set_enum("priority", "high");
    witness.set_enum("temperature_class", "balanced");

    let proof = Proof::create(&compiled, &witness).unwrap();

    c.bench_function("verify_model_route", |b| {
        b.iter(|| verify(black_box(&compiled), black_box(&proof)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_prove_refund,
    bench_verify_refund,
    bench_prove_user,
    bench_verify_user,
    bench_prove_token,
    bench_verify_token,
    bench_prove_rate_limit,
    bench_verify_rate_limit,
    bench_prove_session_ttl,
    bench_verify_session_ttl,
    bench_prove_age_gate,
    bench_verify_age_gate,
    bench_prove_tool_allowlist,
    bench_verify_tool_allowlist,
    bench_prove_quota_budget,
    bench_verify_quota_budget,
    bench_prove_model_route,
    bench_verify_model_route,
);

criterion_main!(benches);
