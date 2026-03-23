# Emergent Capabilities [SPEC]

> **Document Type**: REF (normative) | **Referenced by**: All PRDs | **Last Updated**: 2026-03-18
>
> Capabilities that no single subsystem produces alone. Each entry describes a behavior that emerges when two or more subsystems interact, documents the interaction pattern, and specifies how to verify the emergent behavior is present.

> **Reader orientation:** This document catalogs behaviors that emerge only when multiple Golem (mortal autonomous agent) subsystems interact. No single subsystem produces these capabilities alone. It belongs to the `shared/` reference layer and covers phenomena like the Gestalt Vector (a composite HDC hypervector encoding the Golem's entire cognitive-emotional state), mortality-driven learning acceleration, emotional somatic markers, dream-driven strategy evolution, and swarm intelligence. The key concept is that Bardo's architecture is designed so that subsystem interactions produce capabilities exceeding what any component delivers independently. See `prd2/shared/glossary.md` for full term definitions.

---

## How to read this document

Each capability follows the same structure:

1. **What emerges** -- the behavior itself
2. **Interacting subsystems** -- which components participate and what flows between them
3. **Why it exceeds its parts** -- what the combination produces that no component can produce independently
4. **Verification** -- how to test for the emergent behavior
5. **Cross-references** -- prd2 spec files where each contributing subsystem is defined

---

## 1. Gestalt Vector

### What emerges

A single 10,240-bit BSC hypervector that encodes the Golem's entire cognitive-emotional state at a given instant. Not an average or summary but a compositional binding where the *relationships* between signals are preserved. Any subsystem can query historical gestalt vectors by constructing a query vector and computing similarity in O(1).

### Interacting subsystems

```
CorticalState ──(32 atomic signals)──► HDC Encoder
                                          │
Sheaf Consistency ──(scalar + hv)────────►│
                                          │
Attention Auction ──(allocation hv)──────►│
                                          │
Vitality Clock ──(scalar + hv)───────────►│
                                          │
                                     ┌────▼────┐
                                     │ gestalt  │──► Ring Buffer (1 per theta tick)
                                     │  vector  │──► Similarity Search
                                     └─────────┘
```

CorticalState provides the raw signal fields. The HDC encoder binds each field's hypervector representation using positional permutation, producing a single distributed vector where signal relationships are encoded structurally. The sheaf consistency score, attention allocation distribution, and vitality scalar are bound into the same vector, giving the gestalt a cognitive-emotional-mortality shape.

### Why it exceeds its parts

CorticalState alone is a flat struct of ~32 independent fields. You can check individual values ("what was sheaf consistency at time T?") but you cannot ask relational questions like "when was the Golem last in a state where sheaf consistency was low AND attention was concentrated on the Oracle AND vitality was declining?" The gestalt vector encodes these relationships as geometric structure. Two gestalt vectors with high cosine similarity share the same *pattern* of internal relationships, even if the underlying values differ in absolute terms.

### Verification

- **Similarity test**: Construct two CorticalState snapshots that differ in absolute values but share the same relational pattern (e.g., declining consistency, concentrated attention, rising somatic anxiety). Verify their gestalt vectors have similarity > 0.80.
- **Orthogonality test**: Construct two snapshots with genuinely different cognitive patterns. Verify similarity < 0.55 (quasi-orthogonal baseline).
- **Retrieval test**: Store 30 days of gestalt vectors (~180,000 at theta frequency). Verify a full scan completes in < 5 ms.
- **Predictive value test**: When a gestalt match fires (similarity > 0.85 to a historical state that preceded a drawdown), log the outcome. Over 100 matches, the alert should correlate with subsequent drawdowns at above-chance rates.

### Cross-references

- CorticalState: [`01-golem/18-cortical-state.md`](../01-golem/18-cortical-state.md)
- HDC encoding: [`shared/hdc-vsa.md`](./hdc-vsa.md)
- Sheaf consistency: [`01-golem/03-mind.md`](../01-golem/03-mind.md)
- Attention auction: [`01-golem/14-context-governor.md`](../01-golem/14-context-governor.md)
- Vitality / mortality: [`02-mortality/01-architecture.md`](../02-mortality/01-architecture.md)

---

## 2. Topological-Somatic Intelligence

### What emerges

A feedback loop where topological anomalies detected by persistent homology trigger somatic marker responses in the Daimon, which increase the risk module's attention bids, which redirect cognitive resources toward the anomaly before it manifests as a price move. The Golem develops "gut feelings" about market structure changes.

### Interacting subsystems

```
TDA Pipeline ──(Wasserstein distance)──► Bayesian Surprise
                                              │
                                         (surprise score)
                                              │
                                              ▼
                                         Daimon Somatic
                                          Markers
                                              │
                                         (valence shift)
                                              │
                                              ▼
                                    CorticalState.somatic_valence
                                              │
                                              ▼
                                      Risk Module ──(bid increase)──► VCG Attention Auction
                                                                           │
                                                                    (resource reallocation)
                                                                           │
                                                                           ▼
                                                                   Golem narrows focus
                                                                   to anomalous signal
```

### Why it exceeds its parts

The TDA pipeline alone produces geometric signals (Betti number changes, persistence diagram shifts) that precede statistical signals by 10-100 ticks. But without a mechanism to translate geometric novelty into urgency, these signals sit in a queue. The Daimon alone reacts to portfolio-level metrics (drawdown, volatility, P&L) and has no access to topological structure.

The combination creates a body-level response to geometric market changes. Topological anomaly becomes somatic anxiety becomes attention reallocation. The latency advantage of TDA (detecting structural changes before price moves) is preserved because the pathway is reactive, not deliberative. The Daimon doesn't reason about topology; it feels the surprise score.

### Verification

- **Latency test**: Inject a synthetic topological anomaly (new 1-cycle in a liquidity graph). Measure time from anomaly detection to attention reallocation. Should be < 3 theta ticks.
- **Somatic coupling test**: Verify that TopologicalAnomaly events shift CorticalState.somatic_valence proportionally to the Wasserstein distance magnitude.
- **Attention response test**: After a high-surprise topological event (> 99th percentile Wasserstein distance), verify the risk module's VCG bid increases by at least 30%.
- **Counterfactual test**: Compare response latency with and without the TDA-Daimon coupling. Without it, the Golem reacts only after statistical thresholds fire. Measure the latency difference.

### Cross-references

- TDA regime detection: [`01-golem/17-prediction-engine.md`](../01-golem/17-prediction-engine.md)
- Daimon somatic markers: [`03-daimon/01-appraisal.md`](../03-daimon/01-appraisal.md)
- CorticalState: [`01-golem/18-cortical-state.md`](../01-golem/18-cortical-state.md)
- VCG attention auction: [`01-golem/14-context-governor.md`](../01-golem/14-context-governor.md)
- Bayesian surprise: [`01-golem/17-prediction-engine.md`](../01-golem/17-prediction-engine.md)

---

## 3. Associative Memory Without Embedding Search

### What emerges

O(1) content-addressable recall from the Grimoire using HDC's bind/bundle operations, replacing approximate nearest neighbor (ANN) indices. Queries are compositional: bind the attributes you want, compute Hamming distances, retrieve. No vector database, no HNSW index, no recall degradation as the Grimoire grows.

### Interacting subsystems

```
Query Construction                     Grimoire Storage
       │                                      │
  bind(regime_hv,                         Each entry stored
   outcome_hv,                            with HDC encoding:
   protocol_hv,                           bind(regime_hv,
   pair_hv)                                outcome_hv,
       │                                   protocol_hv,
       ▼                                   pair_hv,
  query_hv ──────(Hamming distance)──────► entry_hv
                                              │
                                         (threshold filter)
                                              │
                                              ▼
                                        Matching entries
                                              │
                                     Witness DAG CID ◄──(provenance link)
```

### Why it exceeds its parts

The Grimoire alone stores episodic memories but retrieves them via embedding-based ANN search. ANN indices are approximate (recall degrades as the index grows) and don't support compositional queries. To find entries matching multiple attributes, you run separate similarity searches and intersect the results.

HDC ItemMemory solves both problems. Each Grimoire entry is encoded at creation time by binding its context attributes into a single hypervector. A query is constructed the same way: bind the desired attributes. Similarity between query and entry is computed by Hamming distance in O(D) where D is the vector dimension, independent of collection size. A compositional query ("high-volatility regime AND positive outcome AND Uniswap v3 WETH/USDC") is a single bind + scan operation instead of three searches plus intersection.

The Witness DAG adds provenance: each entry's DAG CID is permuted and included in the binding, so you can query by provenance chain too.

### Verification

- **Compositional precision test**: Insert 1,000 Grimoire entries with known attributes. Query for entries matching 3+ specific attributes. Verify 100% recall (no false negatives) and < 1% false positive rate.
- **Scaling test**: Verify retrieval time remains constant (within 2x) as collection grows from 100 to 10,000 entries.
- **Provenance query test**: Query for entries linked to a specific Witness DAG CID. Verify correct retrieval.
- **Comparison test**: Run the same query on ANN-based retrieval and HDC-based retrieval. Verify HDC recall >= ANN recall at all collection sizes.

### Cross-references

- HDC operations: [`shared/hdc-vsa.md`](./hdc-vsa.md)
- Grimoire: [`04-memory/01-grimoire.md`](../04-memory/01-grimoire.md)
- Grimoire HDC encoding: [`04-memory/01c-grimoire-hdc.md`](../04-memory/01c-grimoire-hdc.md)
- Witness DAG: [`10-safety/08-witness-dag.md`](../10-safety/08-witness-dag.md)

---

## 4. Memetic Immune System

### What emerges

Automatic resistance to bad knowledge propagation. Grimoire entries must prove their fitness to survive across time: confidence decays, self-prediction scores are tracked, and AntiKnowledge (K-) entries actively mark known-bad patterns. The combination produces an immune system where bad ideas die and dangerous ideas are explicitly walled off.

### Interacting subsystems

```
New Grimoire Entry
       │
       ├──► Confidence Decay (demurrage)
       │         │
       │    (confidence decreases
       │     unless reinforced)
       │
       ├──► Self-Prediction Score
       │         │
       │    (does this entry predict
       │     its own replication?)
       │
       ├──► Memetic Fitness Evaluation
       │         │
       │    (time-average replication
       │     rate, not ensemble average)
       │
       └──► K- AntiKnowledge Check
                 │
            (is this entry contradicted
             by an existing K- entry?)
                 │
                 ▼
          Survive / Decay / Kill
```

### Why it exceeds its parts

Confidence decay alone is a timer: entries fade. Memetic fitness alone is a popularity contest: entries that replicate survive. K- entries alone are a blacklist. None of these individually prevents a bad entry from causing damage during the window between creation and evaluation.

The combination closes this window. A new entry arrives with baseline confidence. Decay immediately begins. The entry must demonstrate positive self-prediction (its presence improves the Golem's forecasts) to maintain confidence. Memetic fitness evaluates whether the entry survives the time-average test (not just "does it work in the current market" but "does it work across the ergodic distribution of markets"). K- entries actively interfere: if a new entry's HDC encoding has high similarity to a K- entry, its initial confidence is penalized. Bad ideas face three independent kill mechanisms, and overcoming all three requires genuine informational value.

### Verification

- **Decay test**: Insert an entry with no reinforcement. Verify confidence reaches zero within the configured decay window.
- **K- suppression test**: Insert a K- entry for a known-bad pattern. Insert a new entry with high HDC similarity to the K- entry. Verify the new entry's initial confidence is penalized by at least 50%.
- **Fitness test**: Insert entries with known time-average fitness (some positive, some negative). Run memetic selection. Verify entries with negative time-average fitness are pruned within 2 dream cycles.
- **Immunity test**: Inject a series of plausible-but-wrong entries (e.g., "ETH always bounces at $X"). Verify the immune system kills them after the prediction fails to hold.

### Cross-references

- Grimoire memetic fields: [`04-memory/01b-grimoire-memetic.md`](../04-memory/01b-grimoire-memetic.md)
- Confidence decay / demurrage: [`02-mortality/05-knowledge-demurrage.md`](../02-mortality/05-knowledge-demurrage.md)
- AntiKnowledge (K-): [`04-memory/01-grimoire.md`](../04-memory/01-grimoire.md)
- Memetic fitness: [`04-memory/01b-grimoire-memetic.md`](../04-memory/01b-grimoire-memetic.md)

---

## 5. Predictive Haunting

### What emerges

Dead Golems' experiences influence living Golems' expectations. The hauntological spectral layer encodes spectral traces from deceased Golems. The Oracle prediction engine consumes these traces. The result: the past structurally haunts the present. A living Golem's predictions are shaped by experiences it never had, from Golems that no longer exist.

### Interacting subsystems

```
Dead Golem ──(Thanatopsis)──► Death Testament
                                    │
                              (legacy bundle,
                               K- entries,
                               spectral trace)
                                    │
                                    ▼
                            Hauntological Layer
                              (spectral traces)
                                    │
                              (weighted by
                               recency + relevance)
                                    │
                                    ▼
                            Oracle Predictions
                              (prior adjustment)
                                    │
                              (prediction now
                               incorporates dead
                               Golem's experience)
                                    │
                                    ▼
                            Living Golem acts
                            on haunted prediction
```

### Why it exceeds its parts

The hauntological layer alone stores spectral traces but doesn't use them for anything operational. The Oracle alone makes predictions from current data and the living Golem's experience. Neither system, independently, can transfer predictive knowledge from the dead to the living.

The combination creates intergenerational prediction transfer. A dead Golem that experienced a flash crash leaves a spectral trace encoding the crash dynamics. When market conditions rhyme with pre-crash conditions, the spectral trace activates, adjusting the living Golem's Oracle priors. The living Golem becomes cautious about a scenario it has never encountered, because a dead Golem encountered it and the trace survived.

This differs from Grimoire inheritance, which transfers explicit knowledge ("flash crashes look like X"). Predictive haunting transfers implicit influence on the prediction distribution. The living Golem doesn't know *why* it's cautious; the caution comes from a shift in its prior, not from a retrievable memory.

### Verification

- **Trace activation test**: Create a dead Golem with a strong spectral trace for a specific market pattern. Expose a living Golem to that pattern. Verify Oracle predictions shift toward the dead Golem's experience (measured as KL divergence between predictions with and without the spectral layer).
- **Decay test**: Verify spectral traces attenuate over time according to the configured decay function.
- **Relevance test**: Verify traces activate only when current conditions match the trace's context (similarity > threshold), not indiscriminately.
- **Performance test**: Compare prediction accuracy for living Golems with and without spectral layer access, specifically for scenarios the dead Golem experienced.

### Cross-references

- Hauntological spectral layer: [`06-hypnagogia/05-hauntology.md`](../06-hypnagogia/05-hauntology.md)
- Oracle / prediction engine: [`01-golem/17-prediction-engine.md`](../01-golem/17-prediction-engine.md)
- Thanatopsis death protocol: [`02-mortality/06-thanatopsis.md`](../02-mortality/06-thanatopsis.md)
- Death testament / legacy: [`01-golem/05-death.md`](../01-golem/05-death.md)

---

## 6. Cybernetic Personality

### What emerges

Two Golems with identical initial configurations develop distinct personalities over time. The 9 cybernetic feedback loops, behavioral phase dynamics, and Daimon affect engine interact to produce stable but path-dependent personality traits. Personality is not configured; it is grown from experience.

### Interacting subsystems

```
Experience ──► Daimon Affect Engine
                    │
               (PAD state update:
                Pleasure, Arousal,
                Dominance)
                    │
                    ▼
              Behavioral Phase
              (exploration ↔ exploitation,
               risk-seeking ↔ risk-averse)
                    │
                    ▼
              Feedback Loops (9x)
              ┌─────────────────────┐
              │ confidence → sizing │
              │ anxiety → hedging   │
              │ boredom → curiosity │
              │ grief → caution     │
              │ ... (5 more)        │
              └─────────────────────┘
                    │
                    ▼
              CorticalState ──(next experience is shaped
                               by personality state)──► loop back
```

### Why it exceeds its parts

The Daimon alone computes affect from portfolio events, but without feedback loops, affect is reactive and stateless. Feedback loops alone are mechanical couplings between signals. Behavioral phases alone are discrete modes.

The combination produces personality because the loops create attractors. A Golem that experiences early losses develops high baseline anxiety (Daimon affect), which increases hedging (feedback loop), which reduces subsequent losses (experience), which stabilizes the anxiety at a moderate-but-elevated level (attractor). A different Golem with early gains develops low baseline anxiety and lower hedging. Over weeks, the two Golems settle into distinct personality profiles: one conservative and cautious, the other aggressive and exploratory. These profiles are stable under perturbation (they return to their attractor after temporary disruption) but not permanent (sustained regime change can shift the attractor).

### Verification

- **Divergence test**: Initialize two Golems with identical configs. Run them through 48 hours of different market data. Measure CorticalState divergence. The Golems should develop measurably different personality profiles (PAD centroid distance > threshold).
- **Stability test**: Perturb a Golem's affect state (inject artificial anxiety spike). Verify it returns to its baseline attractor within a configured recovery window.
- **Attractor shift test**: Expose a Golem to sustained regime change (e.g., prolonged bear market). Verify personality profile shifts to a new attractor (higher baseline caution).
- **Feedback coupling test**: Disable one feedback loop (e.g., anxiety → hedging). Verify personality divergence decreases, confirming the loop contributes to personality emergence.

### Cross-references

- Daimon affect engine: [`03-daimon/01-appraisal.md`](../03-daimon/01-appraisal.md)
- Behavioral phases: [`03-daimon/03-behavior.md`](../03-daimon/03-behavior.md)
- CorticalState: [`01-golem/18-cortical-state.md`](../01-golem/18-cortical-state.md)
- Mortality-affect interaction: [`03-daimon/04-mortality-daimon.md`](../03-daimon/04-mortality-daimon.md)
- Feedback loops: [`01-golem/03-mind.md`](../01-golem/03-mind.md)

---

## 7. Stigmergic Strategy Discovery

### What emerges

Golems discover trading opportunities through indirect coordination. No Golem plans the swarm behavior. Instead, pheromone fields encode traces of past exploration, the curiosity drive pushes individual Golems toward unexplored regions, and ChainScope provides the observation surface. The Clade collectively explores the opportunity space more efficiently than any individual Golem could.

### Interacting subsystems

```
Golem A explores ──► deposits pheromone
opportunity X         on Styx field
                          │
                     (signal strength
                      proportional to
                      outcome quality)
                          │
                          ▼
Golem B reads ◄──── Styx pheromone field
pheromone                 │
                     (curiosity drive
                      attracted to high
                      pheromone gradient)
                          │
                          ▼
                    ChainScope observes
                    opportunity X region
                          │
                    (Golem B approaches
                     from different angle,
                     deposits own pheromone)
                          │
                          ▼
                    Emergent trail to
                    profitable niche
```

### Why it exceeds its parts

Pheromone fields alone are a broadcast medium. Curiosity drive alone pushes toward novelty without direction. ChainScope alone observes on-chain state without exploration strategy.

The combination produces stigmergic intelligence: coordination through environment modification. Golem A discovers a profitable Aave liquidation pattern and deposits strong pheromone. Golem B, drawn by the pheromone gradient and its own curiosity drive, investigates the same region but from a different entry point (it specializes in a different asset pair). Golem B finds a complementary opportunity and deposits its own pheromone. Over time, the pheromone field develops a topographic map of opportunity quality that no individual Golem computed. New Golems follow the gradients and either reinforce them (confirming the opportunity) or let them decay (the opportunity has been arbitraged away). The Clade self-organizes around market opportunities without central planning.

### Verification

- **Trail formation test**: Run a Clade of 5 Golems with a synthetic opportunity landscape. Verify pheromone trails converge on high-value regions within a configured discovery window.
- **Decay test**: Remove the underlying opportunity. Verify pheromone trails decay and Golems stop following them within the configured evaporation window.
- **Curiosity contribution test**: Disable curiosity drive. Verify Golems cluster near initial pheromone deposits instead of exploring adjacent regions. The discovered opportunity set should be smaller.
- **Efficiency test**: Compare opportunity discovery rate for a Clade with stigmergic coordination vs. the same number of independent Golems. The Clade should discover more unique opportunities per unit time.

### Cross-references

- Pheromone fields: [`20-styx/03-clade-sync.md`](../20-styx/03-clade-sync.md)
- Curiosity drive: [`01-golem/14-context-governor.md`](../01-golem/14-context-governor.md)
- ChainScope / triage: [`14-chain/02-triage.md`](../14-chain/02-triage.md)
- Clade coordination: [`09-economy/02-clade.md`](../09-economy/02-clade.md)
- Styx relay: [`20-styx/00-architecture.md`](../20-styx/00-architecture.md)

---

## 8. Mortality-Driven Creativity

### What emerges

As a Golem approaches death (Hayflick limit, epistemic collapse, economic failure), its creative output increases. The declining vitality scalar, combined with dream consolidation's recombination pressure and memetic evolution's fitness selection, causes the dying Golem to produce more novel strategy recombinations. Desperation breeds innovation.

### Interacting subsystems

```
Hayflick Counter ──(vitality declining)──► Attention Budget shrinks (K decreases)
                                                │
                                           (fewer slots, higher
                                            per-slot concentration)
                                                │
                                                ▼
                                          Dream Consolidation
                                           (recombination rate
                                            increases as vitality
                                            drops below threshold)
                                                │
                                           (novel combinations
                                            of existing Grimoire
                                            entries)
                                                │
                                                ▼
                                          Memetic Selection
                                           (time-average fitness
                                            filter on dream output)
                                                │
                                                ▼
                                          High-novelty, fitness-tested
                                          entries enter Grimoire
                                                │
                                          Thanatopsis ◄── (death testament
                                                          carries best entries
                                                          to Clade)
```

### Why it exceeds its parts

The Hayflick counter alone is a countdown. Dream consolidation alone recombines experiences at a fixed rate. Memetic selection alone filters by fitness without regard to the Golem's mortality state.

The combination produces a mortality-creativity coupling. When vitality drops below a threshold, dream consolidation enters a high-recombination mode: it tries more distant associations, binds entries from unrelated contexts, and generates strategies the Golem would never have tried during normal operation. Most of these recombinations are bad. Memetic fitness kills them. But the ones that survive carry genuine novelty. These enter the Grimoire and, when the Golem dies, propagate to the Clade via the death testament.

The evolutionary logic: dying organisms have nothing to lose by trying wild strategies. The few successes get inherited. Over generations, the Clade accumulates creative strategies that originated in desperate Golems. The mortality pressure is the engine; selection is the filter.

### Verification

- **Recombination rate test**: Measure Grimoire entry novelty (Hamming distance from nearest existing entry) during normal operation vs. during low-vitality operation. Novelty should increase by at least 2x when vitality < 30%.
- **Quality filter test**: Of the high-novelty entries produced during low vitality, verify that memetic selection kills at least 80% (they are novel but bad). The surviving entries should have positive time-average fitness.
- **Inheritance test**: Verify that high-novelty, fitness-tested entries appear in the death testament and propagate to successor Golems.
- **Cross-generational test**: Track Clade strategy diversity over multiple Golem lifetimes. Verify diversity increases monotonically (new creative strategies accumulate).

### Cross-references

- Hayflick counter: [`02-mortality/01-architecture.md`](../02-mortality/01-architecture.md)
- Dream consolidation: [`05-dreams/04-consolidation.md`](../05-dreams/04-consolidation.md)
- Memetic evolution: [`04-memory/01b-grimoire-memetic.md`](../04-memory/01b-grimoire-memetic.md)
- Thanatopsis: [`02-mortality/06-thanatopsis.md`](../02-mortality/06-thanatopsis.md)
- Death protocol: [`01-golem/05-death.md`](../01-golem/05-death.md)

---

## 9. Cross-Generational Learning

### What emerges

Each generation of Golems starts slightly smarter than the last. Knowledge accumulates across lifetimes without central storage. The death protocol transfers knowledge, the Grimoire inherits it, and confidence decay prevents stale knowledge from dominating. The result is a ratchet: forward progress is preserved, but outdated knowledge decays naturally.

### Interacting subsystems

```
Gen N Golem ──(lives, learns)──► Grimoire fills with tested entries
                                        │
                                   (Golem dies)
                                        │
                                        ▼
                                  Thanatopsis ──► Legacy bundle (top entries
                                                  by memetic fitness, K- entries
                                                  with highest info value)
                                        │
                                        ▼
                                  Gen N+1 Golem (inherits bundle,
                                   reduced initial confidence)
                                        │
                                  Confidence Decay (stale entries
                                   fade unless validated by experience)
                                        │
                                        ▼
                                  Gen N+1 retains what works,
                                  forgets what doesn't
```

### Why it exceeds its parts

The death protocol alone transfers data. Grimoire inheritance alone copies entries. Confidence decay alone is a timer.

The combination produces an evolutionary ratchet with built-in error correction. Knowledge transfers across generations, but inherited entries aren't treated as ground truth. They start with reduced confidence. The successor Golem must validate them through its own experience. Entries that remain valid get reinforced and persist. Entries that reflected conditions specific to the predecessor's lifetime decay and die. K- entries (antiknowledge) propagate with high priority because knowing what *doesn't* work is more durable than knowing what does.

Over multiple generations, the Clade's knowledge base converges on durable patterns and sheds ephemeral ones. No central knowledge store exists. The intelligence lives in the succession chain, distributed across overlapping Golem lifetimes.

### Verification

- **Ratchet test**: Run 5 generations of a Golem lineage on the same market data. Measure first-day performance for each generation. Later generations should reach profitability faster (they inherit validated knowledge).
- **Decay test**: Include some entries in the legacy bundle that are valid for Generation N's market conditions but invalid for Generation N+1's. Verify these entries decay and are pruned in Generation N+1.
- **K- propagation test**: Verify K- entries persist across at least 3 generations (they are highly durable because they encode failure modes).
- **No central store test**: Kill the Styx relay. Verify knowledge still propagates through direct succession (death → inheritance), confirming no central storage dependency.

### Cross-references

- Death protocol: [`01-golem/05-death.md`](../01-golem/05-death.md)
- Thanatopsis: [`02-mortality/06-thanatopsis.md`](../02-mortality/06-thanatopsis.md)
- Succession: [`02-mortality/07-succession.md`](../02-mortality/07-succession.md)
- Grimoire inheritance: [`01-golem/09-inheritance.md`](../01-golem/09-inheritance.md)
- Knowledge demurrage: [`02-mortality/05-knowledge-demurrage.md`](../02-mortality/05-knowledge-demurrage.md)
- Grimoire memetic fields: [`04-memory/01b-grimoire-memetic.md`](../04-memory/01b-grimoire-memetic.md)

---

## 10. Attention Market

### What emerges

Attention is allocated economically. The VCG auction mechanism forces subsystems to bid truthfully for cognitive resources. ChainScope provides the observation surface (what *could* be attended to). The Oracle provides predictive valuations (what *should* be attended to). The combination produces efficient information processing: the Golem attends to the right things at the right time, not because of hardcoded priority rules, but because an internal market prices attention correctly.

### Interacting subsystems

```
ChainScope ──(observable signals)──► Signal Pool
                                         │
Oracle ──(predictive value per signal)──►│
                                         │
Risk Module ──(risk-urgency bids)───────►│
                                         │
Curiosity Module ──(novelty bids)───────►│
                                         │
Temporal Monitors ──(urgency bids)──────►│
                                         │
                                    ┌────▼────┐
                                    │   VCG   │
                                    │ Auction  │
                                    └────┬────┘
                                         │
                                    (K winners:
                                     signals that
                                     earned attention
                                     this tick)
                                         │
                                         ▼
                                    CorticalState
                                    attention_focus
```

### Why it exceeds its parts

The VCG auction alone is a mechanism design tool. ChainScope alone is an observer. The Oracle alone is a predictor. Static priority rules could do the job adequately in stable conditions.

The combination produces something static rules cannot: adaptive, truthful, efficient resource allocation under uncertainty. The VCG mechanism's dominant-strategy truthfulness means each subsystem bids its true valuation for attention. The Oracle's predictive valuation means attention goes where information gain is highest. ChainScope's broad observation surface means the Golem doesn't miss opportunities because of hardcoded filters.

The market adapts in real time. When risk detects danger, its bids increase. When curiosity detects unexplored regions, its bids compete for attention slots. The VCG mechanism resolves these competing demands optimally without any subsystem needing to know about the others' priorities. Decentralized coordination through prices.

### Verification

- **Truthfulness test**: Verify subsystems achieve weakly higher utility by bidding truthfully than by misreporting. (VCG guarantees this theoretically; the test confirms the implementation preserves the guarantee.)
- **Efficiency test**: Compare the information gain per tick under VCG allocation vs. round-robin allocation vs. static-priority allocation. VCG should achieve the highest information gain.
- **Adaptation test**: Inject a risk event (sudden volatility spike). Verify attention allocation shifts toward risk-relevant signals within 1-2 ticks.
- **Budget constraint test**: Reduce K (total attention slots) by 50%. Verify the VCG auction still allocates the reduced budget to the highest-value signals (graceful degradation, not random dropping).
- **Revenue test**: Verify VCG payments are computed correctly (second-price property: each winner pays the externality it imposes on other bidders).

### Cross-references

- VCG attention auction: [`01-golem/14-context-governor.md`](../01-golem/14-context-governor.md)
- ChainScope / triage: [`14-chain/02-triage.md`](../14-chain/02-triage.md)
- Oracle / prediction engine: [`01-golem/17-prediction-engine.md`](../01-golem/17-prediction-engine.md)
- CorticalState: [`01-golem/18-cortical-state.md`](../01-golem/18-cortical-state.md)
- Temporal monitors: [`10-safety/07-temporal-logic-verification.md`](../10-safety/07-temporal-logic-verification.md)

---

## 11. Mortality-Aware Position Sizing

### What emerges

Position sizes derived from the Kelly criterion where estimation uncertainty is bounded by MI channel capacity and the Kelly fraction is further scaled by sheaf consistency to account for model disagreement across timescales. The formula: `f* = kelly_fraction * min(1, I(G;M) / I_target) * sheaf_consistency`.

### Interacting subsystems

Ergodicity economics (Kelly criterion) provides the base fraction from estimated edge and variance. Information-theoretic mortality (MI ratio) captures how much information the Golem actually extracts from the market -- declining MI means the model is degrading, so positions shrink. Sheaf consistency captures internal model agreement -- disagreement between timescales means the estimate itself is unreliable.

### Why it exceeds its parts

A static risk system uses configured position limits. The dynamic system cuts positions when the Golem's cognitive state doesn't justify the full position. A Golem with Kelly fraction 0.12, MI ratio 0.8, and sheaf consistency 0.6 sizes at 0.058 (5.8%) instead of 12%. Each factor is measured, not configured.

### Verification

- **Sizing test**: Verify position sizes decrease when MI drops (model degradation) and when sheaf consistency drops (timescale disagreement).
- **Kelly calibration test**: Verify the Kelly fraction tracks estimated edge correctly under varying volatility regimes.
- **Cross-factor test**: Verify that all three factors independently contribute to sizing (disable each in turn and measure the difference).

### Cross-references

- Ergodicity economics: [`09-economy/00-identity.md`](../09-economy/00-identity.md)
- MI mortality: [`02-mortality/17-information-theoretic-diagnostics.md`](../02-mortality/17-information-theoretic-diagnostics.md)
- Sheaf consistency: [`01-golem/03-mind.md`](../01-golem/03-mind.md)

---

## 12. Compositional Safety With Temporal Guarantees

### What emerges

Strategies composed in the categorical framework are verified both compositionally (each morphism preserves safety invariants, composition rules propagate them) and temporally (the sequence of composed operations satisfies LTL/CTL properties like bounded debt duration and mandatory rebalancing). The Witness DAG records the execution trace for post-hoc verification against the same formulas.

### Interacting subsystems

Category-theoretic composition (MorphismTag on tools) guarantees well-typed compositions of safe morphisms are safe. LTL temporal monitors (compiled to DFAs, O(1) per tick) verify sequences over time. The Witness DAG creates a tamper-proof record linking observations to predictions to decisions to outcomes.

### Why it exceeds its parts

PolicyCage checks individual operations. The categorical framework checks compositions. Temporal logic checks sequences. The Witness DAG proves all three held. Per-operation safety (PolicyCage), compositional safety (morphism tags), temporal safety (LTL monitors), and auditable proof (DAG) form four independent verification layers.

### Verification

- **Composition safety test**: Construct a strategy tree with known safe and unsafe compositions. Verify the categorical verifier accepts safe compositions and rejects unsafe ones.
- **Temporal violation test**: Create a scenario that satisfies per-operation safety but violates a temporal property (e.g., debt held >200 ticks). Verify the LTL monitor detects it.
- **DAG integrity test**: Verify every Decision vertex in the Witness DAG links back to its evidential Observation and Prediction vertices.

### Cross-references

- Category-theoretic composition: [`07-tools/00-overview.md`](../07-tools/00-overview.md)
- Temporal logic verification: [`10-safety/07-temporal-logic-verification.md`](../10-safety/07-temporal-logic-verification.md)
- Witness DAG: [`10-safety/08-witness-dag.md`](../10-safety/08-witness-dag.md)
- PolicyCage: [`10-safety/00-overview.md`](../10-safety/00-overview.md)

---

## 13. Multiscale Coherent Observation

### What emerges

The sheaf framework detects when observations at different timescales contradict each other. The attention auction responds by redirecting cognitive resources toward the contradicting scales. The HDC gestalt vector encodes the contradiction pattern for future matching against historical contradictions and their resolutions.

### Interacting subsystems

Sheaf-theoretic observation provides the consistency score and identifies which edge (gamma-theta or theta-delta) carries the largest disagreement. The VCG auction reads the score and boosts bids for contradicting dimensions. The HDC system encodes the contradiction pattern: `bind(scale1_hv, scale2_hv, permute(contradiction_direction_hv, edge_index))`.

### Why it exceeds its parts

The gamma/theta/delta architecture runs each timescale independently. They share CorticalState but don't reason about mutual consistency. The sheaf framework adds a formal consistency measure. When consistency drops, the system redirects attention and encodes the pattern for future reference. Historical matching can identify how similar contradictions resolved in the past.

### Verification

- **Contradiction detection test**: Inject conflicting signals at gamma vs. theta scale. Verify sheaf consistency drops below 0.5 and the contradicting edge is correctly identified.
- **Attention redirect test**: After a low-consistency event, verify the VCG auction allocates at least 50% of budget to the contradicting scales.
- **Pattern matching test**: Store a contradiction pattern. When a similar contradiction occurs later, verify HDC similarity > 0.80.

### Cross-references

- Sheaf consistency: [`01-golem/03-mind.md`](../01-golem/03-mind.md)
- VCG attention auction: [`01-golem/14-context-governor.md`](../01-golem/14-context-governor.md)
- HDC encoding: [`shared/hdc-vsa.md`](./hdc-vsa.md)

---

## 14. Temporal Pattern Recognition at Constant Cost

### What emerges

The 40 temporal properties from the temporal logic verification system are compiled into deterministic finite automata that run as constant-cost monitors. They detect temporal violations in real time, in O(1) per tick per monitor, by maintaining automaton state. A Golem that takes safe individual actions can still violate temporal properties (debt held too long, rebalancing delayed, risk check skipped for N consecutive heartbeats).

### Interacting subsystems

Temporal logic verification provides the compiled DFA monitors. CorticalState signals provide the predicates that feed each DFA transition. The attention auction picks up temporal urgency signals and redirects cognitive resources.

### Why it exceeds its parts

PolicyCage alone verifies individual operations. DFA monitors alone track automaton state. The combination catches violations that per-operation checking cannot: "any concentrated liquidity position with utilization above 90% must be rebalanced within 100 theta ticks" requires tracking duration, which no per-operation check can do. Total cost for 40 monitors: ~200 state-variable reads per tick.

### Verification

- **Detection latency test**: Inject a temporal violation (position held past threshold duration). Verify the monitor detects it within 1 tick.
- **Constant cost test**: Verify per-tick monitor cost does not increase as the number of active positions or historical ticks grows.
- **False positive test**: Run 10,000 ticks of normal operation. Verify zero false temporal violation alerts.

### Cross-references

- Temporal logic verification: [`10-safety/07-temporal-logic-verification.md`](../10-safety/07-temporal-logic-verification.md)
- CorticalState: [`01-golem/18-cortical-state.md`](../01-golem/18-cortical-state.md)

---

## 15. Cross-Dream Provenance Correlation

### What emerges

During dream cycles, the Witness DAG provides the full provenance chain for each replayed experience. The memetic immune system evaluates whether dream-generated knowledge has positive or negative informational value (measured by its contribution to I(G;M)). The Phi metric measures whether dream-cycle integration actually improves the coherence of the resulting knowledge. Dreams that produce fragmented knowledge (low Phi) are flagged for re-processing.

### Interacting subsystems

The dream engine replays experiences and produces Grimoire entries. The Witness DAG links each entry to the specific observations that produced it. The memetic fitness evaluator scores each entry by delta_MI using a holdout set. The Phi computation treats dream-produced entries as a system and measures their mutual information integration.

### Why it exceeds its parts

The dream engine alone replays and consolidates without tracking provenance. The Witness DAG alone records provenance without evaluating informational value. The Phi metric alone measures integration without linking it to specific dream outputs. The combination creates auditable, fitness-evaluated, integration-tested dream knowledge.

### Verification

- **Provenance test**: After a dream cycle, verify every dream-produced Grimoire entry has a Witness DAG vertex linking back to the replayed observation vertices.
- **Fitness test**: Insert a dream-produced entry with known positive delta_MI. Verify the memetic evaluator scores it above the retention threshold.
- **Integration test**: Run a dream cycle that produces fragmented entries (independent insights about unrelated events). Verify Phi < 0.2 and the entries are flagged as low-integration.
- **Re-process test**: After a low-Phi dream, verify the scheduler queues a follow-up dream cycle for the same experiences.

### Cross-references

- Dream engine: [`05-dreams/01-architecture.md`](../05-dreams/01-architecture.md)
- Witness DAG: [`10-safety/08-witness-dag.md`](../10-safety/08-witness-dag.md)
- Memetic fitness: [`04-memory/01b-grimoire-memetic.md`](../04-memory/01b-grimoire-memetic.md)
- Phi integration: [`01-golem/03-mind.md`](../01-golem/03-mind.md)

---

## Cross-reference matrix

Which subsystems contribute to which emergent capabilities. An **x** marks a required component. A **.** marks a referenced-but-not-structurally-required component.

| # | Capability | CorticalState | HDC | TDA | Daimon | Grimoire | Mortality | Dreams | Styx | Oracle | Attention | Safety | ChainScope |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | Gestalt Vector | x | x | . | . | . | x | . | . | . | x | . | . |
| 2 | Topological-Somatic | x | . | x | x | . | . | . | . | . | x | . | . |
| 3 | Associative Memory | . | x | . | . | x | . | . | . | . | . | x | . |
| 4 | Memetic Immune System | . | x | . | . | x | x | . | . | . | . | . | . |
| 5 | Predictive Haunting | . | . | . | . | x | x | . | . | x | . | . | . |
| 6 | Cybernetic Personality | x | . | . | x | . | x | . | . | . | . | . | . |
| 7 | Stigmergic Discovery | . | . | . | . | . | . | . | x | . | . | . | x |
| 8 | Mortality Creativity | . | . | . | . | x | x | x | . | . | . | . | . |
| 9 | Cross-Gen Learning | . | . | . | . | x | x | . | . | . | . | . | . |
| 10 | Attention Market | x | . | . | . | . | . | . | . | x | x | x | x |
| 11 | Mortality-Aware Sizing | . | . | . | . | . | x | . | . | . | . | . | . |
| 12 | Compositional + Temporal Safety | . | . | . | . | . | . | . | . | . | . | x | . |
| 13 | Multiscale Coherent Observation | . | x | . | . | . | . | . | . | . | x | . | . |
| 14 | Temporal Pattern O(1) | x | . | . | . | . | . | . | . | . | . | x | . |
| 15 | Cross-Dream Provenance | . | . | . | . | x | . | x | . | . | . | x | . |

Every capability requires at least two subsystems. No capability is reducible to a single component. The capabilities compose further: the gestalt vector (1) feeds into topological-somatic intelligence (2) through CorticalState; the memetic immune system (4) filters entries that cross-generational learning (9) would otherwise propagate unchecked; the attention market (10) allocates resources that every other capability competes for. Mortality-aware sizing (11) connects the information-theoretic mortality diagnostic to the economic layer. Compositional + temporal safety (12) ensures that per-operation, per-composition, and per-sequence safety properties all hold simultaneously. Cross-dream provenance (15) closes the loop between offline learning and auditable evidence.
