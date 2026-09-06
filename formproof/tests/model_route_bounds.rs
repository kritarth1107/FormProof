//! Boundary tests for model_route schema constraints.
//!
//! Verifies that the model routing policy correctly accepts valid models,
//! token ranges, and priority levels while rejecting out-of-range values
//! or invalid enum variants.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("model_route.json");
    let json = fs::read_to_string(path).expect("Failed to read model_route.json");
    FormProofSchema::from_json(&json).expect("Invalid model_route schema")
}

#[test]
fn accepts_minimum_max_tokens() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("model_id", "gpt-4o");
    witness.set_u64("max_tokens", 1);
    witness.set_enum("priority", "normal");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected max_tokens=1 to be accepted");
}

#[test]
fn accepts_maximum_max_tokens() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("model_id", "claude-3-opus");
    witness.set_u64("max_tokens", 128000);
    witness.set_enum("priority", "critical");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected max_tokens=128000 to be accepted");
}

#[test]
fn all_model_ids_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let models = [
        "gpt-4o",
        "gpt-4o-mini",
        "claude-3-opus",
        "claude-3-sonnet",
        "claude-3-haiku",
        "gemini-pro",
        "llama-3",
        "mixtral",
    ];

    for model in &models {
        let mut witness = Witness::new();
        witness.set_enum("model_id", model);
        witness.set_u64("max_tokens", 4096);
        witness.set_enum("priority", "normal");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected model_id={} to be accepted", model);
    }
}

#[test]
fn all_priorities_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for priority in &["low", "normal", "high", "critical"] {
        let mut witness = Witness::new();
        witness.set_enum("model_id", "gpt-4o");
        witness.set_u64("max_tokens", 8192);
        witness.set_enum("priority", priority);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected priority={} to be accepted", priority);
    }
}

#[test]
fn accepts_with_optional_temperature_class() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for temp_class in &["deterministic", "balanced", "creative"] {
        let mut witness = Witness::new();
        witness.set_enum("model_id", "claude-3-sonnet");
        witness.set_u64("max_tokens", 16384);
        witness.set_enum("priority", "high");
        witness.set_enum("temperature_class", temp_class);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(
            result,
            "Expected temperature_class={} to be accepted",
            temp_class
        );
    }
}

#[test]
fn accepts_typical_routing_request() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("model_id", "gpt-4o-mini");
    witness.set_u64("max_tokens", 2048);
    witness.set_enum("priority", "low");
    witness.set_enum("temperature_class", "balanced");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected typical routing request to be accepted");
}

#[test]
fn rejects_zero_max_tokens() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("model_id", "gpt-4o");
    witness.set_u64("max_tokens", 0);
    witness.set_enum("priority", "normal");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected max_tokens=0 to be rejected");
    }
}

#[test]
fn rejects_max_tokens_over_maximum() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("model_id", "claude-3-opus");
    witness.set_u64("max_tokens", 128001);
    witness.set_enum("priority", "high");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected max_tokens=128001 to be rejected");
    }
}

#[test]
fn rejects_invalid_model_id() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("model_id", "unknown-model");
    witness.set_u64("max_tokens", 4096);
    witness.set_enum("priority", "normal");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected model_id=unknown-model to be rejected");
    }
}

#[test]
fn rejects_invalid_priority() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("model_id", "gpt-4o");
    witness.set_u64("max_tokens", 4096);
    witness.set_enum("priority", "urgent");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected priority=urgent to be rejected");
    }
}
