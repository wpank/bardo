# Local Chain Indexer

**Version:** 2.0.0
**Last Updated:** 2026-03-16

---

> **Reader orientation:** This document specifies both the local chain indexer for dev environments and the production protocol state engine for the Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) runtime (section 15). The local indexer uses Ponder + PGlite to provide subgraph-compatible GraphQL access to mirage-rs chain data with no Docker or PostgreSQL. The production protocol state engine maintains a three-layer storage model (hot DashMap for zero-latency reads, warm redb for restart recovery, optional cold layer via rindexer/The Graph) and autonomously discovers new protocols from ~20-30 seed factory addresses. See `prd2/shared/glossary.md` for full term definitions.

## Overview

The local chain indexer provides subgraph-compatible data access for the dev environment. It uses **Ponder v0.9** with **PGlite** for embedded storage -- no Docker, no PostgreSQL. A translation proxy converts The Graph-format GraphQL queries into Ponder's format, so existing data tools work transparently against the local mirage-rs chain.

---

## Architecture

```
Data Tools / Debug UI
        |
        | GraphQL (The Graph format)
        v
Translation Proxy (:42070)
        |
        | GraphQL (Ponder format)
        v
Ponder Indexer (:42069)
        |
        | RPC polling
        v
mirage-rs (:8546)
```

1. **Ponder Indexer** -- Watches mirage-rs via RPC polling, indexes events into PGlite, exposes GraphQL at port 42069.
2. **Translation Proxy** -- HTTP server at port 42070, translates The Graph query format to Ponder's format.
3. **Subgraph Routing** -- Local chain routes through the proxy instead of The Graph hosted service.

---

## Query Transformations

| The Graph | Ponder | Transformation |
| --------- | ------ | -------------- |
| `first: N` | `limit: N` | Parameter rename |
| `skip: N` | `offset: N` | Parameter rename |
| `factories(...)` | `factorys(...)` | Ponder's naive plural |
| Plural results | Unwrap `items` array | Response wrapping |
| `orderBy`, `where` | Same | Pass-through |

---

## Indexed Entities

| Entity | Source | Key Events |
| ------ | ------ | ---------- |
| Factory | V3Factory | PoolCreated |
| Pool | V3Pool (factory pattern) | Swap, Mint, Burn, Collect, Initialize |
| Token | MockERC20 | Transfer |
| Position | NonfungiblePositionManager | IncreaseLiquidity, DecreaseLiquidity, Transfer |
| Agent | IdentityRegistry | AgentRegistered, AgentUpdated |
| Reputation | ReputationRegistry | ScoreUpdated |

V4 events from the singleton PoolManager are indexed via event topic filtering.

---

## Dynamic Contract Discovery

Ponder reads the deployment manifest (`.mirage/deployment.json`) to discover contract addresses:

```typescript
export default createConfig({
  contracts: {
    UniswapV3Factory: {
      abi: v3FactoryAbi,
      address: ({ deployment }) => deployment.v3Factory,
      network: "local",
      startBlock: 0,
    },
    V3Pool: {
      abi: v3PoolAbi,
      network: "local",
      factory: {
        address: ({ deployment }) => deployment.v3Factory,
        event: "PoolCreated(address,address,uint24,int24,address)",
        parameter: "pool",
      },
    },
    // ...
  },
});
```

---

## Lifecycle

1. mirage-rs starts (fresh or fork).
2. Deployment runs (if fresh), producing deployment manifest.
3. Ponder spawned as child process via `npx ponder dev`.
4. Translation proxy starts at port 42070.
5. Ponder indexes from block 0 (or last checkpoint).
6. Data tools and debug UI query via GraphQL.

### Non-Fatal

If Ponder fails to start (port conflict, PGlite corruption), the dev environment continues without it. Data tools return subgraph errors rather than crashing the stack.

### Fresh Mode

Wipes `.ponder/` directory to force re-index from block 0.

---

## Performance

| Metric | Value |
| ------ | ----- |
| Initial index (500 blocks, full deploy) | ~3 seconds |
| Incremental index per block | < 50ms |
| GraphQL query (simple) | < 10ms |
| GraphQL query (aggregation) | < 50ms |
| PGlite storage (after seed) | ~5MB |
| Proxy overhead | < 2ms per request |

---

## Limitations

- V2 not indexed (direct RPC fallback).
- No real-time GraphQL subscriptions (polls at 2s intervals).
- Single chain only (the local mirage-rs instance).
- No historical aggregations (raw events + derived state only).
- In live fork mode with `--follow`, the indexer must keep up with incoming blocks. If mirage-rs replays blocks faster than Ponder indexes, the indexer will lag.

---

## Protocol State Engine

Beyond the local chain indexer (Ponder-based, for dev/test), the production golem runtime uses a dedicated protocol state engine that maintains a live, always-accurate model of every DeFi protocol the golem knows about. State updates on on-chain events, not polling. Reading state is zero-latency through a lock-free DashMap.

### Three-Layer Storage Model

**Hot layer (in-memory).** An `Arc<DashMap<ProtocolKey, ProtocolState>>` updated on every relevant triage event. Reads are lock-free on the fast path. Writes use DashMap's shard-level locking -- concurrent reads on other shards are unaffected. The 60fps TUI render loop and bardo-stream-api read this without blocking the write path.

```rust
pub struct ProtocolStateEngine {
    hot: Arc<DashMap<ProtocolKey, ProtocolState>>,
    warm: Arc<WarmStore>,
    registry: Arc<ProtocolRegistry>,
    fabric: EventFabricHandle,
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

**Warm layer (redb on disk).** Persisted snapshots for restart recovery. On restart, the hot layer seeds from redb -- avoiding cold-start resync from chain. History stored as delta sequences rather than full snapshots per block. A pool's `sqrtPriceX96` (U256, 20 bytes) changes by small amounts per block; delta + varint encoding achieves 10-100x compression vs. raw storage. Retention: 30 days for deltas, unlimited for snapshots.

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

**Cold layer (optional, external).** rindexer (Stevens, 2024) or The Graph provides full event history when configured. Not held in memory -- queried on demand for historical analysis.

### ProtocolDef

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
    OnEvent(Vec<B256>),
    OnBlock(u64),
    Polling(Duration),
    Hybrid { events: Vec<B256>, full_refresh_blocks: u64 },
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

### ProtocolStateReader Trait

New protocol families are added by implementing this trait:

```rust
#[async_trait]
pub trait ProtocolStateReader: Send + Sync {
    async fn read_state(
        &self,
        address: Address,
        provider: &dyn Provider,
    ) -> Result<ProtocolState>;

    fn event_triggers(&self) -> &[B256];
    fn protocol_family(&self) -> ProtocolFamily;
    fn update_trigger(&self) -> UpdateTrigger;
}
```

Example for UniswapV3Pool -- reads four storage slots in parallel:

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
            block_number: 0,
            timestamp_ms: now_ms(),
        }))
    }

    fn event_triggers(&self) -> &[B256] {
        &[UNISWAP_V3_SWAP_TOPIC, UNISWAP_V3_MINT_TOPIC, UNISWAP_V3_BURN_TOPIC]
    }

    fn protocol_family(&self) -> ProtocolFamily {
        ProtocolFamily::UniswapV3Pool
    }

    fn update_trigger(&self) -> UpdateTrigger {
        UpdateTrigger::Hybrid {
            events: vec![UNISWAP_V3_SWAP_TOPIC, UNISWAP_V3_MINT_TOPIC, UNISWAP_V3_BURN_TOPIC],
            full_refresh_blocks: 100,
        }
    }
}
```

### State Update Flow

When `bardo-triage` (the 4-stage transaction classification pipeline that scores on-chain events by relevance) routes a triage event to the protocol state engine:

```rust
impl ProtocolStateEngine {
    pub async fn handle_triage_event(&self, event: TriageEvent) -> Result<()> {
        let key = (event.chain_id, event.protocol_address);
        let def = match self.registry.get(&key) {
            Some(d) => d,
            None => return Ok(()),
        };

        if !def.update_trigger.matches(&event.log_topic) {
            return Ok(());
        }

        let new_state = def.state_reader
            .read_state(def.contract_address, &*self.provider)
            .await?;

        let prev = self.hot.get(&key);
        let delta = compute_delta(prev.as_deref(), &new_state);

        self.hot.insert(key, new_state.clone());

        self.warm.write_delta(
            event.chain_id,
            def.contract_address,
            event.block_number,
            &delta,
        )?;

        self.fabric.emit(GolemEvent::ProtocolStateUpdate {
            protocol_id: def.id.to_string(),
            chain_id: event.chain_id,
            state_delta: serde_json::to_value(&delta)?,
            block_number: event.block_number,
        });

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

---

## Autonomous Protocol Discovery

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
2. Add to `ChainScope.watched_addresses` (the dynamic attention model that determines which on-chain addresses the Golem monitors) immediately.
3. Queue for fingerprinting + ABI resolution.
4. Add to `ProtocolRegistry` with `discovered_at_block`.
5. Emit `GolemEvent::ProtocolDiscovered` (GolemEvent is the typed, serializable event enum that flows through the Event Fabric broadcast channel to all Golem subsystems).

### ABI Resolution Pipeline

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

Partial ABIs are added to the protocol registry and start contributing event selectors to the triage log decoder immediately. Even a single resolved function selector upgrades transactions from `Unknown` to named interactions.

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

### rindexer Integration

When rindexer is configured (`golem.toml: chain.rindexer_enabled = true`), the protocol state engine generates rindexer YAML configs dynamically from discovered `ProtocolDef`s. rindexer runs as a subprocess, providing a local GraphQL endpoint for historical queries. Config is regenerated at each Delta tick if new protocols were discovered.

---

## Open Questions

**Reorg handling**: When a chain reorg invalidates blocks, protocol state derived from orphaned blocks becomes incorrect. The engine needs to detect reorgs (by checking parent hashes on new headers), emit `GolemEvent::ChainReorg`, and roll back to the last canonical redb checkpoint.

**rindexer lifecycle**: When rindexer crashes or produces stale data, the golem should fall back to direct `eth_call` reads transparently. Fallback detection (checking rindexer's latest indexed block vs chain head) needs spec.
