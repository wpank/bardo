# HDC Transaction Fingerprints [SPEC]

> **Document Type**: REF (normative) | **Parent**: `shared/hdc-vsa.md` | **Last Updated**: 2026-03-18
>
> Transaction fingerprint encoding, structured queries, three-tier architecture, and ANN index integration. All types and operations from `shared/hdc-vsa.md` are prerequisites.

> **Reader orientation:** This document specifies how Bardo encodes on-chain transactions into HDC (Hyperdimensional Computing) fingerprints for structured similarity search and algebraic queries. It is a child of `shared/hdc-vsa.md` (which covers the BSC algebra and capacity bounds) and belongs to the `shared/` reference layer. The key concept is role-filler encoding: each transaction field (protocol, selector, gas tier, token pair) is bound to a role vector via XOR, then all role-filler pairs are bundled into a single 1,280-byte fingerprint. This enables queries that flat embeddings cannot support, like "find transactions like X but from a different sender." See `prd2/shared/glossary.md` for full term definitions.

---

## 1. Transaction fingerprint encoding

On-chain transactions carry structured categorical data: sender, recipient, function selector, log topics, gas, value, tokens. HDC encodes each field as a role-filler pair, then bundles them all into a single 1,280-byte fingerprint.

Three fingerprinting approaches appear across the research. The 80-dimensional handcrafted vector is fast and interpretable but limited in expressiveness -- it cannot represent relationships between fields and adding new feature types requires redesigning the vector layout. The 10K-dimensional BSC encoding described here is algebraically composable: you can unbind components, compose protocol family vectors, inherit knowledge across agent generations, and perform structured similarity queries. Contrastive self-supervised embeddings (SimCLR-style learned representations) achieve the best raw recall on similarity search, but they are opaque, non-invertible, and expensive to train. The selected approach uses BSC as the primary representation for all algebraic operations, with contrastive embeddings available as an optional secondary layer for pure similarity search. The 80-dimensional handcrafted fingerprint becomes the cold-start fallback -- used during the first minutes of a Golem's life before the item memory has populated.

### 1.1 Encoding pattern

```
HV_tx = bundle(
    bind(R_protocol,    HV_uniswap_v3),
    bind(R_selector,    HV_exactInputSingle),
    bind(R_gas_tier,    HV_medium),
    bind(R_value_bucket, HV_large),
    bind(R_from_cluster, HV_cluster_7),
    bind(R_to_cluster,   HV_dex_router),
    bind(R_log_topic,   permute(HV_Transfer, 0)),
    bind(R_log_topic,   permute(HV_Swap, 1)),
    bind(R_token_in,    HV_WETH),
    bind(R_token_out,   HV_USDC),
)
```

Role vectors (`R_protocol`, `R_selector`, etc.) are fixed random hypervectors generated once from the item memory seed. Filler vectors (`HV_uniswap_v3`, `HV_exactInputSingle`, etc.) are generated deterministically on first encounter and cached.

### 1.2 What the encoding enables that flat embeddings cannot

1. **Invertible queries.** `unbind(HV_tx, R_protocol)` + nearest-neighbor lookup in the protocol codebook recovers the protocol. No classifier needed.

2. **Algebraic field swapping.** To find "transactions like this one but from a different sender cluster": unbind the old cluster, bind the new one, search. The query is a hypervector, not a database filter.

3. **Open schema.** Adding a new field (e.g., MEV classification) requires only a new role vector. No layout redesign. No retraining. The new field participates in all existing similarity and unbinding operations automatically.

---

## 2. Field encoding details

### 2.1 Protocol identifier

Atomic hypervectors: `HV_uniswap_v3`, `HV_aave_v3`, etc. Unknown protocols are encoded by hashing the factory/router address. Protocol families can be constructed by bundling members: `HV_uniswap_family = bundle(HV_uniswap_v2, HV_uniswap_v3, HV_uniswap_v4)`.

### 2.2 Function selector

The 4-byte selector is hashed to an item memory key: `"selector:0xa9059cbb"`. Selectors with known ABI names get a human-readable alias: `"selector:transfer"`. Two contracts implementing the same interface produce identical selector hypervectors.

### 2.3 Gas tier (thermometer encoding)

Gas consumption is bucketed into discrete tiers. Adjacent tiers share partial similarity through thermometer encoding: the "medium" vector is the bundle of `gas:low` and `gas:medium` atoms, preserving ordinal relationships.

| Tier | Gas range | Label |
|---|---|---|
| minimal | < 30,000 | `gas:minimal` |
| low | 30,000 - 100,000 | `gas:low` |
| medium | 100,000 - 300,000 | `gas:medium` |
| high | 300,000 - 1,000,000 | `gas:high` |
| extreme | > 1,000,000 | `gas:extreme` |

### 2.4 Value bucket (thermometer encoding)

Transaction value bucketed logarithmically with the same thermometer pattern.

| Bucket | Range | Label |
|---|---|---|
| dust | < 0.01 ETH | `value:dust` |
| small | 0.01 - 1 ETH | `value:small` |
| medium | 1 - 10 ETH | `value:medium` |
| large | 10 - 100 ETH | `value:large` |
| whale | > 100 ETH | `value:whale` |

### 2.5 Address cluster

Raw addresses are too numerous for direct encoding. Addresses are mapped to behavioral clusters: MEV bots, DEX routers, lending contracts, bridge contracts, EOA whales, governance multisigs. Unknown addresses fall into ~256 hash-bucketed "other" categories for coarse-grained similarity.

### 2.6 Log topics

Position-encoded: `bind(R_log_topic, permute(HV_Transfer, 0)) + bind(R_log_topic, permute(HV_Swap, 1))`. Permutation by position index distinguishes event ordering. Capped at 8 log events per transaction to maintain SNR.

### 2.7 Token identifiers

`bind(R_token_in, HV_WETH) + bind(R_token_out, HV_USDC)`. Enables token-pair similarity: two WETH/USDC swaps on different DEXes share these role-filler pairs.

---

## 3. Rust encoder

```rust
use crate::hdc::{Hypervector, BundleAccumulator, ItemMemory};

/// Encodes decoded on-chain transactions into BSC hypervectors.
///
/// Each transaction becomes a role-filler bundle:
///   HV_tx = bundle(bind(R_i, F_i) for each field i)
///
/// The encoder owns the item memory and role vector assignments.
/// Thread safety: wrap in Arc<Mutex<>> for concurrent Gamma-tick access,
/// or clone the item memory per thread (deterministic seeds guarantee
/// identical vectors across clones).
pub struct TxHvEncoder {
    item_memory: ItemMemory,
}

/// Decoded transaction fields for encoding.
/// Produced by the triage pipeline's ABI decoder before HDC encoding.
pub struct DecodedTxFields {
    pub protocol_name: Option<String>,
    pub function_selector: [u8; 4],
    pub function_name: Option<String>,
    pub gas_used: u64,
    pub value_eth: f64,
    pub from_cluster: String,
    pub to_cluster: String,
    pub log_topics: Vec<String>,
    pub token_in: Option<String>,
    pub token_out: Option<String>,
}

impl TxHvEncoder {
    pub fn new(seed: u64) -> Self {
        TxHvEncoder {
            item_memory: ItemMemory::new(seed),
        }
    }

    /// Encode a decoded transaction into a single BSC hypervector.
    /// Cost: O(D * num_fields), typically 7-12 fields = ~100 us.
    pub fn encode(&mut self, tx: &DecodedTxFields) -> Hypervector {
        let mut acc = BundleAccumulator::new();

        // Protocol
        let r_protocol = self.item_memory.encode("role:protocol");
        if let Some(ref proto) = tx.protocol_name {
            let filler = self.item_memory.encode(&format!("proto:{}", proto));
            acc.add(&r_protocol.bind(&filler));
        }

        // Function selector
        let r_selector = self.item_memory.encode("role:selector");
        let sel_label = match &tx.function_name {
            Some(name) => format!("selector:{}", name),
            None => format!(
                "selector:0x{}",
                hex::encode(tx.function_selector)
            ),
        };
        let sel_filler = self.item_memory.encode(&sel_label);
        acc.add(&r_selector.bind(&sel_filler));

        // Gas tier (thermometer encoding)
        let r_gas = self.item_memory.encode("role:gas_tier");
        let gas_hv = self.encode_gas_tier(tx.gas_used);
        acc.add(&r_gas.bind(&gas_hv));

        // Value bucket (thermometer encoding)
        let r_value = self.item_memory.encode("role:value_bucket");
        let value_hv = self.encode_value_bucket(tx.value_eth);
        acc.add(&r_value.bind(&value_hv));

        // Address clusters
        let r_from = self.item_memory.encode("role:from_cluster");
        let from_hv = self.item_memory.encode(&format!("cluster:{}", tx.from_cluster));
        acc.add(&r_from.bind(&from_hv));

        let r_to = self.item_memory.encode("role:to_cluster");
        let to_hv = self.item_memory.encode(&format!("cluster:{}", tx.to_cluster));
        acc.add(&r_to.bind(&to_hv));

        // Log topics with positional encoding
        let r_log = self.item_memory.encode("role:log_topic");
        for (pos, topic) in tx.log_topics.iter().take(8).enumerate() {
            let topic_hv = self.item_memory.encode(&format!("topic:{}", topic));
            let positioned = topic_hv.permute(pos);
            acc.add(&r_log.bind(&positioned));
        }

        // Token identifiers
        if let Some(ref token) = tx.token_in {
            let r_token_in = self.item_memory.encode("role:token_in");
            let token_hv = self.item_memory.encode(&format!("token:{}", token));
            acc.add(&r_token_in.bind(&token_hv));
        }
        if let Some(ref token) = tx.token_out {
            let r_token_out = self.item_memory.encode("role:token_out");
            let token_hv = self.item_memory.encode(&format!("token:{}", token));
            acc.add(&r_token_out.bind(&token_hv));
        }

        acc.finish()
    }

    /// Thermometer-encoded gas tier.
    /// Adjacent tiers share partial similarity.
    fn encode_gas_tier(&mut self, gas: u64) -> Hypervector {
        let mut tier_acc = BundleAccumulator::new();
        let tiers = [
            (30_000, "gas:minimal"),
            (100_000, "gas:low"),
            (300_000, "gas:medium"),
            (1_000_000, "gas:high"),
            (u64::MAX, "gas:extreme"),
        ];
        for &(threshold, label) in &tiers {
            tier_acc.add(&self.item_memory.encode(label));
            if gas < threshold {
                break;
            }
        }
        tier_acc.finish()
    }

    /// Thermometer-encoded value bucket.
    fn encode_value_bucket(&mut self, value_eth: f64) -> Hypervector {
        let mut tier_acc = BundleAccumulator::new();
        let tiers = [
            (0.01, "value:dust"),
            (1.0, "value:small"),
            (10.0, "value:medium"),
            (100.0, "value:large"),
            (f64::MAX, "value:whale"),
        ];
        for &(threshold, label) in &tiers {
            tier_acc.add(&self.item_memory.encode(label));
            if value_eth < threshold {
                break;
            }
        }
        tier_acc.finish()
    }

    /// Construct a query hypervector by algebraic manipulation of an existing
    /// transaction encoding.
    ///
    /// Example: "find transactions similar to `tx_hv` but with a different protocol"
    ///   1. Unbind the protocol role-filler: stripped = tx_hv.unbind(bind(R_protocol, HV_old_proto))
    ///   2. Bind the new protocol: query = stripped.bind(bind(R_protocol, HV_new_proto))
    ///   3. Search: find nearest neighbors to `query` in the transaction index
    ///
    /// This method implements step 1+2 for the common case of swapping one field.
    pub fn swap_field(
        &mut self,
        tx_hv: &Hypervector,
        role_name: &str,
        old_filler: &str,
        new_filler: &str,
    ) -> Hypervector {
        let role_hv = self.item_memory.encode(role_name);
        let old_hv = self.item_memory.encode(old_filler);
        let new_hv = self.item_memory.encode(new_filler);

        // Unbind old role-filler, bind new one.
        // Because XOR is self-inverse: unbind = bind.
        // tx_hv XOR bind(role, old) XOR bind(role, new)
        let old_bound = role_hv.bind(&old_hv);
        let new_bound = role_hv.bind(&new_hv);
        tx_hv.bind(&old_bound).bind(&new_bound)
    }

    /// Build a protocol family vector by bundling all known instances.
    /// The family vector is similar to any individual member.
    pub fn build_protocol_family(
        &mut self,
        member_names: &[&str],
    ) -> Hypervector {
        let mut acc = BundleAccumulator::new();
        for name in member_names {
            let hv = self.item_memory.encode(&format!("proto:{}", name));
            acc.add(&hv);
        }
        acc.finish()
    }
}
```

---

## 4. Structured queries via algebraic manipulation

The real payoff of the role-filler encoding is the ability to construct queries by manipulating the hypervector algebra. Here are the query patterns that are impossible with flat embeddings.

### Query: "what protocol produced this transaction?"

```rust
let r_protocol = encoder.item_memory.encode("role:protocol");
let unbound = tx_hv.unbind(&r_protocol);
// Compare `unbound` against all protocol codebook entries.
// The nearest match is the protocol.
let protocol = codebook.nearest(&unbound);
```

Cost: one XOR (unbind) + one linear scan of the protocol codebook (~100 entries). Total: ~1 microsecond.

### Query: "find transactions like X but from a different sender cluster"

```rust
let query = encoder.swap_field(
    &tx_hv,
    "role:from_cluster",
    "cluster:mev_bot",
    "cluster:retail_eoa",
);
// Search the transaction index for nearest neighbors of `query`.
let results = tx_index.search(&query, k: 10);
```

This is an algebraic query: it modifies a specific dimension of the transaction's meaning while preserving everything else. The result is a hypervector that encodes "a transaction identical to X in protocol, function, gas, value, log structure, and tokens, but sent by a retail EOA instead of an MEV bot." No embedding model can do this because learned embeddings entangle all features.

### Query: "how similar are Uniswap v3 swaps to Curve swaps?"

```rust
let uni_family = encoder.build_protocol_family(&["uniswap_v3"]);
let curve_family = encoder.build_protocol_family(&["curve_3pool", "curve_tricrypto"]);
let similarity = uni_family.similarity(&curve_family);
// Returns ~0.5 (near-orthogonal) because they use different contracts,
// but if you bundle their transaction prototypes (not just protocol names),
// the similarity reflects shared structural patterns (both are AMM swaps).
```

### Query: "decompose this anomalous transaction into known factors"

```rust
use crate::hdc::ResonatorNetwork;

let mut resonator = ResonatorNetwork::new(50); // max 50 iterations
resonator.add_factor("protocol", protocol_codebook);
resonator.add_factor("action_type", action_codebook);
resonator.add_factor("value_tier", value_codebook);
resonator.add_factor("gas_tier", gas_codebook);

let result = resonator.factorize(&anomalous_tx_hv).unwrap();
println!("{}", result.attribution_summary());
// "protocol (unknown_0x3f..): 45% | action_type (flash_loan): 31% |
//  value_tier (whale): 18% | gas_tier (extreme): 6%"
```

The resonator network decomposes the transaction into its constituent factors, providing structured attribution without requiring a classifier or LLM call. This is the basis for the curiosity system's explanatory capability: when a transaction scores high on novelty, the resonator can explain *which dimensions* are novel.

---

## 5. Three-tier fingerprint architecture

The system uses three fingerprint representations in a layered architecture:

**Tier 1: 80-dim handcrafted (cold start).** Used during the Golem's first minutes before the item memory has seen enough concepts. Zero setup cost. No learning. Acts as the floor.

**Tier 2: 10,240-bit BSC (primary).** Used for all algebraic operations: structured queries, protocol family bundling, anomaly decomposition, generational inheritance. Deterministic -- same transaction always produces the same hypervector.

**Tier 3: Contrastive self-supervised 128-dim (optional).** SimCLR-trained embedding for pure similarity search. Captures latent patterns that neither handcrafted nor BSC can discover, but opaque and non-invertible.

Routing:
- **Gamma tick**: 80-dim fast reject (>90% of events) -> BSC structured similarity -> optional contrastive recall boost
- **Theta tick**: BSC resonator decomposition for LLM prompt enrichment; contrastive for Grimoire retrieval
- **Delta tick**: Contrastive model retrained; BSC codebook persisted (no retraining needed)

---

## 6. Codebook management

A codebook is an `ItemMemory` instance (or a namespaced partition within one) that maps concept names to their canonical hypervectors.

**Lazy initialization.** Vectors are generated on first access via `ItemMemory::encode()`. No bulk pre-loading required. The seed ensures determinism across process restarts.

**Namespacing convention.** Keys use `domain:value` format: `"proto:uniswap_v3"`, `"selector:transfer"`, `"gas:medium"`, `"role:protocol"`.

**One codebook per role vs. shared.** The cleanest design uses one `ItemMemory` per role domain. The Bardo implementation uses a single shared instance with namespaced keys for simplicity. Either approach produces identical vectors for the same key.

**Codebook size constraints.** The maximum useful codebook size per role depends on the number of items bundled (K) and the required retrieval accuracy. At K = 5 role-filler pairs per transaction and D = 10,240, codebooks of >100,000 entries are safe. In practice, the largest codebook (address clusters) has ~500 entries.

---

## 7. Storage and ANN index integration

A BSC hypervector at D = 10,240 occupies 1,280 bytes. For the Grimoire's projected scale:

| Episode count | BSC vectors | HNSW overhead | Total |
|---|---|---|---|
| 1,000 | 1.25 MB | ~5 MB | ~6 MB |
| 10,000 | 12.5 MB | ~50 MB | ~63 MB |
| 100,000 | 125 MB | ~500 MB | ~625 MB |

**Why usearch binary mode.** HNSW indexes over Hamming distance avoid the float-conversion overhead that cosine-distance HNSW would require on binary data. The `usearch` crate supports binary vectors natively with the Hamming metric.

At 100,000 episodes the HNSW overhead becomes the bottleneck. The mitigation is the HDC curiosity codebook, which replaces the per-episode ANN index with two bundled prototype vectors -- O(1) memory regardless of episode count.

For the primary use case of Hamming-distance similarity search, the `usearch` crate supports binary vectors natively with the Hamming metric. This avoids the float-conversion overhead that would be required to use cosine-distance HNSW on binary data.

**Complementary roles.** The ANN index provides fine-grained nearest-neighbor retrieval. Protocol family vectors (bundled prototypes) provide coarse categorical classification. Both are useful; they serve different query patterns.
