//! Rate Limit Demo Example
//!
//! Demonstrates proving a rate-limit policy is satisfied
//! without revealing the actual request counts or configuration.
//!
//! The rate_limit policy enforces:
//! - requests_per_window: 1..10000 (requests allowed per time window)
//! - window_secs: 1..86400 (window duration in seconds, max 24h)
//! - tier: one of free, basic, pro, enterprise
//!
//! Run with: cargo run -p formproof --example rate_limit_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Rate Limit Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving free tier: 100 requests per 3600s (1 hour)");
    prove_and_verify(&compiled, 100, 3600, "free", true);

    println!("\n2. Proving basic tier: 1000 requests per 60s (1 minute)");
    prove_and_verify(&compiled, 1000, 60, "basic", true);

    println!("\n3. Proving pro tier: 5000 requests per 300s (5 minutes)");
    prove_and_verify(&compiled, 5000, 300, "pro", true);

    println!("\n4. Proving enterprise tier with maximum window: 10000 requests per 86400s (24h)");
    prove_and_verify(&compiled, 10000, 86400, "enterprise", true);

    println!("\n5. Proving minimum boundary: 1 request per 1 second");
    prove_and_verify(&compiled, 1, 1, "free", true);

    println!("\n6. Proving typical API rate limit: 500 requests per 60s");
    prove_and_verify(&compiled, 500, 60, "pro", true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/rate_limit.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline rate_limit schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "requests_per_window": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 10000
                },
                "window_secs": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 86400
                },
                "tier": {
                    "enum": ["free", "basic", "pro", "enterprise"]
                }
            },
            "required": ["requests_per_window", "window_secs", "tier"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(
    compiled: &CompiledSchema,
    requests_per_window: u64,
    window_secs: u64,
    tier: &str,
    expect_valid: bool,
) {
    let mut witness = Witness::new();
    witness.set_u64("requests_per_window", requests_per_window);
    witness.set_u64("window_secs", window_secs);
    witness.set_enum("tier", tier);

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    let window_display = format_window(window_secs);

    match result {
        Ok(true) => {
            println!(
                "   ✓ Proof verified: {} req/{}, tier={}",
                requests_per_window, window_display, tier
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

fn format_window(secs: u64) -> String {
    if secs >= 86400 {
        format!("{}d", secs / 86400)
    } else if secs >= 3600 {
        format!("{}h", secs / 3600)
    } else if secs >= 60 {
        format!("{}m", secs / 60)
    } else {
        format!("{}s", secs)
    }
}
