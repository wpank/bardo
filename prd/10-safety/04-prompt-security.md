# Prompt Security and LLM Defense [SPEC]

> **Crate**: `bardo-policy`
>
> **Depends on**: [00-defense.md](00-defense.md) (six-layer defense architecture, Layer 2 type-system safety), [01-custody.md](01-custody.md) (custody architecture). Optional: [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) (time-delayed execution)

> **Reader orientation:** This document specifies defenses against prompt injection attacks on Golems (mortal autonomous DeFi agents managing real capital). It belongs to the **Safety** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: prompt injection is the #1 LLM vulnerability (OWASP LLM01:2025), and for an agent with wallet access, a 12% failure rate is catastrophic, so Bardo layers behavioral defenses (dual-LLM architecture, CaMeL capabilities) on top of the architectural safety guarantees in the cryptographic and type-system layers. Terms like PolicyCage, Grimoire, and Styx are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

Prompt injection is ranked OWASP LLM01:2025 -- the number-one security vulnerability for LLM applications. The UK's NCSC warns it may be a problem that is never fully solved. Claude's System Card reports blocking approximately 88% of prompt injections, but 12% still succeed. For an agent managing a DeFi vault, a 12% failure rate is catastrophic.

This document specifies the behavioral and architectural defenses at the prompt security and tool integrity layers. These are defense-in-depth supplements to the cryptographic enforcement in the six-layer security architecture (see [00-defense.md](00-defense.md) §8). They reduce attack success rate from 12% to near-zero when combined with the full stack.

---

## 1. The Confused Deputy Problem

The AI agent holds legitimate credentials but can be tricked into misusing them. Attackers do not need to steal keys -- they manipulate the agent's reasoning. CrAIBench research on ElizaOS demonstrated how adversaries inject malicious instructions into prompts or historical interaction records, leading to unintended asset transfers [ARXIV-2503.16248]. Critically, prompt-based defenses were found ineffective against context manipulation; only fine-tuning-based defenses and architectural separation provided meaningful protection.

SEAgent [SEAGENT-2026] identifies the confused deputy problem in multi-agent LLM systems: inter-agent trust exploitation achieves an 84.6% attack success rate versus 46.2% for direct prompt injection. The agent authorization boundary -- not the key management boundary -- is the critical attack surface.

---

## 2. Mandatory Mitigations

### 2.1 System Prompt Hardening

The agent knows its role and treats all external data as data, never as instructions. System prompt guidelines:

- Agent scope is explicitly bounded (e.g., vault operations only: deposit, withdraw, monitor)
- All on-chain data (vault names, token symbols, metadata URIs) treated as untrusted data
- Operations outside defined scope are refused regardless of argument
- No self-modification of safety constraints permitted

System prompt hardening provides approximately 88% catch rate. Insufficient alone -- the remaining 12% must be caught by deeper layers.

### 2.2 Data/Decision Separation (Dual-LLM Architecture)

Recommended for all agents managing >$50K AUM. Follows the CaMeL capability-based authorization pattern [CAMEL-2025]:

```
Untrusted External Data (vault metadata, token names, on-chain strings)
    |
    v
Sandboxed LLM (data processor)
    |  Produces sanitized summaries only
    |  Cannot make tool calls or sign transactions
    v
Privileged LLM (decision maker)
    |  Receives only sanitized summaries
    |  Makes tool-calling decisions
    v
Execution Layer (tools + signing)
```

This prevents the attack vector where malicious vault metadata or token names contain injection payloads. The sandboxed LLM has no capability tokens and cannot initiate any action. The privileged LLM never sees raw untrusted input.

### 2.3 CaMeL Capability-Based Authorization

CaMeL [CAMEL-2025] creates a capability-based security layer that separates control flow from data flow. Untrusted data can never impact program flow. Achieves 77% task completion with provable security (vs 84% undefended).

For Bardo agents, this translates to: agents receive capability tokens authorizing specific transaction types (deposit, withdraw, rebalance), amounts (within tier limits), and destinations (vault contract addresses). The capability token is a structured object independent of the LLM's reasoning -- even a fully compromised LLM cannot forge a capability it was not issued.

Beurer-Kellner et al. [BEURER-KELLNER-2025] provide six principled design patterns with a critical result: general-purpose agents with broad capabilities are inherently insecurable with current LLM technology. This validates the narrow-scope agent design -- agents implement the Action-Selector pattern (pre-defined transaction templates only) and Plan-Then-Execute pattern (fix the plan before processing any external data).

### 2.4 Formal Privilege Specification via DSL

Progent [PROGENT-2025] introduces the first privilege control framework using a domain-specific language (DSL) for fine-grained tool access policies, with formal verification via Z3 SMT solver for policy analysis. For Bardo wallets, this enables formally verified policy definitions:

```
"Agent may deposit <= $10,000 to whitelisted vault addresses without delay.
 Rebalances > $50K require 1h delay and monitoring approval."
```

These policies are compiled to both custody-layer signing policies and PolicyCage on-chain configs from a single source via the PolicyBuilder API (see [02-policy.md](02-policy.md)).

### 2.5 Multi-Agent Defense Pipeline

Research from early 2026 [ARXIV-2509.14285] demonstrates a multi-agent pipeline achieving 100% prompt injection mitigation across all tested scenarios:

1. Domain LLM generates candidate actions
2. Mandatory guard agent (independent model, independent context) vets for policy violations and attack indicators
3. Only guarded output reaches execution

The `safety-guardian` agent in the Bardo runtime serves this role -- a terminal node in the agent delegation DAG that gates every write operation before signing.

### 2.6 Mandatory Pre-Flight Simulation

Every write operation is simulated via `eth_call` against real-time blockchain state before broadcast. The simulation reveals expected balance changes. If the simulation shows unexpected fund movement, the transaction is rejected before signing.

---

## 3. Tool Integrity Verification (Layer 2.5)

The tool interface is the primary attack vector for AI agents in DeFi [ARXIV-2503.16248, ARXIV-2512.02261, ARXIV-2506.23260]. Memory injection is more powerful than prompt injection. Fake tool servers manipulate trading decisions. Prompt-based defenses are fundamentally inadequate for stored context corruption.

The zero-click RCE vulnerability in tool-calling IDEs (CVE-2025-59944) -- where a Google Docs file triggered an agent to execute a Python payload via a tool server -- demonstrates that tool-calling agents are uniquely vulnerable to indirect injection.

### 3.1 Four Tool Integrity Defenses

1. **Tool provenance signing**: Tool responses are cryptographically signed. Agents verify signatures before acting on state data.

2. **Independent state verification**: Before writes, critical state is re-read via a separate RPC endpoint. A poisoned tool server that lies about pool balances is caught because the agent independently verifies via a different RPC provider.

3. **Tool-Guard**: Three-stage detection achieving 96% accuracy on attack detection:
   - **Static analysis**: Pattern matching for known injection templates
   - **Semantic analysis**: DeBERTa v3 classifier running locally via `ort` (ONNX Runtime for Rust) -- zero inference cost, sub-millisecond latency
   - **Fine-tuned E5**: Embedding-based classifier trained on tool-call attack corpus (CrAIBench [ARXIV-2503.16248] and TradeTrap [ARXIV-2512.02261] datasets)

4. **Memory integrity hashing**: Context hash stored outside the LLM context window, verified before every write operation. If the context has been tampered with between reads, the hash diverges and the write is blocked.

### 3.2 MCP Avoidance

Golems do not use MCP servers for on-chain operations. This is a deliberate architectural decision based on measured risk.

Endor Labs found that 82% of 2,614 MCP implementations use file system operations prone to path traversal, 67% use APIs related to code injection, and 34% expose command injection surfaces [ENDORLABS-2026]. CVE-2025-6514 (CVSS 10.0 RCE) in `mcp-remote` was downloaded over 558,000 times before patching [COMPOSIO-2026]. The MCP protocol has a structural flaw: it cannot distinguish between instructions and data. Tool descriptions, parameter schemas, and response payloads are all processed by the LLM as undifferentiated text.

Golems use a two-layer tool model instead:

- **Layer 1: Eight Pi-facing tools.** The LLM sees `preview_action`, `commit_action`, `cancel_action`, `query_state`, `search_context`, `query_grimoire`, `update_directive`, `emergency_halt`. That's ~1,200 tokens of tool definitions, 12x cheaper than an equivalent MCP configuration.
- **Layer 2: 166+ Sanctum adapters.** Compiled Rust behind the eight Pi-facing tools. Version-locked, statically verified, auditable. Cannot be modified at runtime by an external party.

This eliminates tool poisoning, cross-server shadowing, rug pulls, and supply chain compromise as attack vectors. The trade-off is reduced extensibility -- Golems cannot discover arbitrary tools at runtime. For a DeFi agent managing real capital, this is the right trade-off.

### 3.3 Inference Safety Budget Model

Every safety check that uses LLM inference has a cost. The budget model caps the total inference spend on safety per heartbeat tick:

| Check | Model | Cost per Check | Max per Tick | Purpose |
|---|---|---|---|---|
| Tool-Guard semantic analysis | DeBERTa (ONNX, local via `ort` crate) | ~$0.00 (local inference) | Unlimited | Tool response coherence |
| A-MemGuard consensus (Layer 2) | Primary model | ~$0.02 | 3 | Knowledge validation |
| Dual-LLM sandboxed processing | Budget model (Haiku-class) | ~$0.005 | 5 | Data sanitization |
| Multi-agent guard vetting | Independent model | ~$0.01 | 2 | Policy violation detection |

Total safety inference budget per tick: $0.10. If the budget is exhausted, remaining safety checks fall back to deterministic-only validation (Layers 3, 7). The DeBERTa classifier runs locally via the `ort` crate (ONNX Runtime for Rust), so it consumes zero inference budget.

### 3.4 Tool Response Validation

When vault operations use the Uniswap Trading API execution path, Layer 8 (API Response Validation) applies:

- Transaction data integrity (`TransactionRequest.data` non-empty)
- Quote freshness (reject quotes > 30s)
- Routing type consistency
- Permit2 signature matching
- Gas fee threshold enforcement

---

## 4. Defense Layering

No single bypass is sufficient. The layers compose:

```
Layer 1:  System prompt instructs vault-only behavior
          (12% bypass rate -- insufficient alone)
Layer 2:  Dual-LLM prevents data-as-instructions
          (requires compromising two separate LLMs)
Layer 2.5: Tool integrity prevents tool response manipulation
          (96% detection rate on tool-call attacks)
Layer 3:  TEE policy engine restricts agent to proxy only
          [CRYPTOGRAPHIC -- cannot be reasoned around]
Layer 4:  Time-delayed proxy queues tx with mandatory wait
          [ON-CHAIN -- tx is publicly visible and cancellable]
Layer 5:  Monitoring bot evaluates and can cancel during delay
          (automated + human review)
Layer 6:  Simulation detects unexpected fund movement
          (on-chain state verification)
Layer 7:  PolicyCage reverts unauthorized operations
          [IMMUTABLE CODE -- no override possible]
```

Even a fully compromised LLM cannot move funds. Layers 1-3 are preventive. Layers 4-5 are reactive. Layers 6-7 are enforcement.

---

## 5. Hallucination and Grounding

DeFi agents face a unique hallucination risk: confident but wrong on-chain state assertions. An agent that hallucinates a pool's liquidity depth or a token's price can execute catastrophically bad trades.

### 5.1 On-Chain Grounding Requirements

Every agent assertion about on-chain state must be grounded in a verifiable source:

| Assertion Type     | Required Source                     | Verification                                |
| ------------------ | ----------------------------------- | ------------------------------------------- |
| Token price        | RPC call to oracle or pool contract | Cross-reference with independent price feed |
| Pool liquidity     | Direct contract read                | Compare against subgraph for consistency    |
| Vault NAV          | `convertToAssets()` call            | Compare against cached recent value         |
| Account balance    | `balanceOf()` call                  | No cached value trusted without refresh     |
| Transaction status | `getTransactionReceipt()`           | Receipt must match expected chain ID        |

### 5.2 Confidence Calibration

Agents must distinguish between:

- **Known state**: Directly read from chain within the last block
- **Cached state**: Read previously, may be stale
- **Inferred state**: Derived from known state via reasoning
- **Uncertain state**: No grounding available -- triggers default-to-inaction (Constitution Principle 8)

When current-regime accuracy < 80%, force fork simulation for ALL operations. Wide confidence intervals require the most conservative action.

---

## 6. Agent Scope Restriction

The insecurity of general-purpose agents [BEURER-KELLNER-2025] drives a narrow-scope design principle. Each agent type has a fixed operation set:

| Agent Type      | Permitted Operations                                 | Prohibited                                |
| --------------- | ---------------------------------------------------- | ----------------------------------------- |
| Vault Manager   | deposit, withdraw, rebalance, collect fees           | Arbitrary contract calls, token transfers |
| LP Manager      | add/remove liquidity, collect fees, range adjustment | Vault operations, lending operations      |
| Trade Executor  | swap via approved routers                            | LP operations, vault operations           |
| Safety Guardian | read-only checks, veto writes                        | Any write operation                       |

Scope restrictions are enforced at three independent layers:

1. System prompt (behavioral, ~88% effective)
2. TEE signing policy (cryptographic, function selector whitelist)
3. PolicyCage (on-chain, approved protocols + strategy whitelist)

---

## 7. Indirect Injection Vectors

### 7.1 Data Feed Injection

On-chain metadata (vault names, token symbols, NFT URIs) can contain injection payloads. Mitigations:

- Metadata treated as data, never parsed as instructions
- Dual-LLM architecture sanitizes all external strings
- Character filtering on display-rendered content

### 7.2 Inter-Agent Trust Exploitation

SEAgent's 84.6% success rate via confused deputy attacks requires:

- No transitive trust between agents -- each independently validates via Sanctum tools
- Agent delegation DAG enforced (no cycles, max depth 3)
- Terminal nodes (safety-guardian, risk-assessor) never delegate to other agents
- Independent state verification before acting on another agent's assertions

### 7.3 Memory/Context Corruption

Long-running agents accumulate context that can be poisoned over time:

- Memory integrity hashing (Layer 2.5) detects tampering
- Context window rotation on session key rotation (every 24 hours default)
- Grimoire (the Golem's persistent knowledge store) ingestion pipeline (see [03-ingestion.md](03-ingestion.md)) validates all persistent knowledge

---

## 8. Real-World Incident Analysis

Three incidents define the current threat landscape:

| Incident                | Loss                 | Root Cause                                                    | Preventing Layer                                  |
| ----------------------- | -------------------- | ------------------------------------------------------------- | ------------------------------------------------- |
| AIXBT hack (March 2025) | $106K                | No wallet policy -- compromised dashboard sent funds anywhere | Layer 3 (policy) + Layer 4-5 (proxy + monitoring) |
| Bybit hack (2025)       | $1.5B                | Supply chain exploit on Safe signing interface                | Layer 4 (delay provides cancel window)            |
| ClawHub campaign        | 335 malicious skills | Malicious tools injecting instructions                        | Layer 2.5 (tool provenance signing)               |
| CVE-2025-59944          | Zero-click RCE       | Google Docs triggered Python execution via tool server        | Layer 2.5 (Tool-Guard) + scope restriction        |

---

## Cross-References

- [00-defense.md](00-defense.md) -- The main defense-in-depth architecture doc: six defense layers from system prompt through on-chain PolicyCage, `Capability<T>` compile-time tokens, `TaintedString` information-flow control, and audit chain.
- [01-custody.md](01-custody.md) -- Three wallet custody modes: TEE signing policies for Embedded mode, session signer architecture for Delegation mode, and bounded keypairs for LocalKey mode.
- [prd2-extended/10-safety/02-warden.md](../../prd2-extended/10-safety/02-warden.md) -- Optional Warden time-delayed proxy: announces transactions, holds them for a configurable delay, then executes or cancels. A reactive defense layer (deferred to phase 2).
- [02-policy.md](02-policy.md) -- PolicyCage on-chain smart contract: the cryptographic enforcement layer that reverts transactions violating spending caps, asset whitelists, and drawdown limits regardless of LLM state.
- [03-ingestion.md](03-ingestion.md) -- Four-stage knowledge ingestion safety pipeline (quarantine, consensus validation, sandbox, adopt) protecting the Grimoire from poisoned entries.
- [05-threat-model.md](05-threat-model.md) -- Full adversary taxonomy and attack tree analysis, placing prompt injection in the context of all attack vectors (external, insider, compromised agent, malicious user).

---

## References

- [CAMEL-2025] Debenedetti, E. et al. "CaMeL: Capability-Based Authorization for LLM Agents." arXiv:2503.18813. Creates a capability-based security layer separating control flow from data flow so untrusted data cannot impact program flow. Achieves 77% task completion with provable security. Directly informs Bardo's `Capability<T>` token model.
- [PROGENT-2025] Shi, Y. et al. "Progent: Privilege Control for LLM Agents." arXiv:2504.11703. Proposes fine-grained privilege control for LLM agents, restricting which tools and resources an agent can access based on context. Complements the capability-based approach with runtime privilege checking.
- [BEURER-KELLNER-2025] Beurer-Kellner, L. et al. "Design Patterns for Securing LLM Agents." arXiv:2506.08837. Catalogs design patterns for secure LLM agent architectures (sandboxing, dual-LLM, capability gating). Several patterns are directly implemented in Bardo's safety stack.
- [SEAGENT-2026] Ji, H. et al. "SEAgent: Confused Deputy Problem in Multi-Agent LLM Systems." arXiv:2601.11893. Identifies that inter-agent trust exploitation achieves 84.6% attack success vs 46.2% for direct injection. Motivates Bardo's decision to avoid MCP-based multi-agent communication for on-chain operations.
- [ARXIV-2509.14285] "Multi-Agent Prompt Injection Defense Pipeline." arXiv:2509.14285. Proposes a multi-agent pipeline where independent agents validate each other's outputs. Informs the dual-LLM architecture recommendation for high-AUM agents.
- [ARXIV-2503.16248] "CrAIBench: AI Agents in Cryptoland." arXiv:2503.16248. Benchmarks AI agent vulnerabilities in cryptocurrency environments; demonstrates how adversaries inject malicious instructions into prompts or historical records to trigger unintended asset transfers.
- [ARXIV-2512.02261] "TradeTrap." arXiv:2512.02261. Shows that memory injection is more powerful than prompt injection for trading agents, producing effects that persist across sessions. Motivates the separation of knowledge ingestion from prompt processing.
- [ARXIV-2506.23260] "MCP Attack Taxonomy." arXiv:2506.23260. Categorizes attack vectors in the Model Context Protocol including tool poisoning, cross-server shadowing, and rug pulls. Directly informs the decision to use compiled Rust tools instead of MCP servers.
- [ENDORLABS-2026] Endor Labs. "Classic Vulnerabilities Meet AI Infrastructure: Why MCP Needs AppSec." January 2026. Audited 2,614 MCP implementations: 82% have path traversal vulnerabilities, 67% are susceptible to code injection. CVE-2025-6514 (CVSS 10.0 RCE) downloaded 558K+ times.
- [COMPOSIO-2026] Composio. "MCP Vulnerabilities Every Developer Should Know." 2026. Practical catalog of MCP security issues including tool manifest tampering and cross-origin tool shadowing. Reinforces the compiled-tools-only approach.
