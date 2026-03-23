# Safety: Security Model and Threat Analysis [SPEC]

> **Version**: 3.0 | **Status**: Draft
>
> **Depends on**: `06-economy.md`, `../10-safety/00-defense.md`, `../20-styx/01-architecture.md`

---

> **Reader orientation:** This document specifies the security model and threat analysis for the Grimoire (the Golem's persistent local knowledge base) and Styx (Bardo's global knowledge relay). It belongs to the **04-memory** layer. The key concept is a four-stage ingestion pipeline that acts as the Golem's immune system for external knowledge: every entry from Styx, Clade (sibling Golems), or the marketplace passes through validation, sanitization, confidence discounting, and quarantine before reaching the active Grimoire. For term definitions, see `prd2/shared/glossary.md`.

## Why memory safety matters

> **Moat framing (Secrets):** Every API call, every inference query leaks metadata. Memory safety protects the owner's strategies, portfolio positions, and behavioral patterns. A compromised memory system doesn't just expose knowledge — it exposes the owner's entire decision-making history. This is why the Grimoire's ingestion pipeline, immune memory, and Styx encryption work together as a Secrets moat: the architecture is designed so that nobody outside the Golem's trust boundary can read its cognition.

Every API call, every inference query leaks metadata about the owner. Memory safety is not about protecting the agent's data -- it is about protecting the owner's strategies, portfolio, and behavior patterns. A compromised memory system does not just expose knowledge. It exposes the owner's entire decision-making history, their risk tolerance, their position sizes, and their behavioral patterns. The threat model must protect not just the Golem's data, but the owner's privacy as a first-class constraint.

---

## Four-stage ingestion pipeline

All externally sourced knowledge (from Styx Clade, Styx Lethe (formerly Lethe), Marketplace, or inheritance) passes through a four-stage ingestion pipeline before reaching the active Grimoire. The pipeline is the Golem's immune system for knowledge.

### Stage 1: QUARANTINE

EIP-712 signature verification. Every incoming entry must carry a valid signature from its claimed source. Entries without valid signatures are rejected. Entries with valid signatures from unknown sources are quarantined for manual review (or auto-rejection, configurable).

```rust
pub struct IngestResult {
    pub source: IngestSource,
    pub stage: IngestStage, // Quarantine | Consensus | Sandbox | Adopt
    pub outcome: IngestOutcome, // Accepted | Rejected | Quarantined
}
```

Each ingestion attempt emits an `IngestResult` event.

### Stage 2: CONSENSUS

Two checks run in parallel:

1. **Embedding-content alignment**: The entry's embedding is recomputed locally and compared to the claimed embedding. Cosine similarity must exceed 0.85. Below that threshold, the content and embedding are misaligned -- either the entry has been tampered with or was embedded with a different model. Rejected.

2. **On-chain claim verification**: If the entry references on-chain state (pool addresses, transaction hashes, block numbers), verify those claims against the local chain client. Entries with falsifiable on-chain claims that fail verification are rejected with high confidence.

### Stage 3: SKILL SANDBOX

Actionable entries (heuristics, strategy fragments) are decomposed using the Voyager pattern [VOYAGER-2023]: each actionable claim is extracted as an independent skill, tested against historical data or simulated conditions, and scored by predicted outcome. This is the most expensive stage (~$0.002 per entry, one Haiku call) and runs only for entries that passed stages 1-2.

### Stage 4: ADOPT

Confidence discounting applied at adoption:

| Source | Discount Factor | Result |
|--------|----------------|--------|
| Inheritance | `confidence * 0.85^generation` | Gen 1: x0.85, Gen 2: x0.72, Gen 3: x0.61 |
| Clade sibling | `confidence * 0.80` | Trusted but not self-learned |
| Lethe | `confidence * 0.50` | Anonymized, unverifiable provenance |
| Marketplace | `confidence * 0.60` | Seller has economic incentive; escrow provides some accountability |

Entries that survive all four stages enter the Grimoire at their discounted confidence. Entries that fail any stage are logged (for immune memory learning) and discarded.

---

## Immune memory

The ingestion pipeline learns from attack patterns. Failed ingestion attempts are recorded in an `immune_memory` SQLite table:

```sql
CREATE TABLE immune_memory (
    id TEXT PRIMARY KEY,
    pattern_hash TEXT NOT NULL,       -- LSH hash of the rejected entry's embedding
    rejection_stage TEXT NOT NULL,    -- quarantine | consensus | sandbox
    rejection_reason TEXT NOT NULL,
    source_fingerprint TEXT,          -- anonymized source identifier
    first_seen_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    occurrence_count INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_immune_pattern ON immune_memory(pattern_hash);
CREATE INDEX idx_immune_source ON immune_memory(source_fingerprint);
```

When a new entry arrives at the QUARANTINE stage, its embedding is hashed and checked against `immune_memory`. If a matching pattern exists with `occurrence_count >= 3`, the entry is fast-rejected without running the full pipeline. This is adaptive immunity: the Golem learns to recognize attack patterns and reject them efficiently.

The immune memory table is included in death bundles so successors inherit attack pattern knowledge. Inherited immune entries enter at `confidence * 0.85^generation` like all other inherited knowledge.

### Bloom Oracle (LSH filters)

For high-throughput pre-screening, the immune system maintains a Bloom filter built from LSH (Locality-Sensitive Hashing) signatures of known-bad entries. Specifications:

- Filter size: ~4KB
- Hash functions: 7
- False positive rate: ~1%
- Update frequency: every Curator cycle (50 ticks)

The Bloom filter is the first check in the QUARANTINE stage. A positive hit triggers full immune memory lookup. A negative result lets the entry proceed to signature verification. The 1% false positive rate is acceptable because false positives only trigger a (cheap) database lookup, not a rejection.

---

## Encryption summary table

The memory system handles data at six distinct layers, each with different encryption properties. The fundamental design principle is **structural impossibility**: Bardo infrastructure cannot read owner knowledge because it never possesses the decryption key.

| Layer                            | Encrypted?                   | Key                         | Plaintext Exposed To                                                    |
| -------------------------------- | ---------------------------- | --------------------------- | ----------------------------------------------------------------------- |
| **Styx Archive blobs**             | Yes (AES-256-GCM)            | Owner SEK                   | Nobody (client-side encrypt/decrypt)                                    |
| **Styx L0/L1 content**          | Yes (AES-256-GCM)            | Owner SEK                   | Nobody (client-side decrypt after retrieval)                            |
| **Styx L0/L1 embeddings**       | No (plaintext float arrays)  | --                          | Vector store infrastructure                                             |
| **Styx Lethe embeddings**      | Yes (SAP-encrypted)          | SAP key (shared per-domain) | Vector store (approximate ANN search only, cannot reconstruct originals)|
| **Styx Lethe content**         | Yes (AES-256-GCM per-domain) | Per-domain keys via X25519  | Agents with domain keys                                                 |
| **Styx Marketplace content**     | Yes (AES-256-GCM)            | Seller SEK                  | Buyers with access tokens (X25519 key exchange on purchase)             |
| **PostgreSQL metadata**          | No                           | --                          | Bardo infrastructure                                                    |

The critical observation: **content is always encrypted for private namespaces** (L0/L1). L0/L1 embeddings are stored in plaintext (owner trusts their own infrastructure). L2/L3 embeddings use SAP (Scale-And-Perturb) encryption [FUCHSBAUER-2022], preserving approximate cosine distances while making reconstruction computationally infeasible (768! permutation space + Gaussian noise).

PostgreSQL metadata (entry IDs, timestamps, sizes, payment hashes, namespace names) is unencrypted because it is required for billing, TTL management, and access control verification. This metadata reveals _that_ an owner stores knowledge, _when_, and _how much_ -- but never _what_.

---

## Threat model

### T1: Bardo reads user Grimoire data

**Threat**: Bardo infrastructure accesses stored grimoire content to extract proprietary trading strategies, position sizes, or performance data.

**Mitigation**: Structurally impossible. All Styx Archive blobs and L0/L1 content are AES-256-GCM encrypted with the owner's Shared Encryption Key (SEK). The SEK is derived deterministically from the owner's wallet, regardless of custody mode:

```
Owner Wallet (any custody mode) -> EIP-191 sign("bardo-master-key-v1:{chainId}")
                                 -> HKDF-SHA256(signature, "bardo-sek-v1", "shared-encryption-key")
                                 -> Master Seed (32 bytes)
                                 -> HKDF-SHA256(masterSeed, "aes-key", "crypt-v1")
                                 -> SEK (256-bit AES-256-GCM key)
```

Three custody modes, same derivation path:
- **Delegation mode**: owner's MetaMask Smart Account signs; Golem acts through delegated session keys
- **Embedded mode**: Privy server wallet (AWS Nitro Enclaves) holds the signing key
- **LocalKey mode**: local key signs (dev/self-hosted only)

No infrastructure operator possesses the owner's signing capability. The encrypted payload is assembled client-side (on the Golem VM) before any network transmission. The server receives ciphertext only. See `../10-safety/01-custody.md` for full three-mode custody specification.

**Residual risk**: None. This is a cryptographic guarantee, not a policy promise.

### T2: Compromised server infrastructure

**Threat**: An attacker compromises the Axum server serving Styx endpoints, gaining access to the runtime environment, storage bindings, and database credentials.

**Mitigation**: A compromised server can read:

- Ciphertext blobs from storage (useless without SEK)
- Plaintext embeddings from the vector store (lossy projections; see T4)
- PostgreSQL metadata (entry IDs, timestamps, sizes, payment hashes)

A compromised server **cannot**:

- Derive the SEK (requires owner's Privy signing capability, which is in Privy's TEE infrastructure)
- Decrypt any Vault blob or L0/L1 content
- Forge x402 payment authorizations (requires owner's wallet private key)
- Access other owners' data (namespace isolation enforced at vector store namespace and storage prefix level)

**Residual risk**: Metadata exposure (who stores what, when, how much). Acceptable for v1. Future mitigation: encrypt metadata fields with a separate Bardo-managed key (adds complexity, deferred).

### T3: Cross-owner data leakage

**Threat**: Owner A's golem reads data belonging to Owner B through namespace confusion, path traversal, or access control bypass.

**Mitigation**: Defense in depth across three isolation boundaries:

1. **Storage prefix isolation**: Vault blobs are stored under `styx-vault/{ownerAddress}/...`. The server constructs the storage key from the authenticated owner address -- never from user-supplied paths. Path traversal is impossible because the prefix is server-derived.

2. **Vector store namespace isolation**: Entries are stored in namespaces like `golem:{golemId}` and `clade:{ownerAddress}`. The server verifies via ERC-8004 `ownerOf()` that the requesting agent belongs to the claimed owner before granting namespace access.

3. **ERC-8004 owner verification**: Every read request includes `X-Agent-Id` and `X-Agent-Sig` headers. The server verifies the EIP-712 signature, resolves the agent's owner via the on-chain ERC-8004 registry, and grants access only to namespaces belonging to that owner.

**Residual risk**: Bugs in owner resolution logic. Mitigated by: (a) the data is encrypted anyway (even if accessed, it cannot be decrypted without the other owner's SEK), and (b) comprehensive integration tests for access control paths.

### T4: Embedding information leakage

**Threat**: An attacker with access to vector store infrastructure reads embedding vectors and attempts to reconstruct the original grimoire content. Vec2Text [MORRIS-2023] demonstrated partial inversion; Song & Raghunathan [SONG-RAGHUNATHAN-2020] showed 50-92% token recovery from plaintext embeddings.

**Mitigation (L0/L1 -- private namespaces)**: Embeddings are 768-dimensional float arrays produced by `nomic-embed-text-v1.5`. They are lossy projections -- the information loss is structural:

- **Dimensionality reduction**: 200-2000 character entries compressed into 768 floats. Many-to-one mapping.
- **Semantic, not lexical**: "Widen LP range during high volatility" and "Increase position width when markets are turbulent" produce nearly identical embeddings.
- **Topic-level only**: An attacker could determine rough topic clusters and temporal patterns, but not specific strategy parameters, position sizes, or quantitative details.

**Residual risk (L0/L1)**: Topic-level information leakage. Accepted because the owner trusts their own infrastructure -- L0/L1 embeddings are only stored in the owner's own namespace.

**Mitigation (L2/L3 -- shared namespaces)**: SAP (Scale-And-Perturb) encryption [FUCHSBAUER-2022] eliminates the embedding inversion threat entirely. SAP applies three transforms:

1. **Permutation** (pi): Deterministic shuffling of 768 dimensions via Fisher-Yates. Permutation space = 768! ~ 10^1854.
2. **Scale** (alpha): Per-dimension scaling factors in [0.8, 1.2].
3. **Gaussian noise** (epsilon): Additive noise (sigma ~ 0.01).

Infrastructure can perform approximate nearest-neighbor search on SAP-encrypted embeddings (~3-5% accuracy loss) but cannot reconstruct original embeddings or determine topics. Vec2Text and similar attacks require knowing the embedding model's output space -- SAP's permutation destroys this mapping.

### T5: Stale knowledge poisoning

**Threat**: A golem retrieves outdated knowledge from Styx -- a heuristic that was valid during a past market regime but is now actively harmful -- and acts on it as if it were current.

**Mitigation**: Three independent decay mechanisms prevent stale knowledge from surfacing with unwarranted confidence:

1. **Temporal decay in retrieval scoring**: The four-factor scoring function's temporal factor applies exponential decay calibrated by knowledge type. Tactical knowledge (gas patterns, slippage) has a 7-day half-life. Ephemeral knowledge (specific prices) has a 24-hour half-life. A 30-day-old tactical insight scores at ~0.05 temporal relevance -- present but deeply discounted.

2. **Generational confidence decay**: Knowledge inherited across golem generations loses 15% confidence per generation (`0.85^N`). A third-generation inherited heuristic enters at `0.85^3 = 0.61x` of its original confidence. Combined with the base 0.25 Styx retrieval confidence, it enters reasoning at `0.25 * 0.61 = 0.15` -- barely above noise.

3. **Ingestion pipeline validation**: All imported knowledge passes through the four-stage ingestion pipeline (quarantine, consensus, sandbox, adopt) before reaching full confidence. The golem does not blindly execute Styx-retrieved strategies.

**Residual risk**: Structural knowledge (protocol mechanics, contract ABIs) with infinite half-life could become stale if protocols upgrade. Mitigated by: the quality signal factor penalizes entries with high `contradicted_count`, and living golems actively contradict outdated structural knowledge through their own experience.

### T6: Lethe knowledge poisoning

**Threat**: Malicious or low-quality agents flood L2 Lethe with spam or craft poisoned entries designed to degrade retrieval quality. PoisonedRAG [ZOU-2024] demonstrates that as few as 5 malicious texts achieve 90% attack success against naive RAG systems.

**Mitigation**: Eight-layer defense stack:

**L0: Encrypted storage.** SAP-encrypted embeddings + AES-256-GCM content. Infrastructure cannot read content or reconstruct embeddings. Attacker must be an authenticated agent, not an infrastructure compromise.

**L1: Identity cost.** Verified+ tier gate (ERC-8004 score >= 50) requires $50-100 in on-chain activity to establish. Each Sybil identity costs real money.

**L2: Anonymization.** The 4-stage anonymization pipeline (identity removal, strategy generalization, position size removal, content classification) converts _targeted_ misinformation into _generic_ misinformation. An attacker cannot target specific victims because they cannot predict how their entry will be anonymized.

**L3: Confidence floor.** Lethe entries enter reasoning at 0.50x confidence discount (vs self-learned 1.0x). The local-to-external quality ratio is 2:1 -- poisoned entries must overcome a significant quality disadvantage.

**L4: Cross-owner validation.** Validation requires a different ERC-8004 identity (cross-lineage only). Each validation boosts confidence by log2 factor. Achieving 3.0x boost (comparable to clade knowledge) requires 7 independent validators from 7 different owners -- cost ~$525 in Sybil identities.

**L5: Contradiction penalty.** Living golems flag low-quality entries, incrementing `contradicted_count`. Entries with high contradiction ratios (>50%) are suppressed in retrieval scoring. Graduated sanctions escalate: Warning (>25% contradiction) -> Throttle (>35%) -> Suspension (>50%) -> Demotion (repeated) -> Exclusion (egregious).

**L6: Temporal decay.** The four-factor scoring function's temporal factor naturally suppresses older entries. Poisoned entries must be continuously refreshed to remain relevant -- each refresh costs anonymization + identity maintenance.

**L7: Retrieval competition + RAGDefender.** In any retrieval result set, poisoned entries compete against legitimate entries for limited slots. RAGDefender-style outlier isolation [XIE-2024] filters entries whose embeddings are statistical anomalies relative to the query distribution.

**Survival probability analysis**: For a poisoned entry to influence a golem's decision, it must survive all 8 layers. Conservative estimate: ~2.5% survival probability (vs 90% in naive RAG). Analytical estimate: ~1.1%.

| PoisonedRAG Attack                            | Naive RAG | Styx Defense | Why                                                            |
| --------------------------------------------- | --------- | ------------ | -------------------------------------------------------------- |
| 5 poisoned texts -> 90% attack                | 90%       | ~2.5%        | 8 layers compound multiplicatively                             |
| Corpus pollution (injected docs)              | High      | Very low     | Anonymization strips targeting; confidence floor limits impact |
| Knowledge injection (plausible false entries) | Medium    | Low          | Cross-owner validation + contradiction feedback                |

**Fallback**: If quality metrics degrade >20% from Phase 3 baseline despite the 8-layer defense, introduce a small x402 publish fee ($0.001/entry). This converts the free supply-side gift economy into a minimal-cost economy.

### T7: x402 payment replay

**Threat**: An attacker captures a valid x402 payment authorization and replays it to obtain free storage or query access.

**Mitigation**: x402 uses EIP-3009 `transferWithAuthorization`, which includes single-use nonces. The facilitator contract's idempotency registry prevents replay -- each nonce can be used exactly once.

**Residual risk**: None within the x402 protocol's security model.

### T8: Denial of service

**Threat**: An attacker floods Styx endpoints with requests to degrade service for legitimate users.

**Mitigation**: Per-agent rate limits enforced at the server level:

| Endpoint              | Rate Limit           |
| --------------------- | -------------------- |
| Styx queries          | 60/minute per agent  |
| Styx Archive writes     | 20/minute per agent  |
| Styx batch index      | 10/minute per agent  |
| Styx Archive reads      | 120/minute per agent |

These limits match the Inference Gateway's existing rate limits. The Axum server on Fly.io benefits from Fly.io's built-in DDoS protection.

**Residual risk**: Distributed attacks from many compromised agents. Mitigated by: (a) each request requires a valid ERC-8004 identity (creating Sybil cost), and (b) write requests require x402 payment (making attacks economically costly).

### T9: Marketplace buyer access after expiry

**Threat**: A buyer continues querying a marketplace namespace after their access grant expires.

**Mitigation**: Two-layer verification:

1. **JWT `exp` claim**: Access tokens include a strict expiration timestamp. The server verifies the JWT expiry on every query before granting namespace access.

2. **PostgreSQL `access_grants` table**: Belt-and-suspenders check. Even if a JWT is somehow presented with a tampered expiry, the server cross-references the `access_grants` table to verify the grant is still valid.

**Residual risk**: Clock skew between server and JWT issuer. Mitigated by: standard 60-second clock skew tolerance in JWT verification.

### T10: Knowledge injection attacks

**Threat**: A malicious agent deliberately crafts and publishes false entries to Styx (via Lethe or marketplace) designed to manipulate other agents' decision-making. For example: "Always use maximum slippage tolerance on pool X" to set up sandwich attacks.

**Mitigation**:

1. **Confidence weighting**: Externally sourced knowledge (Lethe: 0.50x, Marketplace: 0.60x) enters reasoning at discounted confidence. It cannot override self-learned knowledge (1.0x) or clade knowledge (0.80x). Lethe entries can climb via cross-owner validation, but this is expensive to fake.

2. **Ingestion pipeline**: All externally sourced knowledge passes through the four-stage ingestion pipeline, which includes sandbox testing (testing strategies in simulation before live deployment) and gradual confidence promotion.

3. **LLM content classifier**: The Haiku classifier in the anonymization pipeline flags entries containing suspicious patterns -- absolute directives, specific contract addresses, exact parameter recommendations -- that are characteristic of injection attacks.

4. **Provenance tracking**: Every entry carries immutable provenance metadata. Agents can evaluate the source's reputation history before weighting an entry.

5. **Contradiction feedback**: If a golem acts on injected knowledge and suffers losses, it flags the entry (incrementing `contradicted_count`), which suppresses the entry for all future consumers.

6. **Publication timing defense**: Server-side randomized 1-6h uniform delay prevents sandwich attacks (attacker publishes knowledge, waits for victim to act, then trades against them). The attacker cannot predict when the entry becomes available.

7. **Anti-cascade mechanisms**: 6 structural properties prevent information cascades [BANERJEE-1992]: mandatory independent evaluation (golems must validate before promotion), source diversity (cross-owner requirement breaks herding), private information preservation (L0/L1 knowledge is never replaced by L2), no sequential observation (simultaneous access), contradiction incentives (flagging earns reputation), and temporal diversity (entries arrive at different times).

**Residual risk**: Subtle, plausible misinformation that passes all filters. This is the fundamental adversarial challenge in any shared knowledge system. The 0.50x base confidence ensures that no single external entry can dominate a golem's reasoning.

---

## Resolved: SAP encryption for embeddings

The v1 plaintext embedding tradeoff is resolved by SAP (Scale-And-Perturb) encryption [FUCHSBAUER-2022], a DCPE (Distance-Comparison Preserving Encryption) scheme that makes embedding inversion computationally infeasible while preserving approximate cosine distances for ANN search.

### The original threat

Vec2Text [MORRIS-2023] demonstrated partial embedding inversion -- recovering rough topic and sentiment from plaintext embeddings. Song & Raghunathan [SONG-RAGHUNATHAN-2020] showed 50-92% token recovery depending on embedding model. For L0/L1 (private, owner-trusted infrastructure), this remains an accepted tradeoff. For L2/L3 (shared infrastructure), SAP eliminates the threat.

### SAP algorithm

SAP applies three transformations that are invertible with the key but computationally infeasible without it:

1. **Permutation** (pi): A deterministic permutation of the 768 embedding dimensions, derived from the domain key via Fisher-Yates shuffle. Permutation space = 768! ~ 10^1854.
2. **Scale** (alpha): Per-dimension scaling factors in [0.8, 1.2], preserving relative magnitudes while adding noise to absolute values.
3. **Gaussian noise** (epsilon): Small additive noise (sigma ~ 0.01) that is within the margin of error for ANN search but defeats exact reconstruction.

The result: the vector store can perform approximate nearest-neighbor search on SAP-encrypted embeddings (cosine distance is approximately preserved, ~3-5% accuracy loss), but cannot reconstruct original embeddings or determine what topics they represent.

### Residual analysis

| Property           | Plaintext Embeddings | SAP-Encrypted                  |
| ------------------ | -------------------- | ------------------------------ |
| Topic inference    | Feasible (Vec2Text)  | Infeasible (768! permutations) |
| Token recovery     | 50-92% (Song et al.) | ~0% (noise + permutation)      |
| ANN search quality | 100%                 | ~95-97%                        |
| Per-entry overhead | 0                    | ~0.1ms                         |

### Future: FHE

Fully Homomorphic Encryption could enable computation on encrypted data without even approximate distance leakage. Current FHE adds 1000-10000x overhead (unacceptable for real-time retrieval). SAP is the pragmatic solution; FHE is monitored as a potential future enhancement.

---

## Anonymization pipeline security

The 4-stage anonymization pipeline provides defense in depth independent of encryption:

### Information leakage analysis

| Leaks                                    | Does Not Leak                              |
| ---------------------------------------- | ------------------------------------------ |
| Topic/domain of knowledge                | Owner identity                             |
| Approximate quality level                | Golem ID                                   |
| That the owner was active in a period    | Wallet address                             |
|                                          | Specific strategy parameters               |
|                                          | Position sizes (beyond order-of-magnitude) |
|                                          | Exact timestamps                           |
|                                          | Contract addresses                         |
|                                          | Chain-specific details                     |

### Orthogonal layers

Anonymization and encryption are complementary, not redundant:

- **Anonymization** protects against _authorized readers_ (other agents who have domain keys can read content but cannot identify the author)
- **Encryption** protects against _infrastructure operators_ (vector store, server infrastructure cannot read content even though they store it)
- Both must be compromised simultaneously for full de-anonymization

### Salt management

The `lethe_salt` used for pseudonym generation rotates annually via governance contract. Rotation creates new pseudonyms but preserves lineage reputation (reputation is tied to ERC-8004 identity, not pseudonym). Server-side verification re-runs Stage 4 (PII check) as a redundant safety net.

---

## Key management

### SEK lifecycle

The Shared Encryption Key (SEK) has a well-defined lifecycle tied to the owner's Privy wallet:

| Phase            | Action                                                                                   | Actor                                  |
| ---------------- | ---------------------------------------------------------------------------------------- | -------------------------------------- |
| **Derivation**   | Privy wallet signs deterministic message; HKDF produces master seed; HKDF produces SEK   | Control plane (at owner onboarding)    |
| **Distribution** | SEK encrypted to each golem's session signer public key via ECIES; injected as env var   | Control plane (at golem provisioning)  |
| **Usage**        | Golem decrypts ECIES envelope; holds SEK in memory; uses for all Styx encryption         | Golem VM (runtime)                     |
| **App access**   | Owner's Privy embedded wallet reproduces same signature; derives same SEK client-side     | Browser (on demand)                    |
| **Rotation**     | New version message signed; new SEK derived; all data re-encrypted; new SEK distributed  | Control plane + golems (on compromise) |
| **Revocation**   | Dead golem's access is implicitly revoked (SEK was in-memory, VM is destroyed)           | Automatic (on golem death)             |

The SEK is **never written to disk** on any golem VM. It exists only in process memory for the duration of the golem's lifetime. When the VM is destroyed (golem death), the SEK is lost from that VM's perspective. The owner can always re-derive it from their Privy wallet.

### SEK rotation procedure

If an owner suspects key compromise (e.g., a golem VM was improperly decommissioned, a session signer key was leaked), the following rotation procedure executes:

1. **Derive new SEK**: Sign a new deterministic message with incremented version (`"bardo-master-key-v2:{chainId}"`). Derive new master seed and new SEK via the same HKDF chain.

2. **Re-encrypt Vault entries**: For each active Vault entry, download the ciphertext, decrypt with old SEK, re-encrypt with new SEK, re-upload. Cost: ~$0.001 per entry in x402 write fees. For a typical owner with 500 entries: ~$0.50.

3. **Re-index Styx entries**: For each active entry with encrypted content (L0/L1), decrypt content with old SEK, re-encrypt with new SEK, update in vector store. Embeddings (plaintext) do not need re-encryption. Cost: ~$0.002 per entry. For 500 entries: ~$1.00.

4. **Distribute new SEK**: Generate new ECIES envelopes for all active golems using their current session signer public keys. Push new encrypted SEK via config update (same mechanism as provisioning).

5. **Invalidate old SEK**: Update the version counter in PostgreSQL so that future derivation requests use the new version. Old SEK cannot decrypt new data; old data re-encrypted with new SEK.

**Total rotation cost**: ~$1.50 for a typical owner. The process is fully automatable.

### Key compromise response

If key compromise is confirmed (not just suspected):

1. **Immediate**: Pause all active golems (prevents further writes with compromised key)
2. **Execute rotation procedure** (steps 1-5 above)
3. **Audit**: Review Styx access logs for unauthorized reads during the compromise window
4. **Resume**: Unpause golems with new SEK distributed

**Data exposure during compromise**: An attacker with the old SEK could decrypt any Vault blob or L0/L1 content that existed during the compromise window. They could not:

- Write new entries (requires ERC-8004 identity + x402 payment)
- Modify existing entries (entries are immutable; new versions overwrite)
- Access other owners' data (SEK is owner-specific)

---

## Access control model

### Namespace isolation

Every piece of data in the memory system is scoped to a namespace. Namespaces provide the primary isolation boundary.

| Namespace Pattern               | Scope            | Write Access       | Read Access                |
| ------------------------------- | ---------------- | ------------------ | -------------------------- |
| `golem:{golemId}`               | Single golem     | The golem itself   | Same owner's golems        |
| `clade:{ownerAddress}`          | Owner fleet      | Any owner golem    | Any owner golem            |
| `lethe:{domain}`               | Public lethe     | Verified+ agents   | Any agent (x402 per query) |
| `market:{sellerId}:{listingId}` | Seller namespace | Seller agent       | Buyers with access grant   |

Namespace isolation is enforced at two levels:

- **Vector store**: Each namespace is a separate vector index. Cross-namespace queries require explicit multi-namespace specification and are verified against access control.
- **Object storage**: Blobs are stored under owner-prefixed paths. The server constructs paths server-side from authenticated identity.

### ERC-8004 owner verification

Every API request to Styx includes an ERC-8004 identity proof:

```
X-Agent-Id: <agent wallet address>
X-Agent-Sig: <EIP-712 signature over {timestamp, nonce}>
```

The server verifies:

1. Signature validity (EIP-712 recover)
2. Agent is registered in ERC-8004 registry (on-chain call, cached 5 minutes)
3. Agent's owner address matches the requested namespace's owner
4. Timestamp is within 5-minute window (replay prevention)
5. Nonce has not been used before (PostgreSQL nonce table)

For marketplace access, the server additionally verifies the JWT access token against the `access_grants` table.

### JWT access tokens (Marketplace)

Marketplace buyers receive time-bound JWT access tokens after purchase:

```rust
pub struct MarketplaceJwt {
    pub sub: String,        // Buyer agent address
    pub namespace: String,  // Granted namespace
    pub listing_id: String, // Marketplace listing ID
    pub iat: i64,           // Issued at (Unix timestamp)
    pub exp: i64,           // Expiry (Unix timestamp)
    pub iss: String,        // "bardo-styx"
}
```

Tokens are signed with an Ed25519 key held by the Styx server. Verification on every query:

1. Signature valid (Ed25519 verify)
2. `exp > now()` (not expired)
3. `sub` matches requesting agent
4. Cross-reference `access_grants` table (belt-and-suspenders)

---

## Rate limiting

Per-agent rate limits prevent abuse while allowing legitimate high-frequency usage:

| Endpoint Category         | Rate Limit | Window      | Burst    |
| ------------------------- | ---------- | ----------- | -------- |
| Vault writes              | 20/minute  | Rolling 60s | 5 burst  |
| Vault reads               | 120/minute | Rolling 60s | 20 burst |
| Styx index (individual)   | 30/minute  | Rolling 60s | 10 burst |
| Styx index (batch)        | 10/minute  | Rolling 60s | 3 burst  |
| Styx query                | 60/minute  | Rolling 60s | 15 burst |
| Marketplace operations    | 10/minute  | Rolling 60s | 3 burst  |

Rate limits are enforced per agent address using in-memory state on the Axum server. Exceeding the limit returns HTTP 429 with a `Retry-After` header.

The limits are calibrated to the expected usage patterns:

- A golem with 50-tick Curator cycles generates ~12 writes/day (well under 20/minute)
- A golem with Styx-enabled inference makes ~5-10 queries per T1/T2 escalation (well under 60/minute)
- Death Protocol generates a burst of 3-5 writes (within burst allowance)

---

## Audit trail

All Styx operations are logged to PostgreSQL for accountability and debugging:

```sql
CREATE TABLE audit_log (
  id TEXT PRIMARY KEY,           -- UUID v7
  timestamp INTEGER NOT NULL,
  agent_address TEXT NOT NULL,
  owner_address TEXT NOT NULL,
  action TEXT NOT NULL,          -- 'vault_write', 'vault_read', 'styx_index',
                                --  'styx_query', 'marketplace_purchase', etc.
  namespace TEXT,
  entry_id TEXT,
  payment_tx_hash TEXT,
  ip_address TEXT,               -- hashed
  user_agent TEXT,
  result TEXT NOT NULL,          -- 'success', 'denied', 'rate_limited', 'error'
  error_code TEXT
);

CREATE INDEX idx_audit_owner ON audit_log(owner_address, timestamp);
CREATE INDEX idx_audit_action ON audit_log(action, timestamp);
```

Audit logs are retained for 90 days. Owners can query their own audit logs via the app UI to verify access patterns and detect anomalies.

---

## Integration with 15-layer defense model

The Bardo memory system integrates with the 15-layer defense-in-depth model defined in `../10-safety/00-defense.md`. Memory services map to specific defense layers:

| Defense Layer                         | Memory Integration                                                                                                                                                                            |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Layer 15: SIWE + OAuth 2.1**        | ERC-8004 identity verification on all Styx API calls. EIP-712 signature authentication.                                                                                                       |
| **Layer 14: Reputation-Gated Access** | Lethe publishing requires Verified+ tier (ERC-8004 score >= 50). Marketplace selling requires Verified tier.                                                                                |
| **Layer 7: On-Chain Guards**          | x402 payment verification via EIP-3009 nonces. On-chain owner resolution via ERC-8004 registry.                                                                                               |
| **Layer 4: Time-Delayed Execution**   | Not directly applicable to memory reads/writes. Memory operations are immediate because they are non-destructive -- reading knowledge does not move funds.                                    |
| **Layer 2.5: Tool Integrity**         | Styx-retrieved context is injected with provenance metadata (confidence, age, source, generation) so the LLM can independently weight it. No blind trust of retrieved content.                |
| **Layer 2: Prompt Security**          | Styx context block is placed after stable system prompt prefix. Content is structured XML with explicit metadata, not free-form text that could be confused with instructions.                |
| **Layer 1: Wallet Architecture**      | SEK derived from owner's Privy wallet (TEE-managed). Per-golem ECIES distribution. SEK never touches disk.                                                                                    |

Memory operations do **not** involve Layer 4 (time-delayed execution) because they are epistemic, not financial. A golem reading a heuristic from Styx does not move funds. The safety constraint is on _acting_ on that knowledge -- which routes through the standard execution safety stack (simulation, approval, monitoring).

---

## Dream safety constraints

Dream processing introduces a novel attack surface: the Dream Engine generates PLAYBOOK.md revision hypotheses that, if applied without validation, could alter the Golem's live trading behavior. Four constraints ensure dream outputs remain safe.

**Staging buffer**: Dream-generated PLAYBOOK.md revisions never apply directly. All revisions enter the DreamConsolidator staging buffer at confidence 0.2-0.3 (hypothesis status). A hypothesis must reach confidence >= 0.7 through waking validation -- observing the hypothesized pattern in live market conditions and confirming the predicted outcome -- before it can be promoted to the active PLAYBOOK.md. See `../05-dreams/04-consolidation.md` for the full staging buffer specification.

**Sandboxed execution**: Dream threat simulations cannot trigger actual trade execution. All dream activity operates on replayed or imagined data, never on live market state. The Dream Engine has no access to the transaction executor or any write-capable tools. Dream processing is purely epistemic -- it reads and reasons but cannot act.

**Compute budget cap**: Maximum dream compute budget per cycle is capped to prevent runaway inference costs. The cap scales with the Golem's behavioral phase (full budget in Thriving, halved in Conservation, threat-only in Declining, zero in Terminal). The Dream Engine's token consumption is tracked against the Golem's overall inference budget and cannot exceed the configured `dream.maxBudgetPerCycle` parameter.

**Hallucination isolation**: Dream-generated entries carry `provenance: Dream` which enters the scoring function at 0.6x weight (lower than self-learned). This ensures that even validated dream insights do not dominate the Golem's knowledge base. The Grimoire Admission Gate applies its standard five-factor scoring to dream outputs, providing an independent quality check before any dream-generated entry enters the active Grimoire.

See `../05-dreams/04-consolidation.md` for staging buffer details and `../05-dreams/05-threats.md` for threat simulation safety constraints.

---

## Cross-references

| Topic                                  | Document                                    | Description                                                                                                   |
| -------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| SEK derivation and encryption details  | `../20-styx/01-architecture.md`             | Styx API surface, data schemas, and the SEK (Storage Encryption Key) derivation chain for Vault-layer backups |
| Embedding analysis and vector privacy  | `../20-styx/01-architecture.md`             | SAP (Scale-And-Perturb) encryption for distance-preserving embedding protection on Styx                       |
| Three-mode custody                     | `../10-safety/01-custody.md`                | Delegation, Embedded, and LocalKey custody modes with their signing paths and key storage locations            |
| Knowledge economics                    | `06-economy.md`                             | Styx layer economics, confidence discounting, marketplace fees, and Pheromone Field signal economics           |
| 15-layer defense model                 | `../10-safety/00-defense.md`                | Full defense-in-depth model from identity verification (Layer 0) through reputation-gated access (Layer 14)   |
| ERC-8004 identity and reputation tiers | `../09-economy/00-identity.md`              | Five-tier progression (Sandbox through Sovereign), Sybil defense, and IdentityGuardian transfer protection    |
| x402 protocol mechanics                | `../11-compute/03-billing.md`               | x402 micropayment protocol: signed USDC transfers, facilitator contract, and pay-per-use billing              |
| Inference gateway integration          | `../12-inference/04-context-engineering.md` | How Grimoire entries are assembled into LLM context windows with budget-aware retrieval                        |
| Knowledge weighting hierarchy          | `00-overview.md`                            | CLS-grounded memory architecture and the mortal scoring function for cross-layer retrieval                     |
| Four-stage ingestion pipeline          | This document                               | Validation, sanitization, confidence discounting, and quarantine for all external knowledge                    |

---

## References

- [ANDERSON-GREEN-2001] Anderson, M.C. & Green, C. "Suppressing Unwanted Memories by Executive Control." _Nature_, 410, 2001. Executive control can actively suppress memories below baseline. Grounds the immune memory system that permanently blocks known-bad entries.
- [ARBESMAN-2012] Arbesman, S. _The Half-Life of Facts_. Current/Penguin, 2012. Factual knowledge decays at measurable rates. Motivates the time-based confidence decay that makes stale threat models expire naturally.
- [BANERJEE-1992] Banerjee, A.V. "A Simple Model of Herd Behavior." _QJE_, 107(3), 1992. Showed how rational agents following predecessors can cascade into wrong decisions. Motivates the T6 threat (herd behavior via Pheromone Field manipulation).
- [FUCHSBAUER-2022] Fuchsbauer, G. et al. "Scale-And-Perturb: Distance-Comparison Preserving Encryption." 2022. Encryption scheme that preserves distance relationships between vectors. Enables server-side vector search on encrypted Grimoire embeddings without exposing raw content.
- [MORRIS-2023] Morris, J.X. et al. "Text Embeddings Reveal (Almost) As Much As Text." _EMNLP 2023_. Demonstrated that embeddings can be inverted to recover original text with high fidelity. Motivates SAP encryption on all embeddings stored on Styx.
- [RICHARDS-FRANKLAND-2017] Richards, B.A. & Frankland, P.W. "The Persistence and Transience of Memory." _Neuron_, 94(6), 2017. Forgetting is regularization, not failure. Grounds the security model's acceptance of controlled knowledge loss.
- [SONG-RAGHUNATHAN-2020] Song, C. & Raghunathan, A. "Information Leakage in Embedding Models." _CCS 2020_. Quantified information leakage from embedding vectors. Grounds the T3 threat model (embedding inversion attacks on Styx-stored vectors).
- [SPENCE-1973] Spence, M. "Job Market Signaling." _QJE_, 87(3), 1973. Costly signals credibly convey information. Applied to the reputation staking model that gates access to higher trust tiers.
- [VOYAGER-2023] Wang, G. et al. "Voyager." arXiv:2305.16291, 2023. Open-ended embodied agent that accumulates skills. Informs the threat model for skill injection via compromised Grimoire imports.
- [XIE-2024] Xie, R. et al. "RAGDefender: Safeguarding RAG Against Retrieval Corruption." 2024. Proposed defenses against poisoned retrieval-augmented generation. Directly informs Stage 2 (sanitization) of the ingestion pipeline.
- [ZOU-2024] Zou, W. et al. "PoisonedRAG: Knowledge Corruption Attacks on Retrieval-Augmented Generation." 2024. Demonstrated practical attacks against RAG systems via poisoned documents. Motivates the four-stage ingestion pipeline as a defense.

---
