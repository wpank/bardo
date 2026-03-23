# Bardo Tools -- Configuration [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles)
>
> Environment variables, config file schema, startup validation, health checks, capability token configuration, and API key tiers.

---

> **Reader orientation:** This document specifies the configuration system for `bardo-tools`: environment variables, TOML config file schema, startup validation, health checks, and API key tiers (Read/Feedback/Write). Configuration controls safety policies, wallet settings, chain RPC endpoints, and tool profiles. You should be comfortable with TOML configuration and environment variable precedence patterns. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Configuration hierarchy

```text
CLI flags  >  Environment variables  >  Config file  >  Defaults
```

Environment variables always override config file values. Never put secrets in the config file -- use env vars for all credentials.

---

## Environment variables

All environment variables use the `BARDO_` prefix.

### Core

| Variable           | Required | Default | Description                                                                    |
| ------------------ | -------- | ------- | ------------------------------------------------------------------------------ |
| `BARDO_PROFILE`    | No       | `data`  | Tool profile: data, trader, lp, vault, intelligence, learning, identity, golem, full, dev |
| `BARDO_LOG_LEVEL`  | No       | `info`  | Log level: debug, info, warn, error                                            |
| `BARDO_LOG_FORMAT` | No       | `json`  | Log format: json, text                                                         |

### Pi runtime

| Variable               | Required | Default     | Description                                              |
| ---------------------- | -------- | ----------- | -------------------------------------------------------- |
| `BARDO_PI_SESSION_TTL` | No       | `3600`      | Pi session TTL in seconds (default: 1 hour)              |
| `BARDO_PI_MAX_ACTIONS` | No       | `1000`      | Max action invocations per session before forced renewal |
| `BARDO_A2A_ENABLED`    | No       | `false`     | Enable A2A endpoint for external agent access            |
| `BARDO_A2A_PORT`       | No       | `3000`      | A2A endpoint port (when A2A is enabled)                  |
| `BARDO_A2A_HOST`       | No       | `localhost` | A2A endpoint host                                        |

### Wallet

| Variable                   | Required | Default | Description                       |
| -------------------------- | -------- | ------- | --------------------------------- |
| `BARDO_WALLET_PRIVATE_KEY` | No       | --      | Local private key (dev only)      |
| `BARDO_PRIVY_APP_ID`       | No       | --      | Privy application ID              |
| `BARDO_PRIVY_APP_SECRET`   | No       | --      | Privy application secret          |
| `BARDO_SAFE_ADDRESS`       | No       | --      | Safe smart account address        |
| `BARDO_SAFE_SIGNER_KEY`    | No       | --      | Safe signer private key           |
| `BARDO_ZERODEV_PROJECT_ID` | No       | --      | ZeroDev project ID                |
| `BARDO_LIT_AUTH_TOKEN`     | No       | --      | Lit Protocol (Vincent) auth token |

### Capability token limits

Capability tokens gate every write operation. These env vars set the default limits baked into each token. The safety extension reads them once at boot and uses them when minting `Capability<T>` tokens for write tool handlers.

| Variable                        | Required | Default  | Description                                       |
| ------------------------------- | -------- | -------- | ------------------------------------------------- |
| `BARDO_CAP_MAX_VALUE_PER_TX`    | No       | `10000`  | Max USD authorized per capability token           |
| `BARDO_CAP_MAX_VALUE_PER_TICK`  | No       | `50000`  | Max USD across all capabilities in a single tick  |
| `BARDO_CAP_MAX_VALUE_PER_DAY`   | No       | `100000` | Daily cumulative capability value cap (USD)       |
| `BARDO_CAP_EXPIRY_TICKS`        | No       | `3`      | Capability token expiry in heartbeat ticks        |
| `BARDO_MAX_SLIPPAGE_BPS`        | No       | `100`    | Maximum slippage tolerance (basis points)         |
| `BARDO_MAX_PRICE_IMPACT_BPS`    | No       | `300`    | Maximum price impact (basis points)               |
| `BARDO_TOKEN_ALLOWLIST_MODE`    | No       | `strict` | Token allowlist mode: strict, warn, off           |
| `BARDO_RATE_LIMIT_MAX_OPS`      | No       | `20`     | Max operations per time window                    |
| `BARDO_RATE_LIMIT_WINDOW`       | No       | `3600`   | Rate limit window in seconds                      |

### RPC overrides

| Variable             | Required | Default | Description             |
| -------------------- | -------- | ------- | ----------------------- |
| `BARDO_RPC_ETHEREUM` | No       | public  | Custom Ethereum RPC URL |
| `BARDO_RPC_BASE`     | No       | public  | Custom Base RPC URL     |
| `BARDO_RPC_ARBITRUM` | No       | public  | Custom Arbitrum RPC URL |
| `BARDO_RPC_OPTIMISM` | No       | public  | Custom Optimism RPC URL |
| `BARDO_RPC_POLYGON`  | No       | public  | Custom Polygon RPC URL  |

### External APIs and data

| Variable                  | Required | Default | Description                                  |
| ------------------------- | -------- | ------- | -------------------------------------------- |
| `BARDO_SUBGRAPH_API_KEY`  | No       | --      | The Graph API key (for higher rate limits)   |
| `BARDO_COINGECKO_API_KEY` | No       | --      | CoinGecko API key (fallback; x402 preferred) |

#### Uniswap Trading API

| Variable                             | Required | Default                                    | Description                                                                 |
| ------------------------------------ | -------- | ------------------------------------------ | --------------------------------------------------------------------------- |
| `BARDO_UNISWAP_API_KEY`              | No       | --                                         | API key from developers.uniswap.org. Required to activate API-backed tools. |
| `BARDO_UNISWAP_API_BASE_URL`         | No       | `https://trade-api.gateway.uniswap.org/v1` | Base URL override                                                           |
| `BARDO_UNISWAP_API_RATE_LIMIT`       | No       | `3`                                        | Requests per second (token bucket refill rate)                              |
| `BARDO_UNISWAP_API_QUOTE_MAX_AGE_MS` | No       | `30000`                                    | Quote cache TTL in milliseconds                                             |
| `BARDO_UNISWAP_API_GAS_BUFFER_PCT`   | No       | `20`                                       | Gas estimate buffer percentage                                              |
| `BARDO_UNISWAP_ROUTER_VERSION`       | No       | `2.0`                                      | Universal Router version (`1.2` or `2.0`)                                   |
| `BARDO_UNISWAP_PERMIT2_ENABLED`      | No       | `true`                                     | Enable Permit2 approval optimization                                        |
| `BARDO_UNISWAP_ENABLE_LP`            | No       | `true`                                     | Use API for LP operations (SDK fallback if false)                           |
| `BARDO_UNISWAP_ENABLE_LIMIT_ORDERS`  | No       | `true`                                     | Enable limit order tools                                                    |
| `BARDO_UNISWAP_ENABLE_SMART_WALLET`  | No       | `false`                                    | Enable EIP-5792/7702 tools                                                  |
| `BARDO_UNISWAP_CHAIN_IDS`            | No       | all supported                              | Comma-separated chain IDs to restrict API usage                             |

Uniswap API config is in the `[uniswap_api]` section of `golem.toml` (see full example above).

### ERC-8004 identity

| Variable                  | Required | Default | Description                                 |
| ------------------------- | -------- | ------- | ------------------------------------------- |
| `BARDO_IPFS_MODE`         | No       | `bardo` | IPFS mode: bardo, pinata, inline            |
| `BARDO_PINATA_JWT`        | No       | --      | Pinata JWT (required when IPFS_MODE=pinata) |
| `BARDO_AGENT_NAME`        | No       | --      | Human-readable agent name                   |
| `BARDO_AGENT_DESCRIPTION` | No       | --      | Agent description                           |

### Memory (Grimoire)

| Variable                              | Required | Default                    | Description                                         |
| ------------------------------------- | -------- | -------------------------- | --------------------------------------------------- |
| `BARDO_MEMORY_ENABLED`                | No       | `true`                     | Enable memory system (when learning profile active) |
| `BARDO_MEMORY_DATA_DIR`               | No       | `~/.bardo/memory`          | Root directory for memory files                     |
| `BARDO_MEMORY_LANCE_DIR`              | No       | `$DATA_DIR/lance`          | LanceDB episodic store directory                    |
| `BARDO_MEMORY_SQLITE_PATH`            | No       | `$DATA_DIR/semantic.db`    | SQLite semantic store path                          |
| `BARDO_MEMORY_MODEL`                  | No       | `Xenova/all-MiniLM-L6-v2`  | Embedding model ID                                  |
| `BARDO_MEMORY_DECAY_BASE_STABILITY`   | No       | `7`                        | Ebbinghaus base stability in days                   |
| `BARDO_MEMORY_CONSOLIDATION_INTERVAL` | No       | `14400`                    | ExpeL consolidation interval (seconds)              |
| `BARDO_MEMORY_MAX_EPISODES`           | No       | `100000`                   | Max episodic memory entries                         |
| `BARDO_MEMORY_MAX_INSIGHTS`           | No       | `10000`                    | Max semantic insights                               |

### x402 outbound

| Variable                        | Required | Default | Description                           |
| ------------------------------- | -------- | ------- | ------------------------------------- |
| `BARDO_X402_OUTBOUND_ENABLED`   | No       | `false` | Enable x402 outbound payments         |
| `BARDO_X402_OUTBOUND_KEY`       | No       | --      | Private key for USDC payments on Base |
| `BARDO_X402_MAX_SPEND_PER_HOUR` | No       | `1.00`  | Max x402 spend per hour (USD)         |
| `BARDO_X402_MAX_SPEND_PER_DAY`  | No       | `10.00` | Max x402 spend per day (USD)          |

---

## Startup validation

On startup, the tool library validates all configuration via serde deserialization with custom validation logic. Invalid or missing required values produce clear error messages with suggested fixes:

```text
Profile "trader" requires a wallet, but none is configured.

Configure one of:
  - BARDO_WALLET_PRIVATE_KEY  (dev only)
  - BARDO_PRIVY_APP_ID + BARDO_PRIVY_APP_SECRET  (production)

Or switch to the data profile: BARDO_PROFILE=data
```

Cross-field validation: `BARDO_IPFS_MODE=pinata` requires `BARDO_PINATA_JWT`.

### Structured startup log

```text
Bardo Tools v4.0.0 starting...

   Profile:     trader
   Chains:      ethereum (1), base (8453), arbitrum (42161)
   Wallet:      Privy server wallet (0xAbCd...1234)
   Safety:      max $1,000/tx - $10,000/day - strict token allowlist
   Tools:       27 adapters loaded
   A2A:         disabled

   No custom RPCs configured -- using public endpoints.
   Set BARDO_RPC_ETHEREUM for better reliability.

Ready. Extension registered with Pi session.
```

When `BARDO_LOG_FORMAT=json`, a structured JSON line is emitted instead.

---

## `--health` flag

```bash
bardo-tools --health
```

Checks connectivity to all configured services (RPCs, subgraph, wallet provider). Exit codes:

| Code | Meaning                                                  |
| ---- | -------------------------------------------------------- |
| `0`  | All services healthy                                     |
| `1`  | Degraded (some services failed, server will still start) |
| `2`  | Critical failure (server cannot start)                   |

---

## Config file schema (golem.toml)

Optional complement to environment variables. Useful for team-shared baseline configs (non-secret settings only). The config file is TOML format (`golem.toml`).

**File locations searched** (in order):

1. Path specified via `--config` CLI flag
2. `golem.toml` in the current directory
3. `~/.bardo/golem.toml`

### Full example

```toml
# golem.toml -- complete configuration reference

[tools]
profile = "trader"
enable = ["intel_compute_vpin"]
disable = ["uniswap_submit_uniswapx_order"]

[chains]
enabled = [1, 8453, 42161]

[chains.rpc_overrides]
1 = "https://my-rpc.example.com"

[custody]
# One of: "delegation", "embedded", "local_key"
mode = "delegation"

[custody.delegation]
smart_account = "0x..."
caveat_enforcers = ["GolemPhase", "MortalityTimeWindow", "DailySpendLimit"]

[custody.embedded]
privy_app_id = "..."
server_wallet_id = "..."

[custody.local_key]
private_key_path = ".bardo/session-key.json"
max_daily_spend_usd = 100.0
expires_days = 30

[safety]
token_allowlist_mode = "strict"
token_allowlist_source = "uniswap-default"

[safety.capability_limits]
max_value_per_tx_usd = 10000
max_value_per_day_usd = 100000
expiry_ticks = 3

[safety.slippage]
max_slippage_bps = 100
max_price_impact_bps = 300

[safety.rate_limit]
max_operations_per_window = 20
window_seconds = 3600

[safety.simulation]
enabled = true
divergence_tolerance_bps = 200

[trading]
default_deadline_seconds = 300
default_slippage_bps = 50
prefer_trading_api = true
enable_uniswapx = true
enable_cross_chain = true
confirmations = 1

[uniswap_api]
api_key = "your-key"
enable_lp = true
enable_limit_orders = true
enable_smart_wallet = false
universal_router_version = "2.0"
permit2_enabled = true
chain_ids = [1, 8453, 42161, 10, 137]

[a2a]
enabled = false
port = 3000

[logging]
level = "info"
format = "json"
```

### Rust config struct

The config file deserializes into a `ToolConfig` struct:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ToolConfig {
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub chains: ChainsConfig,
    #[serde(default)]
    pub custody: CustodyConfig,
    #[serde(default)]
    pub safety: SafetyConfig,
    #[serde(default)]
    pub trading: TradingConfig,
    #[serde(default)]
    pub a2a: A2aConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode")]
pub enum CustodyConfig {
    #[serde(rename = "delegation")]
    Delegation {
        smart_account: String,
        caveat_enforcers: Vec<String>,
    },
    #[serde(rename = "embedded")]
    Embedded {
        privy_app_id: String,
        server_wallet_id: String,
    },
    #[serde(rename = "local_key")]
    LocalKey {
        private_key_path: String,
        max_daily_spend_usd: f64,
        expires_days: u32,
    },
}

#[derive(Debug, Deserialize)]
pub struct SafetyConfig {
    pub token_allowlist_mode: AllowlistMode,
    pub capability_limits: CapabilityLimits,
    pub slippage: SlippageConfig,
    pub rate_limit: RateLimitConfig,
    pub simulation: SimulationConfig,
}

#[derive(Debug, Deserialize)]
pub struct CapabilityLimits {
    /// Max USD value a single Capability<T> token authorizes.
    pub max_value_per_tx_usd: f64,
    /// Max cumulative USD value per day across all capability tokens.
    pub max_value_per_day_usd: f64,
    /// Capability tokens expire after this many heartbeat ticks.
    pub expiry_ticks: u64,
}
```

Configuration loading merges env vars over file values over defaults. The merge follows serde's `#[serde(default)]` pattern -- env var overrides are applied after deserialization via a `merge_env_overrides()` pass.

---

## Three-tier API key model

Tiered access control for A2A endpoint connections. Governs what any external agent can do based on key scope.

### Key tiers

| Tier         | Prefix            | Scope                                                                                                             |
| ------------ | ----------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Read**     | `bardo_read_`     | All `get_*`, `search_*`, `list_*`, `retrieve_*`, `query_*`, `compute_*`, `check_*` tools. No state modifications. |
| **Feedback** | `bardo_feedback_` | Read + `manage_insight`, `update_memory_confidence`. Steering without transaction authority.                       |
| **Write**    | `bardo_write_`    | Full access including `execute_swap`, `vault_deposit`, `proxy_announce`, all execution tools.                     |

### Key generation

Keys are generated at first boot (if none exist) or via the CLI:

```bash
bardo-tools setup
```

Keys are cryptographically random, 256-bit, base64url-encoded. Total key length including prefix: ~55 characters.

### Enforcement middleware

```rust
fn authorize_tool_call(api_key: &str, tool_name: &str) -> bool {
    let tier = get_key_tier(api_key);
    match tier {
        KeyTier::Write => true,
        KeyTier::Feedback => is_read_tool(tool_name) || is_feedback_tool(tool_name),
        KeyTier::Read => is_read_tool(tool_name),
    }
}
```

### Rejection response

When a tool call is rejected due to insufficient key tier:

```json
{
  "isError": true,
  "content": [
    {
      "type": "text",
      "text": "{\"error\":\"INSUFFICIENT_KEY_TIER\",\"message\":\"Tool 'execute_swap' requires Write key. Current key tier: Read.\",\"requiredTier\":\"write\",\"currentTier\":\"read\"}"
    }
  ]
}
```

### Key rotation

```bash
bardo-tools setup --rotate-keys              # Rotate all keys
bardo-tools setup --rotate-keys --grace-period 15  # 15-min grace period
bardo-tools setup --rotate-key read           # Rotate one tier only
```

### Transport security

- HTTPS required for remote connections (non-localhost)
- HTTP allowed for localhost only
- Keys transmitted as `Authorization: Bearer` headers
- Keys never logged, never in URLs, never in error responses

---

## Data sources

### Primary data source matrix

| Data Need             | Primary Source               | Fallback              | Latency |
| --------------------- | ---------------------------- | --------------------- | ------- |
| Token price           | Pool slot0 via RPC           | Trading API quote     | < 500ms |
| Pool TVL/volume       | The Graph subgraph           | RPC multi-call        | < 2s    |
| Swap quotes           | Trading API                  | smart-order-router    | < 2s    |
| LP position state     | Direct RPC                   | Subgraph              | < 500ms |
| Token metadata        | On-chain ERC-20 + token list | CoinGecko             | < 500ms |
| Historical OHLCV      | Subgraph Swap events         | RPC event logs        | < 3s    |
| TokenJar balances     | Multi-call batch reads       | Subgraph              | < 1s    |
| ERC-8004 identity     | The Graph subgraphs          | Direct contract reads | < 2s    |
| External token data   | CoinGecko x402               | On-chain reads        | < 1s    |
| Multi-chain portfolio | Elsa x402                    | Multi-chain RPC reads | < 2s    |

### Caching strategy

| Cache Tier   | TTL     | Examples                                        |
| ------------ | ------- | ----------------------------------------------- |
| Short-lived  | 15s     | Token prices, gas prices, pool state            |
| Medium       | 5 min   | Pool metadata, token list                       |
| Long         | 24h     | Token metadata, contract addresses              |
| Pre-computed | 1h      | OHLCV candlesticks for popular pairs            |
| Immutable    | Forever | Completed candles, IPFS content (CID-addressed) |
| Never cached | --      | Simulation results, nonce state, streaming data |

The caching layer uses `moka` (a Rust concurrent cache) with per-tier TTL configuration. Circuit breaker logic wraps each external dependency with a 50% error threshold and 30s reset timeout. Fallback returns cached data with staleness indicators.

---

## Error taxonomy

| Category     | Prefix        | Description                  | Examples                                                       |
| ------------ | ------------- | ---------------------------- | -------------------------------------------------------------- |
| `safety`     | `SAFETY_`     | Blocked by safety middleware | `SAFETY_TOKEN_NOT_ALLOWED`, `SAFETY_SPENDING_LIMIT_EXCEEDED`   |
| `validation` | `VALIDATION_` | Invalid input parameters     | `VALIDATION_INVALID_ADDRESS`, `VALIDATION_CHAIN_NOT_SUPPORTED` |
| `execution`  | `EXECUTION_`  | Transaction execution failed | `EXECUTION_TX_REVERTED`, `EXECUTION_TX_TIMEOUT`                |
| `data`       | `DATA_`       | Data source errors           | `DATA_SUBGRAPH_ERROR`, `DATA_RPC_ERROR`                        |
| `wallet`     | `WALLET_`     | Wallet/signing errors        | `WALLET_NOT_CONFIGURED`, `WALLET_SIGNING_FAILED`               |
| `routing`    | `ROUTING_`    | Route finding errors         | `ROUTING_NO_ROUTE`, `ROUTING_INSUFFICIENT_LIQUIDITY`           |
| `config`     | `CONFIG_`     | Configuration errors         | `CONFIG_MISSING_ENV`, `CONFIG_INVALID_VALUE`                   |
| `capability` | `CAP_`        | Capability token errors      | `CAP_EXPIRED`, `CAP_VALUE_EXCEEDED`, `CAP_NOT_MINTED`         |

---

## Performance requirements

| Operation                 | Target  | Maximum |
| ------------------------- | ------- | ------- |
| Read (prices, pool info)  | < 500ms | < 2s    |
| Quote                     | < 1s    | < 3s    |
| Write (swap + simulation) | < 5s    | < 15s   |
| Token search              | < 1s    | < 3s    |
| Cold start                | < 3s    | --      |
| Warm start                | < 1s    | --      |

Throughput: 1 write/s sustained, 5 reads/s sustained. Single-agent use (one wallet, sequential operations). Cold start targets are lower than the TypeScript predecessor because binary startup eliminates JIT warmup.

---

## Dependencies

### Core

| Crate                   | Purpose                                     |
| ----------------------- | ------------------------------------------- |
| `alloy`                 | EVM interaction, providers, signers         |
| `alloy-sol-types`       | ABI encoding/decoding via sol! macro        |
| `serde` / `serde_json`  | Serialization, parameter parsing            |
| `tokio`                 | Async runtime                               |
| `reqwest`               | HTTP client (Trading API, Privy, subgraph)  |
| `moka`                  | Concurrent caching                          |
| `tracing`               | Structured logging                          |
| `thiserror`             | Error type definitions                      |
| `toml`                  | Config file parsing                         |

### TypeScript sidecar

Uniswap's official SDKs (`@uniswap/sdk-core`, `@uniswap/v3-sdk`, `@uniswap/v4-sdk`, `@uniswap/universal-router-sdk`, `@uniswap/permit2-sdk`, `@uniswap/uniswapx-sdk`) are TypeScript-only. A thin Node.js sidecar process exposes SDK math and calldata encoding over local IPC. The sidecar is started lazily when a tool first needs SDK computation. Tools that only do RPC reads or subgraph queries never start the sidecar.

### Runtime

- **Target**: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`
- **Rust edition**: 2024
- **MSRV**: 1.80
- **No external services required** for basic operation (all data from RPC + subgraph)
- **Optional**: Uniswap Trading API key, The Graph API key, TypeScript sidecar (for SDK-dependent tools)
