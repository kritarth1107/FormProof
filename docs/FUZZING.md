# Property Testing and Fuzzing

FormProof includes property-based tests and fuzz-like testing to ensure
the schema parser and circuit behave correctly on arbitrary inputs.

## Property Tests

Property tests use [proptest](https://crates.io/crates/proptest) to generate
random valid and invalid schemas, then verify the parser and circuit behave
correctly.

### Running Property Tests

```bash
# Run all property tests
cargo test --test proptest_schema

# Run with verbose output
cargo test --test proptest_schema -- --nocapture
```

### What the Property Tests Cover

**Schema Parser Tests:**
- Valid schemas (1-8 properties, all supported types) parse successfully
- Schema roundtrip (parse → serialize → parse) preserves structure
- Invalid schemas (too many properties, bad min/max, etc.) are rejected with correct errors

**Witness/Circuit Tests:**
- Valid witnesses satisfy circuit constraints
- Invalid witnesses (out of range, wrong enum, missing required) fail constraints
- Full prove/verify roundtrip works for various schema types
- Different witnesses produce different commitments

### Configuration

Tests use deterministic seeds for reproducibility. The default configuration runs
30-50 test cases per property, which balances coverage with CI runtime.

To increase test coverage locally:

```bash
# Run more test cases
PROPTEST_CASES=500 cargo test --test proptest_schema
```

## Fuzz-Like Tests

The `fuzz_schema` test suite feeds arbitrary and malformed JSON to the schema
parser, verifying it never panics—only returns structured errors.

### Running Fuzz Tests

```bash
# Run fuzz-like tests
cargo test --test fuzz_schema

# Run with verbose output
cargo test --test fuzz_schema -- --nocapture
```

### What the Fuzz Tests Cover

- Arbitrary JSON strings (valid and invalid)
- Random byte sequences interpreted as UTF-8
- Deeply nested JSON structures
- Malformed schema variations (wrong types, out-of-bounds values)
- Hand-selected edge cases from a curated corpus

### Fuzz Corpus Edge Cases

The test includes a hand-curated corpus of edge cases:
- Empty strings and simple JSON primitives
- Truncated/malformed JSON
- Deeply nested objects
- Schemas with null/wrong-type fields
- Boundary values for all constraints
- Very long strings

## Continuous Fuzzing with cargo-fuzz

For deeper fuzzing coverage, you can use cargo-fuzz. This is optional and
requires nightly Rust.

### Setup

```bash
# Install cargo-fuzz (requires nightly)
rustup install nightly
cargo +nightly install cargo-fuzz

# Initialize fuzz targets (if not already present)
cd formproof
cargo +nightly fuzz init
```

### Creating a Fuzz Target

Create `formproof/fuzz/fuzz_targets/fuzz_schema_parser.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use formproof::FormProofSchema;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = FormProofSchema::from_json(s);
    }
});
```

### Running the Fuzzer

```bash
# Run fuzzing (Ctrl+C to stop)
cd formproof
cargo +nightly fuzz run fuzz_schema_parser

# Run for a specific duration
cargo +nightly fuzz run fuzz_schema_parser -- -max_total_time=60

# Run with a corpus
cargo +nightly fuzz run fuzz_schema_parser corpus/
```

### Reproducing Crashes

If the fuzzer finds a crash:

```bash
# Reproduce with the crash input
cargo +nightly fuzz run fuzz_schema_parser crash-<hash>

# Minimize the crash input
cargo +nightly fuzz tmin fuzz_schema_parser crash-<hash>
```

## CI Integration

Property tests run as part of normal `cargo test`:

```yaml
- name: Run tests
  run: cargo test --all-features
```

The fuzz-like tests are fast enough for CI (< 1 second). True continuous fuzzing
with cargo-fuzz is intended for local development or dedicated fuzzing
infrastructure, not standard CI runs.

## Adding New Test Cases

### Property Tests

Add new property tests to `formproof/tests/proptest_schema.rs`:

```rust
proptest! {
    #[test]
    fn my_new_property(value in some_strategy()) {
        // Test property here
        prop_assert!(/* condition */);
    }
}
```

### Fuzz Corpus

Add edge cases to the `fuzz_corpus_hand_selected_edge_cases` test in
`formproof/tests/fuzz_schema.rs`:

```rust
let edge_cases = [
    // ... existing cases ...
    r#"{"your":"new","edge":"case"}"#,
];
```

## Performance Notes

- Property tests are bounded to keep CI fast (30-50 cases per property)
- Fuzz tests run 200 random iterations per strategy
- Full prove/verify roundtrip tests are limited to 3-5 cases per schema
- Circuit compilation is the slowest operation; tests reuse compiled schemas

For exhaustive testing, increase case counts locally or set up dedicated
fuzzing infrastructure.
