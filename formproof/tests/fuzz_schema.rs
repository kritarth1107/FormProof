//! Fuzz-like integration tests for schema JSON parsing.
//!
//! These tests feed arbitrary and malformed JSON inputs to the schema parser
//! and verify that it never panics - only returns structured errors.
//!
//! For actual continuous fuzzing, see docs/FUZZING.md for cargo-fuzz setup.

use formproof::{FormProofSchema, SchemaError};
use proptest::prelude::*;

fn fuzz_config() -> ProptestConfig {
    ProptestConfig {
        source_file: Some(file!()),
        failure_persistence: None,
        ..ProptestConfig::with_cases(200)
    }
}

fn arb_json_string() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("null".to_string()),
        Just("true".to_string()),
        Just("false".to_string()),
        Just("123".to_string()),
        Just("[]".to_string()),
        Just("{}".to_string()),
        Just(r#"{"type":"object"}"#.to_string()),
        Just(r#"{"type":"array"}"#.to_string()),
        Just(r#"{"type":"string"}"#.to_string()),
        Just(r#"{"type":"object","properties":{}}"#.to_string()),
        "[a-zA-Z0-9{}\",:\\[\\] ]{0,100}".prop_map(|s| s),
        prop::collection::vec(any::<u8>(), 0..200)
            .prop_map(|bytes| { String::from_utf8_lossy(&bytes).into_owned() }),
    ]
}

fn arb_malformed_schema() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r#"{"type":"object","properties":{"a":{"type":"integer","minimum":-1}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"type":"number"}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"type":"boolean"}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"type":"null"}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"type":"array"}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"type":"object"}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"enum":[]}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"enum":[1,2,3]}}}"#.to_string()),
        Just(r#"{"type":"object","properties":{"a":{"enum":[null]}}}"#.to_string()),
        Just(
            r#"{"type":"object","properties":{"a":{"type":"string","maxLength":-5}}}"#.to_string()
        ),
        Just(
            r#"{"type":"object","properties":{"a":{"type":"string","maxLength":999}}}"#.to_string()
        ),
        Just(
            r#"{"type":"object","properties":{"a":{"type":"integer","minimum":100,"maximum":50}}}"#
                .to_string()
        ),
        Just(r#"{"type":"object","required":["missing"]}"#.to_string()),
        Just(r#"{"type":"object","properties":null}"#.to_string()),
        Just(r#"{"type":"object","properties":[]}"#.to_string()),
        Just(r#"{"properties":{"a":{"type":"integer"}}}"#.to_string()),
        (1usize..20).prop_map(|n| {
            let props: String = (0..n)
                .map(|i| format!(r#""p{}": {{"type": "integer"}}"#, i))
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"type":"object","properties":{{{}}}}}"#, props)
        }),
        (1usize..15).prop_map(|n| {
            let variants: String = (0..n)
                .map(|i| format!(r#""V{}""#, i))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"type":"object","properties":{{"e":{{"enum":[{}]}}}}}}"#,
                variants
            )
        }),
    ]
}

fn arb_nested_json() -> impl Strategy<Value = String> {
    (0usize..10).prop_map(|depth| {
        let mut s = String::new();
        for _ in 0..depth {
            s.push_str(r#"{"nested":"#);
        }
        s.push_str("null");
        for _ in 0..depth {
            s.push('}');
        }
        s
    })
}

proptest! {
    #![proptest_config(fuzz_config())]

    #[test]
    fn schema_parser_never_panics_on_arbitrary_input(input in arb_json_string()) {
        let _ = FormProofSchema::from_json(&input);
    }

    #[test]
    fn schema_parser_never_panics_on_malformed_schemas(input in arb_malformed_schema()) {
        let _ = FormProofSchema::from_json(&input);
    }

    #[test]
    fn schema_parser_never_panics_on_nested_json(input in arb_nested_json()) {
        let _ = FormProofSchema::from_json(&input);
    }

    #[test]
    fn schema_parser_returns_errors_not_panics_on_random_bytes(bytes in prop::collection::vec(any::<u8>(), 0..500)) {
        let input = String::from_utf8_lossy(&bytes);
        let result = FormProofSchema::from_json(&input);
        prop_assert!(result.is_err() || result.is_ok());
    }
}

#[test]
fn fuzz_corpus_hand_selected_edge_cases() {
    let edge_cases = [
        "",
        "{}",
        "[]",
        "null",
        "true",
        "false",
        "0",
        "-1",
        "1e308",
        r#""""#,
        r#""\u0000""#,
        r#"{"#,
        r#"}"#,
        r#"[[[[[[[[[["#,
        r#"{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":{"i":{"j":{}}}}}}}}}}}"#,
        r#"{"type":null}"#,
        r#"{"type":123}"#,
        r#"{"type":[]}"#,
        r#"{"type":{}}"#,
        r#"{"type":"object"}"#,
        r#"{"type":"object","properties":null}"#,
        r#"{"type":"object","properties":123}"#,
        r#"{"type":"object","properties":[]}"#,
        r#"{"type":"object","properties":{}}"#,
        r#"{"type":"object","properties":{},"required":null}"#,
        r#"{"type":"object","properties":{},"required":123}"#,
        r#"{"type":"object","properties":{},"required":{}}"#,
        r#"{"type":"object","properties":{"x":null}}"#,
        r#"{"type":"object","properties":{"x":123}}"#,
        r#"{"type":"object","properties":{"x":[]}}"#,
        r#"{"type":"object","properties":{"x":"string"}}"#,
        r#"{"type":"object","properties":{"":"type":"integer"}}"#,
        r#"{"type":"object","properties":{"a":{"type":"integer","minimum":"not a number"}}}"#,
        r#"{"type":"object","properties":{"a":{"type":"integer","maximum":"not a number"}}}"#,
        r#"{"type":"object","properties":{"a":{"type":"string","maxLength":"not a number"}}}"#,
        r#"{"type":"object","properties":{"a":{"enum":"not an array"}}}"#,
        &"x".repeat(10000),
        &format!(
            r#"{{"type":"object","properties":{{"{}": {{"type":"integer"}}}}}}"#,
            "a".repeat(1000)
        ),
    ];

    for case in edge_cases {
        let result = FormProofSchema::from_json(case);
        match result {
            Ok(_) => {}
            Err(e) => match e {
                SchemaError::NotAnObject
                | SchemaError::TooManyProperties(_)
                | SchemaError::TooManyEnumVariants(_, _)
                | SchemaError::StringTooLong(_, _)
                | SchemaError::UnsupportedType(_, _)
                | SchemaError::RequiredNotDefined(_)
                | SchemaError::InvalidJson(_)
                | SchemaError::MissingType(_)
                | SchemaError::EnumNotStrings(_)
                | SchemaError::InvalidBytes32Length(_)
                | SchemaError::MinGreaterThanMax(_) => {}
            },
        }
    }
}

#[test]
fn valid_schema_variations_all_parse() {
    let valid_schemas = [
        r#"{"type":"object","properties":{"a":{"type":"integer"}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"type":"integer","minimum":0}},"required":["a"]}"#,
        r#"{"type":"object","properties":{"a":{"type":"integer","maximum":100}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"type":"integer","minimum":0,"maximum":100}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"enum":["X"]}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"enum":["A","B","C","D","E","F","G","H"]}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"type":"string"}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"type":"string","maxLength":1}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"type":"string","maxLength":64}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"type":"string","format":"bytes32"}},"required":[]}"#,
        r#"{"type":"object","properties":{"a":{"type":"integer"},"b":{"type":"integer"},"c":{"type":"integer"},"d":{"type":"integer"},"e":{"type":"integer"},"f":{"type":"integer"},"g":{"type":"integer"},"h":{"type":"integer"}},"required":[]}"#,
    ];

    for schema_json in valid_schemas {
        let result = FormProofSchema::from_json(schema_json);
        assert!(
            result.is_ok(),
            "Schema should be valid: {} -> {:?}",
            schema_json,
            result.err()
        );
    }
}

#[test]
fn invalid_schema_variations_all_rejected() {
    let invalid_schemas = [
        (
            r#"{"type":"array","items":{"type":"integer"}}"#,
            "array type",
        ),
        (
            r#"{"type":"object","properties":{"a":{"type":"integer"},"b":{"type":"integer"},"c":{"type":"integer"},"d":{"type":"integer"},"e":{"type":"integer"},"f":{"type":"integer"},"g":{"type":"integer"},"h":{"type":"integer"},"i":{"type":"integer"}}}"#,
            "too many properties",
        ),
        (
            r#"{"type":"object","properties":{"a":{"enum":["A","B","C","D","E","F","G","H","I"]}}}"#,
            "too many enum variants",
        ),
        (
            r#"{"type":"object","properties":{"a":{"type":"string","maxLength":65}}}"#,
            "string too long",
        ),
        (
            r#"{"type":"object","properties":{"a":{"type":"integer","minimum":100,"maximum":50}}}"#,
            "min > max",
        ),
        (
            r#"{"type":"object","properties":{},"required":["missing"]}"#,
            "required not defined",
        ),
    ];

    for (schema_json, desc) in invalid_schemas {
        let result = FormProofSchema::from_json(schema_json);
        assert!(
            result.is_err(),
            "Schema should be rejected ({}): {}",
            desc,
            schema_json
        );
    }
}
