//! Boundary tests for age_gate schema constraints.
//!
//! Verifies that the age gate policy correctly accepts boundary ages/regions
//! and rejects under-age or out-of-range values.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("age_gate.json");
    let json = fs::read_to_string(path).expect("Failed to read age_gate.json");
    FormProofSchema::from_json(&json).expect("Invalid age_gate schema")
}

#[test]
fn accepts_minimum_age() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("age", 18);
    witness.set_enum("region", "US");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected age=18 to be accepted");
}

#[test]
fn accepts_maximum_age() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("age", 120);
    witness.set_enum("region", "EU");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected age=120 to be accepted");
}

#[test]
fn all_regions_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for region in &["US", "EU", "UK", "IN"] {
        let mut witness = Witness::new();
        witness.set_u64("age", 25);
        witness.set_enum("region", region);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected region={} to be accepted", region);
    }
}

#[test]
fn rejects_underage() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("age", 17);
    witness.set_enum("region", "US");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected age=17 to be rejected");
    }
}

#[test]
fn rejects_age_over_maximum() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("age", 121);
    witness.set_enum("region", "IN");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected age=121 to be rejected");
    }
}
