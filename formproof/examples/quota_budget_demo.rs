//! Quota Budget Demo Example
//!
//! Demonstrates proving a quota/budget policy configuration satisfies limits
//! without revealing the actual budget values.
//!
//! The quota_budget policy enforces:
//! - budget_units: 1..1000000 (positive budget allocation)
//! - period: one of daily, weekly, monthly
//! - soft_cap (optional): 0..1000000 (warning threshold)
//!
//! Run with: cargo run -p formproof --example quota_budget_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Quota Budget Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving a minimal daily budget: 1 unit");
    prove_and_verify(&compiled, 1, "daily", None, true);

    println!("\n2. Proving maximum monthly budget: 1,000,000 units");
    prove_and_verify(&compiled, 1000000, "monthly", None, true);

    println!("\n3. Proving weekly budget with soft cap warning threshold");
    prove_and_verify(&compiled, 50000, "weekly", Some(40000), true);

    println!("\n4. Proving daily API quota: 10,000 requests/day");
    prove_and_verify(&compiled, 10000, "daily", None, true);

    println!("\n5. Proving enterprise monthly budget with 80% soft cap");
    prove_and_verify(&compiled, 500000, "monthly", Some(400000), true);

    println!("\n6. Proving soft cap at zero (no warnings)");
    prove_and_verify(&compiled, 25000, "weekly", Some(0), true);

    println!("\n7. Proving soft cap equals budget (immediate warning)");
    prove_and_verify(&compiled, 100000, "monthly", Some(100000), true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/quota_budget.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline quota_budget schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "budget_units": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 1000000
                },
                "period": {
                    "enum": ["daily", "weekly", "monthly"]
                },
                "soft_cap": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 1000000
                }
            },
            "required": ["budget_units", "period"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(
    compiled: &CompiledSchema,
    budget_units: u64,
    period: &str,
    soft_cap: Option<u64>,
    expect_valid: bool,
) {
    let mut witness = Witness::new();
    witness.set_u64("budget_units", budget_units);
    witness.set_enum("period", period);
    if let Some(cap) = soft_cap {
        witness.set_u64("soft_cap", cap);
    }

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    let budget_display = format_budget(budget_units);
    let cap_display = soft_cap.map(format_budget);

    match result {
        Ok(true) => {
            if let Some(cap) = cap_display {
                println!(
                    "   ✓ Proof verified: budget={}, period={}, soft_cap={}",
                    budget_display, period, cap
                );
            } else {
                println!(
                    "   ✓ Proof verified: budget={}, period={}",
                    budget_display, period
                );
            }
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

fn format_budget(units: u64) -> String {
    if units >= 1_000_000 {
        format!("{}M", units / 1_000_000)
    } else if units >= 1_000 {
        format!("{}K", units / 1_000)
    } else {
        format!("{}", units)
    }
}
