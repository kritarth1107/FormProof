//! Boundary tests for data_residency schema constraints.
//!
//! Verifies that the data residency policy correctly accepts valid regions,
//! storage classes, and retention ranges while rejecting invalid enums or
//! out-of-range retention_days values.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("data_residency.json");
    let json = fs::read_to_string(path).expect("Failed to read data_residency.json");
    FormProofSchema::from_json(&json).expect("Invalid data_residency schema")
}

#[test]
fn accepts_minimum_retention_days() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("region", "us-east");
    witness.set_enum("storage_class", "hot");
    witness.set_u64("retention_days", 1);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected retention_days=1 to be accepted");
}

#[test]
fn accepts_maximum_retention_days() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("region", "eu-central");
    witness.set_enum("storage_class", "archive");
    witness.set_u64("retention_days", 3650);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected retention_days=3650 to be accepted");
}

#[test]
fn all_regions_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let regions = [
        "us-east",
        "us-west",
        "eu-west",
        "eu-central",
        "ap-south",
        "ap-northeast",
        "sa-east",
        "ca-central",
    ];

    for region in &regions {
        let mut witness = Witness::new();
        witness.set_enum("region", region);
        witness.set_enum("storage_class", "warm");
        witness.set_u64("retention_days", 90);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected region={} to be accepted", region);
    }
}

#[test]
fn all_storage_classes_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for storage_class in &["hot", "warm", "cold", "archive"] {
        let mut witness = Witness::new();
        witness.set_enum("region", "ap-south");
        witness.set_enum("storage_class", storage_class);
        witness.set_u64("retention_days", 365);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(
            result,
            "Expected storage_class={} to be accepted",
            storage_class
        );
    }
}

#[test]
fn accepts_with_optional_cross_border() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for cross_border in &["forbidden", "allowed", "eu-only"] {
        let mut witness = Witness::new();
        witness.set_enum("region", "eu-west");
        witness.set_enum("storage_class", "cold");
        witness.set_u64("retention_days", 730);
        witness.set_enum("cross_border", cross_border);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(
            result,
            "Expected cross_border={} to be accepted",
            cross_border
        );
    }
}

#[test]
fn accepts_typical_residency_request() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("region", "us-west");
    witness.set_enum("storage_class", "hot");
    witness.set_u64("retention_days", 30);
    witness.set_enum("cross_border", "forbidden");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected typical residency request to be accepted");
}

#[test]
fn rejects_zero_retention_days() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("region", "us-east");
    witness.set_enum("storage_class", "hot");
    witness.set_u64("retention_days", 0);

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected retention_days=0 to be rejected");
    }
}

#[test]
fn rejects_retention_days_over_maximum() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("region", "eu-central");
    witness.set_enum("storage_class", "archive");
    witness.set_u64("retention_days", 3651);

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected retention_days=3651 to be rejected");
    }
}

#[test]
fn rejects_invalid_region() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("region", "mars-1");
    witness.set_enum("storage_class", "hot");
    witness.set_u64("retention_days", 30);

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected region=mars-1 to be rejected");
    }
}

#[test]
fn rejects_invalid_storage_class() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("region", "us-east");
    witness.set_enum("storage_class", "plasma");
    witness.set_u64("retention_days", 30);

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected storage_class=plasma to be rejected");
    }
}
