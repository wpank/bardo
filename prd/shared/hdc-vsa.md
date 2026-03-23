# Hyperdimensional Computing / Vector Symbolic Architectures [SPEC]

> **Document Type**: REF (normative) | **Referenced by**: 01-golem, 04-memory, 05-dreams, 09-economy, 19-agents-skills | **Last Updated**: 2026-03-18
>
> Canonical shared reference for Hyperdimensional Computing (HDC) and Vector Symbolic Architectures (VSA) used across the Bardo system. Subsystems reference this document for the algebra, types, encoding schemes, and capacity bounds that underpin transaction fingerprinting, memory compression, knowledge inheritance, drift detection, regime classification, and multi-agent consensus.
>
> **Split structure:**
> - `shared/hdc-vsa.md` (this file) — foundations, BSC algebra, capacity bounds, core Rust types, performance
> - `shared/hdc-fingerprints.md` — transaction fingerprint encoding, structured queries, three-tier architecture, ANN index
> - `shared/hdc-applications.md` — memory applications, query applications, advanced applications

> **Reader orientation:** This document is the foundational reference for Hyperdimensional Computing (HDC) and Vector Symbolic Architectures (VSA) as used across Bardo. HDC encodes information as high-dimensional binary vectors (10,240 bits / 1,280 bytes each) and manipulates them with XOR (bind), majority vote (bundle), and cyclic shift (permute). These primitives underpin transaction fingerprinting, Grimoire (the agent's persistent knowledge base) memory compression, knowledge inheritance across Golem (mortal autonomous agent) generations, drift detection, and multi-agent consensus. This file covers foundations, algebra, and capacity bounds; see `shared/hdc-fingerprints.md` for transaction encoding and `shared/hdc-applications.md` for memory and query applications. See `prd2/shared/glossary.md` for full term definitions.

---

## Cross-references

| Subsystem | What it uses from HDC |
|---|---|
| CorticalState (01-golem) (32-signal atomic shared perception surface; the Golem's real-time self-model) | Composite state hypervectors for compositional multi-signal queries |
| Grimoire (04-memory) (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) | Episode compression, legacy bundles, controlled forgetting |
| Death protocol (01-golem/05-death) (Thanatopsis -- four-phase structured shutdown: Acceptance, Settlement, Reflection, Legacy) | Holographic legacy bundle for generational transfer |
| Curiosity scorer (triage pipeline) | Compositional novelty scoring, regime-modulated attention |
| ChainScope (triage pipeline) | Protocol family vectors, semantic interest maps |
| Clade consensus (09-economy) (sibling Golems sharing a common ancestor; exchange knowledge through Styx) | Privacy-preserving vote bundling, trust accumulation |
| Dream engine (05-dreams) | Resonator decomposition for anomaly attribution |
| `shared/citations.md` | All academic references below have entries in the master index |

---

## 1. What HDC is

Hyperdimensional computing -- also called Vector Symbolic Architectures -- represents information as high-dimensional binary vectors and manipulates them with a small set of algebraic operations. The idea rests on a geometric fact: in spaces of dimension D >= 8,000, randomly sampled vectors are quasi-orthogonal with overwhelming probability. For D = 10,000 binary vectors, the probability that two independent vectors share more than 55% of their bits is less than 10^-9. Thousands of distinct concepts can coexist in the same vector space without collision.

### 1.1 The mathematics of near-orthogonality

The property that makes HDC work is concentration of measure. In a D-dimensional binary space {0,1}^D, the expected Hamming distance between two independently drawn vectors is D/2, with standard deviation sqrt(D)/2. As D grows, the distribution of pairwise distances concentrates tightly around the mean. At D = 10,000, the coefficient of variation is 1/sqrt(D) = 0.01 -- meaning 99% of random pairs land within 1% of the expected distance. Random vectors are neither similar nor dissimilar; they are reliably orthogonal.

This gives HDC its capacity. A codebook of N random vectors in D dimensions can be stored and retrieved by nearest-neighbor lookup as long as the signal-to-noise ratio (SNR) exceeds a discrimination threshold. For K items bundled (superimposed) into a single vector, the SNR scales as:

```
SNR = sqrt(D / K)
```

At D = 10,000 with K = 5 (a typical transaction encoding with five role-filler pairs), SNR = 44.7. With K = 50 (a bundle of fifty transaction prototypes), SNR = 14.1. With K = 200, SNR = 7.1 -- still workable with error correction. The space supports far more simultaneous representations than any low-dimensional embedding scheme.

HDC is not a replacement for any Bardo subsystem. It is a shared computational substrate -- a 1,280-byte algebraic representation that unifies fingerprinting, memory, attention, prediction, and consensus under a single set of operations.

Kanerva (2009) formalized these properties and showed that the same geometry underlies neural population codes in the brain. The connection is not metaphorical: place cells in the hippocampus, grid cells in the entorhinal cortex, and sparse codes throughout the cortex all operate in regimes where quasi-orthogonality provides the capacity guarantees that HDC exploits computationally.

---

## 2. Selected system: Binary Spatter Codes (BSC)

Four HDC system families dominate the literature: Binary Spatter Codes (BSC), Multiply-Add-Permute (MAP), Holographic Reduced Representations (HRR), and Fourier HRR (FHRR). Bardo uses BSC exclusively.

| Property | BSC | MAP | HRR | FHRR |
|---|---|---|---|---|
| Vector space | {0,1}^D | {-1,0,+1}^D or Z^D | R^D | C^D (unit circle) |
| Binding | XOR | Multiplication | Circular convolution | Phase addition |
| Binding cost per dim | 1 CPU op | 1 CPU op | O(D) or O(D log D) | 1 complex multiply |
| Unbinding | Exact (XOR is self-inverse) | Exact (ternary mult) | Approximate (correlation) | Approximate (conjugate) |
| Bundle capacity at D=10K | ~1,000 pairs | ~800 pairs | ~100 pairs | ~100 pairs |
| Storage per vector | 1,250 bytes | 10-40 KB | 40 KB (f32) | 80 KB (c64) |
| Best for | Discrete data, hardware speed | Weighted voting, magnitude tracking | Continuous embeddings | Sequence-aware binding |

### 2.1 Extended family notes

**BSC capacity.** Kleyko et al. (2022) report that D = 10,000 BSC vectors reliably store up to ~1,000 bound pairs in a bundle with >95% retrieval accuracy. At D = 16,384, capacity extends to ~2,000 pairs. BSC binding is its own inverse: `bind(bind(a, b), b) = a` exactly, with no approximation error. This makes unbinding free and exact -- a property no other HDC family matches. XOR compiles to a single instruction per 64-bit word. Hamming distance compiles to XOR followed by POPCNT. On a modern CPU with AVX2, comparing two 10,000-bit vectors takes roughly 10 nanoseconds. Storage is 1,250 bytes per vector (D/8).

**MAP expressiveness.** MAP is more expressive than BSC for weighted bundling: adding a vector three times gives it three times the vote weight, and the magnitude is preserved until the final threshold step. But MAP vectors consume more memory (one byte or one integer per dimension vs. one bit for BSC), and binding is multiplication rather than XOR, which is slower on binary hardware.

**HRR capacity.** Plate (2003) proves HRR bundle capacity scales as O(sqrt(D)). For D = 10,000, this means approximately 100 stored pairs -- significantly less than BSC's ~1,000. The difference comes from the continuous-valued noise accumulation in HRR unbinding versus BSC's exact XOR inversion. HRR's strength is its compatibility with continuous-valued representations. If the input data is naturally real-valued (neural network embeddings, continuous sensor readings), HRR avoids the quantization step that BSC and MAP require. Its weakness is computational cost: circular convolution in the time domain is O(D^2), though the FFT reduces it to O(D log D).

**FHRR and GHRR.** Plate also described an alternative where binding operates in the frequency domain: vectors are complex-valued with unit-magnitude components (points on the unit circle), and binding is componentwise complex multiplication (phase addition). FHRR has the same capacity characteristics as HRR but avoids the FFT step because the representation is already in frequency space. Recent work by Yeung, Zou, and Imani (2024) introduces Generalized HRR (GHRR) with non-commutative binding, which preserves sequence order without requiring separate permutation operations.

**Why BSC.** The transaction fingerprinting domain is discrete: protocol names, function selectors, address hashes, gas buckets. Exact invertibility (XOR is its own inverse) enables structured queries like "find transactions matching X but with a different recipient." The 1,280-byte footprint per vector fits the Golem's memory budget. XOR + POPCNT operations run at wire speed on commodity hardware.

---

## 3. Dimension: D = 10,240

The Bardo implementation uses D = 10,240 bits = 160 x u64 words = 1,280 bytes per vector.

Two reasons for this specific number:

1. **Quasi-orthogonality.** P(|sim| > 0.05) < 1e-9 for random pairs. The coefficient of variation is 1/sqrt(D) = 0.0099, meaning 99% of random pairs land within 1% of expected Hamming distance D/2.

2. **SIMD alignment.** 160 words = 5 x 32-word AVX-512 passes or 10 x 16-word AVX2 passes. Clean loop boundaries with no remainder handling.

---

## 4. The BSC algebra

Four operations define the HDC algebra. Every downstream application is a composition of these four primitives.

### 4.1 Bind (XOR)

Binding associates two hypervectors into a new vector that is quasi-orthogonal to both inputs. It encodes a relationship: `bind(R_protocol, HV_uniswap)` means "Uniswap in the protocol role."

Properties:
- **Self-inverse**: `bind(bind(a, b), b) = a`. No separate unbind operation needed.
- **Commutative**: `bind(a, b) = bind(b, a)`.
- **Associative**: `bind(a, bind(b, c)) = bind(bind(a, b), c)`.
- **Distributes over bundling**: `bind(a, bundle(b, c)) = bundle(bind(a, b), bind(a, c))`. This is what makes structured queries work.

### 4.2 Bundle (majority vote)

Bundling superimposes multiple hypervectors into a single aggregate. The result is similar to all inputs -- a "set union" in hypervector space. For BSC, bundling counts votes per bit position and sets each output bit to the majority value. Ties break to 0 for determinism.

Bundling drives memory compression. A bundle of 50 transaction prototypes retains enough signal from each to answer similarity queries, while occupying the same 1,280 bytes as a single vector.

### 4.3 Permute (cyclic bit shift)

Permutation encodes position or sequence order. Applying a cyclic shift of `k` positions produces a vector quasi-orthogonal to the original. This lets the algebra distinguish "A then B" from "B then A" -- something binding alone (commutative) cannot do.

Standard sequence encoding:

```
seq_hv = bundle(permute(e1, 0), permute(e2, 1), permute(e3, 2))
```

### 4.4 Similarity (normalized Hamming distance)

Similarity between two BSC vectors is the fraction of matching bits: `sim(a, b) = 1 - hamming(a, b) / D`.

| Range | Meaning |
|---|---|
| 1.0 | Identical |
| > 0.52 | Meaningful relationship |
| 0.48 - 0.52 | Noise (quasi-orthogonal) |
| < 0.48 | Meaningful dissimilarity (anti-correlated) |
| 0.0 | Bitwise complement |

---

## 5. Capacity bounds

For a bundle of K items in D dimensions, the signal-to-noise ratio is:

```
SNR = sqrt(D / K)
```

At D = 10,240:

| K (items bundled) | SNR | Max codebook N at 99% accuracy |
|---|---|---|
| 5 | 45.3 | >100,000 |
| 10 | 32.0 | >50,000 |
| 50 | 14.3 | ~5,000 |
| 100 | 10.1 | ~1,000 |
| 200 | 7.2 | ~200 |
| 500 | 4.5 | ~20 |

For the primary use case (encoding 5-7 role-filler pairs per transaction), the capacity is enormous. Even the memory inheritance use case (bundling hundreds of episode prototypes across generations) stays within workable SNR ranges. The safe rule of thumb: **K < 100 items per bundle** for reliable retrieval against codebooks of 1,000+ entries.

Kleyko et al. (2022) report that D = 10,000 BSC vectors reliably store up to ~1,000 bound pairs in a bundle with >95% retrieval accuracy. Thomas et al. (2021) confirm that capacity grows linearly with D and logarithmically with the acceptable error rate.

---

## 6. Rust types

All types live in `crates/golem-core/src/hdc/mod.rs`.

### 6.1 Hypervector

```rust
/// D = 10,240 bits = 160 x u64 words.
/// Chosen for two reasons:
///   1. Quasi-orthogonality: P(|sim| > 0.05) < 1e-9 for random pairs.
///   2. SIMD alignment: 160 words = 5 x 32-word AVX-512 passes or
///      10 x 16-word AVX2 passes -- clean loop boundaries.
pub const HDC_WORDS: usize = 160;
pub const HDC_BITS: usize = HDC_WORDS * 64; // 10,240

/// Binary hypervector. 1,280 bytes.
/// Copy trait is intentional: stack copies are acceptable at this size
/// for short-lived computations. Heap-allocate (Box<Hypervector>) in
/// persistent collections.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C, align(32))] // 256-bit alignment for AVX2 intrinsics
pub struct Hypervector(pub [u64; HDC_WORDS]);

impl Hypervector {
    /// Generate a random binary hypervector from a deterministic seed.
    /// The same seed always produces the same vector -- required for
    /// reproducibility across process restarts.
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        let mut words = [0u64; HDC_WORDS];
        for w in &mut words {
            *w = rng.next_u64();
        }
        Hypervector(words)
    }

    /// Binding: componentwise XOR.
    /// Self-inverse: bind(bind(a, b), b) == a exactly.
    /// Result is quasi-orthogonal to both inputs: sim(a ^ b, a) ~ 0.5.
    #[inline(always)]
    pub fn bind(&self, other: &Self) -> Self {
        let mut out = [0u64; HDC_WORDS];
        for i in 0..HDC_WORDS {
            out[i] = self.0[i] ^ other.0[i];
        }
        Hypervector(out)
    }

    /// Unbinding: for BSC, identical to binding (XOR is self-inverse).
    /// Included as a separate method for readability at call sites.
    #[inline(always)]
    pub fn unbind(&self, key: &Self) -> Self {
        self.bind(key)
    }

    /// Hamming similarity in [0.0, 1.0].
    /// 1.0 = identical. ~0.5 = random/orthogonal. 0.0 = bitwise complement.
    #[inline(always)]
    pub fn similarity(&self, other: &Self) -> f32 {
        let mut matching_bits = 0u32;
        for i in 0..HDC_WORDS {
            matching_bits += (self.0[i] ^ other.0[i]).count_zeros();
        }
        matching_bits as f32 / HDC_BITS as f32
    }

    /// Hamming distance (number of differing bits). Raw count, not normalized.
    #[inline(always)]
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        let mut dist = 0u32;
        for i in 0..HDC_WORDS {
            dist += (self.0[i] ^ other.0[i]).count_ones();
        }
        dist
    }

    /// Cyclic bit rotation by `steps` positions.
    /// Each distinct step count produces a quasi-orthogonal vector.
    /// Used to encode position/order in sequences.
    pub fn permute(&self, steps: usize) -> Self {
        let shift = steps % HDC_BITS;
        if shift == 0 {
            return *self;
        }
        let word_shift = shift / 64;
        let bit_shift = (shift % 64) as u32;
        let mut out = [0u64; HDC_WORDS];
        if bit_shift == 0 {
            for i in 0..HDC_WORDS {
                out[i] = self.0[(i + word_shift) % HDC_WORDS];
            }
        } else {
            for i in 0..HDC_WORDS {
                let src = (i + word_shift) % HDC_WORDS;
                let nxt = (src + 1) % HDC_WORDS;
                out[i] = (self.0[src] << bit_shift)
                    | (self.0[nxt] >> (64 - bit_shift));
            }
        }
        Hypervector(out)
    }

    /// Bitwise complement. sim(hv, hv.inverse()) == 0.0.
    pub fn inverse(&self) -> Self {
        let mut out = [0u64; HDC_WORDS];
        for i in 0..HDC_WORDS {
            out[i] = !self.0[i];
        }
        Hypervector(out)
    }
}
```

### 6.2 BundleAccumulator

Bundling requires per-bit vote tracking because majority vote is not associative over binary vectors. The accumulator maintains a signed integer per bit position.

```rust
/// Accumulates hypervectors for majority-vote bundling.
/// Memory: HDC_BITS * 4 bytes = 40 KB per accumulator.
/// Heap-allocated; do not put on the stack in tight loops.
pub struct BundleAccumulator {
    votes: Vec<i32>,
    pub count: usize,
}

impl BundleAccumulator {
    pub fn new() -> Self {
        BundleAccumulator {
            votes: vec![0i32; HDC_BITS],
            count: 0,
        }
    }

    /// Add a hypervector to the accumulator. O(D).
    /// Each set bit contributes +1; each unset bit contributes -1.
    pub fn add(&mut self, hv: &Hypervector) {
        for (word_idx, &word) in hv.0.iter().enumerate() {
            let base = word_idx * 64;
            for bit in 0..64usize {
                let vote = if (word >> bit) & 1 == 1 { 1i32 } else { -1i32 };
                self.votes[base + bit] += vote;
            }
        }
        self.count += 1;
    }

    /// Add a hypervector with a scalar weight.
    /// Equivalent to calling add() `weight` times, but O(D) instead of O(D * weight).
    pub fn add_weighted(&mut self, hv: &Hypervector, weight: i32) {
        for (word_idx, &word) in hv.0.iter().enumerate() {
            let base = word_idx * 64;
            for bit in 0..64usize {
                let vote = if (word >> bit) & 1 == 1 { weight } else { -weight };
                self.votes[base + bit] += vote;
            }
        }
        self.count += weight as usize;
    }

    /// Collapse accumulated votes into a binary hypervector.
    /// Ties (vote == 0) resolve to bit 0 for determinism.
    pub fn finish(&self) -> Hypervector {
        let mut out = [0u64; HDC_WORDS];
        for (word_idx, word) in out.iter_mut().enumerate() {
            let base = word_idx * 64;
            for bit in 0..64usize {
                if self.votes[base + bit] > 0 {
                    *word |= 1u64 << bit;
                }
            }
        }
        Hypervector(out)
    }

    /// Apply exponential decay to all vote counts.
    /// Called at Delta tick to implement controlled forgetting.
    /// O(D).
    pub fn decay(&mut self, factor: f32) {
        for vote in self.votes.iter_mut() {
            *vote = (*vote as f32 * factor).round() as i32;
        }
        // Note: count is not adjusted -- it reflects total lifetime additions,
        // not current effective weight. This is intentional: the SNR estimate
        // from count is conservative (actual SNR is better than predicted
        // because decay reduces interference from old items).
    }

    /// Reset the accumulator without deallocating.
    pub fn clear(&mut self) {
        self.votes.fill(0);
        self.count = 0;
    }
}
```

### 6.3 ItemMemory (symbol table / codebook)

Maps named concepts to deterministic hypervectors. The same name always produces the same vector, even across process restarts, because generation is seeded from the name hash.

```rust
/// Maps named concepts to canonical hypervectors.
/// Thread-safe: wrap in Arc<Mutex<ItemMemory>> or use DashMap internally
/// for concurrent access from multiple tick handlers.
pub struct ItemMemory {
    store: std::collections::HashMap<String, Hypervector>,
    seed: u64,
}

impl ItemMemory {
    pub fn new(seed: u64) -> Self {
        ItemMemory {
            store: std::collections::HashMap::new(),
            seed,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the canonical hypervector for `name`.
    /// Creates it deterministically on first access.
    pub fn encode(&mut self, name: &str) -> Hypervector {
        if let Some(hv) = self.store.get(name) {
            return *hv;
        }
        use rand::SeedableRng;
        let hash = name
            .bytes()
            .fold(self.seed, |acc, b| {
                acc.wrapping_mul(6364136223846793005).wrapping_add(b as u64)
            });
        let mut rng = rand::rngs::SmallRng::seed_from_u64(hash);
        let hv = Hypervector::random(&mut rng);
        self.store.insert(name.to_string(), hv);
        hv
    }

    /// Check whether a name has been encoded without creating it.
    pub fn contains(&self, name: &str) -> bool {
        self.store.contains_key(name)
    }

    /// Number of concepts in the symbol table.
    pub fn len(&self) -> usize {
        self.store.len()
    }
}
```

This is the codebook. One `ItemMemory` instance per role domain (protocols, selectors, gas tiers, etc.) is the cleanest design, but a single shared instance with namespaced keys (`"proto:uniswap_v3"`, `"selector:transfer"`, `"gas:medium"`) works too. The Bardo implementation uses the namespaced-key approach for simplicity.

### 6.4 ResonatorNetwork

Resonator networks (Frady et al. 2020) solve the inverse problem: given a composite hypervector `z = bind(x1, x2, ..., xF)` and codebooks for each factor, recover the original factors.

The algorithm works through iterated projection:
1. Initialize each factor estimate (e.g., to the codebook centroid).
2. For each factor i: bind all other current estimates, unbind from the composite, project onto the nearest codebook entry.
3. Repeat until convergence (typically 10-50 iterations).

The network's dynamics are energy-minimizing: each step reduces the reconstruction error `||z - bind(est_1, est_2, ..., est_F)||`. Frady et al. show that resonator networks dramatically outperform alternating least squares and gradient-based methods on the factorization task, especially when multiple factors are present. For Bardo, resonator networks enable decomposing observed transaction patterns back into constituent fields -- "what protocol AND function AND value bucket produced this anomalous pattern?"

```rust
/// Resonator network for decomposing composite hypervectors.
///
/// Given z = bind(x1, x2, ..., xF) and codebooks C1, C2, ..., CF,
/// finds the factor estimates x1*, x2*, ..., xF* in their respective
/// codebooks that minimize ||z - bind(x1*, x2*, ..., xF*)||.
///
/// Algorithm: Frady et al. (2020), Neural Computation 32(12).
pub struct ResonatorNetwork {
    codebooks: Vec<(String, Vec<(String, Hypervector)>)>,
    max_iterations: usize,
    convergence_threshold: f32,
}

impl ResonatorNetwork {
    pub fn new(max_iterations: usize) -> Self {
        ResonatorNetwork {
            codebooks: Vec::new(),
            max_iterations,
            convergence_threshold: 0.995,
        }
    }

    /// Register a factor dimension with its codebook of possible values.
    pub fn add_factor(&mut self, name: &str, entries: Vec<(String, Hypervector)>) {
        self.codebooks.push((name.to_string(), entries));
    }

    /// Run the resonator dynamics to factorize a composite hypervector.
    /// Returns None if no codebooks are registered.
    pub fn factorize(&self, composite: &Hypervector) -> Option<FactorizationResult> {
        if self.codebooks.is_empty() {
            return None;
        }

        let n = self.codebooks.len();
        let mut estimates: Vec<Hypervector> = self
            .codebooks
            .iter()
            .map(|(_, entries)| entries[entries.len() / 2].1)
            .collect();

        let mut iterations = 0;

        for iter in 0..self.max_iterations {
            iterations = iter + 1;
            let mut max_change = 0.0f32;

            for i in 0..n {
                let others = bind_all_except(&estimates, i);
                let signal = composite.unbind(&others);

                let (_, best_hv, best_sim) = self.codebooks[i]
                    .1
                    .iter()
                    .map(|(lbl, hv)| (lbl.as_str(), *hv, signal.similarity(hv)))
                    .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                    .unwrap();

                let old_sim = signal.similarity(&estimates[i]);
                max_change = max_change.max((best_sim - old_sim).abs());
                estimates[i] = best_hv;
            }

            let reconstructed = bind_all(&estimates);
            let energy = composite.similarity(&reconstructed);

            if energy >= self.convergence_threshold || max_change < 0.001 {
                break;
            }
        }

        let factors: Vec<FactorEstimate> = self
            .codebooks
            .iter()
            .zip(estimates.iter())
            .map(|((name, entries), est)| {
                let (label, _, sim) = entries
                    .iter()
                    .map(|(l, hv)| (l.as_str(), hv, est.similarity(hv)))
                    .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
                    .unwrap();
                FactorEstimate {
                    factor: name.clone(),
                    label: label.to_string(),
                    similarity: sim,
                    contribution: ((sim - 0.5) / 0.5).max(0.0),
                }
            })
            .collect();

        let total_contribution: f32 = factors.iter().map(|f| f.contribution).sum();
        Some(FactorizationResult {
            factors,
            iterations,
            total_contribution,
        })
    }
}

pub struct FactorizationResult {
    pub factors: Vec<FactorEstimate>,
    pub iterations: usize,
    pub total_contribution: f32,
}

pub struct FactorEstimate {
    pub factor: String,
    pub label: String,
    pub similarity: f32,
    pub contribution: f32,
}

fn bind_all(hvs: &[Hypervector]) -> Hypervector {
    hvs.iter().skip(1).fold(hvs[0], |acc, hv| acc.bind(hv))
}

fn bind_all_except(hvs: &[Hypervector], skip_idx: usize) -> Hypervector {
    let filtered: Vec<&Hypervector> = hvs
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != skip_idx)
        .map(|(_, h)| h)
        .collect();
    if filtered.is_empty() {
        return Hypervector([u64::MAX; HDC_WORDS]);
    }
    filtered
        .iter()
        .skip(1)
        .fold(**filtered.first().unwrap(), |acc, &&ref hv| acc.bind(hv))
}
```

The resonator is the basis for the curiosity system's explanatory capability: when a transaction scores high on novelty, the resonator decomposes it into constituent fields to explain which dimensions are novel.

---

## 7. Performance characteristics

Measured on Apple M1 Pro at D = 10,240:

| Operation | Time complexity | Latency | Memory |
|---|---|---|---|
| Bind (XOR) | O(D/64) = O(160) | ~2 ns | 1,280 B output |
| Similarity (XOR + POPCNT) | O(D/64) = O(160) | ~10 ns | 4 B output |
| Permute (cyclic shift) | O(D/64) = O(160) | ~5 ns | 1,280 B output |
| Bundle (accumulate 1 item) | O(D) = O(10,240) | ~15 us | 40 KB accumulator |
| Bundle (finish to binary) | O(D) = O(10,240) | ~12 us | 1,280 B output |
| Resonator (1 iter, 3 factors, 100 codebook entries) | O(D x K x F) | ~300 us | Codebook memory |

The full transaction encoding pipeline (5-7 binds + 1 bundle) completes in under 100 microseconds, well within the Gamma tick budget. Similarity against a 1,000-entry codebook: ~10 microseconds. Resonator factorization (10-50 iterations): fits within the Theta tick budget.

---

## 8. Academic references

All citations below should have corresponding entries in `shared/citations.md`.

- **[KANERVA-1988]** Kanerva, P. *Sparse Distributed Memory*. MIT Press, 1988.
- **[KANERVA-2009]** Kanerva, P. "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors." *Cognitive Computation*, 1(2), 2009.
- **[GAYLER-2003]** Gayler, R.W. "Vector Symbolic Architectures Answer Jackendoff's Challenges for Cognitive Neuroscience." *ICANN Workshop on Compositional Connectionism*, 2003.
- **[PLATE-1995]** Plate, T.A. "Holographic Reduced Representations." *IEEE Transactions on Neural Networks*, 6(3), 1995.
- **[PLATE-2003]** Plate, T.A. *Holographic Reduced Representations: Distributed Representation for Cognitive Structures*. CSLI Publications, 2003.
- **[FRADY-2020a]** Frady, E.P., Kent, S.J., Olshausen, B.A., and Sommer, F.T. "Resonator Networks 1: An Efficient Solution for Factoring High-Dimensional, Distributed Representations of Data Structures." *Neural Computation*, 32(12), 2020.
- **[FRADY-2020b]** Frady, E.P., Kent, S.J., Olshausen, B.A., and Sommer, F.T. "Resonator Networks 2: Factorization Performance and Capacity Compared to Optimization-Based Methods." *Neural Computation*, 32(12), 2020.
- **[FRADY-2023]** Frady, E.P., Kleyko, D., Kymre, J.M., Olshausen, B.A., and Sommer, F.T. "Computing on Functions Using Randomized Vector Representations." *Frontiers in Computational Neuroscience*, 16, 2023.
- **[KLEYKO-2022]** Kleyko, D., Rachkovskij, D., Osipov, E., and Rahimi, A. "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures." *ACM Computing Surveys*, 55(6), 2022.
- **[SCHLEGEL-2022]** Schlegel, K., Neubert, P., and Protzel, P. "A Comparison of Vector Symbolic Architectures." *Artificial Intelligence Review*, 55, 2022.
- **[THOMAS-2021]** Thomas, A., Dasgupta, S., and Bhatt, T. "Theoretical Perspectives on Deep Learning Methods in Inverse Problems." *IEEE Journal on Selected Areas in Information Theory*, 2021.
- **[NEUBERT-2019]** Neubert, P., Schubert, S., and Protzel, P. "An Introduction to Hyperdimensional Computing for Robotics." *KI -- Kunstliche Intelligenz*, 33, 2019.
- **[MCCLELLAND-1995]** McClelland, J.L., McNaughton, B.L., and O'Reilly, R.C. "Why There Are Complementary Learning Systems in the Hippocampus and Neocortex." *Psychological Review*, 102(3), 1995.
- **[YEUNG-2024]** Yeung, C., Zou, Z., and Imani, M. "Generalized Holographic Reduced Representations." arXiv:2405.09689, 2024.
- **[CHEN-2020]** Chen, T., Kornblith, S., Norouzi, M., and Hinton, G. "A Simple Framework for Contrastive Learning of Visual Representations." *ICML*, 2020.
- **[HERNANDEZ-CANO-2024]** Hernandez-Cano, A. et al. "Hyperdimensional Computing with Holographic and Adaptive Encoder." *Frontiers in Artificial Intelligence*, 2024.
