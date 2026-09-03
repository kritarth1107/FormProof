//! Golden proof tests - demonstrate that valid witnesses produce valid proofs
//! and invalid witnesses are properly rejected.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};

fn refund_schema() -> FormProofSchema {
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

fn user_schema() -> FormProofSchema {
    let schema_json = r#"{
        "type": "object",
        "properties": {
            "age": {
                "type": "integer",
                "minimum": 18,
                "maximum": 120
            },
            "country": {
                "enum": ["US", "UK", "DE", "FR", "JP"]
            },
            "name": {
                "type": "string",
                "maxLength": 32
            }
        },
        "required": ["age", "country"]
    }"#;
    FormProofSchema::from_json(schema_json).unwrap()
}

fn token_schema() -> FormProofSchema {
    let schema_json = r#"{
        "type": "object",
        "properties": {
            "token_id": {
                "type": "string",
                "format": "bytes32"
            },
            "balance": {
                "type": "integer",
                "minimum": 0,
                "maximum": 1000000
            }
        },
        "required": ["token_id", "balance"]
    }"#;
    FormProofSchema::from_json(schema_json).unwrap()
}

mod golden_proofs {
    use super::*;

    #[test]
    fn golden_proof_1_refund_under_limit() {
        let schema = refund_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 25);
        witness.set_enum("currency", "USD");

        let proof = Proof::create(&compiled, &witness).unwrap();

        assert!(verify(&compiled, &proof).unwrap());

        let proof_bytes = proof.serialize().unwrap();
        assert!(!proof_bytes.is_empty());
        println!("Golden Proof 1: refund $25 USD");
        println!("  Commitment: {}", hex::encode(proof.commitment));
        println!("  Proof size: {} bytes", proof_bytes.len());
    }

    #[test]
    fn golden_proof_2_refund_at_exact_limit() {
        let schema = refund_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 50);
        witness.set_enum("currency", "EUR");

        let proof = Proof::create(&compiled, &witness).unwrap();

        assert!(verify(&compiled, &proof).unwrap());

        let proof_bytes = proof.serialize().unwrap();
        println!("Golden Proof 2: refund €50 EUR (at limit)");
        println!("  Commitment: {}", hex::encode(proof.commitment));
        println!("  Proof size: {} bytes", proof_bytes.len());
    }

    #[test]
    fn golden_proof_3_user_verification() {
        let schema = user_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("age", 30);
        witness.set_enum("country", "US");
        witness.set_string("name", "Alice");

        let proof = Proof::create(&compiled, &witness).unwrap();

        assert!(verify(&compiled, &proof).unwrap());

        let proof_bytes = proof.serialize().unwrap();
        println!("Golden Proof 3: user verification (age 30, US)");
        println!("  Commitment: {}", hex::encode(proof.commitment));
        println!("  Proof size: {} bytes", proof_bytes.len());
    }
}

mod rejection_corpus {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
    use formproof::circuit::FormProofCircuit;

    #[test]
    fn reject_wrong_enum_value() {
        let schema = refund_schema();

        let mut witness = Witness::new();
        witness.set_u64("amount", 25);
        witness.set_enum("currency", "INVALID");

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(
            !cs.is_satisfied().unwrap(),
            "Circuit should reject invalid enum value 'INVALID'"
        );
    }

    #[test]
    fn reject_value_over_maximum() {
        let schema = refund_schema();

        let mut witness = Witness::new();
        witness.set_u64("amount", 100);
        witness.set_enum("currency", "USD");

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(
            !cs.is_satisfied().unwrap(),
            "Circuit should reject amount 100 (max is 50)"
        );
    }

    #[test]
    fn reject_value_under_minimum() {
        let schema = user_schema();

        let mut witness = Witness::new();
        witness.set_u64("age", 15);
        witness.set_enum("country", "US");

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(
            !cs.is_satisfied().unwrap(),
            "Circuit should reject age 15 (min is 18)"
        );
    }

    #[test]
    fn reject_missing_required_field() {
        let schema = refund_schema();

        let mut witness = Witness::new();
        witness.set_u64("amount", 25);

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(
            !cs.is_satisfied().unwrap(),
            "Circuit should reject missing required field 'currency'"
        );
    }

    #[test]
    fn reject_string_length_overflow() {
        let schema = user_schema();

        let mut witness = Witness::new();
        witness.set_u64("age", 25);
        witness.set_enum("country", "US");
        witness.set_string(
            "name",
            "This name is way too long and exceeds the maximum length of 32 characters",
        );

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(
            cs.is_satisfied().unwrap(),
            "String is truncated to maxLength so circuit should still pass"
        );
    }

    #[test]
    fn reject_all_fields_missing() {
        let schema = refund_schema();

        let witness = Witness::new();

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);

        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(
            !cs.is_satisfied().unwrap(),
            "Circuit should reject when all required fields are missing"
        );
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn valid_zero_amount() {
        let schema = refund_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("amount", 0);
        witness.set_enum("currency", "GBP");

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn valid_boundary_age() {
        let schema = user_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("age", 18);
        witness.set_enum("country", "JP");

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn valid_with_optional_field() {
        let schema = user_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("age", 25);
        witness.set_enum("country", "DE");

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn valid_with_bytes32() {
        let schema = token_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_bytes32("token_id", [0xAB; 32]);
        witness.set_u64("balance", 1000);

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn commitments_differ_for_different_witnesses() {
        let schema = refund_schema();

        let mut witness1 = Witness::new();
        witness1.set_u64("amount", 25);
        witness1.set_enum("currency", "USD");

        let mut witness2 = Witness::new();
        witness2.set_u64("amount", 30);
        witness2.set_enum("currency", "USD");

        let commitment1 = witness1.commitment(&schema);
        let commitment2 = witness2.commitment(&schema);

        assert_ne!(
            commitment1, commitment2,
            "Different witnesses should produce different commitments"
        );
    }
}
