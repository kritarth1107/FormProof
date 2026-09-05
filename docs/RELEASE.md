# Release Checklist

Steps for preparing a new FormProof release.

## Pre-release

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
