//! Proof Package Demo Example
//!
//! Demonstrates creating and verifying portable proof packages.
//! A package bundles proof + commitment + schema fingerprint into
//! a single JSON artifact for easy transport.
//!
//! Run with: cargo run -p formproof --example proof_package_demo

use formproof::{schema_fingerprint, CompiledSchema, FormProofSchema, ProofPackage, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Proof Package Demo ===\n");

    let schema = load_schema();
    let fingerprint = schema_fingerprint(&schema);
    println!("Schema fingerprint: {}", hex::encode(fingerprint));

    let compiled = CompiledSchema::compile(schema.clone()).expect("Compilation failed");

    println!("\n1. Creating a proof package for rate limit policy");
    let package = create_package(&compiled, 100, 3600, "pro");

    println!("\n2. Serializing package to JSON");
    let json = package.to_json().expect("Serialization failed");
    println!("   Package size: {} bytes", json.len());
    println!("   First 200 chars:\n   {}", &json[..json.len().min(200)]);

    println!("\n3. Simulating transport: deserialize from JSON");
    let loaded = ProofPackage::from_json(&json).expect("Deserialization failed");
    println!("   Version: {}", loaded.version);
    println!("   Commitment: {}...", &loaded.commitment_hex[..16]);
    println!("   Fingerprint: {}...", &loaded.schema_fingerprint[..16]);

    println!("\n4. Verifying the package");
    match loaded.verify(&compiled) {
        Ok(()) => println!("   ✓ Package verified successfully"),
        Err(e) => println!("   ✗ Verification failed: {}", e),
    }

    println!("\n5. Testing tamper detection: modify commitment");
    let mut tampered_json = json.clone();
    tampered_json = tampered_json.replace(
        &loaded.commitment_hex[..8],
        "deadbeef",
    );
    let tampered = ProofPackage::from_json(&tampered_json).expect("Deserialization failed");
    match tampered.verify(&compiled) {
        Ok(()) => println!("   ✗ Should have failed but passed"),
        Err(e) => println!("   ✓ Tamper detected: {}", e),
    }

    println!("\n6. Testing wrong schema detection");
    let other_schema = FormProofSchema::from_json(
        r#"{"type":"object","properties":{"x":{"type":"integer"}},"required":["x"]}"#,
    )
    .unwrap();
    let other_compiled = CompiledSchema::compile(other_schema).expect("Compilation failed");
    match loaded.verify(&other_compiled) {
        Ok(()) => println!("   ✗ Should have failed but passed"),
        Err(e) => println!("   ✓ Wrong schema detected: {}", e),
    }

    println!("\n7. Compact JSON for bandwidth efficiency");
    let compact = package.to_json_compact().expect("Serialization failed");
    println!("   Pretty JSON: {} bytes", json.len());
    println!("   Compact JSON: {} bytes", compact.len());
    println!("   Savings: {} bytes ({:.1}%)",
        json.len() - compact.len(),
        (json.len() - compact.len()) as f64 / json.len() as f64 * 100.0
    );

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/rate_limit.json");

    if schema_path.exists() {
        println!("Loading rate_limit schema from {:?}", schema_path);
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

fn create_package(compiled: &CompiledSchema, requests: u64, window: u64, tier: &str) -> ProofPackage {
    println!("   requests_per_window: {}", requests);
    println!("   window_secs: {}", window);
    println!("   tier: {}", tier);

    let mut witness = Witness::new();
    witness.set_u64("requests_per_window", requests);
    witness.set_u64("window_secs", window);
    witness.set_enum("tier", tier);

    ProofPackage::create(compiled, &witness).expect("Package creation failed")
}
