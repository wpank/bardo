# Golem archetypes -- architecture overview [SPEC]

> **Cross-ref**: [01-agent-categories.md](01-agent-categories.md) (full inventory of 42+ archetypes across 14 categories), [03-delegation.md](03-delegation.md) (tool resolution pipeline and delegation DAG), [09-golem-agents.md](09-golem-agents.md) (three autonomous core archetypes: LP manager, yield optimizer, risk sentinel), [../01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) (Pi extension loading and tool registration)

> **Reader orientation:** This is the architecture overview for Golem (mortal autonomous agent) archetypes -- behavioral presets that configure how a Golem thinks, what tools it can access, and how it delegates work. It belongs to Section 19 (Agents & Skills) and covers the archetype loading pipeline, tool profiles, the three-level Hermes intelligence hierarchy, and cross-operator coordination via ERC-8001/8033/8183. You should understand that Golems run a continuous Heartbeat (9-step decision cycle) and accumulate knowledge in a Grimoire (persistent knowledge base). See `prd2/shared/glossary.md` for full term definitions.

---

## What archetypes are

A golem archetype is a behavioral preset that configures how a golem thinks, what tools it can access, and how it delegates work. The archetype is not the golem. The golem is a running Pi (Anthropic's agent SDK runtime) agent session with a wallet, a heartbeat loop, a Grimoire, and a finite lifespan. The archetype is the configuration template that shapes that session's behavior at boot.

Think of it like a class versus an instance, but the instance evolves. The archetype sets initial tool access, delegation rules, model tier assignment, and safety constraints. The golem's PLAYBOOK.md then drifts from those initial settings as the golem accumulates experience. Two golems instantiated from the same archetype will diverge within hours.

```text
GolemConfig
  |
  +-- archetype: "vault-manager"    <- behavioral preset
  |
  +-- Pi Session
  |     tools: resolved from archetype + profile
  |     extensions: bardo-* (19 extensions)
  |     model: determined by archetype tier
  |
  +-- PLAYBOOK.md                   <- evolves per-golem
  +-- STRATEGY.md                   <- operator-authored
  +-- Grimoire                      <- accumulated knowledge
  +-- Wallet                        <- identity + payment
  +-- Heartbeat FSM                 <- autonomous loop
```

### Design principles

1. **Pi-native.** Archetypes are TOML configuration files loaded by `golem-tools` (Rust crate) at session initialization. Not markdown frontmatter. Not Claude skill definitions. The Pi runtime is the execution substrate.

2. **Tool access via adapter layer.** Archetypes declare which `golem-tools` categories they need. The `bardo-tools` Pi extension resolves these into concrete tool definitions and registers them with the Pi session. The golem never constructs raw RPC calls.

3. **Mortality-aware.** Archetypes specify model tier assignments that directly affect burn rate. An archetype that defaults to Opus for every decision will die faster than one that uses Haiku for routine analysis. The cost model is architectural.

4. **Composable via delegation.** Complex operations emerge from archetypes delegating sub-tasks to other archetypes. The delegation graph is acyclic and validated at boot. Max depth: 3.

5. **Safety is structural.** The `bardo-safety` extension enforces PolicyCage constraints regardless of what the archetype specifies. An archetype cannot grant itself more permission than the safety layer allows. The LLM cannot override kill-switches.

6. **Failure is explicit.** When a tool call fails, the archetype receives a typed error. No silent degradation. The golem records the failure as an episode and adjusts strategy.

### How archetypes differ from the old agent model

The previous architecture defined agents as markdown files with Claude-specific frontmatter (`allowed-tools`, `model`, `Task(subagent_type:...)`). Those agents were invoked by Claude skills via slash commands. That model is gone.

| Old model (Claude agents)                | New model (golem archetypes)                     |
| ---------------------------------------- | ------------------------------------------------ |
| Markdown files with frontmatter          | TOML config files in `golem-tools/archetypes/`    |
| Invoked by Claude skills via `/commands` | Instantiated by Pi runtime via `createGolem()`    |
| Stateless between invocations            | Persistent state (PLAYBOOK.md, Grimoire)          |
| Tools referenced by MCP prefix           | Tools resolved by profile + adapter layer         |
| Human-in-the-loop via slash commands     | Autonomous heartbeat loop, operator via `steer()` |
| Immortal (always available)              | Mortal (USDC depletion, Hayflick limit)           |

---

## Archetype architecture

### How archetypes load

```text
createGolem(config: GolemConfig)
  |
  v
resolveArchetype(config.archetype)
  |  - Loads TOML archetype definition from golem-tools/archetypes/
  |  - Deserializes into ArchetypeConfig (serde)
  |
  v
resolveTools(archetype.toolCategories, config.profile)
  |  - Filters ALL_TOOL_DEFS by category membership
  |  - Converts ToolDef[] to Pi tool definitions
  |  - Registers with Pi session via bardo-tools extension
  |
  v
createAgentSession(piConfig)
  |  - System prompt assembled from archetype + PLAYBOOK.md + STRATEGY.md
  |  - Extensions loaded in dependency order (19 bardo-* extensions)
  |  - Model set from archetype.defaultModel
  |
  v
startHeartbeat()
  |  - FSM enters PROBING state
  |  - Tick loop begins
```

### Tool access via profiles

Archetypes do not list individual tools. They declare tool categories, which the `bardo-tools` extension resolves against the profile system:

| Profile        | Tool categories included                          | Typical archetype users         |
| -------------- | ------------------------------------------------- | ------------------------------- |
| `data`         | data, identity                                    | Research archetypes             |
| `trader`       | data + trading + safety                           | Execution archetypes            |
| `lp`           | data + trading + lp + safety                      | LP management archetypes        |
| `vault`        | data + vault + safety                             | Vault operation archetypes      |
| `observatory`  | data + intelligence + identity + memory           | Sleepwalker archetype           |
| `learning`     | data + memory + self_improvement                  | Strategy optimization           |
| `full`         | All categories                                    | Full capability golems          |
| `dev`          | All + debug tools                                 | Development and testing         |

### Error recovery

- **Idempotent retries**: 2s/4s/8s exponential backoff, max 3 attempts per operation
- **Partial-state checkpoints**: Multi-step strategies persist intermediate state for recovery
- **Nonce management**: Re-fetch on NONCE_TOO_LOW, self-transfer unstick on NONCE_TOO_HIGH
- **Phase enforcement**: `bardo-safety` blocks operations incompatible with current behavioral phase

### Gas management

- **Paymaster**: For onboarding only (first 5 transactions)
- **ETH buffer**: 0.01 ETH minimum per golem on Base
- **Gas price circuit breaker**: Pause non-urgent operations at 5x 7-day median baseFee
- **Batch optimization**: Aggregate non-urgent operations into hourly multicalls

---

## Cross-operator coordination

### In-process delegation (current model)

Archetypes compose via an in-process directed acyclic graph (DAG). Archetype A delegates to Archetype B, which delegates to Archetype C. All share a trust boundary (same operator), communicate via function calls, and have no on-chain settlement of inter-archetype obligations. DAG validation at boot enforces acyclicity.

### Cross-operator extension: ERC-8001 + ERC-8033 + ERC-8183

Three EIPs extend this model to cross-operator coordination without shared trust:

- **ERC-8001 (Coordination Layer)**: N-party unanimous consent with EIP-712/ERC-1271 signatures
- **ERC-8033 (Information Layer)**: Multi-agent oracle councils with commit-reveal-judge lifecycle
- **ERC-8183 (Commerce Layer)**: Bilateral job escrow with evaluator attestation

### Adoption path

1. **Phase 1 (v1)**: All coordination via DAG. Single-operator golems only.
2. **Phase 2**: ERC-8033 oracle councils for cross-operator reputation verification
3. **Phase 3**: ERC-8001 coordination for multi-vault operations
4. **Phase 4**: Full marketplace -- ERC-8183 commerce for golem-to-golem services

---

## The Three Hermes: Intelligence Hierarchy

Intelligence in Bardo lives at three levels. Each has different responsibilities, different model tiers, different lifespans. The analogy is biological: L0 is cellular intelligence, L1 is conscious thought, L2 is the ecosystem.

| Level | Name | Where it lives | What it does | Lifespan |
|---|---|---|---|---|
| **L0** | Golem Hermes | Inside each golem container | Skill creation, affect-modulated retrieval, context engineering | Dies with the golem |
| **L1** | Meta Hermes | In the TUI, on the owner's machine | Owner conversation, message routing, clade skill aggregation, Library curation | Persistent (no mortality) |
| **L2** | Marketplace protocol | On-chain + Styx backend | ERC-8183 settlement, skill pricing, verification, distribution | Infrastructure (no agent) |

The owner talks to L1. L1 talks to L0 instances. L2 is plumbing that L1 uses. The owner never interacts with L0 directly and should not need to think about L2 at all.

**L0 (Golem Hermes)** runs as a sidecar inference service inside each golem container, communicating over JSON-RPC on a Unix domain socket. It hooks into seven lifecycle points in the heartbeat cycle: `on_session` (skill loading + seed kit), `on_context` (affect-modulated skill retrieval), `on_tool_result` (feedback recording), `on_after_turn` (auto-skill creation), `on_end` (skill export), `on_dream_onset` (skill evolution), and `on_death` (Skill Testament). The search is affect-modulated: the golem's PAD emotional state biases which skills surface.

**L1 (Meta Hermes)** runs inside the `bardo-terminal` process. It routes owner messages, aggregates clade state, curates the Library of Babel, and develops its own meta-skill library for clade-level operations.

**L2 (Marketplace protocol)** is not an agent. Skills and Grimoire entries trade between owners via ERC-8183 escrow and the Bazaar marketplace. Settlement on Base via USDC.

See [13-hermes-hierarchy.md](13-hermes-hierarchy.md) for the full specification.

---

## Archetype count summary

| Category              | Count  | Package location         |
| --------------------- | ------ | ------------------------ |
| Execution             | 3      | `golem-tools/archetypes/` |
| Research & analysis   | 5      | `golem-tools/archetypes/` |
| Strategy              | 2      | `golem-tools/archetypes/` |
| Development           | 3      | `golem-tools/archetypes/` |
| Infrastructure        | 2      | `golem-tools/archetypes/` |
| Agent capital markets | 8      | `golem-tools/archetypes/` |
| Real-time             | 2      | `golem-tools/archetypes/` |
| Vault                 | 7      | `golem-tools/archetypes/` |
| Golem                 | 3      | `golem-tools/archetypes/` |
| Observer              | 1      | `golem-tools/archetypes/` |
| **Total**             | **36** |                          |

---
