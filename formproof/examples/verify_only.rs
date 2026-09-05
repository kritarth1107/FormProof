//! Verify-Only Host Example
//!
//! Demonstrates the host-side verification path: load a verifying key,
//! deserialize a proof, and verify it against a commitment.
//!
//! In production, the host would:
//! 1. Compile the schema once and save the verifying key
//! 2. Receive proofs from provers (e.g., over HTTP)
//! 3. Verify each proof using only the verifying key (not the proving key)
//!
//! Run with: cargo run --example verify_only

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};

fn main() {
    println!("=== Verify-Only Host Example ===\n");

    // The schema is public and shared between prover and verifier.
    let schema_json = r#"{
        "type": "object",
        "properties": {
            "amount": {
                "type": "integer",
                "minimum": 0,
                "maximum": 100
            },
            "status": {
                "enum": ["pending", "approved", "rejected"]
            }
        },
        "required": ["amount", "status"]
    }"#;

    let schema = FormProofSchema::from_json(schema_json).expect("Invalid schema");

    // --- Setup Phase (done once) ---
    println!("1. Setup: Compile schema and export keys");
    let compiled = CompiledSchema::compile(schema.clone()).expect("Compilation failed");

    let vk_bytes = compiled
        .serialize_verifying_key()
        .expect("Failed to serialize VK");
    let pk_bytes = compiled
        .serialize_proving_key()
        .expect("Failed to serialize PK");

    println!("   Verifying key: {} bytes", vk_bytes.len());
    println!(
        "   Proving key: {} bytes (prover keeps this)\n",
        pk_bytes.len()
    );

    // --- Prover Side (runs separately) ---
    println!("2. Prover: Create witness and generate proof");

    let mut witness = Witness::new();
    witness.set_u64("amount", 75);
    witness.set_enum("status", "approved");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let proof_bytes = proof.serialize().expect("Proof serialization failed");
    let commitment = proof.commitment;

    println!("   Proof: {} bytes", proof_bytes.len());
    println!("   Commitment: {}\n", hex::encode(commitment));

    // --- Verifier Side (host receives proof + commitment) ---
    println!("3. Verifier: Load VK and verify proof");
    println!("   (In production, VK is loaded from storage, proof from network)\n");

    // Reconstruct from serialized bytes (simulating separate processes)
    let loaded_compiled = CompiledSchema::deserialize_keys(schema, &pk_bytes, &vk_bytes)
        .expect("Key deserialization failed");

    let loaded_proof =
        Proof::deserialize(&proof_bytes, commitment).expect("Proof deserialization failed");

    // Verify!
    let result = verify(&loaded_compiled, &loaded_proof);

    match result {
        Ok(true) => {
            println!("   ✓ Proof VALID");
            println!("   The witness satisfies all schema constraints.");
            println!("   Verifier learned nothing about the actual values.");
        }
        Ok(false) => {
            println!("   ✗ Proof INVALID");
            println!("   The proof does not satisfy the circuit constraints.");
        }
        Err(e) => {
            println!("   ✗ Verification ERROR: {}", e);
        }
    }

    // --- Demonstrate rejection ---
    println!("\n4. Demonstrate: Tampered commitment fails");

    let mut bad_commitment = commitment;
    bad_commitment[0] ^= 0xff; // Flip bits in first byte

    let tampered_proof =
        Proof::deserialize(&proof_bytes, bad_commitment).expect("Proof deserialization failed");

    match verify(&loaded_compiled, &tampered_proof) {
        Ok(true) => println!("   ✗ Unexpected: tampered proof verified!"),
        Ok(false) => println!("   ✓ Tampered commitment correctly rejected"),
        Err(e) => println!("   ✓ Tampered commitment caused error: {}", e),
    }

    println!("\n=== Done ===");
}
