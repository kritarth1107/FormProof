//! Tool Allowlist Demo Example
//!
//! Demonstrates proving a tool access request satisfies allowlist constraints
//! without revealing the actual tool configuration or scope details.
//!
//! The tool_allowlist policy enforces:
//! - tool_name: one of read, write, execute, list, search, delete
//! - max_args (optional): 0..64 (maximum number of arguments)
//! - scope: one of local, remote, any
//!
//! Run with: cargo run -p formproof --example tool_allowlist_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Tool Allowlist Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving read tool with local scope");
    prove_and_verify(&compiled, "read", None, "local", true);

    println!("\n2. Proving write tool with max 10 args, remote scope");
    prove_and_verify(&compiled, "write", Some(10), "remote", true);

    println!("\n3. Proving execute tool with max 64 args (boundary), any scope");
    prove_and_verify(&compiled, "execute", Some(64), "any", true);

    println!("\n4. Proving list tool with no args constraint, local scope");
    prove_and_verify(&compiled, "list", None, "local", true);

    println!("\n5. Proving search tool with max 5 args, remote scope");
    prove_and_verify(&compiled, "search", Some(5), "remote", true);

    println!("\n6. Proving delete tool with zero max_args (boundary), local scope");
    prove_and_verify(&compiled, "delete", Some(0), "local", true);

    println!("\n7. Proving read tool with any scope and moderate args");
    prove_and_verify(&compiled, "read", Some(32), "any", true);

    println!("\n8. Proving write tool with minimal args, any scope");
    prove_and_verify(&compiled, "write", Some(1), "any", true);

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/tool_allowlist.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline tool_allowlist schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "tool_name": {
                    "enum": ["read", "write", "execute", "list", "search", "delete"]
                },
                "max_args": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 64
                },
                "scope": {
                    "enum": ["local", "remote", "any"]
                }
            },
            "required": ["tool_name", "scope"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(
    compiled: &CompiledSchema,
    tool_name: &str,
    max_args: Option<u64>,
    scope: &str,
    expect_valid: bool,
) {
    let mut witness = Witness::new();
    witness.set_enum("tool_name", tool_name);
    witness.set_enum("scope", scope);
    if let Some(args) = max_args {
        witness.set_u64("max_args", args);
    }

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    match result {
        Ok(true) => {
            if let Some(args) = max_args {
                println!(
                    "   ✓ Proof verified: tool={}, max_args={}, scope={}",
                    tool_name, args, scope
                );
            } else {
                println!(
                    "   ✓ Proof verified: tool={}, max_args=unlimited, scope={}",
                    tool_name, scope
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
