use crate::prove::{CompiledSchema, Proof, ProveError};
use ark_bn254::Bn254;
use ark_groth16::Groth16;
use ark_snark::SNARK;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("verification failed: proof is invalid")]
    InvalidProof,
    #[error("verification error: {0}")]
    VerificationError(String),
    #[error("prove error: {0}")]
    ProveError(#[from] ProveError),
}

pub fn verify(compiled: &CompiledSchema, proof: &Proof) -> Result<bool, VerifyError> {
    let result = Groth16::<Bn254>::verify_with_processed_vk(
        &compiled.verifying_key,
        &proof.public_inputs,
        &proof.proof,
    )
    .map_err(|e| VerifyError::VerificationError(e.to_string()))?;

    Ok(result)
}

pub fn verify_or_err(compiled: &CompiledSchema, proof: &Proof) -> Result<(), VerifyError> {
    if verify(compiled, proof)? {
        Ok(())
    } else {
        Err(VerifyError::InvalidProof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::Witness;
    use crate::schema::FormProofSchema;

    fn test_schema() -> FormProofSchema {
        let schema_json = r#"{
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
        }"#;
        FormProofSchema::from_json(schema_json).unwrap()
    }

    #[test]
    fn test_valid_proof_verifies() {
        let schema = test_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 25);
        witness.set_enum("currency", "USD");

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn test_boundary_value_verifies() {
        let schema = test_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 50);
        witness.set_enum("currency", "EUR");

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn test_zero_value_verifies() {
        let schema = test_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 0);
        witness.set_enum("currency", "GBP");

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }
}
