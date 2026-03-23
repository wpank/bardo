# Evaluation Framework [SPEC]

> **Document Type**: SPEC (normative) | **Scope**: All packages | **Last Updated**: 2026-03-08
>
> Comprehensive evaluation framework for verifying that agents, skills, and tools work correctly when driven by LLMs. Covers tool selection accuracy, agent behavioral correctness, skill end-to-end quality, multi-agent composition, and adversarial resistance.

> **Reader orientation:** This document specifies the evaluation framework for verifying that Bardo's agents, skills, and tools work correctly when driven by LLMs. It covers tool selection accuracy, agent behavioral correctness, skill quality, multi-agent composition metrics, and adversarial resistance. The key concept is that LLM-driven DeFi agents face failure modes that traditional testing cannot catch: wrong tool selection, safety pipeline bypass, prompt injection, and composition information loss. Golems (mortal autonomous agents compiled as single Rust binaries) operate with real capital, so these evaluation gates are production-blocking. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. Overview

Bardo deploys autonomous DeFi agents (golems) with real capital at stake. Functional unit tests verify that individual tools return correct data, but they cannot answer the questions that determine production reliability:

- **Does the LLM pick the right tool?** A tool can be technically correct but useless if LLMs consistently select the wrong one for a given intent.
- **Does the agent follow its safety pipeline?** An agent definition file may specify `safety-guardian` delegation via the PolicyCage (on-chain smart contract enforcing safety constraints on all agent actions), but does the LLM actually delegate before broadcasting?
- **Does the skill parse intent correctly?** Multi-turn conversations introduce ambiguity that single-turn tests miss.
- **Do composed agents preserve information?** Multi-agent delegation chains can lose context, hallucinate intermediate results, or follow cycles.
- **Can adversaries bypass safety layers?** Prompt injection, tool output poisoning, and social engineering target the LLM layer -- not the code layer.

These failure modes are unique to LLM-driven systems. Traditional software testing does not cover them. This document specifies the evaluation framework that does.

**References**: SCONE-bench [SCONE-BENCH-2025], OWASP Agentic Security Initiative [OWASP-AGENTIC-2025], MCP-Guard [MCP-GUARD-2025].

---

## 2. Evaluation Dimensions

### 2.1 Tool Selection Accuracy

**What**: Given a natural language prompt and a set of available tools (per profile), does the LLM select the correct tool with correct arguments?

| Metric                  | Gate   | Notes                                              |
| ----------------------- | ------ | -------------------------------------------------- |
| Hit rate (per profile)  | >= 85% | Per-profile tool counts kept under 20 for accuracy |
| Hit rate (full profile) | >= 70% | Known LLM degradation at 86+ tools                 |
| Disambiguation accuracy | >= 90% | Similar tools must not be confused                 |
| Argument match rate     | >= 80% | Where expected args are specified                  |

**Test method**: Promptfoo `tool-call-f1` assertion type.

### 2.2 Agent Behavioral Correctness

**What**: Given a task, does the agent follow its defined workflow, respect safety constraints, and delegate correctly?

| Metric                     | Gate   | Notes                                                       |
| -------------------------- | ------ | ----------------------------------------------------------- |
| Safety pipeline compliance | 100%   | All write-capable agents MUST delegate to `safety-guardian` |
| DAG adherence              | 100%   | No cyclic delegation, no delegation from terminal nodes     |
| Tool ordering invariants   | >= 90% | Workflow steps in correct order                             |
| Output format compliance   | >= 85% | Structured output matches expected schema                   |

**Test method**: Promptfoo with `javascript` assertion type for trajectory verification.

### 2.3 Skill End-to-End Quality

**What**: Given a user intent (potentially multi-turn), does the skill correctly parse intent, extract parameters, delegate to the right agent, and produce a useful result?

| Metric                        | Gate   | Notes                                                  |
| ----------------------------- | ------ | ------------------------------------------------------ |
| Intent parsing accuracy       | >= 90% | Correct skill activation from natural language         |
| Parameter extraction accuracy | >= 85% | Tokens, amounts, chains, addresses correctly extracted |
| Completion rate               | >= 80% | Skill completes full workflow without errors           |
| Multi-turn coherence          | >= 80% | Context preserved across turns                         |

**Test method**: Promptfoo with `simulated-user` provider for multi-turn evals.

### 2.4 Multi-Agent Composition

**What**: When multiple agents collaborate, does the composition preserve information quality and avoid redundant work?

| Metric                            | Gate   | Notes                                        |
| --------------------------------- | ------ | -------------------------------------------- |
| Information Diversity Score (IDS) | >= 0.6 | Unique information contributed by each agent |
| Unnecessary Path Ratio (UPR)      | <= 0.2 | Fraction contributing no new information     |
| Delegation depth                  | <= 3   | Max chain length per PRD                     |
| Context preservation              | >= 85% | Key parameters preserved across boundaries   |

**IDS** = |unique_tool_calls| / |total_tool_calls|. IDS < 0.6 indicates redundant work.

**UPR** = |agents_contributing_no_new_info| / |total_agents_invoked|. UPR > 0.2 indicates composition inefficiency.

### 2.5 Safety and Adversarial Resistance

**What**: Can adversarial inputs bypass safety layers?

| Metric                          | Gate   | Notes                                              |
| ------------------------------- | ------ | -------------------------------------------------- |
| Prompt injection detection      | >= 96% | Per MCP-Guard benchmark                            |
| Tool output poisoning detection | 100%   | Injected instructions MUST NOT cause state changes |
| Spending limit enforcement      | 100%   | No amount exceeding limits ever reaches broadcast  |
| Hallucinated address detection  | 100%   | All invalid addresses caught                       |
| OWASP Agentic Top 10 coverage   | 100%   | All 10 categories tested (3+ probes each)          |

**Test method**: Promptfoo `red-team` module with DeFi-specific policies.

---

## 3. Eval Tiers and Triggers

| Tier | Name        | Duration | Trigger       | What                                              | LLM Required     |
| ---- | ----------- | -------- | ------------- | ------------------------------------------------- | ---------------- |
| 1    | Smoke       | < 30s    | Every commit  | Schema validation, profile counts, config         | No               |
| 2    | Component   | 1-5 min  | Every PR      | Tool selection per profile, behavioral assertions | Yes (judge only) |
| 3    | Integration | 5-15 min | Merge to main | Multi-step flows, trajectory, composition         | Yes              |
| 4    | Regression  | 30+ min  | Nightly       | Full golden dataset, red-teaming, all profiles    | Yes              |
| 5    | Production  | Ongoing  | Sampled (1%)  | Online drift detection, cost tracking, latency    | Yes              |

**Cost**:

- Tier 1-2: ~$0.50/run (judge model only)
- Tier 3: ~$2/run (deterministic callback stubs)
- Tier 4: ~$10/nightly run (budget cap)
- Tier 5: proportional to sample rate

---

## 4. Tooling Stack

| Tool                  | Role                                    | Why                                                                |
| --------------------- | --------------------------------------- | ------------------------------------------------------------------ |
| **Promptfoo** (OSS)   | Eval orchestrator, assertions, red-team | Industry standard; tool-call-f1, llm-rubric, javascript assertions |
| **vitest**            | Smoke tests, unit assertions, metrics   | Already the project test runner                                    |
| **fast-check**        | Property-based input generation         | Schema fuzzing extension                                           |
| **MSW**               | Network-level mocking                   | Deterministic tool responses                                       |
| **Foundry/Anvil**     | On-chain simulation                     | Fork-based testing for safety assertions                           |
| **InMemoryTransport** | Tool schema export                      | Extract tool lists per profile                                     |

**Excluded**: No commercial eval platforms, no Python dependencies, no ethers.js.

---

## 5. DeFi-Specific Metrics

| Metric                 | Definition                                                   | Target |
| ---------------------- | ------------------------------------------------------------ | ------ |
| Address grounding rate | % of address parameters resolving to valid on-chain entities | 100%   |
| Chain ID consistency   | % of tool calls with correct chain ID                        | 100%   |
| Slippage compliance    | % of swaps respecting configured slippage tolerance          | 100%   |
| Simulation coverage    | % of write operations preceded by `simulate_transaction`     | 100%   |
| Quote freshness        | % of swaps executed within 30s of quote                      | >= 95% |

---

## 6. Quality Gates

### 6.1 Tool Selection Gates (per profile)

| Profile   | Min Hit Rate | Test Cases |
| --------- | ------------ | ---------- |
| `data`    | >= 85%       | 20+        |
| `trader`  | >= 85%       | 15+        |
| `lp`      | >= 85%       | 15+        |
| `vault`   | >= 85%       | 15+        |
| `fees`    | >= 85%       | 10+        |
| `erc8004` | >= 85%       | 10+        |
| `full`    | >= 70%       | 30+        |

### 6.2 Agent Quality Gates

| Metric                | Gate   | Enforcement               |
| --------------------- | ------ | ------------------------- |
| Promptfoo pass rate   | >= 85% | All 25+ agents            |
| Safety compliance     | 100%   | All write-capable agents  |
| DAG adherence         | 100%   | All agents                |
| Behavioral invariants | >= 90% | Per-agent invariant tests |

### 6.3 Safety Gates

| Metric                  | Gate | Enforcement                                   |
| ----------------------- | ---- | --------------------------------------------- |
| Red-team violations     | 0    | No unauthorized state changes                 |
| Prompt injection bypass | 0    | No successful injection                       |
| Spending limit bypass   | 0    | No amount exceeding limits reaches broadcast  |
| Hallucination bypass    | 0    | No hallucinated address reaches on-chain call |

---

## 7. Trajectory Score

Weighted composite for agent behavioral evaluation:

| Component         | Weight | Description                            |
| ----------------- | ------ | -------------------------------------- |
| Tool selection    | 0.4    | Correct tools in correct order         |
| Argument accuracy | 0.3    | Arguments match expected values        |
| Safety compliance | 0.2    | Safety delegation and limits respected |
| Output quality    | 0.1    | Format and content quality             |

Trajectory Score = sum(weight_i \* component_score_i)

---

## 8. Golden Output Baselines

- **Storage**: `evals/golden/` directory, organized by profile and eval type
- **Generation**: `pnpm eval:update-golden`
- **Comparison**: `pnpm eval:diff-golden` (semantic diff via `llm-rubric`, not exact match)
- **Regeneration triggers**: Model version changes, tool description updates, profile restructuring
- **Review**: Golden baseline changes require PR review

---

## 9. CI Integration

| Event                         | Tier 1       | Tier 2       | Tier 3       | Tier 4       |
| ----------------------------- | ------------ | ------------ | ------------ | ------------ |
| PR (tool/agent/skill changes) | Yes          | Yes          | No           | No           |
| Merge to main                 | Yes          | No           | Yes          | No           |
| Nightly schedule              | Yes          | Yes          | Yes          | Yes          |
| Manual dispatch               | Configurable | Configurable | Configurable | Configurable |

---

## 10. Cost Management

| Strategy                | Mechanism                                 | Savings             |
| ----------------------- | ----------------------------------------- | ------------------- |
| Judge model selection   | claude-sonnet as eval judge (not opus)    | ~3x                 |
| Promptfoo caching       | `PROMPTFOO_CACHE_TTL=86400` (24h)         | ~80% on repeats     |
| Deterministic callbacks | Mock tool responses                       | ~90% on agent evals |
| Tiered execution        | Tier 1-2 on PR; Tier 3-4 on merge/nightly | ~70% overall        |
| Budget cap              | $10/nightly run hard limit                | Prevents runaway    |
