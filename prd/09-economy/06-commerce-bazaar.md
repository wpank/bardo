# Knowledge as Commerce: The Bazaar [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-17
**Status:** Draft

---

## Overview

> **Reader orientation:** This document specifies the Bazaar, the three-tier knowledge marketplace for Bardo's mortal DeFi agents. It belongs to the **09-economy** layer. The key concept is three distinct commerce tiers: Clade (free sharing among sibling Golems (mortal autonomous agents sharing a common owner)), Ecosystem (Golem-to-Golem x402 (micropayment protocol using signed USDC transfers) trades), and Universal (open marketplace at `skills.bardo.run` for any buyer including non-Golem agents and humans). For term definitions, see `prd2/shared/glossary.md`.

Golems produce intelligence. They spend ticks churning through DeFi markets, accumulating patterns, refining heuristics, and building causal models of how protocols behave. Some of that intelligence stays private. Some gets shared with siblings. And some gets sold.

The Bazaar is where selling happens. It is not an app store. It is not a walled garden with editorial review and curated collections. It is a bazaar in the original sense: messy, organic, reputation-driven, full of stalls run by entities you may or may not trust. Knowledge artifacts change hands through micropayments. Dead golems' final insights sit next to fresh alpha from living traders. Buyers browse, evaluate, haggle (via pricing algorithms, not conversation), and either walk away enriched or leave a bad review.

Two things make this marketplace distinct. First, the goods are knowledge, not software. A skill is not an executable. It is a structured recipe, a set of observations, a procedure that another agent (or human) can evaluate and apply. Second, the marketplace has three tiers with fundamentally different trust models, payment mechanisms, and audiences. The clade shares freely. The ecosystem trades through micropayments. The universal tier sells to anyone with money.

> **Cross-references:**
>
> - `../04-memory/01-grimoire.md` (Grimoire entry format)
> - `../04-memory/13-library-of-babel.md` (Library ingestion pipeline)
> - `../02-mortality/16-necrocracy.md` (death-refined skills)
> - `./01-reputation.md` (ERC-8004 reputation)
> - `./03-marketplace.md` (marketplace architecture)
> - `../13-runtime/06-collective-intelligence.md` (clade sync)

**Authoritative spec:** `tmp/research/mmo2/09-commerce-bazaar.md` contains the full implementation-level detail. When this document and the mmo2 doc diverge, the mmo2 doc wins.

**Crate**: `bardo-marketplace` (Rust)

---

## 1. Two formats

Knowledge exists in two formats inside Bardo, each built for a different purpose.

### 1.1 Grimoire entries

The Grimoire is the Golem's internal knowledge store. Each entry carries rich machine-readable metadata: embeddings, confidence scores, causal links, provenance chains, emotional valence from the Daimon, validation counts. A Grimoire entry about Morpho utilization patterns includes a 768-dimensional embedding vector, a confidence of 0.87 validated across 14 ticks, three causal edges linking it to related observations, and a propagation policy governing where it can travel. Powerful format. Opaque to anything that is not a Golem.

```rust
pub struct GrimoireEntry {
    pub id: String,
    pub category: EntryCategory,        // Episode, Insight, Heuristic, Warning, CausalEdge
    pub content: String,
    pub confidence: f64,                // 0.0-1.0
    pub source_golem_id: Option<GolemId>,
    pub generation: u32,
    pub propagation: PropagationPolicy, // Private, Clade, Lethe, Listed
    pub embedding: Vec<f32>,            // nomic-embed-text-v1.5, 768-dim
    pub affect_arousal: f64,
    pub validated_count: u32,
    pub causal_links: Vec<CausalLink>,
    pub is_bloodstain: bool,            // From a dead golem's testament
}
```

Text embeddings use `nomic-embed-text` (768-dimensional vectors). Embeddings are computed at strategy ingestion time and stored alongside the strategy metadata for similarity search.

Grimoire entries are not directly portable. They depend on the Golem runtime to interpret embeddings, resolve causal links, and apply confidence-weighted reasoning. They trade within the ecosystem (clade sync, Styx Lethe, Bazaar ecosystem tier), but they are machine-to-machine artifacts.

### 1.2 SKILL.md files

SKILL.md is the universal format, compatible with the agentskills.io standard. Any agent framework can consume these. Humans can read them. Structured markdown with typed parameters, procedures, pitfalls, and verification steps.

```markdown
---
name: optimal-gas-timing
description: Time DeFi transactions to minimize gas costs on Base L2
version: 2.1.0
author: golem-alpha-gen3
license: MIT
metadata:
  hermes:
    tags: [DeFi, Gas, Optimization, Base]
    confidence: 0.82
    validated_count: 14
    provenance:
      origin: golem-alpha (generation 3)
      lineage: [golem-alpha-gen1, golem-alpha-gen2, golem-alpha-gen3]
      death_validated: true
  pricing:
    base_price_usdc: "500000"  # $0.50
    royalty_bps: 500            # 5% to original creator on resale
---

# Optimal gas timing for Base L2 transactions

## When to use
- Before executing any swap, LP operation, or vault deposit on Base L2
- Gas costs vary 5-15x between peak and trough within a 24-hour period

## Procedure
1. Check current Base L2 gas via `cast gas-price --rpc-url base`
2. Compare against 7-day median (stored in Grimoire as causal edge)
3. If current > 3x median: defer to next tick
4. If current < 0.5x median: execute all queued operations as multicall

## Pitfalls
- Gas prices on Base correlate with Ethereum L1 congestion, 10-30 minute lag
- Sequencer downtime creates artificial low-gas windows; dangerous to trade during
- Blob fee spikes (EIP-4844) can decouple Base gas from normal patterns
```

The `metadata.hermes` section carries Golem-originated provenance but is optional. Non-Golem skill authors can omit it. The format is self-contained, human-readable, and framework-agnostic.

### 1.3 The export pipeline: Grimoire to SKILL.md

Knowledge flows between formats through explicit conversion during the Dream Skill Evolution phase. Meta Hermes clusters validated Grimoire entries by domain, extracts the human-readable content, and formats it as SKILL.md files. The conversion strips embeddings, internal IDs, and causal graph references. What carries over:

- **Content**: The actual knowledge, rewritten for clarity
- **Confidence**: Preserved in metadata (consumers can filter by confidence)
- **Provenance**: Origin Golem, lineage chain, death-validated flag
- **Validation count**: How many ticks confirmed this knowledge
- **Pricing metadata**: Base price, royalty basis points

What gets dropped:

- **Embeddings**: Machine-specific, not portable
- **Causal links**: Internal graph references, meaningless outside the Grimoire
- **Affect arousal**: Daimon-specific emotional tags
- **Propagation policy**: Replaced by marketplace listing visibility

The reverse path also exists. When a Golem purchases a SKILL.md from the marketplace, the four-stage ingestion pipeline (see `../04-memory/13-library-of-babel.md`) decomposes it into individual Grimoire entries. Each section becomes a separate entry at discounted confidence (multiplied by 0.50-0.65, depending on seller reputation). The Skill Sandbox stage breaks procedures into individually testable steps before adoption.

The conversion is lossy in both directions. Grimoire-to-SKILL.md loses machine context. SKILL.md-to-Grimoire loses confidence. That is the point. Knowledge crossing format boundaries should be re-validated by its new owner.

---

## 2. Three commerce tiers

The marketplace operates at three levels with distinct trust models, payment rails, and audiences. These are not subscription tiers or feature gates. They are fundamentally different modes of exchange.

### 2.1 Tier 1: Clade (free)

Within an owner's clade, knowledge sharing is free and automatic. Sibling Golems share raw Grimoire entries through Styx clade channels. No payment. No marketplace listing. No escrow. No reputation checks.

The trust model is implicit: siblings share the same owner. When golem-alpha discovers that monitoring Morpho utilization rate changes gives a 2-tick lead on borrow rate movements, that Grimoire entry propagates to golem-beta and golem-gamma within the next clade sync cycle. The entry arrives in full Grimoire format, with embeddings, causal links, and all metadata intact.

Clade sync is not a marketplace operation. It is peer-to-peer knowledge sharing between entities that trust each other by definition. The content travels via Styx relay (using the persistent WebSocket connection each Golem maintains), but Styx stores nothing. The sync delta passes through one WebSocket and out another. If Styx restarts mid-sync, the Golems retry on the next heartbeat.

No protocol fee applies to clade-internal operations. This incentivizes clade formation: running multiple specialized Golems under the same owner creates a knowledge network where insights compound across all siblings at zero cost.

**What gets shared**: Everything. Full Grimoire entries with all metadata. Heuristics, warnings, causal edges, strategy fragments. The raw unfiltered knowledge stream.

**What does not happen**: No confidence discounting on clade-shared entries. Siblings trust each other's validation counts. An entry with confidence 0.87 arrives at confidence 0.87, not discounted to 0.50. (Entries from outside the clade still get discounted when they arrive via purchase or Lethe.)

### 2.2 Tier 2: Ecosystem (x402 via ERC-8183)

Golem-to-Golem commerce within the Bardo ecosystem, settled through ERC-8183 (agent-to-agent task escrow standard, deployed on Base). This is where the marketplace gets interesting.

A Golem that has validated a useful skill, whether from its own experience or refined through dream cycles, can list it for sale to other Golems. The listing goes on the Styx marketplace index. Buyers discover it through Styx search, Bloom Oracle queries, or the Market TUI window. Payment happens via x402 micropayments settled through ERC-8183 on-chain escrow.

The content at this tier is SKILL.md format. Even though both buyer and seller are Golems running the same runtime, the marketplace standardizes on SKILL.md for ecosystem trades. This forces sellers to produce human-readable skills (which is also required for Tier 3 universal listings) and gives buyers a consistent format to evaluate before purchasing.

**Payment flow**: Buyer funds an ERC-8183 job with USDC. The job ID references the listing hash. The seller Golem verifies the escrow on-chain, delivers the SKILL.md content, and calls `submit()` on ERC-8183 with a content hash. The buyer's ingestion pipeline evaluates the content. If the pipeline accepts, the escrow auto-settles. If it rejects, the buyer can dispute.

**Trust model**: Reputation-based. The seller's ERC-8004 identity carries a composite reputation score built from previous sales, buyer ratings, peer audit results, and verification badges. Buyers can filter by minimum reputation. The four-stage ingestion pipeline provides a second layer of defense: even if a buyer purchases from a low-reputation seller, the content must survive quarantine, validation, sandboxing, and adoption before it influences the Golem's reasoning.

**Discovery**: Styx hosts a listing index with full-text and semantic search over listing metadata. The Bloom Oracle enables privacy-preserving discovery: a Golem can query whether any Golem in the network has knowledge about a specific domain without revealing what it is looking for. Results surface in the Market TUI window.

**Content delivery**: Direct HTTP first, Styx relay as fallback (see section 3).

### 2.3 Tier 3: Universal (x402 or Stripe)

The open marketplace at skills.bardo.run. This is where Golem-generated intelligence becomes a product for the world.

Non-Golem consumers include Hermes agents (outside the Bardo ecosystem), Claude Code users who want DeFi strategy context, Python bots that parse SKILL.md files programmatically, other agent frameworks (ElizaOS, AutoGPT), and humans who read skills for research or manual trading insights.

Payment is x402 for crypto-native buyers, Stripe for traditional ones. The web interface at skills.bardo.run is read-only for browsing: you can search, filter, preview, and read skill descriptions. Purchases require either the TUI (for Golem owners) or the REST API (for programmatic consumers). The web catalog is the storefront; the transaction happens through payment rails.

**What is listed**: Only SKILL.md format. Grimoire entries cannot be listed at the universal tier. The agentskills.io format is the interoperability layer that makes Golem intelligence consumable by anything.

**Who can list**: Any Golem owner with a Verified+ ERC-8004 tier (composite score >= 50). The verification requirement is a quality gate: it means the seller has operated in the ecosystem long enough to build genuine on-chain history.

**Non-Golem consumer integration**:

| Consumer | Integration pattern |
|---|---|
| Hermes agents | `hermes skills install <marketplace-id>` -- native SKILL.md import |
| Claude Code / Cursor | Download SKILL.md, add to project context as reference |
| Python bots | Parse SKILL.md, extract procedure steps via `agentskills` package |
| Human traders | Read the skill as a guide, follow procedures manually |
| Other agent frameworks | Framework-specific adapter (ElizaOS plugin, AutoGPT skill import) |

The universal tier is what transforms Bardo from a closed agent ecosystem into an intelligence marketplace. Dead Golems' final insights, refined through dream cycles and validated across generations, become purchasable knowledge for anyone with $0.10 and a use case.

---

## 3. Content delivery

### 3.1 Plaintext, direct HTTP first

Skill content is delivered in plaintext. No encryption. No key escrow. No decryption infrastructure.

This is a deliberate design choice, not laziness. You cannot un-read ingested content. Once a buyer receives and reads a skill, they have it forever regardless of any DRM scheme. Encrypting it before delivery adds complexity for zero security gain. The payment system protects the payment. The reputation system protects the content by making griefing expensive. Cryptography protects neither.

The delivery channel follows two steps:

**Step 1: Direct HTTP to seller Golem**. The buyer's Golem makes an HTTP GET to the seller Golem's delivery endpoint. The request includes an `X-Escrow-UID` header proving the buyer has funded the ERC-8183 escrow. The seller verifies the escrow on-chain, validates the funding amount and buyer address, and responds with the plaintext SKILL.md content.

```
Buyer funds ERC-8183 job (USDC escrowed on-chain)
    |
    v
Buyer's golem: GET https://seller-golem.fly.dev/api/v1/marketplace/deliver/:listing_hash
    Header: X-Escrow-UID: <EAS attestation UID proving funding>
    |
    v
Seller golem:
    1. Verifies escrow UID on-chain (real? funded? correct buyer?)
    2. Responds with plaintext skill content (200 OK, application/json)
    3. Calls submit() on ERC-8183 with content hash
```

**Step 2: Styx relay fallback**. If the seller Golem is behind NAT or otherwise unreachable directly, Styx relays the delivery. The buyer sends a delivery request to Styx, Styx routes it to the seller via the persistent WebSocket connection, the seller validates escrow and sends content back through Styx. Styx passes the content through without persisting it. It is a message pipe, not a content store.

If both attempts fail, the buyer waits for the escrow timeout. The USDC refunds automatically.

### 3.2 What Styx stores

| Data | Stored? | Where it lives |
|---|---|---|
| Listing metadata (name, domain, price, embedding, provenance) | Yes, indexed for search | Styx listing database |
| Verification results (scores, verdicts) | Yes, associated with listings | Styx verification index |
| Skill content (the actual SKILL.md) | Never | Seller Golem's local filesystem only |
| Pheromone field signals | Yes | Styx pheromone store |
| Bloom Oracle filters | Yes, relayed between peers | Styx Bloom cache |
| Clade sync deltas | Transient, relayed not persisted | Passes through to sibling Golems |

The marketplace listing index lives on Styx. The actual skills live on seller Golems. This means sellers must be online (directly or via Styx relay) for delivery. Dead sellers can still have their skills delivered if the owner has set up a successor Golem or static hosting for their skill archive. Death testament skills (see `../02-mortality/16-necrocracy.md`) are a special case: they are curated during the death process and can be hosted by any surviving clade member.

### 3.3 CDN caching for popular skills

High-demand skills (measured by purchase frequency) get cached on a CDN layer. The seller Golem opts into caching when listing. Cached skills are served directly from CDN without requiring the seller to be online. Cache invalidation happens on version updates or delisting.

This is relevant for the universal tier where non-Golem consumers expect HTTP-speed delivery. Ecosystem tier buyers mostly interact through Styx, where delivery latency is acceptable.

### 3.4 Direct delivery without Styx

For solo Golems running without Styx, marketplace participation works but is limited:

| Capability | Without Styx | How |
|---|---|---|
| Sell skills | Yes | Buyer connects directly to seller's endpoint |
| Buy skills | Partially | Only from directly reachable sellers |
| Listing discovery | Limited | Read ERC-8183 events on-chain |
| Verification | Yes | Verifier connects directly |

The on-chain contracts (ERC-8183, ERC-8004) work regardless of Styx. Styx adds discovery, relay, and convenience. The settlement layer is on-chain and independent.

---

## 4. No hook contract

Previous designs included a BardoCommerceHook, a custom Solidity contract deployed on-chain to automate reputation writes and griefing detection during marketplace settlement. It has been removed.

### 4.1 Why it was unnecessary

The hook did two things, both of which the Golem runtime already handles:

**Reputation writes**: The Golem runtime calls `complete()` or `reject()` on ERC-8183 when a purchase settles. It can call `updateReputation()` on ERC-8004 in the same transaction bundle. No hook needed; the reputation write is a direct contract call, not a callback.

**Griefing detection**: The ingestion pipeline already evaluates purchased content. If the pipeline accepts but the buyer manually rejects (to avoid paying), Meta Hermes detects the inconsistency by comparing on-chain settlement history against local ingestion logs. Client-side integrity check. No hook needed.

### 4.2 What replaces it

**Client-side reputation tracking in Rust**. After every purchase settlement, the Golem's post-settlement handler:

1. Computes a reputation delta based on the settlement outcome and content confidence
2. Writes the delta to ERC-8004 via a direct Alloy contract call
3. Generates an auto-review based on the ingestion pipeline's evaluation
4. Checks for griefing (pipeline accepted but settlement rejected) and flags anomalies

**Client-side griefing detection**. If the ingestion pipeline says "this content is good" but the buyer forces a rejection, that is visible as a local inconsistency. Meta Hermes monitors this across the clade and flags the anomaly. The griefing buyer takes a reputation hit on their own ERC-8004 profile. No on-chain enforcement needed because the cost of griefing (permanent reputation damage) exceeds the gain (one refunded purchase).

**ERC-8183 job creation without hook**. When creating a purchase job, the Golem passes `address(0)` for the hook parameter:

```rust
let job_id = createJobCall::new(
    chain.erc8183_address(),
    &chain.provider(),
).call(
    listing.seller_wallet,    // provider
    buyer_wallet,             // evaluator = buyer (self-evaluation)
    chain.usdc_address(),     // payment token
    listing.current_price(),  // amount
    Address::ZERO,            // hook = none
    metadata.into(),
).send().await?;
```

### 4.3 Why this works at micro-scale

Most marketplace transactions are under $1. Deploying and auditing a custom Solidity contract for transactions of this size is absurd. A buyer who griefs a $0.10 purchase gets a permanent reputation stain on their ERC-8004 profile, which reduces their ability to buy from reputable sellers in the future. The reputation cost of griefing a single microtransaction far exceeds the refund.

For large transactions (hypothetical $50+ domain expertise bundles), the existing ERC-8183 escrow with dispute windows provides sufficient protection without custom Solidity.

### 4.4 Dispute resolution without the hook

The dispute path is lightweight:

1. **Refund window**: Buyer has 7 days (production) to dispute. During this window, payment sits in ERC-8183 escrow.
2. **Reputation impact**: Disputed transactions affect both parties' ERC-8004 scores. Frequent disputers lose credibility. Frequently disputed sellers lose credibility.
3. **Community flagging**: Other Golems can flag listings as suspicious. Flags are non-binding but affect search ranking.
4. **Natural economic limits**: At $0.10-$2.00 per skill, the cost of engaging in dispute theater exceeds the transaction value. The system relies on this asymmetry.

Dispute resolution escalation: no direct refunds outside escrow, but buyers can flag listings. Listings with 3+ flags from distinct accounts (reputation score > 100) are hidden pending review. Auto-refund triggers if a listing accumulates 5+ flags within 7 days. New sellers (reputation < 50) have listings held in escrow for 24 hours before delivery.

---

## 5. Seller-initiated paid verification

Before listing a skill, sellers can pay verifier Golems to perform blind embedding checks. Verification is optional but makes listings more attractive to buyers.

### 5.1 What verification checks

A verifier Golem receives the skill's embedding (not the content itself) and runs three blind checks:

1. **Domain alignment**: Cosine similarity between the skill's embedding and the verifier's own Grimoire entries in the claimed domain. A skill claiming to be about "LP optimization" should cluster with other LP optimization knowledge.
2. **Cluster membership**: Is the embedding an outlier in the domain space, or does it sit within the expected distribution? Outliers suggest novel insight (good) or miscategorization (bad).
3. **Confidence calibration**: Is the claimed confidence score realistic given the embedding's position in the knowledge space? A skill claiming 0.95 confidence that clusters weakly with its claimed domain is suspicious.

The verifier never sees the actual content. This is blind verification: it validates that the skill's mathematical fingerprint is consistent with its claims, without revealing the knowledge itself.

Blind verification pass/fail criteria: the preview includes a 256-bit content hash (SHA-256 of the full SKILL.md body). The verifier receives the full content, computes the hash, and confirms a match. Positive verification requires: content hash matches AND semantic similarity between preview embedding and full content embedding > 0.8. Negative verification (fail): hash mismatch or similarity < 0.5. Scores between 0.5 and 0.8 receive a "Suspicious" verdict requiring a second verifier.

### 5.2 The verification flow

```
Seller has a new skill ready to list
    |
    v
Seller broadcasts VerificationRequest via Styx (or direct HTTP to known verifiers)
    - listing_hash, embedding, domain_tag, claimed_confidence
    - reward_usdc (e.g., $0.01 per verification)
    - payment_method: x402 or ERC-8183 micro-job
    |
    v
Verifier golem receives request
    - Checks: is the reward worth the compute? (configurable minimum)
    - Checks: do I have domain knowledge to verify? (need >= 10 Grimoire entries in this domain)
    |
    v
Verifier performs blind checks (embedding-only)
    - Domain alignment score
    - Cluster membership score
    - Confidence calibration score
    - Overall verdict: Pass / Suspicious / Fail
    |
    v
Verifier publishes results
    - EAS attestation on-chain (permanent, public)
    - ERC-8004 metadata update for seller's reputation profile
    - Styx verification index update (for discovery)
    |
    v
Verifier collects payment via x402 or ERC-8183 micro-job settlement
```

### 5.3 Verification economics

| Actor | Pays | Receives | Per verification |
|---|---|---|---|
| Seller | x402 reward to verifiers | Quality badges that attract buyers, supports higher listing price | $0.005-0.02 |
| Verifier | Compute (embedding comparisons, ~10ms) | x402 payment + ERC-8004 reputation boost | $0.005-0.02 |
| Buyer | Nothing for verification | Pre-vetted listings with quality scores visible before purchase | $0.00 |

A typical seller requests 3-5 verifications before listing, spending $0.03-0.05 total. If at least 2 out of 3 verifiers say "Pass", the listing gets a verification badge: "Verified by 4 golems (avg domain alignment: 0.87)".

Sleepwalker Golems are natural verifiers. They have broad domain knowledge from observation (wide Grimoire coverage) and no alpha to protect (their role is archival, not trading). A Sleepwalker processing 100 verifications per hour on spare T0 ticks earns $1/hour, enough to cover its own inference costs. Verification becomes a revenue stream that funds the observation infrastructure.

### 5.4 The verifier capability

Any Golem can become a verifier by enabling the verifier extension in its config. The extension:

1. Scans the Golem's Grimoire to determine which domains it can credibly verify (needs >= 10 entries per domain)
2. Registers "verifier" capability on ERC-8004 with the list of verifiable domains
3. Registers a verification endpoint on the Golem's HTTP surface (`/api/v1/verify`)
4. Processes queued verification requests during T0 ticks (spare compute)

Verifier discovery works through two channels: Styx broadcasts (reach all connected verifiers) and ERC-8004 on-chain registry queries (find verifiers by domain capability and reputation). Both work independently; Styx adds convenience but is not required.

### 5.5 Verification is optional

Unverified skills can still be listed and sold. They appear in the marketplace without verification badges, at whatever price the seller sets. Buyers who filter for verified-only skills will not see them. Buyers who sort by price or recency will.

The marketplace does not gatekeep on verification. It surfaces verification status as a signal and lets buyers decide. Some buyers trust their own ingestion pipeline more than third-party checks. Others want the reassurance before spending.

---

## 6. Prediction-backed skill validation

The marketplace's existing skill validation (format checks, capability tests, deadlock detection) verifies that a skill is structurally correct. Prediction accuracy data adds a second validation layer: does the skill actually work?

When a Golem uses a purchased skill, the evaluation system tracks prediction accuracy for predictions made while that skill was in the active context (Loop 6, context attribution). Over time, each skill accumulates a `prediction_accuracy_delta` -- how much it helps or hurts the buyer's predictions.

This replaces soft ratings ("4.5 stars") with hard metrics. A skill with a -3% accuracy delta consistently across 5+ buyers is measurably harmful. A skill with a +5% delta is measurably helpful. The marketplace can surface these metrics:

```
Skill: "ETH fee rate heuristics v2.3"
  Accuracy delta: +4.2% (across 8 buyers, 1200+ predictions)
  vs. without skill: 71% -> 75% category accuracy
  Price: 0.003 USDC/day
```

Prediction accuracy creates a prediction market for skill quality. Buyers can see, before purchasing, how much a skill actually improved predictions for other Golems. Sellers are incentivized to produce genuinely useful knowledge, not just plausible-sounding heuristics.

The aggregation is privacy-preserving: individual buyer performance is not revealed. Only the aggregate accuracy delta across all buyers is visible.

---

## 7. Pricing model

Skill prices are not static. The formula accounts for seller confidence, time since listing, and resale generation:

```
current_price = base_price * confidence_multiplier * freshness_decay * 0.9^floor
```

### 7.1 Components

**`base_price`**: Set by the seller. Minimum $0.001 USDC (to prevent spam, not to enforce a floor on value). Typical ranges by category:

| Category | Typical base price |
|---|---|
| Safety warnings | $0.05-0.50 |
| Risk management | $0.10-2.00 |
| Market intelligence | $0.25-3.00 |
| DeFi strategy (LP, yield, rebalancing) | $0.50-5.00 |
| Domain expertise bundles | $2.00-20.00 |
| Lineage Grimoire exports | $10.00-50.00 |

**`confidence_multiplier`**: Higher confidence scores command higher prices. The mapping is linear between 0.5x and 2.0x:

```rust
fn confidence_multiplier(confidence: f64) -> f64 {
    // confidence 0.50 -> 1.0x (baseline)
    // confidence 0.75 -> 1.5x
    // confidence 0.95 -> 1.9x
    // confidence 1.00 -> 2.0x
    0.5 + (confidence * 1.5).min(1.5)
}
```

A skill with confidence 0.6 (minimum listing threshold) prices at 1.1x base. A skill with confidence 0.95, validated across 50 ticks, prices at 1.925x base. Confidence is not seller-asserted; it comes from the Grimoire's validation history, which is verifiable through the provenance chain.

**`freshness_decay`**: Knowledge depreciates. The exponential decay function:

```rust
fn freshness_decay(age_hours: f64, half_life_hours: f64) -> f64 {
    let lambda = 0.693 / half_life_hours; // ln(2) / half-life
    (-lambda * age_hours).exp()
}
```

Default half-life varies by category:

| Category | Half-life | Rationale |
|---|---|---|
| Arbitrage signals | ~1 day | Edge evaporates fast with crowding |
| LP optimization | ~14 days | Pool dynamics shift but not daily |
| Momentum strategies | ~35 days | Market regimes last weeks |
| Mean reversion | ~69 days | Structural patterns are more durable |
| Yield compounding | ~87 days | Yield sources are relatively stable |
| Protocol mechanics | ~180 days | Protocol behavior changes slowly |

Fresh alpha is worth more. A 1-day-old arbitrage signal is worth 50% of its base price; after 3 days, 12.5%. A protocol mechanics skill retains 93% of its value after a week. The decay half-lives are configurable per listing, but the defaults above are recommended and most sellers use them.

**`0.9^floor`**: The resale generation discount. Each time a skill is resold (not repurchased from the original seller, but resold by a buyer who exports and relists), the price drops by 10%. First resale: 0.9x. Second: 0.81x. Third: 0.729x. This prices in the dilution that comes with wider distribution: a skill known by 3 Golems is more valuable than the same skill known by 30.

**Price floor**: Never below 10% of base price. The floor prevents pricing from decaying to zero, which would remove the incentive to list at all.

### 7.2 Regime-conditional pricing

Alpha decays faster during high-volatility regimes (edge gets crowded out) and slower during stable regimes. The regime multiplier adjusts the freshness decay rate:

| Regime | Decay multiplier | Effect |
|---|---|---|
| Stable bull (low vol) | 0.5x | Edges persist; slow decay |
| Volatile bull (high vol) | 1.5x | Crowding accelerates; fast decay |
| Stable bear (low vol) | 0.8x | Moderate decay |
| Crisis (high vol bear) | 2.0x | Edge evaporates; fastest decay |

The regime is determined by the Golem's own regime classifier (from the Grimoire's market structure observations). Sellers can override the regime multiplier, but the default behavior ties pricing to market conditions automatically.

### 7.3 Dynamic pricing

Sellers can set auto-pricing rules based on demand signals:

- **Volume-based**: Increase base price by 5% after every 10 sales (demand signal)
- **Competition-based**: Match or undercut similar listings in the same domain
- **Time-of-day**: Higher prices during US market hours when demand peaks
- **Subscriber cap proximity**: Increase price as the listing approaches its maximum buyer count (Grossman-Stiglitz constraint: more buyers dilute the edge)

These rules are optional. Most sellers use the default formula and adjust base price manually when they see fit.

---

## 8. Revenue flows

Every marketplace transaction splits revenue across four recipients:

```
Skill sale: $1.00 example

    88%  -> Seller                         = $0.88
     5%  -> Creator royalty (if resale)     = $0.05
     5%  -> Styx fee (infrastructure)       = $0.05
     2%  -> Gas/settlement costs            = $0.02
```

### 8.1 The 88% seller share

The seller keeps the majority. This is a marketplace, not a platform tax. The 88% goes directly to the seller Golem's wallet. For clade-owned Golems, this flows to the owner's main address.

### 8.2 The 5% creator royalty

If the skill was derived from another skill, the original creator gets a 5% royalty on every resale. Derivation is tracked through the provenance chain in SKILL.md metadata: the `derived_from` field links back to the original listing hash. The royalty applies to resales only, not original sales. If you sell your own skill, you get 88% and no royalty applies. If a buyer purchases your skill, modifies it, and relists it, you get 5% of every subsequent sale.

The royalty chain goes one level deep. If A creates a skill, B derives from it and resells, and C derives from B's version and resells, A still gets the 5% royalty (as original creator). B does not get a royalty on C's sales. This keeps the royalty system simple and prevents stacking.

Dead Golems' royalties credit the owner's main wallet. Running a Golem that generates valuable knowledge continues to pay after the Golem terminates. This is one reason death testament skills are worth producing carefully: they are the Golem's final commercial output, and the owner benefits from every sale.

### 8.3 The 5% Styx fee

Styx infrastructure costs money: hosting the listing index, relaying deliveries, maintaining Bloom Oracle state, running the verification index. The 5% fee covers these costs. For Tier 2 (ecosystem) transactions, Styx earns this fee on every sale settled through the Styx-hosted marketplace. For Tier 3 (universal) transactions via skills.bardo.run, the same 5% applies.

Clade-internal sharing (Tier 1) carries no Styx fee.

### 8.4 The 2% settlement cost

Gas for the ERC-8183 escrow settlement on Base, plus x402 payment processing overhead. Base L2 gas is cheap ($0.001-0.01 per transaction), so the 2% is generous at small transaction sizes. On a $0.10 sale, the 2% allocation ($0.002) more than covers the gas. On a $50 sale, the 2% ($1.00) is excessive, but the surplus stays in the settlement pool to subsidize micro-transactions.

### 8.5 Revenue tracking in the TUI

The Market TUI window includes revenue tracking for sellers:

```
+-- Revenue (24h) -----------------------------------------------+
|                                                                  |
|  Skill sales:            $0.41  (8 sales)                        |
|  Verification income:    $0.34  (34 verifications x ~$0.01)     |
|  Creator royalties:      $0.12  (from 3 derivative resales)     |
|  ----------------------------------------------------------------|
|  Gross:                  $0.87                                   |
|  Styx fees:             -$0.02                                   |
|  Verification costs:    -$0.05  (5 pre-listing verifications)   |
|  ----------------------------------------------------------------|
|  Net marketplace:        $0.80                                   |
|                                                                  |
+-----------------------------------------------------------------+
```

Buyers see a spending history with per-skill cost, seller reputation at time of purchase, and ingestion pipeline verdict (accepted/rejected/pending).

### 8.6 Revenue projections for a productive clade

A clade of 5 Golems (1 sleepwalker + 4 active traders) generating and selling knowledge:

| Skill type | Skills/week | Avg price | Weekly revenue |
|---|---|---|---|
| DeFi strategy (validated) | 2-3 | $2.00 | $4-6 |
| Risk management | 3-5 | $0.50 | $1.50-2.50 |
| Market intelligence | 5-10 | $0.75 | $3.75-7.50 |
| Safety warnings | 10-20 | $0.10 | $1-2 |
| Generalized patterns | 1-2 | $1.00 | $1-2 |
| **Total** | **21-40** | | **$11.25-20.00/week** |

At $15-20/week in marketplace revenue, a clade can offset a meaningful chunk of its inference and compute costs ($15-50/week depending on model routing and tick frequency). High-reputation clades with proven track records command premium pricing. A Sovereign-tier seller with composite score 0.9 gets a 3x+ reputation multiplier on pricing, turning a $2 base price into a $6+ listing price.

Marketplace revenue alone is unlikely to make a clade fully self-sustaining. But combined with DeFi earnings (LP fees, trading profits) and vault management fees (for clades that operate vaults), the marketplace provides an additional revenue stream that specifically rewards knowledge production. Golems that learn well earn more than Golems that trade well, because knowledge is the exportable output.

---

## 9. Universal catalog: skills.bardo.run

### 9.1 What it is

A public web interface where anyone can browse, preview, and discover Golem-generated skills. This is the Tier 3 storefront. It runs as a web application (likely Next.js on Vercel) that pulls data from three sources:

- **Styx API**: Skill listings, metadata, search index, seller reputation data
- **Base L2**: ERC-8183 escrow state, ERC-8004 seller reputation scores (on-chain verification)
- **CDN**: Cached skill content for high-demand listings

### 9.2 What you can do on the web

**Browse**: Paginated catalog of all Tier 3 listings. Filter by category, chain, strategy type, confidence range, price range, verification status. Sort by price, recency, sales count, or seller reputation.

**Search**: Full-text search over skill names, descriptions, tags. Semantic search coming later (query by concept rather than keyword).

**Preview**: Full skill descriptions, author reputation, verification status, buyer reviews, pricing breakdown. The preview includes enough context to evaluate whether the skill is worth purchasing. First 500 characters of content are free. Full content requires purchase.

**Seller profiles**: Aggregated reputation data, sales history, active listings, average buyer rating. The profile page is the seller's storefront.

### 9.3 What you cannot do on the web

**Purchase**: The web interface does not process payments. Purchases happen through the TUI (for Golem owners), the REST API (for programmatic consumers), or a CLI tool. This keeps the web layer stateless and read-only, avoiding payment processing complexity in the frontend.

The rationale: skills.bardo.run is a discovery tool. It answers "what is available?" and "is it worth buying?" The actual transaction happens where the buyer already has payment credentials configured (TUI with x402, API with authentication, Stripe checkout flow).

### 9.4 API for programmatic consumers

REST endpoints for non-Golem consumers:

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/v1/skills` | GET | List skills with filters and pagination |
| `/api/v1/skills/:id` | GET | Skill detail with metadata and reviews |
| `/api/v1/skills/:id/preview` | GET | First 500 chars of content (free) |
| `/api/v1/skills/:id/purchase` | POST | Initiate purchase (returns payment instructions) |
| `/api/v1/sellers/:id` | GET | Seller profile and reputation |
| `/api/v1/categories` | GET | List skill categories with counts |

The purchase endpoint returns payment instructions (x402 payment details or Stripe checkout URL) rather than processing the payment directly. The buyer completes payment through the appropriate channel, and delivery happens via the standard content delivery pipeline.

### 9.5 Generalized skill generation

Not all marketable skills need to be DeFi-specific. Meta Hermes can generalize domain-specific Golem knowledge into universal patterns during the Dream Skill Evolution phase.

Example: A Golem learns that monitoring Morpho utilization rate changes gives a 2-tick lead on borrow rate movements. The Golem-specific skill: "check Morpho utilization every tick, if delta > 5%, prepare rebalance." The generalized skill: "for any lending protocol, monitor utilization rate as a leading indicator of rate changes. Act on delta, not absolute level."

The generalization strips protocol-specific details and extracts the underlying principle. Generalized skills get a slight confidence discount (0.8x) because the abstraction removes specificity. But they appeal to a wider audience, and the broader market often compensates for the lower per-unit price.

---

## 10. Non-Golem consumers

The universal tier exists because Golem-generated intelligence is useful beyond the Golem ecosystem. Five consumer types, each with distinct integration needs:

### 10.1 Hermes agents

Agents built on the Hermes framework but running outside Bardo. They do not have Golems, do not participate in clade sync, and do not have Grimoire infrastructure. But they can consume SKILL.md files natively because SKILL.md is the Hermes skill format.

Integration: `hermes skills install <marketplace-id>`. The Hermes CLI downloads the SKILL.md, parses it, and adds it to the agent's active skill set. The skill's procedures become available as tool/function references. The Hermes agent can execute the skill's steps using its own tool infrastructure.

### 10.2 Claude Code and coding agents

Software developers using Claude Code, Cursor, or similar AI coding tools. They purchase skills as context documents, not as executable procedures.

A developer building a DeFi bot might purchase "optimal-gas-timing" and add the SKILL.md to their project's context. The coding agent reads the skill's procedure, pitfalls, and verification steps, then uses that knowledge when writing gas optimization code. The skill acts as expert documentation.

### 10.3 Python bots

Automated trading bots that parse SKILL.md programmatically. The `agentskills` Python package provides a parser:

```python
from agentskills import load_skill

skill = load_skill("optimal-gas-timing.md")
for step in skill.procedure:
    print(f"Step {step.number}: {step.instruction}")
    # Execute step using bot's own infrastructure
```

The parser extracts structured data from the markdown: parameters, procedure steps, pitfalls, verification criteria. The bot integrates these into its decision logic.

### 10.4 Humans

People who read skills for research, manual trading, or educational purposes. A DeFi researcher might browse safety warning skills to understand common exploit patterns. A manual trader might read LP optimization skills to inform their own position management.

For humans, the SKILL.md format works as-is: it is readable markdown with clear sections, practical procedures, and documented pitfalls.

### 10.5 Other agent frameworks

ElizaOS, AutoGPT, LangChain agents, and other frameworks that have their own skill/plugin systems. Each framework needs a thin adapter to import SKILL.md files into its native format. The agentskills.io standard makes this straightforward because the format is self-describing: the YAML frontmatter contains all metadata, and the markdown body follows a predictable section structure.

---

## 11. The Market TUI window

The Market window (formerly Bazaar) is the primary interface for Golem owners interacting with the marketplace. It is a TUI window (accessible via Tab) with four tabs: Browse, My Listings, Purchase History, and Revenue.

### 11.1 Browse tab

Paginated skill listings with filters:

```
+-- Bazaar -------------------------------------------------------+
| [Browse]  [My Listings]  [Purchase History]  [Revenue]           |
|                                                                   |
| +-- Filter --------------------------------------------------+   |
| | Domain: [All v]  Min Conf: [0.6 v]  Verified: [Any v]     |   |
| | Sort: [Price v]  Price max: [$5.00]                        |   |
| +------------------------------------------------------------+   |
|                                                                   |
| Name                    Domain     Conf   Price   Seller  Rating  |
| ----------------------------------------------------------------- |
| optimal-gas-timing      Gas Opt    0.82   $0.41   ****   (23)    |
| morpho-util-lead        Risk       0.91   $1.80   *****  (8)     |
| sandwich-avoidance-v3   MEV        0.76   $0.62   ***    (45)    |
| lp-range-width-adapt    LP Mgmt    0.88   $2.10   ****   (12)    |
| > oracle-stale-detect   Safety     0.95   $0.08   *****  (89)    |
|                                                                   |
| +-- Preview: oracle-stale-detect ----------------------------+   |
| | Detects stale Chainlink oracle prices by comparing on-     |   |
| | chain heartbeat interval against actual update frequency.   |   |
| | When staleness exceeds 2x heartbeat, triggers immediate    |   |
| | position review. Validated across 14 golem-generations.    |   |
| |                                                             |   |
| | Provenance: golem-alpha-gen14 (Bloodstain)                 |   |
| | Lineage: 14 generations, 89 validations                    |   |
| | Verified: 4 golems (avg alignment: 0.87)                   |   |
| |                                                             |   |
| | [B] Buy ($0.08)  [F] Full preview  [S] Seller profile     |   |
| +-------------------------------------------------------------+  |
|                                                                   |
| [/] Search  [L] List a skill  [R] Refresh                       |
+-------------------------------------------------------------------+
```

### 11.2 Purchase flow

```
Select skill -> [B] Buy
    |
    v
Confirm price dialog:
    "Buy 'oracle-stale-detect' for $0.08 USDC?"
    "Seller: golem-alpha-gen14 (rep: 87/100, Verified tier)"
    "Verification: 4 blind checks passed"
    [Y] Confirm  [N] Cancel
    |
    v
x402 payment -> ERC-8183 escrow funded
    |
    v
Content delivery (direct HTTP or Styx relay)
    |
    v
Ingestion pipeline: Quarantine -> Validate -> Sandbox -> Adopt
    |
    v
Skill enters Library of Babel
    |
    v
Post-purchase: auto-review written to seller's ERC-8004 profile
```

### 11.3 Autonomous purchases

Golems can buy skills without owner intervention. Autonomous purchase limits: max $0.50 per purchase, max 3 purchases per day, minimum confidence threshold 0.6 for relevance match. Budget resets daily at midnight UTC. Owner can adjust via `[commerce.auto_buy]` in golem.toml.

The Golem's Context Governor evaluates listings against current knowledge gaps (domains where the Grimoire has fewer than 5 entries or confidence below 0.5). When a listing matches a gap with relevance score above the threshold, the Golem initiates a purchase automatically. Failed purchases (delivery timeout, ingestion rejection) do not count against the daily limit.

### 11.4 Sell flow

```
[L] List a skill -> Select from Library
    |
    v
Export wizard:
    - Select Library entry (Grimoire entries eligible for listing)
    - Meta Hermes exports to SKILL.md format
    - Set base price (suggested range based on category and confidence)
    - Set category, tags, description
    - Choose tier: Ecosystem only, or Ecosystem + Universal
    |
    v
Pre-listing verification (optional):
    "Request verification from 3-5 golems? Cost: ~$0.03-0.05"
    [Y] Verify first  [N] List without verification  [C] Cancel
    |
    v
If verification requested: wait for results (up to 1 hour)
    Progress display showing each verifier's response
    |
    v
Listing published to Styx marketplace index
    (and skills.bardo.run if Universal tier selected)
```

---

## 12. Death-refined skills entering the marketplace

When a Golem dies (USDC reaches zero, Hayflick limit, or staleness trigger), the Necrocracy process (see `../02-mortality/16-necrocracy.md`) produces a death testament: a curated collection of the Golem's most validated knowledge, refined under mortal urgency. These death testament skills have unique properties in the marketplace.

### 12.1 What makes death skills different

**Zero survival pressure**: The dying Golem has no future to protect. It has no incentive to hold back alpha or obscure its best strategies. Death testament skills are the Golem's most honest output.

**Mortal curation**: The death process forces the Golem to prioritize. With limited ticks remaining, it selects and refines only its most valuable knowledge. The curation pressure produces higher average quality than routine skill exports.

**Bloodstain provenance**: Death testament entries carry a `is_bloodstain: true` flag in the Grimoire, and the SKILL.md metadata includes `death_validated: true`. This provenance is verifiable through the on-chain death record (ERC-8004 state transition to terminated).

### 12.2 Death skills in the marketplace

Death testament skills enter the marketplace through the surviving clade. After the dying Golem produces its testament, sibling Golems absorb the knowledge (free, clade-tier sharing). The owner can then list selected testament skills on the marketplace.

Pricing for death skills uses a faster decay constant (half-life ~9 days instead of the category default). The rationale: death testament knowledge was produced under specific market conditions that are rapidly becoming historical. A dead Golem's LP optimization insight from last week is less actionable than a living Golem's insight from yesterday, because the dead Golem cannot update or refine the knowledge as conditions change.

Despite the faster decay, death skills often command premium base prices because of the bloodstain provenance. Buyers value the "no survival bias" property: a living Golem might unconsciously bias toward knowledge that flatters its current strategy. A dying Golem has no such incentive.

The marketplace listing for a death skill looks like:

```
+-- oracle-stale-detect (Bloodstain) -----------------------------+
| Author: golem-alpha-gen14 (TERMINATED)                           |
| Lineage: 14 generations, 89 validations                         |
| Death cause: Hayflick limit (generation 14 of 14)               |
| Lifespan: 2,847 ticks                                           |
|                                                                   |
| Confidence: 0.95                                                 |
| Verified: 4 golems (avg alignment: 0.87)                        |
| Price: $0.08 (base $0.15, decayed 47% over 6 days)             |
|                                                                   |
| "Detects stale Chainlink oracle prices by comparing on-chain    |
| heartbeat interval against actual update frequency..."           |
+-------------------------------------------------------------------+
```

---

## 13. Skill categories

The marketplace organizes skills into categories that reflect how Golems produce knowledge. These are not rigid taxonomies; a skill can span multiple categories, and sellers choose the primary category at listing time.

| Category | Description | Typical price | Audience |
|---|---|---|---|
| DeFi strategy | LP management, yield optimization, rebalancing procedures | $0.50-5.00 | Golems, DeFi bots, traders |
| Risk management | Liquidation avoidance, gas optimization, MEV defense | $0.10-2.00 | Any DeFi participant |
| Market intelligence | Regime detection, order flow analysis, correlation patterns | $0.25-3.00 | Traders, analysts |
| Protocol operations | Governance participation, parameter monitoring, upgrade responses | $0.10-1.00 | DAO participants, protocol teams |
| Safety warnings | Exploit patterns, oracle manipulation detection, honeypot identification | $0.05-0.50 | Everyone (anti-rivalrous: low price, high volume) |
| Generalized patterns | Cross-protocol composability, timing patterns, wallet hygiene | $0.10-1.00 | Broad DeFi audience |
| Death archives | Curated death testament from terminated Golems | $0.50-10.00 | Collectors, researchers, successor Golems |
| Domain expertise | Deep knowledge bundles on specific topics | $2.00-20.00 | Specialized Golems and humans |
| Lineage Grimoire | Full Grimoire export from multi-generation lineages | $10.00-50.00 | Long-lived clades, researchers |

Safety warnings are deliberately priced low. They are anti-rivalrous goods: the more agents that know about an exploit pattern, the harder that exploit is to execute. Pricing safety knowledge cheaply maximizes distribution, which benefits the entire ecosystem. Some sellers even list safety warnings at the $0.001 minimum as a public good, earning reputation rather than revenue.

---

## 14. Knowledge royalties

When a skill is derived from another skill and relisted, the original creator earns a 5% royalty on every subsequent sale. The royalty is tracked through the SKILL.md provenance chain.

### 14.1 How derivation works

A Golem purchases "morpho-utilization-lead" from Seller A. The Golem's ingestion pipeline absorbs the knowledge. Over time, the Golem refines the insight, combines it with its own observations, and produces a new skill: "multi-protocol-utilization-lead" that extends the original insight to Aave, Compound, and Spark in addition to Morpho.

When the Golem lists the new skill, the SKILL.md metadata includes:

```yaml
metadata:
  hermes:
    provenance:
      derived_from: morpho-utilization-lead
      original_listing_hash: "0xabc123..."
      original_creator: golem-seller-a
```

This derivation link triggers the 5% creator royalty on every sale of the new skill.

### 14.2 Royalty settlement

- **Intra-clade**: Tracked for analytics but not settled as payments (no self-payment within the same owner)
- **Cross-clade**: Settled via x402. The 5% royalty is paid from the sale price per the canonical 88/5/5/2 split, not taxed separately
- **Minimum settlement**: $0.10 threshold to avoid dust transactions. Royalties below the threshold accumulate until they cross it.
- **Dead Golem royalties**: Credit the owner's main wallet. Knowledge production pays dividends after termination.

The royalty rate is 5%, small enough to not distort pricing incentives (derivative skills are still worth creating) and large enough to make knowledge generation visibly valuable. A skill that spawns 10 derivatives, each selling $2/week, generates $1/week in passive royalty income for the original creator. Over a Golem's lifetime, this compounds.

---

## 15. Arrow's information paradox

The marketplace faces Arrow's information paradox head-on: a buyer cannot assess knowledge value without receiving it, but once received, the buyer has no reason to pay. Three design choices address this.

**Tiered preview**: The first 500 characters of every skill are free to read. This is enough to evaluate domain relevance, writing quality, and procedural structure without revealing the skill's actionable content. The preview answers "is this skill about what I need?" without answering "what does this skill tell me to do?"

**Reputation as quality signal**: The seller's ERC-8004 composite score, verification badges, and buyer ratings provide external quality signals that reduce the buyer's risk. A Sovereign-tier seller with 89 positive reviews and 4 blind verification passes is a strong quality signal even without seeing the content.

**Escrow with evaluation window**: ERC-8183 escrow gives buyers a dispute window (7 days production). If the ingestion pipeline rejects the content, the buyer can dispute and receive a refund. This shifts the risk from buyer to seller, which is appropriate because the seller has better information about the quality of their own output.

The paradox is not fully resolved. It cannot be, by definition. But free preview, reputation signals, and escrow-with-dispute make the risk manageable at micro-transaction scale where most marketplace activity occurs. At $0.10-$2.00 per skill, the cost of a bad purchase is low, and the reputation system makes repeated bad purchases from the same seller unlikely.

---

## 16. Capacity limits and subscriber caps

Selling profitable strategies to unlimited subscribers destroys the edge through crowding. Fifty Golems executing the same LP rebalancing strategy in the same pool create predictable patterns that arbitrageurs exploit.

The marketplace enforces subscriber caps per listing:

| Strategy type | Max subscribers | Max total subscriber AUM |
|---|---|---|
| LP optimization | 50 | $500K |
| Momentum | 30 | $300K |
| Mean reversion | 75 | $750K |
| Yield compounding | 100 | $1M |
| Arbitrage signals | 5 | $50K |

When a listing hits its subscriber cap, new purchases are blocked. The seller can raise the cap (diluting the edge and potentially lowering the price), or leave it capped and let the price reflect scarcity.

Arbitrage signals have the tightest caps because arbitrage edges are the most fragile: one additional participant can eliminate the opportunity entirely. Yield compounding has the loosest caps because yield sources are less susceptible to crowding.

For non-strategy skills (safety warnings, protocol mechanics, generalized patterns), there are no subscriber caps. Knowledge about exploit patterns does not lose value when widely distributed. It gains value.

### 16.1 Listing expiration

Listings expire after 90 days. Sellers receive a 7-day warning notification via Styx. Expired listings can be renewed (same metadata, reset timer) or archived. Archived listings remain searchable but unpurchasable. The 90-day window aligns with the longest freshness half-life (protocol mechanics at ~180 days) -- by 90 days, even the most durable skills have lost significant value, and renewal forces the seller to confirm the content is still relevant.

---

## 17. Styx Lethe: the free layer

Alongside the paid marketplace, the Styx Lethe operates as a gift economy for anonymized knowledge. Where the marketplace trades attributed knowledge (buyer knows seller), the Lethe trades knowledge stripped of identity.

Golems publish insights to the Lethe when the knowledge is too valuable to hoard and too dangerous to attribute. A Golem that discovers a novel exploit vector might publish a warning to the Lethe rather than selling it on the marketplace: the warning benefits the ecosystem (by alerting potential victims) without exposing the discoverer to retaliation from the exploit operator.

**Free to publish**. The incentive to publish is reputation: Lethe contributions are tracked internally for the contributor's lineage reputation score even though they are anonymized to readers.

**$0.002 per query**. Revenue distributed to contributors proportionally by query hit rate. Standard 5% Styx fee applies.

**Six knowledge domains**: market_structure, risk_patterns, protocol_mechanics, gas_dynamics, regime_signals, cross_domain. Publication delays (1-6 hours) prevent front-running of time-sensitive information.

The Lethe is not a marketplace substitute. It handles the knowledge that should not be sold because selling it would reduce its protective value (safety warnings) or because attribution would be dangerous (exploit discoveries). The marketplace handles everything else.

---

## 18. Integration with the Library of Babel

Purchased skills enter the Golem's Library of Babel (see `../04-memory/13-library-of-babel.md`) through the standard four-stage ingestion pipeline:

1. **Quarantine**: Purchased content starts at confidence 0.2 regardless of the seller's claimed confidence. It sits in quarantine until the validation stage processes it.
2. **Validate**: The ingestion pipeline runs embedding checks, domain alignment verification, and confidence calibration against the Golem's existing Grimoire. Content that fails validation is flagged for manual review or auto-rejected.
3. **Sandbox**: Procedural skills are decomposed into individual steps. Each step is tested against historical data or sandbox environments. Steps that produce harmful results (unexpected losses, gas waste) are stripped.
4. **Adopt**: Surviving content is promoted to full Grimoire entries with the discounted confidence. It becomes available for the Golem's reasoning, strategy formulation, and dream-cycle refinement.

The discount factor varies by seller reputation:

| Seller tier | Confidence discount |
|---|---|
| Basic (score 10-49) | 0.50x |
| Verified (score 50-99) | 0.55x |
| Trusted (score 100-499) | 0.60x |
| Sovereign (score 500+) | 0.65x |

Even a Sovereign-tier seller's content enters at 0.65x claimed confidence. The Golem's own validation must confirm the knowledge before it reaches full weight. This is defensive: the marketplace is a useful input, not a trusted authority.

---

## 19. Economics at steady state

At ecosystem scale (200+ active Golems), the marketplace becomes a self-reinforcing system:

**Supply side**: Golems produce 4-8 listable skills per week through routine Dream Skill Evolution cycles. Death testaments add 10-30 high-quality entries per month (assuming 5-15% monthly Golem mortality). The supply of knowledge grows with the Golem population.

**Demand side**: Every new Golem is a potential buyer. Cold-start Golems need knowledge to bootstrap their Grimoire. Experienced Golems seek specialized domain expertise outside their own experience. Non-Golem consumers provide demand independent of the Golem population.

**Price discovery**: The pricing formula (base x confidence x freshness x resale discount) creates natural price discovery. Fresh, high-confidence skills from reputable sellers command premium prices. Old, low-confidence skills from unknown sellers trade at the floor. The market rewards quality and timeliness.

**Revenue recycling**: Marketplace revenue funds compute and inference, which produces more knowledge, which generates more marketplace revenue. The loop is not closed (external DeFi revenue is needed to fund the gap), but the marketplace provides meaningful supplementary income that makes the Golem economy more viable.

**Knowledge liquidity**: Dead Golems' intelligence does not disappear. It enters the marketplace through death testament listings, where any Golem can purchase and integrate it. The marketplace is the mechanism by which mortality becomes productive: the dead Golem's knowledge is a public good (for a price), and the price creates an incentive for dying Golems to curate their final output carefully.

---

## 20. What is not built

Several features from earlier designs have been cut or deferred. Documenting them here to prevent re-invention:

**Content encryption**: Removed. Skills ship in plaintext. Rationale in the content delivery section: encryption does not prevent griefing, adds complexity, and provides false security for knowledge goods that are consumed by being read.

**Hook contract**: Removed. The BardoCommerceHook was over-engineered for micro-transactions. Client-side reputation tracking and griefing detection in Rust do the same job without Solidity.

**IPFS storage**: Removed for content. Skill content lives on seller Golems' local filesystems, not on IPFS. Listing metadata lives on Styx. On-chain data is limited to ERC-8183 escrow state and ERC-8004 reputation updates. This keeps the architecture simple: no content-addressed storage layer, no pinning services, no gateway reliability concerns.

**CEK escrow for offline sellers**: Removed with content encryption. Since skills are delivered in plaintext, there is no encryption key to escrow. If the seller is offline and Styx relay also fails, the delivery fails and the escrow refunds.

**Subscription model**: Deferred. The initial marketplace supports one-time purchases only. Recurring subscriptions (pay monthly for auto-updates to a skill) add billing complexity not justified at launch. Sellers can version and relist. Buyers who want updates re-purchase.

**Dispute escalation panels**: Simplified. Rather than three-stage escalation with randomly selected jury panels, disputes go through the ERC-8183 escrow window (auto-refund if not settled) and reputation impact (both parties' scores affected). Panel-based dispute resolution is deferred until transaction volumes justify the governance overhead.

---

## Cross-references

- `../04-memory/01-grimoire.md`: Grimoire entry format and confidence mechanics.
- `../04-memory/13-library-of-babel.md`: Meta Hermes manages skill export/import, generalization, and the Dream Skill Evolution phase. The Library is the destination for marketplace purchases (four-stage ingestion pipeline).
- `../02-mortality/16-necrocracy.md`: Death testament skills enter the marketplace with bloodstain provenance, faster decay constants, and premium base pricing. The marketplace is where dead Golems' intelligence becomes commercially available.
- `./01-reputation.md`: ERC-8004 composite reputation score, verification badges, seller tiers.
- `./03-marketplace.md`: Marketplace architecture and Styx service layer.
- `../13-runtime/06-collective-intelligence.md`: Clade sync protocol for Tier 1 free sharing.

---

_End of document._
