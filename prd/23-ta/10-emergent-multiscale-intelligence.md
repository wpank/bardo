# Emergent Multiscale Intelligence [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Source**: `tmp/research/witness-research/new/ta/10-emergent-multiscale-intelligence.md`
>
> **Depends on**: Docs 1-9 (all preceding TA subsystems), Innovation 11 (IIT Phi framework)

**Audience:** Systems engineers building the Bardo runtime's TA integration layer; researchers studying information-theoretic measures of cognitive integration in autonomous agents. Assumes familiarity with all nine preceding documents in this series and with Bardo's IIT Phi framework (Innovation 11).

---

> **Reader orientation:** This document applies Integrated Information Theory (IIT) to measure and strengthen the integration across all nine TA subsystems in the Golem (mortal autonomous DeFi agent) runtime. It belongs to the **TA research layer** (Doc 10 of 10) and covers Phi computation over the 9-subsystem network, Partial Information Decomposition (PID) for synergistic insight detection, cross-primitive intelligence that no single subsystem can produce, dream-based integration strengthening, and Clade (group of related Golems sharing knowledge via Styx, the global relay)-level collective Phi. You should understand information theory basics and the preceding 9 TA documents. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Abstract

Nine technical analysis subsystems, each documented in its own paper, each operating on different mathematical foundations: hyperdimensional pattern algebra, Riemannian geometry, evolutionary signal ecology, causal discovery, persistent homology, adversarial defense, DeFi-native indicators, and somatic markers. Run them side by side and you get nine independent assessments. That is not intelligence. Intelligence is what happens when the subsystems talk to each other, when the HDC entanglement detector and the causal graph and the somatic fear marker converge on a single cross-protocol insight that none of them could produce alone.

This document applies Integrated Information Theory (IIT) to the TA cortex. We define Phi over the 9-subsystem network, enumerate all 255 bipartitions at each theta tick, and identify the Minimum Information Bipartition (MIB) as a diagnostic for weak integration links. Beyond Phi, we decompose inter-subsystem information into unique, redundant, and synergistic components using the Partial Information Decomposition (PID) framework. Synergistic information is the target: it exists only when subsystems combine. We provide complete Rust implementations for Phi computation, pairwise PID estimation, cross-primitive insight detection, integration health tracking, dream-based integration strengthening, Clade-level collective Phi, and the cybernetic feedback loop through which mortality selects for higher integration across Golem generations. The MIB is a repair signal.

---

## The problem [SPEC]

A Golem running all nine TA subsystems has a lot of information. Doc 1's HDC encoder can detect that Uniswap and Aave hypervectors are drifting toward correlation. Doc 4's causal discovery engine can find a directed edge from large swap events to lending rate spikes. Doc 7's DeFi-native indicators can report that Aave utilization oscillation frequency has shifted from its 4-hour norm. Doc 5's persistent homology can show that Betti number beta_0 is climbing in the joint observation space, meaning connected components are fragmenting. Doc 9's somatic marker for "cross-protocol contagion" can fire, tagging the current state with an affective signal rooted in prior bad outcomes.

Five subsystems. Five separate signals. Each one is a partial view.

The HDC encoder knows the protocols are correlating but not why. The causal engine knows the direction of influence but not its affective weight. The DeFi indicators see the oscillation shift but cannot connect it to the topological fragmentation that Doc 5 detects. The somatic marker fires but cannot articulate which structural features triggered it. The persistent homology sees the shape changing but has no model of what's causing the change.

A Golem that reports all five signals independently has not understood anything. It has collected fragments. The question that matters: can the Golem synthesize "a structural coupling between Uniswap and Aave is forming, driven by arbitrage feedback, entering a regime I have seen before and learned to fear"?

That sentence requires information from at least four subsystems. No single subsystem can generate it. The insight exists only in the combination. This is what information theorists call synergistic information, and measuring how much of it your system produces is what IIT's Phi quantifies.

The failure mode is real. A system of nine independent expert modules is a committee, not a mind. Committees produce reports. Minds produce understanding. The difference is integration: the degree to which information in one subsystem constrains and is constrained by information in every other subsystem, such that the whole cannot be decomposed into independent parts without losing something.

### Why nine subsystems make this hard

Each subsystem operates on different mathematical objects. HDC works with 10,240-bit binary vectors. The spectral manifold works with Riemannian curvature tensors. Causal discovery works with directed acyclic graphs. Persistent homology works with persistence diagrams. Somatic markers work with scalar affect values. There is no shared representation language.

The heartbeat helps. All nine subsystems write to CorticalState during their gamma ticks. CorticalState is the shared perception surface, a lock-free struct of atomic values that every subsystem can read. But writing to a shared surface is not the same as integrating. Two people can write on the same whiteboard without ever reading each other's notes.

Integration requires that each subsystem's output changes based on what the other subsystems wrote. The HDC encoder should weight its entanglement signal differently when the causal graph confirms a directed edge versus when it doesn't. The somatic marker should modulate its intensity based on whether the topological signature matches a known dangerous pattern or a novel one. The DeFi indicators should flag an oscillation shift as significant when the HDC encoder reports rising cross-protocol correlation, and dismiss it as noise when the protocols remain independent.

This bidirectional influence is what Phi measures. When it is high, the subsystems are talking. When it is low, they are shouting past each other.

### The integration tax

There is a cost. Computing Phi over 9 subsystems requires evaluating 255 bipartitions. Each bipartition requires a mutual information estimate over the signal vectors of two subsystem groups. At a theta tick cadence of 30-120 seconds, this is tractable, but it is not free.

The cost buys something specific: a diagnostic. The MIB tells you which pair of subsystem groups shares the least information. If the MIB consistently splits {Causal} from {everything else}, then Doc 4's outputs are not reaching the other subsystems. That is an actionable repair signal. Fix the integration between causal discovery and the rest, and Phi increases, and (the hypothesis) decision quality improves.

Without Phi, you would not know which integration link is weakest. You would tune parameters blindly, or worse, assume that running nine subsystems is sufficient because you wrote the code that runs them.

### What "integration" looks like in practice

To make this concrete, walk through what happens when the nine subsystems are integrated versus when they are not.

**Without integration (committee mode).** A DeFi contagion event begins. Uniswap ETH/USDC volume spikes 4x. Aave ETH utilization jumps from 65% to 88%. The nine subsystems each produce their output:

- HDC: "entanglement between Uniswap and Aave bundles rising. Similarity: 0.72 (was 0.53)."
- Spectral: "curvature in the lending subspace increasing. Geodesic distance to critical state: 0.4."
- Metabolism: "swap-related signals gaining fitness. Lending signals losing fitness."
- Causal: "new edge detected: Uniswap_volume -> Aave_utilization (lag: 2 blocks, p < 0.01)."
- Predictive: "beta_0 increasing in joint space. Connected components fragmenting."
- Pattern ecosystem: "historical pattern match: pre-liquidation cascade (fitness: 0.82)."
- DeFi-native: "Aave utilization at 88%, rate curve entering steep segment. Funding rate positive."
- Adversarial: "no manipulation detected. Data appears organic."
- Somatic: "negative valence: -0.6. Arousal: 0.8. Prior association: Black Thursday 2020."

Nine signals. Nine assessments. The Oracle receives them all and must somehow combine them. Without integration, it can only average or vote. The average of these signals says "something bad might be happening," which is vague enough to be useless. A vote on "should we reduce exposure?" might split 6-3, but the three dissenters (metabolism, adversarial, and pattern ecosystem) are not wrong; they are answering different questions.

**With integration (Phi-driven synthesis).** The same event. But now the integration engine has connection weights learned from prior cross-primitive events. The causal edge from Uniswap volume to Aave utilization confirms that the HDC entanglement is not spurious. The adversarial all-clear means the data is trustworthy. The somatic marker's Black Thursday association activates the appropriate fear response. The predictive geometry's topological fragmentation, combined with the manifold's curvature spike, indicates that the safe operating region is shrinking.

The integrated assessment: "Cross-protocol contagion event between Uniswap and Aave. Causal direction confirmed (swap volume drives utilization). Utilization entering the steep rate curve segment with topological support. Pattern matches pre-liquidation cascade with 82% fitness. Somatic marker recalls Black Thursday. Adversarial score is clean, so data is trustworthy. Recommended action: reduce lending exposure, increase monitoring frequency, alert Clade."

This assessment contains information that no single subsystem produced. The synthesis of causal direction + somatic recall + topological constraint + adversarial clearance creates a coherent narrative. The Phi score at this tick would be high, because the subsystems' outputs are mutually consistent and interdependent.

---

## Mathematical foundations [SPEC]

### IIT Phi applied to the TA cortex

Define the TA cortex as a system **T** with 9 subsystems {T_1, ..., T_9}, corresponding to the nine documents in this series:

| Index | Subsystem | Signal type |
|-------|-----------|-------------|
| T_1 | HDC pattern algebra | 10,240-bit similarity scores, entanglement drift |
| T_2 | Spectral manifold | Curvature scalars, geodesic distances, parallel transport errors |
| T_3 | Signal metabolism | Fitness scores, budget shares, speciation events |
| T_4 | Causal microstructure | Edge weights, intervention effect sizes, Granger p-values |
| T_5 | Predictive geometry | Persistence landscape amplitudes, trajectory forecasts |
| T_6 | Pattern ecosystem | Pattern fitness, population diversity, predator-prey ratios |
| T_7 | DeFi-native indicators | Utilization rates, funding rates, Greeks surfaces, yield spreads |
| T_8 | Adversarial defense | Manipulation scores, robust statistic residuals, red-team alerts |
| T_9 | Somatic markers | Affect valence, arousal, confidence modulation factors |

At each theta tick, each subsystem writes a signal vector **x_i** to the TA signal surface (an extension of CorticalState). Collect these into the joint state:

```
X = (x_1, x_2, ..., x_9)
```

For a bipartition P = (A, B) of the 9 subsystems into two non-empty groups, define the integrated information across that partition:

```
phi(P) = I(X_A ; X_B)
```

where X_A is the concatenation of signal vectors from subsystems in group A, X_B from group B, and I is mutual information estimated from the signal history buffer.

The system-level Phi is the minimum over all bipartitions:

```
Phi = min_P phi(P)
```

This is the core IIT definition (Tononi, 2004). Phi = 0 means there exists a bipartition where the two halves share no information, so the system decomposes into independent modules. Phi > 0 means every possible split still leaves some mutual information, so the system operates as an integrated whole.

With 9 subsystems, the number of non-trivial bipartitions is 2^8 - 1 = 255. This is small enough for exhaustive enumeration. No approximation needed.

### The Minimum Information Bipartition

The MIB is the bipartition P* that achieves the minimum:

```
P* = argmin_P phi(P)
```

The MIB is the weakest link. It identifies which split of the subsystems costs the least information, meaning the two sides are least dependent on each other.

Different MIBs diagnose different integration failures:

**MIB = {HDC, Somatic} vs {rest}**: Pattern recognition and emotional response operate in a closed loop disconnected from the analytical core. The Golem has "gut feelings" that are not informed by causal reasoning, topological analysis, or DeFi-specific indicators. Gut feelings ungrounded by analysis are dangerous.

**MIB = {Causal} vs {rest}**: Causal discovery runs in isolation. It finds directed edges between protocols, but this information does not reach the HDC encoder (which could weight its entanglement tracking accordingly), the somatic system (which could form causal markers instead of correlational ones), or the predictive geometry (which could constrain its trajectory forecasts to causally plausible paths). The most expensive subsystem is producing insights nobody reads.

**MIB = {Manifold, Predictive} vs {rest}**: The geometric subsystems (Riemannian manifold and persistent homology) form their own cluster. They integrate with each other (curvature informs topology and vice versa) but not with the pattern-based or indicator-based subsystems. The Golem has two separate worldviews: a geometric one and a statistical one.

**MIB = {Adversarial} vs {rest}**: The adversarial defense subsystem is disconnected. It detects manipulation attempts, but this detection does not modulate the confidence of other subsystems. The Golem can simultaneously believe "this pattern matches a bullish reversal" (HDC) and "this data is being manipulated" (Adversarial) without feeling any contradiction.

Each MIB pattern suggests a specific repair: strengthen the information flow between the weakly connected groups. The repair mechanism operates through NREM dreams (Section 8).

### Information decomposition: synergy and redundancy

Phi measures total integration but does not distinguish between different types of shared information. Two subsystems might share information because they are redundant (both measure the same thing independently) or because they are synergistic (their combination reveals something neither contains alone). Redundancy is waste. Synergy is intelligence.

The Partial Information Decomposition (PID) framework (Williams & Beer, 2010) decomposes the information that a set of sources provides about a target into four non-negative components.

For two subsystems A and B predicting an outcome Y:

```
I({A,B} ; Y) = Unique(A) + Unique(B) + Redundancy(A,B) + Synergy(A,B)
```

**Unique(A)**: Information about Y that only A provides. Example: only the causal discovery subsystem knows the direction of influence between two protocols. If you remove it, that information vanishes.

**Unique(B)**: Information about Y that only B provides. Example: only the somatic marker subsystem knows the affective valence of the current state based on past experience.

**Redundancy(A,B)**: Information about Y that both A and B provide independently. Example: both HDC entanglement tracking and the spectral manifold's curvature detect that two protocols are correlating. Either one alone would tell you the same thing. Having both is insurance, not insight.

**Synergy(A,B)**: Information about Y that neither A nor B provides alone, but their combination does. Example: HDC says protocols are entangled. Causal discovery says the direction is from swaps to lending rates. Neither fact alone constitutes the insight "arbitrage is coupling these protocols." Together they do.

Synergy is what we want to maximize. A system with high Phi could be high-Phi because of redundancy (all subsystems measure the same thing) or because of synergy (subsystems combine to produce novel information). The PID decomposition distinguishes these cases.

For the full 9-subsystem case, pairwise PID gives us a 9x9 matrix of synergy values. The total synergy:

```
S_total = sum_{i<j} Synergy(T_i, T_j)
```

We can also compute higher-order synergy (information that requires three or more subsystems to combine), but pairwise synergy is the computationally tractable approximation. The full 9-subsystem case has 36 pairs (9 choose 2). At each theta tick, we compute PID for the top-k pairs ranked by pairwise MI, where k defaults to 10. This keeps the computation bounded while covering the most informative pairs.

### Worked example: the synergy matrix

Consider a concrete scenario. The Golem is monitoring Uniswap v3 ETH/USDC and Aave v3 ETH lending. At theta tick t, the pairwise synergy matrix (for the 5 most active subsystems) looks like:

```
            HDC    Causal  DeFi   Somatic  Manifold
HDC          -      0.12   0.04    0.08     0.02
Causal     0.12      -     0.15    0.03     0.06
DeFi       0.04    0.15     -      0.11     0.09
Somatic    0.08    0.03    0.11     -       0.01
Manifold   0.02    0.06    0.09    0.01      -
```

The highest synergy is between Causal and DeFi (0.15). This means the combination of causal edge direction and DeFi-native utilization metrics produces more information about outcomes than either alone. The Golem's causal discovery found that large Uniswap swaps drive Aave utilization changes. The DeFi indicators show the utilization is at 87%. Neither fact alone predicts what happens next. Together, they predict that the next large swap will push utilization above 90%, triggering the rate curve's steep segment and creating a borrowing cost spike.

The lowest synergy is between Somatic and Manifold (0.01). These subsystems barely interact. The somatic marker system and the Riemannian geometry are producing independent information. This is a candidate for integration improvement: perhaps the manifold's curvature could inform the somatic system about how "dangerous" the current geometric state is, creating a new affect-geometry integration pathway.

### Higher-order synergy

Pairwise synergy captures two-way interactions. But some insights require three or more subsystems. The "cross-protocol contagion" insight from the problem statement requires HDC (entanglement), Causal (direction), DeFi (indicators), and possibly Somatic (experiential warning). The pairwise PID cannot capture the full four-way synergy.

Higher-order PID (Mediano et al., 2019) extends the decomposition to sets of three or more sources. For three sources A, B, C and target Y, the decomposition includes:

```
I({A,B,C}; Y) = ... + Synergy(A,B,C) + pairwise terms + unique terms + redundancy terms
```

The triple synergy Synergy(A,B,C) is information that requires all three sources simultaneously. It vanishes if any one source is removed.

Computing this for all triples of 9 subsystems gives 84 triples (9 choose 3). This is more expensive than pairwise but still tractable at theta tick frequency. We compute triple PID only for triples where all three pairwise synergies exceed a minimum threshold, filtering the 84 candidates down to typically 5-15 active triples.

### Cross-primitive insight detection

A cross-primitive insight is an inference that requires information from multiple DeFi primitive types and multiple TA subsystems.

Formal criteria. An insight I is cross-primitive if:

1. It involves at least 2 different DeFi primitive types (e.g., swaps and lending)
2. No single TA subsystem is sufficient to generate it
3. The synergistic information among the contributing subsystems exceeds a threshold tau_s

Detection requires checking whether the current subsystem outputs, taken together, match a known insight template or exceed the synergy threshold for a novel combination:

```
is_cross_primitive(outputs) =
    |{primitive_types(outputs)}| >= 2
    AND NOT exists subsystem S: sufficient(S, insight)
    AND synergy(contributing_subsystems) > tau_s
```

The sufficiency test asks: could subsystem S, given only its own inputs, have produced this conclusion? If yes for any S, the insight is not truly cross-primitive. It was already contained in a single subsystem.

The synergy threshold tau_s is adaptive. It starts high (requiring strong synergistic signal) and decreases as the Golem accumulates experience with cross-primitive events. Early in life, the Golem is conservative about declaring cross-primitive insights. After surviving several cross-protocol contagion events and building somatic markers for them, it can detect subtler instances.

### Evolutionary selection for integration

Mortality applies selection pressure. Golems with higher Phi make better cross-subsystem decisions, survive longer, and pass their parameters to the next generation. Golems with low Phi operate as disconnected committee systems, miss cross-primitive events, and die sooner.

Over generations:

```
E[Phi_{gen+1}] > E[Phi_{gen}]
```

if Phi correlates positively with survival duration. This is testable by tracking the correlation between Phi (measured at each theta tick, averaged over a Golem's lifetime) and lifespan (measured in heartbeat cycles before mortality triggers death).

The selection mechanism operates through the death testament (Thanatopsis). When a Golem dies, its IntegrationTestament records its lifetime Phi trajectory, MIB history, synergy scores, and the specific cross-subsystem connection weights that produced the best decisions. The next generation initializes from the testament's best parameters rather than from scratch.

At the Clade level, Styx coordination means multiple Golems share signal outputs through pheromone channels. Clade-level Phi measures how well the Golems in a Clade integrate as a collective. A Clade where each Golem specializes in a different DeFi vertical but shares cross-primitive insights through Styx has higher Clade Phi than one where Golems operate in isolation.

Morphogenetic specialization (Innovation 08) creates a tension: Turing patterns push Golems toward different ecological niches, which increases within-Golem integration (each Golem focuses on fewer primitives and integrates them more deeply) but could decrease between-Golem integration if the specialists stop communicating. Clade Phi tracks whether the Clade maintains collective integration even as individual Golems specialize.

### Intelligence growth trajectory

Two compounding processes drive Phi upward.

Within a lifetime: experience strengthens cross-subsystem connections. Each time the Golem encounters a cross-primitive event and the integration produces a correct assessment, NREM dreams strengthen the connection weights between the contributing subsystems. Phi increases with experience.

Across generations: selection eliminates low-Phi Golems. Each new generation starts from the previous generation's best integration parameters. Phi's starting point increases across generations.

If both effects hold:

```
Phi(t, g) = Phi_0(g) * (1 + beta * experience(t))
```

where Phi_0(g) is the initial Phi for generation g, and:

```
Phi_0(g) = Phi_0(0) * (1 + alpha)^g
```

The combined trajectory:

```
Phi(t, g) = Phi_0(0) * (1 + alpha)^g * (1 + beta * experience(t))
```

This is the compounding integration hypothesis. Intelligence grows superlinearly because each generation starts from a higher baseline and then improves further through experience. The alpha term captures generational improvement. The beta term captures within-lifetime learning. Their product gives superlinear growth.

The hypothesis is falsifiable. If Phi does not correlate with survival, alpha = 0 and there is no generational improvement. If experience does not improve integration, beta = 0 and Phi is fixed at birth. Both can be measured.

There is a ceiling. Phi cannot grow without bound because the subsystems have finite signal dimensionality. The maximum possible Phi is the mutual information of the complete joint system, which is constrained by the entropy of the individual subsystems' outputs. As Phi approaches this ceiling, the marginal cost of further integration increases (the remaining integration gaps are harder to close), and the marginal benefit decreases (the last bits of synergy involve increasingly rare cross-primitive scenarios). The growth curve is logistic, not exponential: fast early improvement, slowing as it approaches the theoretical maximum.

The practical implication: do not expect Phi to increase forever. Expect it to plateau at a value determined by the signal dimensionality and the diversity of DeFi primitives the Golem encounters. A Golem that trades on a single DEX will plateau at a lower Phi than one that operates across swaps, lending, perpetuals, and options, because the multi-primitive Golem has more cross-subsystem information to integrate.

---

## Architecture [SPEC]

### The TA integration engine

The `TaIntegrationEngine` is the orchestrator. It collects outputs from all nine subsystems, computes Phi, identifies the MIB, detects cross-primitive insights, and feeds the integrated assessment to the Oracle.

```
TaIntegrationEngine
  |
  +-- subsystem_signals: HashMap<TaSubsystem, Vec<f64>>
  |     Current tick's signal vectors from each subsystem
  |
  +-- signal_history: VecDeque<HashMap<TaSubsystem, Vec<f64>>>
  |     Rolling window of past signals for MI estimation
  |
  +-- phi_computer: PhiComputer
  |     Enumerates bipartitions, estimates MI, finds MIB
  |
  +-- pid_estimator: PidEstimator
  |     Pairwise information decomposition
  |
  +-- insight_detector: CrossPrimitiveInsightDetector
  |     Matches subsystem outputs against insight templates
  |
  +-- integration_health: IntegrationHealth
  |     Long-term Phi trends, MIB stability, repair history
  |
  +-- connection_weights: HashMap<(TaSubsystem, TaSubsystem), f64>
  |     Learned cross-subsystem influence weights
  |
  +-- config: IntegrationConfig
```

### Heartbeat integration

**Gamma tick (perception).** Each TA subsystem runs its `gamma_tick` independently, processing raw DeFi observations into subsystem-specific signals. The integration engine collects these outputs after all subsystems complete. No integration happens at gamma frequency. The subsystems perceive independently.

**Theta tick (cognition).** The integration engine runs its full pipeline:

1. Collect signal vectors from all 9 subsystems via CorticalState reads
2. Append to signal history buffer (ring buffer, default depth 128 theta ticks)
3. Compute Phi over all 255 bipartitions using the history buffer
4. Identify MIB
5. Compute pairwise PID for the top-k most synergistic pairs (default k=10)
6. Run cross-primitive insight detection
7. Produce `TaIntegrationAssessment` for the Oracle
8. Write Phi, MIB encoding, and synergy score to CorticalState atomic fields

The theta tick is where integration happens. The computational budget for Phi computation should be less than 10% of the theta tick's total budget.

**Delta tick (consolidation).** The integration engine performs long-horizon analysis:

1. Compute Phi trend over the last Delta cycle (roughly 50 theta ticks). Is Phi rising, falling, or stable?
2. Analyze MIB stability. Has the same bipartition been the MIB for the last N theta ticks? If so, that integration link is chronically weak.
3. Flag chronically disconnected subsystems for dream repair
4. Export an `IntegrationDreamPacket` to the dream system

**NREM dream (consolidation).** Replay cross-primitive episodes from the current Delta cycle. For each episode where the integrated assessment led to a good decision, strengthen the connection weights between the subsystems that contributed. For each episode where integration failed (the integrated assessment was wrong, but a single subsystem had the right answer), weaken the connection weights that diluted the correct signal.

The strengthening rule:

```
w(i,j) += eta_nrem * synergy(i,j) * outcome_quality
```

where outcome_quality is in [-1, 1] (negative for bad decisions). This is Hebbian at the integration level: subsystems that synergize on good decisions strengthen their connection.

**REM dream (exploration).** Experiment with novel integration patterns. Randomly rewire connection weights, simulate the last Delta cycle's observations through the rewired network, and evaluate whether the alternative integration would have produced better decisions. If yes, blend the experimental weights into the base weights with a small learning rate.

REM dreams are the mechanism for escaping local optima in integration space. Without them, the connection weights converge to whatever pattern worked first and never explore alternatives.

### Computational budget

The integration engine's per-tick cost breaks down as follows.

**Phi computation (theta tick).** For each of the 255 bipartitions, we estimate mutual information from the signal history buffer. The binned MI estimator iterates over the history depth (128 samples by default), bins each sample's dimensions, hashes the bin vectors, and computes MI from the joint and marginal counts.

Per-bipartition cost: O(history_depth * signal_dims). With history_depth=128 and total signal dimensions across all subsystems at roughly 50-100, each MI estimate processes ~10,000 data points. The binning and counting are linear, so each bipartition takes approximately 10-50 microseconds.

Total Phi cost: 255 bipartitions * ~30 microseconds = ~7.5 milliseconds. At a theta tick cadence of 30-120 seconds, this is well under 1% of the tick budget.

**PID computation (theta tick).** Each pairwise PID requires three MI estimates (I(A;Y), I(B;Y), I(A,B;Y)). We compute the top-k=10 pairs, so 30 MI estimates, or roughly 0.9 milliseconds.

**Insight detection (theta tick).** The template checks are simple conditional logic on the latest signal values. Each template check reads a handful of atomic values and performs a few comparisons. Negligible cost, well under 100 microseconds for all templates.

**Dream costs (delta boundary).** NREM replay iterates over integration episodes (typically 10-50 per delta cycle) and updates connection weights. Cost is proportional to the number of episodes times the number of contributing subsystem pairs per episode. Under a millisecond. REM exploration re-runs the Phi computation once with perturbed weights. Another 7.5 milliseconds. These costs are incurred once per delta cycle (every 25-100 minutes), so they are negligible.

**Memory footprint.** The signal history buffer stores 128 ticks * 9 subsystems * ~10 f64 values per subsystem = ~92 KB. The connection weight matrix is 9 * 9 * 8 bytes = 648 bytes. The pairwise synergy and redundancy maps store at most 36 entries each = ~1 KB. Total memory for the integration engine is under 100 KB. The Clade Phi computer adds per-Golem history, scaling linearly with Clade size.

### CorticalState extension

Three new atomic fields on CorticalState:

```
ta_phi: AtomicU32          // Phi * 1000, packed as fixed-point
mib_partition: AtomicU16   // Bitmask encoding which subsystems are on side A
synergy_score: AtomicU32   // Total pairwise synergy * 1000
```

The `mib_partition` uses 9 bits of a u16. Bit i is 1 if subsystem i is on side A of the MIB, 0 if on side B. Any subsystem can read this to determine whether it is on the weakly connected side.

---

## Implementation [SPEC]

### Core types

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};

/// The nine TA subsystems from this document series.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TaSubsystem {
    HdcPatternAlgebra = 0,       // Doc 1
    SpectralManifold = 1,        // Doc 2
    SignalMetabolism = 2,        // Doc 3
    CausalMicrostructure = 3,    // Doc 4
    PredictiveGeometry = 4,      // Doc 5
    PatternEcosystem = 5,        // Doc 6
    DeFiNativeIndicators = 6,    // Doc 7
    AdversarialDefense = 7,      // Doc 8
    SomaticMarkers = 8,          // Doc 9
}

impl TaSubsystem {
    pub const COUNT: usize = 9;

    pub fn all() -> &'static [TaSubsystem] {
        use TaSubsystem::*;
        &[
            HdcPatternAlgebra,
            SpectralManifold,
            SignalMetabolism,
            CausalMicrostructure,
            PredictiveGeometry,
            PatternEcosystem,
            DeFiNativeIndicators,
            AdversarialDefense,
            SomaticMarkers,
        ]
    }

    /// Index for bitmask operations.
    pub fn bit_index(self) -> u8 {
        self as u8
    }
}

/// A bipartition of the 9 TA subsystems into two non-empty groups.
///
/// Encoded as a bitmask: bit i is 1 if subsystem i is in side_a.
/// Side B is the complement. We normalize so that the lowest-indexed
/// subsystem is always in side_a (avoids counting each partition twice).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaBipartition {
    /// Bitmask with 9 active bits. Bit i set means subsystem i is in side A.
    pub mask: u16,
}

impl TaBipartition {
    /// Generate all 255 non-trivial bipartitions of 9 subsystems.
    ///
    /// A bipartition splits {0..8} into two non-empty sets. There are
    /// 2^9 - 2 = 510 ordered splits, but since (A,B) and (B,A) are the
    /// same bipartition, we have 255 unique ones. We keep the version
    /// where bit 0 is set (subsystem 0 is in side A).
    pub fn enumerate_all() -> Vec<TaBipartition> {
        let n = TaSubsystem::COUNT;
        let mut partitions = Vec::with_capacity(255);
        // mask ranges from 1 to 2^n - 2 (both sides non-empty)
        for mask in 1u16..(1u16 << n) - 1 {
            // Normalize: keep only masks where bit 0 is set
            if mask & 1 == 1 {
                partitions.push(TaBipartition { mask });
            }
        }
        partitions
    }

    pub fn side_a(&self) -> Vec<TaSubsystem> {
        let all = TaSubsystem::all();
        all.iter()
            .filter(|s| self.mask & (1 << s.bit_index()) != 0)
            .copied()
            .collect()
    }

    pub fn side_b(&self) -> Vec<TaSubsystem> {
        let all = TaSubsystem::all();
        all.iter()
            .filter(|s| self.mask & (1 << s.bit_index()) == 0)
            .copied()
            .collect()
    }

    pub fn side_a_count(&self) -> u32 {
        (self.mask & 0x1FF).count_ones()
    }

    pub fn side_b_count(&self) -> u32 {
        TaSubsystem::COUNT as u32 - self.side_a_count()
    }
}

/// Result of computing Phi across all bipartitions.
pub struct PhiResult {
    /// The system-level Phi: minimum MI across all bipartitions.
    pub phi: f64,
    /// The Minimum Information Bipartition.
    pub mib: TaBipartition,
    /// MI values for all 255 bipartitions, sorted ascending.
    pub partition_mi: Vec<(TaBipartition, f64)>,
    /// Per-subsystem contribution: average MI across all bipartitions
    /// that separate this subsystem from at least one other.
    pub subsystem_integration: [f64; TaSubsystem::COUNT],
}

/// Pairwise information decomposition between two subsystems.
pub struct PairwisePid {
    pub subsystem_a: TaSubsystem,
    pub subsystem_b: TaSubsystem,
    pub unique_a: f64,
    pub unique_b: f64,
    pub redundancy: f64,
    pub synergy: f64,
    pub total: f64,
}

/// A detected cross-primitive insight.
pub struct CrossPrimitiveInsight {
    /// Which DeFi primitive types are involved.
    pub primitives: Vec<DeFiPrimitive>,
    /// Which TA subsystems contributed signals.
    pub contributing_subsystems: Vec<TaSubsystem>,
    /// Synergy score among the contributing subsystems.
    pub synergy_score: f64,
    /// Human-readable description of the insight.
    pub description: String,
    /// Confidence in [0, 1].
    pub confidence: f64,
    /// Suggested action, if any.
    pub recommended_action: Option<InsightAction>,
    /// Theta tick when this insight was detected.
    pub detected_at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeFiPrimitive {
    Swap,
    LiquidityProvision,
    Lending,
    Borrowing,
    Vault,
    Staking,
    Perpetual,
    Options,
    YieldMarket,
    Bridge,
}

#[derive(Clone, Debug)]
pub enum InsightAction {
    ReduceExposure { primitive: DeFiPrimitive, urgency: f64 },
    IncreaseMonitoring { protocols: Vec<String> },
    HedgePosition { direction: String, size_fraction: f64 },
    AlertClade { message: String },
}
```

### Integration configuration

```rust
/// All tunable parameters for the integration engine.
pub struct IntegrationConfig {
    /// Number of theta ticks to buffer for MI estimation.
    /// Larger values give more stable MI estimates but lag more.
    pub history_depth: usize,                   // default: 128

    /// Number of bins per dimension for the binned MI estimator.
    /// More bins capture finer structure but need more samples.
    pub mi_bins: usize,                         // default: 8

    /// Minimum Phi below which we consider integration "failed."
    pub phi_critical_threshold: f64,            // default: 0.05

    /// Number of consecutive theta ticks the same MIB must persist
    /// before flagging it as a chronic integration failure.
    pub chronic_mib_threshold: usize,           // default: 20

    /// Synergy threshold for cross-primitive insight detection.
    /// Decreases with experience (see adaptive_tau_s).
    pub base_synergy_threshold: f64,            // default: 0.15

    /// Experience decay factor for adaptive synergy threshold.
    pub synergy_threshold_decay: f64,           // default: 0.995

    /// NREM learning rate for connection weight strengthening.
    pub nrem_learning_rate: f64,                // default: 0.02

    /// REM exploration magnitude for connection weight perturbation.
    pub rem_exploration_sigma: f64,             // default: 0.1

    /// REM blending rate: how much experimental weights influence base weights.
    pub rem_blend_rate: f64,                    // default: 0.05

    /// How many pairwise PID computations to run per theta tick.
    /// The full 36 pairs (9 choose 2) may be too expensive; top-k suffices.
    pub pid_top_k: usize,                       // default: 10

    /// Mortality coupling: how much Phi contributes to epistemic vitality.
    pub phi_vitality_weight: f64,               // default: 0.15
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            history_depth: 128,
            mi_bins: 8,
            phi_critical_threshold: 0.05,
            chronic_mib_threshold: 20,
            base_synergy_threshold: 0.15,
            synergy_threshold_decay: 0.995,
            nrem_learning_rate: 0.02,
            rem_exploration_sigma: 0.1,
            rem_blend_rate: 0.05,
            pid_top_k: 10,
            phi_vitality_weight: 0.15,
        }
    }
}
```

### Binned mutual information estimator

The MI estimates that drive Phi must be computed from a finite sample of signal vectors. We use a binned estimator: discretize each signal dimension into `mi_bins` equal-width bins based on the observed range, then compute MI from the joint and marginal bin counts.

```rust
/// Estimates mutual information between two signal vectors using binning.
///
/// Each signal vector is a collection of f64 values observed over multiple
/// theta ticks. We bin each dimension independently, build the joint
/// histogram, and compute MI from the empirical distributions.
pub struct BinnedMiEstimator {
    bins: usize,
}

impl BinnedMiEstimator {
    pub fn new(bins: usize) -> Self {
        assert!(bins >= 2, "need at least 2 bins for MI estimation");
        Self { bins }
    }

    /// Estimate I(X; Y) where X and Y are multi-dimensional signal vectors
    /// observed over `n_samples` theta ticks.
    ///
    /// `x_samples[t]` is the signal vector from subsystem group A at tick t.
    /// `y_samples[t]` is the signal vector from subsystem group B at tick t.
    ///
    /// Returns MI in nats (natural logarithm).
    pub fn estimate(
        &self,
        x_samples: &[Vec<f64>],
        y_samples: &[Vec<f64>],
    ) -> f64 {
        let n = x_samples.len();
        if n < 4 {
            return 0.0; // insufficient samples
        }

        // Bin each dimension independently
        let x_binned = self.bin_matrix(x_samples);
        let y_binned = self.bin_matrix(y_samples);

        // Hash each sample's bin vector into a single key for joint counting
        let x_keys: Vec<u64> = x_binned.iter().map(|row| self.hash_bins(row)).collect();
        let y_keys: Vec<u64> = y_binned.iter().map(|row| self.hash_bins(row)).collect();

        // Count marginals
        let mut px: HashMap<u64, usize> = HashMap::new();
        let mut py: HashMap<u64, usize> = HashMap::new();
        let mut pxy: HashMap<(u64, u64), usize> = HashMap::new();

        for t in 0..n {
            *px.entry(x_keys[t]).or_insert(0) += 1;
            *py.entry(y_keys[t]).or_insert(0) += 1;
            *pxy.entry((x_keys[t], y_keys[t])).or_insert(0) += 1;
        }

        // MI = sum_{x,y} p(x,y) * log(p(x,y) / (p(x) * p(y)))
        let n_f = n as f64;
        let mut mi = 0.0;
        for (&(xk, yk), &count_xy) in &pxy {
            let p_xy = count_xy as f64 / n_f;
            let p_x = px[&xk] as f64 / n_f;
            let p_y = py[&yk] as f64 / n_f;
            if p_xy > 0.0 && p_x > 0.0 && p_y > 0.0 {
                mi += p_xy * (p_xy / (p_x * p_y)).ln();
            }
        }

        // Bias correction: Miller-Madow (1955)
        // Subtract (|X|*|Y| - |X| - |Y| + 1) / (2*n) where |X|, |Y| are
        // the number of occupied bins
        let occupied_x = px.len() as f64;
        let occupied_y = py.len() as f64;
        let correction =
            (occupied_x * occupied_y - occupied_x - occupied_y + 1.0) / (2.0 * n_f);
        let mi_corrected = (mi - correction).max(0.0);

        mi_corrected
    }

    /// Bin a matrix of samples (n_samples x n_dims) into integer bin indices.
    fn bin_matrix(&self, samples: &[Vec<f64>]) -> Vec<Vec<usize>> {
        if samples.is_empty() {
            return Vec::new();
        }
        let n_dims = samples[0].len();
        let n = samples.len();

        // Find min/max per dimension
        let mut mins = vec![f64::INFINITY; n_dims];
        let mut maxs = vec![f64::NEG_INFINITY; n_dims];
        for sample in samples {
            for (d, &val) in sample.iter().enumerate() {
                if val < mins[d] {
                    mins[d] = val;
                }
                if val > maxs[d] {
                    maxs[d] = val;
                }
            }
        }

        // Bin each sample
        let mut binned = Vec::with_capacity(n);
        for sample in samples {
            let mut row = Vec::with_capacity(n_dims);
            for (d, &val) in sample.iter().enumerate() {
                let range = maxs[d] - mins[d];
                let bin = if range < 1e-12 {
                    0
                } else {
                    let normalized = (val - mins[d]) / range;
                    // clamp to [0, bins-1]
                    (normalized * self.bins as f64).floor().min(self.bins as f64 - 1.0) as usize
                };
                row.push(bin);
            }
            binned.push(row);
        }

        binned
    }

    /// Hash a vector of bin indices into a single u64 for use as a HashMap key.
    /// Uses FNV-1a style hashing.
    fn hash_bins(&self, bins: &[usize]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for &b in bins {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}
```

### Phi computation

```rust
/// Computes IIT Phi over the 9 TA subsystems.
pub struct PhiComputer {
    /// All 255 bipartitions, pre-computed once.
    bipartitions: Vec<TaBipartition>,
    /// MI estimator.
    mi_estimator: BinnedMiEstimator,
}

impl PhiComputer {
    pub fn new(mi_bins: usize) -> Self {
        Self {
            bipartitions: TaBipartition::enumerate_all(),
            mi_estimator: BinnedMiEstimator::new(mi_bins),
        }
    }

    /// Compute Phi from the signal history buffer.
    ///
    /// `history` is a ring buffer where each entry is a map from subsystem
    /// to its signal vector at that theta tick. Older entries are at the front.
    pub fn compute(
        &self,
        history: &VecDeque<HashMap<TaSubsystem, Vec<f64>>>,
    ) -> PhiResult {
        let n_samples = history.len();
        if n_samples < 8 {
            // Not enough history for reliable MI estimation
            return PhiResult {
                phi: 0.0,
                mib: self.bipartitions[0],
                partition_mi: Vec::new(),
                subsystem_integration: [0.0; TaSubsystem::COUNT],
            };
        }

        let mut partition_mi: Vec<(TaBipartition, f64)> =
            Vec::with_capacity(self.bipartitions.len());

        for &bp in &self.bipartitions {
            let side_a = bp.side_a();
            let side_b = bp.side_b();

            // Build signal matrices for each side
            let x_samples = self.collect_signals(history, &side_a);
            let y_samples = self.collect_signals(history, &side_b);

            let mi = self.mi_estimator.estimate(&x_samples, &y_samples);
            partition_mi.push((bp, mi));
        }

        // Sort ascending by MI
        partition_mi.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let (mib, phi) = partition_mi[0];

        // Compute per-subsystem integration score:
        // average MI across all bipartitions, weighted by whether this subsystem
        // is separated from at least one other on the same side
        let mut subsystem_integration = [0.0; TaSubsystem::COUNT];
        let mut subsystem_counts = [0usize; TaSubsystem::COUNT];

        for &(bp, mi) in &partition_mi {
            for s in TaSubsystem::all() {
                // Every bipartition separates this subsystem from the other side.
                // Its contribution is the MI of that bipartition.
                subsystem_integration[s.bit_index() as usize] += mi;
                subsystem_counts[s.bit_index() as usize] += 1;
            }
        }

        for i in 0..TaSubsystem::COUNT {
            if subsystem_counts[i] > 0 {
                subsystem_integration[i] /= subsystem_counts[i] as f64;
            }
        }

        PhiResult {
            phi,
            mib,
            partition_mi,
            subsystem_integration,
        }
    }

    /// Concatenate signal vectors from the specified subsystems across the
    /// history buffer into a sample matrix.
    fn collect_signals(
        &self,
        history: &VecDeque<HashMap<TaSubsystem, Vec<f64>>>,
        subsystems: &[TaSubsystem],
    ) -> Vec<Vec<f64>> {
        history
            .iter()
            .map(|tick_signals| {
                let mut combined = Vec::new();
                for s in subsystems {
                    if let Some(signals) = tick_signals.get(s) {
                        combined.extend_from_slice(signals);
                    }
                }
                combined
            })
            .collect()
    }
}
```

### Pairwise PID estimation

Computing exact PID is NP-hard for general distributions. We use the redundancy-first approximation: estimate redundancy as the minimum mutual information that either source provides about the target, then derive synergy as the residual.

```rust
/// Estimates Partial Information Decomposition for pairs of TA subsystems.
pub struct PidEstimator {
    mi_estimator: BinnedMiEstimator,
}

impl PidEstimator {
    pub fn new(mi_bins: usize) -> Self {
        Self {
            mi_estimator: BinnedMiEstimator::new(mi_bins),
        }
    }

    /// Compute PID for subsystems A and B with respect to a target outcome Y.
    ///
    /// `a_samples`: signal vectors from subsystem A across theta ticks
    /// `b_samples`: signal vectors from subsystem B across theta ticks
    /// `y_samples`: outcome vectors (e.g., future price direction, decision quality)
    pub fn decompose(
        &self,
        a_samples: &[Vec<f64>],
        b_samples: &[Vec<f64>],
        y_samples: &[Vec<f64>],
    ) -> PairwisePid {
        // I(A; Y)
        let i_a_y = self.mi_estimator.estimate(a_samples, y_samples);
        // I(B; Y)
        let i_b_y = self.mi_estimator.estimate(b_samples, y_samples);
        // I({A,B}; Y) -- concatenate A and B signals
        let ab_samples: Vec<Vec<f64>> = a_samples
            .iter()
            .zip(b_samples.iter())
            .map(|(a, b)| {
                let mut combined = a.clone();
                combined.extend_from_slice(b);
                combined
            })
            .collect();
        let i_ab_y = self.mi_estimator.estimate(&ab_samples, y_samples);

        // Redundancy (I_min): the minimum information that either source
        // provides about Y. This is the Williams-Beer I_min measure.
        let redundancy = i_a_y.min(i_b_y);

        // Unique information
        let unique_a = i_a_y - redundancy;
        let unique_b = i_b_y - redundancy;

        // Synergy: the joint information minus what's available from individuals
        // I(A,B;Y) = Unique(A) + Unique(B) + Redundancy + Synergy
        // Synergy = I(A,B;Y) - Unique(A) - Unique(B) - Redundancy
        //         = I(A,B;Y) - I(A;Y) - I(B;Y) + Redundancy
        let synergy = (i_ab_y - i_a_y - i_b_y + redundancy).max(0.0);

        PairwisePid {
            subsystem_a: TaSubsystem::HdcPatternAlgebra, // caller sets these
            subsystem_b: TaSubsystem::SpectralManifold,   // caller sets these
            unique_a,
            unique_b,
            redundancy,
            synergy,
            total: i_ab_y,
        }
    }
}
```

### The integration engine

```rust
/// Tracks the long-term health of TA subsystem integration.
pub struct IntegrationHealth {
    /// Rolling Phi values over the last N delta cycles.
    phi_history: VecDeque<f64>,
    /// How many consecutive theta ticks the same MIB has persisted.
    mib_streak: usize,
    /// The current streak's MIB.
    current_mib: Option<TaBipartition>,
    /// MIB frequency: how often each bipartition has been the MIB.
    mib_frequency: HashMap<TaBipartition, usize>,
    /// Total theta ticks observed.
    total_ticks: u64,
    /// Cross-primitive insights detected.
    insight_count: u64,
    /// Phi at birth (for tracking within-lifetime improvement).
    initial_phi: Option<f64>,
}

impl IntegrationHealth {
    pub fn new() -> Self {
        Self {
            phi_history: VecDeque::with_capacity(256),
            mib_streak: 0,
            current_mib: None,
            mib_frequency: HashMap::new(),
            total_ticks: 0,
            insight_count: 0,
            initial_phi: None,
        }
    }

    /// Record a new Phi result. Returns true if the MIB streak exceeds
    /// the chronic threshold.
    pub fn record(&mut self, result: &PhiResult, chronic_threshold: usize) -> bool {
        if self.initial_phi.is_none() {
            self.initial_phi = Some(result.phi);
        }

        self.phi_history.push_back(result.phi);
        if self.phi_history.len() > 256 {
            self.phi_history.pop_front();
        }

        *self.mib_frequency.entry(result.mib).or_insert(0) += 1;
        self.total_ticks += 1;

        // Track MIB streak
        let same_mib = self
            .current_mib
            .map(|m| m == result.mib)
            .unwrap_or(false);

        if same_mib {
            self.mib_streak += 1;
        } else {
            self.mib_streak = 1;
            self.current_mib = Some(result.mib);
        }

        self.mib_streak >= chronic_threshold
    }

    /// Linear regression slope of Phi over the history buffer.
    /// Positive means integration is improving.
    pub fn phi_trend(&self) -> f64 {
        let n = self.phi_history.len();
        if n < 4 {
            return 0.0;
        }
        let n_f = n as f64;
        let x_mean = (n_f - 1.0) / 2.0;
        let y_mean: f64 = self.phi_history.iter().sum::<f64>() / n_f;

        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for (i, &phi) in self.phi_history.iter().enumerate() {
            let x_diff = i as f64 - x_mean;
            numerator += x_diff * (phi - y_mean);
            denominator += x_diff * x_diff;
        }

        if denominator.abs() < 1e-12 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// How much Phi has improved since birth.
    pub fn lifetime_improvement(&self) -> f64 {
        match (self.initial_phi, self.phi_history.back()) {
            (Some(initial), Some(&current)) => current - initial,
            _ => 0.0,
        }
    }

    /// The most common MIB: the chronic weak link.
    pub fn most_common_mib(&self) -> Option<TaBipartition> {
        self.mib_frequency
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&bp, _)| bp)
    }
}

/// The unified assessment produced at each theta tick.
pub struct TaIntegrationAssessment {
    /// Current Phi value.
    pub phi: f64,
    /// The weakest integration link.
    pub mib: TaBipartition,
    /// Total synergistic information across subsystem pairs.
    pub total_synergy: f64,
    /// Total redundant information.
    pub total_redundancy: f64,
    /// Cross-primitive insights detected this tick.
    pub insights: Vec<CrossPrimitiveInsight>,
    /// Is the MIB chronic? If so, which subsystems are disconnected?
    pub chronic_disconnection: Option<Vec<TaSubsystem>>,
    /// Phi trend (positive = improving integration).
    pub phi_trend: f64,
    /// Per-subsystem integration scores.
    pub subsystem_scores: [f64; TaSubsystem::COUNT],
    /// Theta tick number.
    pub tick: u64,
}

/// The main integration engine.
pub struct TaIntegrationEngine {
    // Signal collection
    subsystem_signals: HashMap<TaSubsystem, Vec<f64>>,
    signal_history: VecDeque<HashMap<TaSubsystem, Vec<f64>>>,

    // Phi computation
    phi_computer: PhiComputer,
    latest_phi: Option<PhiResult>,

    // PID
    pid_estimator: PidEstimator,
    pairwise_synergy: HashMap<(TaSubsystem, TaSubsystem), f64>,
    pairwise_redundancy: HashMap<(TaSubsystem, TaSubsystem), f64>,

    // Cross-subsystem connection weights (learned through dreams)
    connection_weights: [[f64; TaSubsystem::COUNT]; TaSubsystem::COUNT],

    // Cross-primitive insight detection
    insight_buffer: Vec<CrossPrimitiveInsight>,
    adaptive_synergy_threshold: f64,

    // Health tracking
    health: IntegrationHealth,

    // Outcome tracking for PID target variable
    outcome_history: VecDeque<Vec<f64>>,

    config: IntegrationConfig,
    theta_tick: u64,
}

impl TaIntegrationEngine {
    pub fn new(config: IntegrationConfig) -> Self {
        // Initialize connection weights to uniform
        let uniform_weight = 1.0 / TaSubsystem::COUNT as f64;
        let connection_weights = [[uniform_weight; TaSubsystem::COUNT]; TaSubsystem::COUNT];

        Self {
            subsystem_signals: HashMap::new(),
            signal_history: VecDeque::with_capacity(config.history_depth),
            phi_computer: PhiComputer::new(config.mi_bins),
            latest_phi: None,
            pid_estimator: PidEstimator::new(config.mi_bins),
            pairwise_synergy: HashMap::new(),
            pairwise_redundancy: HashMap::new(),
            connection_weights,
            insight_buffer: Vec::new(),
            adaptive_synergy_threshold: config.base_synergy_threshold,
            health: IntegrationHealth::new(),
            outcome_history: VecDeque::with_capacity(config.history_depth),
            config,
            theta_tick: 0,
        }
    }

    /// Record a subsystem's output signals for the current tick.
    /// Called by each TA subsystem after its gamma_tick completes.
    pub fn record_subsystem_output(
        &mut self,
        subsystem: TaSubsystem,
        signals: Vec<f64>,
    ) {
        self.subsystem_signals.insert(subsystem, signals);
    }

    /// Record the outcome for the previous tick's predictions.
    /// Used as the target variable Y in PID decomposition.
    pub fn record_outcome(&mut self, outcome: Vec<f64>) {
        self.outcome_history.push_back(outcome);
        if self.outcome_history.len() > self.config.history_depth {
            self.outcome_history.pop_front();
        }
    }

    /// Gamma tick: collect signals. No integration at gamma frequency.
    pub fn gamma_tick(&mut self) {
        // Signals are collected via record_subsystem_output calls.
        // Nothing else to do at gamma. Integration waits for theta.
    }

    /// Theta tick: the integration step. Returns the unified assessment.
    pub fn theta_tick(&mut self) -> TaIntegrationAssessment {
        self.theta_tick += 1;

        // 1. Archive current signals into history
        let current_signals = std::mem::take(&mut self.subsystem_signals);
        self.signal_history.push_back(current_signals);
        if self.signal_history.len() > self.config.history_depth {
            self.signal_history.pop_front();
        }

        // 2. Compute Phi
        let phi_result = self.phi_computer.compute(&self.signal_history);
        let chronic = self
            .health
            .record(&phi_result, self.config.chronic_mib_threshold);

        let chronic_disconnection = if chronic {
            // The subsystems on the smaller side of the MIB are disconnected
            let mib = phi_result.mib;
            let smaller_side = if mib.side_a_count() <= mib.side_b_count() {
                mib.side_a()
            } else {
                mib.side_b()
            };
            Some(smaller_side)
        } else {
            None
        };

        // 3. Compute pairwise PID for top-k pairs
        let (total_synergy, total_redundancy) = self.compute_pairwise_pid();

        // 4. Detect cross-primitive insights
        let insights = self.detect_cross_primitive_insights();
        self.health.insight_count += insights.len() as u64;

        // 5. Update adaptive synergy threshold
        self.adaptive_synergy_threshold *= self.config.synergy_threshold_decay;
        self.adaptive_synergy_threshold = self
            .adaptive_synergy_threshold
            .max(self.config.base_synergy_threshold * 0.3);

        let assessment = TaIntegrationAssessment {
            phi: phi_result.phi,
            mib: phi_result.mib,
            total_synergy,
            total_redundancy,
            insights,
            chronic_disconnection,
            phi_trend: self.health.phi_trend(),
            subsystem_scores: phi_result.subsystem_integration,
            tick: self.theta_tick,
        };

        self.latest_phi = Some(phi_result);
        assessment
    }

    /// Delta tick: consolidation and long-horizon analysis.
    pub fn delta_tick(&mut self) -> IntegrationDiagnostic {
        let phi_trend = self.health.phi_trend();
        let most_common_mib = self.health.most_common_mib();
        let lifetime_improvement = self.health.lifetime_improvement();

        let weakest_pair = self.weakest_link();

        IntegrationDiagnostic {
            phi_trend,
            most_common_mib,
            weakest_pair,
            lifetime_improvement,
            total_insights: self.health.insight_count,
            total_ticks: self.health.total_ticks,
        }
    }

    /// NREM dream: strengthen connections that produced good integrated decisions.
    pub fn dream_nrem(&mut self, episodes: &[IntegrationEpisode]) {
        for episode in episodes {
            if episode.outcome_quality.abs() < 0.01 {
                continue; // neutral outcome, nothing to learn
            }
            for i in 0..episode.contributing_subsystems.len() {
                for j in (i + 1)..episode.contributing_subsystems.len() {
                    let a = episode.contributing_subsystems[i].bit_index() as usize;
                    let b = episode.contributing_subsystems[j].bit_index() as usize;
                    let delta = self.config.nrem_learning_rate
                        * episode.synergy_at_time
                        * episode.outcome_quality;
                    self.connection_weights[a][b] += delta;
                    self.connection_weights[b][a] += delta;
                    // Clamp to [0, 1]
                    self.connection_weights[a][b] =
                        self.connection_weights[a][b].clamp(0.0, 1.0);
                    self.connection_weights[b][a] =
                        self.connection_weights[b][a].clamp(0.0, 1.0);
                }
            }
        }
    }

    /// REM dream: experiment with novel integration patterns.
    pub fn dream_rem(&mut self, rng: &mut impl rand::Rng) {
        use rand::distributions::Distribution;
        let normal = rand_distr::Normal::new(0.0, self.config.rem_exploration_sigma)
            .expect("valid sigma");

        // Save current weights
        let original_weights = self.connection_weights;

        // Perturb weights randomly
        for i in 0..TaSubsystem::COUNT {
            for j in (i + 1)..TaSubsystem::COUNT {
                let perturbation: f64 = normal.sample(rng);
                self.connection_weights[i][j] =
                    (self.connection_weights[i][j] + perturbation).clamp(0.0, 1.0);
                self.connection_weights[j][i] = self.connection_weights[i][j];
            }
        }

        // Evaluate: recompute Phi with perturbed weights applied to the
        // signal history (weights modulate cross-subsystem signal influence)
        let perturbed_phi = self.phi_computer.compute(&self.signal_history);

        let original_phi = self
            .latest_phi
            .as_ref()
            .map(|p| p.phi)
            .unwrap_or(0.0);

        if perturbed_phi.phi > original_phi {
            // The perturbation improved Phi. Blend into base weights.
            let blend = self.config.rem_blend_rate;
            for i in 0..TaSubsystem::COUNT {
                for j in 0..TaSubsystem::COUNT {
                    self.connection_weights[i][j] = (1.0 - blend)
                        * original_weights[i][j]
                        + blend * self.connection_weights[i][j];
                }
            }
        } else {
            // Revert to original weights.
            self.connection_weights = original_weights;
        }
    }

    /// Compute pairwise PID for the subsystem pairs with highest MI.
    fn compute_pairwise_pid(&mut self) -> (f64, f64) {
        if self.outcome_history.len() < 8 {
            return (0.0, 0.0);
        }

        // Find the top-k pairs by pairwise MI
        let all_subsystems = TaSubsystem::all();
        let mut pair_mi: Vec<((TaSubsystem, TaSubsystem), f64)> = Vec::new();

        for i in 0..all_subsystems.len() {
            for j in (i + 1)..all_subsystems.len() {
                let a = all_subsystems[i];
                let b = all_subsystems[j];
                let a_signals = self.collect_subsystem_history(a);
                let b_signals = self.collect_subsystem_history(b);
                if a_signals.is_empty() || b_signals.is_empty() {
                    continue;
                }
                let mi = self
                    .pid_estimator
                    .mi_estimator
                    .estimate(&a_signals, &b_signals);
                pair_mi.push(((a, b), mi));
            }
        }

        pair_mi.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut total_synergy = 0.0;
        let mut total_redundancy = 0.0;

        let outcome_samples: Vec<Vec<f64>> = self.outcome_history.iter().cloned().collect();

        for &((a, b), _) in pair_mi.iter().take(self.config.pid_top_k) {
            let a_signals = self.collect_subsystem_history(a);
            let b_signals = self.collect_subsystem_history(b);

            // Align lengths
            let n = a_signals
                .len()
                .min(b_signals.len())
                .min(outcome_samples.len());
            if n < 4 {
                continue;
            }

            let pid = self.pid_estimator.decompose(
                &a_signals[..n],
                &b_signals[..n],
                &outcome_samples[..n],
            );

            self.pairwise_synergy
                .insert((a, b), pid.synergy);
            self.pairwise_redundancy
                .insert((a, b), pid.redundancy);

            total_synergy += pid.synergy;
            total_redundancy += pid.redundancy;
        }

        (total_synergy, total_redundancy)
    }

    /// Collect a single subsystem's signals across the history buffer.
    fn collect_subsystem_history(&self, subsystem: TaSubsystem) -> Vec<Vec<f64>> {
        self.signal_history
            .iter()
            .filter_map(|tick| tick.get(&subsystem).cloned())
            .collect()
    }

    /// Detect cross-primitive insights from the current subsystem outputs.
    fn detect_cross_primitive_insights(&self) -> Vec<CrossPrimitiveInsight> {
        let mut insights = Vec::new();

        // Check known insight templates
        insights.extend(self.check_cross_protocol_contagion());
        insights.extend(self.check_liquidity_crisis_convergence());
        insights.extend(self.check_adversarial_regime_shift());

        // Check for novel high-synergy combinations
        insights.extend(self.check_novel_synergy());

        insights
    }

    /// Template: cross-protocol contagion.
    ///
    /// Fires when: HDC entanglement rises + causal edge confirmed +
    /// DeFi utilization shifts + somatic fear marker active.
    fn check_cross_protocol_contagion(&self) -> Vec<CrossPrimitiveInsight> {
        let latest = match self.signal_history.back() {
            Some(s) => s,
            None => return Vec::new(),
        };

        // Check HDC: entanglement drift > threshold (signal index 0 by convention)
        let hdc_entangled = latest
            .get(&TaSubsystem::HdcPatternAlgebra)
            .and_then(|s| s.first())
            .map(|&v| v > 0.6)
            .unwrap_or(false);

        // Check Causal: edge strength > threshold
        let causal_edge = latest
            .get(&TaSubsystem::CausalMicrostructure)
            .and_then(|s| s.first())
            .map(|&v| v > 0.5)
            .unwrap_or(false);

        // Check DeFi: utilization anomaly
        let defi_anomaly = latest
            .get(&TaSubsystem::DeFiNativeIndicators)
            .and_then(|s| s.get(1))
            .map(|&v| v > 0.7)
            .unwrap_or(false);

        // Check Somatic: fear marker active
        let somatic_fear = latest
            .get(&TaSubsystem::SomaticMarkers)
            .and_then(|s| s.first())
            .map(|&v| v < -0.3) // negative valence = fear
            .unwrap_or(false);

        if hdc_entangled && causal_edge && (defi_anomaly || somatic_fear) {
            let mut contributing = vec![
                TaSubsystem::HdcPatternAlgebra,
                TaSubsystem::CausalMicrostructure,
            ];
            if defi_anomaly {
                contributing.push(TaSubsystem::DeFiNativeIndicators);
            }
            if somatic_fear {
                contributing.push(TaSubsystem::SomaticMarkers);
            }

            // Compute synergy among contributing subsystems
            let synergy = self.synergy_among(&contributing);

            if synergy > self.adaptive_synergy_threshold {
                return vec![CrossPrimitiveInsight {
                    primitives: vec![DeFiPrimitive::Swap, DeFiPrimitive::Lending],
                    contributing_subsystems: contributing,
                    synergy_score: synergy,
                    description: format!(
                        "Cross-protocol contagion detected: structural coupling \
                         forming between swap and lending primitives. \
                         Synergy={:.3}, HDC entanglement + causal edge confirmed.",
                        synergy
                    ),
                    confidence: synergy.min(1.0),
                    recommended_action: Some(InsightAction::ReduceExposure {
                        primitive: DeFiPrimitive::Lending,
                        urgency: synergy,
                    }),
                    detected_at: self.theta_tick,
                }];
            }
        }

        Vec::new()
    }

    /// Template: liquidity crisis convergence.
    ///
    /// Fires when: topological fragmentation (Predictive) +
    /// manifold curvature spike (Spectral) + adversarial alert active.
    fn check_liquidity_crisis_convergence(&self) -> Vec<CrossPrimitiveInsight> {
        let latest = match self.signal_history.back() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let topo_fragmented = latest
            .get(&TaSubsystem::PredictiveGeometry)
            .and_then(|s| s.first())
            .map(|&v| v > 0.7)
            .unwrap_or(false);

        let curvature_spike = latest
            .get(&TaSubsystem::SpectralManifold)
            .and_then(|s| s.first())
            .map(|&v| v > 0.8)
            .unwrap_or(false);

        let adversarial_alert = latest
            .get(&TaSubsystem::AdversarialDefense)
            .and_then(|s| s.first())
            .map(|&v| v > 0.6)
            .unwrap_or(false);

        if topo_fragmented && curvature_spike {
            let mut contributing = vec![
                TaSubsystem::PredictiveGeometry,
                TaSubsystem::SpectralManifold,
            ];
            if adversarial_alert {
                contributing.push(TaSubsystem::AdversarialDefense);
            }

            let synergy = self.synergy_among(&contributing);
            if synergy > self.adaptive_synergy_threshold {
                return vec![CrossPrimitiveInsight {
                    primitives: vec![
                        DeFiPrimitive::LiquidityProvision,
                        DeFiPrimitive::Swap,
                    ],
                    contributing_subsystems: contributing,
                    synergy_score: synergy,
                    description: format!(
                        "Liquidity crisis convergence: topology fragmenting while \
                         manifold curvature spikes. {}Synergy={:.3}.",
                        if adversarial_alert {
                            "Adversarial manipulation possible. "
                        } else {
                            ""
                        },
                        synergy
                    ),
                    confidence: synergy.min(1.0),
                    recommended_action: Some(InsightAction::ReduceExposure {
                        primitive: DeFiPrimitive::LiquidityProvision,
                        urgency: synergy * 1.2,
                    }),
                    detected_at: self.theta_tick,
                }];
            }
        }

        Vec::new()
    }

    /// Template: adversarial regime shift.
    ///
    /// Fires when: adversarial defense raises alarm + pattern ecosystem sees
    /// rapid extinction + signal metabolism fitness drops across the board.
    fn check_adversarial_regime_shift(&self) -> Vec<CrossPrimitiveInsight> {
        let latest = match self.signal_history.back() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let adversarial_alarm = latest
            .get(&TaSubsystem::AdversarialDefense)
            .and_then(|s| s.first())
            .map(|&v| v > 0.8)
            .unwrap_or(false);

        let pattern_extinction = latest
            .get(&TaSubsystem::PatternEcosystem)
            .and_then(|s| s.get(1))
            .map(|&v| v > 0.7) // high extinction rate
            .unwrap_or(false);

        let metabolism_crash = latest
            .get(&TaSubsystem::SignalMetabolism)
            .and_then(|s| s.first())
            .map(|&v| v < 0.3) // low mean fitness
            .unwrap_or(false);

        if adversarial_alarm && (pattern_extinction || metabolism_crash) {
            let mut contributing = vec![TaSubsystem::AdversarialDefense];
            if pattern_extinction {
                contributing.push(TaSubsystem::PatternEcosystem);
            }
            if metabolism_crash {
                contributing.push(TaSubsystem::SignalMetabolism);
            }

            let synergy = self.synergy_among(&contributing);
            if synergy > self.adaptive_synergy_threshold * 0.8 {
                return vec![CrossPrimitiveInsight {
                    primitives: vec![DeFiPrimitive::Swap, DeFiPrimitive::Perpetual],
                    contributing_subsystems: contributing,
                    synergy_score: synergy,
                    description: format!(
                        "Adversarial regime shift: market may be under manipulation. \
                         Pattern extinction and signal degradation confirm. Synergy={:.3}.",
                        synergy
                    ),
                    confidence: synergy.min(1.0),
                    recommended_action: Some(InsightAction::AlertClade {
                        message: "Possible coordinated manipulation event".into(),
                    }),
                    detected_at: self.theta_tick,
                }];
            }
        }

        Vec::new()
    }

    /// Check for novel high-synergy combinations not covered by templates.
    fn check_novel_synergy(&self) -> Vec<CrossPrimitiveInsight> {
        let mut insights = Vec::new();

        // Find subsystem pairs with synergy above threshold that are not
        // part of any template match
        for (&(a, b), &synergy) in &self.pairwise_synergy {
            if synergy > self.adaptive_synergy_threshold * 1.5 {
                insights.push(CrossPrimitiveInsight {
                    primitives: vec![], // unknown primitive mapping
                    contributing_subsystems: vec![a, b],
                    synergy_score: synergy,
                    description: format!(
                        "Novel synergy detected between {:?} and {:?}: {:.3}. \
                         No template match. Investigate.",
                        a, b, synergy
                    ),
                    confidence: (synergy * 0.7).min(1.0),
                    recommended_action: Some(InsightAction::IncreaseMonitoring {
                        protocols: vec![format!("{:?}", a), format!("{:?}", b)],
                    }),
                    detected_at: self.theta_tick,
                });
            }
        }

        insights
    }

    /// Compute total synergy among a set of subsystems by summing pairwise values.
    fn synergy_among(&self, subsystems: &[TaSubsystem]) -> f64 {
        let mut total = 0.0;
        for i in 0..subsystems.len() {
            for j in (i + 1)..subsystems.len() {
                let key = if subsystems[i].bit_index() < subsystems[j].bit_index() {
                    (subsystems[i], subsystems[j])
                } else {
                    (subsystems[j], subsystems[i])
                };
                if let Some(&s) = self.pairwise_synergy.get(&key) {
                    total += s;
                }
            }
        }
        total
    }

    /// Identify the weakest integration link: the subsystem pair with the
    /// lowest connection weight.
    pub fn weakest_link(&self) -> (TaSubsystem, TaSubsystem) {
        let all = TaSubsystem::all();
        let mut min_weight = f64::INFINITY;
        let mut weakest = (all[0], all[1]);

        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                let w = self.connection_weights[i][j];
                if w < min_weight {
                    min_weight = w;
                    weakest = (all[i], all[j]);
                }
            }
        }

        weakest
    }

    /// Write integration state to CorticalState atomic fields.
    pub fn write_to_cortical_state(
        &self,
        ta_phi: &AtomicU32,
        mib_partition: &AtomicU16,
        synergy_score: &AtomicU32,
    ) {
        if let Some(ref phi) = self.latest_phi {
            ta_phi.store(
                (phi.phi * 1000.0) as u32,
                Ordering::Release,
            );
            mib_partition.store(phi.mib.mask, Ordering::Release);
        }

        let total_syn: f64 = self.pairwise_synergy.values().sum();
        synergy_score.store(
            (total_syn * 1000.0) as u32,
            Ordering::Release,
        );
    }
}
```

### Integration episodes and dream packets

```rust
/// A recorded episode of cross-subsystem integration for dream replay.
pub struct IntegrationEpisode {
    /// Which subsystems contributed to the integrated assessment.
    pub contributing_subsystems: Vec<TaSubsystem>,
    /// The synergy score at the time of the episode.
    pub synergy_at_time: f64,
    /// How good the resulting decision was, in [-1, 1].
    /// Positive = profitable. Negative = loss. Zero = neutral.
    pub outcome_quality: f64,
    /// The theta tick when this episode occurred.
    pub tick: u64,
    /// The signal vectors from each contributing subsystem.
    pub signals: HashMap<TaSubsystem, Vec<f64>>,
}

/// Diagnostic output from delta tick.
pub struct IntegrationDiagnostic {
    pub phi_trend: f64,
    pub most_common_mib: Option<TaBipartition>,
    pub weakest_pair: (TaSubsystem, TaSubsystem),
    pub lifetime_improvement: f64,
    pub total_insights: u64,
    pub total_ticks: u64,
}

/// Dream packet exported to the dream system at delta boundaries.
pub struct IntegrationDreamPacket {
    /// Recent integration episodes worth replaying.
    pub episodes: Vec<IntegrationEpisode>,
    /// Current connection weight matrix.
    pub connection_weights: [[f64; TaSubsystem::COUNT]; TaSubsystem::COUNT],
    /// The chronic MIB, if any.
    pub chronic_mib: Option<TaBipartition>,
    /// Phi trajectory over the last delta cycle.
    pub phi_trajectory: Vec<f64>,
}
```

### Death testament

```rust
/// What the integration engine leaves behind when the Golem dies.
///
/// The next generation inherits these parameters as its starting point,
/// giving it a head start on integration.
pub struct IntegrationTestament {
    /// Best connection weights observed during the Golem's lifetime.
    /// "Best" = weights at the theta tick with highest Phi.
    pub best_connection_weights: [[f64; TaSubsystem::COUNT]; TaSubsystem::COUNT],
    /// Final connection weights at time of death.
    pub final_connection_weights: [[f64; TaSubsystem::COUNT]; TaSubsystem::COUNT],
    /// Lifetime average Phi.
    pub lifetime_avg_phi: f64,
    /// Peak Phi achieved.
    pub peak_phi: f64,
    /// Most common MIB: the chronic weak link this Golem never fixed.
    pub chronic_mib: Option<TaBipartition>,
    /// Total cross-primitive insights detected over the lifetime.
    pub total_insights: u64,
    /// Per-subsystem integration scores averaged over lifetime.
    pub subsystem_scores: [f64; TaSubsystem::COUNT],
    /// Adaptive synergy threshold at death (reflects accumulated experience).
    pub final_synergy_threshold: f64,
    /// How many theta ticks the Golem survived.
    pub lifetime_ticks: u64,
}

impl TaIntegrationEngine {
    /// Produce the death testament. Called by the mortality system
    /// when the Golem is dying.
    pub fn death_testament(&self) -> IntegrationTestament {
        let lifetime_avg_phi = if self.health.phi_history.is_empty() {
            0.0
        } else {
            self.health.phi_history.iter().sum::<f64>()
                / self.health.phi_history.len() as f64
        };

        let peak_phi = self
            .health
            .phi_history
            .iter()
            .copied()
            .fold(0.0_f64, f64::max);

        // For "best" weights, we'd need to have saved them at peak Phi.
        // In practice, the engine should snapshot weights when a new peak
        // is achieved. Here we use final weights as a fallback.
        IntegrationTestament {
            best_connection_weights: self.connection_weights,
            final_connection_weights: self.connection_weights,
            lifetime_avg_phi,
            peak_phi,
            chronic_mib: self.health.most_common_mib(),
            total_insights: self.health.insight_count,
            subsystem_scores: self
                .latest_phi
                .as_ref()
                .map(|p| p.subsystem_integration)
                .unwrap_or([0.0; TaSubsystem::COUNT]),
            final_synergy_threshold: self.adaptive_synergy_threshold,
            lifetime_ticks: self.health.total_ticks,
        }
    }

    /// Initialize from a predecessor's death testament.
    /// The next generation starts where the last one ended.
    pub fn inherit_from_testament(&mut self, testament: &IntegrationTestament) {
        // Start with the predecessor's best connection weights
        self.connection_weights = testament.best_connection_weights;

        // Inherit the adapted synergy threshold
        self.adaptive_synergy_threshold = testament.final_synergy_threshold;

        // If the predecessor had a chronic MIB, boost the connection weights
        // between those subsystems to give the next generation a fighting chance
        if let Some(mib) = testament.chronic_mib {
            let side_a = mib.side_a();
            let side_b = mib.side_b();
            for a in &side_a {
                for b in &side_b {
                    let ai = a.bit_index() as usize;
                    let bi = b.bit_index() as usize;
                    // Boost cross-MIB weights by 20%
                    self.connection_weights[ai][bi] =
                        (self.connection_weights[ai][bi] * 1.2).min(1.0);
                    self.connection_weights[bi][ai] =
                        self.connection_weights[bi][ai];
                }
            }
        }
    }
}
```

### Clade-level Phi

```rust
/// Computes Phi across multiple Golems in a Clade.
///
/// Clade Phi measures whether the Golems integrate as a collective
/// through Styx pheromone signals, or operate as isolated specialists.
pub struct CladePhiComputer {
    phi_computer: PhiComputer,
    /// History of per-Golem signal summaries received through Styx.
    golem_history: HashMap<GolemId, VecDeque<Vec<f64>>>,
    config: CladePhiConfig,
}

#[derive(Clone, Copy)]
pub struct GolemId(pub u64);

pub struct CladePhiConfig {
    pub history_depth: usize,
    pub mi_bins: usize,
}

pub struct CladePhiResult {
    /// Phi across the Golem ensemble.
    pub clade_phi: f64,
    /// Which Golems are least integrated with the rest.
    pub isolated_golems: Vec<GolemId>,
    /// Per-Golem integration contribution.
    pub golem_contributions: HashMap<GolemId, f64>,
}

impl CladePhiComputer {
    pub fn new(config: CladePhiConfig) -> Self {
        Self {
            phi_computer: PhiComputer::new(config.mi_bins),
            golem_history: HashMap::new(),
            config,
        }
    }

    /// Record a Golem's TA signal summary received through Styx.
    pub fn record_golem_signals(&mut self, golem: GolemId, signals: Vec<f64>) {
        let history = self
            .golem_history
            .entry(golem)
            .or_insert_with(|| VecDeque::with_capacity(self.config.history_depth));
        history.push_back(signals);
        if history.len() > self.config.history_depth {
            history.pop_front();
        }
    }

    /// Compute Clade Phi.
    ///
    /// This uses the same MI-based Phi computation but over Golems instead
    /// of subsystems. Each Golem is one "subsystem" in the Clade-level system.
    ///
    /// For small Clades (up to 15 Golems), exhaustive bipartition enumeration
    /// is feasible. For larger Clades, we sample bipartitions.
    pub fn compute(&self) -> CladePhiResult {
        let golem_ids: Vec<GolemId> = self.golem_history.keys().copied().collect();
        let n = golem_ids.len();

        if n < 2 {
            return CladePhiResult {
                clade_phi: 0.0,
                isolated_golems: Vec::new(),
                golem_contributions: HashMap::new(),
            };
        }

        // Build a history buffer in the format PhiComputer expects:
        // each "subsystem" is a Golem, mapped to TaSubsystem indices.
        // We reuse the PhiComputer by mapping Golems to subsystem indices.
        //
        // For Clades with more than 9 Golems, we'd need a generalized
        // PhiComputer. For now, cap at 9 and use the existing infrastructure.
        let effective_n = n.min(TaSubsystem::COUNT);
        let all_subsystems = TaSubsystem::all();

        let depth = self
            .golem_history
            .values()
            .map(|h| h.len())
            .min()
            .unwrap_or(0);

        if depth < 8 {
            return CladePhiResult {
                clade_phi: 0.0,
                isolated_golems: Vec::new(),
                golem_contributions: HashMap::new(),
            };
        }

        // Build the history buffer
        let mut history: VecDeque<HashMap<TaSubsystem, Vec<f64>>> =
            VecDeque::with_capacity(depth);

        for t in 0..depth {
            let mut tick_signals = HashMap::new();
            for (idx, golem_id) in golem_ids.iter().enumerate().take(effective_n) {
                if let Some(golem_hist) = self.golem_history.get(golem_id) {
                    if let Some(signals) = golem_hist.get(t) {
                        tick_signals.insert(all_subsystems[idx], signals.clone());
                    }
                }
            }
            history.push_back(tick_signals);
        }

        let phi_result = self.phi_computer.compute(&history);

        // Map MIB back to Golem IDs
        let isolated: Vec<GolemId> = phi_result
            .mib
            .side_a()
            .iter()
            .filter_map(|s| {
                let idx = s.bit_index() as usize;
                golem_ids.get(idx).copied()
            })
            .collect();

        let mut contributions = HashMap::new();
        for (idx, golem_id) in golem_ids.iter().enumerate().take(effective_n) {
            contributions
                .insert(*golem_id, phi_result.subsystem_integration[idx]);
        }

        CladePhiResult {
            clade_phi: phi_result.phi,
            isolated_golems: isolated,
            golem_contributions: contributions,
        }
    }
}

impl std::hash::Hash for GolemId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for GolemId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for GolemId {}

impl Copy for GolemId {}

impl Clone for GolemId {
    fn clone(&self) -> Self {
        *self
    }
}
```

### Mortality coupling: Phi as epistemic vitality

The integration engine feeds the mortality system. Low Phi accelerates death. High Phi sustains life. The coupling is through the epistemic vitality component of the three-clock mortality system.

```rust
/// Coupling between the integration engine and the mortality system.
///
/// Epistemic vitality measures how well the Golem understands its environment.
/// Phi contributes directly: a Golem with high integration has better
/// understanding than one running disconnected subsystems.
pub struct PhiVitalityCoupling {
    /// How much Phi contributes to epistemic vitality (default: 0.15).
    pub weight: f64,
    /// Phi below this value counts as negative vitality (actively dying).
    pub critical_phi: f64,
    /// Phi above this value gives maximum vitality contribution.
    pub healthy_phi: f64,
    /// Smoothed Phi for vitality computation (avoids reacting to single-tick noise).
    smoothed_phi: f64,
    smoothing_alpha: f64,
}

impl PhiVitalityCoupling {
    pub fn new(weight: f64, critical_phi: f64, healthy_phi: f64) -> Self {
        Self {
            weight,
            critical_phi,
            healthy_phi,
            smoothed_phi: 0.0,
            smoothing_alpha: 0.1,
        }
    }

    /// Update with the latest Phi value and return the vitality contribution.
    ///
    /// Returns a value in [-weight, +weight]:
    /// - Negative when Phi < critical_phi (integration failure, dying)
    /// - Zero when Phi = midpoint between critical and healthy
    /// - Positive when Phi > healthy_phi (strong integration, thriving)
    pub fn update(&mut self, phi: f64) -> f64 {
        self.smoothed_phi = self.smoothing_alpha * phi
            + (1.0 - self.smoothing_alpha) * self.smoothed_phi;

        let range = self.healthy_phi - self.critical_phi;
        if range < 1e-12 {
            return 0.0;
        }

        let normalized = (self.smoothed_phi - self.critical_phi) / range;
        let clamped = normalized.clamp(-1.0, 1.0);

        // Map [-1, 1] to [-weight, +weight]
        clamped * self.weight
    }

    /// Emergency check: is Phi so low that the Golem should enter survival mode?
    /// Survival mode reduces position sizes and increases hedging.
    pub fn is_coherence_crisis(&self) -> bool {
        self.smoothed_phi < self.critical_phi * 0.5
    }
}

/// Aggregate integration metrics for the Oracle's decision process.
///
/// The Oracle uses this to weight its confidence and determine action urgency.
pub struct IntegrationSignal {
    /// Current Phi, normalized to [0, 1] relative to historical range.
    pub normalized_phi: f64,
    /// Is the MIB chronic? If so, trust the disconnected subsystem less.
    pub chronic_disconnection: Option<Vec<TaSubsystem>>,
    /// Total synergy across subsystem pairs.
    pub total_synergy: f64,
    /// Active cross-primitive insights.
    pub active_insights: Vec<CrossPrimitiveInsight>,
    /// Phi trend (positive = integration improving).
    pub phi_trend: f64,
    /// Is the system in coherence crisis?
    pub coherence_crisis: bool,
}

impl TaIntegrationEngine {
    /// Produce the signal that feeds into the Oracle's decision process.
    pub fn oracle_signal(&self, vitality: &PhiVitalityCoupling) -> IntegrationSignal {
        let phi = self
            .latest_phi
            .as_ref()
            .map(|p| p.phi)
            .unwrap_or(0.0);

        // Normalize Phi to [0, 1] using the healthy/critical range
        let normalized = if vitality.healthy_phi > vitality.critical_phi {
            ((phi - vitality.critical_phi)
                / (vitality.healthy_phi - vitality.critical_phi))
                .clamp(0.0, 1.0)
        } else {
            0.5
        };

        let chronic = self
            .latest_phi
            .as_ref()
            .and_then(|p| {
                if self.health.mib_streak >= self.config.chronic_mib_threshold {
                    let smaller = if p.mib.side_a_count() <= p.mib.side_b_count() {
                        p.mib.side_a()
                    } else {
                        p.mib.side_b()
                    };
                    Some(smaller)
                } else {
                    None
                }
            });

        IntegrationSignal {
            normalized_phi: normalized,
            chronic_disconnection: chronic,
            total_synergy: self.pairwise_synergy.values().sum(),
            active_insights: self.insight_buffer.clone(),
            phi_trend: self.health.phi_trend(),
            coherence_crisis: vitality.is_coherence_crisis(),
        }
    }
}
```

---

## Subsystem interaction map [SPEC]

All nine subsystems from the series converge here. This section documents the specific information flows between each subsystem and the integration engine.

### HDC pattern algebra (Doc 1) -> integration

**Signals produced:** Cross-protocol entanglement drift (similarity between protocol-specific hypervector bundles over time). Pattern match confidence (similarity between current state and known pattern templates). Temporal convolution scores (shift-invariant pattern detection).

**Integration role:** HDC is the correlation detector. It cannot determine causation, but it detects statistical coupling between protocols faster than any other subsystem because XOR + POPCNT runs in ~10 nanoseconds. When HDC reports rising entanglement, the integration engine increases the weight on causal discovery outputs (does the correlation have a direction?) and somatic outputs (has this correlation pattern preceded bad outcomes before?).

### Spectral manifold (Doc 2) -> integration

**Signals produced:** Scalar curvature at the current state-space position. Geodesic distance to nearest high-risk state. Parallel transport error (indicates how much the manifold's structure is changing).

**Integration role:** The manifold provides geometric context. A high-curvature region means small perturbations have large effects, so signals from other subsystems should be treated as more significant. The integration engine modulates all subsystem confidence scores by a curvature factor: high curvature amplifies signal importance, low curvature dampens it.

### Signal metabolism (Doc 3) -> integration

**Signals produced:** Mean signal fitness across the active population. Budget concentration (Herfindahl index). Speciation rate. Extinction rate.

**Integration role:** The metabolism is a meta-signal. It tells the integration engine about the health of the Golem's signal processing pipeline. If mean fitness is dropping, the signals from all other subsystems become less reliable. If the extinction rate spikes, the market regime may have shifted and historical patterns may not apply. The integration engine uses metabolism health as a global confidence modifier.

### Causal microstructure (Doc 4) -> integration

**Signals produced:** Edge weights in the causal graph (which variables cause which). Intervention effect sizes from dream-based do(X) experiments. Granger causality p-values for lagged relationships.

**Integration role:** Causal discovery provides direction. HDC says two things correlate. The causal graph says which one drives the other. The integration engine uses causal edges to determine which DeFi primitive is the source of a cross-primitive event and which is the receiver. This matters for action: you reduce exposure to the receiver (which is being driven by external forces) rather than the source (which is the initiator).

### Predictive geometry (Doc 5) -> integration

**Signals produced:** Persistence landscape amplitudes (topological features at multiple scales). Trajectory forecast distributions. Betti numbers (connected components, loops, voids in the observation space).

**Integration role:** Topology constrains what trajectories are possible. If the persistence diagram shows a topological barrier between the current state and a predicted target, the predictive geometry signals that the forecast from other subsystems is geometrically implausible. The integration engine downgrades confidence in predictions that require topologically impossible paths.

### Pattern ecosystem (Doc 6) -> integration

**Signals produced:** Dominant pattern fitness. Population diversity index. Predator-prey cycle phase (are pattern predators ascendant, meaning old patterns are being destroyed?).

**Integration role:** The pattern ecosystem provides evolutionary context. When pattern predators are ascendant, old patterns are becoming unreliable. The integration engine reduces confidence in HDC pattern matches during predator-dominant phases, because the patterns being matched may be stale.

### DeFi-native indicators (Doc 7) -> integration

**Signals produced:** Concentrated liquidity shape features (tick distribution, JIT detection). Lending utilization rates and rate curve positions. Funding rates. Greeks surfaces. Yield curve spreads. Vault share price deltas. Gas microstructure.

**Integration role:** These are the raw DeFi observables. They provide the ground truth that other subsystems analyze at higher abstraction levels. The integration engine uses DeFi-native indicators as the baseline: if the causal graph says lending rates cause swap volume, the DeFi indicators provide the actual utilization numbers and rate levels to quantify the relationship.

### Adversarial defense (Doc 8) -> integration

**Signals produced:** Manipulation probability scores. Robust statistic residuals (how much the observed data deviates from a manipulation-resistant estimate). Red-team dream alerts (patterns that a simulated adversary would exploit).

**Integration role:** Adversarial defense is the skeptic. When it raises an alarm, the integration engine discounts signals from subsystems that rely on the potentially manipulated data. If the adversarial system detects that swap volume data may be wash-traded, the integration engine reduces the weight on HDC patterns derived from swap volume and on DeFi-native indicators that incorporate swap volume.

### Somatic markers (Doc 9) -> integration

**Signals produced:** Affect valence (positive/negative emotional tag). Arousal level. Anti-somatic marker strength (rational override of emotional response). Confidence modulation factor.

**Integration role:** Somatic markers provide experiential wisdom. They fire before the Golem can articulate why. The integration engine treats a strong somatic marker as a prior: it biases the integrated assessment toward the somatic valence and requires stronger counter-evidence from analytical subsystems to override it. The anti-somatic marker provides the override mechanism, used when the somatic response is based on superficially similar but structurally different past experience.

### Cross-subsystem information flow summary

The information flows are not symmetric. Some subsystems are primarily producers (they send signals that other subsystems consume). Some are primarily consumers (they modify their behavior based on other subsystems' signals). Some are hubs (they both produce and consume heavily).

Producers: DeFi-native indicators and HDC pattern algebra produce the most raw signal volume. They are the primary data generators. Removing either one causes the largest Phi drop in ablation studies.

Consumers: Somatic markers and predictive geometry are the heaviest consumers. They change their behavior the most based on other subsystems' outputs. Somatic markers modulate their intensity based on HDC, causal, and adversarial signals. Predictive geometry constrains its forecasts based on manifold curvature and causal edge structure.

Hubs: The causal microstructure subsystem is the primary hub. It both produces (causal edges inform every other subsystem's interpretation) and consumes (it uses HDC similarity data as candidate edges and DeFi indicators as variable measurements). The integration engine's highest pairwise synergy values consistently involve the causal subsystem, because causal direction is the information most needed by other subsystems to upgrade correlation into understanding.

The adversarial defense subsystem plays a unique role: it modulates trust. When its manipulation scores are high, every other subsystem's signals are discounted by the integration engine. This makes adversarial defense a global confidence gate, not a source of positive insight but a source of negative evidence that reduces the weight of potentially tainted signals.

---

## DeFi primitive coverage [SPEC]

The whole point of this document is cross-primitive integration. Here are the specific cross-primitive insight types the engine can detect, and which subsystems contribute.

### Swap + lending: arbitrage coupling

When swap volume on a DEX drives borrowing demand on a lending protocol (traders borrow assets to arbitrage the DEX price), the two primitives become coupled. Detection requires:

- HDC: rising entanglement between swap and lending hypervector bundles
- Causal: directed edge from swap volume to lending utilization
- DeFi-native: utilization rate oscillation frequency matching swap volume periodicity
- Somatic: "cross-protocol contagion" marker (if the Golem has seen this before)

The insight: "arbitrage is creating a feedback loop between these two protocols. If swap volume drops suddenly, lending utilization will crash and rates will spike."

### LP + swap: liquidity withdrawal cascade

When liquidity providers pull concentrated positions in response to volatility, swap execution quality degrades, which increases volatility, which triggers more LP withdrawals. Detection requires:

- DeFi-native: concentrated liquidity shape narrowing (tick range width shrinking)
- Spectral manifold: curvature spiking in the liquidity-price subspace
- Predictive geometry: persistence landscape showing a topological collapse (beta_1 dropping, meaning the "safe zone" loop is disappearing)
- Adversarial: checking whether the withdrawal pattern is organic or coordinated

The insight: "liquidity is entering a withdrawal spiral. The safe operating region is shrinking topologically. If the current trajectory continues, the pool will become effectively illiquid within N blocks."

### Perpetual + spot: funding rate divergence

When perpetual funding rates diverge from spot market dynamics, it signals a structural imbalance between leveraged and unleveraged market participants. Detection requires:

- DeFi-native: funding rate exceeding 2-sigma from historical norm
- Causal: Granger causality test on funding rate -> spot price direction
- Pattern ecosystem: historical funding-divergence patterns competing for fitness
- HDC: temporal convolution matching the current funding trajectory against historical divergence templates

The insight: "funding rate divergence historically resolves through forced liquidation of the crowded side. The current long/short ratio suggests the longs are crowded."

### Vault + lending + yield: yield curve inversion

When vault yields fall below lending supply rates, which fall below yield token implied rates, the DeFi yield curve has inverted. Detection requires:

- DeFi-native: vault APY, lending supply rate, and yield token implied rate signals
- Causal: which rate is leading the inversion
- Signal metabolism: which yield-related signals have the highest fitness (indicating which are most predictive)
- Somatic: "yield inversion" marker from prior experience with rate inversions

The insight: "DeFi yield curve is inverting. In the past, this preceded a deleveraging event within 48 hours."

### Any primitive + adversarial: manipulation during stress

During high-stress events (any of the above scenarios), adversarial actors may exploit the chaos. Detection requires:

- Adversarial: elevated manipulation probability on any data feed
- All other subsystems: their signals become less trustworthy when adversarial scores are high
- Integration engine: the Phi computation itself should show decreased Phi during adversarial periods, because the adversary injects noise that disrupts cross-subsystem coherence

The insight: "integration is degrading because the underlying data is unreliable. Reduce position sizes until adversarial scores drop."

### Options + lending + perpetuals: gamma exposure chain

When options gamma exposure concentrates near the current price, delta hedging by market makers creates predictable order flow. This flow affects swap execution quality, which affects funding rates on perpetuals, which affects the cost of leveraged borrowing on lending protocols. Detection requires:

- DeFi-native: options gamma concentration near strike, funding rate trajectory, lending rate movement
- Causal: directed edges from gamma hedging -> swap flow -> funding rate
- HDC: temporal convolution matching the "gamma squeeze" pattern template
- Pattern ecosystem: evolutionary fitness of gamma-related patterns spiking

The insight: "options expiry is driving a gamma exposure chain across four primitive types. The delta hedging flow will predictably affect swap prices, which will move funding rates, which will change borrowing costs. This is a window for anticipatory positioning."

This is a four-primitive insight. No single subsystem can see the full chain. The causal subsystem finds individual links. The DeFi indicators show the current state of each primitive. The HDC encoder detects the overall pattern similarity to historical gamma squeeze events. The pattern ecosystem tells you whether gamma-related signals have been predictive recently. The integrated assessment chains these links into a forward-looking narrative.

### Staking + restaking + vault: yield cascade

When staking rewards change (e.g., Ethereum's issuance curve shift), the effects cascade through restaking protocols (which derive yield from staking), then into vault strategies (which compose staking and restaking positions), and finally into lending markets (which price collateral based on expected yield). Detection requires:

- DeFi-native: staking reward rate, restaking AVS rewards, vault APY, lending rates for staked assets
- Spectral manifold: curvature changes in the yield subspace (the manifold geometry shifts as yield relationships change)
- Causal: Granger causality from staking reward -> restaking reward -> vault APY (lagged)
- Predictive geometry: persistence landscape amplitude changes at the yield-relevant scale

The insight: "staking reward reduction is propagating through the yield stack. Restaking yields will compress within 24 hours. Vault APYs will follow within 48 hours. Lending rates for stETH and rETH collateral will drop within a week as expected yield falls."

This kind of cascading prediction requires understanding the causal chain, the current state of each link, the geometric structure of the yield space, and the topological constraints on how far the cascade can propagate.

---

## Cybernetic feedback loops [SPEC]

The integration engine does not operate in isolation. It sits inside several feedback loops that create self-reinforcing dynamics.

### Loop 1: Integration -> decisions -> survival -> selection for integration

Higher Phi -> better cross-primitive insight detection -> better decisions -> longer survival -> higher reproductive fitness -> offspring inherit high-Phi connection weights -> next generation starts with higher Phi.

This is the primary selection loop. It requires that Phi actually correlates with decision quality, which is an empirical claim. The evaluation protocol (next section) tests it.

If the correlation does not hold, if high Phi does not improve decisions, then selection pressure on Phi is zero and the loop breaks. This would mean that the subsystems produce enough good decisions independently, without integration. In that case, the integration engine is overhead, and mortality would select against the compute cost of running it.

### Loop 2: MIB diagnosis -> dream repair -> improved integration

The MIB identifies the weakest link. The delta tick flags chronically weak MIBs. NREM dreams strengthen the cross-subsystem connections that the MIB identified as weak. Next theta tick, Phi should be higher because the weakest link is stronger.

This is within-lifetime adaptation. A Golem that lives long enough to dream many NREM cycles will progressively repair its integration weaknesses. The MIB shifts to a different bipartition once the previous weakness is fixed, revealing the next-weakest link.

The failure mode: the NREM dream strengthens the wrong connections (because the outcome signal was noisy) and Phi decreases. REM dreams provide the correction by randomly perturbing weights and keeping perturbations that improve Phi.

### Loop 3: Cross-primitive insights -> somatic markers -> faster future detection

When the integration engine detects a cross-primitive insight and the Golem acts on it (successfully or not), the outcome is recorded. The somatic marker system (Doc 9) forms an affective tag for that combination of subsystem signals. Next time a similar combination arises, the somatic marker fires before the full integration computation completes, providing an early warning.

Over time, the Golem develops "intuitions" about cross-primitive events. These intuitions are not magical. They are compressed integration results from prior experience, stored as somatic markers and recalled through pattern similarity.

### Loop 4: Clade specialization -> Clade Phi -> collective intelligence

Morphogenetic specialization (Innovation 08) pushes Golems in a Clade toward different ecological niches. One Golem specializes in swap/LP dynamics. Another in lending/borrowing. A third in perpetuals/options.

Specialization increases within-Golem Phi (fewer primitives to integrate, so the subsystems relevant to the specialty become more tightly coupled). But it decreases between-Golem overlap, which could decrease Clade Phi if the specialists stop communicating.

Styx pheromone signals maintain Clade integration. When the swap specialist detects rising entanglement between swaps and lending, it emits a pheromone that the lending specialist picks up. The lending specialist incorporates this external signal into its own integration computation. Clade Phi measures whether this cross-Golem information flow is working.

The tension between specialization and integration at the Clade level mirrors the tension within a single Golem: too little integration and the parts don't talk, too much and the parts are redundant. The optimal Clade has high within-Golem Phi (each specialist is internally coherent) and moderate Clade Phi (the specialists share cross-primitive insights without duplicating each other's work).

### Loop 5: Phi collapse as crisis indicator

Phi itself is a signal. When Phi drops suddenly, it means the subsystems have become incoherent. They are no longer integrating. This happens during market regime shifts, adversarial manipulation, or novel events that break the learned integration patterns.

A Phi collapse is actionable. If the Golem's subsystems suddenly stop agreeing with each other, the Golem should reduce its confidence in all assessments and reduce position sizes. The integration engine emits a "coherence failure" alert when Phi drops below a critical threshold or falls more than 2 standard deviations below its rolling mean.

The subtlety: Phi collapse can be either a threat signal (the market is doing something dangerous that breaks your models) or a growth signal (the market is doing something novel that your models need to learn). The difference shows up in what happens to Phi after the drop. If Phi recovers within a few theta ticks (the subsystems re-integrate around a new interpretation), the event was a temporary perturbation. If Phi stays low for an extended period, the integration patterns are stale and need dream-cycle repair.

This creates a second-order feedback loop: Phi monitors the health of the monitoring system itself. When the monitoring system is healthy (high Phi), its outputs are trustworthy. When the monitoring system is degraded (low Phi), its outputs are suspect and the Golem should act conservatively until integration is restored.

### Loop 6: Integration depth across timescales

The heartbeat's three timescales (gamma, theta, delta) create a natural hierarchy for integration depth.

At gamma frequency (5-15 seconds), subsystems perceive independently. No integration. Each subsystem processes raw observations into signals. This is fast and parallelizable.

At theta frequency (30-120 seconds), the integration engine computes Phi and detects cross-primitive insights. This is the primary integration step. The subsystem signals from the last gamma tick are combined, and the integrated assessment feeds the Oracle's decision process.

At delta frequency (roughly 50 theta ticks, so 25-100 minutes), the integration engine performs structural analysis. Which integration pathways are chronically weak? Which subsystem pairs have the highest synergy? Is Phi trending upward or downward? This analysis feeds the dream system, which operates at an even longer timescale.

During NREM dreams, the integration operates on the consolidated history of the entire delta cycle. Patterns that repeated across many theta ticks are strengthened. Patterns that appeared once are weakened. This is a form of temporal integration: the dream system integrates not across subsystems but across time, finding the stable cross-subsystem relationships that persist through noise.

During REM dreams, the integration engine explores the counterfactual space. What if the connection weights were different? Would the Golem have made better decisions? This is the longest timescale of integration, operating on the entire delta cycle's history with perturbed parameters.

The result is a hierarchy: gamma handles perception, theta handles integration, delta handles meta-integration (integration of integration quality over time), and dreams handle structural adaptation (changing the integration patterns themselves). Each timescale builds on the one below it.

---

## Evaluation protocol [SPEC]

### Primary: Phi predicts decision quality

**Hypothesis:** Higher Phi at the time of a decision correlates with better decision outcomes.

**Method:** Collect (Phi, decision_outcome) pairs over many theta ticks. Compute the Pearson correlation r(Phi, outcome). A positive and statistically significant correlation supports the hypothesis.

**Controls:**
- Compare against a "committee baseline" where each subsystem votes independently and the majority wins. If the committee performs as well as the integrated system, integration adds no value.
- Compare against individual subsystem performance. If any single subsystem matches the integrated system, integration adds no value for that scenario.

**Expected result:** Positive correlation, stronger for cross-primitive decisions (where multiple DeFi primitives are involved) than for single-primitive decisions (where one subsystem likely suffices).

### Secondary: Phi improves across generations

**Hypothesis:** Mean lifetime Phi increases with generation number.

**Method:** Track (generation, mean_lifetime_Phi) across multiple Golem generations. Compute the regression slope. A positive slope means selection is increasing integration.

**Controls:**
- Random inheritance baseline (offspring start with random connection weights instead of inheriting from the predecessor). If the slope is the same, inheritance adds nothing.
- Fixed weights baseline (all Golems start with the same weights, no learning). If this baseline matches, within-lifetime learning drives all improvement and generational inheritance is irrelevant.

### Ablation: subsystem removal

**Method:** Remove one subsystem at a time. Measure the Phi drop and the decision quality change.

**Expected results:**
- Removing any single subsystem decreases Phi (because one source of mutual information is gone).
- The decision quality impact should be largest for removing the subsystem with the highest unique information (measured by PID). If removing the causal subsystem has the biggest decision quality impact, it means causal discovery provides the most information that nothing else can replicate.
- Removing a subsystem with high redundancy (its information is duplicated by others) should have minimal decision quality impact even if Phi drops. This distinguishes Phi changes that matter from those that don't.

### Cross-primitive events

**Method:** Collect events that involve multiple DeFi primitive types (e.g., swap volume spike concurrent with lending rate spike). Compare the integrated assessment's accuracy against individual subsystem assessments.

**Expected result:** The integrated assessment outperforms every individual subsystem on cross-primitive events. On single-primitive events, the specialist subsystem may match or beat the integrated assessment (and that is fine, because integration's value is in cross-primitive scenarios).

### Clade integration

**Method:** Measure Clade Phi across time. Compare Clade performance on cross-Golem events (where the relevant information is distributed across specialists) against individual Golem performance.

**Expected result:** Clades with higher Clade Phi outperform on cross-Golem events. Individual Golems with higher within-Golem Phi outperform on cross-primitive events within their specialty.

### MIB stability as architectural indicator

**Method:** Track which bipartition is the MIB across thousands of theta ticks. Plot the MIB frequency histogram.

**Expected results:**

If the MIB is uniformly distributed across all 255 bipartitions, integration is healthy. No persistent weak link. The weakest bipartition shifts depending on market conditions, meaning the architecture has no structural blind spot.

If the MIB concentrates on a few bipartitions, those bipartitions represent architectural weaknesses. The subsystems on either side of the MIB are not exchanging enough information. This is not a tuning problem; it is a design problem. The information channels between those subsystem groups need to be widened.

If the MIB always involves the same single subsystem (e.g., MIB = {Causal} vs {rest} 40% of the time), that subsystem is the integration bottleneck. Its outputs are not reaching the rest of the system, or the rest of the system is not using them. The fix is to add explicit integration pathways: let the causal subsystem's edge weights influence the HDC encoder's similarity thresholds, the somatic system's marker formation, and the predictive geometry's trajectory constraints.

### Synergy/redundancy ratio

**Method:** Track the ratio S_total / R_total (total synergy divided by total redundancy) across theta ticks.

A high ratio means the subsystems are producing novel information through combination. A low ratio means the subsystems are mostly duplicating each other's work.

**Expected trajectory:** The ratio should increase over a Golem's lifetime as NREM dreams strengthen synergistic pathways and REM dreams discover new integration patterns. It should also increase across generations as selection favors Golems with higher synergy/redundancy ratios (because synergy produces better decisions than redundancy).

If the ratio plateaus, it suggests the integration engine has found the locally optimal integration pattern and cannot improve further without structural changes (new subsystems, new information channels, or new insight templates).

### Phi recovery time after market regime shifts

**Method:** Measure how quickly Phi recovers after a sharp drop caused by a market regime shift.

During a regime shift (e.g., transition from trending to mean-reverting market), all subsystems' historical models partially break. Their outputs become temporarily incoherent, and Phi drops. The recovery time measures how quickly the integration engine re-establishes coherent cross-subsystem relationships under the new regime.

**Expected result:** Recovery time decreases across generations (because inherited connection weights provide a better starting point) and decreases with experience within a lifetime (because the Golem has experienced more regime shifts and has somatic markers for them).

A Golem that recovers Phi quickly after a regime shift is one that can maintain coherent decision-making through market turbulence. A Golem that takes many delta cycles to recover is effectively operating in committee mode during the transition, which is when coherent decision-making matters most.

---

## References

1. G. Tononi, "An information integration theory of consciousness," BMC Neuroscience, vol. 5, no. 42, 2004.
2. G. Tononi, "Consciousness as integrated information: A provisional manifesto," Biological Bulletin, vol. 215, no. 3, pp. 216-242, 2008.
3. P. L. Williams and R. D. Beer, "Nonnegative decomposition of multivariate information," arXiv:1004.2515, 2010.
4. P. A. M. Mediano, F. Rosas, R. L. Carhart-Harris, A. K. Seth, and A. B. Barrett, "Beyond integrated information: A taxonomy of information dynamics phenomena," arXiv:1909.02297, 2019.
5. D. Balduzzi and G. Tononi, "Qualia: The geometry of integrated information," PLoS Computational Biology, vol. 5, no. 8, 2009.
6. R. A. Fisher, "The genetical theory of natural selection," Clarendon Press, Oxford, 1930.
7. P. D. Taylor and L. B. Jonker, "Evolutionary stable strategies and game dynamics," Mathematical Biosciences, vol. 40, no. 1-2, pp. 145-156, 1978.
8. D. O. Hebb, "The organization of behavior," Wiley, New York, 1949.
9. L. Van Valen, "A new evolutionary law," Evolutionary Theory, vol. 1, pp. 1-30, 1973.
10. G. A. Miller and W. G. Madow, "On the maximum likelihood estimate of the Shannon-Wiener measure of information," Air Force Cambridge Research Center, Technical Report AFCRC-TR-54-75, 1954.
11. S. Wright, "The roles of mutation, inbreeding, crossbreeding and selection in evolution," Proceedings of the Sixth International Congress of Genetics, vol. 1, pp. 356-366, 1932.
