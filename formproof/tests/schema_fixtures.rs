//! Tests that all schema fixtures in schemas/ parse and compile correctly.

use formproof::{CompiledSchema, FormProofSchema};
use std::fs;
use std::path::Path;

fn schema_dir() -> &'static Path {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("schemas");
    Box::leak(path.into_boxed_path())
}

#[test]
fn all_schema_fixtures_parse_and_compile() {
    let schemas_path = schema_dir();
    assert!(
        schemas_path.exists(),
        "schemas/ directory not found at {:?}",
        schemas_path
    );

    let mut tested = 0;
    let mut failures: Vec<String> = Vec::new();

    for entry in fs::read_dir(schemas_path).expect("failed to read schemas/") {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();

        let ext = path.extension();
        if ext.is_none() || ext.unwrap() != "json" {
            continue;
        }

        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let json = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", filename, e));

        let schema = match FormProofSchema::from_json(&json) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!("{}: parse error: {}", filename, e));
                continue;
            }
        };

        match CompiledSchema::compile(schema) {
            Ok(_) => tested += 1,
            Err(e) => failures.push(format!("{}: compile error: {}", filename, e)),
        }
    }

    if !failures.is_empty() {
        panic!("Schema fixture failures:\n{}", failures.join("\n"));
    }

    assert!(
        tested >= 1,
        "expected at least 1 schema fixture, found {}",
        tested
    );
    println!("Validated {} schema fixtures", tested);
}
