//! Boundary tests for session_ttl schema constraints.
//!
//! Verifies that the session TTL policy correctly accepts boundary values
//! and rejects out-of-range values.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("session_ttl.json");
    let json = fs::read_to_string(path).expect("Failed to read session_ttl.json");
    FormProofSchema::from_json(&json).expect("Invalid session_ttl schema")
}

#[test]
fn accepts_minimum_ttl() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("ttl_seconds", 1);
    witness.set_enum("tier", "free");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected ttl_seconds=1 to be accepted");
}

#[test]
fn accepts_maximum_ttl() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("ttl_seconds", 86400);
    witness.set_enum("tier", "enterprise");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected ttl_seconds=86400 (24h) to be accepted");
}

#[test]
fn all_tiers_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for tier in &["free", "pro", "enterprise"] {
        let mut witness = Witness::new();
        witness.set_u64("ttl_seconds", 3600);
        witness.set_enum("tier", tier);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected tier={} to be accepted", tier);
    }
}

#[test]
fn rejects_zero_ttl() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("ttl_seconds", 0);
    witness.set_enum("tier", "free");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected ttl_seconds=0 to be rejected");
    }
}

#[test]
fn rejects_ttl_over_maximum() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("ttl_seconds", 86401);
    witness.set_enum("tier", "pro");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected ttl_seconds=86401 to be rejected");
    }
}
