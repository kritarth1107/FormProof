# FormProof Threat Model

This document describes what FormProof protects against, what it does not protect against, and the trust assumptions involved.

## Security Model

FormProof uses Groth16 zero-knowledge proofs to allow a prover (agent) to convince a verifier (tool host) that private data satisfies schema constraints, without revealing the data itself.

### What is Public

- **Schema**: The JSON Schema defining constraints is always public. Both prover and verifier know the schema.
- **Commitment**: A SHA-256 hash of the serialized witness. This is the public input to the circuit.
- **Proving/Verifying Keys**: Circuit-specific keys generated during setup.

### What is Private

- **Witness Data**: The actual field values (amounts, currencies, names, etc.) are never revealed to the verifier.
- **Proving Key** (optionally): The prover needs the proving key, but it can be kept private from the verifier.

## Trust Assumptions

### Trusted Setup

FormProof uses Groth16, which requires a **circuit-specific trusted setup**. The setup generates proving and verifying keys for a particular schema.

**Implications:**
- If the toxic waste (random values from setup) is compromised, a malicious prover could generate fake proofs.
- In production, use a proper trusted setup ceremony or consider alternatives like PLONK.
- For this v0/toy implementation, the setup is done locally with `OsRng`.

### Circuit Correctness

The security relies on the circuit correctly encoding the schema constraints. Bugs in the circuit implementation could allow invalid witnesses to produce valid proofs.

### Cryptographic Assumptions

- BN254 curve security (~128-bit security level)
- SHA-256 collision resistance for commitment

## What a Malicious Tool Host Cannot Learn

Given only the proof and commitment, a malicious tool host **cannot**:

1. **Learn exact values**: Cannot determine that amount = 42, only that 0 ≤ amount ≤ 50
2. **Distinguish between valid witnesses**: Cannot tell if amount was 10 or 40, only that it satisfied constraints
3. **Extract the witness**: The proof reveals nothing beyond constraint satisfaction
4. **Correlate proofs**: Two proofs with different commitments cannot be linked to the same prover (unless commitment reuse)

## What a Malicious Tool Host CAN Learn

A malicious tool host **can**:

1. **Know the schema**: The schema is public by design
2. **Know constraints were satisfied**: That's the point of verification
3. **See the commitment**: Can store and compare commitments
4. **Perform timing attacks**: Proving time may leak information about witness complexity
5. **Refuse service**: Can reject valid proofs arbitrarily

## What a Cheating Agent Cannot Prove

A malicious agent **cannot** generate a valid proof for a witness that violates the schema:

1. **Out-of-range integers**: Cannot prove amount ≤ 50 if amount = 100
2. **Invalid enum values**: Cannot prove currency ∈ {USD, EUR, GBP} if currency = "INVALID"
3. **Missing required fields**: Cannot prove required fields exist if they're absent
4. **String length violations**: Cannot prove length ≤ 64 if string is longer

**Unless** the trusted setup is compromised or there's a circuit bug.

## What FormProof Does NOT Provide

### Not Authentication

FormProof proves **what** the data contains, not **who** sent it. It does not replace:

- OAuth / OpenID Connect for identity
- API keys for authorization
- TLS for transport security
- Digital signatures for non-repudiation

Use FormProof **alongside** authentication, not instead of it.

### Not Schema Privacy

The schema (policy) is always public. If you need to hide the constraints themselves, FormProof is not suitable.

### Not Arbitrary JSON Schema

Only the v0 subset is supported. See [SCHEMA_V0.md](SCHEMA_V0.md) for exact grammar.

### Not Production-Ready

This is a v0/toy implementation with known limitations:

- Circuit-specific trusted setup (no universal setup)
- Limited schema expressiveness
- No formal security audit
- Performance not optimized for production

## Commitment Scheme

The witness commitment uses SHA-256:

```
commitment = SHA256(serialize(witness))
```

**Properties:**
- **Binding**: Cannot find two different witnesses with the same commitment (collision resistance)
- **Hiding**: Commitment reveals nothing about witness (preimage resistance)

**Limitations:**
- Commitment is deterministic: same witness always produces same commitment
- Verifier can test guesses if the value space is small (e.g., boolean fields)

## Replay Attacks

FormProof does not inherently prevent replay attacks. A valid proof can be submitted multiple times.

**Mitigations (application-level):**
- Include nonces in the schema
- Track used commitments
- Timestamp or sequence requirements

## Side Channels

### Timing

- Proving time may vary with witness complexity
- Verification time is essentially constant

### Proof Size

- Groth16 proofs are constant size regardless of witness
- Does not leak information about witness size

## Recommendations

1. **Use proper trusted setup** for production deployments
2. **Combine with authentication** (OAuth, API keys)
3. **Include nonces** to prevent replay
4. **Audit the circuit** for constraint correctness
5. **Don't rely on commitment privacy** for small value spaces
6. **Consider the v0 limitations** before production use
