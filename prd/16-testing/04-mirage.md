# Mainnet Mirage: Live Fork Infrastructure for Golem Testing [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-09
**Package:** `packages/mirage/` (`@bardo/mirage`)

> A mirage is an environment where an agent operates as if on mainnet — prices move, pools rebalance, oracles update, positions get liquidated — except the agent's own transactions are the only ones that aren't real.

> **Reader orientation:** This document specifies Mirage, the live fork infrastructure that lets Golems (mortal autonomous agents) operate against a continuously-updated local copy of mainnet state. It belongs to Section 16 (Testing) and covers the Block Follower, Transaction Replayer, Protocol Simulators, and per-protocol replay strategies. The key concept is that unlike static Anvil forks, Mirage streams new mainnet state so the Golem's Heartbeat (9-step decision cycle) sees realistic market dynamics. See `prd2/shared/glossary.md` for full term definitions.

---

## Document Map

| Section             | Topic                                                            |
| ------------------- | ---------------------------------------------------------------- |
| Overview            | Problem statement, architecture, design principles               |
| Core Architecture   | Block Follower, Transaction Replayer, Protocol Simulators        |
| Protocol Coverage   | Per-protocol replay and simulation strategies                    |
| Chain Support       | Ethereum mainnet and Base L2 specifics                           |
| Configuration       | Filtering, performance tuning, chain selection                   |
| Integration         | How Mirage connects to @bardo/dev, @bardo/testnet, Golem runtime |
| Implementation Plan | Phased build-out with milestones                                 |
| Contract Reference  | Mainnet addresses for all watched contracts                      |

---

## The Problem

Standard Anvil forks create a **point-in-time snapshot** of mainnet state. After the fork block, no new state enters the local chain. For a Golem heartbeat loop, this means:

| What Breaks                                 | Why It Matters                                   | Impact on Golem                         |
| ------------------------------------------- | ------------------------------------------------ | --------------------------------------- |
| Chainlink oracle prices freeze              | Golem sees stale ETH/USD, triggers wrong trades  | Strategy executes on fiction            |
| Uniswap pool reserves stop changing         | No swaps, no fee accrual, no tick crossings      | LP rebalancing has no stimulus          |
| Morpho/Aave interest rate accumulators stop | Supply/borrow rates frozen at fork block         | Yield calculations are wrong            |
| No new deposits/withdrawals to vaults       | Golem is the only depositor/manager in the world | Share price dynamics are unrealistic    |
| No MEV, no liquidations, no governance      | The adversarial environment disappears entirely  | Safety layers go untested               |
| Gas prices are static                       | No baseFee fluctuation                           | Gas circuit breakers never trigger      |
| Block timestamps only advance on local tx   | Time-dependent logic (vesting, epochs) stalls    | Anything using `block.timestamp` drifts |

A Golem tested against a static fork is a Golem tested in a vacuum. The heartbeat's 11 probes (price delta, position health, regime detection, etc.) have nothing to detect. The System 1 → System 2 escalation path never fires. The PLAYBOOK.md (the agent's evolving strategy document) never evolves. The Grimoire collects no meaningful episodes.

**Mainnet Mirage** solves this by continuously replaying mainnet transactions into the local Anvil fork, creating a living environment where the Golem coexists with real-world market activity.

---

## Design Principles

1. **Self-hosted, no external services.** The only external dependency is an RPC endpoint (Alchemy, QuickNode, or self-hosted node). No Tenderly, no Shadow.xyz, no hosted APIs. Everything runs on the operator's machine.

2. **Anvil-native.** Built on Anvil's existing RPC methods (`anvil_impersonateAccount`, `anvil_setStorageAt`, `evm_setNextBlockTimestamp`, `evm_mine`). No custom EVM modifications. Any Foundry installation works.

3. **Selective replay, not full replay.** Base produces ~7,500 transactions per block. Replaying everything is wasteful and slow. Mirage filters for transactions that affect contracts the Golem interacts with. The operator configures which protocols, pools, and addresses matter.

4. **Graceful divergence.** Once the Golem acts on the local fork, state diverges from mainnet. Replayed transactions will start failing (different reserves, different balances). This is expected and correct — the Golem's actions _should_ change the environment. Failed replays are logged but do not halt the system.

5. **Dual-chain.** First-class support for both Ethereum mainnet and Base. Chain-specific adapters handle differences in block time (12s vs ~2s), transaction format (L1 vs L2 with L1 data), and contract addresses.

6. **Composable with existing dev tooling.** Mirage is a layer on top of `@bardo/testnet` (AnvilManager) and `@bardo/dev` (UniswapDeployment). It doesn't replace them — it feeds live data into the same Anvil instance they manage.

7. **Observable.** Every replayed transaction, every simulation, every failure is logged as structured JSONL. The Golem's telemetry system can ingest Mirage events alongside heartbeat events for unified debugging.

---

## Goals

| ID  | Goal                                                                              | Rationale                                                     |
| --- | --------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| G1  | Continuously replay filtered mainnet transactions into a local Anvil fork         | Core value proposition: living test environment               |
| G2  | Support Ethereum mainnet and Base L2 as source chains                             | Golems operate on both; testing must cover both               |
| G3  | Protocol-specific simulators for oracle prices, lending rates, and gas            | Some state changes are better simulated than replayed         |
| G4  | Filter transactions by contract address, function selector, and protocol category | Performance: only replay what matters to the Golem's strategy |
| G5  | Integrate with `@bardo/testnet` AnvilManager and `@bardo/dev` UniswapDeployment   | Zero friction for existing dev workflows                      |
| G6  | Expose as both a CLI tool and a programmatic TypeScript API                       | CLI for manual testing; API for SwarmRunner and CI            |
| G7  | Structured JSONL logging of all replay activity                                   | Post-hoc debugging and Golem episode correlation              |
| G8  | Configurable via a single `mirage.config.ts` file                                 | Operator-controlled protocol coverage and tuning              |
| G9  | Handle state divergence gracefully (log, skip, continue)                          | Golem actions legitimately change local state                 |
| G10 | Sub-5-second latency from mainnet block confirmation to local replay completion   | Golem probes must see fresh state                             |

## Non-Goals

| ID  | Non-Goal                                                     | Reason                                                                            |
| --- | ------------------------------------------------------------ | --------------------------------------------------------------------------------- |
| NG1 | Full block replay (every transaction in every block)         | Too slow; Base does ~7,500 tx/block. Selective replay is sufficient.              |
| NG2 | Consensus-layer simulation (slots, attestations, validators) | Golems operate at the execution layer only                                        |
| NG3 | MEV simulation (sandwich, frontrun, backrun)                 | Phase 2+ concern. Phase 1 focuses on state accuracy, not adversarial ordering.    |
| NG4 | Historical replay (backfill past blocks)                     | Mirage is forward-looking. For backtesting, use `forge test --fork-block-number`. |
| NG5 | Cross-chain message relay (L1↔L2 bridge messages)            | Out of scope for v1. Each chain is an independent mirage.                         |
| NG6 | Production deployment or use with real funds                 | Mirage is a dev/test tool. `NODE_ENV=development` guard.                          |

---

## High-Level Architecture

```
                                 ┌───────────────────────────┐
                                 │   Mainnet RPC Endpoint    │
                                 │  (Alchemy / QuickNode /   │
                                 │   self-hosted node)       │
                                 └─────────┬─────────────────┘
                                           │
                            ┌──────────────┼──────────────┐
                            │              │              │
                            ▼              ▼              ▼
                    ┌──────────┐  ┌──────────────┐  ┌──────────────┐
                    │  Block   │  │   Mempool    │  │   Oracle     │
                    │ Follower │  │  Listener    │  │   Poller     │
                    │ (HTTP)   │  │  (WebSocket) │  │  (HTTP)      │
                    └────┬─────┘  └──────┬───────┘  └──────┬───────┘
                         │               │                 │
                         ▼               ▼                 ▼
                   ┌─────────────────────────────────────────────┐
                   │              Transaction Router              │
                   │                                             │
                   │  ┌─────────┐ ┌──────────┐ ┌─────────────┐  │
                   │  │ Address │ │ Function │ │  Protocol   │  │
                   │  │ Filter  │ │ Selector │ │  Classifier │  │
                   │  │         │ │  Filter  │ │             │  │
                   │  └─────────┘ └──────────┘ └─────────────┘  │
                   └──────────────────┬──────────────────────────┘
                                      │
                         ┌────────────┼────────────┐
                         │            │            │
                         ▼            ▼            ▼
                  ┌────────────┐ ┌──────────┐ ┌──────────────┐
                  │ Transaction│ │ Storage  │ │  Timestamp   │
                  │  Replayer  │ │ Injector │ │  Advancer    │
                  │            │ │          │ │              │
                  │ impersonate│ │ setStore │ │ setNextBlock │
                  │ + sendTx   │ │ at slots │ │  Timestamp   │
                  └─────┬──────┘ └────┬─────┘ └──────┬───────┘
                        │             │              │
                        ▼             ▼              ▼
                  ┌──────────────────────────────────────────┐
                  │            Anvil Fork Instance            │
                  │         (localhost:8545 / 8546)           │
                  │                                          │
                  │  ┌──────────────────────────────────┐    │
                  │  │  Golem Contracts (deployed local) │    │
                  │  │  AgentVaultFactory, PolicyCage,   │    │
                  │  │  VaultReputationEngine             │    │
                  │  └──────────────────────────────────┘    │
                  │                                          │
                  │  ┌──────────────────────────────────┐    │
                  │  │  Mainnet Contracts (from fork)    │    │
                  │  │  Uniswap V2/V3/V4, Morpho Blue,  │    │
                  │  │  Aave V3, Chainlink Feeds,        │    │
                  │  │  Permit2, ERC-8004 Registry       │    │
                  │  └──────────────────────────────────┘    │
                  └──────────────────────────────────────────┘
                                      ▲
                                      │
                  ┌──────────────────────────────────────────┐
                  │           Golem Runtime                   │
                  │  Heartbeat FSM → Sanctum Tools → Anvil   │
                  └──────────────────────────────────────────┘
```

---

## Core Components

### 1. Block Follower

The Block Follower polls the mainnet RPC for new blocks and dispatches their transactions to the Transaction Router.

```typescript
// packages/mirage/src/block-follower.ts

interface BlockFollowerConfig {
  /** Mainnet RPC URL (HTTP). Archive node recommended but not required. */
  rpcUrl: string;
  /** Chain to follow */
  chain: "ethereum" | "base";
  /** Poll interval in ms. Default: 1000 for Base (2s blocks), 3000 for Ethereum (12s blocks) */
  pollIntervalMs?: number;
  /** How many blocks behind tip to start. Default: 0 (latest) */
  startOffset?: number;
  /** Max blocks to process per poll cycle (burst catch-up). Default: 5 */
  maxBlocksPerCycle?: number;
  /** Include full transaction objects (not just hashes). Must be true. */
  includeTransactions: true;
}

interface BlockFollowerEvents {
  "block:received": (block: Block, chain: ChainId) => void;
  "block:processed": (
    blockNumber: bigint,
    txCount: number,
    replayedCount: number,
  ) => void;
  "block:skipped": (blockNumber: bigint, reason: string) => void;
  error: (error: Error) => void;
  "lag:warning": (lagBlocks: number) => void;
}
```

**Implementation details:**

- Uses `eth_getBlockByNumber` with `true` for full transaction objects
- Tracks `lastProcessedBlock` persistently (writes to `.mirage/cursor-{chain}.json`)
- On restart, resumes from cursor position or re-forks if gap is too large (configurable `maxGapBeforeRefork`)
- Emits `lag:warning` if local processing falls more than 10 blocks behind tip
- For Base: polls every 1s to stay within the ~2s block time
- For Ethereum: polls every 3s (12s block time gives ample margin)
- Uses a simple HTTP polling loop, not WebSocket subscriptions (more reliable across RPC providers, reconnection is automatic)

### 2. Transaction Router

The Transaction Router receives blocks from the Block Follower, classifies each transaction, and dispatches to the appropriate handler (Replayer or Storage Injector).

```typescript
// packages/mirage/src/router.ts

interface TransactionRouterConfig {
  /** Watched contract addresses (lowercase). Transactions TO these addresses are replayed. */
  watchedContracts: Set<Address>;
  /** Watched FROM addresses. Transactions FROM these are replayed regardless of target. */
  watchedSenders?: Set<Address>;
  /** Protocol classifiers: map function selectors to protocol categories */
  protocolClassifiers: ProtocolClassifier[];
  /** Maximum transactions to replay per block. Default: 100 */
  maxTxPerBlock?: number;
  /** Skip transactions with value only (pure ETH transfers). Default: true */
  skipPureTransfers?: boolean;
}

interface ProtocolClassifier {
  /** Human-readable name */
  name: string;
  /** Contract addresses belonging to this protocol */
  addresses: Address[];
  /** Function selectors to match (4-byte hex). Empty = match all calls to these addresses. */
  selectors?: Hex[];
  /** Handling strategy */
  strategy: "replay" | "simulate" | "storage-inject" | "skip";
  /** Priority (lower = processed first within a block). Default: 100 */
  priority?: number;
}
```

**Routing logic (per transaction):**

```
1. Is tx.to in watchedContracts?
   ├── YES → Classify by protocol
   │         ├── strategy = 'replay'          → Send to Transaction Replayer
   │         ├── strategy = 'simulate'        → Skip (handled by Protocol Simulators)
   │         ├── strategy = 'storage-inject'  → Send to Storage Injector
   │         └── strategy = 'skip'            → Log and skip
   └── NO  → Is tx.from in watchedSenders?
             ├── YES → Replay (track whale/counterparty behavior)
             └── NO  → Skip
```

### 3. Transaction Replayer

The core replay engine. Impersonates the original sender on Anvil and re-executes the transaction.

```typescript
// packages/mirage/src/replayer.ts

interface ReplayerConfig {
  /** Anvil RPC URL */
  anvilRpcUrl: string;
  /** Use anvil_autoImpersonateAccount (simpler) or per-tx impersonation (safer) */
  impersonationMode: "auto" | "per-tx";
  /** Continue on replay failure. Default: true */
  continueOnFailure: boolean;
  /** Maximum gas to allow per replayed transaction. Default: 30_000_000 */
  maxGasPerTx: bigint;
  /** Fund impersonated accounts with ETH if they lack gas. Default: true */
  autoFundSenders: boolean;
  /** Amount of ETH to fund (in wei). Default: 10 ETH */
  autoFundAmount: bigint;
}

interface ReplayResult {
  originalTxHash: Hex;
  localTxHash: Hex | null;
  status: "success" | "reverted" | "skipped" | "error";
  gasUsed: bigint | null;
  error?: string;
  /** How much local state diverged from expected (e.g., different revert reason) */
  divergenceNote?: string;
}
```

**Replay sequence per transaction:**

```typescript
async function replayTransaction(tx: Transaction): Promise<ReplayResult> {
  // 1. Ensure sender has ETH for gas on the local fork
  if (autoFundSenders) {
    const balance = await getBalance(tx.from);
    if (balance < autoFundAmount) {
      await anvil_setBalance(tx.from, autoFundAmount);
    }
  }

  // 2. Impersonate the sender
  if (impersonationMode === "per-tx") {
    await anvil_impersonateAccount(tx.from);
  }

  // 3. Send the transaction with original calldata
  //    Note: We do NOT preserve the original nonce — Anvil manages nonces.
  //    We do NOT preserve gas price — let Anvil use its local baseFee.
  const localTxHash = await eth_sendTransaction({
    from: tx.from,
    to: tx.to,
    data: tx.input,
    value: toHex(tx.value),
    gas: toHex(min(tx.gas, maxGasPerTx)),
  });

  // 4. Wait for receipt
  const receipt = await eth_getTransactionReceipt(localTxHash);

  // 5. Stop impersonating
  if (impersonationMode === "per-tx") {
    await anvil_stopImpersonatingAccount(tx.from);
  }

  // 6. Return result
  return {
    originalTxHash: tx.hash,
    localTxHash,
    status: receipt.status === "0x1" ? "success" : "reverted",
    gasUsed: BigInt(receipt.gasUsed),
  };
}
```

**Nonce handling:** Anvil auto-manages nonces for impersonated accounts. We do not attempt to match mainnet nonce sequences because state divergence makes that impossible after the Golem's first local transaction. This is fine — the goal is state effect reproduction, not exact transaction identity.

**Gas handling:** Replayed transactions use Anvil's local baseFee, not mainnet gas prices. The `GasSimulator` component (see Protocol Simulators) separately updates Anvil's baseFee to track mainnet gas conditions.

### 4. Storage Injector

For state that is better injected directly rather than replayed (oracle prices, interest rate accumulators, gas baseFee), the Storage Injector writes values directly into Anvil's state trie using `anvil_setStorageAt`.

```typescript
// packages/mirage/src/storage-injector.ts

interface StorageInjection {
  /** Target contract address */
  address: Address;
  /** Storage slot (32-byte hex) */
  slot: Hex;
  /** New value (32-byte hex) */
  value: Hex;
  /** Human-readable description */
  description: string;
}

async function injectStorageValues(
  injections: StorageInjection[],
): Promise<void> {
  for (const injection of injections) {
    await fetch(anvilRpcUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method: "anvil_setStorageAt",
        params: [injection.address, injection.slot, injection.value],
        id: 1,
      }),
    });
  }
}
```

### 5. Timestamp Advancer

Keeps the local fork's `block.timestamp` in sync with mainnet block timestamps.

```typescript
async function advanceTimestamp(mainnetBlock: Block): Promise<void> {
  // Set the next block's timestamp to match the mainnet block
  await anvil_setNextBlockTimestamp(mainnetBlock.timestamp);

  // Mine a block to apply the timestamp
  // (only if no transactions were replayed — if they were, mining already happened)
  await evm_mine();
}
```

This is critical for:

- Chainlink heartbeat freshness checks
- Morpho/Aave time-weighted interest accrual
- Vault fee accrual (`report()` uses timestamp deltas)
- Golem heartbeat tick scheduling
- Any contract using `block.timestamp` for epoch/period logic

---

## Protocol Coverage

Each DeFi protocol the Golem interacts with has a dedicated `ProtocolAdapter` that determines how its mainnet activity is reflected locally. The adapter decides between three strategies:

| Strategy               | When to Use                                                                          | Example                                                  |
| ---------------------- | ------------------------------------------------------------------------------------ | -------------------------------------------------------- |
| **Transaction Replay** | When the full execution side-effects matter (state changes, events, callback chains) | Uniswap swaps, Morpho deposits, vault interactions       |
| **Storage Injection**  | When only a specific value matters and replaying the tx is wasteful or fragile       | Oracle price updates, gas baseFee, interest accumulators |
| **Skip**               | When the protocol activity doesn't affect the Golem's strategy                       | Irrelevant token transfers, NFT mints, governance votes  |

### Protocol Adapter: Uniswap

The most important protocol for Golem testing. Covers V2, V3, V4, Universal Router, and Permit2.

```typescript
// packages/mirage/src/protocols/uniswap.ts

export const uniswapAdapter: ProtocolAdapter = {
  name: "uniswap",
  chains: ["ethereum", "base"],

  getWatchedContracts(chain: ChainId): WatchedContract[] {
    const addresses = UNISWAP_ADDRESSES[chain];
    return [
      // V3 SwapRouter — most Golem swap volume
      {
        address: addresses.v3SwapRouter,
        selectors: [
          "0x414bf389", // exactInputSingle
          "0xc04b8d59", // exactInput
          "0xdb3e2198", // exactOutputSingle
          "0xf28c0498", // exactOutput
        ],
        strategy: "replay",
        priority: 10,
      },
      // V3 NonfungiblePositionManager — LP operations
      {
        address: addresses.v3PositionManager,
        selectors: [
          "0x88316456", // mint
          "0x219f5d17", // increaseLiquidity
          "0x0c49ccbe", // decreaseLiquidity
          "0xfc6f7865", // collect
        ],
        strategy: "replay",
        priority: 20,
      },
      // V4 PoolManager — V4 swaps and LP
      {
        address: addresses.v4PoolManager,
        selectors: [], // All calls
        strategy: "replay",
        priority: 10,
      },
      // V4 PositionManager
      {
        address: addresses.v4PositionManager,
        selectors: [],
        strategy: "replay",
        priority: 20,
      },
      // Universal Router — multicall swaps
      {
        address: addresses.universalRouter,
        selectors: [
          "0x3593564c", // execute
          "0x24856bc3", // execute (with deadline)
        ],
        strategy: "replay",
        priority: 10,
      },
      // Permit2 — approval state matters for Golem's own approvals
      {
        address: addresses.permit2,
        selectors: [],
        strategy: "replay",
        priority: 50,
      },
    ];
  },

  /** Optional: filter by specific pool addresses the Golem trades */
  getWatchedPools?(strategy: GolemStrategy): Address[] {
    // Parse STRATEGY.md for target pool addresses
    return strategy.targetPools;
  },
};
```

**What Uniswap replay gives the Golem:**

- Pool `sqrtPriceX96` and `tick` change with real swaps
- Liquidity distributions shift as LPs add/remove
- Fee accrual happens on the Golem's own LP positions when others swap through the pool
- V4 hook callbacks fire on replayed swaps (tests hook behavior with real traffic patterns)
- Permit2 approval state stays realistic

**Pool-level filtering (recommended):** Rather than replaying all Uniswap transactions, the operator specifies the pool addresses from the Golem's `STRATEGY.md`. Mirage replays only swaps that route through those pools. This dramatically reduces replay volume while maintaining full fidelity for the Golem's operating environment.

```typescript
// In mirage.config.ts:
uniswap: {
  // Only replay swaps through these pools
  watchedPools: [
    '0x...WETH-USDC-500',   // Main trading pool
    '0x...WETH-USDC-3000',  // Backup pool
    '0x...cbETH-WETH-500',  // Staking derivative pool
  ],
  // Also replay all V4 PoolManager calls (Golem manages V4 hooks)
  replayAllV4: true,
}
```

### Protocol Adapter: Chainlink Oracles

Oracle freshness is the single most critical state to maintain. Stale oracles cause the Golem to make decisions on wrong prices.

**Strategy: Storage Injection** (not transaction replay)

Replaying the actual Chainlink keeper transactions is fragile — they use complex multi-sig and OCR (Off-Chain Reporting) mechanics that are difficult to impersonate correctly. Instead, we read the latest answer directly from mainnet and inject it into the local fork's oracle storage.

```typescript
// packages/mirage/src/protocols/chainlink.ts

interface ChainlinkFeed {
  /** Human-readable name */
  name: string;
  /** Aggregator proxy address */
  proxy: Address;
  /** Underlying aggregator (resolved at init) */
  aggregator?: Address;
  /** Update frequency: how often to poll mainnet (ms) */
  pollIntervalMs: number;
}

// Storage layout for Chainlink AccessControlledOffchainAggregator:
// Slot 0x01: s_hotVars (contains latestAggregatorRoundId)
// Dynamic mapping for round data: keccak256(roundId . 0x2B) → answer
// We use a simpler approach: read via the proxy, then write via setStorageAt

export class ChainlinkSimulator {
  private feeds: ChainlinkFeed[];
  private mainnetClient: PublicClient;
  private anvilClient: TestClient;

  async syncFeed(feed: ChainlinkFeed): Promise<void> {
    // 1. Read latest round data from mainnet
    const [roundId, answer, startedAt, updatedAt, answeredInRound] =
      await this.mainnetClient.readContract({
        address: feed.proxy,
        abi: aggregatorV3InterfaceABI,
        functionName: "latestRoundData",
      });

    // 2. Resolve the underlying aggregator address
    const aggregator = await this.mainnetClient.readContract({
      address: feed.proxy,
      abi: aggregatorProxyABI,
      functionName: "aggregator",
    });

    // 3. Compute the storage slot for the latest round's answer
    //    In AccessControlledOffchainAggregator, transmissions are stored as:
    //    mapping(uint32 => Transmission) at slot 44 (0x2C)
    //    Transmission { int192 answer; uint64 timestamp }
    const roundSlot = keccak256(
      encodePacked(["uint32", "uint256"], [roundId & 0xffffffffn, 44n]),
    );

    // 4. Pack the Transmission struct: answer (int192) | timestamp (uint64)
    const packedValue = encodeAbiParameters(
      [{ type: "int192" }, { type: "uint64" }],
      [answer, updatedAt],
    );

    // 5. Write to Anvil
    await this.anvilClient.setStorageAt({
      address: aggregator,
      slot: roundSlot,
      value: packedValue,
    });

    // 6. Update the hot vars to reflect the new latest round
    //    Slot 0x01: s_hotVars contains latestAggregatorRoundId in lower 32 bits
    await this.updateHotVars(aggregator, roundId);
  }

  async syncAll(): Promise<void> {
    await Promise.all(this.feeds.map((feed) => this.syncFeed(feed)));
  }
}
```

**Default feeds (Base):**

| Feed      | Proxy Address                                | Poll Interval |
| --------- | -------------------------------------------- | ------------- |
| ETH/USD   | `0x71041dddad3595F9CEd3DcCFBe3D1F4b0a16Bb70` | 5s            |
| USDC/USD  | `0x7e860098F58bBFC8648a4311b374B1D669a2bc6B` | 30s           |
| cbETH/ETH | `0x806b4Ac04501c29769051e42783cF04dCE41440b` | 30s           |
| WBTC/USD  | `0xCCADC697c55bbB68dc5bCdf8d3CBe83CdD4E071E` | 30s           |
| DAI/USD   | `0x591e79239a7d679378eC8c847e5038150364C78F` | 60s           |

**Default feeds (Ethereum):**

| Feed     | Proxy Address                                | Poll Interval |
| -------- | -------------------------------------------- | ------------- |
| ETH/USD  | `0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419` | 5s            |
| USDC/USD | `0x8fFfFfd4AfB6115b954Bd326cbe7B4BA576818f6` | 30s           |
| WBTC/BTC | `0xfdFD9C85aD200c506Cf9e21F1FD8dd01932FBB23` | 30s           |
| DAI/USD  | `0xAed0c38402a5d19df6E4c03F4E2DceD6e29c1ee9` | 60s           |

**Why storage injection over tx replay for oracles:**

- Chainlink OCR transmissions require multi-party signature verification that cannot be impersonated
- The only thing the Golem cares about is the answer value and timestamp
- Storage injection is ~50x faster than replaying the full OCR round
- It always succeeds (no state-dependent revert risk)
- The Golem's `price_delta` probe immediately sees the updated price

### Protocol Adapter: Morpho Blue

Morpho Blue is a core yield source for Golem vaults via `MorphoSupplyAdapter`.

**Strategy: Hybrid (replay + storage injection)**

- **Replay**: `supply()`, `withdraw()`, `borrow()`, `repay()`, `liquidate()` calls to Morpho Blue. These change market utilization ratios, which affect supply APY.
- **Storage Inject**: Interest rate accumulator (`totalSupplyAssets`, `totalBorrowAssets`, `lastUpdate`) can be injected directly if replay lags behind.

```typescript
// packages/mirage/src/protocols/morpho.ts

export const morphoAdapter: ProtocolAdapter = {
  name: "morpho-blue",
  chains: ["ethereum", "base"],

  getWatchedContracts(chain: ChainId): WatchedContract[] {
    return [
      {
        // Morpho Blue singleton (deterministic CREATE2)
        address: "0xBBBBBbbBBb9cC5e90e3b3Af64bdAF62C37EEFFCb",
        selectors: [
          "0x6e553f65", // supply
          "0xb460af94", // withdraw
          "0x0c0a769b", // borrow
          "0x20b76e81", // repay
          "0x50d8cd4b", // liquidate
          "0x3644e515", // accrueInterest
        ],
        strategy: "replay",
        priority: 30,
      },
    ];
  },
};

// Interest rate accumulator sync (runs on a timer, not per-block)
export class MorphoAccrualSimulator {
  async syncMarket(marketId: Hex): Promise<void> {
    // Read current market state from mainnet
    const [
      totalSupplyAssets,
      totalSupplyShares,
      totalBorrowAssets,
      totalBorrowShares,
      lastUpdate,
      fee,
    ] = await mainnetClient.readContract({
      address: MORPHO_BLUE,
      abi: morphoABI,
      functionName: "market",
      args: [marketId],
    });

    // Compute storage slot for market mapping
    // mapping(Id => Market) at slot 3
    const marketSlot = keccak256(
      encodePacked(["bytes32", "uint256"], [marketId, 3n]),
    );

    // Write each field to the appropriate sub-slot
    await anvilClient.setStorageAt({
      address: MORPHO_BLUE,
      slot: marketSlot, // totalSupplyAssets
      value: pad(toHex(totalSupplyAssets)),
    });
    // ... (totalSupplyShares at slot+1, totalBorrowAssets at slot+2, etc.)
  }
}
```

**Watched Morpho markets (configured per Golem strategy):**

The operator specifies which Morpho market IDs the Golem's vault adapters use. Mirage only replays activity in those markets. For a USDC vault supplying to a WETH/USDC market, only supply/withdraw/borrow/liquidate transactions for that specific market ID are replayed.

### Protocol Adapter: Aave V3

Similar to Morpho but with Aave V3's pool-based architecture.

**Strategy: Hybrid (replay + storage injection)**

```typescript
// packages/mirage/src/protocols/aave.ts

export const aaveAdapter: ProtocolAdapter = {
  name: "aave-v3",
  chains: ["ethereum", "base"],

  getWatchedContracts(chain: ChainId): WatchedContract[] {
    const pool = AAVE_POOL_ADDRESSES[chain];
    return [
      {
        address: pool,
        selectors: [
          "0x617ba037", // supply
          "0x69328dec", // withdraw
          "0xa415bcad", // borrow
          "0x573ade81", // repay
          "0x00a718a9", // liquidationCall
        ],
        strategy: "replay",
        priority: 30,
      },
    ];
  },
};
```

### Protocol Adapter: Gas Simulator

Keeps the local fork's baseFee in sync with mainnet, so the Golem's gas circuit breaker (pause at 5x 7-day median baseFee) triggers correctly.

**Strategy: Storage injection via Anvil RPC**

```typescript
// packages/mirage/src/protocols/gas.ts

export class GasSimulator {
  async syncBaseFee(mainnetBlock: Block): Promise<void> {
    // Anvil supports setting the next block's baseFee
    await fetch(anvilRpcUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method: "anvil_setNextBlockBaseFeePerGas",
        params: [toHex(mainnetBlock.baseFeePerGas!)],
        id: 1,
      }),
    });
  }
}
```

### Protocol Adapter: ERC-8004 Identity Registry

Replays identity registrations and reputation attestations so the Golem sees realistic agent population growth.

```typescript
// packages/mirage/src/protocols/erc8004.ts

export const erc8004Adapter: ProtocolAdapter = {
  name: "erc-8004",
  chains: ["ethereum", "base"],

  getWatchedContracts(chain: ChainId): WatchedContract[] {
    return [
      {
        address: ERC8004_IDENTITY_REGISTRY[chain],
        selectors: [], // All calls
        strategy: "replay",
        priority: 50,
      },
      {
        address: ERC8004_REPUTATION_REGISTRY[chain],
        selectors: [], // All calls
        strategy: "replay",
        priority: 50,
      },
    ];
  },
};
```

### Protocol Adapter: ERC-20 Token Transfers

Selectively replays large token transfers that affect the Golem's operating tokens (USDC, WETH, etc.) to keep balance distributions realistic. Filtered heavily to avoid replaying every USDC transfer on Base.

**Strategy: Selective replay** (only transfers above a threshold, only to/from watched addresses)

```typescript
// packages/mirage/src/protocols/erc20.ts

export const erc20Adapter: ProtocolAdapter = {
  name: "erc-20",
  chains: ["ethereum", "base"],

  getWatchedContracts(chain: ChainId): WatchedContract[] {
    // Only watch the specific tokens the Golem uses
    return GOLEM_TOKENS[chain].map((token) => ({
      address: token.address,
      selectors: [
        "0xa9059cbb", // transfer
        "0x23b872dd", // transferFrom
        "0x095ea7b3", // approve
      ],
      strategy: "replay",
      priority: 80, // Low priority — process after protocol-level txs
      // Custom filter: only replay if amount > threshold
      filter: (tx) => decodeTransferAmount(tx) > token.minReplayAmount,
    }));
  },
};
```

---

## Chain-Specific Behavior

### Base L2

| Property                            | Value                                                    |
| ----------------------------------- | -------------------------------------------------------- |
| Block time                          | ~2 seconds                                               |
| Transactions per block              | ~100-7,500 (highly variable)                             |
| Block follower poll interval        | 1,000ms                                                  |
| Sequencer                           | Centralized (Coinbase) — no mempool in traditional sense |
| L1 data                             | Included in block but not relevant for Mirage            |
| Recommended max tx replay per block | 200                                                      |

Base's fast block time means Mirage must be efficient. The filtering layer is critical — without it, processing 7,500 transactions every 2 seconds is infeasible.

### Ethereum Mainnet

| Property                            | Value                                                   |
| ----------------------------------- | ------------------------------------------------------- |
| Block time                          | ~12 seconds                                             |
| Transactions per block              | ~150-300                                                |
| Block follower poll interval        | 3,000ms                                                 |
| Mempool                             | Available via `eth_subscribe('newPendingTransactions')` |
| Recommended max tx replay per block | 100                                                     |

Ethereum's slower block time gives more headroom. The optional WebSocket Mempool Listener can provide sub-block-time visibility into pending transactions, which is useful for MEV simulation (Phase 2+).

### Multi-Chain Mirage

Mirage can run two instances simultaneously — one following Base, one following Ethereum — each feeding into a separate Anvil fork. This tests cross-chain Golem strategies.

```bash
# Terminal 1: Base mirage
pnpm mirage --chain base --anvil-port 8545

# Terminal 2: Ethereum mirage
pnpm mirage --chain ethereum --anvil-port 8546

# Golem connects to both
GOLEM_BASE_RPC=http://localhost:8545
GOLEM_ETH_RPC=http://localhost:8546
```

---

## Configuration

All Mirage configuration lives in a single file: `mirage.config.ts` (or `mirage.config.json`).

```typescript
// mirage.config.ts
import { defineConfig } from "@bardo/mirage";

export default defineConfig({
  // === Chain Configuration ===
  chains: {
    base: {
      enabled: true,
      rpcUrl: process.env.BASE_RPC_URL!,
      anvilPort: 8545,
      // Fork at latest block (default) or pin a specific block
      forkBlockNumber: "latest",
      // Anvil chain ID (distinct from mainnet to prevent replay attacks)
      chainId: 31337,
      blockTime: 2, // Anvil block time in seconds (match Base)
    },
    ethereum: {
      enabled: false, // Enable for dual-chain testing
      rpcUrl: process.env.ETH_RPC_URL!,
      anvilPort: 8546,
      forkBlockNumber: "latest",
      chainId: 31338,
      blockTime: 12,
    },
  },

  // === Protocol Adapters ===
  protocols: {
    uniswap: {
      enabled: true,
      // Pool addresses to watch (from Golem's STRATEGY.md)
      watchedPools: ["0x...WETH-USDC-500", "0x...cbETH-WETH-500"],
      replayAllV4: true,
      replayPermit2: true,
    },
    chainlink: {
      enabled: true,
      feeds: [
        { name: "ETH/USD", pollIntervalMs: 5000 },
        { name: "USDC/USD", pollIntervalMs: 30000 },
        // Additional feeds auto-resolved from STRATEGY.md tokens
      ],
    },
    morpho: {
      enabled: true,
      // Market IDs from the Golem's MorphoSupplyAdapter config
      watchedMarkets: ["0x...market-id"],
      accrualSyncIntervalMs: 30000,
    },
    aave: {
      enabled: true,
      watchedReserves: ["USDC", "WETH"],
      accrualSyncIntervalMs: 30000,
    },
    erc8004: {
      enabled: true,
    },
    erc20: {
      enabled: true,
      tokens: [
        { symbol: "USDC", minReplayAmount: 10_000n * 10n ** 6n }, // Only >$10K transfers
        { symbol: "WETH", minReplayAmount: 5n * 10n ** 18n }, // Only >5 ETH transfers
      ],
    },
    gas: {
      enabled: true,
    },
  },

  // === Performance Tuning ===
  performance: {
    maxTxPerBlock: 200,
    maxBlocksPerCycle: 5,
    skipPureTransfers: true,
    maxGapBeforeRefork: 100, // Re-fork if >100 blocks behind
  },

  // === Logging ===
  logging: {
    level: "info", // 'debug' for full tx-level logging
    outputDir: ".mirage/logs",
    jsonl: true,
    // Correlate Mirage events with Golem heartbeat ticks
    golemCorrelation: true,
  },

  // === Auto-derive from STRATEGY.md ===
  // If set, Mirage reads the Golem's STRATEGY.md and automatically
  // configures watched pools, tokens, and protocol adapters.
  strategyFile: "./STRATEGY.md",
});
```

### Auto-Derive from STRATEGY.md

When `strategyFile` is set, Mirage parses the Golem's strategy file and automatically populates the watched contracts:

```
STRATEGY.md mentions "WETH/USDC pool on V3 (0x...)" →
  Mirage adds that pool to uniswap.watchedPools

STRATEGY.md mentions "Supply to Morpho Blue WETH/USDC market" →
  Mirage enables morpho adapter and adds the market ID

STRATEGY.md mentions "Target assets: WETH, USDC, cbETH" →
  Mirage configures chainlink feeds for ETH/USD, USDC/USD, cbETH/ETH
  Mirage configures erc20 filters for those tokens
```

---

## Integration with Existing Dev Tooling

### With @bardo/testnet (AnvilManager)

Mirage delegates all Anvil lifecycle management to `AnvilManager`:

```typescript
import { AnvilManager } from "@bardo/testnet";
import { MirageRunner } from "@bardo/mirage";

// AnvilManager starts and manages the Anvil process
const anvil = new AnvilManager({
  forkUrl: process.env.BASE_RPC_URL,
  port: 8545,
  chainId: 31337,
  blockTime: 2,
});

await anvil.start();

// Mirage attaches to the running Anvil instance
const mirage = new MirageRunner({
  anvilRpcUrl: anvil.rpcUrl,
  mainnetRpcUrl: process.env.BASE_RPC_URL,
  chain: "base",
  config: mirageConfig,
});

await mirage.start();

// Mirage is now replaying mainnet transactions into Anvil
// The Golem connects to anvil.rpcUrl and sees a living environment
```

### With @bardo/dev (UniswapDeployment)

In **fork mode**, `@bardo/dev` forks Base mainnet and all Uniswap contracts are already present. Mirage works immediately — just start the follower.

In **from-scratch mode**, `@bardo/dev` deploys fresh Uniswap contracts. Mirage still works, but the watched contract addresses must be updated to the local deployment addresses (from `UniswapDeployment.addresses`). Oracle simulation and lending protocol adapters are typically disabled in from-scratch mode since those contracts aren't deployed locally.

```typescript
import { createDevEnvironment } from "@bardo/dev";
import { MirageRunner } from "@bardo/mirage";

const dev = await createDevEnvironment({ mode: "fork" });

const mirage = new MirageRunner({
  anvilRpcUrl: dev.rpcUrl,
  mainnetRpcUrl: process.env.BASE_RPC_URL,
  chain: "base",
  config: {
    ...mirageConfig,
    // Override addresses if using from-scratch mode
    protocols: {
      ...mirageConfig.protocols,
      uniswap: {
        ...mirageConfig.protocols.uniswap,
        watchedPools:
          dev.deployment.mode === "from-scratch"
            ? dev.deployment.seededPools // Local pool addresses
            : mirageConfig.protocols.uniswap.watchedPools, // Mainnet addresses
      },
    },
  },
});
```

### With Golem Runtime

The Golem doesn't know Mirage exists. It connects to the Anvil RPC URL and interacts normally through Sanctum (the Golem's on-chain execution tool suite) tools. Mirage operates entirely in the background, keeping the environment alive.

```
Golem Runtime
  │
  ├── Heartbeat tick
  │     ├── Probe: price_delta    → reads Chainlink feed → sees Mirage-injected price
  │     ├── Probe: position_health → reads Uniswap pool → sees Mirage-replayed swap effects
  │     ├── Probe: credit_balance  → reads vault NAV → sees Mirage-replayed deposits
  │     └── Probe: gas_circuit     → reads baseFee → sees Mirage-synced gas price
  │
  ├── System 2 escalation → decides to rebalance LP
  │     └── Calls execute_swap via Sanctum → tx goes to Anvil
  │         (Anvil now has divergent state — future replays may fail for this pool.
  │          This is correct: the Golem changed the world.)
  │
  └── Grimoire records episode with real price movements, real counterparty activity
```

### With SwarmRunner

`SwarmRunner` from `@bardo/dev` runs multiple Golems simultaneously. Mirage feeds the same live data to all of them on the same Anvil instance. Each Golem's actions affect the shared local state, creating natural multi-agent dynamics.

```typescript
import { SwarmRunner } from "@bardo/dev";
import { MirageRunner } from "@bardo/mirage";

const mirage = new MirageRunner(config);
await mirage.start();

const swarm = new SwarmRunner({
  rpcUrl: mirage.anvilRpcUrl,
  golems: [
    { strategy: "./strategies/conservative-lp.md", name: "golem-1" },
    { strategy: "./strategies/aggressive-trader.md", name: "golem-2" },
    { strategy: "./strategies/yield-optimizer.md", name: "golem-3" },
  ],
});

await swarm.start();
// 3 Golems + live mainnet replay = realistic multi-agent test
```

---

## CLI Interface

```bash
# Start Mirage with default config (reads mirage.config.ts)
pnpm mirage

# Start with specific chain
pnpm mirage --chain base

# Start with specific Anvil port
pnpm mirage --chain base --anvil-port 8545

# Dry run: show what would be replayed without executing
pnpm mirage --dry-run

# Start with verbose logging (every replayed tx logged)
pnpm mirage --log-level debug

# Start in auto-derive mode (reads STRATEGY.md)
pnpm mirage --strategy ./STRATEGY.md

# Show replay statistics
pnpm mirage stats

# Reset: re-fork from latest block
pnpm mirage reset
```

### Integration with existing dev commands:

```bash
# Full dev stack with Mirage enabled:
pnpm dev:node --mirage

# This is equivalent to:
pnpm dev:node && pnpm mirage --chain base --anvil-port 8545

# For Golem local testing:
pnpm dev:golem --mirage
# Starts: Anvil (fork) → Deploy Golem contracts → Start Mirage → Start Golem heartbeat
```

---

## Structured Logging

All Mirage activity is logged as JSONL to `.mirage/logs/mirage-{chain}-{date}.jsonl`:

```jsonl
{"ts":"2026-03-09T14:00:01.234Z","event":"block:received","chain":"base","block":28500001,"txCount":342,"filteredCount":12}
{"ts":"2026-03-09T14:00:01.456Z","event":"tx:replayed","chain":"base","block":28500001,"originalHash":"0x...","localHash":"0x...","protocol":"uniswap","status":"success","gasUsed":152340}
{"ts":"2026-03-09T14:00:01.567Z","event":"tx:replayed","chain":"base","block":28500001,"originalHash":"0x...","localHash":"0x...","protocol":"uniswap","status":"reverted","error":"STF","divergenceNote":"pool state diverged from Golem swap at block 28499998"}
{"ts":"2026-03-09T14:00:01.678Z","event":"oracle:synced","chain":"base","feed":"ETH/USD","price":"3245.67","roundId":110680464442257320917}
{"ts":"2026-03-09T14:00:01.789Z","event":"gas:synced","chain":"base","baseFee":"0.0012 gwei"}
{"ts":"2026-03-09T14:00:02.001Z","event":"block:processed","chain":"base","block":28500001,"replayedCount":11,"skippedCount":1,"durationMs":767}
```

When `golemCorrelation: true`, Mirage events include a `golemTick` field if a Golem heartbeat tick occurred during the same block window. This enables post-hoc analysis: "what did the Golem see when ETH/USD dropped 2% in block 28500001?"

---

## Package Layout

```
packages/mirage/
├── src/
│   ├── index.ts                    # Public API: MirageRunner, defineConfig
│   ├── config.ts                   # Config schema (zod), defaults, auto-derive
│   ├── runner.ts                   # MirageRunner orchestrator
│   ├── block-follower.ts           # Block polling + dispatch
│   ├── router.ts                   # Transaction classification + routing
│   ├── replayer.ts                 # Impersonate-and-send core loop
│   ├── storage-injector.ts         # Direct state writes (setStorageAt)
│   ├── timestamp.ts                # Block timestamp advancement
│   ├── logger.ts                   # Structured JSONL logger
│   ├── cursor.ts                   # Persistent block cursor
│   ├── protocols/
│   │   ├── index.ts                # Protocol adapter registry
│   │   ├── uniswap.ts             # Uniswap V2/V3/V4/Router/Permit2
│   │   ├── chainlink.ts           # Oracle price injection
│   │   ├── morpho.ts              # Morpho Blue supply/borrow replay
│   │   ├── aave.ts                # Aave V3 supply/borrow replay
│   │   ├── erc8004.ts             # Identity + reputation registry
│   │   ├── erc20.ts               # Filtered token transfer replay
│   │   ├── gas.ts                 # baseFee synchronization
│   │   └── custom.ts              # User-defined protocol adapter template
│   ├── chains/
│   │   ├── base.ts                # Base-specific addresses, block time, config
│   │   └── ethereum.ts            # Ethereum-specific addresses, block time, config
│   └── utils/
│       ├── addresses.ts            # Mainnet contract address constants
│       ├── selectors.ts            # Known function selector registry
│       ├── storage-layout.ts       # Chainlink/Morpho/Aave storage slot helpers
│       └── strategy-parser.ts      # Parse STRATEGY.md for auto-derive
├── bin/
│   └── mirage.ts                   # CLI entry point
├── test/
│   ├── replayer.test.ts
│   ├── chainlink-simulator.test.ts
│   ├── router.test.ts
│   └── integration/
│       └── full-mirage.test.ts     # End-to-end: fork + mirage + golem heartbeat
├── mirage.config.example.ts
├── package.json
├── tsconfig.json
└── README.md
```

---

## Implementation Plan

### Phase 1: Core Replay Engine (Week 1-2)

**Deliverables:**

- `BlockFollower` with HTTP polling for both Base and Ethereum
- `TransactionRouter` with address + selector filtering
- `TransactionReplayer` with impersonation and failure handling
- `TimestampAdvancer` with block-aligned timestamps
- `ChainlinkSimulator` for oracle price injection (ETH/USD, USDC/USD)
- `GasSimulator` for baseFee sync
- CLI: `pnpm mirage --chain base`
- JSONL logging

**Acceptance criteria:**

- Fork Base mainnet, start Mirage, observe Uniswap pool states updating in real-time
- Chainlink ETH/USD price on local fork matches mainnet within 10 seconds
- Golem heartbeat `price_delta` probe detects real price movements
- Mirage processes Base blocks with <2s latency (block-to-replay complete)

### Phase 2: Protocol Adapters (Week 3-4)

**Deliverables:**

- `MorphoAdapter` + `MorphoAccrualSimulator`
- `AaveAdapter` + rate accumulator sync
- `ERC8004Adapter` for identity/reputation replay
- `ERC20Adapter` with configurable thresholds
- `UniswapAdapter` with pool-level filtering
- Config auto-derive from STRATEGY.md
- `mirage.config.ts` with full schema validation

**Acceptance criteria:**

- Golem vault using MorphoSupplyAdapter sees realistic supply rate changes
- ERC-8004 registrations from mainnet appear on local fork
- Pool-level Uniswap filtering reduces replayed transactions by >90% vs. unfiltered

### Phase 3: Integration + SwarmRunner (Week 5-6)

**Deliverables:**

- `@bardo/dev` integration: `pnpm dev:node --mirage`
- `@bardo/testnet` integration: `AnvilManager` → `MirageRunner` composition
- SwarmRunner support: multi-Golem testing with shared Mirage
- Golem correlation logging
- `pnpm mirage stats` CLI command
- Dual-chain mode (Base + Ethereum simultaneously)

**Acceptance criteria:**

- `pnpm dev:golem --mirage` starts full stack in one command
- 3-Golem swarm running with Mirage for 1 hour without crashes
- State divergence from Golem actions is logged, not fatal
- Mirage stats show replay success rate, latency percentiles, divergence count

### Phase 4: Adversarial Simulation (Future)

**Not in v1 scope. Documented for roadmap clarity.**

- Sandwich attack simulator: inject front-run/back-run transactions around the Golem's swaps
- Liquidation simulator: trigger Aave/Morpho liquidation events on the Golem's positions
- Oracle manipulation simulator: inject stale or manipulated oracle prices to test circuit breakers
- Gas spike simulator: inject sudden baseFee increases (10x-50x) to test gas circuit breaker behavior
- Whale simulator: inject large deposit/withdrawal events into the Golem's vault to test share price impact handling

---

## Dependencies

| Dependency     | Version   | Purpose                                       |
| -------------- | --------- | --------------------------------------------- |
| viem           | ^2.0      | Ethereum client, ABI encoding, contract reads |
| @bardo/testnet | workspace | AnvilManager, SnapshotStore                   |
| @bardo/core    | workspace | Error hierarchies, config I/O                 |
| @bardo/chain   | workspace | RPC clients, chain configuration              |
| zod            | ^3.0      | Config schema validation                      |
| commander      | ^12.0     | CLI argument parsing                          |
| pino           | ^9.0      | Structured logging                            |

No external services. No API keys beyond the RPC endpoint. No hosted infrastructure.

---

## References

- Anvil RPC Methods: https://getfoundry.sh/anvil/reference/ — *Complete RPC reference for the Foundry Anvil fork environment that Mirage wraps.*
- Chainlink Data Feeds (Base): https://docs.chain.link/data-feeds/price-feeds/addresses?network=base — *Oracle feed addresses used for price injection in Base fork scenarios.*
- Chainlink Data Feeds (Ethereum): https://docs.chain.link/data-feeds/price-feeds/addresses?network=ethereum — *Oracle feed addresses for Ethereum mainnet fork scenarios.*
- Morpho Blue Docs: https://docs.morpho.org/ — *Lending protocol API reference; Mirage tests validate health factor and interest accrual against Morpho.*
- Aave V3 Docs: https://docs.aave.com/developers/ — *Lending protocol API reference; Mirage tests validate liquidation triggers and borrow rate tracking against Aave.*
- Shadow Forks (Ethereum Foundation): https://etherworld.co/2022/04/20/ethereum-mainnet-shadow-forking-an-overview/ — *Describes shadow forking technique where a parallel chain replays mainnet transactions; prior art for Mirage's approach.*
- shadow-reth (open source): https://github.com/shadow-hq/shadow-reth — *Open-source shadow fork implementation for reth; reference implementation for Mirage's shadow execution mode.*
- Kurtosis mempool-bridge: https://github.com/ethpandaops/mempool-bridge — *Mempool relay tool used for pre-confirmation testing in Mirage scenarios.*
- Tenderly State Sync: https://docs.tenderly.co/virtual-testnets/state-sync — *Hosted fork infrastructure with state synchronization; commercial alternative that informed Mirage's state sync design.*

---

## Implementation Interfaces

### MirageConfig

```typescript
interface MirageConfig {
  /** Target chain to follow (chain ID or name) */
  chain: number | string;
  /** RPC URL for the source chain */
  rpcUrl: string;
  /** Local Anvil instance configuration */
  anvil: {
    /** Port for the local Anvil fork (default: 8545) */
    port: number;
    /** Block number to fork from (default: latest) */
    forkBlockNumber?: number;
  };
  /** Cursor persistence path (default: .mirage/cursor.json) */
  cursorPath: string;
  /** JSONL log output path (default: .mirage/events.jsonl) */
  logPath: string;
  /** Contracts to watch for selective replay */
  watchedContracts: `0x${string}`[];
  /** Replay mode configuration */
  replay: {
    /** Which transactions to replay */
    mode: "all" | "watched" | "none";
    /** Skip transactions below this gas threshold */
    minGas?: number;
  };
  /** Polling interval for new blocks in milliseconds (default: 2000) */
  pollIntervalMs: number;
}
```

### BlockFollower

```typescript
interface BlockFollower {
  /** Start following blocks from the cursor position */
  start(): Promise<void>;
  /** Stop following and persist cursor */
  stop(): Promise<void>;
  /** Get the current cursor position */
  getCursor(): BlockCursor;
  /** Register a callback for new blocks */
  onBlock(callback: (block: BlockEvent) => Promise<void>): void;
  /** Register a callback for errors */
  onError(callback: (error: Error) => void): void;
}

interface BlockCursor {
  chainId: number;
  blockNumber: number;
  blockHash: string;
  timestamp: number;
  updatedAt: number;
}
```

### TransactionRouter

```typescript
interface TransactionRouter {
  /** Classify a transaction for replay decisions */
  classify(tx: TransactionEvent): TxClassification;
  /** Determine if a transaction should be replayed locally */
  shouldReplay(tx: TransactionEvent): boolean;
  /** Get parameters for local replay */
  getReplayParams(tx: TransactionEvent): ReplayParams | null;
}

type TxClassification =
  | "uniswap_swap"
  | "uniswap_lp"
  | "vault_operation"
  | "erc20_transfer"
  | "contract_deploy"
  | "other";

interface ReplayParams {
  to: `0x${string}`;
  data: `0x${string}`;
  value: bigint;
  gasLimit: bigint;
  /** Whether to impersonate the original sender */
  impersonate: boolean;
}
```

### ReplayResult

```typescript
type ReplayResult =
  | { status: "success"; txHash: string; gasUsed: bigint }
  | { status: "skip"; reason: string }
  | { status: "fail"; reason: string; error?: Error };
```

### JSONL Event Schemas

```typescript
interface BaseEvent {
  type: string;
  timestamp: number;
  blockNumber?: number;
  source: "mirage";
}

interface BlockEvent extends BaseEvent {
  type: "block";
  blockHash: string;
  transactionCount: number;
  gasUsed: bigint;
}

interface TxReplayEvent extends BaseEvent {
  type: "tx_replay";
  originalTxHash: string;
  classification: TxClassification;
  result: ReplayResult;
  duration: number; // milliseconds
}

interface DivergenceEvent extends BaseEvent {
  type: "divergence";
  field: string; // what diverged (e.g., "balance", "storage_slot")
  expected: string;
  actual: string;
  contract: `0x${string}`;
}

interface CursorEvent extends BaseEvent {
  type: "cursor";
  action: "save" | "load" | "reset";
  cursor: BlockCursor;
}
```

### CLI Commands

| Command         | Flags                         | Description                                     |
| --------------- | ----------------------------- | ----------------------------------------------- |
| `mirage init`   | `--chain <id>`, `--rpc <url>` | Initialize mirage config in current directory   |
| `mirage start`  | `--config <path>`, `--fresh`  | Start block following and replay                |
| `mirage status` |                               | Show current cursor position and replay stats   |
| `mirage tail`   | `--follow`, `--filter <type>` | Tail the JSONL event log                        |
| `mirage reset`  | `--to-block <n>`              | Reset cursor to a specific block                |
| `mirage doctor` |                               | Validate config, RPC connectivity, Anvil health |

### Package File Layout

```
packages/mirage/
├── package.json
├── tsconfig.json
├── eslint.config.ts
├── vitest.config.ts
├── src/
│   ├── index.ts              # barrel exports
│   ├── config.ts             # MirageConfig + Zod schema + loader
│   ├── events.ts             # JSONL event type definitions
│   ├── cursor.ts             # BlockCursor persistence
│   ├── follower.ts           # BlockFollower implementation
│   ├── router.ts             # TransactionRouter + classification
│   ├── replay.ts             # Transaction replay engine
│   ├── divergence.ts         # State divergence detection
│   ├── cli.ts                # Commander CLI entrypoint
│   ├── runtime.ts            # Main runtime orchestration
│   └── errors.ts             # Mirage-specific error types
└── test/
    ├── config.test.ts
    ├── cursor.test.ts
    ├── follower.test.ts
    ├── router.test.ts
    └── replay.test.ts
```

---

## mirage-rs v2 Test Additions

The following sections cover testing for the v2 architecture. v2 replaces v1's full block replay with a lazy-latest read model, automatic dirty tracking, targeted replay, and CoW branching. These changes introduce new invariants that need verification. The full v2 test specification lives in `16-testing/11-mirage-v2-testing.md`; this section provides the core test sketches integrated into the existing mirage testing framework.

---

### HybridDB Three-Tier Read Correctness

The HybridDB reads in priority order: DirtyStore, then ReadCache, then upstream RPC. A violation means the golem sees stale or incorrect state.

#### Test: DirtyStore Override Priority

**Property:** For any address A and storage slot S, if DirtyStore contains value V for (A, S), then `HybridDB::storage(A, S)` returns V regardless of what ReadCache or upstream would return.

```rust
#[test]
fn dirty_store_overrides_cache_and_upstream() {
    let mut db = HybridDB::new(test_config());
    let addr = address!("0x1111111111111111111111111111111111111111");
    let slot = U256::from(42);

    // Upstream returns 100.
    db.upstream.mock_storage(addr, slot, U256::from(100));

    // First read: goes to upstream, cached.
    assert_eq!(db.storage(addr, slot).unwrap(), U256::from(100));

    // Set dirty value to 999.
    db.dirty.set_storage(addr, slot, U256::from(999));

    // Must return dirty value, not cached upstream.
    assert_eq!(db.storage(addr, slot).unwrap(), U256::from(999));
}
```

#### Test: ReadCache Serves Before Upstream on Warm Path

```rust
#[test]
fn read_cache_avoids_upstream_on_warm_reads() {
    let mut db = HybridDB::new(test_config());
    let addr = address!("0x2222222222222222222222222222222222222222");
    let slot = U256::from(7);

    db.upstream.mock_storage(addr, slot, U256::from(50));

    // First read: cold, hits upstream.
    let _ = db.storage(addr, slot).unwrap();
    let calls_after_first = db.upstream.call_count();

    // Second read: warm, should not hit upstream.
    let _ = db.storage(addr, slot).unwrap();
    assert_eq!(db.upstream.call_count(), calls_after_first);
}
```

#### Test: Cache TTL Expiry Forces Re-Fetch

```rust
#[test]
fn cache_ttl_expiry_refetches_from_upstream() {
    let mut db = HybridDB::new(MirageConfig {
        cache_ttl: Duration::from_millis(50),
        ..test_config()
    });
    let addr = address!("0x3333333333333333333333333333333333333333");
    let slot = U256::from(1);

    db.upstream.mock_storage(addr, slot, U256::from(10));
    let _ = db.storage(addr, slot).unwrap();

    // Change upstream value.
    db.upstream.mock_storage(addr, slot, U256::from(20));

    // Wait for TTL to expire.
    std::thread::sleep(Duration::from_millis(60));

    // Should re-fetch and return new value.
    assert_eq!(db.storage(addr, slot).unwrap(), U256::from(20));
}
```

#### Test: Pinned Block in Historical Mode

```rust
#[test]
fn historical_mode_pins_all_reads_to_from_block() {
    let mut db = HybridDB::new(MirageConfig {
        mode: Mode::Historical,
        from_block: Some(19500000),
        ..test_config()
    });
    let addr = address!("0x4444444444444444444444444444444444444444");
    let slot = U256::from(0);

    let _ = db.storage(addr, slot).unwrap();

    // Verify the upstream call used block 19500000, not "latest".
    let last_call = db.upstream.last_call();
    assert_eq!(last_call.block_tag, BlockTag::Number(19500000));
}
```

---

### CoW Branch Isolation

CoW state layers share a baseline via `Arc` and store only per-branch mutations. Branches must not contaminate each other.

#### Test: Writes to One Branch Do Not Affect Another

```rust
#[test]
fn cow_branches_are_isolated() {
    let baseline = Arc::new(HashMap::from([
        ((addr_a(), slot_0()), U256::from(100)),
    ]));

    let mut branch_1 = CowState::branch(&baseline);
    let mut branch_2 = CowState::branch(&baseline);

    // Branch 1 writes.
    branch_1.write(addr_a(), slot_0(), U256::from(200));

    // Branch 2 still sees baseline.
    assert_eq!(branch_2.read(addr_a(), slot_0()), Some(U256::from(100)));

    // Branch 1 sees its own write.
    assert_eq!(branch_1.read(addr_a(), slot_0()), Some(U256::from(200)));
}
```

#### Test: Baseline is Immutable After Branching

```rust
#[test]
fn baseline_immutable_after_branching() {
    let baseline_data = HashMap::from([
        ((addr_a(), slot_0()), U256::from(100)),
    ]);
    let baseline = Arc::new(baseline_data);
    let baseline_clone = Arc::clone(&baseline);

    let mut branch = CowState::branch(&baseline);
    branch.write(addr_a(), slot_0(), U256::from(999));

    // Original Arc data unchanged.
    assert_eq!(
        baseline_clone.get(&(addr_a(), slot_0())),
        Some(&U256::from(100))
    );
}
```

#### Test: Overlay Memory Proportional to Mutations Only

```rust
#[test]
fn cow_overlay_size_tracks_mutations_only() {
    let baseline = Arc::new(
        (0..50_000u64)
            .map(|i| ((addr_a(), U256::from(i)), U256::from(i)))
            .collect::<HashMap<_, _>>()
    );

    let mut branch = CowState::branch(&baseline);

    // Write 200 slots.
    for i in 0..200u64 {
        branch.write(addr_a(), U256::from(i), U256::from(i + 1000));
    }

    assert_eq!(branch.overlay_size(), 200);
    // 200 entries * ~64 bytes each = ~12.8 KB, not 50,000 * 64 = 3.2 MB.
}
```

---

### State Diff Classification Accuracy

The DiffClassifier determines whether a contract enters the watch list (Protocol), gets slot-level overrides (SlotOnly), or is ignored (ReadOnly). Misclassification has real consequences: a protocol misclassified as a token won't get mainnet replay, so its state drifts.

#### Test: ERC-20 Transfer Classified as SlotOnly

```rust
#[test]
fn erc20_transfer_is_slot_only() {
    let classifier = DiffClassifier::new(ClassificationConfig::default());
    let diff = StateDiff {
        accounts: HashMap::from([(
            usdc_address(),
            AccountDiff {
                storage_written: HashMap::from([
                    (balance_slot(sender()), U256::from(900)),
                    (balance_slot(receiver()), U256::from(100)),
                ]),
                storage_read: HashSet::new(),
                info_changed: false,
                new_balance: None,
                new_nonce: None,
                new_code: None,
            },
        )]),
    };

    let result = classifier.classify(&diff, &empty_dirty_store(), 1);
    assert!(result.new_watched.is_empty());
    assert_eq!(result.slot_overrides.len(), 1);
}
```

#### Test: Uniswap V3 Swap Classified as Protocol

```rust
#[test]
fn uniswap_v3_swap_is_protocol() {
    let classifier = DiffClassifier::new(ClassificationConfig::default());
    let diff = StateDiff {
        accounts: HashMap::from([(
            pool_address(),
            AccountDiff {
                storage_written: HashMap::from([
                    (U256::from(0), U256::from(1)),  // slot0 (sqrtPriceX96)
                    (U256::from(1), U256::from(2)),  // slot0 (tick)
                    (U256::from(3), U256::from(3)),  // liquidity
                    (U256::from(4), U256::from(4)),  // feeGrowthGlobal0X128
                ]),
                storage_read: HashSet::new(),
                info_changed: false,
                new_balance: None,
                new_nonce: None,
                new_code: None,
            },
        )]),
    };

    let result = classifier.classify(&diff, &empty_dirty_store(), 1);
    assert_eq!(result.new_watched.len(), 1);
    assert_eq!(result.new_watched[0].1.source, WatchSource::AutoClassified);
}
```

#### Test: Rebasing Token (3+ Slots, All High-Entropy) Classified as SlotOnly

```rust
#[test]
fn rebasing_token_with_high_entropy_slots_is_slot_only() {
    let classifier = DiffClassifier::new(ClassificationConfig {
        check_token_interface: true,
        ..ClassificationConfig::default()
    });

    // stETH-like: 3 slot writes, all keccak-derived (> slot 20).
    let diff = StateDiff {
        accounts: HashMap::from([(
            steth_address(),
            AccountDiff {
                storage_written: HashMap::from([
                    (keccak_slot(sender()), U256::from(900)),
                    (keccak_slot(receiver()), U256::from(100)),
                    (keccak_slot(rebase_index()), U256::from(1_000_000)),
                ]),
                storage_read: HashSet::new(),
                info_changed: false,
                new_balance: None,
                new_nonce: None,
                new_code: None,
            },
        )]),
    };

    let result = classifier.classify(&diff, &empty_dirty_store(), 1);
    // Should be SlotOnly despite 3 written slots, because all are
    // high-entropy (no low-numbered slots).
    assert!(result.new_watched.is_empty());
}
```

#### Test: Watch List at Capacity Demotes to SlotOnly

```rust
#[test]
fn watch_list_at_capacity_demotes_new_protocols() {
    let classifier = DiffClassifier::new(ClassificationConfig {
        max_watched_contracts: 2,
        ..ClassificationConfig::default()
    });

    let mut dirty = DirtyStore::new();
    dirty.watch_list.insert(addr_a(), watch_entry());
    dirty.watch_list.insert(addr_b(), watch_entry());

    // Third protocol-like contract should be demoted.
    let diff = protocol_diff(addr_c());
    let result = classifier.classify(&diff, &dirty, 1);

    assert!(result.new_watched.is_empty());
    assert_eq!(result.slot_overrides.len(), 1);
}
```

---

### Targeted Replay Accuracy

TargetedFollower replays only mainnet transactions touching watched contracts. The test invariant: for every block in a historical range, the set of storage slots modified by targeted replay for watched addresses must match the set modified by full replay for the same addresses.

#### Test: Targeted Replay Matches Full Replay for Watched Contracts

```rust
#[test]
fn targeted_replay_matches_full_replay_for_watched_contracts() {
    let watch_list = hashset![pool_address()];
    let block = fetch_test_block(19500000);

    // Full replay: execute all transactions.
    let full_state = full_replay(&block);

    // Targeted replay: execute only transactions matching watch list.
    let targeted_state = targeted_replay(&block, &watch_list);

    // For every watched address, the dirty slots must match.
    for addr in &watch_list {
        let full_slots = full_state.dirty_slots_for(addr);
        let targeted_slots = targeted_state.dirty_slots_for(addr);
        assert_eq!(full_slots, targeted_slots,
            "Targeted replay diverged from full replay for {addr}");
    }
}
```

#### Test: Missed Aggregator Transaction Does Not Corrupt State

If the matcher misses a transaction routed through an aggregator (tx.to is the router, not the pool), the lazy-latest read path should still provide correct state for the next read.

```rust
#[test]
fn missed_aggregator_tx_does_not_corrupt_state() {
    let mut state = MirageState::new(test_config());
    state.db.dirty.watch_list.insert(pool_address(), watch_entry());

    // Simulate a missed aggregator swap (not replayed).
    // The pool's sqrtPriceX96 changed on mainnet but we didn't replay it.

    // Next eth_call should read from upstream at latest,
    // getting the correct post-swap sqrtPriceX96.
    let slot0 = state.db.storage(pool_address(), U256::from(0)).unwrap();

    // Verify it matches the upstream value (not a stale cached value).
    let upstream_val = state.db.upstream.get_storage_at(
        pool_address(), U256::from(0), BlockTag::Latest
    ).unwrap();
    assert_eq!(slot0, upstream_val);
}
```

---

### Block-STM Determinism

Block-STM executes transactions in parallel with optimistic concurrency. The determinism property: for any block, the final state after Block-STM parallel execution must be identical to the final state after sequential execution.

#### Test: Parallel Execution Matches Sequential

```rust
#[test]
fn block_stm_matches_sequential_execution() {
    let block = fetch_test_block(19500000);
    let initial_state = fork_state_at(19499999);

    // Sequential execution.
    let mut seq_state = initial_state.clone();
    for tx in &block.transactions {
        EvmExecutor::transact(&mut seq_state, tx);
    }

    // Block-STM parallel execution.
    let mut par_state = initial_state.clone();
    block_stm_execute(&mut par_state, &block.transactions);

    // All storage slots must match.
    for (addr, account) in seq_state.all_accounts() {
        for (slot, value) in account.storage.iter() {
            assert_eq!(
                par_state.storage(addr, *slot),
                *value,
                "Block-STM diverged at {addr} slot {slot}"
            );
        }
    }
}
```

#### Test: Conflicting Transactions Re-Execute Correctly

```rust
#[test]
fn block_stm_conflict_reexecution() {
    // Two transactions that both write to the same slot.
    let tx_a = swap_tx(pool_address(), U256::from(1000));  // tx index 0
    let tx_b = swap_tx(pool_address(), U256::from(2000));  // tx index 1

    let initial_state = fork_state_at(19499999);

    // Sequential: tx_a first, then tx_b reads tx_a's output.
    let mut seq_state = initial_state.clone();
    EvmExecutor::transact(&mut seq_state, &tx_a);
    EvmExecutor::transact(&mut seq_state, &tx_b);

    // Parallel: Block-STM detects the conflict, re-executes tx_b.
    let mut par_state = initial_state.clone();
    block_stm_execute(&mut par_state, &[tx_a, tx_b]);

    // Results must match.
    let seq_price = seq_state.storage(pool_address(), U256::from(0)).unwrap();
    let par_price = par_state.storage(pool_address(), U256::from(0)).unwrap();
    assert_eq!(seq_price, par_price);
}
```

---

### Property Tests (proptest Sketches)

#### DirtyStore Override Invariant

```rust
proptest! {
    #[test]
    fn dirty_always_wins(
        addr in any_address(),
        slot in any_u256(),
        dirty_val in any_u256(),
        cache_val in any_u256(),
        upstream_val in any_u256(),
    ) {
        let mut db = HybridDB::new(test_config());
        db.upstream.mock_storage(addr, slot, upstream_val);
        db.read_cache.insert_storage(addr, slot, cache_val, 0);
        db.dirty.set_storage(addr, slot, dirty_val);

        prop_assert_eq!(db.storage(addr, slot).unwrap(), dirty_val);
    }
}
```

#### CoW Branch Isolation Invariant

```rust
proptest! {
    #[test]
    fn cow_branch_isolation(
        baseline_entries in prop::collection::vec(
            (any_address(), any_u256(), any_u256()), 10..100
        ),
        branch_a_writes in prop::collection::vec(
            (any_address(), any_u256(), any_u256()), 1..20
        ),
        branch_b_writes in prop::collection::vec(
            (any_address(), any_u256(), any_u256()), 1..20
        ),
    ) {
        let baseline: HashMap<(Address, U256), U256> = baseline_entries
            .into_iter()
            .map(|(a, s, v)| ((a, s), v))
            .collect();
        let baseline = Arc::new(baseline);

        let mut branch_a = CowState::branch(&baseline);
        let mut branch_b = CowState::branch(&baseline);

        for (a, s, v) in &branch_a_writes {
            branch_a.write(*a, *s, *v);
        }
        for (a, s, v) in &branch_b_writes {
            branch_b.write(*a, *s, *v);
        }

        // Branch A's writes are not visible to Branch B.
        for (a, s, v) in &branch_a_writes {
            let b_val = branch_b.read(*a, *s);
            let baseline_val = baseline.get(&(*a, *s)).copied();
            prop_assert_eq!(b_val, baseline_val);
        }
    }
}
```

#### Classification Determinism

```rust
proptest! {
    #[test]
    fn classification_is_deterministic(
        slot_count in 0..10usize,
        has_low_slot in any::<bool>(),
    ) {
        let classifier = DiffClassifier::new(ClassificationConfig::default());
        let diff = generate_diff(slot_count, has_low_slot);
        let dirty = empty_dirty_store();

        let result_1 = classifier.classify(&diff, &dirty, 1);
        let result_2 = classifier.classify(&diff, &dirty, 1);

        prop_assert_eq!(result_1.new_watched.len(), result_2.new_watched.len());
        prop_assert_eq!(result_1.slot_overrides.len(), result_2.slot_overrides.len());
    }
}
```

---

### Cross-References

- Full v2 test specification: `16-testing/11-mirage-v2-testing.md`
- mirage-rs v2 architecture: `15-dev/01-mirage-rs.md`
- Scenario runner and historical mode: `15-dev/01c-mirage-scenarios.md`
- Bardo integration and golem workflows: `15-dev/01d-mirage-integration.md`
