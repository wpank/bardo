# Performance Targets and Technical Specifications [SPEC]

> **Status**: Reference specification
>
> **Source**: Consolidated performance, chain support, gas, and subsystem specifications.

> **Reader orientation:** This appendix consolidates all performance targets and technical specifications across the Bardo ecosystem: vault gas costs, Heartbeat (the 9-step decision cycle) tick latency, HDC operation benchmarks, chain intelligence pipeline timing, Grimoire (the agent's persistent knowledge base) memory operations, per-tick compute budgets at each timescale (Gamma/Theta/Delta), chain support matrix, contract gas estimates, ERC standards checklist, and subsystem specifications. The key concept is the per-tick compute budget: all Triage Augmentation innovations are T0-compatible (zero inference cost, no LLM calls). See `prd2/shared/glossary.md` for full term definitions.

---

## Performance Targets

| Metric                         | Target                    |
| ------------------------------ | ------------------------- |
| Vault deployment gas           | < 3M gas                  |
| Deposit/withdraw latency       | < 2 blocks                |
| Tool response time             | < 500ms (p95)             |
| Heartbeat tick (suppressed)    | < 100ms, $0               |
| Heartbeat tick (escalated)     | < 10s, < $0.10 LLM cost   |
| Memory search latency          | < 25ms                    |
| Share pool price convergence   | < 5 min to NAV ±1%        |
| Wallet creation                | < 500ms                   |
| Transaction signing            | < 200ms                   |
| Onboarding (atomic)            | < 30s, $0 gas (paymaster) |
| GottsLoop daily cost (typical) | < $5 (95% Haiku ticks)    |
| Memory RAM footprint           | 150–250 MB                |

---

## HDC Operations

Latency targets measured on Apple M1 Pro (NEON SIMD). x86-64 with AVX2 should match or beat these numbers thanks to native POPCNT. Dimension D=10,240 bits (160 u64 words) unless noted.

| Operation | Target Latency | Method |
| --- | --- | --- |
| Hamming distance (D=10,240) | ~10 ns | 160 XOR64 + 160 POPCNT64 |
| HyperVector bind (XOR) | ~2 ns | 160 XOR64 |
| HyperVector unbind (XOR, self-inverse) | ~2 ns | 160 XOR64 |
| HyperVector permute (cyclic shift) | ~5 ns | 160 shift + 160 OR |
| HyperVector bundle (majority vote, K=10) | ~15 us accumulate, ~12 us finish | Per-bit vote counting + threshold |
| Full transaction fingerprint encode | < 100 us | Bind + permute across transaction fields |
| Resonator factorization (1 iter, 3 factors, 100 codebook) | ~300 us | Iterative projection |
| ANN query (usearch binary HNSW, 100K items) | < 1 ms (main), ~1.5 ms (main + staging) | Hamming metric, M=16, ef_search=100 |
| ANN staging insert | ~100 us | HNSW incremental insert |
| Hamming top-K pre-filter (100 candidates) | ~1 us | 100 x Hamming distance |

**Total POPCNT budget per Gamma tick**: ~3.5 us across ~355 comparisons (curiosity pre-filter, anti-pattern screening, context dedup, regime classification, dream correlation). This is noise within a 5-15s Gamma interval.

---

## Chain Intelligence Pipeline

| Operation | Target Latency | Notes |
| --- | --- | --- |
| Binary Fuse8 lookup | ~10 ns per address | 3 memory accesses, branchless (xorf crate) |
| Binary Fuse8 construction (10K keys) | ~100 us | Rebuilt at each Gamma tick via arc-swap |
| DDSketch quantile query | O(1), ~15 ns observe | Logarithmic bucketing, alpha=0.01, ~2 KB fixed memory |
| DDSketch quantile read | O(log n) bucket scan | Formal relative error guarantee |
| MIDAS-R anomaly score | ~50 ns per edge | 3 CMS queries + chi-squared |
| CMS frequency update | ~20 ns | k hash + k increment |
| Single block triage (150 txs) | < 5 ms total | Binary Fuse8 filter + severity scoring |
| Bayesian surprise update | ~300 ns per observation | Conjugate prior arithmetic + digamma |
| Bayesian surprise decay (100 protocols) | ~50 us | Linear scan of parameters |

**False positive rates**: Binary Fuse8 achieves 0.39% FPR at 8.7 bits/entry, down from ~1% FPR at 9.6 bits/entry with standard Bloom filters.

---

## Memory / Grimoire

| Operation | Target Latency | Notes |
| --- | --- | --- |
| HDC episode compression | < 500 us per episode | Bundle accumulate at Theta cadence |
| Similarity search (10K episodes) | < 5 ms | Hamming pre-filter then cosine refinement |
| Grimoire HDC retrieval (top-10) | < 2 ms | Binary HNSW + Hamming distance |
| SQLite query | < 1 ms | Existing PRD2 target |
| LanceDB vector search | < 25 ms | Existing PRD2 target |
| Memory per HDC vector | 1,280 bytes | 10,240 bits (vs. 3,072 bytes for 768-dim f32) |

---

## Per-Tick Compute Budget

All TA (Triage Augmentation) innovations are T0-compatible: zero inference cost, no LLM calls, no external API calls, no blocking I/O.

### Gamma Tick (5-15s interval)

Target total: < 200 ms (sensing + triage + CorticalState update). Current measured budget: ~7-17 ms.

| Component | Per-Event Cost | Events/Tick | Total/Tick |
| --- | --- | --- | --- |
| Block ingestion + log filtering (existing) | -- | -- | 5-10 ms |
| Probe severity scoring (existing) | -- | -- | 1-2 ms |
| CorticalState atomic writes (existing) | ~0.1 us | -- | ~0.1 us |
| HDC fingerprint encoding | ~100 us | 10-50 | 1-5 ms |
| Binary Fuse8 queries | ~10 ns | 100-500 | 1-5 us |
| Bayesian surprise updates | ~300 ns | 10-50 | 3-15 us |
| MIDAS-R scoring | ~50 ns | 10-50 | 0.5-2.5 us |
| DDSketch observations | ~15 ns | 10-50 | 0.15-0.75 us |
| Hamming pre-filter | ~1 us | 5-20 | 5-20 us |
| CMS frequency updates | ~20 ns | 10-50 | 0.2-1 us |
| CorticalState HV encoding | ~100 us | 1 | 100 us |
| **Total new TA overhead** | | | **~1.2-5.3 ms** |

TA overhead: < 0.1% of available Gamma budget.

### Theta Tick

Target total: < 2 s (includes deliberation, prediction, context assembly). LLM-dominated (30-120s cycle).

| Component | Total/Tick |
| --- | --- |
| ANN staging insert | ~0.1 ms |
| Episode compression (accumulate) | ~0.015 ms |
| Regime classification | ~0.001 ms |
| Hedge weight update | ~0.01 ms |
| Thompson threshold selection | ~0.05 ms |
| Bayesian surprise (Theta batch) | ~0.05 ms |
| **Total new TA overhead** | **~0.2 ms** |

### Delta Tick

Target total: < 30 s (includes dream scheduling, consolidation check). Runs on 5-20 minute cadence.

| Component | Total/Tick |
| --- | --- |
| Bundle prototype refresh | 1-5 ms |
| Drift detection (centroid comparison) | ~0.15 ms |
| Curator consolidation | 50-200 ms |
| ANN main rebuild | 100 ms - 2 s |
| Bayesian surprise (Delta batch) | ~0.1 ms |
| **Total new TA overhead** | **~150 ms - 2.2 s** |

---

## TA Pipeline (Topological Analysis)

| Operation | Target Latency | Notes |
| --- | --- | --- |
| Persistence diagram computation (1000-point cloud) | < 50 ms | Scheduled at Theta/Delta cadence |
| Wasserstein distance | < 10 ms | Between persistence diagrams |
| Somatic marker lookup | < 1 ms | HDC similarity (Hamming distance) |

---

## mirage-rs v2 Simulation

| Metric | Target | Notes |
| --- | --- | --- |
| Per-branch overhead (CoW) | ~12.8 KB | CoW overlay with ~100 dirty slots |
| Branch creation time | ~1 us | Arc clone + empty HashMap |
| Transactions replayed per block | 5-15 | Targeted, watched contracts only (vs. ~150 full replay) |
| Max memory per branch | 512 MB standard, 2 GB power | Resource profile `max_memory` setting |

---

## Hardware Assumptions

- **CPU**: Apple M1 Pro or equivalent (NEON SIMD). x86-64 with AVX2/POPCNT.
- **POPCNT**: Available on all modern targets. LLVM compiles `count_ones()` / `count_zeros()` to hardware intrinsic automatically.
- **Memory**: DDSketch uses ~2 KB fixed regardless of observation count. HDC vectors use 1,280 bytes each.
- **Dependencies**: xorf (Binary Fuse8), sketches-ddsketch, usearch (HNSW), statrs (digamma), arc-swap (lock-free replacement). Total compiled footprint: ~2.5 MB, no C++ dependencies required.

---

## Chain Support Matrix

| Chain      | Chain ID | V4 Deployed | Vault Support   | Share Pools |
| ---------- | -------- | ----------- | --------------- | ----------- |
| Ethereum   | 1        | Yes         | Yes (v1 target) | Yes         |
| Base       | 8453     | Yes         | Yes (v1 target) | Yes         |
| Unichain   | 130      | Yes         | Yes (v1 target) | Yes         |
| Optimism   | 10       | No          | Tools only      | No          |
| Polygon    | 137      | No          | Tools only      | No          |
| Arbitrum   | 42161    | No          | Tools only      | No          |
| BNB Chain  | 56       | No          | Tools only      | No          |
| Avalanche  | 43114    | No          | Tools only      | No          |
| Celo       | 42220    | No          | Tools only      | No          |
| Blast      | 81457    | No          | Tools only      | No          |
| zkSync Era | 324      | No          | Tools only      | No          |

v1 targets full vault and share pool support on Ethereum, Base, and Unichain -- the three chains with V4 deployed. All other supported chains receive tool coverage for data reads, trading, and LP operations via the Uniswap Trading API.

---

## Contract Gas Estimates

| Operation                     | Estimated Gas | Est. Cost (Base L2) | Est. Cost (Ethereum) |
| ----------------------------- | ------------- | ------------------- | -------------------- |
| Vault deployment (CREATE2)    | 2.5–3M        | $0.05–0.10          | $5–15                |
| Deposit (ERC-4626)            | 80–120K       | $0.002–0.005        | $0.50–1.50           |
| Withdraw (instant)            | 80–120K       | $0.002–0.005        | $0.50–1.50           |
| Rebalance (single adapter)    | 150–250K      | $0.005–0.01         | $1.00–3.00           |
| Hook swap (VaultHook)         | 100–180K      | $0.003–0.007        | $0.80–2.00           |
| Agent registration (ERC-8004) | 150–200K      | $0.004–0.008        | $1.00–2.50           |
| Milestone attestation         | 50–80K        | $0.001–0.003        | $0.30–0.80           |
| Proxy announce                | 60–100K       | $0.002–0.004        | $0.40–1.00           |
| Proxy execute                 | 80–120K       | $0.002–0.005        | $0.50–1.50           |
| Proxy cancel                  | 40–60K        | $0.001–0.002        | $0.25–0.60           |

Gas estimates assume Base L2 gas price of 0.001–0.005 gwei and Ethereum mainnet gas price of 20–60 gwei.

---

## ERC Standards Checklist

### Required (v1)

| Standard | Purpose                                                              |
| -------- | -------------------------------------------------------------------- |
| ERC-20   | Token interface for vault shares and base assets                     |
| ERC-721  | Identity NFTs for ERC-8004 agent registration                        |
| ERC-4337 | Account abstraction for smart wallet operations                      |
| ERC-4626 | Tokenized vault standard for deposits, withdrawals, share accounting |
| ERC-7265 | Circuit breakers for automated risk intervention                     |
| ERC-7540 | Asynchronous redemption for queued withdrawals                       |
| ERC-7579 | Session keys for bounded agent authorization                         |
| ERC-7677 | Paymaster for gasless onboarding                                     |
| ERC-7683 | Cross-chain intents (initial support)                                |
| ERC-8004 | Agent identity and reputation registry                               |
| EIP-7702 | EOA delegation for account abstraction migration                     |

### Deferred (post-v1)

| Standard | Purpose                     | Reason for Deferral                    |
| -------- | --------------------------- | -------------------------------------- |
| ERC-6900 | Modular accounts            | Complexity; ERC-7579 sufficient for v1 |
| ERC-7683 | Full cross-chain settlement | Single-chain focus in v1               |
| ERC-7821 | Minimal batch executor      | Evaluate need post-launch              |
| ERC-6551 | Token-bound accounts        | Alternative identity model; evaluate   |

---

## Subsystem Specifications

### Bardo Sanctum

| Property            | Value                                            |
| ------------------- | ------------------------------------------------ |
| Total tools         | ~171 across 26 modules                           |
| Composable profiles | 12                                               |
| Authentication      | SIWE + OAuth 2.1                                 |
| API key tiers       | Three-tier (read, standard, admin)               |
| Transport           | stdio, SSE, Streamable HTTP                      |
| Rate limiting       | Configurable per key and per identity tier       |
| Cache TTL           | 30s (prices), 5min (pool data), 1h (static data) |

### Agent System

| Property                 | Value                                      |
| ------------------------ | ------------------------------------------ |
| Total agents             | 32 (25 core + 7 vault) across 8 categories |
| Delegation hierarchy     | 4-level                                    |
| Terminal nodes           | 4                                          |
| DAG constraint           | Acyclic (validated at registration)        |
| Maximum delegation depth | 3                                          |

### GottsLoop (Heartbeat Engine)

| Property                                | Value          |
| --------------------------------------- | -------------- |
| Minimum heartbeat interval              | 60,000ms       |
| Maximum heartbeats/day per strategy     | 1,000          |
| Typical suppression rate                | ~95%           |
| Maximum active playbook heuristics      | 30             |
| Curator consolidation cadence           | Every 50 ticks |
| ExpeL consolidation cadence             | Every 4 hours  |
| Maximum parameter change per iteration  | 10%            |
| Maximum consecutive errors before pause | 5              |
| Lane Queue priority levels              | 6              |
| Session history window                  | 200 turns      |

### Gotts Grimoire (Memory System)

| Property                      | Value                                             |
| ----------------------------- | ------------------------------------------------- |
| Memory types                  | 4 (working, episodic, semantic, procedural)       |
| Embedding dimensions          | 384 (all-MiniLM-L6-v2, 23MB INT8 quantized)       |
| Insight categories            | 12                                                |
| Maximum insights per agent    | 10,000                                            |
| Maximum episodes per agent    | 100,000                                           |
| SQLite query latency          | Sub-1ms                                           |
| LanceDB vector search latency | Sub-25ms                                          |
| Confidence updates            | Asymmetric (+0.10 optimistic / −0.15 pessimistic) |
| Drift detection               | ADWIN with Hoeffding bounds at p < 0.01           |

### Safety System

| Property                        | Value                                                        |
| ------------------------------- | ------------------------------------------------------------ |
| Architecture                    | Three concentric rings (preventive, cryptographic, reactive) |
| Defense layers                  | 15 (7 preventive + cryptographic enforcement + reactive)     |
| Default per-transaction limit   | $10,000                                                      |
| Default per-session limit       | $50,000                                                      |
| Default per-day limit           | $100,000                                                     |
| Proxy delay tiers               | 5 (0s, 10min, 1h, 24h, 48h)                                  |
| Circuit breaker levels          | 3 (3% / 7% / 13% drawdown)                                   |
| Multi-layer defense requirement | Independent bypass of all 7 preventive layers                |

---

## Cross-References

| Topic                  | Document                           |
| ---------------------- | ---------------------------------- |
| Vault contracts        | `../08-vault/01-contracts.md` (ERC-4626 Solidity contracts: AgentVaultCore, AgentVaultFactory, FeeModule, NAVAwareHook)      |
| Safety architecture    | `../10-safety/00-defense.md` (15-layer defense model with three concentric rings: preventive, cryptographic, reactive)       |
| Sanctum tools          | `../07-tools/01-architecture.md` (tool library architecture: profiles, transports, safety pipeline, caching) |
| Agent system           | `../01-golem/01-cognition.md` (T0/T1/T2 cognitive tier routing and LLM-driven decision-making)      |
| Memory system          | `../04-memory/01-grimoire.md` (Grimoire persistent knowledge base: episodes, insights, heuristics, causal links)      |
| Compute infrastructure | `../11-compute/00-overview.md` (Fly.io two-app topology, x402 billing, VM tiers and warm pool)     |
| Chain configuration    | `../shared/port-allocation.md` (canonical port assignments for local development services)     |
