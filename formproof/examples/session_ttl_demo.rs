//! Session TTL Demo Example
//!
//! Demonstrates proving a session TTL configuration is within policy limits
//! without revealing the actual TTL value.
//!
//! The session_ttl policy enforces:
//! - ttl_seconds: 1..86400 (1 second to 24 hours)
//! - tier: one of free, pro, enterprise
//!
//! Run with: cargo run -p formproof --example session_ttl_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Session TTL Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving a typical session: 1 hour TTL, pro tier");
    prove_and_verify(&compiled, 3600, "pro", true);

    println!("\n2. Proving minimum TTL: 1 second, free tier");
    prove_and_verify(&compiled, 1, "free", true);

    println!("\n3. Proving maximum TTL: 24 hours (86400s), enterprise tier");
    prove_and_verify(&compiled, 86400, "enterprise", true);

    println!("\n4. Proving 15-minute session: free tier");
    prove_and_verify(&compiled, 900, "free", true);

    println!("\n5. Proving 8-hour workday session: pro tier");
    prove_and_verify(&compiled, 28800, "pro", true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/session_ttl.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline session_ttl schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "ttl_seconds": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 86400
                },
                "tier": {
                    "enum": ["free", "pro", "enterprise"]
                }
            },
            "required": ["ttl_seconds", "tier"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(compiled: &CompiledSchema, ttl_seconds: u64, tier: &str, expect_valid: bool) {
    let mut witness = Witness::new();
    witness.set_u64("ttl_seconds", ttl_seconds);
    witness.set_enum("tier", tier);

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    let ttl_display = if ttl_seconds >= 3600 {
        format!("{}h {}m", ttl_seconds / 3600, (ttl_seconds % 3600) / 60)
    } else if ttl_seconds >= 60 {
        format!("{}m {}s", ttl_seconds / 60, ttl_seconds % 60)
    } else {
        format!("{}s", ttl_seconds)
    };

    match result {
        Ok(true) => {
            println!(
                "   ✓ Proof verified: TTL={} ({}), tier={}",
                ttl_seconds, ttl_display, tier
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
