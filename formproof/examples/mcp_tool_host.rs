//! MCP Tool Host Example
//!
//! Demonstrates how a tool host can verify that an agent's refund request
//! satisfies policy constraints (amount ≤ $50, valid currency) without
//! seeing the actual refund amount.
//!
//! Run with: cargo run --example mcp_tool_host

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};

fn main() {
    println!("=== MCP Tool Host: Zero-Knowledge Refund Verification ===\n");

    // TOOL HOST SETUP
    // ----------------
    // The tool host defines the policy schema that all refund requests must satisfy.
    // This schema is PUBLIC - both the agent and host know it.

    println!("1. Tool host defines refund policy schema:");
    let policy_schema = FormProofSchema::from_json(
        r#"{
        "type": "object",
        "properties": {
            "amount": {
                "type": "integer",
                "minimum": 0,
                "maximum": 50
            },
            "currency": {
                "enum": ["USD", "EUR", "GBP"]
            }
        },
        "required": ["amount", "currency"]
    }"#,
    )
    .expect("Invalid schema");

    println!("   - amount: integer, 0 ≤ amount ≤ 50");
    println!("   - currency: one of [USD, EUR, GBP]");
    println!("   - Both fields required\n");

    // Compile the schema into proving/verifying keys.
    // In production, the host would do this once and cache the keys.
    println!("2. Tool host compiles schema to Groth16 circuit...");
    let compiled = CompiledSchema::compile(policy_schema).expect("Failed to compile schema");
    println!("   Done. Verifying key ready.\n");

    // AGENT SIDE (would run on agent's machine)
    // ------------------------------------------
    // The agent has a refund request with actual values.
    // These values are PRIVATE - the host never sees them.

    println!("3. Agent prepares refund request (PRIVATE data):");
    println!("   amount: $42 (host will NOT see this)");
    println!("   currency: USD\n");

    let mut witness = Witness::new();
    witness.set_u64("amount", 42); // The actual refund amount
    witness.set_enum("currency", "USD");

    // Agent generates a proof that their request satisfies the policy.
    println!("4. Agent generates zero-knowledge proof...");
    let proof = Proof::create(&compiled, &witness).expect("Failed to create proof");

    // The commitment is a hash of the witness data - it's public but reveals nothing.
    let commitment_hex = hex::encode(proof.commitment);
    println!("   Proof generated.");
    println!("   Commitment: {}...\n", &commitment_hex[..16]);

    // VERIFICATION (tool host side)
    // ------------------------------
    // The host receives: proof + commitment
    // The host does NOT receive: the actual amount or currency values

    println!("5. Tool host receives proof and commitment (not the values)");
    println!("   Host verifying proof...");

    let is_valid = verify(&compiled, &proof).expect("Verification error");

    if is_valid {
        println!("\n   ✓ PROOF VALID");
        println!("   The refund request satisfies all policy constraints:");
        println!("   - Amount is between $0 and $50");
        println!("   - Currency is one of USD/EUR/GBP");
        println!("   - All required fields are present");
        println!("\n   The tool host can now process the refund,");
        println!("   knowing the policy is satisfied WITHOUT seeing the amount.");
    } else {
        println!("\n   ✗ PROOF INVALID - Request rejected");
    }

    // DEMONSTRATION: Invalid request would be rejected
    println!("\n--- Bonus: What happens with an invalid request ---\n");

    println!("Agent tries to request $100 refund (exceeds $50 limit):");
    let mut bad_witness = Witness::new();
    bad_witness.set_u64("amount", 100); // Over the limit!
    bad_witness.set_enum("currency", "USD");

    // The proof creation succeeds, but the circuit constraints are not satisfied.
    // When compiled in debug mode, this would fail at proof creation.
    // In release mode, the proof is created but verification will fail.
    println!("   (In a real scenario, the agent's prover would fail to");
    println!("    generate a valid proof for an invalid witness.)");

    println!("\n=== Demo Complete ===");
}
