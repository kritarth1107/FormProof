//! Boundary tests for rate_limit and tool_allowlist policy schemas.
//!
//! Verifies that the MCP policy schemas correctly accept boundary values
//! and reject out-of-range values.

use formproof::{verify, CompiledSchema, FormProofSchema, Proof, Witness};
use std::fs;
use std::path::Path;

fn load_schema(name: &str) -> FormProofSchema {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas")
        .join(name);
    let json = fs::read_to_string(path).expect("Failed to read schema");
    FormProofSchema::from_json(&json).expect("Invalid schema")
}

mod rate_limit {
    use super::*;

    #[test]
    fn accepts_minimum_requests() {
        let schema = load_schema("rate_limit.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        let mut witness = Witness::new();
        witness.set_u64("requests_per_window", 1);
        witness.set_u64("window_secs", 60);
        witness.set_enum("tier", "free");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected requests_per_window=1 to be accepted");
    }

    #[test]
    fn accepts_maximum_requests() {
        let schema = load_schema("rate_limit.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        let mut witness = Witness::new();
        witness.set_u64("requests_per_window", 10000);
        witness.set_u64("window_secs", 3600);
        witness.set_enum("tier", "enterprise");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected requests_per_window=10000 to be accepted");
    }

    #[test]
    fn accepts_maximum_window() {
        let schema = load_schema("rate_limit.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        let mut witness = Witness::new();
        witness.set_u64("requests_per_window", 100);
        witness.set_u64("window_secs", 86400);
        witness.set_enum("tier", "pro");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected window_secs=86400 (24h) to be accepted");
    }

    #[test]
    fn rejects_zero_requests() {
        use std::panic;

        let schema = load_schema("rate_limit.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        let mut witness = Witness::new();
        witness.set_u64("requests_per_window", 0);
        witness.set_u64("window_secs", 60);
        witness.set_enum("tier", "free");

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            Proof::create(&compiled, &witness)
        }));

        if let Ok(Ok(proof)) = result {
            let verified = verify(&compiled, &proof).unwrap_or(false);
            assert!(!verified, "Expected requests_per_window=0 to be rejected");
        }
    }

    #[test]
    fn all_tiers_accepted() {
        let schema = load_schema("rate_limit.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        for tier in &["free", "basic", "pro", "enterprise"] {
            let mut witness = Witness::new();
            witness.set_u64("requests_per_window", 100);
            witness.set_u64("window_secs", 3600);
            witness.set_enum("tier", tier);

            let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
            let result = verify(&compiled, &proof).expect("Verification failed");
            assert!(result, "Expected tier={} to be accepted", tier);
        }
    }
}

mod tool_allowlist {
    use super::*;

    #[test]
    fn accepts_all_tools() {
        let schema = load_schema("tool_allowlist.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        for tool in &["read", "write", "execute", "list", "search", "delete"] {
            let mut witness = Witness::new();
            witness.set_enum("tool_name", tool);
            witness.set_enum("scope", "local");
            witness.set_absent("max_args");

            let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
            let result = verify(&compiled, &proof).expect("Verification failed");
            assert!(result, "Expected tool_name={} to be accepted", tool);
        }
    }

    #[test]
    fn accepts_all_scopes() {
        let schema = load_schema("tool_allowlist.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        for scope in &["local", "remote", "any"] {
            let mut witness = Witness::new();
            witness.set_enum("tool_name", "read");
            witness.set_enum("scope", scope);

            let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
            let result = verify(&compiled, &proof).expect("Verification failed");
            assert!(result, "Expected scope={} to be accepted", scope);
        }
    }

    #[test]
    fn accepts_max_args_boundary() {
        let schema = load_schema("tool_allowlist.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        let mut witness = Witness::new();
        witness.set_enum("tool_name", "execute");
        witness.set_u64("max_args", 64);
        witness.set_enum("scope", "remote");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected max_args=64 to be accepted");
    }

    #[test]
    fn accepts_zero_max_args() {
        let schema = load_schema("tool_allowlist.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        let mut witness = Witness::new();
        witness.set_enum("tool_name", "list");
        witness.set_u64("max_args", 0);
        witness.set_enum("scope", "any");

        let proof = Proof::create(&compiled, &witness).expect("Proof generation failed");
        let result = verify(&compiled, &proof).expect("Verification failed");
        assert!(result, "Expected max_args=0 to be accepted");
    }

    #[test]
    fn rejects_max_args_over_64() {
        use std::panic;

        let schema = load_schema("tool_allowlist.json");
        let compiled = CompiledSchema::compile(schema).expect("Compilation failed");

        let mut witness = Witness::new();
        witness.set_enum("tool_name", "execute");
        witness.set_u64("max_args", 65);
        witness.set_enum("scope", "local");

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            Proof::create(&compiled, &witness)
        }));

        if let Ok(Ok(proof)) = result {
            let verified = verify(&compiled, &proof).unwrap_or(false);
            assert!(!verified, "Expected max_args=65 to be rejected");
        }
    }
}
