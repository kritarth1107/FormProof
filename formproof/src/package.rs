//! Portable proof packages for FormProof.
//!
//! A proof package bundles together everything needed to verify a proof:
//! - The serialized Groth16 proof
//! - The 32-byte commitment (public input)
//! - A SHA-256 fingerprint of the schema JSON
//!
//! This allows hosts to pass around a single artifact instead of
//! juggling separate files for proof, commitment, and schema reference.

use crate::prove::{CompiledSchema, Proof, ProveError};
use crate::schema::FormProofSchema;
use crate::verify::{verify_or_err, VerifyError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Current version of the proof package format.
pub const PACKAGE_VERSION: u8 = 1;

/// Errors that can occur with proof packages.
#[derive(Error, Debug)]
pub enum PackageError {
    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// The package version is not supported.
    #[error("unsupported package version: {0} (expected {PACKAGE_VERSION})")]
    UnsupportedVersion(u8),
    /// The schema fingerprint doesn't match.
    #[error("schema fingerprint mismatch: expected {expected}, got {actual}")]
    SchemaFingerprintMismatch {
        /// The expected fingerprint (from the compiled schema).
        expected: String,
        /// The actual fingerprint (from the package).
        actual: String,
    },
    /// Error during proof verification.
    #[error("verification error: {0}")]
    Verify(#[from] VerifyError),
    /// Error deserializing proof.
    #[error("proof deserialization error: {0}")]
    ProofDeserialize(#[from] ProveError),
    /// Invalid hex encoding.
    #[error("invalid hex encoding: {0}")]
    HexDecode(#[from] hex::FromHexError),
    /// Invalid commitment length.
    #[error("invalid commitment length: expected 32 bytes, got {0}")]
    InvalidCommitmentLength(usize),
}

/// A portable proof package containing proof, commitment, and schema fingerprint.
///
/// The package is serialized as JSON for easy transport and inspection.
/// Use [`ProofPackage::to_json`] to serialize and [`ProofPackage::from_json`]
/// to deserialize.
///
/// # Example
///
/// ```no_run
/// use formproof::{FormProofSchema, CompiledSchema, Witness, ProofPackage};
///
/// let schema_json = r#"{"type":"object","properties":{"x":{"type":"integer"}},"required":["x"]}"#;
/// let schema = FormProofSchema::from_json(schema_json).unwrap();
/// let compiled = CompiledSchema::compile(schema).unwrap();
///
/// let mut witness = formproof::Witness::new();
/// witness.set_u64("x", 42);
///
/// // Create package from proof
/// let package = ProofPackage::create(&compiled, &witness).unwrap();
///
/// // Serialize to JSON
/// let json = package.to_json().unwrap();
///
/// // Later: verify against the same schema
/// let loaded = ProofPackage::from_json(&json).unwrap();
/// assert!(loaded.verify(&compiled).is_ok());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofPackage {
    /// Package format version (for future compatibility).
    pub version: u8,
    /// Hex-encoded serialized Groth16 proof.
    pub proof_hex: String,
    /// Hex-encoded 32-byte commitment.
    pub commitment_hex: String,
    /// Hex-encoded SHA-256 fingerprint of the schema JSON (normalized).
    pub schema_fingerprint: String,
}

impl ProofPackage {
    /// Create a proof package from a compiled schema and witness.
    ///
    /// This generates a proof and bundles it with the commitment and
    /// schema fingerprint.
    ///
    /// # Errors
    ///
    /// Returns an error if proof generation or serialization fails.
    pub fn create(
        compiled: &CompiledSchema,
        witness: &crate::circuit::Witness,
    ) -> Result<Self, PackageError> {
        let proof = Proof::create(compiled, witness)?;
        Self::from_proof(&proof, &compiled.schema)
    }

    /// Create a proof package from an existing proof and schema.
    ///
    /// Use this if you already have a proof and want to bundle it.
    pub fn from_proof(proof: &Proof, schema: &FormProofSchema) -> Result<Self, PackageError> {
        let proof_bytes = proof.serialize()?;
        let proof_hex = hex::encode(&proof_bytes);
        let commitment_hex = hex::encode(proof.commitment);
        let schema_fingerprint = hex::encode(schema_fingerprint(schema));

        Ok(ProofPackage {
            version: PACKAGE_VERSION,
            proof_hex,
            commitment_hex,
            schema_fingerprint,
        })
    }

    /// Serialize the package to JSON.
    pub fn to_json(&self) -> Result<String, PackageError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Serialize the package to compact JSON (no whitespace).
    pub fn to_json_compact(&self) -> Result<String, PackageError> {
        Ok(serde_json::to_string(self)?)
    }

    /// Deserialize a package from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON parsing fails or version is unsupported.
    pub fn from_json(json: &str) -> Result<Self, PackageError> {
        let package: ProofPackage = serde_json::from_str(json)?;

        if package.version != PACKAGE_VERSION {
            return Err(PackageError::UnsupportedVersion(package.version));
        }

        Ok(package)
    }

    /// Extract the commitment bytes from the package.
    pub fn commitment(&self) -> Result<[u8; 32], PackageError> {
        let bytes = hex::decode(&self.commitment_hex)?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|v: Vec<u8>| PackageError::InvalidCommitmentLength(v.len()))?;
        Ok(arr)
    }

    /// Verify the package against a compiled schema.
    ///
    /// This checks:
    /// 1. The schema fingerprint matches
    /// 2. The proof verifies against the commitment
    ///
    /// # Errors
    ///
    /// Returns an error if fingerprint doesn't match or proof is invalid.
    pub fn verify(&self, compiled: &CompiledSchema) -> Result<(), PackageError> {
        let expected_fingerprint = hex::encode(schema_fingerprint(&compiled.schema));
        if self.schema_fingerprint != expected_fingerprint {
            return Err(PackageError::SchemaFingerprintMismatch {
                expected: expected_fingerprint,
                actual: self.schema_fingerprint.clone(),
            });
        }

        let commitment = self.commitment()?;
        let proof_bytes = hex::decode(&self.proof_hex)?;
        let proof = Proof::deserialize(&proof_bytes, commitment)?;

        verify_or_err(compiled, &proof)?;
        Ok(())
    }
}

/// Compute a SHA-256 fingerprint of a schema.
///
/// The fingerprint is computed from the normalized JSON representation
/// of the schema (via `to_json()`), ensuring consistent hashing regardless
/// of original formatting.
pub fn schema_fingerprint(schema: &FormProofSchema) -> [u8; 32] {
    let normalized = schema.to_json();
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::Witness;

    fn test_schema() -> FormProofSchema {
        FormProofSchema::from_json(
            r#"{
            "type": "object",
            "properties": {
                "value": { "type": "integer", "minimum": 0, "maximum": 100 }
            },
            "required": ["value"]
        }"#,
        )
        .unwrap()
    }

    #[test]
    fn test_schema_fingerprint_deterministic() {
        let schema = test_schema();
        let fp1 = schema_fingerprint(&schema);
        let fp2 = schema_fingerprint(&schema);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_schema_fingerprint_changes_with_schema() {
        let schema1 = test_schema();
        let schema2 = FormProofSchema::from_json(
            r#"{
            "type": "object",
            "properties": {
                "value": { "type": "integer", "minimum": 0, "maximum": 200 }
            },
            "required": ["value"]
        }"#,
        )
        .unwrap();

        let fp1 = schema_fingerprint(&schema1);
        let fp2 = schema_fingerprint(&schema2);
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_package_json_roundtrip() {
        let schema = test_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("value", 50);

        let package = ProofPackage::create(&compiled, &witness).unwrap();
        let json = package.to_json().unwrap();

        let loaded = ProofPackage::from_json(&json).unwrap();
        assert_eq!(loaded.version, package.version);
        assert_eq!(loaded.proof_hex, package.proof_hex);
        assert_eq!(loaded.commitment_hex, package.commitment_hex);
        assert_eq!(loaded.schema_fingerprint, package.schema_fingerprint);
    }

    #[test]
    fn test_package_verify_success() {
        let schema = test_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("value", 50);

        let package = ProofPackage::create(&compiled, &witness).unwrap();
        assert!(package.verify(&compiled).is_ok());
    }

    #[test]
    fn test_package_verify_wrong_schema() {
        let schema1 = test_schema();
        let compiled1 = CompiledSchema::compile(schema1).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("value", 50);
        let package = ProofPackage::create(&compiled1, &witness).unwrap();

        let schema2 = FormProofSchema::from_json(
            r#"{
            "type": "object",
            "properties": {
                "value": { "type": "integer", "minimum": 0, "maximum": 200 }
            },
            "required": ["value"]
        }"#,
        )
        .unwrap();
        let compiled2 = CompiledSchema::compile(schema2).unwrap();

        let result = package.verify(&compiled2);
        assert!(matches!(
            result,
            Err(PackageError::SchemaFingerprintMismatch { .. })
        ));
    }

    #[test]
    fn test_package_unsupported_version() {
        let json = r#"{
            "version": 99,
            "proof_hex": "00",
            "commitment_hex": "0000000000000000000000000000000000000000000000000000000000000000",
            "schema_fingerprint": "0000000000000000000000000000000000000000000000000000000000000000"
        }"#;

        let result = ProofPackage::from_json(json);
        assert!(matches!(result, Err(PackageError::UnsupportedVersion(99))));
    }
}
