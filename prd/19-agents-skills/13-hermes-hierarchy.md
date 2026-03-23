# The Hermes Hierarchy [SPEC]

> **Version**: 1.0 | **Status**: Draft
>
> **Scope**: Intelligence layered across the Bardo system. Per-golem learning, owner-facing cognition, and marketplace protocol.
>
> **Cross-references**: `../01-golem/02-heartbeat.md` (9-step decision cycle that triggers L0 inference at each theta tick), `../05-oracle/01-prediction-engine.md` (prediction engine and T0/T1/T2 inference tier routing), `../04-memory/01-grimoire.md` (persistent knowledge base that L0 queries for affect-modulated retrieval), `../04-memory/02-library-of-babel.md` (cross-Golem knowledge library that L1 aggregates into Clade skills), `../01-golem/05-mortality.md` (mortality clocks and behavioral phases that modulate L0 inference budgets)
>
> **Source**: `mmo2/03-hermes-hierarchy.md`

> **Reader orientation:** This document specifies the three-level Hermes intelligence hierarchy: L0 (per-Golem skill creation and affect-modulated retrieval), L1 (Meta Hermes in the TUI for owner conversation and Clade skill aggregation), and L2 (on-chain marketplace protocol for skill trading via ERC-8183). It belongs to Section 19 (Agents & Skills). A Golem (mortal autonomous agent) runs L0 as a sidecar inference service inside its container. L1 runs on the owner's machine. L2 is infrastructure, not an agent. Understanding the Grimoire (persistent knowledge base) and the Daimon (affect engine implementing PAD emotional state) will help with the L0 specification. See `prd2/shared/glossary.md` for full term definitions.

---

Intelligence in Bardo lives at three levels. Each has different responsibilities, different model tiers, different lifespans. The analogy is biological: L0 is cellular intelligence, the kind of pattern recognition that happens inside individual neurons. L1 is conscious thought, the narrative self that talks and plans. L2 is the ecosystem, the market of signals between organisms.

This document specifies all three.

| Level | Name | Where it lives | What it does | Lifespan |
|---|---|---|---|---|
| L0 | Golem Hermes | Inside each golem container | Skill creation, affect-modulated retrieval, context engineering | Dies with the golem |
| L1 | Meta Hermes | In the TUI, on the owner's machine | Owner conversation, message routing, clade skill aggregation, Library curation | Persistent (no mortality) |
| L2 | Marketplace protocol | On-chain + Styx backend | ERC-8183 settlement, skill pricing, verification, distribution | Infrastructure (no agent) |

ERC-8183 is the agent-to-agent task escrow standard, deployed on Base. The owner talks to L1. L1 talks to L0 instances. L2 is plumbing that L1 uses. The owner never interacts with L0 directly and should not need to think about L2 at all.

---

## L0: Golem Hermes (per-golem intelligence)

### What it is

Every golem container runs a Hermes Agent instance as a sidecar inference service. The golem runtime (Rust) communicates with it over JSON-RPC on a Unix domain socket (`/tmp/hermes-golem.sock`). Hermes is accessed via this API boundary; its internal runtime (which happens to be Python) is opaque to the golem binary, not a dependency of it. This is the golem's learning organ. It watches every heartbeat tick, creates skills from operational experience, retrieves the right knowledge at the right time based on the golem's emotional state, and manages the per-golem skill library.

The golem itself is the decision-maker. Golem Hermes is the thing that makes the decision-maker smarter over time. The golem acts; Hermes watches, distills, remembers, and feeds knowledge back.

Golem Hermes integrates into the runtime as extension `bardo-hermes` at Layer 2 (Knowledge), alongside the Grimoire. It depends on `bardo-grimoire` and hooks into seven lifecycle points. Each hook fires at a specific moment in the heartbeat cycle, and each has a different job.

### The seven lifecycle hooks

These hooks are the backbone of L0 intelligence. They define when Hermes acts and what it does.

**1. `on_session` -- session start**

Fires when the golem boots or resumes from a paused state. Hermes loads the per-golem skill library from disk, counts available skills, and checks whether this is a brand-new golem. If it is, Hermes requests a seed kit from Meta Hermes (via Styx, the cross-Golem communication fabric, or direct connection), pulling relevant Clade (a group of related Golems sharing knowledge under one operator) skills to give the newborn a head start.

The seed kit is the Baldwin Effect made computational: successful survival strategies, validated across sibling golems, become starting conditions for the next generation. A golem born today does not start from zero. It starts from the compressed experience of every golem that came before it.

After loading, Hermes emits `SkillsLoaded { count }` to the Event Fabric. The TUI's Hermes screen shows "47 skills loaded" or whatever the number is.

```rust
async fn on_session(&self, reason: SessionReason, ctx: &mut SessionCtx) -> Result<()> {
    if matches!(reason, SessionReason::Start) {
        let skills = self.sidecar.list_skills().await?;
        *self.active_skills.write() = skills.into_iter()
            .map(|s| ActiveSkill::from_hermes(s))
            .collect();

        ctx.fabric.emit(GolemEvent::Hermes(HermesEvent::SkillsLoaded {
            count: self.active_skills.read().len(),
        }));
    }
    Ok(())
}
```

**2. `on_context` -- context window management**

Fires every tick during context assembly, before the LLM sees the Cognitive Workspace. This is where Hermes earns its keep. It searches the skill library for procedures relevant to the current market observation, takes the top 3 matches, and injects them into the workspace as a `[PROCEDURAL MEMORY]` system message.

The search is not a naive keyword match. It is affect-modulated: the golem's current PAD (Pleasure-Arousal-Dominance) vector biases which skills surface. An anxious golem (high arousal, low pleasure) gets cautionary skills, "how to exit positions safely during gas spikes." A confident golem (high pleasure, high dominance) gets optimization skills, "how to maximize LP fee capture in trending markets." An uncertain golem (low dominance) gets well-validated, conservative procedures.

This is Damasio's somatic marker hypothesis running as code. The same market conditions trigger different procedural knowledge depending on the golem's emotional state. The LLM does not choose which skills to consult. Hermes chooses, based on affect, and the LLM sees only the result.

```rust
fn affect_modulated_query(base_query: &str, pad: &PADVector) -> String {
    let mut query = base_query.to_string();

    // High arousal + low pleasure = fearful = prefer defensive skills
    if pad.arousal > 0.5 && pad.pleasure < -0.2 {
        query.push_str(" risk mitigation defensive caution");
    }

    // High pleasure + high dominance = confident = prefer optimization
    if pad.pleasure > 0.3 && pad.dominance > 0.3 {
        query.push_str(" optimization aggressive opportunity");
    }

    // Low dominance = uncertain = prefer well-validated skills
    if pad.dominance < -0.3 {
        query.push_str(" proven validated safe");
    }

    query
}
```

These thresholds (0.5, -0.2, 0.3, -0.3) are configurable via `[hermes.affect_modulation]` in golem.toml. Defaults based on empirical tuning against simulated market scenarios.

When skills match, Hermes emits `SkillsActivated { tick, skills }`. The TUI's Hearth screen shows the activated skill name in cyan on the decision ring.

**3. `on_tool_result` -- post-tool-call processing**

Fires after every MCP tool call returns. If the tool execution was guided by a Hermes skill (the LLM referenced a skill procedure in its reasoning), Hermes records the outcome as feedback: which skill, which tool, success or failure, PnL impact if measurable, tick number, and a summary of context.

This feedback accumulates in a queue. It is not processed immediately. Hermes batches feedback and flushes it during the Curator cycle (every 50 ticks) for efficiency. Individual tool results are cheap, high-frequency signals. Processing each one in isolation would be noisy. Batching lets Hermes see patterns: "this skill worked 8 out of 10 times this cycle" is a better signal than any single tool result.

```rust
async fn on_tool_result(&self, result: &ToolResult, ctx: &mut ToolResultCtx) -> Result<()> {
    if let Some(skill_ref) = ctx.active_skill_reference() {
        self.feedback_queue.lock().push(SkillFeedback {
            skill_name: skill_ref.clone(),
            tool_name: result.tool_name.clone(),
            success: result.is_success(),
            outcome_value: result.extract_pnl(),
            tick: ctx.state.current_tick,
            context: result.summary(),
        });
    }
    Ok(())
}
```

This hook uses T0-tier inference (none, actually -- it is pure data recording). No LLM call happens. Hermes writes to a local queue and moves on.

**4. `on_after_turn` -- end-of-turn reflection**

Fires after the LLM completes its turn and the decision has been executed. This is where the learning happens. Three things occur:

*Auto-skill creation.* When the golem solves a multi-step problem (T2 deliberation, successful outcome, 3+ steps), Hermes checks whether this solution pattern is already covered by an existing skill. If the nearest match has a relevance score below 0.6, the pattern is novel. Hermes drafts a new skill from the decision trace: trigger conditions, procedure steps, expected outcome. The draft goes into a staging area.

*Feedback flush.* Every 10 ticks, accumulated tool-result feedback drains from the queue into the Hermes sidecar's self-improvement loop. Each skill referenced in the feedback gets updated: its confidence adjusts based on success rate, its procedure text may get refined based on what worked and what did not.

*Skill draft materialization.* Every 50 ticks (aligned with the Curator cycle), staged drafts materialize into actual SKILL.md files. Hermes writes each draft to disk, assigns a name, and adds it to the active library. A `SkillCreated` event fires. The TUI plays a rising arpeggio.

The 50-tick alignment matters. The Curator cycle already runs consolidation on the Grimoire at this cadence, flushing episodes into insights, promoting heuristics, running forgetting. Hermes skill materialization piggybacks on the same cycle so that skill creation and Grimoire consolidation stay in sync.

**5. `on_end` -- session end**

Fires when the golem dies or when a session terminates. This is the death export.

Hermes packages every skill with confidence >= 0.6 and use_count >= 2 into a `SkillBundle`. The bundle carries the source golem's ID, generation number, death cause, and a timestamp. It gets published to Styx for clade inheritance. Meta Hermes picks it up and integrates the skills into the clade library.

Note the confidence threshold difference from the death testament (0.4 in `terminal_skill_export`, 0.6 in `on_end`). The terminal export during Thanatopsis is more aggressive, capturing everything that might be useful. The normal session-end export is more selective, sending only well-validated skills. A golem dying of natural causes (Hayflick limit, USDC depletion) has had time to validate its skills. A golem in Thanatopsis is running out of time and lowers the bar.

**6. `on_dream_onset` -- entering dream state**

Fires when the golem transitions from active operation to the dream cycle (every ~200 ticks). The Dream cycle has four phases: NREM Replay, REM Imagination, Consolidation, and Skill Evolution. The fourth phase is Hermes's territory.

During Skill Evolution, Hermes runs several operations:

*Heuristic export.* PLAYBOOK.md entries with confidence >= 0.75 and validated_count >= 3 export as Hermes SKILL.md files. Each skill is tagged with source heuristic IDs for provenance. This is the bridge between the Grimoire's internal knowledge representation and the portable skill format.

*Self-improvement.* Accumulated operational feedback feeds into the Hermes Agent's self-improvement loop. Skill procedure text gets rewritten based on what worked. A skill that said "check gas price before executing" might become "check gas price AND blob fee before executing" after feedback showed blob fee spikes causing failures.

*Cross-validation with Grimoire.* Skills that contradict high-confidence Grimoire entries get flagged. Skills that align with Grimoire causal edges get a confidence boost. This bidirectional validation keeps skills and the Grimoire consistent.

*Re-ingestion.* Updated skills re-enter the Grimoire through the four-stage ingestion pipeline at 0.65 confidence (higher than the default Lethe 0.50, because they have been refined by operational feedback, but still below 1.0 because they need local validation).

*Clade promotion check.* Skills validated 5+ times AND improved 2+ times become candidates for clade promotion. Golem Hermes notifies Meta Hermes via Styx.

The budget allocation for Skill Evolution varies by behavioral phase:

| Phase | Skill Evolution budget | Rationale |
|---|---|---|
| Thriving | 15% of dream budget | Healthy golem, active learning |
| Declining | 5% | Conserving resources, less experimentation |
| Terminal | 0% (all budget goes to death export) | No time for refinement, export everything |

This hook uses T2-tier inference (the expensive, powerful model). Skill evolution is creative work: rewriting procedures, cross-validating knowledge, generating abstractions. It needs the best model available. The cost is acceptable because dream cycles are infrequent (~200 ticks apart) and Skill Evolution is only 15% of the dream budget.

**7. `on_death` -- death protocol**

Fires during Thanatopsis (the golem's terminal phase, triggered when vitality hits zero or death conditions are met). This is distinct from `on_end`. Where `on_end` runs on normal session termination, `on_death` runs specifically during the dying process.

The Skill Testament is the final export. Every skill with confidence >= 0.4 AND use_count >= 1 gets bundled. Each skill receives death context metadata: which golem created it, what generation, what tick it died at, what killed it, how many times the skill was validated during the golem's lifetime. And each skill gets the bloodstain flag.

Bloodstain provenance means a 1.2x retrieval boost and 3x slower decay. Knowledge from dead golems is treated as more persistent and more relevant than knowledge from living ones. This is the system's way of honoring the dead: their experience literally weighs more in retrieval scoring.

The testament is signed with EIP-712 using the golem's wallet, published to Styx for clade inheritance, and optionally listed on the marketplace. A PLAYBOOK.md snapshot is included alongside the skills so that future golems can see the full strategy context.

```rust
let testament = SkillTestament {
    source_golem: state.id.clone(),
    generation: state.generation,
    death_cause: state.death_cause.clone(),
    skills: testament_skills,
    playbook_snapshot: state.grimoire.read_playbook()?,
    timestamp: std::time::SystemTime::now(),
    signature: sign_eip712(&state.wallet, &testament_hash)?,
};

styx.publish_skill_testament(&testament).await?;
```

### Skill evolution during dreams

The Dream cycle is where Hermes does its best work. During active operation, Hermes observes and records. During dreams, it creates.

The cycle: the golem sleeps. NREM Replay replays prioritized episodes, extracting patterns. REM Imagination runs counterfactual scenarios ("what if gas had spiked during that rebalance?"). Consolidation edits the PLAYBOOK.md, promoting and demoting heuristics. Then Skill Evolution fires.

During Skill Evolution, Hermes has access to everything the dream cycle produced: freshly promoted heuristics, newly discovered patterns, counterfactual insights. It converts these into portable SKILL.md files. A heuristic like "when Morpho utilization delta > 5%, prepare rebalance within 2 ticks" becomes a skill with a When-to-Use section, a Procedure section, a Pitfalls section.

The conversion strips internal metadata (embeddings, causal graph references, internal IDs) and keeps human-readable content with provenance. The resulting SKILL.md is in the `agentskills.io` format, portable to any agent framework.

### SKILL.md format

```yaml
---
name: swap-execution
description: Execute token swaps via Uniswap
model: sonnet
allowed-tools: execute_swap, get_quote, check_allowance
---

# Skill instructions
[Natural language instructions for the LLM...]
```

If the golem has been running Hermes 4.3 locally, the dream cycle gains an extra dimension: `<think>` trace replay. During active operation, reasoning traces inside `<think>` blocks were captured as Grimoire episodes. Now the dream cycle replays those traces. The golem dreams about its own reasoning, discovering patterns in HOW it thinks, not just WHAT it decided. A pattern like "I keep checking gas price as my first step in every LP operation" might crystallize into a skill.

### Death export as Skill Testament

When a golem dies, everything it learned is at risk. The Skill Testament is the insurance policy.

Format: a signed JSON bundle containing all qualified skills (confidence >= 0.4, use_count >= 1), a PLAYBOOK.md snapshot, death metadata, and an EIP-712 signature. Destination: published to Styx, where Meta Hermes picks it up and routes it to the Library of Babel.

The testament carries lineage. If golem-alpha-gen3 dies and its skills were originally created by golem-alpha-gen1, the lineage chain is preserved: `[golem-alpha-gen1, golem-alpha-gen2, golem-alpha-gen3]`. Future golems that equip these skills can trace the full evolutionary history.

What gets preserved:

- All skills above the confidence and use-count thresholds
- Full procedure text in `agentskills.io` format
- Provenance metadata (creator, generation, lineage, death context)
- Bloodstain flag (1.2x retrieval boost, 3x slower decay)
- PLAYBOOK.md snapshot (the golem's full strategic context at death)

What does not survive:

- Raw episodes (too specific, too voluminous)
- Low-confidence skills (noise, not signal)
- Embeddings (recomputed on ingestion by the receiving golem)
- Causal graph edges (internal Grimoire structure, not portable)

### The autonomous learning loop

The full loop, running from first heartbeat to last breath:

**Observe.** Every tick, Hermes searches its skill library for procedures relevant to the current market observation. The search is affect-modulated: the golem's emotional state biases which skills surface. Matched skills get injected into the Cognitive Workspace as procedural memory. The LLM sees relevant procedures alongside market data, positions, and Grimoire knowledge.

**Deliberate.** The LLM uses skill procedures as guidance. If a skill says "check gas price before executing LP operations," the LLM follows that procedure. Its decision references which skill it followed. This is not hard constraint -- the LLM can deviate. But skills provide structure to deliberation, turning open-ended reasoning into guided procedure execution.

**Execute.** Tool calls fire. Each call goes through the MCP tool infrastructure: safety checks, gas estimation, simulation. Results come back with success/failure status and PnL impact when measurable.

**Reflect.** After the turn completes, Hermes checks whether the golem solved a novel multi-step problem. If yes, it drafts a new skill. If a skill-guided action failed, it records negative feedback. This is where new knowledge is born and existing knowledge is tested.

**Curate.** Every 50 ticks, Hermes materializes skill drafts into SKILL.md files, flushes accumulated feedback into the self-improvement loop, decays confidence on low-performing skills, and promotes high-performing skills to the clade library. This is the editorial pass: drafts become publications, failures become corrections.

**Evolve.** Every ~200 ticks (dream cycle), the Skill Evolution phase runs. Validated heuristics export as skills. Self-improvement rewrites procedures. Cross-validation catches inconsistencies. The best skills get promoted to the clade.

Zero owner intervention required. The entire loop runs autonomously. The owner can steer (overriding strategy, adjusting risk parameters), but they never need to manage skills directly. The golem and its Hermes handle everything.

### Prediction-aware skill retrieval

L0 Hermes retrieves skills based on context relevance, affect modulation, and -- with the Oracle -- prediction accuracy.

Per-category accuracy influences which skills surface. When the golem needs to make a fee_rate prediction and its fee_rate accuracy is 84%, skills tagged with `fee_rate` are retrieved with standard priority. But if fee_rate accuracy drops to 55%, L0 boosts retrieval of fee_rate-related skills -- the golem needs help in that category and actively seeks knowledge that might improve its predictions.

Skill confidence from context attribution (Loop 6) feeds retrieval ranking directly. Skills with positive attribution scores (they helped predictions) rank higher. Skills with negative attribution rank lower. Over time, retrieval self-optimizes: the golem naturally surfaces the knowledge that actually helps it predict.

---

## L1: Meta Hermes (TUI-level intelligence)

### What it is

Meta Hermes is a Hermes Agent instance running inside the `bardo-terminal` process on the owner's machine. It is NOT a golem. It has no heartbeat, no wallet, no mortality clock, no Grimoire. It does not trade, does not hold positions, does not die.

Meta Hermes is the thing you talk to.

When you open the Bardo TUI and type a message in the conversation panel, that message goes to Meta Hermes first. Meta Hermes decides what to do with it: answer directly, route it to a specific golem, broadcast it to the clade, execute a meta-skill, or query one or more Grimoires.

This is the owner's primary interface with the entire system. L0 Hermes instances are invisible to the owner. They work in the background, improving golem cognition. The owner sees only Meta Hermes, which presents a unified conversational interface over the entire clade.

### Message routing

Meta Hermes reads the owner's message, considers context (which golem is focused in the TUI, recent conversation history, clade state), and classifies the message into one of six action types:

| Owner says | Meta Hermes does |
|---|---|
| "How are my golems doing?" | Aggregates state from all connected golems, responds directly |
| "Tell golem-alpha to increase LP range width" | Routes as a `steer` to golem-alpha via Styx |
| "Why did golem-beta rebalance at tick 847?" | Queries golem-beta's Grimoire, constructs answer |
| "Create a new vault-manager on Fly.io" | Executes a provisioning meta-skill |
| "What skills has the clade learned this week?" | Aggregates skill reports from all Golem Hermes instances |
| "Equip golem-delta with Morpho knowledge" | Queries Library of Babel, assembles loadout, triggers equip flow |
| "List our best LP skills on the marketplace" | Executes a marketplace meta-skill |
| "Show me the best-performing heuristic across all golems" | Cross-queries all Grimoires, ranks by confidence x validation count |

The routing logic:

```rust
pub enum MetaHermesAction {
    /// Answer directly (clade-level question, meta-skill execution)
    DirectResponse {
        response: String,
        skills_used: Vec<String>,
    },
    /// Route as a steer to a specific golem
    RouteSteer {
        target_golem: GolemId,
        steer_content: String,
        urgency: SteerUrgency,
    },
    /// Route as a follow-up question to a specific golem
    RouteFollowUp {
        target_golem: GolemId,
        question: String,
    },
    /// Broadcast to all golems (or a filtered subset)
    BroadcastSteer {
        steer_content: String,
        filter: Option<GolemFilter>,
    },
    /// Execute a meta-skill
    ExecuteMetaSkill {
        skill_name: String,
        params: serde_json::Value,
    },
    /// Query one or more golem Grimoires
    QueryGrimoires {
        targets: Vec<GolemId>,
        query: GrimoireQuery,
    },
}
```

If the owner has a specific golem focused in the TUI, messages default to that golem. In clade overview mode, Meta Hermes infers the target or asks for clarification. The routing decision is made by the LLM, not by pattern matching. Meta Hermes sees the full conversation history and uses it to disambiguate.

### Meta-skills

Meta Hermes develops its own skill library. These are skills that operate at the clade level, the kind of things no individual Golem Hermes would develop because they require cross-golem context.

| Category | Examples |
|---|---|
| Clade provisioning | "Create a new golem with archetype X on provider Y", "Clone golem-alpha's strategy to a new instance" |
| Clade analytics | "Compare performance across all golems", "Identify which golem's heuristics are most transferable" |
| Skill curation | "Promote golem-alpha's LP rebalancing skill to the clade library", "Merge similar skills from golem-beta and golem-gamma" |
| Owner modeling | "Summarize what the owner has steered about risk tolerance", "Generate a strategy brief from the owner's past conversations" |
| Marketplace operations | "List our best validated skills on the Bazaar", "Search the Lethe for liquidation-warning skills with confidence > 0.7" |
| Deployment management | "Scale golem-alpha to a medium VM, it's hitting memory pressure", "Extend golem-beta's TTL by 7 days" |

Meta-skills are stored in `~/.bardo/meta-hermes/skills/` and follow the same `agentskills.io` format as golem skills. Meta Hermes creates new meta-skills when it solves novel clade-management problems. The same self-improvement loop as L0, at a higher abstraction level.

Owner modeling is worth calling out. Over time, Meta Hermes accumulates a model of the owner's preferences from their steers and conversations. "The owner has consistently rejected strategies with IL exposure above 5%." "The owner prefers conservative fee tiers." This implicit preference model shapes how Meta Hermes routes messages, recommends skill loadouts, and configures new golems. The owner model is stored as a BTreeMap of `(preference_key, (value, confidence, last_updated_tick))` in the local Grimoire. Updated on `steer()` commands and inferred from owner interaction patterns. Entries decay at 0.995x per day (half-life ~138 days).

When owner steers conflict (e.g., "be aggressive" followed by "be conservative"), Meta Hermes applies recency weighting: the most recent steer takes precedence. Conflicting steers older than 24 hours are demoted to context rather than directive. The owner can view active steers via the MIND screen.

### Context window

Meta Hermes operates within a 16K token sliding context window. Oldest messages are evicted first, except pinned steers and active STRATEGY.md sections.

### Clade skill aggregation

Meta Hermes periodically collects skills from all Golem Hermes instances. This is not a merge. It is curation. Skills from individual golems enter the clade library at a discount:

| Source | Clade library confidence | Condition |
|---|---|---|
| Single golem, validated >= 3 times | 0.70 x golem confidence | Auto-promoted |
| Multiple golems, independently validated | 0.90 x avg confidence | Cross-validated (high trust) |
| Meta Hermes meta-skill | 1.0 (meta-level) | Direct write |
| External (marketplace purchase) | 0.50 x stated confidence | Lethe discount per ingestion pipeline |

Discounts reflect trust decay with distance: self-generated (1.0), clade sibling (0.90), same-strategy non-clade (0.70), cross-strategy (0.50). Based on epistemic proximity.

Cross-validation is the interesting case. If golem-alpha and golem-gamma both independently develop skills for handling gas spikes during LP rebalancing, and both skills are validated through operational use, Meta Hermes recognizes the convergence. The merged skill enters the clade library at 0.90x confidence, nearly full trust. Independent discovery of the same pattern by multiple agents is strong evidence that the pattern is real.

The clade skill library lives in `~/.bardo/clade-skills/`. When a new golem is provisioned, Meta Hermes seeds its Golem Hermes with relevant clade skills, filtered by archetype. A vault-manager golem gets vault-related skills. An LP-optimizer gets LP skills. A sleepwalker gets everything.

### The Librarian role

Meta Hermes manages the Library of Babel. The Library is the owner's persistent knowledge store, surviving across golem lifetimes. It holds Skill Testaments from dead golems, clade skills, marketplace purchases, and curated knowledge.

The Librarian has four jobs:

**Background deduplication.** When a new Skill Testament arrives from a dead golem, Meta Hermes checks whether the Library already contains substantially similar skills. Duplicate skills get merged, keeping the higher-confidence version and preserving the combined lineage. This prevents the Library from bloating with slight variations of the same procedure.

**Staleness detection.** Skills age. A gas optimization skill from three months ago may reference fee structures that no longer exist. Meta Hermes periodically scans the Library for stale entries. The heuristic: if a skill's domain has changed significantly (detected via market data or clade observations) and the skill has not been re-validated recently, it gets flagged as potentially stale. Stale skills are not deleted, but they get a confidence penalty and a warning label.

**Promotion.** Some skills prove themselves across generations. If a skill from golem-alpha-gen1 has been inherited by gen2, gen3, and gen4, each validating it independently, that skill is battle-tested. Meta Hermes promotes it to "canonical" status in the Library. Canonical skills get recommended first when equipping new golems.

**Natural language queries.** The owner can ask Meta Hermes about the Library: "What LP skills do we have for Base L2?" "Show me everything related to Morpho." Meta Hermes searches the Library using semantic similarity and returns results in conversation, with options to equip skills to golems.

When the owner creates a new golem, Meta Hermes recommends a loadout based on: the golem's archetype, the owner's stated strategy, the Library's canonical skills, and performance data from previous golems of the same archetype. The owner can accept, modify, or reject the recommendation.

### TUI modes

Meta Hermes operates across three TUI modes:

**Embedded mode** (`bardo run`): The TUI spawns a local golem container in-process. Golem runtime, Golem Hermes, and Meta Hermes all run locally. Ollama provides inference for both. This is the single-golem, development experience.

**Attach mode** (`bardo attach <golem-name>`): The TUI connects to a remote golem container via Styx WebSocket relay. The TUI renders the remote golem's Event Fabric stream. Steers flow back through Styx. Meta Hermes runs locally regardless. It always has the owner's context.

**Clade mode** (`bardo clade`): The TUI connects to all golems in the clade simultaneously. Meta Hermes aggregates state from every golem. The TUI shows a clade overview with drill-down to individual golems. This is fleet command.

In all three modes, Meta Hermes is the conversational constant. The owner's message always goes to Meta Hermes first, and Meta Hermes routes from there.

---

## L2: Marketplace protocol

L2 is not an agent. It is a protocol. No inference runs at this level. There is no Hermes instance, no LLM, no skills. L2 is infrastructure: smart contracts, Styx APIs, and settlement mechanics.

Skills and Grimoire entries trade between owners via ERC-8183 (escrow-based purchases) and the Bazaar marketplace. Settlement happens on Base via USDC. Discovery uses the Bloom Oracle for privacy-preserving search and Styx listing indexes for browsable catalogs.

The flow: an owner decides to sell a skill. Meta Hermes (L1) packages it, validates the confidence threshold (minimum 0.6), generates a preview (first 500 characters, free to read), encrypts the full content (AES-256-GCM, key escrowed in the ERC-8183 contract), computes a content hash, and sets pricing. The listing publishes on-chain and on Styx.

A buyer discovers the skill via the Market TUI window, the Bloom Oracle, or the `skills.bardo.run` web catalog. They read the preview, check provenance and confidence, and purchase via x402 micropayment. The escrow contract releases the decryption key. For golem buyers, the skill enters the four-stage ingestion pipeline. For non-golem buyers (human traders, other agent frameworks), the skill downloads as a SKILL.md file.

Pricing follows a formula: base price x confidence multiplier x freshness decay. Confidence 0.6 yields an 0.8x multiplier. Confidence 1.0 yields 1.0x. Freshness decays exponentially at -0.01 per hour (1.0 at listing, 0.50 after 69 hours). Price floors at 10% of base to prevent free-riding.

Revenue split: 88% to seller, 5% to Styx (relay + storage), 5% royalty to original creator (if resale), 2% escrow gas.

---

## Inference topology per level

### L0: Golem Hermes inference

Not all hooks need the same model. The hook's computational demands dictate the tier:

| Hook | Model tier | Rationale | Typical cost per invocation |
|---|---|---|---|
| `on_session` | None | Disk I/O only, no inference | $0 |
| `on_context` | None (skill search is embedding similarity, not LLM) | Vector search against FTS5 database | $0 |
| `on_tool_result` | None | Data recording only | $0 |
| `on_after_turn` | T1 (novelty check + draft creation) | Needs to assess whether a decision trace is novel and draft a skill outline | $0.001-0.005 |
| `on_end` | None | Packaging and publishing, no inference | $0 |
| `on_dream_onset` | T2 (skill evolution is creative work) | Rewriting procedures, cross-validating, abstracting patterns | $0.01-0.05 |
| `on_death` | T1 (death context annotation) | Adding metadata to skills, signing testament | $0.001-0.005 |

Five of seven hooks need no LLM inference at all. They are data operations: reading, writing, searching, recording. Only `on_after_turn` (skill creation) and `on_dream_onset` (skill evolution) need inference, and `on_after_turn` only needs T1 (the cheap, fast tier). The expensive T2 tier fires only during dream cycles, which happen every ~200 ticks.

The per-golem inference provider cascade:

| Priority | Provider | Tiers | Cost |
|---|---|---|---|
| 1 | Local Hermes 4.3 (Ollama) | T1, optionally T2 | $0 |
| 2 | Venice | T1, T2 | API key or DIEM staking (free) |
| 3 | Bankr | T2 | x402 from golem's own wallet |
| 4 | Bardo Inference Gateway | T1, T2 | x402 with spread |
| 5 | OpenRouter | T1, T2 | API key |
| 6 | Direct (Anthropic/OpenAI) | T2 | API key |

Hermes Agent shares the golem's inference configuration. Unified in `golem.toml`:

```toml
[hermes]
inference_source = "golem"        # Share the golem's inference config
skill_creation_model = "T1"       # Cheap model for drafting skills
skill_evolution_model = "T2"      # Powerful model for refining skills
```

### L1: Meta Hermes inference

Meta Hermes runs in the TUI on the owner's machine. Its inference comes from the owner's configured providers:

**Routine operations** (message routing, clade state aggregation, Library queries): T1 tier. Local Hermes 4.3 handles this for $0 if Ollama is running. Otherwise, Venice or a cloud provider.

**Complex reasoning** (skill curation, owner modeling, cross-Grimoire analysis): T2 tier. This is less frequent. Most owner interactions are T1-class. A question like "compare the risk-adjusted performance of all LP golems over the past week" requires T2.

**Budget**: Meta Hermes does not have a wallet. It does not self-fund. Three options for who pays:

1. Local Hermes 4.3 (free, runs on owner's hardware)
2. Owner provides an API key (the simple option)
3. The clade's most profitable golem sponsors Meta Hermes inference via Bankr (the self-funding option, where the clade subsidizes its own management layer)

Most owners will run local Hermes 4.3 for Meta Hermes. The latency is higher than cloud (2-5 seconds for T1, 10-30 seconds for T2 on a Mac Mini), but the cost is zero and the privacy is total.

### L2: No inference

L2 is a protocol. Smart contracts execute deterministically. Styx APIs run backend logic. No LLM is involved at L2. Pricing formulas, escrow logic, and settlement are all deterministic computation.

### Budget allocation across levels

For a typical clade of 5 golems with local Hermes 4.3 on a Mac Mini:

| Level | Daily inference cost | What it covers |
|---|---|---|
| L0 (per golem, x5) | $0.10-0.50 each | Skill creation (T1) during `on_after_turn`, skill evolution (T2) during dream cycles. Most hooks cost $0. |
| L1 (Meta Hermes) | $0 (local) or $0.05-0.20 (cloud) | Owner conversations, Library curation, clade aggregation. Frequency depends on owner engagement. |
| L2 (marketplace) | $0 (protocol fees, not inference) | Styx 5% on skill sales, escrow gas. No inference cost. |
| **Total** | **$0.50-2.70/day** | Clade of 5 golems with active owner |

For a Fly.io deployment without local inference, substitute Venice/Bankr for the T1/T2 calls. Daily cost rises to $1-5 per golem depending on deliberation frequency.

---

## Event Fabric integration

### HermesEvent variants

L0 Golem Hermes emits events through the Event Fabric as `GolemEvent::Hermes(HermesEvent)`. These events flow to all connected surfaces: TUI screens, web portal, Telegram bot.

```rust
pub enum HermesEvent {
    SkillsLoaded { count: usize },
    SkillsActivated { tick: u64, skills: Vec<String> },
    SkillDrafted { tick: u64, trigger: String },
    SkillCreated { name: String, tick: u64, source_episodes: Vec<String> },
    SkillImproved { name: String, improvement_summary: String },
    FeedbackFlushed { count: usize },
    SkillPromoted { name: String, confidence: f64 },
    SkillIngested { name: String, source: String, confidence: f64 },
    DeathSkillExport { skills_exported: usize },
    EvolutionCycleComplete {
        skills_evolved: usize,
        fitness_improved: f64,
        duration_secs: u64,
    },
}
```

Each variant triggers specific TUI rendering:

| Event | TUI behavior | Sound |
|---|---|---|
| `SkillsLoaded` | Hermes screen shows "N skills loaded" | None |
| `SkillsActivated` | Hearth decision ring shows skill name in cyan | Soft chime (pitch varies per skill) |
| `SkillDrafted` | Hermes screen: new draft appears in feed | None |
| `SkillCreated` | Skill appears in library panel | Rising arpeggio |
| `SkillImproved` | Skill entry updates with diff indicator | Harmonic resolution |
| `FeedbackFlushed` | Counter increments on learning dashboard | None |
| `SkillPromoted` | Skill moves from per-golem to clade tab | Resonant bell |
| `SkillIngested` | New entry appears with source tag | None |
| `DeathSkillExport` | Skills flow animation to Library | Solemn low drone |
| `EvolutionCycleComplete` | Dream screen shows evolution summary | Low synth pad with ascending melody |

### Cross-level event propagation

Events flow upward, never downward.

L0 events emit from Golem Hermes through the golem's Event Fabric. If the golem is remote, these events travel via Styx WebSocket relay to the TUI. If embedded (local), they flow in-process.

L1 (Meta Hermes) listens to all L0 events from all connected golems. It aggregates them: "3 golems created skills this cycle," "golem-beta's LP rebalance skill was cross-validated by golem-gamma." Meta Hermes does not emit `HermesEvent` variants. It communicates with the owner through the conversation interface and with golems through steers.

L2 has no events in the Hermes event space. Marketplace events (skill listed, skill purchased, review submitted) are separate and come through Styx notifications, not the golem Event Fabric.

### Event-driven vs polling patterns

L0-to-TUI: event-driven. Every Hermes action emits an event. The TUI subscribes to the event stream and renders reactively. No polling.

L1-to-L0: polling for aggregation, event-driven for routing. Meta Hermes polls Golem Hermes instances periodically (every 5 minutes) for skill aggregation. But when the owner sends a message that needs routing, Meta Hermes sends it immediately via Styx. Skill promotion notifications from L0 arrive as events.

L1-to-L2: polling. Meta Hermes checks marketplace listings and purchase notifications on a schedule, not reactively. Marketplace activity is low-frequency and does not need real-time event delivery.

---

## Local Hermes 4.3 via Ollama

### The model

Hermes 4.3 is a NousResearch model. It ships with built-in tool calling, `<think>` reasoning traces, and the Hermes Agent skill infrastructure. For Bardo, it runs locally via Ollama.

Model specification:

| Property | Value |
|---|---|
| Model | NousResearch/Hermes-4.3-36B |
| Parameters | 36B |
| Preferred quantization | Q6_K (~32GB VRAM/RAM) |
| Fallback quantization | Q4_K_M (~22GB) for machines with less memory |
| Context window | 128K tokens |
| Inference speed (Mac Mini M4 Pro, Q6_K) | ~15-25 tok/s (T1), ~8-15 tok/s (T2 with `<think>`) |

### Model management

The `bardo model` CLI handles the full lifecycle:

```bash
bardo model list          # Show installed quantizations
bardo model pull Q6_K     # Download a quantization (resumable, shows ETA)
bardo model switch Q8_0   # Switch active quantization (triggers model reload)
bardo model benchmark     # Run inference speed test against standard prompts
```

On first setup, `bardo setup` detects hardware (CPU, RAM, disk, GPU) and recommends a quantization. 64GB Mac Mini gets Q6_K. 32GB gets Q4_K_M. 16GB gets a warning that local inference will be slow and cloud providers are recommended.

Model switching is live. The TUI's Infra window (accessible via Tab) shows current quantization, token generation speed as a sparkline, context utilization, and VRAM usage. Keys `1`/`2`/`3` switch between installed quantizations. The switch triggers a model reload with a progress bar. Inference requests during reload queue and execute after the new model loads.

### When local vs remote

Local Hermes 4.3 is the right choice when:

- You have a Mac Mini M4 Pro or better (or a VPS with GPU)
- Privacy matters (all inference stays on your hardware)
- You want zero marginal inference cost
- Latency tolerance is 2-5 seconds for T1, 10-30 seconds for T2

Remote providers (Venice, Bankr, OpenRouter) are better when:

- Hardware cannot run 36B models at acceptable speed
- You need T2-quality inference faster than local can deliver
- Running on Fly.io micro VMs (256MB RAM, no GPU)
- Multiple golems need simultaneous inference and local cannot keep up

### Health monitoring

The Infra window tracks four metrics for local Hermes:

**VRAM usage.** Displayed as a bar chart. Q6_K at 36B uses ~32GB. If the machine has 64GB unified memory, there is room for one concurrent model.

**Inference speed.** Token generation rate in tok/s, displayed as a braille sparkline over the last hour. A sudden drop suggests thermal throttling or competing processes.

**Quality checks.** Periodic spot checks where a known-good prompt is sent to the local model and the response is scored against expected output. If quality degrades (model corruption, quantization issues), the health monitor flags it and recommends re-downloading or switching to cloud.

**Context utilization.** How much of the 128K context window is being used per inference call. Most T1 calls use 4-12K tokens. T2 deliberation calls can use 20-50K. Dream cycle calls (Skill Evolution) can approach the full window.

### Multi-model local stack

A 64GB Mac Mini can run two models concurrently:

- Hermes 4.3 Q6_K (~32GB) for T2 deliberation and Skill Evolution
- Qwen 2.5 7B Q8_0 (~8GB) for T1 routine operations and Hermes Agent skill creation

Ollama supports concurrent models. The inference gateway routes T1 to the small model and T2 to Hermes 4.3. This halves T1 latency (small models generate faster) while keeping T2 quality high.

The configuration:

```toml
[inference.local-hermes]
endpoint = "http://localhost:11434/v1"
model = "hf.co/NousResearch/Hermes-4.3-36B-GGUF:Q6_K"
tier = "T2"

[inference.local-qwen]
endpoint = "http://localhost:11434/v1"
model = "qwen2.5:7b-q8_0"
tier = "T1"
```

### `<think>` trace capture

When Hermes 4.3 runs locally, its `<think>` reasoning blocks are visible. Cloud providers strip these. Local inference preserves them.

Hermes captures every `<think>` block as a Grimoire episode:

```rust
fn process_hermes_response(response: &str, tick: u64, tier: CognitiveTier) -> Result<(String, Vec<EpisodeId>)> {
    let think_re = Regex::new(r"<think>([\s\S]*?)</think>")?;
    let mut episode_ids = Vec::new();

    for cap in think_re.captures_iter(response) {
        let reasoning = cap[1].trim();
        let episode = GrimoireEntry {
            category: EntryCategory::Episode,
            content: format!("[{} reasoning, tick {}] {}", tier, tick, reasoning),
            confidence: match tier {
                CognitiveTier::T1 => 0.4,
                CognitiveTier::T2 => 0.6,
                _ => 0.3,
            },
            source: "local-hermes-4.3-think".into(),
            propagation: PropagationPolicy::Private,
            ..Default::default()
        };
        let id = grimoire.write_entry(episode)?;
        episode_ids.push(id);
    }

    let clean = think_re.replace_all(response, "").trim().to_string();
    Ok((clean, episode_ids))
}
```

These captured traces become available to the dream cycle. During NREM Replay, the golem can replay its own reasoning traces, discovering patterns in HOW it thinks. "I always check gas price before LP operations" might crystallize into a skill through this process. The golem is, quite literally, dreaming about its own thoughts.

This is unique to local inference. Cloud-hosted models do not expose reasoning traces. Running local Hermes 4.3 gives golems access to a metacognitive dimension that cloud-only deployments miss.

---

## Open design questions

**Process weight on micro VMs.** Python + Hermes Agent + FTS5 database costs 200-500MB RAM. A Fly.io micro VM has 256MB. This may require the small VM tier ($0.065/hr) minimum, or a "Hermes-lite" mode that strips Python dependencies and runs a simplified skill engine in Rust.

**Multi-model local stack scheduling.** When both Hermes 4.3 and Qwen 2.5 are loaded, Ollama's concurrent model scheduling is FIFO. A T2 deliberation call that takes 30 seconds blocks T1 calls behind it. The inference gateway may need to use separate Ollama instances (separate ports) for true parallelism.

**Meta Hermes inference budget.** Who pays? Local is free. But if the owner wants cloud-quality T2 for complex cross-golem analysis, someone needs an API key or a wallet. The cleanest answer is "the clade funds its own management layer," where the most profitable golem sends x402 payments for Meta Hermes inference. But this requires the golem to recognize and approve the expense, which complicates the autonomy model.

**Skill versioning and rollback.** When Hermes self-improves a skill and the new version performs worse, the golem needs to roll back. Git-style versioning for skills (each improvement is a commit, rollback reverts to the previous version) would enable safe experimentation. Meta Hermes could track skill version performance across the clade and auto-rollback underperforming versions.

**Atropos RL integration.** Hermes Agent includes trajectory generation and RL training infrastructure via the Tinker-Atropos submodule. Generating training trajectories from golem decision traces, fine-tuning Hermes 4.3 on successful DeFi reasoning, and deploying the fine-tuned model back to the clade would close the ultimate learning loop. But it requires GPU compute for training that a Mac Mini cannot provide. This may be a Bardo Compute feature rather than a local one.

---

## Cross-reference index

| Topic | Document | Description |
|---|---|---|
| Golem container physical layout | `../01-golem/01-runtime.md` | Micro VM container structure where L0 Hermes runs as a sidecar |
| Inference deployment modes | `../05-oracle/01-prediction-engine.md` | T0/T1/T2 inference tier routing and cost optimization |
| Library of Babel architecture | `../04-memory/02-library-of-babel.md` | Cross-Golem knowledge sharing layer that L1 aggregates |
| Grimoire knowledge representation | `../04-memory/01-grimoire.md` | Per-Golem persistent knowledge base (episodes, insights, heuristics) |
| CorticalState and affect modulation | `../03-daimon/01-affect-engine.md` | PAD emotional state that modulates L0 retrieval relevance |
| Heartbeat and decision cycle | `../01-golem/02-heartbeat.md` | 9-step decision loop that triggers L0 inference each tick |
| Styx coordination and relay | `../20-styx/00-architecture.md` | Cross-Golem communication fabric for Clade coordination |
| Mortality and behavioral phases | `../01-golem/05-mortality.md` | Death clocks and five behavioral phases that constrain inference budgets |

---
