//! Boundary tests for purpose_bind schema constraints.
//!
//! Verifies that the purpose-limitation policy correctly accepts valid
//! purpose/legal_basis enums and optional max_secondary_uses while rejecting
//! invalid enums and out-of-range secondary use counts.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("purpose_bind.json");
    let json = fs::read_to_string(path).expect("Failed to read purpose_bind.json");
    FormProofSchema::from_json(&json).expect("Invalid purpose_bind schema")
}

#[test]
fn accepts_minimum_secondary_uses() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "inference");
    witness.set_enum("legal_basis", "consent");
    witness.set_u64("max_secondary_uses", 0);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected max_secondary_uses=0 to be accepted");
}

#[test]
fn accepts_maximum_secondary_uses() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "training");
    witness.set_enum("legal_basis", "contract");
    witness.set_u64("max_secondary_uses", 8);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected max_secondary_uses=8 to be accepted");
}

#[test]
fn all_purposes_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let purposes = [
        "inference",
        "analytics",
        "training",
        "support",
        "billing",
        "debugging",
    ];

    for purpose in &purposes {
        let mut witness = Witness::new();
        witness.set_enum("purpose", purpose);
        witness.set_enum("legal_basis", "consent");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected purpose={} to be accepted", purpose);
    }
}

#[test]
fn all_legal_bases_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for legal_basis in &[
        "consent",
        "contract",
        "legitimate_interest",
        "legal_obligation",
    ] {
        let mut witness = Witness::new();
        witness.set_enum("purpose", "analytics");
        witness.set_enum("legal_basis", legal_basis);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(
            result,
            "Expected legal_basis={} to be accepted",
            legal_basis
        );
    }
}

#[test]
fn accepts_without_optional_secondary_uses() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "support");
    witness.set_enum("legal_basis", "legitimate_interest");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected omitted max_secondary_uses to be accepted");
}

#[test]
fn accepts_typical_purpose_bind_request() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "inference");
    witness.set_enum("legal_basis", "contract");
    witness.set_u64("max_secondary_uses", 2);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(
        result,
        "Expected typical purpose_bind request to be accepted"
    );
}

#[test]
fn accepts_mid_range_secondary_uses() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "billing");
    witness.set_enum("legal_basis", "legal_obligation");
    witness.set_u64("max_secondary_uses", 4);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected max_secondary_uses=4 to be accepted");
}

#[test]
fn rejects_secondary_uses_over_maximum() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "training");
    witness.set_enum("legal_basis", "consent");
    witness.set_u64("max_secondary_uses", 9);

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected max_secondary_uses=9 to be rejected");
    }
}

#[test]
fn rejects_invalid_purpose() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "marketing");
    witness.set_enum("legal_basis", "consent");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected purpose=marketing to be rejected");
    }
}

#[test]
fn rejects_invalid_legal_basis() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("purpose", "inference");
    witness.set_enum("legal_basis", "verbal_agreement");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(
            !verified,
            "Expected legal_basis=verbal_agreement to be rejected"
        );
    }
}
