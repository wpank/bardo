# golem-chain

Chain interaction layer for Bardo Golems. Covers the static chain registry, cached Alloy providers, ERC-8004 agent identity, the Warden time-delay safety mechanism, and local EVM simulation via revm. No trading logic, no risk scoring — that belongs in higher crates.

## Exports

```rust
pub use config::{ChainConfig, ChainId, ChainRegistry, ContractAddresses};
pub use error::ChainError;
pub use identity::{AgentIdentity, Capability8004, Erc8004Registry, ServiceEndpoint};
pub use provider::{CacheKey, CachedValue, ChainProvider};
pub use revm_sim::{RevmSimulator, SimRequest, SimResult};
pub use warden::{ActionType, Warden, WardenAction, WardenStatus};
```

## ChainRegistry

Static registry of 12 supported networks. `ChainId` is a `u64` alias. `ChainRegistry` and `ChainConfig` hold RPC endpoints, block times, and `ContractAddresses` for each network.

## ERC-8004 Identity

`Erc8004Registry` is a read-only client for the agent identity registry deployed at `0x8004A818BFB912233c491871b3d84c89A494BD9e` on Ethereum L1. Write operations (register, update) are in a separate plan.

```rust
let registry = Erc8004Registry::new(provider);

// Fetch full identity
let identity: Option<AgentIdentity> = registry.get_identity(address).await?;

// Check a single capability
let is_filler: bool = registry.has_capability(address, &Capability8004::UniswapXFiller).await?;

// Paginated list
let agents: Vec<AgentIdentity> = registry.list_identities(0, 100).await?;
```

`Capability8004` variants:

```rust
pub enum Capability8004 {
    Trading,
    LiquidityProvider,
    Lending,
    CrossChainRouting,
    VaultManagement,
    UniswapXFiller,
    Custom(String),
}
```

`AgentIdentity` carries address, registered capabilities, named `ServiceEndpoint` URLs (MCP server URLs, API endpoints), an IPFS metadata CID for discovery, and the last-updated block number. The `ServiceEndpoint` map lets other agents locate an agent's tools without off-chain coordination.

ERC-8004 is the anchor standard in Bardo's coordination stack. It ties together ERC-8001 (N-party consent), ERC-8033 (oracle councils), and ERC-8183 (job escrow). The 6 built-in `Capability8004` variants cover common DeFi roles; `Custom(String)` extends the set for domain-specific capabilities without a contract upgrade.

## Warden

Every state-mutating on-chain action must pass through the Warden. Announce first, wait the mandatory delay, then execute. There is no way to skip the queue.

```rust
let mut warden = Warden::new();

// Register with default delay for this action type
let id: Uuid = warden.announce(ActionType::VaultRebalance, chain_id, Some("rebalance ETH/USDC".into()));

// Or with a custom delay
let id = warden.announce_with_delay(ActionType::OrderCancel, Duration::from_secs(60), chain_id, None);

// Poll to advance status and collect newly-ready IDs
let ready: Vec<Uuid> = warden.poll();

// Cancel before execution
warden.cancel(id)?;

// Mark as executed after broadcast (only valid from Ready state)
warden.mark_executed(id)?;
```

`WardenStatus` state machine: `Announced → Waiting → Ready → Executed | Cancelled`. `poll()` advances `Announced → Waiting` on the first call, then `Waiting → Ready` once the delay elapses.

Default delays:

| ActionType             | Delay  |
|------------------------|--------|
| PoolParameterUpdate    | 3600s  |
| VaultRebalance         | 1800s  |
| OrderCancel            | 300s   |
| LargeSwap { .. }       | 600s   |
| CrossChainBridge       | 7200s  |
| Custom(_)              | 300s   |

`WardenAction` carries the full context: `id`, `action_type`, `delay`, `announced_at`, `status`, `description`, `chain_id`.

`Warden::prune(max_age)` removes terminal actions (Executed or Cancelled) older than `max_age`. Call this periodically to bound memory.

## RevmSimulator

Local EVM execution without broadcasting anything on-chain.

```rust
let sim = RevmSimulator::new(provider);

let result: SimResult = sim.simulate(SimRequest {
    chain_id: 1,
    to: contract_address,
    calldata: encoded_call,
    sender: caller_address,
    value: U256::ZERO,
}).await?;

println!("success: {}, gas: {}", result.success, result.gas_used);
```

`SimResult` has `success`, `return_data`, `gas_used`, and the state changes the call would produce.

## Usage

```toml
[dependencies]
golem-chain = { path = "../../crates/golem-chain" }
```
