//! Property-based tests for FormProof schema parser and circuit.
//!
//! Uses proptest to generate random valid and invalid schemas, witnesses,
//! and verify that the parser and circuit behave correctly.

use formproof::{
    circuit::FormProofCircuit, verify, CompiledSchema, FormProofSchema, Proof, Property,
    PropertyType, Witness,
};
use proptest::prelude::*;
use proptest::strategy::ValueTree;

fn deterministic_config() -> ProptestConfig {
    ProptestConfig {
        source_file: Some(file!()),
        failure_persistence: None,
        ..ProptestConfig::with_cases(30)
    }
}

fn arb_property_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_filter("valid identifier", |s| !s.is_empty())
}

fn arb_enum_variants(max_variants: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[A-Z][A-Z0-9_]{0,7}", 1..=max_variants).prop_filter(
        "unique variants",
        |v| {
            let unique: std::collections::HashSet<_> = v.iter().collect();
            unique.len() == v.len()
        },
    )
}

fn arb_u64_property() -> impl Strategy<Value = PropertyType> {
    prop_oneof![
        Just(PropertyType::U64 {
            minimum: None,
            maximum: None
        }),
        (0u64..100).prop_map(|min| PropertyType::U64 {
            minimum: Some(min),
            maximum: None
        }),
        (0u64..100).prop_map(|max| PropertyType::U64 {
            minimum: None,
            maximum: Some(max)
        }),
        (0u64..50, 50u64..100).prop_map(|(min, max)| PropertyType::U64 {
            minimum: Some(min),
            maximum: Some(max)
        }),
    ]
}

fn arb_enum_property() -> impl Strategy<Value = PropertyType> {
    arb_enum_variants(8).prop_map(|variants| PropertyType::Enum { variants })
}

fn arb_string_property() -> impl Strategy<Value = PropertyType> {
    (1usize..=64).prop_map(|max_length| PropertyType::String { max_length })
}

fn arb_bytes32_property() -> impl Strategy<Value = PropertyType> {
    Just(PropertyType::Bytes32)
}

fn arb_property_type() -> impl Strategy<Value = PropertyType> {
    prop_oneof![
        arb_u64_property(),
        arb_enum_property(),
        arb_string_property(),
        arb_bytes32_property(),
    ]
}

fn arb_property(name: String) -> impl Strategy<Value = Property> {
    (arb_property_type(), any::<bool>()).prop_map(move |(prop_type, required)| Property {
        name: name.clone(),
        prop_type,
        required,
    })
}

fn arb_valid_schema(num_props: usize) -> impl Strategy<Value = FormProofSchema> {
    prop::collection::vec(arb_property_name(), num_props)
        .prop_filter("unique names", |names| {
            let unique: std::collections::HashSet<_> = names.iter().collect();
            unique.len() == names.len()
        })
        .prop_flat_map(|names| {
            let props: Vec<_> = names.into_iter().map(arb_property).collect();
            props
        })
        .prop_map(|mut properties| {
            properties.sort_by(|a, b| a.name.cmp(&b.name));
            FormProofSchema { properties }
        })
}

fn schema_to_json(schema: &FormProofSchema) -> String {
    schema.to_json()
}

proptest! {
    #![proptest_config(deterministic_config())]

    #[test]
    fn valid_schema_parses_successfully(num_props in 1usize..=8) {
        let runner = proptest::test_runner::TestRunner::deterministic();
        let strategy = arb_valid_schema(num_props);
        let mut runner = runner;

        for _ in 0..10 {
            if let Ok(schema) = strategy.new_tree(&mut runner).map(|t| t.current()) {
                let json = schema_to_json(&schema);
                let parsed = FormProofSchema::from_json(&json);
                prop_assert!(parsed.is_ok(), "Valid schema should parse: {}", json);

                let parsed = parsed.unwrap();
                prop_assert_eq!(
                    parsed.properties.len(),
                    schema.properties.len(),
                    "Property count should match"
                );
            }
        }
    }

    #[test]
    fn schema_roundtrip_preserves_structure(num_props in 1usize..=8) {
        let runner = proptest::test_runner::TestRunner::deterministic();
        let strategy = arb_valid_schema(num_props);
        let mut runner = runner;

        for _ in 0..10 {
            if let Ok(schema) = strategy.new_tree(&mut runner).map(|t| t.current()) {
                let json = schema_to_json(&schema);
                let parsed = FormProofSchema::from_json(&json).unwrap();
                let json2 = schema_to_json(&parsed);
                let parsed2 = FormProofSchema::from_json(&json2).unwrap();
                prop_assert_eq!(parsed, parsed2, "Roundtrip should preserve schema");
            }
        }
    }
}

mod invalid_schemas {
    use super::*;
    use formproof::SchemaError;

    #[test]
    fn rejects_too_many_properties() {
        let mut props = String::new();
        for i in 0..10 {
            if i > 0 {
                props.push_str(", ");
            }
            props.push_str(&format!(r#""prop{i}": {{"type": "integer"}}"#));
        }

        let schema_json = format!(
            r#"{{
            "type": "object",
            "properties": {{ {props} }},
            "required": []
        }}"#
        );

        let result = FormProofSchema::from_json(&schema_json);
        assert!(matches!(result, Err(SchemaError::TooManyProperties(_))));
    }

    #[test]
    fn rejects_too_many_enum_variants() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "status": {
                    "enum": ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]
                }
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(
            result,
            Err(SchemaError::TooManyEnumVariants(_, _))
        ));
    }

    #[test]
    fn rejects_string_too_long() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "maxLength": 100
                }
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::StringTooLong(_, _))));
    }

    #[test]
    fn rejects_min_greater_than_max() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 100,
                    "maximum": 50
                }
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::MinGreaterThanMax(_))));
    }

    #[test]
    fn rejects_non_object_type() {
        let schema_json = r#"{ "type": "array" }"#;
        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::NotAnObject)));
    }

    #[test]
    fn rejects_missing_type() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {}
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::MissingType(_))));
    }

    #[test]
    fn rejects_unsupported_type() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "data": {
                    "type": "array"
                }
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::UnsupportedType(_, _))));
    }

    #[test]
    fn rejects_required_not_defined() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": { "type": "integer" }
            },
            "required": ["nonexistent"]
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::RequiredNotDefined(_))));
    }

    #[test]
    fn rejects_invalid_json() {
        let schema_json = r#"{ not valid json }"#;
        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::InvalidJson(_))));
    }

    proptest! {
        #![proptest_config(deterministic_config())]

        #[test]
        fn rejects_invalid_min_max_range(min in 51u64..=100, max in 0u64..=50) {
            let schema_json = format!(
                r#"{{
                "type": "object",
                "properties": {{
                    "value": {{
                        "type": "integer",
                        "minimum": {min},
                        "maximum": {max}
                    }}
                }},
                "required": []
            }}"#
            );

            let result = FormProofSchema::from_json(&schema_json);
            prop_assert!(matches!(result, Err(SchemaError::MinGreaterThanMax(_))));
        }

        #[test]
        fn rejects_excessive_max_length(max_len in 65usize..=200) {
            let schema_json = format!(
                r#"{{
                "type": "object",
                "properties": {{
                    "text": {{
                        "type": "string",
                        "maxLength": {max_len}
                    }}
                }},
                "required": []
            }}"#
            );

            let result = FormProofSchema::from_json(&schema_json);
            prop_assert!(matches!(result, Err(SchemaError::StringTooLong(_, _))));
        }
    }
}

mod witness_circuit {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};

    fn simple_refund_schema() -> FormProofSchema {
        FormProofSchema::from_json(
            r#"{
            "type": "object",
            "properties": {
                "amount": { "type": "integer", "minimum": 0, "maximum": 50 },
                "currency": { "enum": ["USD", "EUR", "GBP"] }
            },
            "required": ["amount", "currency"]
        }"#,
        )
        .unwrap()
    }

    fn user_schema() -> FormProofSchema {
        FormProofSchema::from_json(
            r#"{
            "type": "object",
            "properties": {
                "age": { "type": "integer", "minimum": 18, "maximum": 120 },
                "country": { "enum": ["US", "UK", "DE", "FR"] },
                "name": { "type": "string", "maxLength": 32 }
            },
            "required": ["age", "country"]
        }"#,
        )
        .unwrap()
    }

    fn token_schema() -> FormProofSchema {
        FormProofSchema::from_json(
            r#"{
            "type": "object",
            "properties": {
                "token_id": { "type": "string", "format": "bytes32" },
                "balance": { "type": "integer", "minimum": 0, "maximum": 1000000 }
            },
            "required": ["token_id", "balance"]
        }"#,
        )
        .unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn valid_refund_witnesses_satisfy_circuit(amount in 0u64..=50, currency_idx in 0usize..3) {
            let currencies = ["USD", "EUR", "GBP"];
            let schema = simple_refund_schema();

            let mut witness = Witness::new();
            witness.set_u64("amount", amount);
            witness.set_enum("currency", currencies[currency_idx]);

            let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);
            let cs = ConstraintSystem::<Fr>::new_ref();
            circuit.generate_constraints(cs.clone()).unwrap();

            prop_assert!(cs.is_satisfied().unwrap(), "Valid witness should satisfy circuit");
        }

        #[test]
        fn invalid_amount_fails_circuit(amount in 51u64..=1000) {
            let schema = simple_refund_schema();

            let mut witness = Witness::new();
            witness.set_u64("amount", amount);
            witness.set_enum("currency", "USD");

            let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);
            let cs = ConstraintSystem::<Fr>::new_ref();
            circuit.generate_constraints(cs.clone()).unwrap();

            prop_assert!(!cs.is_satisfied().unwrap(), "Amount {} should fail (max 50)", amount);
        }

        #[test]
        fn valid_age_witnesses_satisfy_circuit(age in 18u64..=120, country_idx in 0usize..4) {
            let countries = ["US", "UK", "DE", "FR"];
            let schema = user_schema();

            let mut witness = Witness::new();
            witness.set_u64("age", age);
            witness.set_enum("country", countries[country_idx]);

            let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);
            let cs = ConstraintSystem::<Fr>::new_ref();
            circuit.generate_constraints(cs.clone()).unwrap();

            prop_assert!(cs.is_satisfied().unwrap(), "Valid witness should satisfy circuit");
        }

        #[test]
        fn underage_witnesses_fail_circuit(age in 0u64..18) {
            let schema = user_schema();

            let mut witness = Witness::new();
            witness.set_u64("age", age);
            witness.set_enum("country", "US");

            let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);
            let cs = ConstraintSystem::<Fr>::new_ref();
            circuit.generate_constraints(cs.clone()).unwrap();

            prop_assert!(!cs.is_satisfied().unwrap(), "Age {} should fail (min 18)", age);
        }

        #[test]
        fn valid_token_witnesses_satisfy_circuit(balance in 0u64..=1_000_000) {
            let schema = token_schema();

            let mut witness = Witness::new();
            witness.set_bytes32("token_id", [0xAB; 32]);
            witness.set_u64("balance", balance);

            let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);
            let cs = ConstraintSystem::<Fr>::new_ref();
            circuit.generate_constraints(cs.clone()).unwrap();

            prop_assert!(cs.is_satisfied().unwrap(), "Valid witness should satisfy circuit");
        }
    }

    #[test]
    fn full_prove_verify_roundtrip_refund() {
        let schema = simple_refund_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let test_cases = [(0, "USD"), (25, "EUR"), (50, "GBP")];

        for (amount, currency) in test_cases {
            let mut witness = Witness::new();
            witness.set_u64("amount", amount);
            witness.set_enum("currency", currency);

            let proof = Proof::create(&compiled, &witness).unwrap();
            assert!(
                verify(&compiled, &proof).unwrap(),
                "Proof for amount={}, currency={} should verify",
                amount,
                currency
            );
        }
    }

    #[test]
    fn full_prove_verify_roundtrip_user() {
        let schema = user_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_u64("age", 25);
        witness.set_enum("country", "US");
        witness.set_string("name", "Alice");

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn full_prove_verify_roundtrip_token() {
        let schema = token_schema();
        let compiled = CompiledSchema::compile(schema).unwrap();

        let mut witness = Witness::new();
        witness.set_bytes32("token_id", [0xDE; 32]);
        witness.set_u64("balance", 500_000);

        let proof = Proof::create(&compiled, &witness).unwrap();
        assert!(verify(&compiled, &proof).unwrap());
    }

    #[test]
    fn different_witnesses_produce_different_commitments() {
        let schema = simple_refund_schema();

        let mut w1 = Witness::new();
        w1.set_u64("amount", 10);
        w1.set_enum("currency", "USD");

        let mut w2 = Witness::new();
        w2.set_u64("amount", 20);
        w2.set_enum("currency", "USD");

        let mut w3 = Witness::new();
        w3.set_u64("amount", 10);
        w3.set_enum("currency", "EUR");

        let c1 = w1.commitment(&schema);
        let c2 = w2.commitment(&schema);
        let c3 = w3.commitment(&schema);

        assert_ne!(
            c1, c2,
            "Different amounts should produce different commitments"
        );
        assert_ne!(
            c1, c3,
            "Different currencies should produce different commitments"
        );
        assert_ne!(c2, c3, "All three should be different");
    }

    #[test]
    fn missing_required_field_fails() {
        let schema = simple_refund_schema();

        let mut witness = Witness::new();
        witness.set_u64("amount", 25);

        let circuit = FormProofCircuit::<Fr>::new(schema).with_witness(witness);
        let cs = ConstraintSystem::<Fr>::new_ref();
        circuit.generate_constraints(cs.clone()).unwrap();

        assert!(
            !cs.is_satisfied().unwrap(),
            "Missing required currency should fail"
        );
    }
}
