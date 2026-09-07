//! Purpose Bind Demo Example
//!
//! Demonstrates proving a purpose-limitation policy is satisfied without
//! revealing the exact processing purpose, legal basis, or secondary-use bound.
//!
//! The purpose_bind policy enforces:
//! - purpose: one of 6 allowed processing purposes
//! - legal_basis: one of 4 GDPR-style legal bases
//! - max_secondary_uses (optional): 0..8 secondary use limit
//!
//! Run with: cargo run -p formproof --example purpose_bind_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Purpose Bind Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving inference with consent (no secondary uses specified)");
    prove_and_verify(&compiled, "inference", "consent", None, true);

    println!("\n2. Proving analytics with legitimate interest (2 secondary uses)");
    prove_and_verify(&compiled, "analytics", "legitimate_interest", Some(2), true);

    println!("\n3. Proving training with contract (strict: 0 secondary uses)");
    prove_and_verify(&compiled, "training", "contract", Some(0), true);

    println!("\n4. Proving support with legal obligation (max 8 secondary uses)");
    prove_and_verify(&compiled, "support", "legal_obligation", Some(8), true);

    println!("\n5. Proving billing with consent (4 secondary uses)");
    prove_and_verify(&compiled, "billing", "consent", Some(4), true);

    println!("\n6. Proving debugging with contract (no secondary uses specified)");
    prove_and_verify(&compiled, "debugging", "contract", None, true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/purpose_bind.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline purpose_bind schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "purpose": {
                    "enum": ["inference", "analytics", "training", "support", "billing", "debugging"]
                },
                "legal_basis": {
                    "enum": ["consent", "contract", "legitimate_interest", "legal_obligation"]
                },
                "max_secondary_uses": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 8
                }
            },
            "required": ["purpose", "legal_basis"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(
    compiled: &CompiledSchema,
    purpose: &str,
    legal_basis: &str,
    max_secondary_uses: Option<u64>,
    expect_valid: bool,
) {
    let mut witness = Witness::new();
    witness.set_enum("purpose", purpose);
    witness.set_enum("legal_basis", legal_basis);
    if let Some(uses) = max_secondary_uses {
        witness.set_u64("max_secondary_uses", uses);
    }

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    match result {
        Ok(true) => {
            let uses_display = max_secondary_uses
                .map(|u| u.to_string())
                .unwrap_or_else(|| "(omitted)".to_string());
            println!(
                "   ✓ Proof verified: purpose={}, legal_basis={}, max_secondary_uses={}",
                purpose, legal_basis, uses_display
            );
            assert!(expect_valid, "Expected proof to fail but it passed");
        }
        Ok(false) => {
            println!("   ✗ Proof rejected");
            assert!(!expect_valid, "Expected proof to pass but it was rejected");
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
            assert!(!expect_valid, "Expected proof to pass but got error");
        }
    }
}
