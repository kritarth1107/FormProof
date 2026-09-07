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
