//! Spend Cap Demo Example
//!
//! Demonstrates proving a spending transaction is within policy limits
//! without revealing the actual transaction amount.
//!
//! The spend_cap policy enforces:
//! - cents: 0..10000 (up to $100)
//! - currency: one of USD, EUR, GBP, INR
//!
//! Run with: cargo run -p formproof --example spend_cap_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Spend Cap Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving a valid transaction: $75.00 USD");
    prove_and_verify(&compiled, 7500, "USD", true);

    println!("\n2. Proving boundary value: exactly $100.00 EUR");
    prove_and_verify(&compiled, 10000, "EUR", true);

    println!("\n3. Proving minimum value: $0.00 GBP");
    prove_and_verify(&compiled, 0, "GBP", true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/spend_cap.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline spend_cap schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "cents": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 10000
                },
                "currency": {
                    "enum": ["USD", "EUR", "GBP", "INR"]
                }
            },
            "required": ["cents", "currency"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(compiled: &CompiledSchema, cents: u64, currency: &str, expect_valid: bool) {
    let mut witness = Witness::new();
    witness.set_u64("cents", cents);
    witness.set_enum("currency", currency);

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    match result {
        Ok(true) => {
            println!("   ✓ Proof verified: {} cents in {}", cents, currency);
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
