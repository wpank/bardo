# bardo-primitives

Pure compute primitives with zero internal workspace dependencies. Two things live here: a 10,240-bit hyperdimensional computing vector and a three-tier inference gate. If you need either of those without pulling in the full golem platform stack, this is the right crate.

## Exports

```rust
pub use hdc::HdcVector;
pub use tier::{InferenceTier, TierError, TierRouter};
```

## HdcVector

A 10,240-bit binary sparse distributed representation stored as `[u64; 160]` — 1,280 bytes, `Copy`, no heap allocation.

The three core operations:

- `HdcVector::bind(&self, other: &HdcVector) -> HdcVector` — XOR. This is involution: `a.bind(&b).bind(&b) == a`. Use it to associate two concepts.
- `HdcVector::bundle(vectors: &[&HdcVector]) -> HdcVector` — majority vote across all input vectors, bit by bit. Ties go to 0. Use it to form a superposition of multiple concepts.
- `HdcVector::similarity(&self, other: &HdcVector) -> f32` — Hamming similarity in `[0.0, 1.0]`. Two random vectors will score around 0.5. Identical vectors score 1.0.

Construction:

```rust
let random = HdcVector::random();          // seeded from UUID v4
let zero   = HdcVector::zeros();           // all bits clear
let stable = HdcVector::from_seed(b"swap"); // deterministic from byte slice
```

Serialization:

```rust
let bytes: [u8; 1280] = vec.to_bytes();
let recovered = HdcVector::from_bytes(&bytes);
```

There's also a `permute(n)` method for cyclic bit rotation, useful when you want to encode position in a sequence.

**Why not float embeddings?** Comparison is ~50 ns vs 10–50 ms for typical float embedding cosine similarity. The tradeoff is lower fidelity per dimension — HDC compensates with width (10,240 bits).

## InferenceTier

```rust
pub enum InferenceTier {
    T0 = 0,  // suppress — no LLM call
    T1 = 1,  // analyze — Haiku-class
    T2 = 2,  // deliberate — Opus or Sonnet based on vitality
}
```

Converts `TryFrom<u8>` (returns `TierError` on values > 2) and `Into<u8>`.

## TierRouter

A zero-state unit struct. `select_model` is a pure function with no side effects.

```rust
TierRouter::select_model(tier: InferenceTier, vitality: f32) -> Option<&'static str>
```

| Tier | Vitality   | Result                  |
|------|------------|-------------------------|
| T0   | any        | `None`                  |
| T1   | any        | `"claude-haiku-4-5"`    |
| T2   | ≥ 0.3      | `"claude-opus-4-6"`     |
| T2   | < 0.3      | `"claude-sonnet-4"`     |

The vitality threshold at 0.3 is exact, not fuzzy. At exactly 0.3, the result is `"claude-opus-4-6"`.

All model selection logic in the workspace flows through this one function.

## Usage

```toml
[dependencies]
bardo-primitives = { path = "../../crates/bardo-primitives" }
```

Optional feature: `rkyv` enables zero-copy deserialization for `HdcVector`, useful when reading vectors from a memory-mapped LanceDB buffer. With the `rkyv` feature, `HdcVector::similarity_archived` compares directly against an `ArchivedHdcVector` without deserializing it first.
