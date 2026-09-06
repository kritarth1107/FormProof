//! Boundary tests for access_country schema constraints.
//!
//! Verifies that the access country policy correctly accepts valid countries/regions
//! and rejects invalid country codes or tier values.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("access_country.json");
    let json = fs::read_to_string(path).expect("Failed to read access_country.json");
    FormProofSchema::from_json(&json).expect("Invalid access_country schema")
}

#[test]
fn accepts_all_valid_countries() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for country in &["US", "CA", "GB", "DE", "FR", "IN", "JP", "AU"] {
        let mut witness = Witness::new();
        witness.set_enum("country", country);
        witness.set_enum("tier", "free");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected country={} to be accepted", country);
    }
}

#[test]
fn accepts_all_valid_tiers() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for tier in &["free", "pro", "enterprise"] {
        let mut witness = Witness::new();
        witness.set_enum("country", "US");
        witness.set_enum("tier", tier);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected tier={} to be accepted", tier);
    }
}

#[test]
fn accepts_with_optional_token_id() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("country", "JP");
    witness.set_enum("tier", "enterprise");
    witness.set_bytes32("token_id", [0xAB; 32]);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected country with token_id to be accepted");
}

#[test]
fn accepts_country_tier_combinations() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let combinations = [
        ("US", "free"),
        ("CA", "pro"),
        ("GB", "enterprise"),
        ("DE", "free"),
        ("FR", "pro"),
        ("IN", "enterprise"),
        ("JP", "free"),
        ("AU", "pro"),
    ];

    for (country, tier) in combinations {
        let mut witness = Witness::new();
        witness.set_enum("country", country);
        witness.set_enum("tier", tier);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(
            result,
            "Expected country={}, tier={} to be accepted",
            country, tier
        );
    }
}

#[test]
fn rejects_invalid_country() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("country", "XX");
    witness.set_enum("tier", "free");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected country=XX to be rejected");
    }
}

#[test]
fn rejects_invalid_tier() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("country", "US");
    witness.set_enum("tier", "premium");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected tier=premium to be rejected");
    }
}
