//! Groth16 proof generation for FormProof.
//!
//! This module provides schema compilation and proof generation using
//! the Groth16 proving system with the BN254 curve.

use crate::circuit::{FormProofCircuit, Witness};
use crate::schema::FormProofSchema;
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, PreparedVerifyingKey, ProvingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use ark_std::rand::rngs::OsRng;
use thiserror::Error;

/// Errors that can occur during proving or compilation.
#[derive(Error, Debug)]
pub enum ProveError {
    /// Error serializing or deserializing keys/proofs.
    #[error("serialization error: {0}")]
    Serialization(String),
    /// Error during proof generation.
    #[error("proving error: {0}")]
    Proving(String),
    /// Error during circuit setup.
    #[error("setup error: {0}")]
    Setup(String),
}

/// A compiled schema with proving and verifying keys.
///
/// This is created by compiling a schema once, then used to create
/// multiple proofs and verify them.
#[derive(Clone)]
pub struct CompiledSchema {
    /// The original schema.
    pub schema: FormProofSchema,
    /// The Groth16 proving key.
    pub proving_key: ProvingKey<Bn254>,
    /// The Groth16 verifying key (preprocessed).
    pub verifying_key: PreparedVerifyingKey<Bn254>,
}

impl CompiledSchema {
    /// Compile a schema into proving and verifying keys.
    ///
    /// This performs Groth16 circuit-specific setup, which is computationally
    /// expensive. The result should be cached and reused.
    ///
    /// # Errors
    ///
    /// Returns an error if circuit setup fails.
    pub fn compile(schema: FormProofSchema) -> Result<Self, ProveError> {
        let circuit = FormProofCircuit::<Fr>::new(schema.clone());

        let mut rng = OsRng;
        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)
            .map_err(|e| ProveError::Setup(e.to_string()))?;

        let pvk =
            Groth16::<Bn254>::process_vk(&vk).map_err(|e| ProveError::Setup(e.to_string()))?;

        Ok(CompiledSchema {
            schema,
            proving_key: pk,
            verifying_key: pvk,
        })
    }

    /// Serialize the proving key to bytes.
    pub fn serialize_proving_key(&self) -> Result<Vec<u8>, ProveError> {
        let mut bytes = Vec::new();
        self.proving_key
            .serialize_compressed(&mut bytes)
            .map_err(|e| ProveError::Serialization(e.to_string()))?;
        Ok(bytes)
    }

    /// Serialize the verifying key to bytes.
    pub fn serialize_verifying_key(&self) -> Result<Vec<u8>, ProveError> {
        let mut bytes = Vec::new();
        self.verifying_key
            .serialize_compressed(&mut bytes)
            .map_err(|e| ProveError::Serialization(e.to_string()))?;
        Ok(bytes)
    }

    /// Deserialize keys from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn deserialize_keys(
        schema: FormProofSchema,
        pk_bytes: &[u8],
        vk_bytes: &[u8],
    ) -> Result<Self, ProveError> {
        let proving_key = ProvingKey::<Bn254>::deserialize_compressed(pk_bytes)
            .map_err(|e| ProveError::Serialization(e.to_string()))?;
        let verifying_key = PreparedVerifyingKey::<Bn254>::deserialize_compressed(vk_bytes)
            .map_err(|e| ProveError::Serialization(e.to_string()))?;

        Ok(CompiledSchema {
            schema,
            proving_key,
            verifying_key,
        })
    }
}

/// A zero-knowledge proof that a witness satisfies a schema.
pub struct Proof {
    /// The Groth16 proof.
    pub proof: ark_groth16::Proof<Bn254>,
    /// Public inputs (the commitment bytes as field elements).
    pub public_inputs: Vec<Fr>,
    /// The commitment to the witness data.
    pub commitment: [u8; 32],
}

impl Proof {
    /// Create a proof that the witness satisfies the compiled schema.
    ///
    /// # Errors
    ///
    /// Returns an error if proof generation fails.
    pub fn create(compiled: &CompiledSchema, witness: &Witness) -> Result<Self, ProveError> {
        let commitment = witness.commitment(&compiled.schema);

        let circuit =
            FormProofCircuit::<Fr>::new(compiled.schema.clone()).with_witness(witness.clone());

        let public_inputs: Vec<Fr> = commitment.iter().map(|&b| Fr::from(b as u64)).collect();

        let mut rng = OsRng;
        let proof = Groth16::<Bn254>::prove(&compiled.proving_key, circuit, &mut rng)
            .map_err(|e| ProveError::Proving(e.to_string()))?;

        Ok(Proof {
            proof,
            public_inputs,
            commitment,
        })
    }

    /// Serialize the proof to bytes.
    pub fn serialize(&self) -> Result<Vec<u8>, ProveError> {
        let mut bytes = Vec::new();
        self.proof
            .serialize_compressed(&mut bytes)
            .map_err(|e| ProveError::Serialization(e.to_string()))?;
        Ok(bytes)
    }

    /// Deserialize a proof from bytes.
    ///
    /// The commitment must be provided separately as it is not included
    /// in the serialized proof.
    pub fn deserialize(bytes: &[u8], commitment: [u8; 32]) -> Result<Self, ProveError> {
        let proof = ark_groth16::Proof::<Bn254>::deserialize_compressed(bytes)
            .map_err(|e| ProveError::Serialization(e.to_string()))?;

        let public_inputs: Vec<Fr> = commitment.iter().map(|&b| Fr::from(b as u64)).collect();

        Ok(Proof {
            proof,
            public_inputs,
            commitment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_schema() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100
                }
            },
            "required": ["amount"]
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let pk_bytes = compiled.serialize_proving_key().unwrap();
        let vk_bytes = compiled.serialize_verifying_key().unwrap();

        assert!(!pk_bytes.is_empty());
        assert!(!vk_bytes.is_empty());
    }

    #[test]
    fn test_create_proof() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100
                }
            },
            "required": ["amount"]
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 50);

        let proof = Proof::create(&compiled, &witness).unwrap();

        let proof_bytes = proof.serialize().unwrap();
        assert!(!proof_bytes.is_empty());
    }
}
