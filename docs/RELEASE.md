# Release Checklist

Steps for preparing a new FormProof release.

## v0.1.0 Pre-release Checklist

Outstanding items before tagging v0.1.0:

### Schemas
- [x] Core policy schemas: refund, age_gate, access_country, spend_cap
- [x] MCP schemas: session_ttl, rate_limit, tool_allowlist, quota_budget, model_route
- [x] All schemas have catalog entries in schemas/README.md
- [x] All schemas have boundary tests

### Benchmarks
- [x] Criterion benches for all major schemas
- [x] Prove/verify benchmarks documented in README

### Documentation
- [x] SCHEMA_V0.md frozen specification
- [x] HOST_INTEGRATION.md verify-only guide
- [x] PROOF_PACKAGE.md portable format docs
- [ ] WASM.md: verify browser build actually works (currently deferred)
- [ ] API docs: ensure all public items have rustdoc comments

### Testing
- [x] Golden proof tests
- [x] Property-based tests (proptest)
- [x] Boundary tests for each schema
- [ ] Integration test with real MCP host (manual verification)

### CI
- [x] Format, clippy, test, docs jobs
- [x] Schema JSON validation
- [x] Schema parsing validation
- [x] Security audit job

### Pre-tag verification
- [ ] Verify all examples run without errors
- [ ] Verify CLI commands work end-to-end
- [ ] Review CHANGELOG for completeness

---

## General Pre-release Steps

1. **Version bump** — update `version` in:
   - `Cargo.toml` (workspace `[workspace.package]`)

2. **Update CHANGELOG.md**
   - Move items from `[Unreleased]` to a new versioned section
   - Add release date in `## [X.Y.Z] - YYYY-MM-DD` format

3. **Run checks**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo doc --no-deps -p formproof
   ```

4. **Build release**
   ```bash
   cargo build --release
   ./target/release/formproof --help
   ```

## Tagging

1. Create annotated tag:
   ```bash
   git tag -a v0.X.Y -m "Release v0.X.Y"
   ```

2. Push tag:
   ```bash
   git push origin v0.X.Y
   ```

## Tag naming

- Use semantic versioning: `vMAJOR.MINOR.PATCH`
- Pre-releases: `v0.1.0-alpha.1`, `v0.1.0-rc.1`

## crates.io (not yet)

Crate publishing is deferred until the API stabilizes. For now, use git dependencies:

```toml
formproof = { git = "https://github.com/kritarth1107/FormProof", tag = "v0.X.Y" }
```

## Post-release

- Verify the tag appears on GitHub releases
- Update any dependent projects to the new tag
