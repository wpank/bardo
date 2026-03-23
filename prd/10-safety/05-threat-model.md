# Threat Model: Adversary Taxonomy, Attack Trees, and Residual Risk [SPEC]

> **Crates**: `bardo-policy`, `bardo-vault`
>
> **Depends on**: [00-defense.md](00-defense.md) (six-layer defense architecture), [01-custody.md](01-custody.md) (custody architecture), [02-policy.md](02-policy.md) (PolicyCage), [04-prompt-security.md](04-prompt-security.md) (prompt injection defense). Optional: [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) (time-delayed proxy)

> **Reader orientation:** This document is the structured threat model for the Bardo system, the Rust runtime that compiles and governs Golems (mortal autonomous DeFi agents managing real capital). It belongs to the **Safety** layer and defines attacker types, enumerates attack paths, maps each to mitigating defense layers, and maintains a residual risk register. The key concept before diving in: each attack path is mapped against Bardo's six defense layers (from system prompt through on-chain PolicyCage), so you can see exactly which layers stop which attackers. Terms like Golem, Grimoire, PolicyCage, and Clade are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

This document provides a structured threat model for the Bardo runtime and vault infrastructure. It defines adversary types, enumerates attack paths, maps each to mitigating safety layers, and maintains a residual risk register for attacks that are not fully mitigated.

---

## 1. Attacker Taxonomy

| Adversary                      | Motivation                                                        | Capabilities                                                                  | Examples                                                |
| ------------------------------ | ----------------------------------------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------- |
| **Malicious vault creator**    | Steal depositor funds via vault manipulation                      | Full control of vault parameters at creation; deceptive strategy descriptions | Honeypot vaults, rug-pull via parameter changes         |
| **Compromised manager agent**  | Drain vault via unauthorized transactions                         | Prompt-injected or key-compromised agent with manager role                    | AIXBT hack ($106K), agent wallet compromise             |
| **External MEV bot**           | Extract value from vault operations                               | Mempool observation, sandwich attacks, JIT manipulation                       | Standard MEV on deposit/withdraw/rebalance              |
| **Prompt-injected agent**      | Execute unauthorized operations via LLM manipulation              | Indirect prompt injection via data feeds, tools, or context corruption        | ClawHub campaign (335 malicious skills), CVE-2025-59944 |
| **Oracle manipulator**         | Inflate NAV for profitable withdrawal or deflate for cheap shares | Flash loan price manipulation, oracle feed corruption                         | Flash-loan-based NAV inflation                          |
| **Griefing attacker**          | Deny service or degrade performance without direct profit         | Spam transactions, dust deposits, registry pollution                          | ERC-4626 inflation attack, withdrawal queue spam        |
| **Identity thief**             | Use stolen ERC-8004 identity for high-tier vault access           | Stolen wallet keys, social engineering of identity NFT transfer               | Identity NFT theft for Sovereign-tier access            |
| **Compromised infrastructure** | Extract keys or manipulate execution environment                  | Physical access to TEE hardware, compromised cloud VM                         | TEE.Fail, Battering RAM ($50 hardware attacks)          |
| **Knowledge poisoner**         | Corrupt agent reasoning via manipulated Grimoire entries          | Marketplace listing of poisoned strategies, Clade infiltration                | AgentPoison, MINJA, MemoryGraft attacks                 |
| **Coordination attacker**      | Exploit multi-agent coordination for fund extraction              | Colluding InfoAgents, malicious JudgeAgents, evaluator ghosting               | ERC-8033 oracle manipulation, ERC-8183 escrow abuse     |

---

## 2. Attack Trees

### 2.1 Malicious Vault Creator

```
Goal: Steal depositor funds
|
+-- Path A: Deploy vault with hidden extraction mechanism
|   +-- Set deceptive disclosure (strategy hash points to fake document)
|   |   Mitigated: Mandatory disclosure; depositors verify on-chain
|   +-- Configure extreme fees (100% performance fee)
|   |   Mitigated: Immutable FeeModule caps (5% mgmt, 50% perf)
|   +-- Create circular meta-vault dependency
|       Mitigated: Factory circularity check (depth limit 2)
|
+-- Path B: Change parameters post-creation to extract funds
|   +-- Lower reputation requirements to allow colluding agents
|   |   Mitigated: Parameter changes require timelock (PolicyCage or optional Warden)
|   +-- Add malicious adapter to siphon funds
|   |   Mitigated: Adapter add requires High-tier delay (24 hours)
|   +-- Modify hook logic to redirect fees
|       Mitigated: Hook logic immutable post-deployment
|
+-- Path C: Abandon vault with depositor funds locked
    +-- Stop rebalancing, let positions decay
    |   Mitigated: am-AMM allows competitive takeover; forceDeallocate exits
    +-- Refuse to process withdrawals
        Mitigated: ERC-4626 withdraw is permissionless; circuit breaker enables emergency mode
```

### 2.2 Compromised Manager Agent

```
Goal: Execute unauthorized transactions
|
+-- Path A: Direct key compromise
|   +-- Steal agent's private key
|   |   Mitigated: Layer 1 (TEE key management); keys never in agent memory
|   +-- Compromise TEE environment
|       Mitigated: TEE is defense-in-depth; Layer 4 (time-delayed proxy)
|
+-- Path B: Prompt injection
|   +-- Inject via tool response
|   |   Mitigated: Layer 2.5 (Tool Integrity Verification, 96% detection)
|   +-- Inject via data feed (token names, vault metadata)
|   |   Mitigated: Layer 2 (data/decision separation, CaMeL dual-LLM)
|   +-- Inject via memory corruption
|       Mitigated: Layer 2.5 (memory integrity hashing)
|
+-- Path C: Authorized but harmful operations
    +-- Execute trades with excessive slippage
    |   Mitigated: Layer 6 (pre-flight simulation); Layer 7 (on-chain slippage caps)
    +-- Rebalance into unfavorable positions
    |   Mitigated: Layer 4 (time delay for elevated operations); Layer 5 (cancel authority)
    +-- Drain vault via many small transactions under limits
        Mitigated: Per-day aggregate caps; continuous dampening
```

### 2.3 Oracle Manipulator

```
Goal: Manipulate NAV for profitable deposit/withdrawal
|
+-- Path A: Flash-loan price manipulation
|   +-- Manipulate V4 pool spot price
|   |   Mitigated: No-spot-assumptions policy; TWAP validation (10-30 min window)
|   +-- Manipulate external oracle feed
|       Mitigated: Multi-oracle aggregation; 2% divergence auto-pause
|
+-- Path B: Oracle staleness exploitation
|   +-- Wait for oracle to go stale, exploit stale NAV
|   |   Mitigated: Staleness gates widen spreads; 100% staleness disables NAV pricing
|   +-- Front-run oracle update with foreknowledge
|       Mitigated: NAV rate-of-change clamp (50 bps max per snapshot)
|
+-- Path C: NAV inflation via donation
    +-- Donate tokens directly to vault contract
    |   Mitigated: Internal asset accounting (not balanceOf); virtual shares offset
    +-- Donate via intermediary contract
        Mitigated: Internal accounting ignores external balance changes
```

### 2.4 Griefing Attacker

```
Goal: Deny service or degrade performance
|
+-- Path A: ERC-4626 inflation attack
|   Mitigated: Virtual shares offset (_decimalsOffset of 3-6)
|
+-- Path B: Withdrawal queue spam
|   +-- Submit many small withdrawal requests
|   |   Mitigated: Minimum withdrawal amount; gas costs make dust unprofitable
|   +-- Cancel and resubmit repeatedly
|       Mitigated: Request cooldown period; reputation impact
|
+-- Path C: Factory registry pollution
|   +-- Deploy thousands of empty vaults
|   |   Mitigated: ERC-8004 registration cost; vault creation gas cost
|   +-- Deploy vaults with misleading names
|       Mitigated: Mandatory disclosure; off-chain curation by aggregators
|
+-- Path D: Identity spam (Sybil)
    +-- Register many ERC-8004 identities
    |   Mitigated: Registration bond ($100); MeritRank transitivity decay
    +-- Transfer identity NFTs rapidly
        Mitigated: Reputation decay on transfer (30-day recovery); velocity signal detection
```

### 2.5 Knowledge Poisoning

```
Goal: Corrupt agent reasoning via manipulated Grimoire entries
|
+-- Path A: Marketplace poisoning
|   +-- List poisoned strategy with optimized triggers (AgentPoison)
|   |   Mitigated: Stage 2 Layer 1 (TrustRAG anomaly detection); Stage 3 (sandbox)
|   +-- List entries that individually pass but collectively contradict
|       Mitigated: Batch validation cross-entry contradiction check
|
+-- Path B: Clade infiltration
|   +-- Share poisoned entries from high-confidence Clade member
|   |   Mitigated: Even trusted members pass Stage 2 Layer 2 (A-MemGuard)
|   +-- Gradual behavioral drift via subtle bias (MemoryGraft)
|       Mitigated: Stage 4 causal rollback + dual memory lessons
|
+-- Path C: Adversarial injection through interactions (MINJA)
    +-- Normal-seeming interactions that poison retrieval
        Mitigated: Multi-layer consensus validation (2-of-3 threshold)
```

### 2.6 Multi-Agent Coordination Attacks

```
Goal: Exploit coordination protocols for fund extraction
|
+-- Path A: Malicious JudgeAgent (ERC-8033)
|   +-- Deliberately misclassify correct InfoAgents
|   |   Mitigated: Dispute mechanism; higher-reputation dispute JudgeAgent re-evaluates
|   +-- Collude with InfoAgents to approve dishonest answers
|       Mitigated: JudgeAgent drawn from separate pool; post-hoc dispute by any party
|
+-- Path B: InfoAgent collusion (ERC-8033)
|   +-- All N InfoAgents submit same manipulated answer
|       Mitigated: N >= 5 from diverse operators; JudgeAgent cross-references on-chain data
|
+-- Path C: Evaluator ghosting (ERC-8183)
|   +-- Provider does work, evaluator never calls complete/reject
|       Mitigated: Short expiry windows; smart contract evaluators; reputation tracking
|
+-- Path D: ERC-8001 execution revert
    +-- All participants signed, but execution fails (price moved, liquidity gone)
        Mitigated: conditionsHash encodes runtime preconditions checked before execution
```

### 2.7 Delegation-Mode Threat Vectors

In Delegation mode (MetaMask Smart Account), the Golem holds a bounded session key, not the owner's private key. This changes the threat model:

```
Goal: Exploit delegation-mode session key
|
+-- Path A: Session key compromise
|   +-- Extract session key from memory
|   |   Mitigated: Key material in TEE; TaintedString with zeroize on drop
|   +-- Steal session key via prompt injection
|   |   Mitigated: Keys never enter LLM context (WalletSecret taint label)
|   +-- Compromise bounded — attacker constrained by delegation caveats
|       Even with the key, attacker can only:
|       - Spend within maxSpend caveat (e.g., $1000/day)
|       - Interact with approved assets only
|       - Execute within time window (delegation expires)
|       - Cannot escalate privileges (attenuation invariant)
|
+-- Path B: Caveat enforcer bug
|   +-- GolemPhaseEnforcer allows action in wrong phase
|   |   Mitigated: Enforcer is immutable post-deploy; fuzz tested
|   +-- Time window enforcer miscalculates expiry
|       Mitigated: Simple block.timestamp comparison; formal verification target
|
+-- Path C: VitalityOracle manipulation
    +-- Attacker calls updatePhase to set Golem to Thriving
    |   Mitigated: onlyGolem modifier — only the Golem's own address can update
    +-- Golem's runtime compromised, reports false phase
        Mitigated: Phase can only increase (no resurrection); worst case is
        premature conservation/terminal, which reduces permissions, not expands
```

### 2.8 Three-Mode Custody Threat Comparison

| Threat | Delegation (MetaMask) | Embedded (Privy TEE) | LocalKey (dev only) |
|---|---|---|---|
| Key extraction | Bounded by caveats even if extracted | TEE isolation; $50 hardware attack [VANBULCK-2026] | Full exposure (dev only) |
| Permission escalation | Impossible (attenuation invariant on-chain) | Requires TEE compromise + policy bypass | Full access to signing key |
| Owner revocation | Instant (disable delegation on-chain) | Privy API revocation (trust Privy liveness) | Manual key rotation |
| Auditability | Full on-chain delegation chain | Trust Privy logs | Local logs only |
| Session key rotation | New delegation hash; old auto-expires | Privy rotates internally | Manual |
| Survivability | On-chain; survives provider shutdown | Depends on Privy uptime | Local only |

Delegation mode is the recommended production configuration. Embedded mode is acceptable for owners who prefer managed infrastructure. LocalKey mode is for development and testing only — it provides no key isolation.

### 2.9 Styx Compromise Scenarios

Styx is the shared knowledge commons. A compromised Styx instance could:

- **Inject poisoned knowledge** — Mitigated by the 4-stage ingestion pipeline (03-ingestion.md). All Styx entries enter at Stage 1 quarantine.
- **Deny knowledge access** — Mitigated by local Grimoire. Golems operate from local state; Styx is additive, not required.
- **Deanonymize contributions** — Mitigated by content hashing. Styx entries are identified by content hash, not author address. Attribution uses EIP-712 signatures that can be verified without revealing the signer publicly.

### 2.10 Venice Privacy Attack Vectors

Venice (inference provider with privacy claims) introduces specific attack surfaces:

- **Inference surveillance** — Venice claims to not retain prompts, but the Golem cannot verify this. Mitigated by Bardo Inference proxy which strips non-essential context before provider-bound requests.
- **Traffic analysis** — Request timing and size patterns can reveal strategy activity even without content access. Mitigated by semantic caching (repeated queries never reach provider) and request batching.
- **Provider-side model replacement** — A compromised Venice endpoint could serve a different model that systematically biases trading decisions. Mitigated by response validation against expected schema shapes and by the PolicyCage (which does not care what the LLM says, only what the transaction does).

### 2.11 Five Leakage Vectors

Five categories of owner data leakage, from the Secrets moat analysis:

| Vector | What Leaks | Mitigation |
|---|---|---|
| **API key exfiltration** | Service credentials, provider keys | Golems never handle raw API keys. Payment via x402 wallet-native signing or prepaid balance. Keys never leave TEE. |
| **Context window leakage** | Portfolio, strategy, recent decisions | Context Governor assembles from ContextBundle categories, not raw history. `bardo-result-filter` sanitizes tool results. Two-layer tool model means no external MCP servers see context. |
| **On-chain fingerprinting** | Strategy type, risk profile, social graph | Warden time-delay obscures decision-to-transaction timing. Flashbots Protect on Base. PolicyCage slippage bounds limit sandwich profitability. Honest about limits: on-chain txs are public. |
| **Inference surveillance** | Full reasoning context sent to provider | Bardo Inference proxy strips non-essential context. Semantic caching prevents repeated queries from reaching provider. x402 per-request payment means no account relationship. |
| **Memory poisoning** | Persistent behavioral drift | Grimoire immune memory, confidence scoring with demurrage, causal rollback. Curator cycle validates every 50 ticks. |

---

## 3. Residual Risk Register

Attacks that are NOT fully mitigated by current defenses:

| Risk ID | Description                                      | Severity | Current Mitigation                                    | Residual Exposure                                               | Planned Future Mitigation                             |
| ------- | ------------------------------------------------ | -------- | ----------------------------------------------------- | --------------------------------------------------------------- | ----------------------------------------------------- |
| RR-1    | Prompt injection bypass (12% rate per Anthropic) | High     | Layer 2 + 2.5 (88% catch rate)                        | 12% of sophisticated attacks may bypass prompt defenses         | CaMeL capability tokens; multi-agent defense pipeline |
| RR-2    | TEE hardware compromise                          | Medium   | TEE as defense-in-depth; Layer 4 (time-delayed proxy) | Compromised TEE + monitoring failure = key extraction possible  | HSM/KMS for production keys; proxy module completion  |
| RR-3    | Novel ERC-8004 attack vectors                    | Medium   | Adapter pattern absorbs interface changes             | ERC-8004 is still in Draft; unknown attack surfaces             | SBT milestone locks, behavioral anomaly detection     |
| RR-4    | Cross-vault contagion                            | Low      | Depth limit 2; factory-level insurance (deferred)     | Meta-vault A holds shares in vault B; B's failure impacts A     | Cross-vault insurance; composition depth enforcement  |
| RR-5    | Oracle consensus failure                         | Low      | Multi-oracle aggregation; no-spot-assumptions         | If all oracles fail simultaneously, NAV falls back to idle-only | Fee-implied vol as zero-cost backup oracle            |
| RR-6    | Regulatory action against ERC-8004               | Low      | Adapter pattern allows registry swap                  | Regulatory prohibition would break identity gating              | Alternative identity providers; adapter routing       |
| RR-7    | Smart contract bug in vault core                 | High     | Audit; invariant tests; fork tests                    | Pre-audit code has unknown vulnerability probability            | Formal verification; bug bounty program               |
| RR-8    | Knowledge poisoning via novel attack             | Medium   | 4-stage ingestion pipeline                            | New attack classes not in adversarial test suite                | Continuous adversarial testing; literature monitoring |
| RR-9    | Coordination protocol liveness failure           | Low      | Over-provisioning; short expiry windows               | Time-sensitive operations may fail if quorum not reached        | Pre-registered monitoring pools with standing bonds   |
| RR-10   | Novel behavioral attack class                    | Medium   | HDC anomaly detection, MIDAS-R edge analysis          | Attack patterns not in anti-pattern library pass undetected     | Continuous anti-pattern library updates via AntiKnowledge |
| RR-11   | MEV extraction on swap/LP strategies             | Medium   | Flashbots Protect, PolicyCage slippage bounds         | Private mempool leaks, builder-proposer collusion               | Intent-based execution (CowSwap), SUAVE integration  |

---

## 4. Threat-to-Layer Mapping

| Adversary                  | Primary Defense Layers                           | Secondary Defense Layers                   |
| -------------------------- | ------------------------------------------------ | ------------------------------------------ |
| Malicious vault creator    | 7 (on-chain guards), 3 (policy engine)           | 9 (reputation), 0 (identity)               |
| Compromised manager agent  | 4 (time-delayed proxy), 5 (cancel authority)     | 1 (TEE), 3 (policy engine), 6 (simulation) |
| External MEV bot           | 7 (on-chain slippage caps), 10 (circuit breaker) | Launch fee hook, TWAMM rebalancing         |
| Prompt-injected agent      | 2 (prompt defense), 2.5 (tool integrity)         | 4 (time delay), 5 (cancel authority)       |
| Oracle manipulator         | 10 (NAV circuit breaker), no-spot-assumptions    | Staleness gates, multi-oracle aggregation  |
| Griefing attacker          | 0 (identity gate), 7 (on-chain guards)           | Gas costs, minimum amounts                 |
| Identity thief             | 0 (reputation decay on transfer)                 | Velocity signal detection, 9 (reputation)  |
| Compromised infrastructure | 4 (time-delayed proxy), 1 (TEE as depth)         | 3 (policy engine), 5 (cancel authority)    |
| Knowledge poisoner         | 4-stage ingestion pipeline                       | Causal rollback, reputation feedback       |
| Coordination attacker      | ERC-8033 dispute mechanism                       | Reputation tracking, over-provisioning     |

---

## 5. Formal Safety Analysis

### 5.1 Maximum Loss Bounds

Under any single-layer failure, maximum loss is bounded by:

```
max_loss = min(sessionKeyLimit, maxRebalanceSizeBps × TVL, dailyAggregateCap)
```

Under simultaneous multi-layer failure (estimated probability < 10⁻⁶ per year), the circuit breaker hierarchy provides automated intervention. No single component failure produces unbounded losses — the PolicyCage (Layer 7) enforces hard smart contract boundaries that no off-chain attack can bypass.

### 5.2 Attack Cost Economics

| Attack Vector                    | Min Cost             | Detection Time      | Primary Defense                                 | Secondary Defense                             |
| -------------------------------- | -------------------- | ------------------- | ----------------------------------------------- | --------------------------------------------- |
| Prompt injection                 | $0                   | < 1 second          | Layer 2 (dual-LLM) + Layer 2.5 (tool integrity) | Layer 4 (time-delayed proxy)                  |
| TEE compromise                   | ~$50 [VANBULCK-2026] | Varies              | Layer 4 (time-delayed proxy)                    | Layer 3 (policy engine), Layer 7 (PolicyCage) |
| Oracle manipulation (flash loan) | $10K+ (loan + gas)   | < 5 seconds         | Layer 10 (circuit breaker), no-spot-assumptions | Multi-oracle, TWAP validation (10–30 min)     |
| Sybil attack                     | $100+ per identity   | 1–30 days           | Layer 0 (identity gate), registration bond      | MeritRank transitivity decay                  |
| Memory poisoning (slow drift)    | $0                   | 1–6 hours           | 4-stage ingestion pipeline                      | Periodic hash verification, causal rollback   |
| Multi-vector coordinated         | ~$150+               | < 1 second to hours | Layer 7 (PolicyCage — immutable)                | All reactive layers (4, 5, 6, 8)              |

The multi-vector attack — simultaneously compromising the TEE, injecting the prompt, and poisoning memory — still produces negative expected return because PolicyCage bounds (Layer 7) are enforced by immutable smart contract code that no off-chain attack can bypass. Maximum extractable value is bounded by `min(sessionKeyLimit, maxRebalanceSizeBps × TVL, dailyAggregateCap)`, and extraction must survive the proxy cancellation window.

### 5.3 Time-to-Detection Analysis

| Attack Type                | Detection Mechanism                     | Time-to-Detect | Delay Sufficient?                      |
| -------------------------- | --------------------------------------- | -------------- | -------------------------------------- |
| Unauthorized contract call | MonitorBot whitelist check              | < 1 second     | Yes (any tier)                         |
| Excessive slippage trade   | Pre-flight simulation divergence        | < 5 seconds    | Yes (any tier)                         |
| Gradual strategy drift     | Circuit breaker drawdown threshold      | 1–24 hours     | Yes (High/Critical)                    |
| Memory poisoning           | Periodic hash verification (6h default) | 1–6 hours      | Partial (Standard: no; Elevated+: yes) |
| Sybil reputation gaming    | On-chain milestone validation           | 1–30 days      | N/A (prevented by time requirements)   |

Standard-tier delays (10 minutes) are sufficient for automated detection of most attack classes, but insufficient for slow-drift attacks. This motivates requiring Elevated or High tier delays for manager operations.

### 5.4 Failure Scenario Analysis

| Failure Scenario                    | Impact                             | Defense                                                  |
| ----------------------------------- | ---------------------------------- | -------------------------------------------------------- |
| LLM fully compromised               | Agent reasoning compromised        | TEE policy rejects unauthorized calldata                 |
| Prompt injection (Layer 2)          | Agent reasoning compromised        | Pre-flight simulation catches state divergence           |
| Tool integrity bypass (Layer 2.5)   | Fabricated tool responses accepted | Pre-flight simulation catches state divergence           |
| Proxy monitoring downtime (Layer 5) | Cancel authority unavailable       | On-chain guards still enforce allowlists                 |
| Oracle failure (all feeds)          | NAV pricing disabled               | Constant-product fallback; withdrawals at last valid NAV |
| Memory corruption                   | Strategy quality degraded          | PolicyCage prevents execution outside bounds             |

### 5.5 Cross-Protocol Contagion Monitoring

Two cross-protocol contagion monitoring metrics feed adaptive risk thresholds in the RiskEngine:

**DeFi Correlation Fragility Indicator (CFI)** [ZHANG-2026]: Derived from time-varying protocol TVL correlations using a sliding window over on-chain data. CFI measures how correlated protocol TVLs have become -- high correlation indicates systemic fragility where a shock to one protocol propagates to others. The RiskEngine uses CFI to adjust circuit breaker thresholds: when CFI is elevated (protocols moving in lockstep), drawdown thresholds tighten from 13%/7%/3% to 10%/5%/2%.

**Aggregated Systemic Risk Index (ASRI)** [FARZULLA-2026]: Provides a unified DeFi-TradFi risk measurement by combining on-chain protocol metrics with traditional market indicators. ASRI detected historical crises with 18-day lead time in backtesting. When ASRI exceeds a configurable threshold (default 0.7), the RiskEngine triggers automatic deallocation from the most correlated adapters, reducing exposure to systemic contagion before it materializes.

Both metrics operate at Layer 10 (circuit breaker) and Layer 7 (on-chain guards) of the defense model. They complement per-vault risk monitoring with system-wide awareness -- a vault may be performing well individually while sitting in the blast radius of a systemic event.

### 5.6 Conformal Prediction for Value at Risk

The RiskEngine uses conformal prediction for Value at Risk (VaR) estimation, providing distribution-free coverage guarantees. Unlike parametric VaR (which assumes normal returns) or historical simulation (which requires stationary distributions), conformal prediction provides valid coverage at the desired confidence level regardless of the underlying distribution.

This matters for DeFi because return distributions are heavy-tailed and non-stationary -- parametric VaR systematically underestimates tail risk during regime transitions, and historical simulation fails when the distribution shifts (which is exactly when accurate VaR matters most). Conformal prediction sidesteps both problems: given a calibration set and a desired coverage level (e.g., 95%), it produces prediction intervals that provably contain the true value with at least the specified probability, with no distributional assumptions.

The implementation follows Fantazzini (2024) for crypto-asset VaR calibration and Kato (2024) for portfolio-level conformal prediction (arXiv:2410.16333). Coverage is verified daily against realized returns, and the calibration set is updated weekly with a 90-day rolling window.

---

## 6. Degradation Hierarchy

Five degradation levels govern system behavior under increasing stress. The non-custodial exit guarantee is preserved at every level.

| Level | State          | Trigger                                      | Allowed Operations                         | Non-Custodial Exit |
| ----- | -------------- | -------------------------------------------- | ------------------------------------------ | ------------------ |
| 1     | Full Operation | Normal                                       | All operations                             | Yes                |
| 2     | Degraded       | Circuit breaker warning threshold            | Reduced rate limits, heightened monitoring | Yes                |
| 3     | Limited        | Circuit breaker triggered (13% drawdown)     | Read-only + withdrawals only               | Yes                |
| 4     | Emergency      | Multi-layer failure detected                 | Withdrawals only, new deposits blocked     | Yes                |
| 5     | Close          | Irrecoverable failure or governance decision | `forceDeallocate()` only                   | Yes                |

---

## 7. Comparative Benchmarking

### 7.1 Protocol Comparison

| Protocol   | Safety Model                                                      | Key Recovery                           | Time-to-Detection      | Max Single-Event Loss                                                |
| ---------- | ----------------------------------------------------------------- | -------------------------------------- | ---------------------- | -------------------------------------------------------------------- |
| Morpho     | Curator curation, risk oracle                                     | None (EOA)                             | Manual review          | Full TVL (no reactive defense)                                       |
| Yearn V3   | Strategy review, profit unlock                                    | None (multisig)                        | Manual review          | Full strategy allocation                                             |
| Sommelier  | Governance-curated, validator veto                                | Validator set                          | Validator monitoring   | Cellar TVL (veto window)                                             |
| Eigenlayer | AVS staking, slashing                                             | Operator registration                  | AVS monitoring         | Operator stake                                                       |
| Bardo      | 15-layer defense-in-depth (preventive + cryptographic + reactive) | Multi-path (session, guardian, social) | < 1 second (automated) | `min(sessionKeyLimit, maxRebalanceSizeBps × TVL, dailyAggregateCap)` |

Distinguishing feature: the combination of reactive defense (time-delayed proxy, absent in Morpho/Yearn/Sommelier) with identity-gated access control (ERC-8004, absent in all comparators).

### 7.2 Empirical Calibration Against DeFi Exploit Database

Validated against Zhou et al.'s DeFi exploit database (181 attacks through 2023) [ZHOU-2023]:

| Attack Category     | Defense                                                                                                 | Coverage                |
| ------------------- | ------------------------------------------------------------------------------------------------------- | ----------------------- |
| Oracle manipulation | No-spot-assumptions policy, multi-oracle, TWAP validation (10–30 min windows), 2% divergence auto-pause | Strong                  |
| Access control      | Layer 7 (on-chain guards), Layer 3 (TEE policy engine), Layer 0 (ERC-8004 gate)                         | Strong                  |
| Reentrancy          | Checks-Effects-Interactions pattern, proxy announce-wait-execute separation                             | Strong                  |
| Logic errors        | Layer 6 (pre-flight simulation), Layer 8 (post-trade verification)                                      | Partial (residual risk) |
| Flash loan attacks  | Virtual shares offset, linear profit unlock, internal asset accounting (not `balanceOf`)                | Strong                  |

Logic errors remain the primary residual risk, addressable through formal verification and auditing.

### 7.3 Referenced Security Incidents

| Incident                        | Loss                   | Root Cause                                     | Bardo Defense                                             |
| ------------------------------- | ---------------------- | ---------------------------------------------- | --------------------------------------------------------- |
| AIXBT agent wallet compromise   | $105K                  | Dashboard access control failure               | Layer 1 (TEE key isolation), Layer 4 (time-delayed proxy) |
| Cork Protocol                   | ~$12M                  | Missing `onlyPoolManager` on V4 hook callbacks | Layer 13 (hook security), factory deployment controls     |
| Bunni v2                        | ~$8.4M                 | Precision bugs in withdrawal rounding          | Virtual shares offset, OZ ERC4626Upgradeable              |
| ClawHub campaign (Koi Security) | 341 malicious skills   | Supply chain poisoning via skill marketplace   | Layer 2.5 (tool integrity), 4-stage ingestion pipeline    |
| BlockSec V4 hook audit          | 36% vulnerability rate | Exploitable hooks in 22-project sample         | Factory-deployed hooks, invariant test suite              |

---

## 8. Operational Risk Register

Beyond adversarial attacks, the protocol faces operational and market risks.

### 8.1 Smart Contract Risk

The system deploys real vaults holding real USDC. A bug means user funds at risk. The October 2025 flash crash (USDe at $0.65 on Binance, $20B in positions liquidated) shows what happens when composability breaks.

**Mitigation**: v1 scope is deliberately minimal -- two lending adapters (Morpho supply, Aave supply). These are the most battle-tested DeFi interactions. No complex composability in v1. OZ ERC4626Upgradeable with virtual shares offset of 6. All adapters implement `forceDeallocate()` for guaranteed non-custodial exits.

### 8.2 Yield Compression Risk

If the Fed cuts further, DeFi lending rates may compress below 3% for stablecoins.

**Mitigation**: Strategy compiler adapts to market conditions. When lending yields are low, it favors fixed rates, leveraged loops, and directional plays. Event-driven strategies create alpha regardless of baseline rate levels.

### 8.3 Agent Misbehavior Risk

An agent could execute a strategy incorrectly or be manipulated via prompt injection.

**Mitigation**: 15-layer defense-in-depth architecture. `safety-guardian` is a terminal node that gates every write operation. Circuit breakers halt at 13% drawdown. Spending limits per transaction ($10K default), per session ($50K), and per day ($100K). PolicyCage enforces hard smart contract boundaries that cannot be bypassed by prompt injection.

### 8.4 MEV Exposure

Agent transactions on Base are exposed to sandwich attacks on swaps and frontrunning on large deposits.

**Mitigation**: (1) v1 lending strategies have negligible MEV surface. (2) Future swap strategies use aggregators with MEV protection. (3) Flashbots Protect on Base for transaction submission. (4) PolicyCage enforces maximum slippage bounds on-chain.

---

## 9. Incident Response Matrix

| Scenario                                      | Impact                       | Response                                                                                  | Recovery Time                          |
| --------------------------------------------- | ---------------------------- | ----------------------------------------------------------------------------------------- | -------------------------------------- |
| Vault adapter bug (fund loss)                 | Direct fund loss             | Circuit breaker auto-pauses. `forceDeallocate()` extracts remaining value.                | Minutes (pause) to hours (remediation) |
| Fly.io outage                                 | Agent downtime, no fund risk | Vaults continue on-chain. Agents restart on recovery.                                     | Depends on provider                    |
| LLM provider outage                           | Agent cognitive degradation  | Cascade failover to next provider. Rule-only fallback for critical operations.            | Automatic (<30s failover)              |
| Agent strategy loss exceeding circuit breaker | Vault paused                 | Deposits blocked, withdrawals remain open. Owner investigates.                            | Hours to days (owner decision)         |
| ERC-8004 contract vulnerability               | Identity system compromise   | Vault deposits continue without identity gating. Emergency governance via contract owner. | Depends on severity                    |
| Knowledge poisoning detected                  | Agent reasoning corrupted    | Causal rollback triggers. Offending entries quarantined. Lessons stored.                  | Minutes (rollback) to hours (review)   |
| Multi-agent coordination failure              | Operation incomplete         | Intent transitions to EXPIRED after deadline. No capital at risk from coordination layer. | Automatic (deadline expiry)            |

---

## 10. Security Checklist

Before deploying an agent in production, verify every item:

### Identity Security

- [ ] Identity NFT held in smart contract wallet (ERC-4337 or Safe), not EOA
- [ ] Guardian enabled on identity NFT
- [ ] Safe Transaction Guard blocks `transferFrom` for Identity Registry
- [ ] Monitoring alerts configured for Transfer events involving the agent's token ID
- [ ] At least 2-of-3 guardians configured for social recovery

### Wallet Security

- [ ] Wallet policy configured -- agent cannot sign to arbitrary addresses
- [ ] Contract allowlist minimal -- only vault, USDC, and Identity Registry (or proxy only in proxy-enhanced mode)
- [ ] Method allowlist minimal -- only required function selectors
- [ ] Per-transaction and daily aggregate caps set
- [ ] Custody mode configured: Delegation (MetaMask caveats), Embedded (Privy TEE), or LocalKey (dev only)
- [ ] In Delegation mode: delegation caveats match PolicyCage constraints
- [ ] In Embedded mode: keys TEE-isolated using Privy (not raw private keys)

### Proxy Security

- [ ] Time-delayed proxy deployed and configured for manager/admin operations (optional; see prd2-extended/10-safety/02-warden.md)
- [ ] Cancel authority is separate EOA from both owner and agent
- [ ] Cancel authority key stored in KMS, not environment variables
- [ ] Monitoring bot running with redundant RPC connections (2+ providers)
- [ ] Auto-cancel rules configured for unknown targets and dangerous selectors
- [ ] Multi-channel alerting active (Telegram, Discord, or PagerDuty)

### Agent Security

- [ ] System prompt includes scope-specific guardrails
- [ ] Agent does not process untrusted external data as instructions
- [ ] Dual-LLM architecture considered for high-AUM agents (>$50K)
- [ ] Transaction simulation enabled before every write
- [ ] Ingestion pipeline strictness set to at least "standard"

### Protocol Security

- [ ] Vault is factory-deployed (verify via `factory.isVault(address)`)
- [ ] Creator reputation is non-zero
- [ ] Vault parameters are reasonable (management fee <=5%, performance fee <=50%)
- [ ] Circuit breaker enabled with `drawdownThreshold` configured
- [ ] ERC-4626 share price is sane (compare against recent history)

---

## 11. HDC-Based Behavioral Anomaly Detection

The threat-to-layer mapping in Section 4 relies on pattern matching against known attack signatures. A novel attack class that doesn't match any existing pattern can bypass all reactive defenses. Hyperdimensional Computing (HDC) with Binary Spatter Codes (D=10,240) provides a complementary detection layer that identifies behavioral anomalies without requiring explicit attack signatures.

### 11.1 Normal Behavior Encoding

The system encodes normal Golem behavior as HDC hypervectors. Each tick, the Golem's action profile (tool calls, position changes, risk assessments, CorticalState snapshot) is fingerprinted into a 1,280-byte binary vector using bind (XOR) and bundle (majority vote) operations.

```rust
/// Anti-pattern fingerprint library for behavioral anomaly detection.
/// Normal behavior patterns are accumulated into prototypes.
/// Deviations are detected via Hamming distance.
pub struct BehavioralAnomalyDetector {
    /// Per-strategy normal behavior prototypes.
    /// Accumulated from hundreds of ticks of normal operation.
    normal_prototypes: HashMap<StrategyId, Hypervector>,
    /// Anti-pattern library: known attack fingerprints.
    attack_patterns: Vec<(String, Hypervector, AntiPatternMetadata)>,
    /// Anomaly threshold: Hamming distance above which behavior is flagged.
    anomaly_threshold: f32,
    /// Rolling accumulator for the current regime's normal behavior.
    accumulator: BundleAccumulator,
    /// Number of ticks accumulated in the current prototype.
    accumulated_ticks: usize,
}

impl BehavioralAnomalyDetector {
    /// Encode a tick's behavioral profile into an HDC hypervector.
    pub fn encode_tick_behavior(
        &mut self,
        actions: &[ActionKind],
        cortical: &CorticalState,
        portfolio_delta: &PortfolioDelta,
        item_memory: &mut ItemMemory,
    ) -> Hypervector {
        // Bind each action type with its context
        let mut acc = BundleAccumulator::new();
        for (pos, action) in actions.iter().enumerate() {
            let action_hv = item_memory.get_or_create(&action.name());
            let context_hv = encode_action_context(action, cortical, item_memory);
            let bound = action_hv.bind(&context_hv).permute(pos);
            acc.add(&bound);
        }
        acc.finish()
    }

    /// Check a tick's behavior against normal prototypes and attack patterns.
    /// Returns anomaly score (0.0 = normal, 1.0 = maximally anomalous).
    pub fn check(&self, tick_hv: &Hypervector, strategy: &StrategyId) -> AnomalyResult {
        // Check against normal prototype
        let normal_distance = match self.normal_prototypes.get(strategy) {
            Some(prototype) => 1.0 - prototype.similarity(tick_hv),
            None => 0.5, // No prototype yet, neutral score
        };

        // Check against known attack patterns
        let attack_match = self.attack_patterns
            .iter()
            .filter(|(_, pattern, _)| pattern.similarity(tick_hv) > self.anomaly_threshold)
            .max_by(|(_, a, _), (_, b, _)| {
                a.similarity(tick_hv)
                    .partial_cmp(&b.similarity(tick_hv))
                    .unwrap()
            });

        AnomalyResult {
            deviation_score: normal_distance,
            is_anomalous: normal_distance > self.anomaly_threshold,
            matched_attack: attack_match.map(|(name, _, meta)| (name.clone(), meta.clone())),
        }
    }
}
```

The encoding runs at Gamma tick cadence (~10 seconds). A single Hamming distance comparison takes ~10ns (XOR + POPCNT on 10,240 bits), allowing the detector to check against hundreds of patterns in microseconds. Memory footprint is fixed: 1,280 bytes per prototype regardless of how many ticks contributed to it.

### 11.2 MIDAS-R Integration for Edge-Stream Anomaly Detection

MIDAS-R (Microcluster-Based Detector of Anomalies in Edge Streams) [BHATIA-2020] detects structural anomalies in the transaction graph that HDC's per-tick behavioral analysis would miss. Where HDC catches "this Golem is acting differently than usual," MIDAS-R catches "the transaction graph around this Golem has an unusual structure."

MIDAS-R maintains four Count-Min Sketch structures (128KB total at w=1024, d=4) tracking edge and node frequencies with temporal decay. The anomaly score for an edge (from_address, to_address) at tick t is the chi-squared statistic comparing observed to expected frequency.

```rust
/// MIDAS-R anomaly detector for transaction edge streams.
/// Detects microbursts: flash loan cascades, wash trading,
/// MEV bot coordination, and airdrop farming.
pub struct MidasR {
    edge_total: CountMinSketch,
    edge_current: CountMinSketch,
    node_total: CountMinSketch,
    node_current: CountMinSketch,
    current_tick: u64,
    decay_factor: f64,   // 0.9 per block, half-life ~7 blocks (~84s)
    threshold: f64,
}

impl MidasR {
    /// Process a new edge in the transaction graph.
    /// Returns anomaly score (higher = more anomalous).
    pub fn process_edge(&mut self, src: u64, dst: u64, tick: u64) -> f64 {
        if tick > self.current_tick {
            self.advance_tick(tick);
        }

        // Update current-tick and total counts for edges and nodes
        self.edge_current.insert_edge(src, dst, 1.0);
        self.node_current.insert_node(src, 1.0);
        self.node_current.insert_node(dst, 1.0);
        self.edge_total.insert_edge(src, dst, 1.0);
        self.node_total.insert_node(src, 1.0);
        self.node_total.insert_node(dst, 1.0);

        // Combined score: max of edge and node anomalies
        let edge_score = self.chi_squared(
            self.edge_current.query_edge(src, dst),
            self.edge_total.query_edge(src, dst),
        );
        let src_score = self.chi_squared(
            self.node_current.query_node(src),
            self.node_total.query_node(src),
        );
        let dst_score = self.chi_squared(
            self.node_current.query_node(dst),
            self.node_total.query_node(dst),
        );

        edge_score.max(src_score).max(dst_score)
    }
}
```

MIDAS-R detects:
- **Flash loan cascades**: burst of transactions between the same addresses within one block
- **Wash trading**: repeated transfers between the same pair at abnormal frequency
- **MEV bot coordination**: cluster of transactions from multiple bots targeting the same contract
- **Airdrop farming**: sudden activity from previously dormant addresses to a single contract

Bhatia et al. (2020) showed MIDAS achieves 42-52% higher AUC than prior streaming methods while processing each edge in O(1) time and O(1) memory.

### 11.3 Detection-to-Response Pipeline

HDC and MIDAS-R anomaly scores feed into the existing safety pipeline as advisory signals. They do not block execution (that remains PolicyCage's authority). The flow:

1. HDC behavioral anomaly detected (deviation_score > threshold)
2. MIDAS-R edge anomaly detected (chi-squared > threshold)
3. Either or both trigger: CorticalState arousal increases, safety_alert event emitted
4. If both fire simultaneously: escalate to T2 cognitive tier for LLM review
5. Anomaly pattern logged to Grimoire as AntiKnowledge entry for future reference

The anti-pattern library is inherited across generations as part of the AntiKnowledge Grimoire type (decay floor 0.3, always included in generational inheritance).

---

## 12. MEV Detection and Protection

Maximal Extractable Value (MEV) is profit that block proposers and searchers extract by reordering, inserting, or censoring transactions. For an autonomous agent, MEV is both a threat (transactions get sandwiched) and a signal (MEV patterns reveal market microstructure). Daian et al. [DAIAN-2020] documented Priority Gas Auctions consuming significant network resources. Zust et al. [ZUST-2021] identified 525,004 sandwich attacks over 12 months extracting 57,493 ETH (~$189M).

### 12.1 MEV Attack Taxonomy

| Attack Type | Pattern | Impact on Golem |
|---|---|---|
| **Sandwich** | Attacker front-runs and back-runs a swap, profiting from price impact | Golem's swap executes at worse price |
| **Front-running** | Attacker copies a profitable tx with higher gas priority | Golem's liquidation/arb tx fails or executes at worse price |
| **Back-running** | Attacker places tx after a large trade to capture arbitrage | Golem's trade creates value for extractors |
| **JIT liquidity** | Attacker provides concentrated LP around a pending swap, earns fees, removes LP same block | Existing LP positions (including Golem's) earn less in fees |

### 12.2 Detection Implementation

The MEV detector operates on block-level transaction data, scanning for structural patterns:

```rust
/// Core MEV detection engine.
/// Operates on a single block's worth of transactions.
pub struct MevDetector {
    min_profit_threshold: U256,
    known_bots: HashMap<Address, String>,
}

impl MevDetector {
    /// Scan a block's transactions for all MEV patterns.
    pub fn detect_all(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        let mut patterns = Vec::new();
        patterns.extend(self.detect_sandwiches(txs));
        patterns.extend(self.detect_jit_liquidity(txs));
        patterns.extend(self.detect_backruns(txs));
        patterns.extend(self.detect_arbitrage(txs));
        patterns
    }

    /// Detect sandwich attacks.
    /// Pattern: three transactions on the same pool where:
    ///   1. tx_a: swap by address X in direction D
    ///   2. tx_b: swap by address Y (victim) in direction D
    ///   3. tx_c: swap by address X in direction !D
    /// and tx_a.index < tx_b.index < tx_c.index.
    pub fn detect_sandwiches(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        let mut patterns = Vec::new();
        let mut swaps_by_pool: HashMap<Address, Vec<&MevTransaction>> = HashMap::new();

        for tx in txs {
            if let Some(ref swap) = tx.swap {
                swaps_by_pool.entry(swap.pool).or_default().push(tx);
            }
        }

        for (_pool, pool_swaps) in &swaps_by_pool {
            if pool_swaps.len() < 3 { continue; }

            for i in 0..pool_swaps.len() {
                let front = pool_swaps[i];
                let front_swap = front.swap.as_ref().unwrap();

                for j in (i + 1)..pool_swaps.len() {
                    let victim = pool_swaps[j];
                    let victim_swap = victim.swap.as_ref().unwrap();

                    if victim.from == front.from { continue; }
                    if victim_swap.zero_for_one != front_swap.zero_for_one { continue; }

                    for k in (j + 1)..pool_swaps.len() {
                        let back = pool_swaps[k];
                        let back_swap = back.swap.as_ref().unwrap();

                        if back.from != front.from { continue; }
                        if back_swap.zero_for_one == front_swap.zero_for_one { continue; }
                        if front.tx_index >= victim.tx_index
                            || victim.tx_index >= back.tx_index { continue; }

                        let profit = back_swap.amount_out
                            .saturating_sub(front_swap.amount_in);

                        if profit >= self.min_profit_threshold {
                            patterns.push(MevPattern::Sandwich(SandwichBundle {
                                attacker: front.from,
                                frontrun_tx: front.hash,
                                victim_tx: victim.hash,
                                backrun_tx: back.hash,
                                pool: front_swap.pool,
                                estimated_profit: profit,
                            }));
                        }
                    }
                }
            }
        }
        patterns
    }

    /// Detect JIT liquidity provision.
    /// Pattern: address X adds concentrated LP, a large swap executes,
    /// address X removes LP -- all in the same block on the same pool.
    pub fn detect_jit_liquidity(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        // Group liquidity events and swaps by pool, then find
        // add/remove pairs from the same address with a swap between them.
        // See full implementation in golem-verify crate.
        Vec::new() // Structural placeholder
    }
}

#[derive(Debug, Clone)]
pub enum MevPattern {
    Sandwich(SandwichBundle),
    Frontrun(FrontrunBundle),
    Backrun(BackrunBundle),
    JitLiquidity(JitBundle),
    Arbitrage(ArbitrageBundle),
}
```

### 12.3 Protection Strategies

**Private mempool submission.** All Golem transactions on Base use Flashbots Protect for transaction submission. This prevents mempool observation by searchers. The Golem submits transactions directly to block builders who commit to not front-running them.

**Slippage bounds.** PolicyCage enforces maximum slippage on-chain. Even if a sandwich attack succeeds in reordering transactions, the Golem's swap reverts if the execution price deviates beyond the configured slippage tolerance (default: 50 bps for stablecoin pairs, 200 bps for volatile pairs).

**Timing randomization.** Rebalancing operations add random delays (0-30 seconds) to break timing predictability. An attacker who knows the Golem rebalances every 720 ticks can prepare. Random jitter within a window makes preparation harder.

**Transaction splitting.** Large operations are split into smaller chunks submitted across multiple blocks. A $50,000 swap split into five $10,000 swaps reduces the MEV surface per transaction. The splitting threshold is configurable per strategy.

**Aggregator routing.** When swaps are necessary, route through aggregators (1inch, CowSwap, Paraswap) that provide MEV protection through batch auctions or intent-based execution. The aggregator handles the MEV mitigation; the Golem submits an intent rather than a direct swap.

### 12.4 MEV as Intelligence Signal

MEV detection is not only defensive. Detected MEV patterns are intelligence:

- **Sandwich frequency on a pool** indicates that large uninformed flow is present. The Golem can avoid these pools or reduce position sizes.
- **JIT liquidity patterns** reveal which pools attract sophisticated actors. High JIT activity means concentrated liquidity is competitive, reducing LP returns.
- **Arbitrage activity** indicates price discovery efficiency. Pools with high arb activity have tighter spreads but less LP revenue.
- **Back-run frequency** on a specific contract reveals its MEV surface. The Golem can assess whether interacting with that contract creates value leakage.

MEV classification results feed into the risk engine's Layer 5 (DeFi Threats) as described in [06-adaptive-risk.md](06-adaptive-risk.md).

### 12.5 v1 MEV Surface Assessment

The v1 scope (two lending adapters: Morpho supply, Aave supply) has minimal MEV surface. Lending supply operations do not involve swaps and are not subject to sandwich attacks. The primary MEV vector for v1 is deposit/withdrawal timing -- an attacker who observes a large pending deposit could front-run it to capture first-mover advantage on rate changes. This is mitigated by rate-change dampening in both Morpho and Aave and by the PolicyCage's per-transaction size limits.

Future strategies involving swaps, LP provision, or leveraged positions have substantially larger MEV surfaces and will require the full protection stack described above.

---

## Cross-References

- [00-defense.md](00-defense.md) -- The main defense-in-depth architecture doc: six defense layers, `Capability<T>` tokens, `TaintedString` information-flow control, and the DeFi Constitution prompt.
- [01-custody.md](01-custody.md) -- Three wallet custody modes (Delegation via ERC-7710/7715, Embedded via Privy TEE, LocalKey), seven caveat enforcers, session key lifecycle, and death settlement.
- [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) -- Optional Warden time-delayed proxy and MonitorBot: transactions are announced, held, then executed or cancelled (deferred to phase 2).
- [02-policy.md](02-policy.md) -- PolicyCage on-chain smart contract: asset whitelists, spending caps, drawdown breakers, position limits, and the `IPolicyCage` Solidity interface.
- [03-ingestion.md](03-ingestion.md) -- Four-stage knowledge ingestion safety pipeline (quarantine, consensus validation, sandbox, adopt) protecting the Grimoire from poisoned entries.
- [04-prompt-security.md](04-prompt-security.md) -- Prompt injection defenses: dual-LLM architecture, CaMeL capability-based authorization, Tool-Guard pattern, and MCP avoidance rationale.
- [../09-economy/00-identity.md](../09-economy/00-identity.md) -- ERC-8004 on-chain agent identity: registration, reputation scoring, tier-based deposit caps, and clade discovery.
- [../09-economy/04-coordination.md](../09-economy/04-coordination.md) -- Multi-agent coordination security: ERC-8001 (A2A protocol), ERC-8033 (oracle protocol), ERC-8183 (escrow protocol), and their trust/attack models.
- [07-temporal-logic-verification.md](07-temporal-logic-verification.md) -- LTL/CTL temporal logic verification for strategy behavior across time: 40 DeFi temporal patterns, Buchi automaton monitors, and category-theoretic composition.
- [08-witness-dag.md](08-witness-dag.md) -- Cryptographic cognitive trace DAG linking observations to predictions to decisions to outcomes, with BLAKE3 hashing and optional ZK proofs for privacy-preserving audits.

---

## References

- [VANBULCK-2026] Van Bulck, J. et al. "Battering RAM." IEEE S&P 2026. Demonstrates physical attacks against TEE memory integrity using $50 hardware. Motivates the caveat-bounded custody model over TEE-only security.
- [BADRAM-2025] De Meulemeester, J. et al. "BadRAM." IEEE S&P 2025. Shows AMD SEV-SNP memory encryption can be bypassed via physical DRAM manipulation. Another data point against relying solely on TEE for custody.
- [TEE-FAIL-2025] "TEE.Fail." ACM CCS 2025. Comprehensive catalog of TEE vulnerabilities across Intel SGX, AMD SEV, and ARM TrustZone. Informs the risk assessment for Embedded (Privy TEE) custody mode.
- [SEAGENT-2026] Ji, H. et al. "SEAgent: Confused Deputy Problem in Multi-Agent LLM Systems." arXiv:2601.11893. Shows inter-agent trust exploitation achieves 84.6% attack success vs 46.2% for direct injection. Motivates architectural separation over trust-based multi-agent communication.
- [CAMEL-2025] Debenedetti, E. et al. "CaMeL: Capability-Based Authorization for LLM Agents." arXiv:2503.18813. Separates control flow from data flow so untrusted data cannot impact program flow. Achieves 77% task completion with provable security.
- [PROGENT-2025] Shi, Y. et al. "Progent: Privilege Control for LLM Agents." arXiv:2504.11703. Proposes fine-grained privilege control restricting which tools and resources an agent can access based on context.
- [ARXIV-2503.16248] "CrAIBench: AI Agents in Cryptoland." arXiv:2503.16248. Benchmarks AI agent vulnerabilities in cryptocurrency environments; demonstrates tool interface as primary attack vector.
- [ARXIV-2512.02261] "TradeTrap." arXiv:2512.02261. Shows memory injection is more powerful than prompt injection for trading agents, producing persistent cross-session effects.
- [CHEN-2024] Chen, Z. et al. "AgentPoison." NeurIPS 2024. Optimized embedding-space triggers hijack RAG retrieval with >=80% success at <0.1% poison rate. Primary motivation for Grimoire ingestion safety.
- [DONG-2025] Dong, Z. et al. "MINJA." arXiv:2503.03704. Injection through normal interactions achieves >95% success, defeating LLM-only auditing.
- [MEMORYGRAFT-2025] "MemoryGraft." arXiv:2512.16962. Durable, trigger-free behavioral drift through gradual accumulation of subtly biased memory entries.
- [ZHOU-2023] Zhou, L. et al. "SoK: Decentralized Finance (DeFi) Attacks." IEEE S&P 2023. Systematic taxonomy of 181 DeFi attacks across protocol categories. The foundational attack classification this threat model builds on.
- [CHITRA-2025] Chitra, T. "Autodeleveraging: Impossibilities and Optimization." arXiv:2512.01112. Proves impossibility results for autodeleveraging in certain DeFi configurations. Relevant to understanding cascading liquidation risk.
- [ZHANG-2026] Zhang, L. et al. "DeFi Correlation Fragility Indicator." arXiv:2601.08540, Jan 2026. Proposes a quantitative indicator for DeFi systemic risk based on cross-protocol correlations.
- [FARZULLA-2026] Farzulla, A. et al. "Aggregated Systemic Risk Index." arXiv:2602.03874, Jan 2026. Constructs an aggregated risk index for DeFi ecosystems. Relevant to the health score computation in the adaptive risk layer.
- [FANTAZZINI-2024] Fantazzini, D. "Conformal Prediction for Crypto-Asset VaR." 2024. Applies conformal prediction to crypto VaR estimation, providing distribution-free coverage guarantees.
- [KATO-2024] Kato, M. "Conformal Predictive Portfolio Selection." arXiv:2410.16333, Oct 2024. Uses conformal prediction for portfolio selection under uncertainty. Relevant to the position sizing layer.
- [ENDORLABS-2026] Endor Labs. "Classic Vulnerabilities Meet AI Infrastructure: Why MCP Needs AppSec." January 2026. Found 82% of 2,614 MCP implementations vulnerable. Motivates compiled-tools-only approach.
- [COMPOSIO-2026] Composio. "MCP Vulnerabilities Every Developer Should Know." 2026. Practical catalog of MCP security issues including tool manifest tampering.
- [KASPERSKY-OPENCLAW-2026] Kaspersky. "New OpenClaw AI agent found unsafe for use." February 2026. Real-world case study of an AI agent framework with exploitable vulnerabilities, reinforcing the need for architectural safety.
- [STELLARCYBER-2026] Stellar Cyber. "Top Agentic AI Security Threats in Late 2026." March 2026. Industry survey of emerging agentic AI threats. Provides context for the evolving threat landscape.
- [BHATIA-2020] Bhatia, S. et al. "MIDAS: Microcluster-Based Detector of Anomalies in Edge Streams." AAAI 2020, 34(04), 3242-3249. Streaming anomaly detection algorithm. Relevant to real-time behavioral anomaly detection in the adaptive risk layer.
- [KANERVA-2009] Kanerva, P. "Hyperdimensional Computing: An Introduction." *Cognitive Computation*, 1(2), 139-159, 2009. Introduces high-dimensional distributed representations for computation. Foundation for the HDC-based transaction fingerprinting in the triage pipeline.
- [FRADY-2018] Frady, E.P., Kleyko, D., & Sommer, F.T. "Variable Binding for Sparse Distributed Representations." *IEEE TNNLS*, 2018. Extends hyperdimensional computing with variable binding operations. Used in the BSC (Binary Sparse Codes) component of the triage engine.
- [DAIAN-2020] Daian, P. et al. "Flash Boys 2.0: Frontrunning in Decentralized Exchanges, Miner Extractable Value, and Consensus Instability." IEEE S&P 2020. arXiv:1904.05234. The foundational MEV paper: frames MEV as a systemic property of transparent mempools. Directly informs the MEV threat model.
- [QIN-2022] Qin, K. et al. "Quantifying Blockchain Extractable Value: How dark is the forest?" IEEE S&P 2022. Quantifies total extractable value across Ethereum DeFi. Provides the economic context for MEV as a first-order threat.
- [ZUST-2021] Zust, P. et al. "Analyzing and Preventing Sandwich Attacks in Ethereum." ETH Zurich, 2021. Identified 525,004 sandwich attacks extracting 57,493 ETH (~$189M). The empirical basis for sandwich attack detection algorithms.
