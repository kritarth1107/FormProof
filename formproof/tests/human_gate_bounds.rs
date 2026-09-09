//! Boundary tests for human_gate schema constraints.
//!
//! Verifies that the human-in-the-loop approval policy correctly accepts valid
//! action_class/approval_tier enums and optional max_auto_approve_secs while
//! rejecting invalid enums and out-of-range auto-approve windows.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema() -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join("human_gate.json");
    let json = fs::read_to_string(path).expect("Failed to read human_gate.json");
    FormProofSchema::from_json(&json).expect("Invalid human_gate schema")
}

#[test]
fn accepts_minimum_auto_approve_secs() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "read");
    witness.set_enum("approval_tier", "self");
    witness.set_u64("max_auto_approve_secs", 0);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected max_auto_approve_secs=0 to be accepted");
}

#[test]
fn accepts_maximum_auto_approve_secs() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "write");
    witness.set_enum("approval_tier", "manager");
    witness.set_u64("max_auto_approve_secs", 86400);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(
        result,
        "Expected max_auto_approve_secs=86400 to be accepted"
    );
}

#[test]
fn all_action_classes_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let classes = ["read", "write", "delete", "export", "admin", "payment"];

    for action_class in &classes {
        let mut witness = Witness::new();
        witness.set_enum("action_class", action_class);
        witness.set_enum("approval_tier", "peer");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(
            result,
            "Expected action_class={} to be accepted",
            action_class
        );
    }
}

#[test]
fn all_approval_tiers_accepted() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    for approval_tier in &["self", "peer", "manager", "security"] {
        let mut witness = Witness::new();
        witness.set_enum("action_class", "export");
        witness.set_enum("approval_tier", approval_tier);

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(
            result,
            "Expected approval_tier={} to be accepted",
            approval_tier
        );
    }
}

#[test]
fn accepts_without_optional_auto_approve() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "delete");
    witness.set_enum("approval_tier", "security");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(
        result,
        "Expected omitted max_auto_approve_secs to be accepted"
    );
}

#[test]
fn accepts_typical_human_gate_request() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "payment");
    witness.set_enum("approval_tier", "manager");
    witness.set_u64("max_auto_approve_secs", 300);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected typical human_gate request to be accepted");
}

#[test]
fn accepts_mid_range_auto_approve() {
    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "admin");
    witness.set_enum("approval_tier", "security");
    witness.set_u64("max_auto_approve_secs", 3600);

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");
    assert!(result, "Expected max_auto_approve_secs=3600 to be accepted");
}

#[test]
fn rejects_auto_approve_over_maximum() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "write");
    witness.set_enum("approval_tier", "self");
    witness.set_u64("max_auto_approve_secs", 86401);

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(
            !verified,
            "Expected max_auto_approve_secs=86401 to be rejected"
        );
    }
}

#[test]
fn rejects_invalid_action_class() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "exfiltrate");
    witness.set_enum("approval_tier", "self");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected action_class=exfiltrate to be rejected");
    }
}

#[test]
fn rejects_invalid_approval_tier() {
    use std::panic;

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_enum("action_class", "read");
    witness.set_enum("approval_tier", "anonymous");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    if let Ok(Ok(proof)) = result {
        let verified = verify(&compiled, &proof).unwrap_or(false);
        assert!(!verified, "Expected approval_tier=anonymous to be rejected");
    }
}
