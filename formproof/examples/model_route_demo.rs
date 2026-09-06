//! Model Route Demo Example
//!
//! Demonstrates proving a model routing request satisfies policy constraints
//! without revealing the actual model selection or token limits.
//!
//! The model_route policy enforces:
//! - model_id: one of 8 allowed model identifiers
//! - max_tokens: 1..128000 (output token limit)
//! - priority: one of low, normal, high, critical
//! - temperature_class (optional): deterministic, balanced, creative
//!
//! Run with: cargo run -p formproof --example model_route_demo

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Model Route Demo ===\n");

    let schema = load_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    println!("1. Proving a standard GPT-4o request: 4K tokens, normal priority");
    prove_and_verify(&compiled, "gpt-4o", 4096, "normal", None, true);

    println!("\n2. Proving a high-priority Claude request with deterministic output");
    prove_and_verify(
        &compiled,
        "claude-3-opus",
        16384,
        "high",
        Some("deterministic"),
        true,
    );

    println!("\n3. Proving minimum tokens: 1 token request");
    prove_and_verify(&compiled, "gpt-4o-mini", 1, "low", None, true);

    println!("\n4. Proving maximum tokens: 128K context window");
    prove_and_verify(&compiled, "claude-3-sonnet", 128000, "critical", None, true);

    println!("\n5. Proving creative writing request with balanced temperature");
    prove_and_verify(
        &compiled,
        "claude-3-haiku",
        8192,
        "normal",
        Some("creative"),
        true,
    );

    println!("\n6. Proving low-priority batch processing with Mixtral");
    prove_and_verify(&compiled, "mixtral", 32768, "low", Some("balanced"), true);

    println!("\n7. Proving Gemini Pro request at normal priority");
    prove_and_verify(&compiled, "gemini-pro", 8192, "normal", None, true);

    println!("\n8. Proving Llama-3 code generation (deterministic)");
    prove_and_verify(
        &compiled,
        "llama-3",
        4096,
        "high",
        Some("deterministic"),
        true,
    );

    println!("\n=== Done ===");
}

fn load_schema() -> FormProofSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas/model_route.json");

    if schema_path.exists() {
        println!("Loading schema from {:?}", schema_path);
        let json = fs::read_to_string(&schema_path).expect("Failed to read schema file");
        FormProofSchema::from_json(&json).expect("Invalid schema")
    } else {
        println!("Using inline model_route schema");
        let inline = r#"{
            "type": "object",
            "properties": {
                "model_id": {
                    "enum": ["gpt-4o", "gpt-4o-mini", "claude-3-opus", "claude-3-sonnet", "claude-3-haiku", "gemini-pro", "llama-3", "mixtral"]
                },
                "max_tokens": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 128000
                },
                "priority": {
                    "enum": ["low", "normal", "high", "critical"]
                },
                "temperature_class": {
                    "enum": ["deterministic", "balanced", "creative"]
                }
            },
            "required": ["model_id", "max_tokens", "priority"]
        }"#;
        FormProofSchema::from_json(inline).expect("Invalid inline schema")
    }
}

fn prove_and_verify(
    compiled: &CompiledSchema,
    model_id: &str,
    max_tokens: u64,
    priority: &str,
    temperature_class: Option<&str>,
    expect_valid: bool,
) {
    let mut witness = Witness::new();
    witness.set_enum("model_id", model_id);
    witness.set_u64("max_tokens", max_tokens);
    witness.set_enum("priority", priority);
    if let Some(temp) = temperature_class {
        witness.set_enum("temperature_class", temp);
    }

    let proof = Proof::create(compiled, &witness).expect("Proof generation failed");
    let result = verify(compiled, &proof);

    let tokens_display = format_tokens(max_tokens);

    match result {
        Ok(true) => {
            if let Some(temp) = temperature_class {
                println!(
                    "   ✓ Proof verified: model={}, tokens={}, priority={}, temp={}",
                    model_id, tokens_display, priority, temp
                );
            } else {
                println!(
                    "   ✓ Proof verified: model={}, tokens={}, priority={}",
                    model_id, tokens_display, priority
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

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1000 {
        format!("{}K", tokens / 1000)
    } else {
        format!("{}", tokens)
    }
}
