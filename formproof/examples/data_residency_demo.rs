//! Data Residency Demo Example
//!
//! Demonstrates proving a data-residency policy is satisfied without revealing
//! the exact region, storage class, or retention configuration.
//!
//! The data_residency policy enforces:
//! - region: one of 8 allowed placement regions
//! - storage_class: hot / warm / cold / archive
//! - retention_days: 1..3650
//! - cross_border (optional): forbidden / allowed / eu-only
//!
//! Run with: cargo run -p formproof --example data_residency_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Data Residency Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving US east hot storage with 30-day retention (cross-border forbidden)");
    prove_and_verify(&compiled, "us-east", "hot", 30, Some("forbidden"), true);

    println!("\n2. Proving EU west cold storage with 730-day retention (eu-only)");
    prove_and_verify(&compiled, "eu-west", "cold", 730, Some("eu-only"), true);

    println!("\n3. Proving AP south warm storage with 90-day retention (no cross_border)");
    prove_and_verify(&compiled, "ap-south", "warm", 90, None, true);

    println!("\n4. Proving minimum retention boundary: 1 day archive in ca-central");
    prove_and_verify(&compiled, "ca-central", "archive", 1, Some("allowed"), true);

    println!("\n5. Proving maximum retention boundary: 3650 days archive in eu-central");
    prove_and_verify(
        &compiled,
        "eu-central",
        "archive",
        3650,
        Some("eu-only"),
        true,
    );

    println!("\n6. Proving SA east hot storage with typical weekly retention");
    prove_and_verify(&compiled, "sa-east", "hot", 7, Some("allowed"), true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/data_residency.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline data_residency schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "region": {
                    "enum": ["us-east", "us-west", "eu-west", "eu-central", "ap-south", "ap-northeast", "sa-east", "ca-central"]
                },
                "storage_class": {
                    "enum": ["hot", "warm", "cold", "archive"]
                },
                "retention_days": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 3650
                },
                "cross_border": {
                    "enum": ["forbidden", "allowed", "eu-only"]
                }
            },
            "required": ["region", "storage_class", "retention_days"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(
    compiled: &CompiledSchema,
    region: &str,
    storage_class: &str,
    retention_days: u64,
    cross_border: Option<&str>,
    expect_valid: bool,
) {
    let mut witness = Witness::new();
    witness.set_enum("region", region);
    witness.set_enum("storage_class", storage_class);
    witness.set_u64("retention_days", retention_days);
    if let Some(cb) = cross_border {
        witness.set_enum("cross_border", cb);
    }

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    match result {
        Ok(true) => {
            let cb_display = cross_border.unwrap_or("(omitted)");
            println!(
                "   ✓ Proof verified: region={}, storage={}, retention={}d, cross_border={}",
                region, storage_class, retention_days, cb_display
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
