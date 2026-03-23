# Protocol State Engine [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crate**: `bardo-protocol-state`

---

> **Reader orientation:** This document specifies the protocol state engine, one of the five crates in Bardo's chain intelligence layer (section 14). It maintains a live, zero-latency model of every DeFi protocol a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) knows about, using a three-layer storage architecture (in-memory DashMap, redb on disk, optional external indexer). The key concept is autonomous discovery: the only hardcoded knowledge is a set of ~20 seed factory addresses, and the engine discovers all child protocols from factory events. See `prd2/shared/glossary.md` for full term definitions.

> **Terminology:** In the codebase, "vault" always means "ERC-4626 tokenized vault contract" unless qualified.

The protocol state engine maintains a live, always-accurate model of every DeFi protocol the golem knows about. State updates on on-chain events, not on polling. Reading state is zero-latency through a lock-free DashMap. The system discovers new protocols autonomously -- the only hardcoded knowledge is a short list of seed factory addresses that root the DeFi protocol tree.

---

## Three-Layer Architecture

### Hot Layer (In-Memory)

```rust
pub struct ProtocolStateEngine {
    /// Hot layer: zero-contention concurrent reads.
    /// The 60fps TUI render loop and bardo-stream-api read this
    /// without blocking the write path.
    hot: Arc<DashMap<ProtocolKey, ProtocolState>>,

    /// Warm layer: redb on disk for restart recovery and history.
    warm: Arc<WarmStore>,

    /// Protocol registry: definitions of all known protocols.
    registry: Arc<ProtocolRegistry>,

    /// Event fabric for emitting state change events.
    fabric: EventFabricHandle,

    /// Provider for eth_call reads.
    provider: Arc<dyn Provider>,
}

pub type ProtocolKey = (u64, Address);  // (chain_id, contract_address)

pub enum ProtocolState {
    UniswapV3Pool(UniswapV3PoolState),
    UniswapV4Pool(UniswapV4PoolState),
    AaveMarket(AaveMarketState),
    MorphoMarket(MorphoMarketState),
    ERC4626Vault(ERC4626VaultState),
    Generic(serde_json::Value),  // discovered but unrecognized family
}

pub struct UniswapV3PoolState {
    pub tick: i32,
    pub sqrt_price_x96: U256,
    pub liquidity: u128,
    pub fee_growth_global_0_x128: U256,
    pub fee_growth_global_1_x128: U256,
    pub block_number: u64,
    pub timestamp_ms: u64,
}
```

The hot layer is an `Arc<DashMap>` updated on every relevant triage event. Reads are lock-free on the fast path. Writes use DashMap's shard-level locking, which means concurrent reads on other shards are unaffected.

### Warm Layer (redb on Disk)

Persisted snapshots for restart recovery. On restart, the hot layer seeds from redb -- avoiding cold-start resync from chain.

```
Table "protocol_state":
  key:   (chain_id: u32, address: [u8; 20])
  value: bincode-encoded ProtocolStateSnapshot { state, block_number, timestamp }

Table "protocol_history":
  key:   (chain_id: u32, address: [u8; 20], block_number: u64)
  value: bincode-encoded StateDelta (field-level diff, delta-encoded values)

Table "protocol_defs":
  key:   (chain_id: u32, address: [u8; 20])
  value: bincode-encoded ProtocolDef
```

History is stored as delta sequences rather than full snapshots per block. A pool's `sqrtPriceX96` (U256, 20 bytes) changes by small amounts per block under normal conditions. Delta + varint encoding achieves 10-100x compression vs. raw storage.

Retention: 30 days for deltas, unlimited for snapshots (small). History serves `bardo-stream-api`'s `/protocol/{address}/history` endpoint (see [08-stream-api.md](./08-stream-api.md)).

### Cold Layer (Optional, External)

rindexer (Stevens, 2024) or The Graph provide full event history when configured. Not held in memory -- queried on demand for historical analysis. rindexer is configured dynamically as protocols are discovered (see autonomous discovery below).

---

## ProtocolDef

The runtime description of every tracked protocol:

```rust
pub struct ProtocolDef {
    pub id: ProtocolId,                  // derived: keccak(family + chain_id + address)
    pub chain_id: u64,
    pub contract_address: Address,
    pub family: ProtocolFamily,
    pub abi: Option<Abi>,                // resolved from ABI chain (async)
    pub update_trigger: UpdateTrigger,
    pub state_reader: Arc<dyn ProtocolStateReader + Send + Sync>,
    pub subgraph_url: Option<Url>,       // auto-discovered from The Graph API
    pub rindexer_config: Option<RindexerEventConfig>,
    pub parent: Option<Address>,         // factory that deployed this
    pub discovered_at_block: u64,
    pub last_updated_block: u64,
}

pub enum UpdateTrigger {
    /// Update when these event topics appear in a block.
    OnEvent(Vec<B256>),
    /// Update every N blocks regardless of events.
    OnBlock(u64),
    /// Poll at a fixed interval (for contracts without event coverage).
    Polling(Duration),
    /// Most protocols: event-triggered + periodic refresh.
    Hybrid {
        events: Vec<B256>,
        full_refresh_blocks: u64,
    },
}

pub enum ProtocolFamily {
    UniswapV2Pair,
    UniswapV3Pool,
    UniswapV4Pool,
    AaveV3Market,
    MorphoMarket,
    CompoundV3Market,
    CurvePool,
    BalancerPool,
    ERC4626Vault,
    ERC20Token,
    Unknown { bytecode_hash: B256 },
}
```

---

## ProtocolStateReader Trait

New protocol families are added by implementing this trait:

```rust
#[async_trait]
pub trait ProtocolStateReader: Send + Sync {
    /// Read fresh state from chain via eth_call.
    /// Implementations should parallelize independent calls via tokio::join!.
    async fn read_state(
        &self,
        address: Address,
        provider: &dyn Provider,
    ) -> Result<ProtocolState>;

    /// Which event topics trigger a state re-read.
    fn event_triggers(&self) -> &[B256];

    /// Protocol family classification.
    fn protocol_family(&self) -> ProtocolFamily;

    /// How this protocol's state should be refreshed.
    fn update_trigger(&self) -> UpdateTrigger;
}
```

### Example: UniswapV3Pool Reader

Reads four storage slots in parallel:

```rust
pub struct UniswapV3PoolReader;

#[async_trait]
impl ProtocolStateReader for UniswapV3PoolReader {
    async fn read_state(
        &self,
        addr: Address,
        provider: &dyn Provider,
    ) -> Result<ProtocolState> {
        let pool = IUniswapV3Pool::new(addr, provider);

        // Four independent eth_call reads in parallel.
        let (slot0, liquidity, fee0, fee1) = tokio::join!(
            pool.slot0().call(),
            pool.liquidity().call(),
            pool.fee_growth_global0_x128().call(),
            pool.fee_growth_global1_x128().call(),
        );

        Ok(ProtocolState::UniswapV3Pool(UniswapV3PoolState {
            tick: slot0?.tick,
            sqrt_price_x96: slot0?.sqrt_price_x96,
            liquidity: liquidity?,
            fee_growth_global_0_x128: fee0?,
            fee_growth_global_1_x128: fee1?,
            block_number: 0, // set by engine after read
            timestamp_ms: now_ms(),
        }))
    }

    fn event_triggers(&self) -> &[B256] {
        &[
            UNISWAP_V3_SWAP_TOPIC,
            UNISWAP_V3_MINT_TOPIC,
            UNISWAP_V3_BURN_TOPIC,
        ]
    }

    fn protocol_family(&self) -> ProtocolFamily {
        ProtocolFamily::UniswapV3Pool
    }

    fn update_trigger(&self) -> UpdateTrigger {
        UpdateTrigger::Hybrid {
            events: vec![
                UNISWAP_V3_SWAP_TOPIC,
                UNISWAP_V3_MINT_TOPIC,
                UNISWAP_V3_BURN_TOPIC,
            ],
            full_refresh_blocks: 100,
        }
    }
}
```

---

## State Update Flow

When `bardo-triage` routes a triage event to the protocol state engine:

```rust
impl ProtocolStateEngine {
    pub async fn handle_triage_event(&self, event: TriageEvent) -> Result<()> {
        let key = (event.chain_id, event.protocol_address);
        let def = match self.registry.get(&key) {
            Some(d) => d,
            None => return Ok(()),  // unknown protocol -- fingerprinter handles it
        };

        // Check if this event triggers a state read.
        if !def.update_trigger.matches(&event.log_topic) {
            return Ok(());
        }

        // Read fresh state from chain (parallel eth_call).
        let new_state = def.state_reader
            .read_state(def.contract_address, &*self.provider)
            .await?;

        // Compute delta vs previous state.
        let prev = self.hot.get(&key);
        let delta = compute_delta(prev.as_deref(), &new_state);

        // Atomic swap in hot layer.
        self.hot.insert(key, new_state.clone());

        // Persist delta to redb warm layer.
        self.warm.write_delta(
            event.chain_id,
            def.contract_address,
            event.block_number,
            &delta,
        )?;

        // Emit GolemEvent.
        self.fabric.emit(GolemEvent::ProtocolStateUpdate {
            protocol_id: def.id.to_string(),
            chain_id: event.chain_id,
            state_delta: serde_json::to_value(&delta)?,
            block_number: event.block_number,
        });

        // Alert if significant change.
        if delta.is_significant(&def.significance_thresholds) {
            self.fabric.emit(GolemEvent::TriageAlert {
                chain_id: event.chain_id,
                tx_hash: event.tx_hash.clone(),
                block_number: event.block_number,
                category: "ProtocolStateSignificantChange".to_string(),
                score: 0.9,
                reason: format!(
                    "significant state change in {}",
                    def.family.display_name()
                ),
            });
        }

        Ok(())
    }
}
```

Significance thresholds are configurable per protocol family and modulated by behavioral phase -- in `declining` phase, smaller moves warrant attention.

---

## Autonomous Protocol Discovery

### Philosophy

The only hardcoded knowledge is a list of ~20-30 seed factory addresses -- the roots of the DeFi protocol tree. Everything else is discovered from these seeds by watching factory events.

### Seed Factories

```rust
const SEED_FACTORIES: &[(&str, &str, u64)] = &[
    ("UniswapV2Factory",        "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f", 1),
    ("UniswapV3Factory",        "0x1F98431c8aD98523631AE4a59f267346ea31F984", 1),
    ("UniswapV4PoolManager",    "0x000000000004444c5dc75cB358380D2e3dE2e8b1", 1),
    ("AavePoolAddressesProvider", "0x2f39d218133AFaB8F2B819B1066c7E434Ad94E9e", 1),
    ("CompoundComptroller",     "0x3d9819210A31b4961b30EF54bE2aeD79B9c9Cd3b", 1),
    ("CurveAddressProvider",    "0x0000000022D53366457F9d5E68Ec105046FC4383", 1),
    ("BalancerVault",           "0xBA12222222228d8Ba445958a75a0704d566BF2C8", 1),
    // + L2 equivalents: Base, Arbitrum, Optimism factories
    // ~20 total
];
```

### Factory Event Watching

When a factory's `PoolCreated` / `PairCreated` / `MarketListed` event is decoded:

1. Extract the new child contract address from the event.
2. Add to `ChainScope.watched_addresses` immediately.
3. Queue for fingerprinting + ABI resolution.
4. Add to `ProtocolRegistry` with `discovered_at_block`.
5. Emit `GolemEvent::ProtocolDiscovered`.

```rust
const FACTORY_CREATION_TOPICS: &[(&str, B256)] = &[
    ("PairCreated",        keccak256("PairCreated(address,address,address,uint256)")),
    ("PoolCreated",        keccak256("PoolCreated(address,uint24,address)")),
    ("ReserveInitialized", keccak256("ReserveInitialized(address,...)")),
    // ~20 more for other protocol families
];
```

### ABI Resolution Chain

```
Unknown contract address
    |
    v
1. bytecode_hash -> BYTECODE_REGISTRY         (local, instant)
    | miss
    v
2. supportsInterface() -> ERC-165             (1 eth_call)
    |
    v
3. Sourcify API                              (~60% coverage, no key)
    | miss
    v
4. Etherscan API                             (~80% verified, optional key)
    | miss
    v
5. 4byte.directory                           (selector fragments, no key)
    | miss
    v
6. Heimdall-rs / WhatsABI bytecode analysis  (heuristic, unverified contracts)
    |
    v
 ProtocolDef constructed with whatever ABI depth was recovered
```

Partial ABIs are added to the protocol registry and start contributing event selectors to the triage log decoder immediately. Even a single resolved function selector upgrades transactions from `Unknown` to named interactions, improving curiosity scoring.

### Autonomous Subgraph Discovery

When The Graph API is configured, the discovery service queries for subgraphs by contract address after each new protocol registration:

```rust
async fn discover_subgraphs(address: Address, chain_id: u64) -> Vec<SubgraphEndpoint> {
    the_graph_client.search_subgraphs(chain_id, &address).await
        .unwrap_or_default()
        .into_iter()
        .filter(|s| s.indexes_address(&address))
        .collect()
}
```

Discovered subgraph URLs go into `ProtocolDef.subgraph_url`. Without The Graph configured, the system is fully functional -- it just lacks historical aggregates.

---

## rindexer Integration

When rindexer is configured (`golem.toml: chain.rindexer_enabled = true`), the protocol state engine generates rindexer YAML configs dynamically from discovered `ProtocolDef`s:

```rust
async fn generate_rindexer_config(protocols: &[ProtocolDef]) -> RindexerConfig {
    RindexerConfig {
        contracts: protocols.iter().filter_map(|def| {
            def.abi.as_ref().map(|abi| RindexerContract {
                name: def.id.to_string(),
                address: def.contract_address,
                abi: abi.clone(),
                start_block: def.discovered_at_block,
            })
        }).collect(),
        ..Default::default()
    }
}
```

rindexer runs as a subprocess, providing a local GraphQL endpoint for historical queries. The golem manages its lifecycle via the graceful shutdown sequence. rindexer config is regenerated at each Delta tick if new protocols were discovered since the last Delta.

---

## Emitted GolemEvents

```rust
GolemEvent::ProtocolStateUpdate {
    protocol_id: String,
    chain_id: u64,
    state_delta: serde_json::Value,  // only changed fields
    block_number: u64,
},

GolemEvent::ProtocolDiscovered {
    chain_id: u64,
    address: String,
    protocol_family: String,
    discovered_via: String,  // "factory_event" | "bytecode_match" | "abi_match"
    parent_address: Option<String>,
},
```

See [06-events-signals.md](./06-events-signals.md) for wire format and subscription categories.

---

## CorticalState Updates

| Signal | Update trigger | Value |
|--------|----------------|-------|
| `attention.active_count` | Each Gamma tick | Count of protocols with state updates in the last Gamma window |

---

## Open Questions

**Reorg handling**: When a chain reorg invalidates blocks, protocol state derived from orphaned blocks becomes incorrect. The engine needs to detect reorgs (by checking parent hashes on new headers), emit `GolemEvent::ChainReorg`, and roll back to the last canonical redb checkpoint. The redb snapshot model supports this but the rollback logic needs explicit spec before implementation.

**rindexer lifecycle**: When rindexer crashes or produces stale data, the golem should fall back to direct `eth_call` reads transparently. Fallback detection (checking rindexer's latest indexed block vs chain head) needs spec.

---

## Dependencies

```toml
[dependencies]
alloy = { version = "1", features = ["contract"] }
alloy-primitives = "1"
redb = "2"
dashmap = "6"
arc-swap = "1"
lru = "0.12"
reqwest = { version = "0.12", features = ["json"] }
serde_json = "1"
bincode = "1"
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

---

## Cross-References

- **Architecture**: [00-architecture.md](./00-architecture.md) -- Five-crate overview of chain intelligence, data flow, storage budget per chain
- **Triage**: [02-triage.md](./02-triage.md) -- Upstream: 4-stage classification pipeline whose routed events trigger state updates here
- **Chain scope**: [04-chain-scope.md](./04-chain-scope.md) -- Discovery results feed into the interest list that determines what the Golem watches
- **Generative views**: [07-generative-views.md](./07-generative-views.md) -- Protocol family classification drives auto-generated TUI views from contract ABIs
- **Stream API**: [08-stream-api.md](./08-stream-api.md) -- Hot layer DashMap backs the `/protocol/{address}` snapshot endpoints
- **Events**: [06-events-signals.md](./06-events-signals.md) -- ProtocolStateUpdate and ProtocolDiscovered GolemEvent variant definitions and wire format
- **Heartbeat**: [01-golem/02-heartbeat.md](../01-golem/02-heartbeat.md) -- Delta tick triggers rindexer config regeneration and protocol state compaction

---

## References

- Stevens, J. (2024). rindexer: A no-code EVM indexing framework. GitHub. -- *The optional cold-layer indexer that provides full event history via dynamically generated YAML configs.*
- The Graph Protocol. Subgraph Studio and Hosted Service documentation. -- *Decentralized indexing protocol; subgraph URLs are auto-discovered per protocol and stored in ProtocolDef.*
- Wood, G. (2014). Ethereum Yellow Paper. Section 4.3. -- *Defines the logsBloom filter used upstream in the witness layer for block pre-screening.*
