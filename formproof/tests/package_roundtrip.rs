//! Integration tests for proof package roundtrip: create → verify.
//!
//! Tests the full workflow of creating proof packages and verifying them,
//! including tamper detection.

use formproof::{
    schema_fingerprint, CompiledSchema, FormProofSchema, PackageError, Proof, ProofPackage, Witness,
};

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

fn age_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "age": { "type": "integer", "minimum": 18, "maximum": 120 }
        },
        "required": ["age"]
    }"#,
    )
    .unwrap()
}

#[test]
fn test_package_roundtrip_refund() {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 25);
    witness.set_enum("currency", "USD");

    let package = ProofPackage::create(&compiled, &witness).unwrap();
    let json = package.to_json().unwrap();

    let loaded = ProofPackage::from_json(&json).unwrap();
    assert!(loaded.verify(&compiled).is_ok());
}

#[test]
fn test_package_roundtrip_boundary_values() {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 0);
    witness.set_enum("currency", "GBP");

    let package = ProofPackage::create(&compiled, &witness).unwrap();
    let json = package.to_json().unwrap();

    let loaded = ProofPackage::from_json(&json).unwrap();
    assert!(loaded.verify(&compiled).is_ok());

    let mut witness_max = Witness::new();
    witness_max.set_u64("amount", 50);
    witness_max.set_enum("currency", "EUR");

    let package_max = ProofPackage::create(&compiled, &witness_max).unwrap();
    assert!(package_max.verify(&compiled).is_ok());
}

#[test]
fn test_package_compact_json() {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 10);
    witness.set_enum("currency", "EUR");

    let package = ProofPackage::create(&compiled, &witness).unwrap();
    let compact = package.to_json_compact().unwrap();
    let pretty = package.to_json().unwrap();

    assert!(compact.len() < pretty.len());
    assert!(!compact.contains('\n'));

    let loaded = ProofPackage::from_json(&compact).unwrap();
    assert!(loaded.verify(&compiled).is_ok());
}

#[test]
fn test_package_reject_tampered_proof() {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 25);
    witness.set_enum("currency", "USD");

    let package = ProofPackage::create(&compiled, &witness).unwrap();
    let json = package.to_json().unwrap();

    let mut tampered: serde_json::Value = serde_json::from_str(&json).unwrap();
    let proof_hex = tampered["proof_hex"].as_str().unwrap().to_string();
    let mut proof_bytes = hex::decode(&proof_hex).unwrap();
    proof_bytes[0] ^= 0xFF;
    tampered["proof_hex"] = serde_json::json!(hex::encode(&proof_bytes));

    let tampered_json = serde_json::to_string(&tampered).unwrap();
    let loaded = ProofPackage::from_json(&tampered_json).unwrap();

    let result = loaded.verify(&compiled);
    assert!(result.is_err());
}

#[test]
fn test_package_reject_tampered_commitment() {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 25);
    witness.set_enum("currency", "USD");

    let package = ProofPackage::create(&compiled, &witness).unwrap();
    let json = package.to_json().unwrap();

    let mut tampered: serde_json::Value = serde_json::from_str(&json).unwrap();
    let commitment_hex = tampered["commitment_hex"].as_str().unwrap().to_string();
    let mut commitment_bytes = hex::decode(&commitment_hex).unwrap();
    commitment_bytes[0] ^= 0xFF;
    tampered["commitment_hex"] = serde_json::json!(hex::encode(&commitment_bytes));

    let tampered_json = serde_json::to_string(&tampered).unwrap();
    let loaded = ProofPackage::from_json(&tampered_json).unwrap();

    let result = loaded.verify(&compiled);
    assert!(result.is_err());
}

#[test]
fn test_package_reject_wrong_schema() {
    let schema1 = refund_schema();
    let compiled1 = CompiledSchema::compile(schema1).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 25);
    witness.set_enum("currency", "USD");

    let package = ProofPackage::create(&compiled1, &witness).unwrap();

    let schema2 = age_schema();
    let compiled2 = CompiledSchema::compile(schema2).unwrap();

    let result = package.verify(&compiled2);
    assert!(matches!(
        result,
        Err(PackageError::SchemaFingerprintMismatch { .. })
    ));
}

#[test]
fn test_proof_to_package_method() {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema.clone()).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 30);
    witness.set_enum("currency", "GBP");

    let proof = Proof::create(&compiled, &witness).unwrap();
    let package = proof.to_package(&schema).unwrap();

    assert!(package.verify(&compiled).is_ok());
}

#[test]
fn test_schema_fingerprint_stability() {
    let schema1 = refund_schema();
    let schema2 = refund_schema();

    let fp1 = schema_fingerprint(&schema1);
    let fp2 = schema_fingerprint(&schema2);

    assert_eq!(fp1, fp2);
}

#[test]
fn test_schema_fingerprint_different_schemas() {
    let schema1 = refund_schema();
    let schema2 = age_schema();

    let fp1 = schema_fingerprint(&schema1);
    let fp2 = schema_fingerprint(&schema2);

    assert_ne!(fp1, fp2);
}

#[test]
fn test_package_commitment_extraction() {
    let schema = refund_schema();
    let compiled = CompiledSchema::compile(schema).unwrap();

    let mut witness = Witness::new();
    witness.set_u64("amount", 40);
    witness.set_enum("currency", "EUR");

    let proof = Proof::create(&compiled, &witness).unwrap();
    let package = ProofPackage::from_proof(&proof, &compiled.schema).unwrap();

    let extracted = package.commitment().unwrap();
    assert_eq!(extracted, proof.commitment);
}
