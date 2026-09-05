//! Boundary tests for spend_cap schema constraints.
//!
//! Verifies that the spend_cap policy correctly accepts boundary values
//! and rejects out-of-range values.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};

fn spend_cap_schema() -> FormProofSchema {
    FormProofSchema::from_json(
        r#"{
            "type": "object",
            "properties": {
                "cents": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 10000
                },
                "currency": {
                    "enum": ["USD", "EUR", "GBP", "INR"]
                }
            },
            "required": ["cents", "currency"]
        }"#,
    )
    .expect("Invalid spend_cap schema")
}

#[test]
fn accepts_boundary_10000() {
    let schema = spend_cap_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("cents", 10000);
    witness.set_enum("currency", "USD");

    let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
    let result = verify(&compiled, &proof).expect("Verification failed");

    assert!(
        result,
        "Expected cents=10000 to be accepted (boundary value)"
    );
}

#[test]
fn rejects_cents_over_10000() {
    use std::panic;

    let schema = spend_cap_schema();
    let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

    let mut witness = Witness::new();
    witness.set_u64("cents", 10001);
    witness.set_enum("currency", "USD");

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        Proof::create(&compiled, &witness)
    }));

    match result {
        Ok(Ok(proof)) => {
            let verified = verify(&compiled, &proof).unwrap_or(false);
            assert!(
                !verified,
                "Expected cents=10001 to be rejected (over maximum)"
            );
        }
        Ok(Err(_)) => {
            // Proof creation returning an error is acceptable for out-of-range values
        }
        Err(_) => {
            // Proof creation panicking (circuit unsatisfied) is also acceptable
        }
    }
}
