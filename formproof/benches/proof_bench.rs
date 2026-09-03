use criterion::{black_box, criterion_group, criterion_main, Criterion};
use formproof::{CompiledSchema, FormProofSchema, Proof, Witness, verify};

fn refund_schema() -> FormProofSchema {
    FormProofSchema::from_json(r#"{
        "type": "object",
        "properties": {
            "amount": { "type": "integer", "minimum": 0, "maximum": 50 },
            "currency": { "enum": ["USD", "EUR", "GBP"] }
        },
        "required": ["amount", "currency"]
    }"#).unwrap()
}

fn user_schema() -> FormProofSchema {
    FormProofSchema::from_json(r#"{
        "type": "object",
        "properties": {
            "age": { "type": "integer", "minimum": 18, "maximum": 120 },
            "country": { "enum": ["US", "UK", "DE", "FR", "JP"] },
            "name": { "type": "string", "maxLength": 32 }
        },
        "required": ["age", "country"]
    }"#).unwrap()
}

fn token_schema() -> FormProofSchema {
    FormProofSchema::from_json(r#"{
        "type": "object",
        "properties": {
            "token_id": { "type": "string", "format": "bytes32" },
            "balance": { "type": "integer", "minimum": 0, "maximum": 1000000 }
        },
        "required": ["token_id", "balance"]
    }"#).unwrap()
}

fn bench_prove_refund(c: &mut Criterion) {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();
    
    let mut witness = Witness::new();
    witness.set_u64("amount", 25);
    witness.set_enum("currency", "USD");
    
    c.bench_function("prove_refund", |b| {
        b.iter(|| {
            Proof::create(black_box(&compiled), black_box(&witness)).unwrap()
        })
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
        b.iter(|| {
            verify(black_box(&compiled), black_box(&proof)).unwrap()
        })
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
        b.iter(|| {
            Proof::create(black_box(&compiled), black_box(&witness)).unwrap()
        })
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
        b.iter(|| {
            verify(black_box(&compiled), black_box(&proof)).unwrap()
        })
    });
}

fn bench_prove_token(c: &mut Criterion) {
    let schema = token_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();
    
    let mut witness = Witness::new();
    witness.set_bytes32("token_id", [0xAB; 32]);
    witness.set_u64("balance", 1000);
    
    c.bench_function("prove_token", |b| {
        b.iter(|| {
            Proof::create(black_box(&compiled), black_box(&witness)).unwrap()
        })
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
        b.iter(|| {
            verify(black_box(&compiled), black_box(&proof)).unwrap()
        })
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
);

criterion_main!(benches);
