//! Boundary tests for quota_budget schema constraints.
//!
//! Verifies that the quota budget policy correctly accepts boundary values
//! and rejects out-of-range values or invalid enum variants.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("quota_budget.json");
    let json = fs::read_to_string(path).expect("Failed to read quota_budget.json");
    FormProofSchema::from_json(&json).expect("Invalid quota_budget schema")
}

#[test]
fn accepts_minimum_budget_units() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 1);
    witness.set_enum("period", "daily");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected budget_units=1 to be accepted");
}

#[test]
fn accepts_maximum_budget_units() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 1000000);
    witness.set_enum("period", "monthly");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected budget_units=1000000 to be accepted");
}

#[test]
fn all_periods_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for period in &["daily", "weekly", "monthly"] {
        let mut witness = Witness::new();
        witness.set_u64("budget_units", 5000);
        witness.set_enum("period", period);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected period={} to be accepted", period);
    }
}

#[test]
fn accepts_with_optional_soft_cap() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 100000);
    witness.set_enum("period", "monthly");
    witness.set_u64("soft_cap", 80000);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected quota with soft_cap to be accepted");
}

#[test]
fn accepts_soft_cap_at_zero() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 10000);
    witness.set_enum("period", "weekly");
    witness.set_u64("soft_cap", 0);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected soft_cap=0 to be accepted");
}

#[test]
fn accepts_soft_cap_at_maximum() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 1000000);
    witness.set_enum("period", "monthly");
    witness.set_u64("soft_cap", 1000000);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected soft_cap=1000000 to be accepted");
}

#[test]
fn rejects_zero_budget_units() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 0);
    witness.set_enum("period", "daily");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected budget_units=0 to be rejected");
    }
}

#[test]
fn rejects_budget_units_over_maximum() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 1000001);
    witness.set_enum("period", "monthly");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected budget_units=1000001 to be rejected");
    }
}

#[test]
fn rejects_invalid_period() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("budget_units", 5000);
    witness.set_enum("period", "yearly");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected period=yearly to be rejected");
    }
}
