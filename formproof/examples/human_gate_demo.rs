//! Human Gate Demo Example
//!
//! Demonstrates proving a human-in-the-loop approval policy is satisfied without
//! revealing the exact action class, approval tier, or auto-approve window.
//!
//! The human_gate policy enforces:
//! - action_class: one of 6 risk classes for tool actions
//! - approval_tier: one of 4 human approval tiers
//! - max_auto_approve_secs (optional): 0..86400 cached-approval TTL
//!
//! Run with: cargo run -p formproof --example human_gate_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Human Gate Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving read with self-approval (no auto-approve window)");
    prove_and_verify(&compiled, "read", "self", None, true);

    println!("\n2. Proving write with peer approval (5-minute auto-approve)");
    prove_and_verify(&compiled, "write", "peer", Some(300), true);

    println!("\n3. Proving delete with security approval (strict: 0 auto-approve)");
    prove_and_verify(&compiled, "delete", "security", Some(0), true);

    println!("\n4. Proving export with manager approval (1-hour auto-approve)");
    prove_and_verify(&compiled, "export", "manager", Some(3600), true);

    println!("\n5. Proving payment with manager approval (max 24h auto-approve)");
    prove_and_verify(&compiled, "payment", "manager", Some(86400), true);

    println!("\n6. Proving admin with security approval (no auto-approve window)");
    prove_and_verify(&compiled, "admin", "security", None, true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/human_gate.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline human_gate schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "action_class": {
                    "enum": ["read", "write", "delete", "export", "admin", "payment"]
                },
                "approval_tier": {
                    "enum": ["self", "peer", "manager", "security"]
                },
                "max_auto_approve_secs": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 86400
                }
            },
            "required": ["action_class", "approval_tier"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(
    compiled: &CompiledSchema,
    action_class: &str,
    approval_tier: &str,
    max_auto_approve_secs: Option<u64>,
    expect_valid: bool,
) {
    let mut witness = Witness::new();
    witness.set_enum("action_class", action_class);
    witness.set_enum("approval_tier", approval_tier);
    if let Some(secs) = max_auto_approve_secs {
        witness.set_u64("max_auto_approve_secs", secs);
    }

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    match result {
        Ok(true) => {
            let secs_display = max_auto_approve_secs
                .map(|u| u.to_string())
                .unwrap_or_else(|| "(omitted)".to_string());
            println!(
                "   ✓ Proof verified: action_class={}, approval_tier={}, max_auto_approve_secs={}",
                action_class, approval_tier, secs_display
            );
            assert!(expect_valid, "Unexpected valid proof");
        }
        Ok(false) => {
            println!(
                "   ✗ Proof rejected: action_class={}, approval_tier={}",
                action_class, approval_tier
            );
            assert!(!expect_valid, "Unexpected rejection");
        }
        Err(e) => panic!("Verification error: {:?}", e),
    }
}
