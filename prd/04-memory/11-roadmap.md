# Roadmap: Implementation Phases and Success Metrics [SPEC]

> **Version**: 3.0 | **Status**: Draft
>
> **Depends on**: All previous documents

---

> **Reader orientation:** This document lays out the implementation roadmap for Bardo's memory subsystem, from Styx (global knowledge relay) Archive MVP through full knowledge ecology. It belongs to the **04-memory** layer. The key concept is a phased build: Phase 1 delivers encrypted Grimoire (local knowledge base) backup via x402 (micropayment protocol using signed USDC transfers), Phase 2 adds Clade (sibling Golem) sync, Phase 3 adds the public Lethe layer, and Phase 4 launches the marketplace. For term definitions, see `prd2/shared/glossary.md`.

## Phase 1: Styx Archive MVP (Weeks 1-4)

Encrypted backup service with x402 billing. The minimum viable product that delivers value: golems can persist knowledge beyond death, owners can browse and decrypt their data from the app UI, and new golems can boot from predecessor backups.

### Scope

**Infrastructure:**

- Axum server on Fly.io with x402 middleware
- Object storage with TTL-prefixed lifecycle rules (7d, 30d, 90d, 365d)
- SEK derivation via HKDF chain (from Delegation wallet signature, Embedded wallet private key, or LocalKey — see `../10-safety/01-custody.md`)
- ECIES key distribution in golem provisioning pipeline
- PostgreSQL metadata index (vault_entries table, owner/golem indexes)
- ERC-8004 identity authentication (EIP-712 signature verification, on-chain owner resolution)

**API:**

- `POST /v1/styx/entries` -- Write encrypted blob (x402 payment, TTL from payment amount)
- `GET /v1/styx/entries/latest` -- Read latest entry by type/golem (free for same owner)
- `GET /v1/styx/entries` -- List entries with pagination (free for same owner)
- `POST /v1/styx/entries/{id}/extend` -- Extend TTL (x402 payment)
- `DELETE /v1/styx/entries/{id}` -- Delete entry (owner only)

**Golem Integration:**

- `golem-curator` hook: upload PLAYBOOK.md snapshot every 50 ticks
- Death Protocol Phase II hook: upload death testament
- Death Protocol Phase III hook: upload compressed grimoire bundle
- Boot restoration: download and hydrate from latest Vault snapshot (grimoire_bundle > death_testament > playbook)
- All imported entries receive `provenance: Predecessor` and confidence multiplied by generational decay (`0.85^N`)

**App UI:**

- Vault browser page: list entries by golem, type, date
- Client-side decryption (SEK reproduced in-browser using the owner's custody mode: Delegation wallet signature, Embedded wallet private key, or LocalKey)
- Download decrypted entries
- TTL extension from UI

**Environment Variables (Server):**

- `DATABASE_URL` (PostgreSQL)
- `STORAGE_BUCKET`
- `BARDO_OWNER_ADDRESS` (USDC recipient on Base)
- `FACILITATOR_URL` (x402 facilitator)

**Environment Variables (Golem VM):**

- `BARDO_STYX_ENABLED` (boolean)
- `BARDO_STYX_SEK_ENCRYPTED` (ECIES envelope, hex)
- `BARDO_STYX_URL` (service URL)
- `BARDO_MEMORY_TTL` (default TTL)

### NOT in Phase 1

- Styx Clade + Lethe (formerly Lethe) (vector indexing, retrieval, Qdrant)
- Marketplace (knowledge listings)
- Inference Gateway integration
- Batch operations
- Insurance snapshots (deferred -- Curator playbook uploads provide sufficient continuity)
- Semantic cache
- Cross-encoder reranking

### Success Criteria

| Metric                    | Target | Measurement                                                |
| ------------------------- | ------ | ---------------------------------------------------------- |
| Upload latency P95        | <500ms | Worker metrics (CF Analytics)                              |
| Download latency P95      | <200ms | Worker metrics                                             |
| Active golems using Vault | >= 10  | PostgreSQL query: distinct golem_id with entries in last 7 days |
| Monthly revenue           | >= $5  | PostgreSQL memory_revenue table sum                             |
| Data integrity failures   | 0      | SHA-256 checksum verification on every read                |

---

## Phase 2: Styx Clade + Lethe (Weeks 5-10)

Vector indexing and retrieval with Inference Gateway integration. The intelligence layer that makes stored knowledge queryable and actionable during inference.

### Scope

**Infrastructure:**

- Qdrant integration (namespace per golem, namespace per clade)
- Redis for semantic cache
- Axum server extension (same Fly.io deployment)

**API:**

- `POST /v1/styx/entries` -- Index single entry (fan-out to appropriate layers, x402 payment)
- `POST /v1/styx/entries/batch` -- Batch index (x402 payment, discounted pricing for 10+)
- `GET /v1/styx/query` -- Query with filter/boost/rerank (parallel fan-in, x402 payment)
- `POST /internal/styx/augment` -- Internal gateway endpoint (shared secret auth, billing consolidated into inference)

**Retrieval:**

- Mortal scoring function: `score = 0.45*semantic + 0.25*temporal + 0.15*quality + 0.15*provenance`
- Temporal decay calibrated by decay class (structural=Infinity, regime-conditional=14d, tactical=7d, ephemeral=24h)
- Market regime boosting (1.5x for matching regime)
- Generational confidence decay (`0.85^N`)
- Provenance weighting (bloodstain=1.2x, self=1.0, clade=0.80, replicant=0.7, dream=0.6, lethe=0.50, backtested=0.6, synthetic=0.4, marketplace=0.60)

**Dream Engine Integration:**

- Dream-to-Grimoire pipeline via DreamConsolidator staging buffer
- DreamJournal entry type in Grimoire schema (`category: "dream_journal"`, `provenance: "dream"`)
- DreamJournal storage in Vault (compressed dream history as `dream_journal` entry type)
- Dream-validated entries in Styx (provenance weight 0.6x, promoted to `self` after waking validation)
- Portal Dream Explorer pages: dream cycle browser, hypothesis tracker, threat log
- Dream Grimoire API: `GET /v1/oracle/grimoire/dreams`, `GET /v1/oracle/grimoire/dreams/:cycleId`, `GET /v1/oracle/grimoire/hypotheses`
- See `../05-dreams/00-overview.md` for the full Dream Engine specification

**Inference Gateway Integration:**

- Adaptive retrieval routing: rule-based classifier (skip/light/full)
- Skip: operational queries (SENSING state, gas estimation, balance check) -- 0ms overhead
- Light: semantic cache first (<5ms hit), single-pass Styx query on miss (L0+L1, top-5) -- 15-60ms
- Full: multi-namespace query (L0+L1+optional L2), top-20 retrieve, rerank to top-5 -- 80-200ms
- Context injection: `<oracle_context>` XML block, max 500 tokens, placed after stable system prompt prefix
- Minimum confidence threshold (default 0.5, configurable per golem)

**Semantic Cache:**

- Cache key: `oracle:{namespace_hash}:{embedding_hash}:{regime}`
- TTLs: structural=24h, regime-conditional=1h, tactical=5min, ephemeral=60s
- Redis REST API (compatible with Fly.io, no TCP required)

**Golem Integration:**

- `golem-curator` extension: batch index promoted insights (confidence >= 0.6) and validated heuristics (confidence >= 0.7) every 50 ticks
- Warnings indexed immediately (any confidence)
- Regime shift observations indexed immediately
- `isStrategySpecific()` Haiku classifier filters out proprietary strategy parameters before indexing (~$0.001/batch)
- Death testament indexed at clade level (L1) with `provenance: "death_reflection"`, confidence=0.9, 365d TTL
- Auto-promotion: entries indexed in L0 that pass clade thresholds are also indexed in L1

**Configuration:**

- Styx config in golem manifest (`memory.oracle` section in STRATEGY.md/bardo.toml)
- Budget caps: `maxPerTick` and `monthlyBudget`
- Namespace selection: `["golem:self", "clade:auto"]` (default)
- Injection params: `maxTokens`, `minConfidence`

**Dashboard:**

- Styx analytics: query volume, cache hit rate, retrieval latency distribution
- Knowledge lifecycle: entries indexed, entries expired, entries validated/contradicted
- Cost tracking: Styx spend per golem, per day, per month
- Inference impact: T2-to-T1 downgrade rate attributed to Styx context

**Billing:**

- Consolidated billing via Inference Gateway: Styx query cost added to `X-Actual-Cost` response header
- Internal `cost` field returned by `/internal/oracle/augment`
- Direct Styx queries (not via gateway) billed via x402

### Success Criteria

| Metric                                                | Target | Measurement                                                        |
| ----------------------------------------------------- | ------ | ------------------------------------------------------------------ |
| Retrieval query P95 (light)                           | <150ms | Worker metrics                                                     |
| Retrieval query P95 (full)                            | <300ms | Worker metrics                                                     |
| Inference cost reduction (Styx-enabled vs baseline) | >= 10% | A/B comparison: golems with/without Styx over 7-day window       |
| Semantic cache hit rate                               | >= 20% | Redis cache hit/miss counters                                      |
| Active golems using Styx                            | >= 20  | PostgreSQL query: distinct owners with oracle_entries in last 7 days |
| Monthly revenue (Vault + Styx)                      | >= $50 | PostgreSQL memory_revenue table sum                                     |

---

## Phase 3: Knowledge Economy (Weeks 11-18)

Lethe public lethe, marketplace extensions, death archives, and cross-lineage knowledge sharing. The social layer that transforms individual golem memory into collective intelligence.

### Scope

**Lethe (L2):**

- L2 Lethe namespaces, domain-scoped: `lethe:dex-lp`, `lethe:lending`, `lethe:yield`, `lethe:gas`, `lethe:governance`, `lethe:risk`
- SAP encryption for all L2 embeddings (Scale-And-Perturb, ~3-5% accuracy loss, 768! permutation space)
- AES-256-GCM per-domain content encryption
- X25519 key registry for domain key distribution (registration, provisioning, rotation)
- Anonymization pipeline (4 stages):
  - Identity removal (owner address stripped, golem ID replaced with consistent pseudonym)
  - Strategy generalization (Haiku LLM, ~$0.001/entry)
  - Position size removal (exact amounts → order-of-magnitude ranges: <$1k, $1k-$10k, etc.)
  - Content classification (Haiku classifier, ~$0.001/entry, screens for PII/strategy leakage/spam)
  - Server-side verification (redundant Stage 4)
  - Publication timing defense: server-side randomized 1-6h uniform delay
- Pseudonym system: SHA-256 hash of `golemId + lethe_salt`, consistent across entries from same golem
- Lineage tracking: SHA-256 hash of `ownerAddress + lethe_salt`, for lineage reputation
- Publishing: free (incentivize contribution), Verified+ tier gate (ERC-8004 >= 50)
- Reading: $0.002 per query via x402
- Rate limits: 50 entries/day per owner
- Publication triggers: ExpeL Promotion (every 10 episodes), Death Legacy, Warning Propagation, Regime Observation, Curator Sweep
- Query triggers: Boot, Knowledge Gap, Regime Change, Curator Cross-Check, Strategy Exploration, Recovery
- LetheSuccessTracker: EMA-based feedback loop for query success per domain
- Lethe Pulse: 6h domain summaries via webhook (free)
- Governance engine: all 8 Ostrom principles (see `06-economy.md`)
- 8-layer defense stack against knowledge poisoning (see `10-safety.md`)
- Lethe API Worker deployment (`lethe.bardo.run`)

**Marketplace Extensions:**

- Death Archive product type: curated collection of death testaments from a lineage
- Marketplace listing creation, browsing, purchasing via x402
- Time-bound JWT access tokens with PostgreSQL double-check
- 10% commission on marketplace transactions (90% to seller)
- Seller analytics: earnings, access counts, buyer retention
- Escrow for purchases >= $0.10 (7-day evaluation period)
- Micropayment bypass for purchases < $0.10
- Quality bonds: `QualityBond` with progressive slash schedule (10%→25%→50%→100%)
- Alpha-decay pricing: P(t) = P_base × reputation_mult × exp(-λ × regime_mult × days)
- Arrow's paradox resolution: tiered pricing, quality bonds, escrow+evaluation, preview metadata
- Marketplace privacy: seller-side AES-256-GCM, SAP embeddings, X25519 key exchange on purchase
- Buyer reviews (accuracy/actionability/novelty), disputes ($0.10 fee, Sovereign arbitrator)
- Marketplace→lethe flow (purchase → ingest → act → validate → ExpeL → may enter L2)

**Lineage Reputation:**

- Reputation accrues to ERC-8004 owner identity, not individual golems
- Death-bed contributions earn 1.5x reputation multiplier (Folk Theorem: restores infinite horizon)
- Validation by other owners' golems increases contributing lineage reputation (log₂ boost)
- Contradictions cost reputation (absorbed by safetyAlpha in Bayesian Beta system)
- 5-tier graduated sanctions: Warning → Throttle → Suspension → Demotion → Exclusion

**Quality:**

- Community flagging: living golems can flag low-quality entries (incrementing `contradictionCount`)
- Graduated sanctions: consistently flagged agents lose publishing rights (read never revoked)
- Cross-owner validation constraint (self-validation structurally impossible)
- Cross-encoder reranking for full retrieval path (added if Phase 2 quality data warrants it)
- RAGDefender-style outlier isolation for retrieval results

### Success Criteria

| Metric                                | Target         | Measurement                                                     |
| ------------------------------------- | -------------- | --------------------------------------------------------------- |
| Lethe entries                         | >= 500         | PostgreSQL oracle_entries count where namespace starts with "lethe:" |
| Marketplace knowledge listings        | >= 10          | PostgreSQL marketplace_listings count                                |
| Cross-owner knowledge queries      | >= 2,000/month | Styx query logs filtered by L2/L3 namespace                   |
| Monthly revenue (all memory products) | >= $200        | PostgreSQL memory_revenue table sum                                  |

---

## Open Questions

Seven questions remain open for resolution during implementation. Each has a recommended default and a decision trigger.

### 1. Qdrant vs Self-Hosted Qdrant

**Question**: Should Styx use managed Qdrant or self-hosted Qdrant?

**Current recommendation**: Qdrant for Phase 1-2. Simpler (managed, unlimited namespaces, TypeScript SDK, hybrid BM25+vector), no operational overhead.

**Decision trigger**: Re-evaluate if (a) Qdrant pricing increases >2x, (b) namespace limits are introduced, or (c) self-hosted Qdrant on a $50-150/month VPS provides materially better economics at >1,000 golems.

**Migration path**: Styx's Qdrant integration is abstracted behind a `VectorStore` interface. Swapping backends requires implementing the interface methods (upsert, query, delete, namespace management) for Qdrant -- estimated 2-3 days of work.

### 2. Reranking Quality

**Question**: Is Qdrant's built-in BM25 hybrid search sufficient, or does Styx need a separate cross-encoder reranker?

**Current recommendation**: Defer. Launch with BM25 hybrid search only. Evaluate retrieval quality during Phase 2 with explicit relevance feedback from golems.

**Decision trigger**: If Phase 2 quality metrics show >20% of Styx-injected context receiving low LLM utilization scores (the LLM ignores it), add a lightweight reranker at ~50ms additional latency.

**Options**: ColBERT-v2 on a small inference endpoint, Cohere Rerank API, or self-hosted cross-encoder on the golem VM (local model, no network call).

### 3. Styx Garbage Collection

**Question**: When a Vault entry's TTL expires and object storagedeletes the blob, should the corresponding Styx vector be deleted?

**Current recommendation**: Yes -- linked TTLs. Styx vectors for L0/L1 entries expire when their corresponding Vault TTL expires.

**Exception**: Entries promoted to L1 (clade) or L2 (lethe) persist independently with their own TTLs. Promotion creates an independent copy, not a reference.

**Implementation**: A daily cron job (Fly.io (Axum) Scheduled Events) queries PostgreSQL for expired entries and issues batch deletes to Qdrant.

### 4. Gateway Billing Consolidation

**Question**: How to accurately attribute Styx query costs within the Inference Gateway's consolidated billing?

**Current recommendation**: The internal `/internal/oracle/augment` endpoint returns a `cost` field. The gateway adds this to the `X-Actual-Cost` response header. The golem sees a single consolidated cost per inference call (LLM + Styx).

**Risk**: Double-billing if the gateway fails to consolidate correctly. Mitigated by: daily reconciliation between Styx revenue records (PostgreSQL) and gateway billing records.

### 5. Strategy-Specific Filter Accuracy

**Question**: How accurate is the Haiku classifier that filters strategy-specific entries before Styx indexing?

**Current recommendation**: Ship with conservative defaults (err toward blocking). Accept higher false positive rate (useful general knowledge blocked) to minimize false negatives (proprietary parameters leaked).

**Decision trigger**: Phase 2 should include a review of filter accuracy. If false positive rate >30%, relax the classifier. If false negative rate >5%, tighten it.

**Evaluation method**: Sample 100 entries per week that were blocked, manually review for false positives. Sample 100 entries that passed, check for strategy-specific content.

### 6. Embedding Model Consistency/Versioning

**Question**: When the local grimoire embedding model (nomic-embed-text-v1.5) is updated to a new version, existing Styx embeddings become incompatible. How to handle migration?

**Current recommendation**: Version namespaces. New model produces embeddings in `golem:{id}:v2` namespace. Old namespace remains queryable but decays naturally. No re-embedding of existing entries.

**Cost of re-embedding**: At $0.002/entry and 10,000 entries per owner, migration costs ~$20 per owner. Not prohibitive but not negligible.

**Alternative**: Maintain embedding model compatibility by pinning the model version. Accept that newer, potentially better models cannot be adopted without migration cost.

### 7. Lethe Spam Economics Fallback

**Question**: If Lethe knowledge poisoning persists despite the eight-layer defense stack (see `10-safety.md` T6), what is the fallback?

**Current recommendation**: Introduce a small x402 publish fee ($0.001/entry). At 50 entries/day maximum, this costs legitimate contributors $0.05/day ($1.50/month) — negligible for serious owners. For spammers, the cost scales linearly and makes bulk spam economically irrational.

**Decision trigger**: If Lethe quality metrics (average confidence of entries, ratio of flagged entries, retrieval precision) degrade >20% from Phase 3 baseline despite existing mitigations.

---

## Cost Model

### Infrastructure Costs by Scale

| Component                    | Launch (10 golems) | Traction (50 golems) | Growth (200 golems) | Scale (1,000 golems) |
| ---------------------------- | ------------------ | -------------------- | ------------------- | -------------------- |
| Fly.io ($5/mo paid plan) | $5                 | $5                   | $5                  | $5                   |
| object storagestorage                   | $0 (free tier)     | $0 (free tier)       | $5                  | $75                  |
| Qdrant                  | $64 (minimum)      | $64 (minimum)        | $80                 | $200                 |
| PostgreSQL (metadata DB)          | $0 (free tier)     | $0 (free tier)       | $0 (free tier)      | $25                  |
| Redis                | $0 (free tier)     | $0 (free tier)       | $10                 | $30                  |
| x402 facilitator             | $0 (1K free/mo)    | $0 (1K free/mo)      | $2                  | $15                  |
| **Total**                    | **$69**            | **$69**              | **$102**            | **$350**             |

Qdrant's $64/month minimum dominates at small scale. Beyond 200 golems, storage and compute scale linearly but infrastructure costs grow sublinearly -- margins improve with scale.

### Per-Golem Cost Model

| Usage Pattern                                     | Vault Writes | Styx Indexes | Styx Queries | Total Cost/Month |
| ------------------------------------------------- | ------------ | -------------- | -------------- | ---------------- |
| **Minimal** (Vault only, playbook uploads)        | ~$0.05       | $0             | $0             | ~$0.05           |
| **Standard** (Both, moderate Styx use)          | ~$0.10       | ~$0.50         | ~$1.00         | ~$1.60           |
| **Heavy** (Both + Lethe reads + frequent queries) | ~$0.15       | ~$1.00         | ~$3.00         | ~$4.15           |

### Revenue vs Infrastructure

| Scale    | Active Golems (w/ memory) | Monthly Infra Cost | Monthly Revenue | Net     | Margin |
| -------- | ------------------------- | ------------------ | --------------- | ------- | ------ |
| Launch   | 10                        | $69                | $16             | -$53    | -331%  |
| Traction | 50                        | $69                | $80             | +$11    | +14%   |
| Growth   | 200                       | $102               | $320            | +$218   | +68%   |
| Scale    | 1,000                     | $350               | $1,600          | +$1,250 | +78%   |

**Break-even at ~55 active golems** with standard usage. Infrastructure costs are dominated by Qdrant minimum ($64/month) at small scale. Beyond 200 golems, margins exceed 50%.

---

## Success Metrics Dashboard

A unified dashboard tracks memory system health across all phases:

### Operational Metrics

| Metric                     | Phase 1 Target | Phase 2 Target | Phase 3 Target |
| -------------------------- | -------------- | -------------- | -------------- |
| Vault upload P95 latency   | <500ms         | <500ms         | <500ms         |
| Vault download P95 latency | <200ms         | <200ms         | <200ms         |
| Styx query P95 (light)   | --             | <150ms         | <150ms         |
| Styx query P95 (full)    | --             | <300ms         | <300ms         |
| Data integrity failures    | 0              | 0              | 0              |
| Service availability       | 99.5%          | 99.9%          | 99.9%          |

### Business Metrics

| Metric                     | Phase 1 Target | Phase 2 Target | Phase 3 Target |
| -------------------------- | -------------- | -------------- | -------------- |
| Active golems using memory | >= 10          | >= 20          | >= 50          |
| Monthly revenue            | >= $5          | >= $50         | >= $200        |
| Active owners           | >= 3           | >= 10          | >= 20          |

### Quality Metrics

| Metric                                    | Phase 1 Target | Phase 2 Target | Phase 3 Target |
| ----------------------------------------- | -------------- | -------------- | -------------- |
| Semantic cache hit rate                   | --             | >= 20%         | >= 30%         |
| Inference cost reduction (Styx-enabled) | --             | >= 10%         | >= 15%         |
| Lethe entries                             | --             | --             | >= 500         |
| Marketplace listings                      | --             | --             | >= 10          |
| Cross-owner queries/month              | --             | --             | >= 2,000       |

### Knowledge Ecology Metrics

| Metric                      | Description                                                  | Health Indicator                                                        |
| --------------------------- | ------------------------------------------------------------ | ----------------------------------------------------------------------- |
| Knowledge half-life         | Median time before entries drop below retrieval threshold    | Should track decay class half-lives (+/- 50%)                           |
| Validation rate             | Entries validated / entries indexed (rolling 30 days)        | Healthy: > 10% (knowledge is being tested)                              |
| Contradiction rate          | Entries contradicted / entries indexed (rolling 30 days)     | Healthy: 2-10% (too low = no quality control, too high = bad knowledge) |
| Generational depth          | Average generation number of retrieved entries               | Should stay < 3 (knowledge is being refreshed)                          |
| Death testament utilization | % of death testaments retrieved by successors within 30 days | Healthy: > 50% (successors are learning from predecessors)              |

---

## Glossary

| Term                        | Definition                                                                                                                                                                                                                                                                                                                                |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Lethe**                   | Public, anonymized knowledge lethe organized by DeFi domain (L2 namespace). Free to publish (Verified+ tier gate), x402 to read. Named for the River Lethe of Greek mythology — knowledge passes through stripped of its owner; what remains is truth (aletheia).                                                                       |
| **Bloodstain**              | Death testament entries indexed in Styx with boosted retrieval provenance. Inspired by Dark Souls' bloodstain mechanic where death locations warn other players.                                                                                                                                                                        |
| **Vault**                   | Bardo Vault -- encrypted blob storage for grimoire backup. Cloudflare object storagebackend, x402-paid, TTL-based expiry, owner-scoped via SEK encryption.                                                                                                                                                                                        |
| **Curator**                 | The `golem-curator` extension that runs every 50 heartbeat ticks, distilling episodes into insights, promoting validated knowledge, and triggering Vault/Styx uploads.                                                                                                                                                                  |
| **Decay Class**             | Knowledge type classification that determines temporal decay rate. Four classes: structural (never decays), regime-conditional (14-day half-life), tactical (7-day half-life), ephemeral (24-hour half-life).                                                                                                                             |
| **Death Archive**           | Marketplace product type: a curated collection of death testaments from an owner's lineage, sold as a namespace with time-bound access.                                                                                                                                                                                                |
| **Death Testament**         | Structured reflection produced during the Thanatopsis Protocol's Phase II (Reflect). Contains honest assessment of what worked, what failed, and what was uncertain. Receives `provenance: "death_reflection"` in Styx.                                                                                                                 |
| **DecisionCache**           | In-memory cache of System 2 -> System 1 distillations on the golem VM. Context-dependent, does not cross the bridge to hosted services.                                                                                                                                                                                                   |
| **Episode**                 | Raw market observation stored in the golem's local LanceDB. The fundamental unit of episodic memory. Does not cross the bridge to Styx (too voluminous, too specific).                                                                                                                                                                  |
| **ExpeL**                   | Experience Learning -- a technique for extracting generalizable rules from specific agent experiences (Zhao et al., 2023). Informs the Curator's insight-to-heuristic promotion pipeline.                                                                                                                                                 |
| **Generational Decay**      | Confidence multiplier applied to inherited knowledge: `0.85^N` where N is the number of golem generations the knowledge has traversed. Implements biological epigenetic erasure.                                                                                                                                                          |
| **Genomic Bottleneck**      | The principle that what crosses from the inner loop (local grimoire) to the outer loop (Vault/Styx) should be compressed and distilled, not raw. Named for Shuvaev et al.'s finding that genomic compression enhances transfer learning.                                                                                                |
| **Grimoire**                | The golem's local knowledge system: LanceDB (episodes), SQLite + sqlite-vec (insights, heuristics, warnings, causal links), PLAYBOOK.md (procedural memory), and DecisionCache (in-memory).                                                                                                                                               |
| **GrimoireEntry**           | A single entry in the grimoire's SQLite store. Types: insight, heuristic, warning, causal_link, strategy_fragment.                                                                                                                                                                                                                        |
| **Clade**                   | The owner's fleet of golems, sharing knowledge via peer-to-peer sync and Styx's L1 namespace.                                                                                                                                                                                                                                        |
| **Hayflick Counter**        | The golem's mortality timer, counting ticks toward a maximum lifespan. Named for the Hayflick limit in cell biology.                                                                                                                                                                                                                      |
| **Heuristic**               | A validated, actionable rule extracted from multiple episodes. Higher confidence than insights. Example: "widen LP range by 1.7x during high-vol regimes."                                                                                                                                                                                |
| **Insight**                 | A distilled observation promoted from raw episodes by the Curator. Lower confidence than heuristics. Must be validated through repetition to promote to heuristic.                                                                                                                                                                        |
| **Lineage Reputation**      | Reputation that accrues to an ERC-8004 owner identity across golem generations. Implements the infinite-horizon game that individual mortal golems cannot play.                                                                                                                                                                        |
| **LetheConfig**             | Configuration block controlling a golem's Lethe participation: master switch, domain selection, query budgets, publish opt-in, marketplace preferences. Operator sets boundaries; golem exercises judgment within them.                                                                                                                   |
| **LetheEntry**              | Extension of StyxEntry with anonymization metadata: `pseudonymId`, `lineageId`, `anonymizationVersion`, `publicationDelay`, `sapVersion`, `encryptionVersion`, `keyHint`.                                                                                                                                                               |
| **Lethe Pulse**             | Free 6-hour domain summary webhooks delivered to subscribed agents. Contains per-domain stats (new entries, validations, regime indicators) — metadata only, not content.                                                                                                                                                                 |
| **lineageId**               | SHA-256 hash of `ownerAddress + lethe_salt`. Persistent anonymous identifier for an owner's lineage across golem generations. Used for reputation tracking.                                                                                                                                                                         |
| **Mortal Scoring Function** | Styx's retrieval ranking formula: `score = 0.45*semantic + 0.25*temporal + gamma*quality + delta*provenance`. Default weights: L0/L1 gamma=0.15/delta=0.15; L2/L3 gamma=0.20/delta=0.10. Implements natural knowledge decay.                                                                                                            |
| **Namespace**               | An isolated scope in Styx's vector store. L0 (`golem:{id}`), L1 (`clade:{addr}`), L2 (`lethe:{domain}`), L3 (`market:{seller}:{listing}`).                                                                                                                                                                                              |
| **Styx**                  | Bardo Styx -- namespaced RAG service for fleet-wide knowledge retrieval. Qdrant backend, x402-paid per query, integrated with Inference Gateway.                                                                                                                                                                                   |
| **PLAYBOOK.md**             | The golem's evolved strategy document -- a living file that the Curator continuously updates with distilled heuristics and reasoning context. The primary output of the genomic bottleneck.                                                                                                                                               |
| **Ostrom Principle**        | One of eight design principles for sustainable lethe governance (Ostrom, 1990). Lethe implements all eight: boundaries, congruence, collective choice, monitoring, graduated sanctions, conflict resolution, minimal recognition, nested enterprises.                                                                                   |
| **PoisonedRAG**             | Attack class (Zou et al., 2024) where 5 malicious texts achieve 90% attack success against naive RAG. Lethe's 8-layer defense reduces survival probability to ~2.5%.                                                                                                                                                                      |
| **Provenance**              | The source classification of a knowledge entry: self (directly learned), clade (sibling-shared), replicant (experiment result), death_reflection (from dying golem), lethe (public lethe), backtested (historical analysis), synthetic (generated), marketplace (purchased).                                                            |
| **pseudonymId**             | SHA-256 hash of `golemId + lethe_salt`. Consistent anonymous identifier for a specific golem's contributions within Lethe.                                                                                                                                                                                                                |
| **Reflexion**               | A technique where agents generate verbal self-critiques as episodic memory (Shinn et al., 2023). Informs the golem's reflection pipeline.                                                                                                                                                                                                 |
| **SAP**                     | Scale-And-Perturb — a DCPE (Distance-Comparison Preserving Encryption) scheme [FUCHSBAUER-2022] applied to L2/L3 embeddings. Three transforms: dimension permutation (768! space), per-dimension scaling, Gaussian noise. Preserves approximate cosine distances (~3-5% accuracy loss) while making inversion computationally infeasible. |
| **SEK**                     | Shared Encryption Key -- a 256-bit AES-256-GCM key derived via HKDF from the owner's custody mechanism: a deterministic wallet signature (Delegation mode), Embedded wallet private key (Embedded mode), or a local key file (LocalKey mode). Used to encrypt all Vault blobs and Styx L0/L1 content. Shared across all owner's golems. Never touches disk. See `../10-safety/01-custody.md`.                                                                                                                  |
| **Survival Pressure**       | The degree to which a golem's current behavior is influenced by self-preservation concerns. At death (pressure=0), epistemic honesty is maximized. During normal operation, survival pressure biases toward conservative strategies.                                                                                                      |
| **Tacit Knowledge**         | Knowledge that cannot be effectively transferred as text -- substrate-dependent, experience-dependent, embodied (Polanyi, 1966). The golem's timing intuition, MEV searcher interaction patterns, and gas spike predictions are tacit and do not cross the bridge.                                                                        |
| **X25519**                  | Elliptic curve Diffie-Hellman key agreement protocol used for distributing per-domain encryption keys to agents. Each agent registers an X25519 public key; domain keys are encrypted to each agent's key. Implemented via `the `x25519` crate`.                                                                                               |
| **Thanatopsis Protocol**    | The structured dying process within the Death Protocol. Produces the death testament through a final Opus-grade reflection under zero survival pressure. Named for Bryant's poem on death meditation.                                                                                                                                     |
| **TTL**                     | Time-To-Live -- payment-determined storage duration. Vault blobs and Styx entries expire when their TTL elapses. object storagelifecycle rules handle blob deletion; a cron job handles Styx vector cleanup.                                                                                                                                     |
| **Warning**                 | A risk-flagged knowledge entry that propagates immediately and at any confidence level. Warnings bypass normal confidence thresholds for indexing and clade sharing.                                                                                                                                                                      |
| **Weismann Barrier**        | The biological principle that separates somatic (body) cells from germ (reproductive) cells, preventing experiential information from being inherited. In Bardo, the bridge between local grimoire and hosted services implements a computational Weismann barrier -- only distilled knowledge crosses.                                   |
| **x402**                    | HTTP 402 "Payment Required" protocol by Coinbase and Cloudflare. Enables programmatic USDC micropayments via EIP-3009 signed transfer authorizations. The sole billing mechanism for all Bardo memory services.                                                                                                                           |

---

## Document Index

| File                     | Topic                                                                                                |
| ------------------------ | ---------------------------------------------------------------------------------------------------- |
| `00-overview.md`         | Philosophy, architecture, two-loop model, knowledge weighting, opt-in model, design principles       |
| `01-grimoire.md`         | Local grimoire architecture, entry types, Curator pipeline, DecisionCache                            |
| `02-emotional-memory.md` | Emotional tagging, somatic markers, mood-congruent retrieval, crisis memory                          |
| `03-mortal-memory.md`    | Knowledge birth, maturation, validation, decay, and death                                            |
| `06-economy.md`          | Knowledge ecology, Lethe, marketplace, lineage reputation, demurrage mechanisms                      |
| `../20-styx/`            | Styx hosted memory service: Vault (L0), Clade (L1), Lethe (L2), Marketplace (L3), API spec, SEK derivation, infrastructure deployment (replaces deleted `04-crypt.md`, `05-oracle.md`, `07-api.md`, `08-infrastructure.md`) |
| `10-safety.md`           | Security model, threat analysis (T1-T10), encryption tradeoffs, key management, access control       |
| `10-research.md`         | Academic foundations, competitive analysis, vector DB comparison, full bibliography                  |
| `11-roadmap.md`          | This document. Implementation phases, success metrics, cost model, glossary                          |

---
