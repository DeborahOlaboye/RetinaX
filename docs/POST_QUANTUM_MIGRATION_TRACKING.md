# Tracking Issue: Full Post-Quantum Migration for RetinaX ZK Verification

- **Status**: In Progress / Phase 1 Complete
- **Assigned Domain**: Security & Zero-Knowledge Cryptography (`contracts/zk_verifier`, `sdk/zk_prover`)
- **Related Issue**: #266
- **Architecture Reference**: [`docs/post-quantum-zk-research.md`](file:///c:/Users/PAB-NETWORK/Downloads/RetinaX-2/docs/post-quantum-zk-research.md)

---

## 1. Problem Statement & Threat Model

The `zk_verifier` smart contract currently evaluates Zero-Knowledge Succinct Non-Interactive Arguments of Knowledge (zk-SNARKs) based on:
1. **BN254 (alt_bn128) Elliptic Curve**: A pairing-friendly elliptic curve of order $r \approx 2^{254}$.
2. **Groth16 & KZG-PLONK Protocols**: Protocols whose computational soundness rests on the Discrete Logarithm Problem (DLP) in $\mathbb{G}_1, \mathbb{G}_2$ and the Computational Bilinear Diffie-Hellman (CBDH) assumption in $\mathbb{G}_T$.

### Quantum Vulnerability (Shor's Algorithm)
Shor's algorithm solves abelian hidden subgroup problems and discrete logarithms in polynomial time:
$$\mathcal{O}((\log N)^3)$$
When a cryptanalytically relevant quantum computer (CRQC) becomes viable, an attacker can:
- Recover toxic waste parameters $(\alpha, \beta, \gamma, \delta)$ from trusted setups.
- Solve discrete logs to forge proofs of arbitrary false statements.
- Bypass access control in [`contracts/zk_verifier/src/lib.rs`](file:///c:/Users/PAB-NETWORK/Downloads/RetinaX-2/contracts/zk_verifier/src/lib.rs) and falsify optometry data in [`contracts/vision_records`](file:///c:/Users/PAB-NETWORK/Downloads/RetinaX-2/contracts/vision_records).

---

## 2. Post-Quantum Cryptographic Targets

| Metric | Current BN254 Groth16 | Hash-Based STARK (Target) | Lattice-Based ZK (Target) |
|---|---|---|---|
| **Underlying Primitive** | Elliptic Curve Pairings | Collision-Resistant Hashes (Blake3/Keccak) | Learning With Errors (LWE / SIS) |
| **Quantum Security** | ❌ Vulnerable (0 bits) | ✅ Post-Quantum (128+ bits) | ✅ Post-Quantum (128+ bits) |
| **Trusted Setup** | Required (Per-circuit toxic waste) | **Transparent** (No trusted setup) | Transparent or Standard Randomness |
| **Proof Footprint** | ~256 bytes | ~40 - 100 KB | ~10 - 30 KB |
| **Soroban CPU Feasibility** | Current baseline | Requires Host Precompiles / Chunking | Feasible with vectorized ops |

---

## 3. Migration Roadmap & Milestones

### Phase 1: Architectural Abstraction & Extensibility (Complete in #266)
- [x] Defined [`ProofSystemId`](file:///c:/Users/PAB-NETWORK/Downloads/RetinaX-2/contracts/zk_verifier/src/verifier.rs) enum supporting:
  - `Groth16Bn254` (ID 1)
  - `PlonkBn254` (ID 2)
  - `StarkFri` (ID 3)
  - `LatticeZk` (ID 4)
- [x] Introduced [`ExtensibleProof`](file:///c:/Users/PAB-NETWORK/Downloads/RetinaX-2/contracts/zk_verifier/src/verifier.rs) envelope.
- [x] Implemented strict validation and rejection of invalid proof system identifiers (`ProofValidationError::InvalidProofSystemId` -> `ContractError::InvalidProofSystem`).
- [x] Added unit and integration test suite in [`contracts/zk_verifier/tests/test_post_quantum_readiness.rs`](file:///c:/Users/PAB-NETWORK/Downloads/RetinaX-2/contracts/zk_verifier/tests/test_post_quantum_readiness.rs).

### Phase 2: Soroban Host Hash Precompiles & SDK Updates
- [ ] Implement prototype STARK FRI verifier in `contracts/zk_verifier/src/stark.rs`.
- [ ] Propose Stellar Core host functions for high-speed Blake3/Poseidon hash execution to satisfy CPU instruction limits for 40KB+ Merkle auth paths.
- [ ] Extend [`sdk/zk_prover`](file:///c:/Users/PAB-NETWORK/Downloads/RetinaX-2/sdk/zk_prover) to generate STARK execution traces for identity and consent circuits.

### Phase 3: Dual-Verification & Deprecation Period
- [ ] Deploy contract upgrade introducing parallel verification paths:
  - `request.proof_type == ProofSystemId::Groth16Bn254`: BN254 pairing evaluation.
  - `request.proof_type == ProofSystemId::StarkFri`: STARK FRI verification.
- [ ] Emit on-chain deprecation warnings for legacy BN254 proof submissions.
- [ ] Migrate optometry clinics and vision health clients to STARK prover client toolchain.

### Phase 4: Full Cutover & Legacy Purge
- [ ] Disable `ProofSystemId::Groth16Bn254` on-chain.
- [ ] Purge `ark-bn254` and elliptic curve pairing routines from contract binary, saving Wasm code size and attack surface.
