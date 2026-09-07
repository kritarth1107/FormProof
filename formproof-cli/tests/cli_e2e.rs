//! End-to-end tests for the formproof CLI.
//!
//! These tests exercise the CLI commands (info, compile, prove, verify,
//! package-build, package-verify) against real schemas under `schemas/`.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli crate should be in workspace")
        .to_path_buf()
}

fn cli_binary() -> PathBuf {
    let mut path = project_root();
    path.push("target");
    path.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    path.push("formproof");
    path
}

fn schema_path(name: &str) -> PathBuf {
    let mut path = project_root();
    path.push("schemas");
    path.push(name);
    path
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(cli_binary())
        .args(args)
        .output()
        .expect("failed to execute CLI")
}

fn assert_success(output: &Output) {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "CLI command failed:\nstdout: {}\nstderr: {}",
            stdout, stderr
        );
    }
}

fn assert_failure(output: &Output) {
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "CLI command should have failed but succeeded:\nstdout: {}",
            stdout
        );
    }
}

mod info_tests {
    use super::*;

    #[test]
    fn info_displays_spend_cap_schema() {
        let output = run_cli(&[
            "info",
            "--schema",
            schema_path("spend_cap.json").to_str().unwrap(),
        ]);
        assert_success(&output);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("FormProof Schema Info"),
            "should show header"
        );
        assert!(stdout.contains("Properties: 2"), "should show 2 properties");
        assert!(stdout.contains("cents"), "should list cents property");
        assert!(stdout.contains("currency"), "should list currency property");
        assert!(stdout.contains("Fingerprint:"), "should show fingerprint");
    }

    #[test]
    fn info_displays_age_gate_schema() {
        let output = run_cli(&[
            "info",
            "--schema",
            schema_path("age_gate.json").to_str().unwrap(),
        ]);
        assert_success(&output);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("age"), "should list age property");
        assert!(stdout.contains("region"), "should list region property");
        assert!(stdout.contains("[REQUIRED]"), "should mark required fields");
    }

    #[test]
    fn info_fails_on_missing_schema() {
        let output = run_cli(&["info", "--schema", "nonexistent.json"]);
        assert_failure(&output);
    }
}

mod compile_tests {
    use super::*;

    #[test]
    fn compile_spend_cap_schema() {
        let tmp = TempDir::new().expect("create temp dir");

        let output = run_cli(&[
            "compile",
            "--schema",
            schema_path("spend_cap.json").to_str().unwrap(),
            "--output",
            tmp.path().to_str().unwrap(),
        ]);
        assert_success(&output);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Compiled successfully"),
            "should report success"
        );

        assert!(
            tmp.path().join("proving_key.bin").exists(),
            "proving key should exist"
        );
        assert!(
            tmp.path().join("verifying_key.bin").exists(),
            "verifying key should exist"
        );
        assert!(
            tmp.path().join("schema.json").exists(),
            "schema copy should exist"
        );
    }

    #[test]
    fn compile_age_gate_schema() {
        let tmp = TempDir::new().expect("create temp dir");

        let output = run_cli(&[
            "compile",
            "--schema",
            schema_path("age_gate.json").to_str().unwrap(),
            "--output",
            tmp.path().to_str().unwrap(),
        ]);
        assert_success(&output);

        let pk_path = tmp.path().join("proving_key.bin");
        let vk_path = tmp.path().join("verifying_key.bin");

        assert!(pk_path.exists());
        assert!(vk_path.exists());

        let pk_size = fs::metadata(&pk_path).unwrap().len();
        let vk_size = fs::metadata(&vk_path).unwrap().len();

        assert!(pk_size > 1000, "proving key should be substantial");
        assert!(vk_size > 100, "verifying key should be substantial");
    }

    #[test]
    fn compile_fails_on_invalid_schema() {
        let tmp = TempDir::new().expect("create temp dir");
        let invalid_schema = tmp.path().join("invalid.json");
        fs::write(&invalid_schema, "{ not valid json }").unwrap();

        let output = run_cli(&[
            "compile",
            "--schema",
            invalid_schema.to_str().unwrap(),
            "--output",
            tmp.path().to_str().unwrap(),
        ]);
        assert_failure(&output);
    }
}

mod prove_verify_tests {
    use super::*;

    fn setup_compiled_schema(schema_name: &str) -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().expect("create temp dir");

        let output = run_cli(&[
            "compile",
            "--schema",
            schema_path(schema_name).to_str().unwrap(),
            "--output",
            tmp.path().to_str().unwrap(),
        ]);
        assert_success(&output);

        let pk_path = tmp.path().join("proving_key.bin");
        let vk_path = tmp.path().join("verifying_key.bin");

        (tmp, pk_path, vk_path)
    }

    #[test]
    fn prove_and_verify_spend_cap() {
        let (tmp, pk_path, vk_path) = setup_compiled_schema("spend_cap.json");

        let witness_json = r#"{"cents": 5000, "currency": "USD"}"#;
        let witness_path = tmp.path().join("witness.json");
        fs::write(&witness_path, witness_json).unwrap();

        let proof_path = tmp.path().join("proof.bin");

        let prove_output = run_cli(&[
            "prove",
            "--schema",
            schema_path("spend_cap.json").to_str().unwrap(),
            "--proving-key",
            pk_path.to_str().unwrap(),
            "--witness",
            witness_path.to_str().unwrap(),
            "--output",
            proof_path.to_str().unwrap(),
        ]);
        assert_success(&prove_output);

        let stdout = String::from_utf8_lossy(&prove_output.stdout);
        assert!(stdout.contains("Proof generated successfully"));
        assert!(stdout.contains("Commitment:"));

        let commitment = stdout
            .lines()
            .find(|l| l.contains("Commitment:"))
            .and_then(|l| l.split_whitespace().last())
            .expect("should have commitment in output");

        let verify_output = run_cli(&[
            "verify",
            "--schema",
            schema_path("spend_cap.json").to_str().unwrap(),
            "--verifying-key",
            vk_path.to_str().unwrap(),
            "--proof",
            proof_path.to_str().unwrap(),
            "--commitment",
            commitment,
        ]);
        assert_success(&verify_output);

        let verify_stdout = String::from_utf8_lossy(&verify_output.stdout);
        assert!(verify_stdout.contains("Proof is VALID"));
    }

    #[test]
    fn prove_and_verify_age_gate() {
        let (tmp, pk_path, vk_path) = setup_compiled_schema("age_gate.json");

        let witness_json = r#"{"age": 21, "region": "US"}"#;
        let witness_path = tmp.path().join("witness.json");
        fs::write(&witness_path, witness_json).unwrap();

        let proof_path = tmp.path().join("proof.bin");

        let prove_output = run_cli(&[
            "prove",
            "--schema",
            schema_path("age_gate.json").to_str().unwrap(),
            "--proving-key",
            pk_path.to_str().unwrap(),
            "--witness",
            witness_path.to_str().unwrap(),
            "--output",
            proof_path.to_str().unwrap(),
        ]);
        assert_success(&prove_output);

        let stdout = String::from_utf8_lossy(&prove_output.stdout);
        let commitment = stdout
            .lines()
            .find(|l| l.contains("Commitment:"))
            .and_then(|l| l.split_whitespace().last())
            .expect("should have commitment");

        let verify_output = run_cli(&[
            "verify",
            "--schema",
            schema_path("age_gate.json").to_str().unwrap(),
            "--verifying-key",
            vk_path.to_str().unwrap(),
            "--proof",
            proof_path.to_str().unwrap(),
            "--commitment",
            commitment,
        ]);
        assert_success(&verify_output);

        let verify_stdout = String::from_utf8_lossy(&verify_output.stdout);
        assert!(verify_stdout.contains("Proof is VALID"));
    }

    #[test]
    fn verify_fails_with_wrong_commitment() {
        let (tmp, pk_path, vk_path) = setup_compiled_schema("spend_cap.json");

        let witness_json = r#"{"cents": 5000, "currency": "USD"}"#;
        let witness_path = tmp.path().join("witness.json");
        fs::write(&witness_path, witness_json).unwrap();

        let proof_path = tmp.path().join("proof.bin");

        let prove_output = run_cli(&[
            "prove",
            "--schema",
            schema_path("spend_cap.json").to_str().unwrap(),
            "--proving-key",
            pk_path.to_str().unwrap(),
            "--witness",
            witness_path.to_str().unwrap(),
            "--output",
            proof_path.to_str().unwrap(),
        ]);
        assert_success(&prove_output);

        let wrong_commitment = "0000000000000000000000000000000000000000000000000000000000000000";

        let verify_output = run_cli(&[
            "verify",
            "--schema",
            schema_path("spend_cap.json").to_str().unwrap(),
            "--verifying-key",
            vk_path.to_str().unwrap(),
            "--proof",
            proof_path.to_str().unwrap(),
            "--commitment",
            wrong_commitment,
        ]);
        assert_failure(&verify_output);
    }
}
