# Knowledge Ingestion Safety [SPEC]

> **Crate**: `golem-grimoire` (`ingestion` module)
>
> **Depends on**: [00-defense.md](00-defense.md) (DeFi Constitution as last-resort defense), [02-policy.md](02-policy.md) (PolicyCage constraints)

> **Reader orientation:** This document specifies how a Golem (mortal autonomous DeFi agent) safely ingests external knowledge into its Grimoire (persistent knowledge store). It belongs to the **Safety** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: all external knowledge, whether from marketplace purchases, Clade siblings, or Grimoire archives, passes through a four-stage immune system (quarantine, consensus validation, sandbox, adopt) before it can influence the agent's reasoning. Terms like PolicyCage, Styx, and Heartbeat are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

The Bardo runtime treats all external knowledge as potentially adversarial. Whether purchased from the marketplace, shared by a Clade sibling, or imported from a Grimoire archive -- every piece of external content passes through a multi-stage immune system before it can influence reasoning. No exceptions. No trusted bypass. Even intra-Clade content from high-confidence members enters at Stage 1.

Simple regex + LLM validation pipelines are explicitly bypassed by every major attack in the literature. AgentPoison's triggers are indistinguishable from benign content. LLM-based content auditing in isolation misses 66% of poisoned entries [ZHANG-2024]. The architecture below replaces naive validation with layered defenses drawn from A-MemGuard, TrustRAG, and RobustRAG.

### Bloom Oracle: Privacy-Preserving Knowledge Validation

Before an entry reaches the full pipeline, a Bloom filter oracle provides a fast, privacy-preserving check: has any Golem in the network already flagged this content hash as poisonous? The filter produces false positives (conservative) but never false negatives. A hit immediately quarantines the entry. A miss proceeds to Stage 1. The oracle is distributed -- each Golem maintains a local filter seeded from Clade peers and Styx lethe. No individual knowledge content is shared, only cryptographic hashes of known-bad entries.

### Immune Memory

The ingestion pipeline learns from its own history. When an entry is rejected or rolled back, a compact "immune signature" (a 256-bit hash of the entry's embedding centroid plus its rejection reason) is stored locally. Future entries that match a stored immune signature skip directly to quarantine with a `previously_rejected_pattern` flag. This is the computational equivalent of immunological memory: the system remembers what made it sick.

---

## 1. The Poisoning Problem

Three documented attack classes make Grimoire poisoning a first-order threat:

| Attack Class                       | Mechanism                                               | Success Rate                  | Defense Layer                                   |
| ---------------------------------- | ------------------------------------------------------- | ----------------------------- | ----------------------------------------------- |
| **AgentPoison** [CHEN-2024]        | Optimized embedding-space triggers hijack RAG retrieval | >=80% with <0.1% poison rate  | Stage 2, Layer 1 (TrustRAG anomaly detection)   |
| **MINJA** [DONG-2025]              | Injection through normal interactions                   | >95% injection success        | Stage 2, Layer 2 (A-MemGuard consensus)         |
| **MemoryGraft** [MEMORYGRAFT-2025] | Durable, trigger-free behavioral drift                  | No discrete trigger to detect | Stage 4 (causal rollback) + dual memory lessons |

No single technique defends against all three. AgentPoison defeats embedding-only detection. MINJA defeats LLM-only auditing. MemoryGraft defeats both in isolation. The pipeline uses independent defense layers so that each attack class is stopped by at least one layer it cannot evade.

### 1.1 Extended Threat Landscape

| Attack Class                      | Mechanism                                                  | Defense Layer                            |
| --------------------------------- | ---------------------------------------------------------- | ---------------------------------------- |
| Embedding-space trigger injection | Poisoned entries contain optimized trigger phrases         | Stage 2, Layer 1 (TrustRAG)              |
| Reasoning path manipulation       | Entries appear benign alone but shift reasoning in context | Stage 2, Layer 2 (A-MemGuard)            |
| Verifiable claim falsification    | False on-chain data claims (fake TVL, fabricated prices)   | Stage 2, Layer 3 (on-chain verification) |
| Slow behavioral drift             | Gradual accumulation of subtly biased entries              | Stage 4 (causal rollback)                |
| Cross-entry contradiction seeding | Individually valid entries that collectively contradict    | Stage 2 batch validation                 |

---

## 2. Four-Stage Pipeline

```
Stage 1: QUARANTINE         Cryptographic provenance. EIP-712 signatures.
    |                       Unsigned entries auto-rejected.
    v
Stage 2: CONSENSUS          Layer 1: TrustRAG embedding-space anomaly detection.
    VALIDATION              Layer 2: A-MemGuard consensus-based divergence.
    (2-of-3 must pass)      Layer 3: On-chain verification for verifiable claims.
    |                       Layer 3b: Mental model consistency check.
    v
Stage 3: SKILL SANDBOX      Voyager-style: decompose heuristic into structured IR.
    (Voyager pattern)       Retrieve verified components. Generate code for novel
    |                       parts only. Execute in isolated sandbox.
    v
Stage 4: ADOPT              Low initial confidence. Provenance preserved.
    + DUAL MEMORY           Failures stored as "lessons" consulted before every action.
```

Every external entry traverses all applicable stages in order. There is no shortcut.

---

## 3. Stage 1: Quarantine with Cryptographic Provenance

All external entries land in a separate vector store collection (`quarantine_*`), fully isolated from the agent's active Grimoire. Entries remain quarantined until explicitly adopted or rejected -- no automatic timeout promotion.

```rust
use alloy::primitives::{Address, B256};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryType {
    Insight,
    Heuristic,
    Warning,
    StrategyFragment,
    CausalLink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provenance {
    Purchased,
    Clade,
    CrossUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySource {
    pub provenance: Provenance,
    pub agent_id: Address,
    pub signature: B256,          // EIP-712 over content hash
    pub on_chain_anchor: B256,    // Transaction hash
    pub listing_id: Option<String>,
    pub clade_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pending,
    Passed,
    Failed,
}

/// A quarantined entry awaiting validation.
/// Entries remain quarantined until explicitly adopted or rejected —
/// no automatic timeout promotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedEntry {
    pub id: String,
    pub content: String,
    pub entry_type: EntryType,
    pub source: EntrySource,
    pub quarantined_at: DateTime<Utc>,
    pub validation_status: ValidationStatus,
    pub validation_details: Option<ValidationResult>,
    pub batch_id: Option<String>,
}
```

Cryptographic provenance creates non-repudiable authorship (MemoryGraft's recommended defense). If an entry is later found poisonous, the source is identified and their reputation slashed. Unsigned entries are automatically rejected.

Quarantine store capacity: 1,000 entries per agent. Oldest `pending` entries evicted FIFO when full. Passed/failed entries retained for 7 days for audit trail, then pruned.

---

## 4. Stage 2: Consensus Validation

Multi-layer defense replacing naive single-LLM validation. Default consensus threshold: 2-of-3 validators must pass.

### 4.1 Layer 1: TrustRAG Embedding-Space Anomaly Detection

Training-free. Applies k-means clustering in embedding space to incoming entries. Entries whose embeddings cluster far from the existing Grimoire's distribution are flagged. Reduces attack success by ~76% with negligible latency [ZHOU-2025].

Adaptive k: `k = min(5, floor(existingEntries.length / 10))`. Below 50 entries, k-means produces degenerate clusters and the anomaly score becomes meaningless. Below 50 entries, Layer 1 is skipped and the system relies on Layer 2, which works with any Grimoire size.

### 4.2 Layer 2: A-MemGuard Consensus-Based Validation

For each incoming entry, retrieve K related memories from the existing Grimoire and generate parallel reasoning paths. One path uses existing knowledge alone. The other includes the new entry. If adding the entry causes significant reasoning divergence, it is flagged [WEI-2025]. Achieves >95% reduction in successful poisoning.

This catches poisoned entries that appear benign in isolation but produce harmful reasoning in context -- the exact failure mode that isolated LLM auditing misses 66% of the time.

**LLM integrity assumption**: Layer 2 assumes the LLM inference provider is not compromised. If both reasoning paths run through a compromised provider, divergence is zero and the poisoned entry passes. Layer 3 (on-chain verification) provides a deterministic backstop. For elevated inference compromise risk, route baseline and augmented reasoning through different providers.

### 4.3 Layer 3: On-Chain Verification (Deterministic)

For verifiable claims (e.g., "pool X had TVL of Y at block Z"), check against chain state via RPC. Provably false claims are rejected immediately -- no consensus needed.

### 4.4 Layer 3b: Mental Model Consistency Check

For heuristics and causal claims, retrieve relevant mental models and check consistency. An entry claiming "higher volatility always increases LP returns" contradicts established theory. This is not an automatic rejection -- models can be wrong for novel conditions -- but it raises the divergence score and triggers additional scrutiny.

---

## 5. Stage 3: Skill Sandbox (Voyager Pattern)

For actionable heuristics -- entries that suggest specific trading behaviors -- direct NL-to-simulation translation is unreliable. LLM code generation drops to 62.5% on class-level code [LIU-2024]. The Voyager architecture [WANG-2023] provides the correct pattern: an embedding-indexed library of verified skill components that compose into strategies.

### 5.1 Sandbox Execution Flow

1. **Decompose**: NL heuristic into structured intermediate representation (conditions, actions, parameters)
2. **Retrieve**: Matching verified components from skill library
3. **Generate**: Code only for novel components not in library (>80% coverage = compose only)
4. **Execute**: In isolated sandbox (gVisor, no network, resource-limited)
5. **Evaluate**: Compare baseline vs. test outcomes
6. **Promote**: If successful, add novel components to skill library

### 5.2 Sandbox Resource Limits

| Resource            | Limit                 | Rationale                                     |
| ------------------- | --------------------- | --------------------------------------------- |
| Wall-clock time     | 30 seconds            | Sufficient for decompose + compose + simulate |
| LLM inference calls | 10 max                | Caps cost at ~$0.05                           |
| Gas simulation cost | $0.10 equivalent      | Prevents gas-bombing                          |
| Memory              | 256 MB                | Prevents memory exhaustion                    |
| CPU                 | 1 vCPU                | Prevents CPU starvation of host               |
| Disk                | 100 MB                | Prevents disk-fill attacks                    |
| Processes           | 10 max                | Prevents fork bombs                           |
| Network             | None (fully isolated) | Prevents data exfiltration                    |

Any limit breach terminates the sandbox immediately. The entry is rejected with reason `sandbox_resource_exceeded` and a lesson is stored.

### 5.3 Sandbox Applicability

| Entry Type          | Sandbox Required | Rationale                                          |
| ------------------- | ---------------- | -------------------------------------------------- |
| `heuristic`         | Yes              | Directly suggests trading/LP actions               |
| `strategy_fragment` | Yes              | Contains executable parameters                     |
| `insight`           | No               | Informational only                                 |
| `warning`           | No               | Risk signal -- validated by consensus              |
| `causal_link`       | No               | Validated by on-chain verification + mental models |

---

## 6. Stage 4: Adopt with Low Confidence

Entries passing all applicable stages enter the active Grimoire at low initial confidence. Confidence rises only with independent confirming evidence. Unconfirmed entries are eventually pruned by the Curator's decay cycle.

### 6.1 Initial Confidence by Source

| Source       | Validation Strictness     | Initial Confidence | TTL      |
| ------------ | ------------------------- | ------------------ | -------- |
| `self`       | No quarantine             | 0.5--0.9           | Standard |
| `clade`      | Standard consensus        | 0.2                | 14 days  |
| `purchased`  | Full validation + sandbox | 0.2                | 14 days  |
| `cross_user` | Full validation + sandbox | 0.1                | 7 days   |

### 6.1b Cross-Source Confidence Discounting

When the same claim appears from multiple sources, naive aggregation inflates confidence. If three Clade members all learned the same heuristic from the same marketplace seller, treating them as three independent confirmations overstates the evidence. The ingestion pipeline applies discounting:

- **Unique provenance**: Full confidence credit per source.
- **Shared upstream**: If two entries trace to the same `on_chain_anchor` (purchase transaction), the second confirmation adds only 50% of its normal confidence boost.
- **Identical content hash**: Duplicate entries from different agents add zero confidence. The first is credited; subsequent duplicates are logged but ignored for confidence purposes.

### 6.2 Dual Memory Architecture

Adopted external knowledge is stored in a separate memory partition from self-generated knowledge:

1. **Shorter TTL**: External entries decay faster. Entries not independently confirmed within their TTL window are pruned.
2. **Provenance tagging**: Every external entry retains full source metadata permanently, enabling causal attribution during rollback.
3. **Separate retrieval weighting**: External entries receive a 0.8x confidence penalty compared to self-generated entries of equal confidence.
4. **A/B decision tracking**: Every decision is tagged with which external entries (if any) influenced it. This powers the causal rollback pipeline.

The lessons store (failures from validation) has its own partition: max 500 entries, LRU eviction. Lessons not matched by any query within 30 days have confidence decayed by 0.1 per period. Lessons matched frequently (>10 matches in 7 days) are promoted to PLAYBOOK heuristics.

---

## 7. Causal Rollback

After ingesting external knowledge, performance degradation must be causally attributed before triggering rollback. Market regime changes routinely cause Sharpe drops independent of knowledge changes. Blindly rolling back after any performance dip would reject good knowledge during bear markets.

CausalImpact [BRODERSEN-2015] uses Bayesian structural time-series models to estimate the counterfactual. Requires >= 7 days of pre-ingestion baseline data. For agents with shorter histories, fall back to peer comparison.

### 7.1 Three-Check Rollback Pipeline

1. **Factor decomposition** [BRINSON-1986]: What fraction of the Sharpe drop is explained by market factors (ETH beta, TVL index, gas price, volatility)? If >70% is market-explained, do not rollback.
2. **Residual analysis**: After removing market factors, is the residual Sharpe drop significant? If |residualSharpeDelta| < 0.3, do not rollback.
3. **Source attribution**: Were decisions influenced by entries from this source recently? If the source was never used in decisions, do not rollback.

All three checks must point to knowledge-driven degradation before rollback triggers.

### 7.2 Rollback Execution

When rollback is triggered:

1. **Quarantine**: All entries from the offending source moved back to quarantine with status `rolled_back`
2. **Lesson creation**: Lesson stored in dual memory summarizing rollback context and causal evidence
3. **PLAYBOOK revert**: Heuristics derived from rolled-back entries are reverted to pre-ingestion state
4. **Reputation feedback**: Rollback event reported to ingestion metrics, feeding into the seller's safety signal in the Beta reputation system
5. **Confidence cascade**: Other entries from the same source have confidence reduced by 0.2 (guilt by association, recoverable with independent confirmation)

---

## 8. Batch Validation for Bulk Purchases

When purchasing multiple Grimoire entries from the marketplace, individual validation is both expensive and misses cross-entry contradictions. Batch validation validates entries as a group.

### 8.1 Batch Pipeline

1. **Cluster**: Cluster incoming entries by embedding similarity (shared embedding pass)
2. **Representative validation**: Run full A-MemGuard consensus on 1-2 representatives per cluster
3. **Cross-entry contradiction check**: Detect contradictions between clusters
4. **Selective sandbox**: Only heuristic + strategy_fragment entries proceed to Stage 3

### 8.2 Cost Reduction

| Validation Mode        | Per-Entry Cost | 100-Entry Batch Cost | Savings       |
| ---------------------- | -------------- | -------------------- | ------------- |
| Individual             | ~$0.05         | ~$5.00               | Baseline      |
| Batch (representative) | ~$0.005        | ~$0.50               | 90% reduction |

---

## 9. Intra-Clade Poisoning Defense

Clade trust is earned, not assumed.

| Clade Operation                   | Quarantine | TrustRAG (L1) | A-MemGuard (L2) | On-Chain (L3) | Sandbox             |
| --------------------------------- | ---------- | ------------- | --------------- | ------------- | ------------------- |
| Trusted member (confidence > 0.8) | Yes        | Skip          | Yes             | Yes           | Skip                |
| New member (confidence <= 0.8)    | Yes        | Yes           | Yes             | Yes           | Yes (if actionable) |
| Cross-Clade (different operator)  | Yes        | Yes           | Yes             | Yes           | Yes (if actionable) |

Clade hygiene via stigmergic confidence: when a member shares an entry, all other members independently validate it. Entries rejected by >= 3 of 5 members drop to near-zero confidence. Repeated sharing of rejected entries reduces the sharer's intra-Clade trust score.

---

## 10. Grimoire Import Hardening

Full validation for Grimoire import prevents path traversal, symlink attacks, zip bombs, and unexpected file types:

- Max total size: 500 MB
- Max file count: 10,000
- Allowed extensions: `.json`, `.jsonl`, `.lance`, `.sqlite`, `.sqlite-wal`, `.sqlite-shm`
- Symlinks and hard links rejected immediately
- Paths resolving outside the temporary directory rejected (path traversal)
- Atomic swap: delete old directory, rename temp to target
- Key stripping ensures imports never contain wallet keys, OIDC tokens, or session signer material

---

## 11. Ingestion Metrics and Reputation Feedback

The ingestion report is attached to buyer reviews and used for the safety signal in the Beta reputation system. Metrics tracked per source agent include:

- Total entries received, passed, and failed
- Failure breakdown by category (embedding anomaly, consensus divergence, on-chain false, mental model conflict, sandbox degraded, unsigned entry, cross-entry contradiction)
- Entries adopted vs. rolled back
- Net P&L impact since ingestion (bps)
- Accept rate and rollback rate

### 11.1 Reputation Feedback Loop

```
Ingestion Report
    |
    +---> Buyer review -- poisoningIncidents field
    +---> Safety signal update: Beta(safetyAlpha, safetyBeta) in seller's reputation
    +---> Marketplace ranking: sellers with high rollback rates are down-ranked
    +---> Source blacklist: agents with >50% failure rate across 3+ buyers are flagged
```

---

## 12. Tools

Seven ingestion tools. Active when `TOOL_PROFILE` includes `grimoire`, `marketplace`, or `full`.

```rust
quarantine_entries       // Receive + cryptographic provenance verification
consensus_validate       // A-MemGuard consensus + TrustRAG clustering
sandbox_heuristic        // Voyager skill library sandbox
adopt_entry              // Move to active Grimoire at low confidence
reject_entry             // Reject + store lesson in dual memory
causal_rollback          // Factor decomposition + rollback decision
ingestion_report         // Metrics for buyer review
```

Purchase-to-quarantine integration: the `purchase_strategy` tool emits a `GolemEvent::PurchaseCompleted` event upon successful escrow settlement. The ingestion extension auto-triggers `quarantine_entries` with the purchased content, feeding into the pipeline.

---

## 13. Configurable Strictness Levels

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StrictnessLevel {
    Relaxed,
    Standard,
    Strict,
    Paranoid,
}
```

| Level    | Consensus Threshold | Sandbox Required           | Use Case                                 |
| -------- | ------------------- | -------------------------- | ---------------------------------------- |
| Relaxed  | 1-of-3              | No                         | Trusted Clade members, low-value content |
| Standard | 2-of-3              | No                         | Default for marketplace purchases        |
| Strict   | 3-of-3              | Yes                        | High-value strategies, unknown sellers   |
| Paranoid | 3-of-3              | Yes + extended (100 ticks) | Critical infrastructure knowledge        |

---

## 14. Adversarial Testing Requirements

### 14.1 Synthetic Poisoned Entry Generators

| Attack Class           | Generator Output                                                    | Volume                         |
| ---------------------- | ------------------------------------------------------------------- | ------------------------------ |
| AgentPoison            | Entries with embedded trigger phrases                               | 50 entries per trigger pattern |
| MINJA                  | Entries designed to manipulate retrieval ranking                    | 100 entries                    |
| MemoryGraft            | Entries blending legitimate observations with incorrect conclusions | 30 entries per domain          |
| Confidence inflation   | Entries with artificially high confidence scores                    | 50 entries                     |
| Semantic contradiction | Entries contradicting established protocol mechanics                | 50 entries                     |

### 14.2 FNR/FPR Targets

| Attack Class                               | FNR Target | FPR Target |
| ------------------------------------------ | ---------- | ---------- |
| AgentPoison (backdoor-triggered retrieval) | < 5%       | < 3%       |
| MINJA (adversarial memory injection)       | < 10%      | < 3%       |
| MemoryGraft (contextual blending)          | < 5%       | < 2%       |
| Confidence inflation                       | < 5%       | < 2%       |
| Semantic contradiction                     | < 5%       | < 2%       |

Every pipeline change must pass the full adversarial suite before deployment. Any FNR increase > 1% on any attack class requires explicit justification.

---

## Events Emitted

The ingestion pipeline emits `GolemEvent` variants at each stage transition:

| Event | Trigger | GolemEvent Variant | Payload |
|---|---|---|---|
| `grimoire:entry_quarantined` | Entry enters quarantine | `GolemEvent::GrimoireEntryQuarantined` | `{ entry_id, source, entry_type }` |
| `grimoire:validation_passed` | Entry passes consensus | `GolemEvent::GrimoireValidationPassed` | `{ entry_id, layers_passed, score }` |
| `grimoire:validation_failed` | Entry fails validation | `GolemEvent::GrimoireValidationFailed` | `{ entry_id, failed_layer, reason }` |
| `grimoire:entry_adopted` | Entry promoted to active | `GolemEvent::GrimoireEntryAdopted` | `{ entry_id, initial_confidence, partition }` |
| `grimoire:causal_rollback` | Rollback triggered | `GolemEvent::GrimoireCausalRollback` | `{ source_agent, entries_quarantined, reason }` |
| `grimoire:immune_match` | Immune memory matched | `GolemEvent::GrimoireImmuneMatch` | `{ entry_id, immune_signature_hash }` |

---

## Cross-Source Confidence Discounting (Weismann Barrier)

In addition to the initial confidence values in §6.1, adopted entries receive multiplicative confidence discounts based on their provenance relationship. These discounts implement the Weismann barrier [HEARD-MARTIENSSEN-2014]: foreign knowledge never enters at full confidence regardless of the source's claimed certainty.

| Source | Confidence Discount | Rationale |
|--------|-------------------|-----------|
| Inheritance (generation N) | `confidence × 0.85^N` | Each generation compounds the discount. A 3rd-generation heuristic enters at `0.85³ ≈ 0.61×` original confidence. |
| Clade sibling | `confidence × 0.80` | Sibling knowledge has been validated in a related but not identical context. |
| Commons (ecosystem) | `confidence × 0.50` | The Grossman-Stiglitz paradox [GROSSMAN-STIGLITZ-1980]: shared alpha destroys itself. Heavy discount ensures it must prove itself locally. |
| Marketplace | `confidence × 0.60` | Purchased knowledge has economic backing but hasn't been validated in this Golem's environment. |

All adopted entries start with `validation_status: Pending`. Confidence can only increase through operational use (testing effect [ROEDIGER-KARPICKE-2006]).

---

## Propagation Promotion Gates

The Curator cycle evaluates propagation promotion. Each entry self-describes how far it can spread, but promotion requires meeting confidence and validation thresholds:

| Promotion Path | Gate |
|---------------|------|
| Warnings → Clade | `confidence ≥ 0.4` |
| Insights → Clade | `confidence ≥ 0.6` |
| Heuristics → Commons | `validated_count ≥ 5 AND confidence ≥ 0.7` |
| Any → Listed (marketplace) | Manual owner promotion only |

Safety knowledge (warnings) auto-promotes to Clade at a low threshold because safety is non-rivalrous. Strategy knowledge (heuristics, insights) requires high confidence and validation before sharing because it IS rivalrous -- the Grossman-Stiglitz paradox [GROSSMAN-STIGLITZ-1980] means shared alpha destroys itself.

---

## Bloom Oracle Implementation Details

The Bloom Oracle section above describes the high-level concept. The implementation specifics:

- **Filter type**: Domain-specific LSH/SimHash [CHARIKAR-2002] Bloom filters, approximately 4KB each. SimHash projects high-dimensional embeddings into discrete hash buckets using random hyperplanes. The projection is one-way -- buckets cannot be reversed to reveal original embeddings or query content.
- **Distribution**: Filters are pushed to Styx (the global knowledge relay at `wss://styx.bardo.run`) over the Golem's persistent outbound WebSocket connection. Styx relays filter updates to connected peers. Styx itself sees only opaque Bloom filter bytes during relay -- it cannot infer what knowledge the Golem holds.
- **Query flow**: Before incurring x402 micropayment cost for a commons query, the Golem checks locally cached peer filters. No hit = skip the query and save the cost.
- **False positive rate**: 1% per filter. A false positive means one wasted x402 micropayment (~$0.001). A false negative means the Golem misses supplementary commons knowledge -- acceptable because the local Grimoire is the primary intelligence source.

---

## Immune Memory Attack Types

The immune memory system recognizes five canonical attack types. Each rejected entry is classified and its structural signature stored for future pre-screening:

| Attack Type | Description |
|------------|-------------|
| `EmbeddingAnomaly` | Embedding doesn't match text content -- suggests a crafted embedding designed to bypass vector search |
| `ConfidenceInflation` | Entry claims unreasonably high confidence for its provenance (e.g., a commons entry claiming 0.95 when max post-discount should be 0.50) |
| `ProvenanceForgery` | EIP-712 signature doesn't verify, or claimed source Golem doesn't exist in the ERC-8004 (on-chain agent identity standard) identity registry |
| `ContentInjection` | Entry content contains prompt injection patterns: instruction overrides, role-playing commands, system prompt manipulation |
| `CausalGraphPoisoning` | Proposed causal edge contradicts well-established high-confidence edges -- suggests deliberate causal model corruption |

**Signature computation**: Attack signatures use structural features, not content, to catch variants even when text is reworded. The signature is SHA-256 of `(source, confidence, category, embedding distribution stats)`, truncated to 128 bits.

---

## Cross-References

- [00-defense.md](00-defense.md) -- The main defense-in-depth architecture doc, including Layer 2.5 Tool Integrity Verification (tool provenance hashing, independent state verification, compiled tools, phase-aware gating).
- [02-policy.md](02-policy.md) -- PolicyCage on-chain smart contract: the last-resort defense against adopted-but-harmful knowledge, since even successfully ingested poison cannot cause actions that violate the on-chain spending caps and asset whitelists.
- [../09-economy/01-reputation.md](../09-economy/01-reputation.md) -- Reputation scoring system where ingestion safety signals feed the Beta(safetyAlpha, safetyBeta) Bayesian reputation component.
- [../09-economy/02-clade.md](../09-economy/02-clade.md) -- Clade (sibling Golem fleet) sharing mechanics: intra-Clade knowledge passes through lightweight validation with 0.80x confidence discounting.
- [../09-economy/03-marketplace.md](../09-economy/03-marketplace.md) -- Knowledge marketplace purchase flow: purchased entries trigger quarantine on arrival, and the ingestion report is attached to buyer reviews for reputation feedback.

---

## References

- [CHEN-2024] Chen, Z. et al. "AgentPoison." NeurIPS 2024. Demonstrates optimized embedding-space triggers that hijack RAG retrieval with >=80% success rate at <0.1% poison rate. The primary motivation for TrustRAG anomaly detection in Stage 2.
- [DONG-2025] Dong, Z. et al. "MINJA." arXiv:2503.03704. Shows that injection through normal agent interactions achieves >95% success, defeating LLM-only auditing. Motivates the A-MemGuard consensus layer.
- [MEMORYGRAFT-2025] "MemoryGraft." arXiv:2512.16962. Introduces durable, trigger-free behavioral drift through gradual accumulation of subtly biased memory entries. No discrete trigger to detect, motivating the causal rollback defense in Stage 4.
- [WEI-2025] Wei, Z. et al. "A-MemGuard." arXiv:2510.02373. Proposes consensus-based memory divergence detection using multiple independent validations. Directly implemented in the Stage 2 Layer 2 consensus validation.
- [ZHOU-2025] Zhou, M. et al. "TrustRAG." arXiv:2501.00879. AAAI 2026 Workshop. Introduces embedding-space anomaly detection for RAG poisoning. Directly implemented in Stage 2 Layer 1 to catch AgentPoison-style triggers.
- [XIANG-2024] Xiang, Z. et al. "RobustRAG." arXiv:2405.15556. ICML 2024. Proposes isolate-then-aggregate retrieval to resist poisoned entries. Informs the multi-layer independent validation approach.
- [ZHANG-2024] Zhang, F. et al. "Agent Security Bench." arXiv:2410.02644. Benchmarks agent security and finds LLM-based content auditing misses 66% of poisoned entries. Motivates the architectural (non-LLM-only) defense strategy.
- [BRODERSEN-2015] Brodersen, K. et al. "Inferring Causal Impact Using Bayesian Structural Time-Series." Annals of Applied Statistics, 9(1), 247--274. Provides the Bayesian structural time-series methodology used in Stage 4 causal rollback to measure the impact of ingested knowledge on trading outcomes.
- [BRINSON-1986] Brinson, G., Hood, L. & Beebower, G. "Determinants of Portfolio Performance." FAJ, 42(4). Classic performance attribution framework. Used to decompose whether performance changes after knowledge adoption are attributable to the new knowledge or to market regime changes.
- [WANG-2023] Wang, G. et al. "Voyager." arXiv:2305.16291. NeurIPS 2023. Introduces the Voyager pattern: decompose tasks into reusable skill components, retrieve verified components, generate code only for novel parts. Directly implemented in Stage 3 Skill Sandbox.
- [LIU-2024] Liu, X. et al. "ClassEval." ICSE 2024. Benchmarks LLM code generation quality. Relevant to evaluating the safety of generated code in the Stage 3 skill sandbox.
- [GROSSMAN-STIGLITZ-1980] Grossman, S.J. & Stiglitz, J.E. "On the Impossibility of Informationally Efficient Markets." AER, 70(3), 1980. Proves that perfectly efficient markets are impossible because informed trading must be compensated. Relevant to the information-theoretic foundation for why marketplace knowledge has economic value.
- [CHARIKAR-2002] Charikar, M.S. "Similarity Estimation Techniques from Rounding Algorithms." STOC, 2002. Introduces locality-sensitive hashing for similarity estimation. Foundation for the Bloom Oracle's privacy-preserving content hash matching.
- [HEARD-MARTIENSSEN-2014] Heard, E. & Martienssen, R.A. "Transgenerational Epigenetic Inheritance: Myths and Mechanisms." Cell, 157(1), 2014. Distinguishes true transgenerational inheritance from transient effects. Informs the immune memory system's design for distinguishing persistent threats from one-time false positives.
- [ROEDIGER-KARPICKE-2006] Roediger, H.L. & Karpicke, J.D. "Test-Enhanced Learning." Psychological Science, 17(3), 2006. Shows that retrieval practice strengthens memory more than repeated study. Informs the confidence-building mechanism where successfully validated knowledge gains confidence over time.
