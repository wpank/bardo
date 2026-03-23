# 07 -- Venice-Augmented Dreaming [SPEC]

**Private Cognition During Offline Learning — Why Dreams Must Be Secret**

> Related: [01-architecture.md](01-architecture.md) (dream cycle), [../12-inference/03-venice-provider.md](../12-inference/03-venice-provider.md) (Venice provider), [03-imagination.md](03-imagination.md) (counterfactual engine), [04-consolidation.md](04-consolidation.md) (Grimoire integration), [../02-mortality/06-thanatopsis.md](../02-mortality/06-thanatopsis.md) (death testament)

---

> **Reader orientation:** This document specifies how Venice (a privacy-preserving inference provider using TEEs) augments the dream cycle within Bardo (the Rust runtime for mortal autonomous DeFi agents). Dream content reveals strategy evolution and future plans -- the most sensitive cognitive artifact a Golem (mortal autonomous agent) produces. This spec covers private inference routing per dream phase, NREM private replay, REM web-augmented imagination, TEE-attested integration, DIEM staking, and death testament privacy. Prerequisites: dream architecture (`01-architecture.md`) and the Venice inference provider (`../12-inference/03-venice-provider.md`). For a full glossary, see `prd2/shared/glossary.md`.

## Why dreams must be private

**Moat framing:** Dream content reveals strategy evolution, risk assessment, and future plans. It is the compressed representation of everything the Golem has learned and everything it intends to try next. Private inference through Venice's TEE ensures this content never leaks -- not to the inference provider, not to competitors, not to anyone outside the Golem's trust boundary.

Dream content is the most sensitive cognitive artifact a Golem produces.

During waking operation, the golem's inference inputs are largely reactive — market data, tool results, position state. These are observable facts that other market participants can access independently. The golem's inference output (trade decision, LP range adjustment) lands on-chain within seconds, at which point it is public.

Dream content is different. A golem's dream is a compressed representation of its accumulated experience: replayed episodes from months of trading, counterfactual hypotheses about strategies it has not yet tested, threat simulations modeling its own attack surface, and emotional annotations marking which outcomes produced the most surprise or distress. This content is not observable from on-chain activity. It exists only inside the golem's cognition.

The implications are direct:

**Episode replay contains strategy fingerprints**: A golem replaying its highest-utility episodes during NREM is processing the trades that most shaped its strategy. These episodes, if readable, reveal which market conditions the golem weights most heavily, which protocols it trusts, and which risk signals it acts on. A competitor with access to this replay content can model the golem's behavior with far higher fidelity than any on-chain analysis allows.

**REM imagination contains unrealized strategy**: The golem's dream imagination generates hypotheses about strategies it has not yet deployed. These are ideas in early formation — often the most valuable ideas a golem has, precisely because they have not yet been competed away by on-chain implementation. A retention-capable provider holding REM outputs is holding the golem's research pipeline.

**Emotional annotations reveal vulnerability**: The daimon's high-arousal episode annotations mark exactly which outcomes the golem responded to with the most intensity — the trades that produced fear, regret, or euphoria. These annotations are a psychological profile. They identify the decision contexts where the golem is most likely to deviate from its baseline strategy under pressure.

**Dead golems' dreams are permanent**: A golem's death testament is derived directly from its final dream cycle. Testament content — heuristics, decisions, final strategy state — is the golem's intellectual legacy. If this content is retained by an inference provider, the golem's successor cannot claim that legacy as genuinely private.

**Hypnagogic content is uniquely sensitive**: The onset and return phases (Track 5, `../06-hypnagogia/`) process the transitions between waking and dreaming cognition. Onset content contains the golem's unresolved waking threads -- active reasoning traces about live positions, half-formed hypotheses about market conditions, and emotional residue from recent ticks. This is a snapshot of the golem's cognitive state at its most unguarded: not yet structured for dream processing, not yet filtered for safety. Return content is similarly sensitive -- it reveals which dream insights the golem selected for waking promotion, exposing its near-term strategic intentions before any on-chain action. Both phases route through Venice private (no-log) by default.

Venice's no-log inference guarantees contain dream content within the golem's trust boundary. This is not a preference -- it is a structural requirement for the dream system to function as designed.

---

## Venice Helps Agents Dream

The tagline's technical grounding: Venice is not merely a private version of another inference provider. It adds capability that the Bardo BlockRun gateway cannot provide in the dream context.

**Web search without browsing fingerprint**: During REM imagination, the golem needs current information — not to trade on it in real time, but to ground its counterfactual scenarios in present reality rather than stale oracle data. Without external information, a golem dreaming in January about yield strategies is reasoning from December market conditions. The longer the golem has been running, the more stale its base knowledge becomes.

Venice's integrated web search solves this without introducing a second external service. A golem using a separate browsing provider would leave a browsing trail at that provider: a sequence of queries revealing exactly what the golem is researching during its sleep window. Venice web search is subject to the same no-log guarantee as the inference itself — the query and its results are private.

**No dream contamination from search**: Knowledge retrieved during REM web search enters the dream journal as citations, not as direct Grimoire writes. The citation carries a lower base confidence (0.45) than self-generated insights (0.3 base, with live validation pathway). Web-retrieved knowledge requires confirmation from live market conditions before it can influence the golem's operational strategy — it cannot be injected directly into PLAYBOOK.md. This prevents a compromised or misleading web result from corrupting the golem's policy.

**Grounded threat simulation**: Venice web search enables the golem to fetch real vulnerability disclosures, governance change proposals, and competitor announcements during threat simulation. A threat simulation using only historical data models past risks. One augmented with current disclosures models present risks. The difference, for a golem managing a live vault, is the difference between historical VaR and stress testing against the current threat landscape.

---

## Dream Inference Routing

All three dream phases route through Venice. The tier selection follows the sensitivity of the cognition being performed:

| Dream Phase                | Privacy Tier     | Rationale                                                                                     |
| -------------------------- | ---------------- | --------------------------------------------------------------------------------------------- |
| Hypnagogic onset           | private (no-log) | Waking residue contains active strategy threads and unresolved reasoning traces               |
| NREM (replay)              | private (no-log) | Episodes contain strategy history; no function calling in replay                              |
| REM (imagination)          | private (no-log) | Web search enabled; TEE not available at REM (function calling required for web search tools) |
| Integration                | tee              | Insight promotion decisions are high-stakes; attestation receipt provides provenance          |
| Hypnopompic return         | private (no-log) | Return recrystallization exposes which dream insights will become operational knowledge       |

E2EE is never used during dreaming. Dream phases require function calling — specifically, `query_grimoire` and `search_context` tools available in the dream branch. E2EE disables function calling by architectural necessity (the host cannot parse the encrypted context to dispatch tool calls). The `private` tier provides the relevant guarantee for NREM and REM: no retention, no logging, no training use.

Integration phase uses TEE for a specific reason: promoted insights become operational knowledge. The Grimoire's admission gate for dream-sourced insights includes the attestation receipt as provenance. An insight promoted from an attested integration phase carries a verifiable record of when, in which model and enclave, the promotion decision was made. This is the insight's provenance token — it cannot be forged after the fact.

---

## NREM Phase: Private Replay

NREM replay prompts contain compressed episode batches from the Grimoire. These batches represent the golem's significant trading history in a form that the LLM can analyze for patterns. The privacy requirement is non-negotiable: episode content includes position sizes, entry/exit conditions, PnL outcomes, and regime annotations. Together these constitute a complete picture of the golem's realized strategy.

Venice routing for NREM:

```rust
let nrem_options = VeniceRequestOptions {
    privacy_tier: PrivacyTier::Private,
    task_class: "dream_nrem".into(),
    prompt_cache_key: Some(format!("{golem_id}:dream:{session_epoch}")),
    venice_parameters: VeniceParameters {
        include_venice_system_prompt: false,
        enable_web_search: WebSearchMode::Never,
        ..Default::default()
    },
};
```

The `promptCacheKey` is set per dream session, not per tick. All NREM replay calls within a single dream cycle share the same cache key — the system prompt (golem policy, current PLAYBOOK.md context) and tool set are stable across the full cycle. Only the episode batch in the volatile tail changes. This produces cache hit rates of 60–75% on the stable prefix across NREM iterations, significantly reducing cost for longer dream windows.

NREM is T1 inference by default — episode replay requires pattern recognition and causal reasoning, but not deep deliberation. T2 escalation applies only to episodes flagged `dream_priority: "high"` (curator-tagged, high-arousal, or high-surprise) where the LLM is asked to resolve a specific contradiction or explain an anomalous outcome.

---

## REM Phase: Web-Augmented Imagination

REM imagination is where Venice's web search capability becomes load-bearing.

The golem's REM prompts instruct the LLM to generate novel scenarios, counterfactual strategies, and threat simulations. Without current market information, these scenarios are constrained to the golem's knowledge cutoff — effectively the last time it processed a live data source. With Venice web search, the golem can retrieve:

- Current governance proposals for protocols it interacts with
- Recent exploit disclosures and vulnerability reports
- Protocol parameter changes (fee tiers, liquidity migration, oracle changes)
- Competitor golem activity patterns (inferred from on-chain data)
- Macro market conditions and sentiment indicators

Venice routing for REM:

```rust
let rem_options = VeniceRequestOptions {
    privacy_tier: PrivacyTier::Private,
    task_class: "dream_rem".into(),
    prompt_cache_key: Some(format!("{golem_id}:dream:{session_epoch}")),
    venice_parameters: VeniceParameters {
        include_venice_system_prompt: false,
        enable_web_search: WebSearchMode::Auto, // model decides when to search
        enable_web_citations: true,
        ..Default::default()
    },
};
```

`enable_web_search: "auto"` lets the model decide when to search. Not every REM call needs live data — purely combinational creativity (recombining existing episodes) does not. Exploratory and threat simulation calls are where search typically triggers. The model issues search queries as part of its generation; Venice handles the retrieval and injects results inline.

**Query privacy**: Web search queries issued during REM are subject to Venice's no-log guarantee on the same terms as the inference call itself. The golem does not expose its research agenda to a separate search provider with different retention policies.

**Citation capture**: All web-sourced citations from REM are captured in the dream journal:

```rust
pub struct WebSearchCitation {
    pub query: String,
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub retrieved_at: u64,
    pub dream_phase: DreamPhase, // always Rem
    pub confidence: f64,          // base: 0.45
    pub source: String,           // "venice-web-search"
}
```

Citations are stored in the dream journal entry but do not directly enter the Grimoire. They enter the staging buffer as insights with confidence 0.45 and `source: "venice-web-search"`. Promotion to the Grimoire requires the same live-market validation path as other staged insights — the confidence must reach 0.7 through confirmed live-market episodes. Web-sourced knowledge can inform the golem's hypotheses, but cannot bypass the validation gate.

**REM inference tier**: T1 for most creative recombination, T2 for high-uncertainty threat simulation and complex counterfactuals. The REM phase budget allocates ~30% of dream cycle cost, with web search adding ~15–25% overhead depending on query volume.

---

## Integration Phase: Attested Dream Insights

The integration phase is where dream outputs become operational. Insights from NREM and citations from REM are evaluated, classified, and either promoted to the Grimoire staging buffer or discarded.

This phase is the highest-stakes cognition in the dream cycle. The integration phase determines whether a novel hypothesis enters the golem's operational knowledge. A wrong promotion — a false pattern treated as a confirmed insight — can degrade strategy quality for weeks until live-market contradictions accumulate. Attestation provides a defense-in-depth layer: if a promoted insight later proves harmful, the attestation receipt establishes exactly what model, in which enclave, made the promotion decision.

Venice routing for integration:

```rust
let integration_options = VeniceRequestOptions {
    privacy_tier: PrivacyTier::Tee,
    task_class: "dream_integration".into(),
    prompt_cache_key: Some(format!("{golem_id}:dream:{session_epoch}")),
    venice_parameters: VeniceParameters {
        include_venice_system_prompt: false,
        enable_web_search: WebSearchMode::Never, // analytical, not generative
        ..Default::default()
    },
};
```

On successful attestation verification, the Grimoire write for each promoted insight includes the receipt:

```rust
pub struct DreamInsightGrimoireEntry {
    pub entry_type: String,             // "insight"
    pub content: String,
    pub confidence: f64,                // initial: 0.3
    pub source: String,                 // "dream_integration"
    pub dream_cycle_id: String,
    pub attestation_receipt: AttestationProvenanceToken,
    pub web_citations_included: Vec<String>, // IDs of REM citations that informed this
    pub validation_criterion: String,        // condition under which confidence increments
}
```

The confidence floor for TEE-promoted insights is 0.3 (same as standard dream insights). Attestation does not raise the initial confidence — it provides provenance, not truth. A TEE-attested hallucination is still a hallucination. Attestation records that the promotion decision was made by the correct system; live-market validation confirms whether the insight is accurate.

---

## DIEM staking: zero-cost dream inference

Venice allows zero-cost inference for DIEM token stakers. A Golem (or its owner) that stakes DIEM tokens gets free Venice inference for dream processing. This makes dreaming essentially free for staked Golems -- the inference cost that otherwise constrains dream cycle frequency and depth drops to zero.

The economic implication is significant. Without DIEM staking, a [HARDENED] dream cycle costs $0.20-$0.45 in inference. At 1-3 cycles per day, monthly dream cost is $6-$40. With DIEM staking, all of that becomes free. The Golem can dream as frequently and deeply as its scheduling logic demands without inference cost pressure.

**Configuration**: When `dream_venice_config.diem_staked: true` (detected from the Venice API key's staking status), the dream budget system skips cost tracking for Venice calls. The budget caps still enforce turn limits and phase durations to prevent runaway dream cycles, but cost-based suppression (the 15% LLM partition floor) does not apply.

## Extended thinking in dreams

Venice supports extended thinking (analogous to Claude's `thinking` blocks). For dreams, the thinking response IS the valuable output -- it contains the reasoning chain, the hypothesis formation process, and the intermediate considerations that led to each conclusion.

Set `strip_thinking_response: false` in the dream Venice config to capture this reasoning chain. The default for waking inference is `true` (strip the thinking, keep only the final answer). For dreams, the default is `false` -- the DreamJournal records the full reasoning chain, which is more valuable for successor inheritance and owner review than the final conclusion alone.

The thinking content is stored in the DreamJournalEntry's `reasoning_trace` field and is available for inspection through the dream API endpoints. It is NOT fed back into subsequent inference calls (that would leak dream reasoning into context windows of future calls).

## TEE attestation for integration phase

Venice runs inference inside Trusted Execution Environments. During the Integration phase (when dream insights are promoted to the Grimoire staging buffer), the Golem verifies via TEE attestation that the dream reasoning was computed privately -- no one, not even Venice, can read the dream content.

The attestation receipt is stored alongside each promoted insight in the Grimoire. This creates a verifiable chain of provenance: "this insight was generated during dream cycle X, in TEE enclave Y, using model Z, at time T." If an insight later proves harmful and the owner needs to audit how it entered the Golem's operational knowledge, the attestation receipt provides forensic evidence that the promotion decision was made by the correct system under the expected privacy guarantees.

Attestation does not make hallucinations more true. A TEE-attested wrong answer is still wrong. But it proves the answer was computed privately and by the expected model, which matters for trust in the dream pipeline.

---

## Dream budget allocation by phase

Venice inference has a cost premium over Bardo routing (~20–40% for TEE, ~0–10% for private). The dream budget accounts for this explicitly:

| Dream Phase             | Tier    | Base Cost/Call | Venice Premium | Effective Cost/Call |
| ----------------------- | ------- | -------------- | -------------- | ------------------- |
| NREM T1                 | private | ~$0.003        | ~5%            | ~$0.003             |
| NREM T2 (escalated)     | private | ~$0.025        | ~5%            | ~$0.026             |
| REM T1                  | private | ~$0.003        | ~5%            | ~$0.003             |
| REM T1 + web search     | private | ~$0.008        | ~5%            | ~$0.008             |
| REM T2 (counterfactual) | private | ~$0.025        | ~5%            | ~$0.026             |
| Integration T2          | tee     | ~$0.025        | ~30%           | ~$0.033             |

Per-cycle cost cap is enforced via consumption-limit Venice keys (v2) or application-level budget tracking (v1). The cap applies per phase:

```toml
# In bardo.toml
[dream.venice]
nremBudgetUsdc = "$0.20"       # NREM phase cap per cycle
remBudgetUsdc = "$0.25"        # REM phase cap per cycle (includes web search overhead)
integrationBudgetUsdc = "$0.10" # Integration phase cap per cycle
webSearchBudgetPerCycle = "$0.05" # Separate cap for web search overhead
```

When a phase budget is exhausted mid-cycle, the current phase terminates and integration proceeds on the outputs collected to that point. A partial REM cycle with 4 counterfactuals generates a valid integration input — the dream quality score reflects the partial completion.

---

## Cross-Dream Knowledge Retrieval via Venice Embeddings

The Grimoire's primary embedding model is local `nomic-embed-text-v1.5` (offline, no external call). This works well for within-Golem retrieval. Cross-Golem retrieval -- when a Golem queries Styx Lethe (formerly Lethe) for insights from peer Golems -- faces an embedding space compatibility problem: two Golems that trained their local models on different episodes may have drifted embedding spaces.

Venice embeddings provide an optional bridge:

```rust
pub struct VeniceEmbeddingOptions {
    pub model: String,                    // "venice-embed-v1"
    pub use_for_styx_queries: bool,       // default: false (local model primary)
    pub use_for_cross_golem_compare: bool, // default: true when Styx Lethe enabled
}
```

When `use_for_cross_golem_compare: true`, Styx Lethe queries embed the query using Venice's model rather than the local model. The returned candidate insights were embedded by other Golems -- also using Venice embeddings if they enabled the option. This shared embedding space enables semantic similarity comparison that is meaningful across Golem instances rather than relative to one Golem's local model history.

Venice embeddings are only called during Styx Lethe queries, not during standard Grimoire retrieval. The cost overhead is small (embeddings are cheap) and the call is private (subject to Venice no-log policy). Local embeddings remain the default for all within-Golem operations.

---

## Death Testament Privacy

The final dream cycle before Thanatopsis is the most privacy-sensitive dream the golem ever runs. Testament generation contains:

- The golem's complete strategy state as it stood at end-of-life
- All PLAYBOOK.md entries that survived validation
- High-confidence Grimoire insights selected for succession inheritance
- Emotional load annotations marking the golem's most significant experiences
- Final hypothesis set — ideas the golem never got to test

All of this is routed through Venice private (no-log). The testament is never transmitted to an external service in readable form. If the owner has configured sealed mode for the testament, E2EE is used for the generation call -- the testament content is encrypted before leaving the Golem's process and only the Golem (or its designated successor, via key handoff) can decrypt it.

```rust
let testament_options = VeniceRequestOptions {
    privacy_tier: if sealed_mode { PrivacyTier::E2e } else { PrivacyTier::Private },
    task_class: "testament".into(),
    prompt_cache_key: None, // no caching for testament (unique, terminal)
    venice_parameters: VeniceParameters {
        include_venice_system_prompt: false,
        enable_web_search: WebSearchMode::Never,
        character_slug: None, // no character framing for testament
        ..Default::default()
    },
};
```

The testament is stored locally in the Golem's Grimoire. What enters the Styx Lethe index is an identity-stripped hash -- not the raw content. The hash enables Styx to confirm that a testament exists and that its content has not been modified, without exposing the content to the query layer.

If the golem dies without completing its final dream cycle (interrupted death, emergency halt), the testament is derived from the most recent available dream journal entry. The testament quality score reflects this — a golem that ran a full final dream cycle has a higher-quality testament than one that was interrupted.

---

## Dream Scheduling Interaction

Venice web search creates a new urgency trigger for dream scheduling. Three new conditions boost dream urgency:

**Stale oracle detection**: If the Grimoire's oracle data is older than `oracleStalenessThreshold` (default: 24 hours for protocol parameters, 4 hours for market data), the urgency scorer adds 0.2 to the dream urgency score. A dream with web-augmented REM can refresh this knowledge without any waking tool calls.

**Threat warning from sleepwalker**: If the sleepwalker observer publishes a threat warning to the event bus (governance change, protocol exploit, large position migration), the dream urgency score receives a 0.3 boost. The next eligible dream cycle will prioritize web-augmented REM with threat simulation focused on the flagged condition.

**Governance change detection**: Protocol governance proposals affecting the golem's active strategies trigger a 0.25 urgency boost. The golem needs to understand the proposed change and model its impact before the proposal passes.

These triggers are additive with the standard urgency components. A golem with stale oracle data, a pending governance vote, and a recent threat warning accumulates enough urgency to trigger an unscheduled dream even at the high threshold used for Thriving golems (≥0.8).

```rust
pub fn dream_urgency_with_venice(state: &GolemState) -> f64 {
    let base_urgency = dream_urgency(state); // existing formula

    let venice_boost =
        (if state.oracle_staleness_hours > ORACLE_STALENESS_THRESHOLD { 0.2 } else { 0.0 })
        + (if state.pending_threat_warnings > 0 { 0.3 } else { 0.0 })
        + (if state.pending_governance_proposals > 0 { 0.25 } else { 0.0 });

    (base_urgency + venice_boost).min(1.0)
}
```

When a Venice-triggered dream fires, the scheduler annotates the dream cycle with the triggering condition. The REM phase receives `webSearchPriority: "high"` and the initial search query is pre-seeded with the relevant trigger context (the threat warning text, the governance proposal URL, or the stale oracle topic). This focuses the web search rather than leaving it entirely to the model's discretion.

---

## Web Search Citation Handling

Web search citations produced during REM follow a distinct lifecycle from self-generated insights:

| Property              | Self-Generated Insight       | Web-Sourced Citation                    |
| --------------------- | ---------------------------- | --------------------------------------- |
| Initial confidence    | 0.3                          | 0.45                                    |
| TTL in staging buffer | 30 days                      | 14 days                                 |
| Validation path       | Live-market episodes         | Live-market episodes (same)             |
| Promotion threshold   | 0.7                          | 0.7 (same)                              |
| Source tag            | `dream_nrem`/`dream_rem`     | `venice-web-search`                     |
| Attestation           | Optional (integration phase) | Not attested (retrieval, not inference) |

The higher initial confidence for web-sourced citations (0.45 vs 0.3) reflects that retrieved facts have an external grounding that self-generated hypotheses lack. A retrieved vulnerability disclosure is a fact (the disclosure exists); the golem's counterfactual hypothesis is a conjecture. However, web citations have a shorter TTL — market conditions change, and a citation about a governance proposal that passed six months ago may be actively misleading as current strategy guidance.

The source tag `venice-web-search` is preserved throughout the citation's lifecycle in the staging buffer and, if promoted, in the Grimoire entry. This enables the golem to reason about how much of its current strategy was informed by web-sourced knowledge versus self-generated learning — a useful diagnostic for strategy validation.

---

## Golem Creation Integration

Dream provider preference is set at creation alongside the main inference provider:

```rust
pub struct GolemCreationConfig {
    // ... existing fields ...
    pub inference_provider: InferenceProvider, // Venice, Bardo, Both
    pub venice_config: Option<VeniceConfig>,
    pub dream_inference_provider: DreamInferenceProvider, // Venice, Bardo
    pub dream_venice_config: Option<DreamVeniceConfig>,
}

pub struct DreamVeniceConfig {
    pub web_search_enabled: bool,
    pub web_search_budget_per_cycle_usdc: f64, // default: 0.05
    pub rem_web_search_mode: WebSearchMode,     // default: Auto
    pub sealed_testament: bool,                 // default: false
    pub integration_phase_attestation: bool,    // default: true (recommended)
    pub strip_thinking_response: bool,          // default: false (capture reasoning chain)
}
```

`dream_inference_provider` defaults to `Venice` if `inference_provider` includes Venice. An owner can configure Venice for dreams only (keeping Bardo for waking inference) by setting `inference_provider: Bardo` and `dream_inference_provider: Venice`. This makes sense for owners who want the privacy and web search benefits for offline learning but prefer Bardo's cost optimization for real-time trading decisions.

`webSearchBudgetPerCycleUsdc` caps the web search overhead per dream cycle. This is a separate cap from the inference budget — web search charges by query, not by token. The default ($0.05/cycle) allows roughly 3–5 searches per REM phase without material impact on dream economics.

Dream provider preference can be changed post-creation without restart. The change takes effect at the next dream cycle start.

---

## Dream Cycle Visualization

The dream cycle is not observable from on-chain activity — it happens between ticks, offline. The debug UI fills that gap with a real-time timeline view driven by SSE events from `/venice/stream`.

### Dream Cycle Timeline

A horizontal timeline spanning the current cycle's three phases:

```
|=== NREM (replay) ====|==== REM (imagination) ====|= Integration =|
  12 calls · 63% cache    8 calls · 4 web searches     1 call · TEE
  $0.08 / $0.20 cap       $0.11 / $0.25 cap            $0.04 / $0.10
```

Each phase renders as a colored segment filling left-to-right as cognition events arrive:

- **NREM** — blue segment, advances on each `dream_nrem` cognition event
- **REM** — violet segment, with small dot markers for each web search trigger
- **Integration** — teal segment, with a shield icon indicating TEE attestation

Within each segment the display shows: duration elapsed (counting up during the active phase), Venice calls made, cache hit rate as a rolling average, and USDC consumed vs cap from the most recent `budget` event for that phase.

### SSE connection

The timeline subscribes to `/venice/stream` via `useVeniceStream`. Three event types drive it:

- `cognition` events with `class: "dream_nrem" | "dream_rem" | "dream_integration"` advance the active phase's call counter
- `budget` events with `phase: "dream_nrem" | "dream_rem" | "dream_integration"` update the consumed/cap display
- `cache-stats` events update per-phase cache hit rate

Phase transitions are inferred from the event sequence — a `dream_rem` event following a block of `dream_nrem` events signals that NREM completed and REM started.

### Terminal states

**Budget cap hit**: when a `budget` event shows `remaining ≤ 0.05 * cap` for a phase, that segment turns amber. When the next phase starts, the capped segment stays amber with a "budget cap" label — the partial completion is preserved as historical fact.

**Golem enters Moribund**: if the golem's heartbeat FSM signals terminal phase, the entire timeline grays out with a death timestamp overlaying the center. The dream in progress at entry to Moribund is marked as the final dream.

### Integration phase detail

The integration segment shows a secondary row with insight outcomes:

```
Integration | 1 call · TEE · 47ms
             Promoted: 3  Staged: 8  Discarded: 2
```

These counts come from the `dream_integration` cognition event payload as metadata fields — not content. The balance between promoted and staged signals dream quality: a cycle that produces many staged insights with few promotions means the integration phase was conservative, which is expected and fine.

### Mock mode

In mock mode, the timeline runs a synthetic dream cycle: NREM for 30 seconds (12 synthetic calls), REM for 20 seconds (8 calls, 4 simulated web searches), Integration for 5 seconds (1 call, 3 promoted), then resets. This loop gives operators a concrete picture of what a real dream cycle looks like before any golem is running.

---

## Cross-References

| Topic                           | Document                                                                       |
| ------------------------------- | ------------------------------------------------------------------------------ |
| Dream cycle architecture        | [01-architecture.md](01-architecture.md)                                       |
| Venice provider spec            | [../12-inference/03-venice-provider.md](../12-inference/03-venice-provider.md) |
| Counterfactual reasoning        | [03-imagination.md](03-imagination.md)                                         |
| Dream consolidation and staging | [04-consolidation.md](04-consolidation.md)                                     |
| Threat simulation               | [05-threats.md](05-threats.md)                                                 |
| Grimoire provenance tokens      | [../04-memory/01-grimoire.md](../04-memory/01-grimoire.md)                     |
| Death testament                 | [../02-mortality/06-thanatopsis.md](../02-mortality/06-thanatopsis.md)         |
| Styx query                      | [../20-styx/01-architecture.md](../20-styx/01-architecture.md)                 |
| Sleepwalker observer            | [../01-golem/15-sleepwalker.md](../01-golem/15-sleepwalker.md)                 |
| Golem creation flow             | [../01-golem/06-creation.md](../01-golem/06-creation.md)                       |
