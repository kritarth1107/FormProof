# WebAssembly Support

This document describes the current state of WASM support for FormProof verification
and the path toward lightweight browser/edge verification.

## Current Status: Not Ready

**FormProof does not currently have production-ready WASM support.** This document
outlines the technical path and honest caveats.

## The Goal

Enable a host (browser, edge worker, embedded system) to verify FormProof proofs
without running the full Rust prover. The verifier is much lighter than the prover,
making WASM deployment feasible for verification-only use cases.

## Technical Path: arkworks-wasm

The [arkworks](https://arkworks.rs/) ecosystem (which FormProof uses) has experimental
WASM support via the `ark-serialize` and `ark-ec` crates with `wasm-bindgen`.

### What Would Work

```rust
// Hypothetical verify-only WASM API
use formproof::{verify, CompiledSchema, Proof, FormProofSchema};

#[wasm_bindgen]
pub fn verify_proof(
    schema_json: &str,
    vk_bytes: &[u8],
    proof_bytes: &[u8],
    commitment: &[u8],
) -> Result<bool, JsValue> {
    let schema = FormProofSchema::from_json(schema_json)?;
    // Load only the verifying key (not the proving key)
    let compiled = CompiledSchema::deserialize_vk_only(schema, vk_bytes)?;
    let proof = Proof::deserialize(proof_bytes, commitment.try_into()?)?;
    Ok(verify(&compiled, &proof)?)
}
```

### Required Changes

1. **Verify-only deserialization**: Add `CompiledSchema::deserialize_vk_only()` that
   loads only the verifying key (the proving key is much larger and not needed for
   verification).

2. **WASM feature flag**: Gate WASM-specific code behind a feature to avoid bloating
   the native build.

3. **RNG handling**: Replace `OsRng` with a WASM-compatible RNG for any operations
   that need randomness (verification doesn't, but some arkworks APIs require it).

4. **Build configuration**: Use `wasm-pack` or `wasm-bindgen` CLI with appropriate
   target and optimization settings.

## Size and Performance Caveats

### Bundle Size

| Component | Estimated Size (gzipped) |
|-----------|--------------------------|
| arkworks BN254 verifier | ~200-400 KB |
| FormProof verify logic | ~20-50 KB |
| wasm-bindgen glue | ~10-20 KB |
| **Total** | **~250-500 KB** |

These are rough estimates. Actual size depends on:
- Optimization level (`wasm-opt -Oz` vs `-O3`)
- Feature flags and dead code elimination
- Whether curves are tree-shaken properly

### Performance

- **Verification**: Expected 2-10x slower than native (arkworks pairing operations
  are compute-intensive)
- **Key loading**: Deserializing the verifying key may take 50-200ms in WASM
- **No SIMD**: WASM SIMD support in arkworks is experimental

### Browser Compatibility

- Chrome/Edge: Full support via WebAssembly
- Firefox: Full support
- Safari: Works but may have performance variations
- Node.js: Works with `--experimental-wasm-*` flags

## What is NOT Ready

1. **No `formproof-wasm` crate exists yet** — This document describes the path, not
   a shipped feature.

2. **No tested WASM build** — arkworks WASM compilation has known rough edges
   (missing `getrandom` backends, feature flag conflicts).

3. **No performance benchmarks** — The estimates above are extrapolated from similar
   arkworks projects, not measured on FormProof.

4. **No JavaScript API design** — The TypeScript types, error handling, and async
   patterns for browser use haven't been designed.

5. **No proving in WASM** — Proof generation requires the proving key (~10-50 MB
   serialized) and is too slow/large for browser use. Only verification is practical.

## Future Work

If WASM support becomes a priority:

1. Add a `wasm` feature flag to `formproof/Cargo.toml`
2. Create `formproof-wasm` crate with `wasm-bindgen` exports
3. Implement `deserialize_vk_only()` for lightweight verification
4. Add integration tests using `wasm-pack test`
5. Benchmark and optimize bundle size
6. Publish to npm as `@formproof/wasm`

## Alternative: Server-Side Verification

For many use cases, server-side verification is simpler:

- Host runs native FormProof verifier
- Client submits proof + commitment via HTTP
- No WASM bundle, no browser compatibility concerns
- Verification is fast (< 2ms native)

This is the recommended approach until WASM support matures.

## References

- [arkworks WASM discussion](https://github.com/arkworks-rs/algebra/issues/299)
- [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
