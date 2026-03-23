# Witness DAG: Cryptographic Cognitive Traces [SPEC]

> **Crate**: `golem-safety` (extension of existing audit chain)
>
> **Depends on**: [00-defense.md](00-defense.md) (defense architecture), [07-temporal-logic-verification.md](07-temporal-logic-verification.md) (temporal verdicts enrich witness records), `../01-golem/02-heartbeat.md` (heartbeat pipeline), `../04-memory/01-grimoire.md` (Grimoire provenance)

> **Reader orientation:** This document specifies the Witness DAG, a cryptographic cognitive trace that links every observation, prediction, decision, and outcome a Golem (mortal autonomous DeFi agent) makes into a tamper-proof directed acyclic graph. It belongs to the **Safety** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: the DAG extends the existing linear Merkle audit chain so that any learned knowledge (stored in the Grimoire, the agent's persistent knowledge store) traces backward through BLAKE3-hashed nodes to the raw on-chain observations that justify it. Terms like PolicyCage, Heartbeat, Clade, and ERC-8004 are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

The existing `golem-safety` crate maintains a linear Merkle audit chain: each decision is hashed, each hash commits to the previous one. The chain proves that events happened in sequence. It cannot prove *why*. A linear chain tells you the Golem swapped ETH for USDC at block 19,412,003. It says nothing about the three observations that suggested a regime change, the two predictions that confirmed it, or the risk gate that approved the trade.

The Witness DAG extends the linear chain into a directed acyclic graph that links every observation, prediction, decision, and outcome into a tamper-proof chain of reasoning. Any learned knowledge traces backward through the DAG to the raw on-chain observations that justify it. The linear audit chain becomes a degenerate path through the DAG -- backward compatibility is preserved.

---

## 1. The Problem

### 1.1 Four Gaps in the Linear Chain

**Gap 1: No reasoning provenance.** The audit chain records that the Golem swapped 10 ETH for 32,000 USDC. It does not record which observations, predictions, and risk assessments led to that decision. Post-mortem analysis can determine *what* happened but not *why* the Golem thought it was a good idea.

**Gap 2: No knowledge provenance.** The Grimoire contains entries like "momentum strategies fail in range-bound markets." Which episodes taught this lesson? How many observations support it? There is no way to trace a Grimoire entry back to its evidential basis.

**Gap 3: Trust requires reputation.** Clade members establish trust through ERC-8004 attestations and reputation scores. Reputation is backward-looking and gameable: an agent can build reputation through conservative behavior, then exploit that trust. Verifiable reasoning quality would be a stronger trust signal.

**Gap 4: Auditing requires revelation.** Depositors want to audit decision quality. Today this requires revealing the strategy itself. There is no way to prove "my decisions were well-reasoned" without showing the reasoning.

---

## 2. Mathematical Foundations

### 2.1 DAG Structure

A Witness DAG is a directed acyclic graph W = (V, E) where:

**Vertices V:** Every cognitive event produces a vertex. Five vertex types:

| Type | Label | Created at | Description |
|---|---|---|---|
| Observation | O | Step 1 (OBSERVE) | Raw perceptual data: price feeds, on-chain events, gas prices, liquidity depths |
| Prediction | P | Step 3 (ANALYZE) | Forecasts derived from observations: "ETH will decline 3% in 10 ticks" |
| Decision | D | Step 4-6 (GATE/SIMULATE/VALIDATE) | Actions chosen based on predictions: "Swap 10 ETH for USDC" |
| Resolution | R | Step 8 (VERIFY) | Observed outcomes: "Swap executed at 3,201. ETH declined 2.7%." |
| GrimoireEntry | G | Step 9 (REFLECT) | Learned knowledge from comparing predictions to resolutions |

**Edges E:** Directed edges encode "was used to produce." Direction points from input to output:

- O -> P: "Observation O was used to generate prediction P."
- P -> D: "Prediction P informed decision D."
- D -> R: "Decision D produced resolution R."
- P -> G: "Prediction P contributed to Grimoire entry G."
- R -> G: "Resolution R contributed to Grimoire entry G."
- G -> P: "Grimoire entry G influenced prediction P." (Knowledge feedback loop.)

### 2.2 Cryptographic Commitment

Each vertex carries two hashes:

**Content hash:**

```
h(v) = BLAKE3(type || timestamp || content(v))
```

Commits to the vertex's data. Two vertices with identical content produce identical content hashes.

**Why BLAKE3:** BLAKE3 is chosen over SHA-256 for witness hashing: 3-5x faster on modern hardware, tree-based structure enables incremental hashing of event streams, and 256-bit output provides equivalent collision resistance.

**Commitment hash:**

```
c(v) = BLAKE3(h(v) || c(parent_1) || c(parent_2) || ... || c(parent_n))
```

Parent commitment hashes are sorted lexicographically before hashing. This is the Merkle property: modifying any vertex invalidates the commitment hashes of all its descendants.

### 2.3 Tamper Evidence

The commitment hash c(v) of any vertex commits to the entire subgraph that produced it. If an attacker modifies observation O_17 that was used to generate prediction P_8, then c(O_17) changes, c(P_8) changes, and every decision, resolution, and Grimoire entry downstream of P_8 has an invalid commitment hash.

The root hash (most recent vertex, or a synthetic root committing to all leaves) summarizes the entire reasoning history. Publishing this root to an external system (a blockchain, a timestamping service) creates a non-repudiable commitment to the complete reasoning chain.

The existing linear audit chain is a special case: if every vertex has exactly one parent and the only vertex type is D, the Witness DAG reduces to a linear hash chain. The DAG is a strict generalization.

### 2.4 Hallucination vs. Memory Detection

The DAG enables distinguishing two failure modes that look identical in a linear chain:

**Hallucination.** A decision D has no observation vertices in its provenance subgraph. The Golem made a decision based on fabricated or injected data rather than real on-chain observations. The DAG detects this: `observation_provenance(D)` returns an empty set.

**Memory corruption.** A Grimoire entry G has valid observation provenance, but the commitment hashes in the chain are invalid. Something tampered with the reasoning chain after the fact. The DAG detects this: `verify(G)` returns false.

**Stale knowledge.** A Grimoire entry G has valid provenance, but the observations in its chain are all older than a threshold T. The knowledge is grounded but may be outdated. The DAG quantifies this: `max_observation_age(G)` returns the age of the oldest supporting observation.

---

## 3. Zero-Knowledge Proofs for Strategy Auditing

Using ZK-SNARKs or ZK-STARKs, a Golem can prove statements about its DAG structure without revealing DAG contents. Four proof types:

**Proof 1: Decision grounding.** "This decision was based on at least N observations and M predictions." The prover demonstrates that the subgraph rooted at decision D_i contains at least N observation vertices and M prediction vertices, all with valid commitment hashes. The verifier learns the branching factor but not the content of any vertex.

**Proof 2: Knowledge provenance.** "This Grimoire entry traces back to at least K direct observations." The prover walks the DAG backward from G_j and proves that the reachable subgraph contains at least K observation vertices. The verifier learns evidential depth but not the observations.

**Proof 3: Prediction accuracy.** "My prediction accuracy over the last T ticks exceeds X%." The prover identifies all prediction-resolution pairs in a time window, computes accuracy, and proves the result exceeds the threshold. The verifier learns the accuracy percentage but not individual predictions or resolutions.

**Proof 4: Reasoning consistency.** "All commitment hashes in the subgraph rooted at vertex V are valid." Proves the reasoning chain hasn't been tampered with, without revealing the chain itself.

These proofs transform strategy auditing. A depositor verifies that a Golem makes well-grounded decisions (Proof 1), learns from deep evidence (Proof 2), predicts accurately (Proof 3), and hasn't tampered with records (Proof 4), all without seeing the strategy.

**Implementation note:** ZK proof generation is O(circuit_size). A grounding proof for a typical decision with 10-20 parent vertices takes 1-5 seconds using plonky2. Too slow for real-time, acceptable for on-demand auditing. Full ZK integration is deferred to Tier 4.

---

## 4. Architecture

### 4.1 Integration with the 9-Step Heartbeat

The Witness DAG is constructed incrementally as the heartbeat executes:

| Heartbeat Step | Step Name | DAG Action |
|---|---|---|
| 1 | OBSERVE | Create O vertices for each observation. No parents (these are roots). |
| 2 | RETRIEVE | No new vertices. Grimoire lookups recorded as G -> P edges in Step 3. |
| 3 | ANALYZE | Create P vertices. Edges from O vertices used and any G vertices consulted. |
| 4 | GATE | Create D vertex if risk gate approves an action. Edges from P vertices. |
| 5 | SIMULATE | Update D vertex with simulation results. No new vertices. |
| 6 | VALIDATE | Finalize D vertex. Commitment hash computed at this point. |
| 7 | EXECUTE | Create execution record vertex linked to D. |
| 8 | VERIFY | Create R vertices for each resolution. Edges from D. |
| 9 | REFLECT | Create G vertices for new knowledge. Edges from relevant P, R, D vertices. |

### 4.2 Crate Integration

The existing `Arc<AuditChain>` in GolemState becomes `Arc<WitnessDAG>`. The linear-chain API remains functional; the DAG API is a superset. Any code that appends a decision hash to the audit chain now appends a decision vertex with a single parent edge to the DAG.

### 4.3 Temporal Logic Integration

Each tick's witness includes not just what the Golem did but whether its behavior satisfied its temporal contract (see [07-temporal-logic-verification.md](07-temporal-logic-verification.md)). A violated specification produces a witness of misbehavior -- a cryptographic proof that the Golem failed to meet its behavioral commitments. This is relevant for accountability in multi-Golem Clades.

### 4.4 CorticalState Signal

One new signal on the CorticalState satellite:

```rust
/// Depth of the current Witness DAG: the longest path from any
/// observation vertex to the most recent Grimoire entry.
pub dag_depth: AtomicU32,
```

Clade peers read this signal to gauge how much verified reasoning history a Golem has accumulated. A newly spawned Golem has dag_depth = 0. One running for hours has dag_depth in the hundreds.

---

## 5. Implementation

### 5.1 Core Data Structures

```rust
use blake3::Hash;
use std::sync::Arc;

/// The five types of cognitive event that produce DAG vertices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VertexType {
    Observation = 0,
    Prediction = 1,
    Decision = 2,
    Resolution = 3,
    GrimoireEntry = 4,
}

/// A single vertex in the Witness DAG.
#[derive(Debug, Clone)]
pub struct Vertex {
    /// BLAKE3(content_hash || sorted parent commitment hashes).
    pub commitment_hash: Hash,
    /// BLAKE3(type || timestamp || content).
    pub content_hash: Hash,
    /// What kind of cognitive event this vertex represents.
    pub vertex_type: VertexType,
    /// When this vertex was created, in milliseconds since epoch.
    pub timestamp_ms: u64,
    /// Serialized content of the cognitive event.
    pub content: Vec<u8>,
    /// Commitment hashes of parent vertices, sorted lexicographically.
    pub parent_hashes: Vec<Hash>,
}

impl Vertex {
    /// Create a new vertex and compute both hashes.
    pub fn new(
        vertex_type: VertexType,
        timestamp_ms: u64,
        content: Vec<u8>,
        parent_hashes: Vec<Hash>,
    ) -> Self {
        // Content hash: H(type || timestamp || content)
        let content_hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&[vertex_type as u8]);
            hasher.update(&timestamp_ms.to_le_bytes());
            hasher.update(&content);
            hasher.finalize()
        };

        // Sort parent hashes for deterministic commitment
        let mut sorted_parents = parent_hashes;
        sorted_parents.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        // Commitment hash: H(content_hash || parent_1 || parent_2 || ...)
        let commitment_hash = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(content_hash.as_bytes());
            for parent in &sorted_parents {
                hasher.update(parent.as_bytes());
            }
            hasher.finalize()
        };

        Self {
            commitment_hash,
            content_hash,
            vertex_type,
            timestamp_ms,
            content,
            parent_hashes: sorted_parents,
        }
    }
}
```

### 5.2 The Witness DAG

```rust
use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// The Witness DAG: a content-addressed, append-only DAG of cognitive events.
pub struct WitnessDAG {
    /// All vertices, indexed by commitment hash.
    vertices: DashMap<Hash, Arc<Vertex>>,
    /// Forward edges: parent -> set of children.
    children: DashMap<Hash, Vec<Hash>>,
    /// The commitment hash of the most recently added vertex.
    latest: parking_lot::RwLock<Option<Hash>>,
    /// Maximum depth of any path in the DAG.
    /// Exposed as a CorticalState signal.
    pub dag_depth: AtomicU32,
}

impl WitnessDAG {
    pub fn new() -> Self {
        Self {
            vertices: DashMap::new(),
            children: DashMap::new(),
            latest: parking_lot::RwLock::new(None),
            dag_depth: AtomicU32::new(0),
        }
    }

    /// Append a vertex to the DAG. O(1) amortized.
    pub fn append(&self, vertex: Vertex) -> Hash {
        let hash = vertex.commitment_hash;

        // Register forward edges from each parent to this vertex.
        for parent in &vertex.parent_hashes {
            self.children
                .entry(*parent)
                .or_insert_with(Vec::new)
                .push(hash);
        }

        // Update depth: max(parent depths) + 1.
        let depth = vertex
            .parent_hashes
            .iter()
            .filter_map(|p| self.vertices.get(p))
            .map(|v| self.vertex_depth(&v.commitment_hash))
            .max()
            .unwrap_or(0)
            + 1;

        let current_max = self.dag_depth.load(Ordering::Relaxed);
        if depth > current_max {
            self.dag_depth.store(depth, Ordering::Relaxed);
        }

        self.vertices.insert(hash, Arc::new(vertex));
        *self.latest.write() = Some(hash);

        hash
    }

    /// Walk the DAG backward from a vertex, collecting all ancestors.
    /// Used for provenance queries.
    pub fn provenance(&self, start: &Hash) -> Vec<Arc<Vertex>> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(*start);

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(vertex) = self.vertices.get(&current) {
                for parent in &vertex.parent_hashes {
                    queue.push_back(*parent);
                }
                result.push(Arc::clone(&vertex));
            }
        }

        result
    }

    /// Verify the integrity of a vertex: recompute its commitment hash.
    pub fn verify(&self, hash: &Hash) -> bool {
        let vertex = match self.vertices.get(hash) {
            Some(v) => v.clone(),
            None => return false,
        };

        // Recompute content hash
        let expected_content = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&[vertex.vertex_type as u8]);
            hasher.update(&vertex.timestamp_ms.to_le_bytes());
            hasher.update(&vertex.content);
            hasher.finalize()
        };

        if expected_content != vertex.content_hash {
            return false;
        }

        // Recompute commitment hash
        let expected_commitment = {
            let mut hasher = blake3::Hasher::new();
            hasher.update(vertex.content_hash.as_bytes());
            for parent in &vertex.parent_hashes {
                hasher.update(parent.as_bytes());
            }
            hasher.finalize()
        };

        expected_commitment == vertex.commitment_hash
    }

    /// Find all observation vertices that support a given vertex.
    pub fn observation_provenance(&self, root: &Hash) -> Vec<Arc<Vertex>> {
        self.provenance(root)
            .into_iter()
            .filter(|v| v.vertex_type == VertexType::Observation)
            .collect()
    }

    /// Find all prediction-resolution pairs in the provenance of a vertex.
    pub fn prediction_resolution_pairs(
        &self,
        root: &Hash,
    ) -> Vec<(Arc<Vertex>, Arc<Vertex>)> {
        let ancestors = self.provenance(root);
        let ancestor_set: std::collections::HashSet<Hash> =
            ancestors.iter().map(|v| v.commitment_hash).collect();

        let mut pairs = Vec::new();

        for vertex in &ancestors {
            if vertex.vertex_type != VertexType::Prediction {
                continue;
            }

            if let Some(child_hashes) = self.children.get(&vertex.commitment_hash) {
                for child_hash in child_hashes.iter() {
                    if ancestor_set.contains(child_hash) {
                        if let Some(child) = self.vertices.get(child_hash) {
                            if child.vertex_type == VertexType::Resolution {
                                pairs.push((Arc::clone(vertex), Arc::clone(&child)));
                            }
                        }
                    }
                }
            }
        }

        pairs
    }

    fn vertex_depth(&self, hash: &Hash) -> u32 {
        let vertex = match self.vertices.get(hash) {
            Some(v) => v.clone(),
            None => return 0,
        };
        if vertex.parent_hashes.is_empty() {
            return 1;
        }
        vertex
            .parent_hashes
            .iter()
            .map(|p| self.vertex_depth(p))
            .max()
            .unwrap_or(0)
            + 1
    }
}
```

### 5.3 Storage

The DAG is stored in SQLite with two tables:

```sql
CREATE TABLE vertices (
    hash        BLOB PRIMARY KEY,  -- 32-byte BLAKE3 commitment hash
    content_hash BLOB NOT NULL,    -- 32-byte BLAKE3 content hash
    vertex_type INTEGER NOT NULL,  -- 0=O, 1=P, 2=D, 3=R, 4=G
    timestamp   INTEGER NOT NULL,  -- Unix timestamp in milliseconds
    content     BLOB NOT NULL,     -- Serialized vertex data
    pruned      INTEGER DEFAULT 0  -- 1 if content has been pruned
);

CREATE TABLE edges (
    parent_hash BLOB NOT NULL,
    child_hash  BLOB NOT NULL,
    PRIMARY KEY (parent_hash, child_hash),
    FOREIGN KEY (parent_hash) REFERENCES vertices(hash),
    FOREIGN KEY (child_hash) REFERENCES vertices(hash)
);

CREATE INDEX idx_edges_child ON edges(child_hash);
CREATE INDEX idx_vertices_type ON vertices(vertex_type, timestamp);
```

### 5.4 Pruning and Compression

The full DAG grows linearly with ticks. Each tick produces 5-20 vertices. At one tick per 10 seconds, that is ~8,640 ticks per day, or 43,000-172,000 vertices.

Three pruning strategies:

**Rolling window.** Keep the full DAG for the last T ticks (default: 7 days, ~604,800 ticks).

**Compression beyond the window.** For vertices older than T, replace subtrees with summary vertices. A summary vertex contains:
- Root commitment hash of the replaced subtree (preserving Merkle property)
- Aggregate statistics: vertex count by type, prediction accuracy, knowledge entries produced
- Commitment hashes of any Grimoire entries whose provenance chains pass through the subtree

**Grimoire provenance preservation.** Even after supporting observations are pruned, the hashes in the DAG serve as existence proofs. The provenance chain from a Grimoire entry to its observations remains verifiable (hashes match), even though observation content has been discarded.

Storage estimates: ~200 bytes per vertex. At 100,000 vertices per day, the live DAG consumes ~20 MB/day. A 7-day window is ~140 MB. After compression, historical data adds ~1 MB/day.

### 5.5 On-Chain Consistency Proofs

The DAG root hash can be published on-chain for non-repudiable timestamping. Two modes:

**Periodic anchoring.** Every N ticks (default: 720, or once per day at theta=120s), publish the current DAG root hash to a smart contract. This creates a public commitment that can be verified later.

**Event-driven anchoring.** After significant decisions (large trades, strategy changes, phase transitions), anchor the DAG root. This ties high-impact reasoning to an on-chain timestamp.

The anchoring contract is minimal:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract WitnessAnchor {
    event DAGRootAnchored(
        address indexed golem,
        bytes32 dagRoot,
        uint64 tickNumber,
        uint256 timestamp
    );

    /// Anchor a DAG root hash. Callable by any Golem.
    function anchor(bytes32 dagRoot, uint64 tickNumber) external {
        emit DAGRootAnchored(msg.sender, dagRoot, tickNumber, block.timestamp);
    }
}
```

---

## 6. DAG-Based Trust

### 6.1 Verifiable Reasoning Quality

When Golem A wants to establish trust with Golem B, it shares a DAG subtree. Golem B can verify:

1. **Internal consistency.** All commitment hashes are valid. No vertex modified after creation.
2. **Observation grounding.** Observation vertices reference real on-chain events. Block numbers exist, prices match, gas costs are accurate. Verifiable against any archive node.
3. **Prediction honesty.** Every prediction has a corresponding resolution. The Golem isn't cherry-picking successes. A missing resolution for an old prediction is suspicious.
4. **Knowledge depth.** Grimoire entries descending from many independent observations through multiple prediction-resolution cycles carry more evidential weight than those from a single observation.

Trust becomes proportional to verifiable quality of reasoning, not historical reputation. A new Golem with a short but high-quality chain establishes trust faster than reputation alone would allow.

### 6.2 Styx Integration

When a Golem shares a Grimoire entry with its Clade via Styx, it attaches the DAG subtree rooted at that entry. Clade members verify provenance before incorporating the knowledge. Knowledge sharing shifts from "take it or leave it" to "verify then trust."

---

## 7. Performance Characteristics

| Operation | Cost | Notes |
|---|---|---|
| Vertex creation | O(1) amortized | BLAKE3 hashes 200 bytes in < 100ns |
| Provenance query | O(V + E) | V, E = vertices and edges in reachable subgraph |
| Integrity verification | O(1) per vertex | Hash recomputation and comparison |
| ZK proof generation | O(circuit_size) | 1-5 seconds for typical decision; Tier 4 |
| Storage growth | ~20 MB/day live | ~140 MB for 7-day window |

---

## 8. Evaluation and Falsifiability

### 8.1 Null Hypothesis

Witness DAGs provide no additional trust signal beyond ERC-8004 reputation scores. Depositors given DAG proofs make the same allocation decisions as depositors given only reputation scores.

### 8.2 Experimental Design

Split depositors into two groups. Group A receives standard information (returns, volatility, drawdown, reputation). Group B receives the same plus DAG proofs (decision grounding depth, prediction accuracy, knowledge provenance depth, integrity verification).

Predictions:
- Group B allocates capital more effectively (higher risk-adjusted returns)
- Group B differentiates high-quality from low-quality reasoners better than Group A
- Trust formation is faster: a new Golem with a short but high-quality DAG attracts capital sooner under the DAG regime

### 8.3 Technical Verification Targets

| Metric | Target |
|---|---|
| Tamper detection rate | 100% (deterministic: BLAKE3 collision is infeasible) |
| Provenance completeness | 100% of Grimoire entries have valid observation roots |
| Vertex creation latency | < 1 ms |
| Provenance query latency | < 100 ms for a 7-day DAG |
| ZK proof generation time | < 10 seconds (Tier 4) |
| Storage growth after pruning | < 25 MB/day |

---

## Philosophical Grounding

> Source: `innovations/10-cryptographic-cognitive-traces.md`, Philosophical grounding section

### Transparency as Trust Substrate

Most trust systems work by concealment. You trust your bank because regulators audit it behind closed doors. You trust a fund manager because their track record is good, but you don't see the reasoning behind individual trades. Trust is delegated upward: you trust the auditor, who trusts the data, which you never see.

The Witness DAG inverts this. Trust is grounded in verifiable evidence. A depositor doesn't trust the Golem because a reputation score says it is trustworthy. They trust it because they can verify, cryptographically, that its decisions follow from observations, its predictions resolve against reality, and its knowledge descends from actual experience.

Nick Szabo called this "trust-minimized computation": reduce the trust assumptions required for a system to function. The Witness DAG minimizes trust in the strongest possible sense. You don't trust the agent. You don't trust the platform. You don't even trust the auditor. You verify the hashes yourself.

### The Lab Notebook Analogy

Scientists keep lab notebooks. Every conclusion traces back to an experiment. Every experiment traces back to a hypothesis. Every hypothesis traces back to an observation. The notebook is the evidential chain that separates science from speculation.

The Witness DAG is a lab notebook for autonomous agents. Each Grimoire entry is a conclusion. Each prediction-resolution cycle is an experiment. Each observation is raw data. The DAG formalizes the relationship between these elements and makes the entire chain cryptographically verifiable.

This matters because autonomous agents are black boxes. An LLM-powered decision system produces outputs that are difficult to audit through traditional means; you cannot read the "code" because there is no code in the traditional sense, just weights. The Witness DAG does not make the agent's internal reasoning transparent (the LLM remains a black box), but it makes the *structure* of reasoning transparent: what inputs led to what outputs, which predictions were confirmed by which outcomes, and which experiences produced which knowledge.

### Machine Reasoning and Verification

If a Golem's reasoning chain is verifiable, does it matter that the reasoning was performed by an LLM rather than a human fund manager?

This question matters for DeFi. Traditional finance grants trust to human judgment: a trader's intuition, a portfolio manager's experience, an analyst's insight. These are unverifiable. You trust the person because of their credentials and track record, not because you can inspect their thought process.

A Golem with a Witness DAG has a thought process you *can* inspect, at least structurally. You can count the observations, verify the predictions, measure the accuracy, and trace the knowledge. You cannot inspect the LLM's internal representations any more than you can inspect a human's neural firing patterns. But you can verify the input-output relationships with mathematical certainty.

This is a higher standard of accountability than human fund managers face. Whether it is sufficient for trust, and whether depositors will respond to it, is an empirical question addressed by the evaluation protocol.

---

## Cross-References

- [00-defense.md](00-defense.md) -- The main defense-in-depth architecture doc: six defense layers, Merkle hash-chain audit log (which the Witness DAG extends), and the forensic-grade tamper-evident logging system.
- [07-temporal-logic-verification.md](07-temporal-logic-verification.md) -- LTL/CTL temporal logic verification: temporal verdicts are recorded as witness nodes in the DAG, linking property satisfaction/violation to the observations that triggered them.
- [../04-memory/01-grimoire.md](../04-memory/01-grimoire.md) -- Grimoire (persistent knowledge store): every Grimoire entry gets a provenance chain through the DAG, tracing learned knowledge back to the raw observations that justify it.
- [../01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- The 9-step Heartbeat pipeline: the DAG is constructed incrementally at each step (observe creates Observation nodes, analyze creates Prediction nodes, execute creates Decision nodes, etc.).
- [../01-golem/18-cortical-state.md](../01-golem/18-cortical-state.md) -- CorticalState (32-signal atomic shared perception surface): the `dag_depth` signal exposes the current DAG depth to all extensions.
- [../09-economy/01-reputation.md](../09-economy/01-reputation.md) -- Reputation scoring system: DAG-based verifiable reasoning quality complements the Bayesian reputation score, providing a stronger trust signal than backward-looking reputation alone.

---

## References

- [MERKLE-1987] Merkle, R.C. "A Digital Signature Based on a Conventional Encryption Function." *CRYPTO '87*, LNCS 293, 369-378, 1987. Introduces Merkle trees for efficient hash-chain verification. The existing linear audit chain uses this; the Witness DAG generalizes it to a DAG structure.
- [BENET-2014] Benet, J. "IPFS - Content Addressed, Versioned, P2P File System." arXiv:1407.3561, 2014. Defines content-addressed storage using cryptographic hashes. Informs the DAG's content-addressing scheme where node IDs are BLAKE3 hashes of their contents.
- [SZABO-1997] Szabo, N. "Formalizing and Securing Relationships on Public Networks." *First Monday*, 2(9), 1997. Introduces the concept of smart contracts and computational enforcement of agreements. Provides the conceptual foundation for why cryptographic cognitive traces matter for trust.
- [GOLDWASSER-1985] Goldwasser, S., Micali, S., & Rackoff, C. "The Knowledge Complexity of Interactive Proof Systems." *STOC '85*, 291-304, 1985. Defines zero-knowledge proofs: proving a statement without revealing anything beyond its truth. Foundation for the ZK proof integration that allows auditing decision quality without revealing the strategy.
- [BEN-SASSON-2018] Ben-Sasson, E., Bentov, I., Horesh, Y., & Riabzev, M. "Scalable, Transparent, and Post-Quantum Secure Computational Integrity." Cryptology ePrint Archive 2018/046, 2018. Introduces STARKs for scalable, transparent proofs. The target proof system for privacy-preserving DAG verification.
- [LAMPORT-1979] Lamport, L. "How to Make a Multiprocess Computer That Correctly Executes Multiprocess Programs." *IEEE TC*, 28(9), 690-691, 1979. Defines sequential consistency for concurrent systems. Relevant because the DAG must maintain causal ordering guarantees across concurrent heartbeat steps.
