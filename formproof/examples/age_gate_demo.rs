//! Age Gate Demo Example
//!
//! Demonstrates proving age verification satisfies policy constraints
//! without revealing the actual age of the user.
//!
//! The age_gate policy enforces:
//! - age: 18..120 (must be at least 18, maximum 120)
//! - region: one of US, EU, UK, IN
//!
//! Run with: cargo run -p formproof --example age_gate_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Age Gate Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving minimum age boundary: 18 years old in US");
    prove_and_verify(&compiled, 18, "US", true);

    println!("\n2. Proving typical adult: 25 years old in EU");
    prove_and_verify(&compiled, 25, "EU", true);

    println!("\n3. Proving middle-aged user: 45 years old in UK");
    prove_and_verify(&compiled, 45, "UK", true);

    println!("\n4. Proving senior user: 75 years old in IN");
    prove_and_verify(&compiled, 75, "IN", true);

    println!("\n5. Proving maximum age boundary: 120 years old in US");
    prove_and_verify(&compiled, 120, "US", true);

    println!("\n6. Proving young adult: 21 years old in EU (legal drinking age)");
    prove_and_verify(&compiled, 21, "EU", true);

    println!("\n7. Proving typical working adult: 35 years old in UK");
    prove_and_verify(&compiled, 35, "UK", true);

    println!("\n8. Proving retirement age: 65 years old in IN");
    prove_and_verify(&compiled, 65, "IN", true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/age_gate.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline age_gate schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 18,
                    "maximum": 120
                },
                "region": {
                    "enum": ["US", "EU", "UK", "IN"]
                }
            },
            "required": ["age", "region"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(compiled: &CompiledSchema, age: u64, region: &str, expect_valid: bool) {
    let mut witness = Witness::new();
    witness.set_u64("age", age);
    witness.set_enum("region", region);

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    match result {
        Ok(true) => {
            println!("   ✓ Proof verified: age={}, region={}", age, region);
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
