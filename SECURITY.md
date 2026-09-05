# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes (best-effort while v0 is unstable) |

FormProof is an early public release. Treat it as research / prototype-grade until a later version is explicitly marked production-ready.

## What FormProof Protects

When used correctly, a verifier learns that a private witness satisfies a **public** schema, without learning field values. See [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md).

## What FormProof Does Not Protect

- Schema / policy secrecy (schemas are public)
- Authentication or authorization of the caller (use OAuth / similar)
- Replay without an application-level nonce or commitment store
- Side channels in your host integration
- Compromised proving keys or malicious circuits from a bad trusted setup

## Reporting a Vulnerability

Please report security issues privately when possible:

- Email: **singhalkritarth@gmail.com** with subject `FormProof security`
- Or open a GitHub security advisory on this repository if you prefer GitHub's private reporting flow

Include:

1. Affected commit SHA or version
2. Description and impact
3. Minimal reproduction (schema + witness / proof steps) if safe to share

Do **not** open a public issue for unfixed cryptographic or verification bypasses.

## Expected Response

- Acknowledgement within a few days when possible
- Fix or public write-up depending on severity and whether a patch is feasible in v0

## Safe Disclosure Guidelines

- Prefer private disclosure until a fix or mitigation is available
- Avoid publishing exploit proofs against live deployments that rely on FormProof
- Coordinated disclosure preferred for issues that affect proof soundness or verification bypass
