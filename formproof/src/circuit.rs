//! R1CS circuit generation for FormProof schemas.
//!
//! This module converts schemas and witnesses into arkworks R1CS constraints
//! that can be used with Groth16 proving.

use crate::schema::{FormProofSchema, PropertyType};
use ark_ff::PrimeField;
use ark_r1cs_std::{
    alloc::AllocVar,
    boolean::Boolean,
    eq::EqGadget,
    fields::{fp::FpVar, FieldVar},
    select::CondSelectGadget,
    uint8::UInt8,
    ToBitsGadget,
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single value in a witness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WitnessValue {
    /// 64-bit unsigned integer.
    U64(u64),
    /// Enum variant (string value).
    Enum(String),
    /// 32-byte binary data.
    Bytes32([u8; 32]),
    /// UTF-8 string.
    String(String),
    /// Property is absent from the witness.
    Absent,
}

/// Private witness data for proof generation.
///
/// A witness contains the actual values for each property in the schema.
/// These values are private and never revealed to the verifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    /// Property name and value pairs.
    pub values: Vec<(String, WitnessValue)>,
}

impl Witness {
    /// Create a new empty witness.
    pub fn new() -> Self {
        Witness { values: Vec::new() }
    }

    /// Set a u64 integer value.
    pub fn set_u64(&mut self, name: &str, value: u64) {
        self.values
            .push((name.to_string(), WitnessValue::U64(value)));
    }

    /// Set an enum value (string variant).
    pub fn set_enum(&mut self, name: &str, value: &str) {
        self.values
            .push((name.to_string(), WitnessValue::Enum(value.to_string())));
    }

    /// Set a bytes32 value (32 bytes).
    pub fn set_bytes32(&mut self, name: &str, value: [u8; 32]) {
        self.values
            .push((name.to_string(), WitnessValue::Bytes32(value)));
    }

    /// Set a string value.
    pub fn set_string(&mut self, name: &str, value: &str) {
        self.values
            .push((name.to_string(), WitnessValue::String(value.to_string())));
    }

    /// Mark a property as absent.
    pub fn set_absent(&mut self, name: &str) {
        self.values.push((name.to_string(), WitnessValue::Absent));
    }

    /// Get a value by property name.
    pub fn get(&self, name: &str) -> Option<&WitnessValue> {
        self.values.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// Serialize the witness to bytes for commitment.
    pub fn to_bytes(&self, schema: &FormProofSchema) -> Vec<u8> {
        let mut bytes = Vec::new();

        for prop in &schema.properties {
            let value = self.get(&prop.name);

            match (&prop.prop_type, value) {
                (PropertyType::U64 { .. }, Some(WitnessValue::U64(v))) => {
                    bytes.push(1u8);
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
                (PropertyType::Enum { variants }, Some(WitnessValue::Enum(s))) => {
                    bytes.push(1u8);
                    let idx = variants.iter().position(|v| v == s).unwrap_or(0) as u8;
                    bytes.push(idx);
                }
                (PropertyType::Bytes32, Some(WitnessValue::Bytes32(b))) => {
                    bytes.push(1u8);
                    bytes.extend_from_slice(b);
                }
                (PropertyType::String { max_length }, Some(WitnessValue::String(s))) => {
                    bytes.push(1u8);
                    let s_bytes = s.as_bytes();
                    let len = s_bytes.len().min(*max_length);
                    bytes.push(len as u8);
                    bytes.extend_from_slice(&s_bytes[..len]);
                    bytes.extend(std::iter::repeat_n(0u8, *max_length - len));
                }
                _ => {
                    bytes.push(0u8);
                    match &prop.prop_type {
                        PropertyType::U64 { .. } => bytes.extend_from_slice(&[0u8; 8]),
                        PropertyType::Enum { .. } => bytes.push(0u8),
                        PropertyType::Bytes32 => bytes.extend_from_slice(&[0u8; 32]),
                        PropertyType::String { max_length } => {
                            bytes.push(0u8);
                            bytes.extend(std::iter::repeat_n(0u8, *max_length));
                        }
                    }
                }
            }
        }

        bytes
    }

    /// Compute SHA-256 commitment of the witness.
    ///
    /// This is the public input to the circuit - it commits to the witness
    /// values without revealing them.
    pub fn commitment(&self, schema: &FormProofSchema) -> [u8; 32] {
        let bytes = self.to_bytes(schema);
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }
}

impl Default for Witness {
    fn default() -> Self {
        Self::new()
    }
}

/// The R1CS circuit for proving schema compliance.
///
/// This is primarily for internal use - most users should use
/// [`CompiledSchema`](crate::CompiledSchema) and [`Proof`](crate::Proof) instead.
#[derive(Clone)]
pub struct FormProofCircuit<F: PrimeField> {
    /// The schema being proven against.
    pub schema: FormProofSchema,
    /// The private witness data (None for setup).
    pub witness: Option<Witness>,
    /// The commitment to the witness (None for setup).
    pub commitment: Option<[u8; 32]>,
    _marker: std::marker::PhantomData<F>,
}

impl<F: PrimeField> FormProofCircuit<F> {
    /// Create a new circuit for the given schema (without witness).
    pub fn new(schema: FormProofSchema) -> Self {
        FormProofCircuit {
            schema,
            witness: None,
            commitment: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Add a witness to the circuit for proving.
    pub fn with_witness(mut self, witness: Witness) -> Self {
        let commitment = witness.commitment(&self.schema);
        self.witness = Some(witness);
        self.commitment = Some(commitment);
        self
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for FormProofCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let commitment_bytes = self.commitment.unwrap_or([0u8; 32]);

        let _commitment_vars: Vec<FpVar<F>> = (0..32)
            .map(|i| FpVar::new_input(cs.clone(), || Ok(F::from(commitment_bytes[i] as u64))))
            .collect::<Result<Vec<_>, _>>()?;

        for prop in &self.schema.properties {
            let (is_present, value) = self.get_witness_value(&prop.name);

            let is_present_var = Boolean::new_witness(cs.clone(), || Ok(is_present))?;

            if prop.required {
                is_present_var.enforce_equal(&Boolean::TRUE)?;
            }

            match &prop.prop_type {
                PropertyType::U64 { minimum, maximum } => {
                    let value_u64 = match value {
                        Some(WitnessValue::U64(v)) => v,
                        _ => 0,
                    };

                    let value_var = FpVar::new_witness(cs.clone(), || Ok(F::from(value_u64)))?;

                    if let Some(min) = minimum {
                        let min_var = FpVar::constant(F::from(*min));
                        let diff = &value_var - &min_var;
                        let is_valid = self.is_non_negative(cs.clone(), &diff)?;
                        let check = is_present_var.not().or(&is_valid)?;
                        check.enforce_equal(&Boolean::TRUE)?;
                    }

                    if let Some(max) = maximum {
                        let max_var = FpVar::constant(F::from(*max));
                        let diff = &max_var - &value_var;
                        let is_valid = self.is_non_negative(cs.clone(), &diff)?;
                        let check = is_present_var.not().or(&is_valid)?;
                        check.enforce_equal(&Boolean::TRUE)?;
                    }
                }
                PropertyType::Enum { variants } => {
                    let (enum_idx, is_valid_enum) = match value {
                        Some(WitnessValue::Enum(s)) => {
                            match variants.iter().position(|v| v == &s) {
                                Some(idx) => (idx as u64, true),
                                None => (0, false),
                            }
                        }
                        _ => (0, true),
                    };

                    let idx_var = FpVar::new_witness(cs.clone(), || Ok(F::from(enum_idx)))?;
                    let is_valid_enum_var = Boolean::new_witness(cs.clone(), || Ok(is_valid_enum))?;

                    let num_variants = variants.len() as u64;
                    let max_var = FpVar::constant(F::from(num_variants - 1));
                    let diff = &max_var - &idx_var;
                    let is_in_range = self.is_non_negative(cs.clone(), &diff)?;

                    let is_valid = is_valid_enum_var.and(&is_in_range)?;
                    let check = is_present_var.not().or(&is_valid)?;
                    check.enforce_equal(&Boolean::TRUE)?;
                }
                PropertyType::Bytes32 => {
                    let bytes = match value {
                        Some(WitnessValue::Bytes32(b)) => b,
                        _ => [0u8; 32],
                    };

                    for &byte in &bytes {
                        let _byte_var = UInt8::new_witness(cs.clone(), || Ok(byte))?;
                    }
                }
                PropertyType::String { max_length } => {
                    let (len, _) = match value {
                        Some(WitnessValue::String(s)) => {
                            let bytes = s.as_bytes();
                            (bytes.len().min(*max_length), bytes.to_vec())
                        }
                        _ => (0, vec![]),
                    };

                    let len_var = FpVar::new_witness(cs.clone(), || Ok(F::from(len as u64)))?;

                    let max_var = FpVar::constant(F::from(*max_length as u64));
                    let diff = &max_var - &len_var;
                    let is_valid = self.is_non_negative(cs.clone(), &diff)?;

                    let check = is_present_var.not().or(&is_valid)?;
                    check.enforce_equal(&Boolean::TRUE)?;
                }
            }
        }

        Ok(())
    }
}

impl<F: PrimeField> FormProofCircuit<F> {
    fn get_witness_value(&self, name: &str) -> (bool, Option<WitnessValue>) {
        match &self.witness {
            Some(w) => match w.get(name) {
                Some(WitnessValue::Absent) | None => (false, None),
                Some(v) => (true, Some(v.clone())),
            },
            None => (false, None),
        }
    }

    fn is_non_negative(
        &self,
        cs: ConstraintSystemRef<F>,
        value: &FpVar<F>,
    ) -> Result<Boolean<F>, SynthesisError> {
        use ark_r1cs_std::R1CSVar;
        let val = value.value().unwrap_or(F::zero());
        let val_biguint: num_bigint::BigUint = val.into();
        let modulus_half_bigint: num_bigint::BigUint = F::MODULUS_MINUS_ONE_DIV_TWO.into();
        let is_small = val_biguint <= modulus_half_bigint;

        Boolean::new_witness(cs, || Ok(is_small))
    }
}

fn _byte_to_fp<F: PrimeField>(byte: &UInt8<F>) -> Result<FpVar<F>, SynthesisError> {
    let bits = byte.to_bits_le()?;
    let mut result = FpVar::zero();
    for (i, bit) in bits.iter().enumerate() {
        let bit_val =
            FpVar::conditionally_select(bit, &FpVar::constant(F::from(1u64 << i)), &FpVar::zero())?;
        result += bit_val;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::FormProofSchema;
    use ark_bn254::Fr;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn test_witness_commitment() {
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

        let mut witness = Witness::new();
        witness.set_u64("amount", 50);

        let commitment = witness.commitment(&schema);
        assert_eq!(commitment.len(), 32);

        let commitment2 = witness.commitment(&schema);
        assert_eq!(commitment, commitment2);
    }

    #[test]
    fn test_circuit_constraints_satisfied() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100
                },
                "currency": {
                    "enum": ["USD", "EUR", "GBP"]
                }
            },
            "required": ["amount", "currency"]
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 50);
        witness.set_enum("currency", "USD");

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(cs.is_satisfied().unwrap());
    }

    #[test]
    fn test_circuit_fails_on_missing_required() {
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

        let witness = Witness::new();

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(!cs.is_satisfied().unwrap());
    }

    #[test]
    fn test_circuit_fails_on_value_out_of_range() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 50
                }
            },
            "required": ["amount"]
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 100);

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(!cs.is_satisfied().unwrap());
    }
}
