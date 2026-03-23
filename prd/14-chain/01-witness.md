# Witness Layer [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crate**: `bardo-witness`

---

> **Reader orientation:** This document specifies the witness layer, the lowest level of Bardo's chain intelligence pipeline (section 14). The witness maintains a perpetual WebSocket connection per chain, ingests every block header, and uses a Binary Fuse filter to decide which blocks deserve a full fetch. It feeds filtered blocks into the triage pipeline for classification. The key concept is pre-screening: over 90% of blocks are skipped in O(1) time with zero false negatives. See `prd2/shared/glossary.md` for full term definitions.

The witness layer is the golem's eyes on the chain. One Tokio task per configured chain maintains a perpetual WebSocket connection to an Ethereum node, ingests every new block header, determines which blocks are relevant using a Binary Fuse pre-screening filter, fetches those blocks in full, and passes them to the triage pipeline. Block ingestion is continuous -- it runs in its own task outside any heartbeat clock. The Heartbeat (the 9-step decision cycle) clocks gate the Golem's cognitive response to what it sees, not the seeing itself.

---

## Binary Fuse Pre-Screening

### Why It Exists

Every Ethereum block header contains a `logsBloom` -- a 2048-bit Bloom filter (256 bytes) that encodes the union of all log-emitting addresses and all indexed event topics across every transaction in that block (Wood, 2014, Section 4.3). This built-in filter answers "might this block contain an event from address X?" with zero false negatives and a tunable false positive rate.

The golem maintains its own filter -- a `BinaryFuse8` from the `xorf` crate -- built from `ChainScope` at each Gamma tick. Binary fuse filters (Lemire et al., 2022) are immutable, rebuild-oriented structures that use only 8.7 bits per entry with sub-1% false positive rates. Since the golem-side filter is rebuilt from scratch every Gamma tick via arc-swap (never updated in place), the immutable-rebuild pattern of binary fuse filters is a natural fit. The older `fastbloom` Bloom filter's mutable-insert advantage is irrelevant here.

The original xor filter paper (Graf & Lemire, 2020) showed 9.1 bits/entry at <1% FPR. Binary fuse filters improved construction speed by ~2x while reducing space further to 8.7 bits/entry. Both outperform standard Bloom filters (~9.6 bits/entry at the same FPR) and cuckoo filters (~8.5 bits/entry but with deletion support the golem doesn't need).

### How It Works

For each arriving block header, the witness tests the header's `logsBloom` against the golem's Binary Fuse filter:

```rust
use xorf::{BinaryFuse8, Filter};
use arc_swap::ArcSwap;
use std::sync::Arc;

pub struct WitnessEngine {
    /// Binary Fuse filter of watched addresses + event topics.
    /// Rebuilt at each Gamma tick, swapped atomically.
    watch_filter: Arc<ArcSwap<BinaryFuse8>>,

    /// Dedicated WS subscription connection.
    subscription: Arc<WsProvider>,

    /// Pool for block + receipt fetches.
    query_pool: deadpool::Pool<WsProvider>,

    /// Tracks processed blocks for gap detection.
    seen_blocks: Arc<Mutex<RoaringBitmap>>,

    /// Latest chain head block number.
    latest_block: AtomicU64,

    /// CorticalState handle for gas_gwei and blocks_behind.
    cortical: Arc<CorticalState>,
}

impl WitnessEngine {
    /// Main ingestion loop. Runs as a single long-lived Tokio task.
    pub async fn run(&self, triage_tx: mpsc::Sender<FullBlock>) -> Result<()> {
        let mut sub = self.subscription.subscribe_blocks().await?;

        while let Some(header) = sub.next().await {
            // Always update gas price -- it's free from the header.
            self.cortical.environment_gas_gwei.store(
                header.base_fee_per_gas.unwrap_or_default() as u32,
                Ordering::Relaxed,
            );

            // Track chain head for staleness detection.
            self.latest_block.store(header.number, Ordering::Relaxed);

            // Binary Fuse filter check against the block's logsBloom.
            let filter = self.watch_filter.load();
            let hit = self.check_bloom_against_filter(&header.logs_bloom, &filter);

            if !hit {
                // Definitive miss. Zero false negatives.
                // Cost: O(k) where k = number of watched items, via hash probes.
                // Typically ~10ns with POPCNT.
                continue;
            }

            // Bloom hit: fetch full block with receipts.
            // ~1% of these will be false positives.
            let conn = self.query_pool.get().await?;
            let full_block = conn.get_block_with_receipts(header.hash).await?;

            // Mark as seen for gap detection.
            self.seen_blocks.lock().insert(header.number as u32);

            // Forward to triage pipeline.
            triage_tx.send(full_block).await?;
        }

        Ok(())
    }

    /// Check whether the block's logsBloom intersects any watched item
    /// in the Binary Fuse filter. The block's logsBloom is a fixed 2048-bit
    /// Ethereum Bloom filter; the golem's filter is a BinaryFuse8.
    ///
    /// For each watched address/topic, compute its Ethereum Bloom hash
    /// and check if the corresponding bits are set in the block's logsBloom.
    /// If any watched item's bits are all set, the block might be relevant.
    fn check_bloom_against_filter(
        &self,
        logs_bloom: &Bloom,
        filter: &BinaryFuse8,
    ) -> bool {
        // The BinaryFuse8 contains hashes of all watched addresses and topics.
        // We extract the log-emitting addresses and topics from the logsBloom
        // and check membership in the BinaryFuse8.
        //
        // In practice, the witness extracts candidate addresses from decoded
        // logsBloom bit positions and tests each against the filter.
        // False positive rate: <1% at typical watch list sizes.
        check_logs_bloom_intersection(logs_bloom, filter)
    }
}
```

On mainnet (~7,500 blocks/day), a golem watching 50 addresses and 100 event topics skips >90% of blocks. The remaining ~10% trigger full block fetches. `CorticalState.environment.gas_gwei` is updated on every block header regardless of filter result.

### Filter Construction

Built from `ChainScope.interest_entries` at each Gamma tick:

```rust
use xorf::BinaryFuse8;
use xxhash_rust::xxh3::xxh3_64;

fn build_watch_filter(entries: &[InterestEntry]) -> BinaryFuse8 {
    let keys: Vec<u64> = entries
        .iter()
        .flat_map(|e| {
            let addr_hash = xxh3_64(e.address.as_slice());
            let topic_hashes = e.event_topics.iter()
                .map(|t| xxh3_64(t.as_slice()));
            std::iter::once(addr_hash).chain(topic_hashes)
        })
        .collect();

    BinaryFuse8::try_from(&keys).expect("filter construction failed")
}
```

The new filter atomically replaces the old one via `arc_swap::ArcSwap`. In-flight checks against the old filter are safe -- they have zero false negatives, so at worst the golem fetches one extra block during the swap.

---

## Block Ingestion Pipeline

```
eth_subscribe("newHeads")
    | block header arrives
    |-- update CorticalState.environment.gas_gwei (always)
    |-- update latest_block_number (AtomicU64)
    +-- BinaryFuse8 filter check
        |-- miss (>90%): continue
        +-- hit:
            |-- eth_getBlockByHash(hash, true)          // full block w/ txs
            |-- eth_getBlockReceipts(hash)               // receipts (logs)
            +-- send (block, receipts) to triage channel
```

The subscription connection is dedicated to `eth_subscribe("newHeads")` only. Block fetches fan out across a separate query connection pool, so burst block activity can't starve the subscription.

---

## Reconnection and Gap Handling

WebSocket connections drop. The witness handles this without losing coverage:

1. **On disconnect**: record `last_processed_block` in `AtomicU64`.
2. **On reconnect**:
   - Call `eth_blockNumber` to get current chain head.
   - Load seen-blocks Roaring Bitmap from redb.
   - Scan for gaps between `last_processed` and current head.
3. **Gap <= 1,000 blocks**: backfill via `eth_getLogs` with watched address filter.
4. **Gap > 1,000 blocks**:
   - Emit `GolemEvent::ChainGapDetected { chain_id, from_block, to_block, gap_size }`.
   - Resume from current head; the gap is a permanent hole in awareness.
   - The Oracle handles this via uncertainty adjustment -- the golem knows it missed something.

### Seen-Blocks Tracking

Uses a Roaring Bitmap (Lemire et al., 2016). Roaring partitions the integer space into 2^16-element chunks, using dense bitsets for continuous ranges and sorted arrays for sparse ones. At ~7,500 blocks/day on mainnet, a 30-day bitmap (~225,000 block numbers) compresses to a few KB. Gap detection on reconnect is O(n) over the gap size, not O(total blocks ever seen).

The bitmap persists to redb at each Delta tick. Retention: 90 days (handles long outages gracefully).

```rust
use roaring::RoaringBitmap;

pub struct GapDetector {
    seen: RoaringBitmap,
    last_processed: u64,
}

impl GapDetector {
    /// Detect gaps between last_processed and current chain head.
    /// Returns ranges of missing block numbers.
    pub fn find_gaps(&self, chain_head: u64) -> Vec<(u64, u64)> {
        let mut gaps = Vec::new();
        let mut expected = self.last_processed + 1;

        for block in expected..=chain_head {
            if !self.seen.contains(block as u32) {
                if gaps.last().map(|(_, end)| *end == block - 1).unwrap_or(false) {
                    gaps.last_mut().unwrap().1 = block;
                } else {
                    gaps.push((block, block));
                }
            }
        }

        gaps
    }

    /// Persist to redb at Delta tick.
    pub fn persist(&self, db: &redb::Database) -> Result<()> {
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(SEEN_BLOCKS_TABLE)?;
            let mut bytes = Vec::new();
            self.seen.serialize_into(&mut bytes)?;
            table.insert("seen_blocks", bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
```

---

## Connection Pool

```rust
pub struct WitnessPool {
    /// Dedicated subscription connection -- never used for anything else.
    /// A single long-lived WS connection for eth_subscribe("newHeads").
    subscription_conn: Arc<WsProvider>,

    /// Pool for block + receipt fetches. Multiple concurrent fetches
    /// when a burst of relevant blocks arrives.
    query_conns: deadpool::Pool<WsProvider>,

    /// HTTP fallback when WS query pool saturates.
    rpc_fallbacks: Vec<Arc<HttpProvider>>,
}
```

The subscription connection is isolated -- burst block activity can't starve it. Query connections fan out parallel fetches (tokio::spawn per fetch, bounded by pool size). HTTP fallback activates when the WS pool is saturated.

Reconnection logic for the subscription connection uses exponential backoff: 3s, 6s, 12s, 30s max.

---

## Per-Chain Configuration

```rust
pub struct ChainWitnessConfig {
    pub chain_id: u64,
    pub ws_url: String,
    pub rpc_urls: Vec<String>,       // HTTP fallbacks
    pub query_pool_size: usize,      // default: 4
    pub gap_backfill_limit: u64,     // default: 1_000 blocks
    pub max_watch_size: usize,       // cap on filter items, default: 10_000
}
```

One `WitnessEngine` per configured chain, each running independently. If one chain's WS drops, others are unaffected.

---

## Block Normalization

Before forwarding to triage, the witness normalizes the block into an internal `NormalizedBlock` that abstracts over chain-specific differences (L1 vs L2 transaction types, EIP-4844 blob transactions, etc.):

```rust
pub struct NormalizedBlock {
    pub chain_id: u64,
    pub number: u64,
    pub hash: B256,
    pub timestamp: u64,
    pub base_fee_per_gas: u64,
    pub transactions: Vec<NormalizedTx>,
    pub receipts: Vec<TransactionReceipt>,
}

pub struct NormalizedTx {
    pub hash: B256,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub input: Bytes,
    pub gas_used: u64,
    pub creates: Option<Address>,  // contract deployment
    pub tx_type: TxType,
}
```

---

## Emitted GolemEvents

| Event | Trigger | Payload |
|-------|---------|---------|
| `ChainGapDetected` | WS reconnect with gap > 1,000 blocks | `chain_id, from_block, to_block, gap_size` |

All other chain intelligence events are emitted by downstream crates (triage, protocol-state). The witness itself is silent except for gaps.

See [06-events-signals.md](./06-events-signals.md) for the full event catalog.

---

## CorticalState Updates

| Signal | Update trigger | Source |
|--------|----------------|--------|
| `environment.gas_gwei` | Every block header | `block.base_fee_per_gas` |
| `chain.blocks_behind` | Every Gamma tick | `chain_head - last_processed_block` |

See [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) for CorticalState layout and [06-events-signals.md](./06-events-signals.md) for the `chain_blocks_behind` signal specification.

---

## Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `witness.blocks_received` | counter | Total block headers received |
| `witness.filter_hits` | counter | Blocks passing filter check |
| `witness.filter_misses` | counter | Blocks rejected by filter |
| `witness.full_fetches` | counter | Full block+receipts fetches |
| `witness.gaps_detected` | counter | WS reconnect gaps |
| `witness.gap_blocks_backfilled` | counter | Blocks recovered via backfill |
| `witness.ws_reconnects` | counter | WebSocket reconnection events |
| `witness.fetch_latency_ms` | histogram | Full block fetch latency |

Filter hit rate (`hits / (hits + misses)`) is the primary tuning metric. If it exceeds 20%, the filter may be too permissive -- review `ChainScope` interest scoring (see [04-chain-scope.md](./04-chain-scope.md)).

---

## Dependencies

```toml
[dependencies]
alloy = { version = "1", features = ["ws", "pubsub"] }
alloy-primitives = "1"
xorf = "0.11"                        # Binary Fuse filters
xxhash-rust = { version = "0.8", features = ["xxh3"] }
arc-swap = "1"
tokio = { version = "1", features = ["full"] }
deadpool = { version = "0.10", features = ["managed"] }
roaring = "0.10"
redb = "2"
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
```

---

## Cross-References

- **Architecture**: [00-architecture.md](./00-architecture.md) -- Five-crate overview of chain intelligence, data flow from WebSocket to storage, cybernetic feedback loop, storage budget
- **Triage pipeline**: [02-triage.md](./02-triage.md) -- Downstream consumer of witness output; 4-stage classification pipeline that scores each transaction's relevance
- **Chain scope**: [04-chain-scope.md](./04-chain-scope.md) -- Source of the BinaryFuse8 filter; dynamic attention model rebuilt each Gamma tick from CorticalState
- **Events and signals**: [06-events-signals.md](./06-events-signals.md) -- ChainGapDetected event variant and all other chain intelligence GolemEvent definitions
- **Heartbeat**: [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- Gamma/Theta/Delta clock definitions; Gamma tick triggers filter rebuild

---

## References

- Bloom, B.H. (1970). Space/time trade-offs in hash coding with allowable errors. *Communications of the ACM*, 13(7). -- *Original probabilistic membership filter; the Ethereum logsBloom is a direct descendant.*
- Fan, B. et al. (2014). Cuckoo Filter: Practically Better Than Bloom. *CoNEXT*. -- *Alternative filter with deletion support at ~8.5 bits/entry; serves as comparison point for the Binary Fuse choice.*
- Graf, T.M. & Lemire, D. (2020). Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters. arXiv:1912.08258. *Journal of Experimental Algorithmics*. -- *Predecessor to Binary Fuse filters at 9.1 bits/entry; establishes the immutable-rebuild pattern used here.*
- Lemire, D. et al. (2016). Consistently faster and smaller compressed bitmaps with Roaring. *Software: Practice & Experience*. -- *Describes the Roaring Bitmap used for seen-block tracking and gap detection on reconnect.*
- Lemire, D. et al. (2022). Binary Fuse Filters: Fast and Smaller Than Xor Filters. *Journal of Experimental Algorithmics*. -- *The specific filter implementation used: 8.7 bits/entry, sub-1% FPR, 2x faster construction than xor filters.*
- Wood, G. (2014). Ethereum Yellow Paper. Section 4.3 (logsBloom definition). -- *Defines the 2048-bit block-level Bloom filter that the witness tests against.*
