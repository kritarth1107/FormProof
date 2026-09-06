#![warn(missing_docs)]

//! FormProof: Zero-knowledge proofs for JSON Schema validation
//!
//! This library allows you to prove that a JSON payload satisfies schema constraints
//! (required keys, enum values, integer ranges, string lengths) without revealing
//! the actual field values.
//!
//! # Supported Schema Subset (v0)
//!
//! - Object with ≤8 properties
//! - Types: `u64` (integer), `enum` (≤8 variants), `bytes32`, `string` (≤64 chars)
//! - Constraints: `required`, `minimum`, `maximum`, `maxLength`
//!
//! # Example
//!
//! ```no_run
//! use formproof::{FormProofSchema, CompiledSchema, Witness, Proof, verify};
//!
//! // Define schema: refund amount ≤50, valid currency
//! let schema_json = r#"{
//!     "type": "object",
//!     "properties": {
//!         "amount": { "type": "integer", "minimum": 0, "maximum": 50 },
//!         "currency": { "enum": ["USD", "EUR", "GBP"] }
//!     },
//!     "required": ["amount", "currency"]
//! }"#;
//!
//! // Parse and compile schema
//! let schema = FormProofSchema::from_json(schema_json).unwrap();
//! let compiled = CompiledSchema::compile(schema).unwrap();
//!
//! // Create witness (private data)
//! let mut witness = Witness::new();
//! witness.set_u64("amount", 25);  // actual refund amount (private)
//! witness.set_enum("currency", "USD");
//!
//! // Generate proof
//! let proof = Proof::create(&compiled, &witness).unwrap();
//!
//! // Verify proof (verifier only sees commitment, not values)
//! assert!(verify(&compiled, &proof).unwrap());
//! ```

pub mod circuit;
pub mod package;
pub mod prove;
pub mod schema;
pub mod verify;

pub use circuit::{Witness, WitnessValue};
pub use package::{schema_fingerprint, PackageError, ProofPackage, PACKAGE_VERSION};
pub use prove::{CompiledSchema, Proof, ProveError};
pub use schema::{FormProofSchema, Property, PropertyType, SchemaError};
pub use verify::{verify, verify_or_err, VerifyError};
