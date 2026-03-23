# 05 -- Session management [SPEC]

**Checkpoint/resume, working memory (scratchpad), and sub-agent spawning**

> Related: [04-context-engineering.md](04-context-engineering.md) (8-layer context engineering pipeline; compaction operates on sessions), [06-memory.md](06-memory.md) (persistent cross-session memory via Styx retrieval), [09-api.md](09-api.md) (API reference with 33 endpoints including session management)

---

> **Reader orientation:** This document specifies session management for Bardo Inference (the LLM inference gateway for mortal autonomous DeFi agents called Golems). It belongs to the inference plane and covers checkpoint/resume, scratchpad working memory, and sub-agent spawning. The key concept is that sessions provide stateful conversation management with fault tolerance and cost control, separate from the Golem's persistent memory (which lives in Styx). For term definitions, see `prd2/shared/glossary.md`.

## Session management with checkpoint/resume

Sessions provide stateful conversation management with fault tolerance, strategy branching, and cost control. Every session tracks messages, context utilization, compaction history, and cost accumulation.

### Session lifecycle

```text
create -> [messages...] -> checkpoint -> [more messages...] -> checkpoint
                              |                                    |
                              +-> resume (branch A)                +-> resume (branch B)
                              |   (different model/config)         |   (different strategy)
                              +-> resume (branch C)                +-> resume (branch D)
```

### CreateSessionRequest

```rust
// crates/bardo-gateway/src/sessions/api.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// ERC-8004 agent identity.
    pub agent_id: u64,
    /// Session configuration.
    pub config: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Model to use (or "auto" for routing).
    pub model: String,
    /// System prompt override (otherwise from agent config).
    pub system_prompt: Option<String>,
    /// Template to use (see 04-context-engineering.md).
    pub template: Option<SessionTemplate>,
    /// Compaction configuration.
    pub compaction: Option<CompactionConfig>,
    /// Maximum cost for this session in USDC (hard limit).
    pub max_cost_usdc: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTemplate {
    pub name: String,
    pub version: Option<String>,
    pub params: Option<HashMap<String, String>>,
}
```

### Checkpoint/resume

Checkpoints are labeled snapshots of session state. They enable:

- **Fault tolerance**: Resume from last checkpoint on crash or timeout
- **Strategy branching**: Try different approaches from the same starting point
- **Model switching**: Resume with a different model (e.g., switch from Sonnet to Opus for a complex sub-problem)
- **Cost management**: Branch before an expensive operation, abandon if results are poor

```rust
// crates/bardo-gateway/src/sessions/api.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRequest {
    /// Human-readable label for this checkpoint.
    pub label: String,
    /// Optional metadata (strategy name, market conditions, etc.).
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeRequest {
    /// Checkpoint ID to resume from.
    pub checkpoint_id: String,
    /// Optional config overrides for the resumed session.
    pub config_override: Option<SessionConfigOverride>,
}

/// Partial version of SessionConfig for resume overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfigOverride {
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub template: Option<SessionTemplate>,
    pub compaction: Option<CompactionConfig>,
    pub max_cost_usdc: Option<f64>,
}
```

Checkpoint storage is efficient -- only the delta from the previous checkpoint is stored. Checkpoints expire after 24 hours by default (configurable per agent tier).

### Endpoints

| Method | Path                           | Purpose                                        |
| ------ | ------------------------------ | ---------------------------------------------- |
| `POST` | `/v1/sessions`                 | Create a new session                           |
| `POST` | `/v1/sessions/{id}/checkpoint` | Create a labeled checkpoint                    |
| `POST` | `/v1/sessions/{id}/resume`     | Resume from a checkpoint (creates new session) |

---

## Working memory (scratchpad)

A mutable per-session document that the agent can read and update at any time. Injected at the END of each request, exploiting recency bias for better task adherence.

This is the Manus/Claude Code/Devin pattern: externalize working state into a scratchpad that persists across turns and occupies the model's strongest attention zone (the end of context).

### Scratchpad struct

```rust
// crates/bardo-gateway/src/sessions/scratchpad.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scratchpad {
    /// Session this scratchpad belongs to.
    pub session_id: String,
    /// Freeform Markdown content. The agent controls this entirely.
    pub content: String,
    /// Last update timestamp (Unix millis).
    pub last_updated_at: u64,
    /// Current token count of scratchpad content.
    pub token_count: u32,
}
```

### How it works

1. Agent creates a session (scratchpad starts empty)
2. Agent updates scratchpad via `PATCH /v1/sessions/{id}/scratchpad` with current execution plan, intermediate results, or any working state
3. On every subsequent request, the gateway appends the scratchpad content after the user's message:

```rust
// crates/bardo-gateway/src/sessions/scratchpad.rs

fn inject_scratchpad(messages: &mut Vec<Message>, scratchpad: &Scratchpad) {
    if scratchpad.content.is_empty() {
        return;
    }

    messages.push(Message {
        role: Role::User,
        content: format!(
            "<working_memory>\n{}\n</working_memory>",
            scratchpad.content
        ),
    });
}
```

### Why end-of-context

Placing the scratchpad at the END of the context (after the user query) does two things. It externalizes working state so the model doesn't need to "remember" its plan -- the plan is always visible. And it manipulates attention: LLMs attend most strongly to the beginning and end of context, so the scratchpad occupies that final high-attention zone and keeps the execution plan dominant even as conversation history grows to thousands of tokens.

This combats "lost in the middle" without any architectural complexity.

### Endpoint

| Method  | Path                           | Purpose                   |
| ------- | ------------------------------ | ------------------------- |
| `PATCH` | `/v1/sessions/{id}/scratchpad` | Update scratchpad content |

---

## Sub-agent spawning

Single-level sub-sessions for focused subtasks. A parent session spawns a child session that runs independently, and the parent receives only the child's final output, not its full conversation history.

This is the Claude Code Task tool pattern: constrain context pollution by isolating sub-tasks.

### SpawnRequest

```rust
// crates/bardo-gateway/src/sessions/api.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReturnFormat {
    Summary,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRequest {
    /// Task description for the sub-agent.
    pub task: String,
    /// Model for the sub-agent (can be different/cheaper than parent).
    pub model: Option<String>,
    /// Specific context to include (not the full parent conversation).
    pub context: Option<String>,
    /// Maximum tokens for the sub-agent's response.
    pub max_tokens: Option<u32>,
    /// What the parent receives back.
    pub return_format: ReturnFormat,
}
```

### Common pattern: cheap research, expensive reasoning

```text
Parent Session (Sonnet)
  |
  +-- spawn: "Research current ETH/USDC pool state on Base" (Haiku)
  |   -> Returns: pool TVL, fee tier, current tick, 24h volume
  |
  +-- spawn: "Analyze optimal LP range given this data" (Sonnet)
  |   -> Returns: recommended range, expected IL, APR estimate
  |
  +-- Parent synthesizes and decides (Sonnet)
```

The parent orchestrates while sub-agents do focused work. Insight from production systems: >50% of LLM calls should use a cheap model. Sub-agent spawning makes this structural rather than ad-hoc.

### Constraints

- **Single-level only**: Sub-agents cannot spawn their own sub-agents. This prevents unbounded cost escalation.
- **Isolated context**: Sub-agents get only the context explicitly provided in the spawn request, plus the global prefix (Layer 1).
- **Cost attribution**: Sub-agent costs are attributed to the parent session's agent and strategy.

### Endpoint

| Method | Path                            | Purpose                   |
| ------ | ------------------------------- | ------------------------- |
| `POST` | `/v1/sessions/{parentId}/spawn` | Spawn a sub-agent session |

---

## Golem-RS session management

Golems (mortal autonomous DeFi agents managed by the Bardo runtime) manage sessions through the Golem-RS runtime, which operates at a different layer than the inference gateway's session management described above.

### Session struct

```rust
// crates/golem-inference/src/session.rs

/// A session managed by the inference gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceSession {
    pub session_id: String,
    pub golem_id: u64,
    pub model: String,
    pub created_at: u64,
    /// Labeled snapshots for fault tolerance and branching.
    pub checkpoints: Vec<Checkpoint>,
    /// Mutable per-session document (the Golem's working notepad).
    pub scratchpad: Scratchpad,
    /// Sub-agent sessions spawned from this session.
    pub sub_agents: Vec<SubAgentRef>,
    /// Accumulated cost in USDC.
    pub total_cost_usd: f64,
    /// Hard cost cap for this session.
    pub max_cost_usd: Option<f64>,
    /// Number of in-place compactions applied.
    pub compaction_count: u32,
}
```

### JSONL persistence and crash recovery

Every message, tool call, and extension state entry is appended to a session JSONL file. Crash recovery replays from this log. See `01-golem/13-runtime-extensions.md` S6.

### Session branching

The runtime supports forking sessions for simulation. Three branch types:

- **`sim`** -- What-if analysis. The Golem explores hypothetical market conditions from a checkpoint.
- **`pre-steer`** -- Parallel simulation before owner intervention. Runs alongside the main session to preview the effect of a steer.
- **`dream`** -- Creative exploration during sleep cycles. Branches inherit context but not GolemState. State is serialized via `session(before_branch)`.

Branches are single-level only. A branched session cannot branch again. This prevents unbounded cost escalation.

### Owner intervention routing

Two intervention primitives:

- **Steer** (high priority): "Change what you're doing RIGHT NOW." Preempts the current tick, injected into context immediately, can cancel in-flight tool calls.
- **FollowUp** (low priority): "Consider this next time you deliberate." Queued for the appropriate decision window, incorporated at the next relevant point.

Both become Styx episodes after outcome resolution -- the Golem learns from owner feedback the same way it learns from market outcomes. Repeated owner steers can graduate to self-protective heuristics through dream consolidation.

### Custom compaction

When the context window fills, the runtime fires `session(before_compact)`. The compaction extension generates DeFi-aware summaries that preserve positions, PolicyCage hash, vitality, risk parameters, and top strategy heuristics as structured data.

### Multi-surface multiplexing

For multi-surface deployment, the session multiplexer allows terminal, web, and bot surfaces to share a single session with cross-surface continuity.

---

## Cross-references

| Topic                             | Document                                               | What it covers |
| --------------------------------- | ------------------------------------------------------ | -------------- |
| Compaction (operates on sessions) | [04-context-engineering.md](04-context-engineering.md) | 8-layer context engineering pipeline including history compression that operates on session conversation state |
| Persistent memory (cross-session) | [06-memory.md](06-memory.md)                           | Agent memory service backed by Styx: importance scoring, background consolidation, and cross-session persistence |
| Session endpoints (full API)      | [09-api.md](09-api.md)                                 | API reference with 33 endpoints including session create, checkpoint, resume, and branching |
| Cost attribution                  | [03-economics.md](03-economics.md)                     | x402 spread revenue model with per-tenant and per-session cost tracking |
