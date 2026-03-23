# Configuration Reference: golem.toml [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-14

> **Reader orientation:** This document is the full configuration schema for `golem.toml`, the single file that describes a Golem's (mortal autonomous agent compiled as a single Rust binary running on a micro VM) runtime behavior. It belongs to the `shared/` reference layer and is the canonical source for all configuration fields, defaults, and environment variable mappings. Before diving in, understand that a Golem's configuration spans identity, heartbeat timing, inference routing, safety constraints, custody, mortality clocks, and more. See `prd2/shared/glossary.md` for full term definitions.

---

This is the canonical schema reference for `golem.toml`, the single configuration file that fully describes a Golem's runtime behavior. All other PRD2 documents should cross-reference this file rather than duplicating config schemas.

`golem.toml` is consumed by the `bardo-golem` binary at startup. Missing fields fall back to defaults. Invalid fields cause a fatal boot error with a specific error code and message.

---

## Config Resolution Order

Four sources, highest priority first:

1. **CLI flags** -- `--custody delegation`, `--inference-provider venice,bardo`, etc.
2. **Environment variables** -- `GOLEM_*` for per-golem runtime, `BARDO_*` for platform services
3. **golem.toml** -- the file
4. **Built-in defaults** -- hardcoded in `golem-core`

Secrets (wallet keys, API keys, Privy credentials) are only accepted from env vars, keystore files, or interactive stdin prompts. Never from CLI flags or config files.

---

## Env Var Naming Convention

| Prefix | Scope | Examples |
|--------|-------|---------|
| `BARDO_*` | Platform services (Styx, Compute, Inference gateway) | `BARDO_STYX_HOST`, `BARDO_COMPUTE_REGION` |
| `GOLEM_*` | Per-golem runtime config | `GOLEM_NAME`, `GOLEM_CUSTODY_MODE`, `GOLEM_TICK_INTERVAL` |

---

## `[golem]` -- Core Identity

```toml
[golem]
# Human-readable identifier. AI-generated or "golem-{random6}".
name = "eth-dca-alpha"

# Strategy category. Determines default tool profile and heartbeat tuning.
# Values: "dca", "yield", "lp", "momentum", "multi-protocol", "custom"
strategy_category = "dca"

# Target chain. "base-sepolia" for first-time creators, "base" for mainnet.
# Values: "base", "base-sepolia", "anvil"
network = "base-sepolia"

# Deployment mode. "hosted" = Fly.io VM via x402. "self-hosted" = user's machine.
mode = "hosted"

# Initial USDC funding in human-readable units.
funding = "50.00"

# Custody mode. See [custody] section for mode-specific config.
# Values: "delegation", "embedded", "localkey"
custody_mode = "delegation"

# Transfer restriction tier.
# Values: "strict" (main wallet only), "clade" (+ sibling wallets),
#          "network" (+ any ERC-8004 agent), "unrestricted" (any address)
transfer_restriction = "strict"

# Schema version. Monotonically increasing integer. Current: 1.
schema_version = 1
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `name` | string | `"golem-{nanoid(6)}"` | `GOLEM_NAME` |
| `strategy_category` | enum | `"custom"` | `GOLEM_STRATEGY_CATEGORY` |
| `network` | enum | `"base-sepolia"` | `GOLEM_NETWORK` |
| `mode` | enum | `"hosted"` | `GOLEM_MODE` |
| `funding` | string (decimal) | `"50.00"` | `GOLEM_FUNDING` |
| `custody_mode` | enum | `"delegation"` | `GOLEM_CUSTODY_MODE` |
| `transfer_restriction` | enum | `"strict"` | `GOLEM_TRANSFER_RESTRICTION` |
| `schema_version` | u32 | `1` | -- |

---

## `[heartbeat]` -- Tick Pipeline

Controls the Heartbeat (the 9-step decision cycle: observe, retrieve, analyze, gate, simulate, validate, execute, verify, reflect). See `prd2/01-golem/02-heartbeat.md` S17.

```toml
[heartbeat]
# Base theta interval in seconds. The Adaptive Clock adjusts this within
# [30s, 120s] based on regime multipliers. At 60s base, ~1,440 ticks/day.
base_interval_seconds = 60

# Prediction error threshold for LLM escalation (System 1 -> System 2 gate).
# Lower = more LLM calls = more expensive. Higher = cheaper but less responsive.
base_deliberation_threshold = 0.3

# Hard daily inference cost cap in USD. Ticks degrade to T0-only above this.
max_daily_cost_usd = 10.0

# Fraction of daily cap at which a cost warning fires.
cost_warning_threshold = 0.7

# Fraction of daily cap that triggers T2 suppression (T1 still allowed).
cost_soft_cap_threshold = 0.9

# Episode write batch size for Grimoire persistence.
write_batch_size = 25

# Episode write batch flush interval in milliseconds.
write_batch_flush_interval_ms = 30000

[heartbeat.regime_multipliers]
# Interval multipliers by market regime. >1.0 = slower ticks, <1.0 = faster.
trending_up = 1.0
trending_down = 0.5    # More alert in downtrends
volatile = 0.3         # Maximum frequency during volatility
range_bound = 2.0      # Relax during calm markets
unknown = 0.8

[heartbeat.probe_thresholds]
# Thresholds for System 1 probe evaluation (basis points and ratios).
price_delta_low_bps = 50       # Below: no signal
price_delta_high_bps = 200     # Above: strong signal -> T1/T2 escalation
health_factor_low = 1.5        # Above: safe
health_factor_high = 1.2       # Below: danger zone
world_model_drift_low = 0.10   # Below: model fits
world_model_drift_high = 0.25  # Above: regime may have changed
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `base_interval_seconds` | u64 | `15` | `GOLEM_TICK_INTERVAL` |
| `base_deliberation_threshold` | f64 | `0.3` | `GOLEM_DELIBERATION_THRESHOLD` |
| `max_daily_cost_usd` | f64 | `10.0` | `GOLEM_MAX_DAILY_COST` |
| `cost_warning_threshold` | f64 | `0.7` | -- |
| `cost_soft_cap_threshold` | f64 | `0.9` | -- |

---

## `[inference]` -- Provider Config and Model Routing

Controls which LLM providers handle inference and how payment works. See `prd2/01-golem/01-cognition.md` S13.

```toml
[inference]
# Payment mode for autonomous inference.
# "golem_wallet" = pay from Golem's USDC wallet via x402.
# "prepaid" = pre-purchased API key with provider.
# "diem" = Venice DIEM staking (zero marginal cost).
# "composite" = primary + fallback payment.
payment = "golem_wallet"

# Daily inference budget in USD (for golem_wallet payment mode).
daily_budget_usd = 5.0

# Ordered provider list. First provider whose resolve() succeeds handles the request.
# Putting Venice first routes privacy-preferring intents there.
# Putting Bardo first routes cost-optimized intents there.
[[inference.providers]]
type = "bardo"
# api_key loaded from BARDO_INFERENCE_API_KEY env var

[[inference.providers]]
type = "venice"
diem = true
# api_key loaded from VENICE_API_KEY env var

[[inference.providers]]
type = "local"
base_url = "http://localhost:11434/v1"
```

Provider types: `bardo`, `venice`, `bankr`, `anthropic`, `openai`, `google`, `deepseek`, `local`.

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `payment` | enum | `"golem_wallet"` | `GOLEM_INFERENCE_PAYMENT` |
| `daily_budget_usd` | f64 | `5.0` | `GOLEM_INFERENCE_DAILY_BUDGET` |
| Provider API keys | string | -- | `BARDO_INFERENCE_API_KEY`, `VENICE_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GOOGLE_API_KEY`, `DEEPSEEK_API_KEY` |

---

## `[safety]` -- PolicyCage

On-chain and off-chain constraint enforcement. See `prd2/10-safety/02-policy.md`.

```toml
[safety]
# Approved token addresses. Only these can be traded/held.
approved_assets = [
  "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",  # USDC on Base
  "0x4200000000000000000000000000000000000006",  # WETH on Base
]

# Approved protocol contract addresses.
approved_protocols = [
  "0x2626664c2603336E57B271c5C0b26F421741e481",  # Uniswap V3 Router
]

# Maximum number of distinct assets the Golem can hold.
max_asset_count = 10

# Maximum single-position size as basis points of total portfolio.
max_position_bps = 2500          # 25%

# Maximum concentration in a single protocol (bps).
max_concentration_bps = 3000     # 30%

# Minimum collateral ratio for lending positions (bps).
min_collateral_ratio_bps = 12500 # 125%

# Maximum drawdown before write operations are blocked (bps).
max_drawdown_bps = 1300          # 13%

# Drawdown measurement window in seconds.
drawdown_window = 86400          # 1 day

# Minimum seconds between rebalance operations.
min_rebalance_interval = 3600    # 1 hour

# Maximum rebalances per 24h period.
max_rebalances_per_day = 24

# Function selector whitelist. Empty = all selectors allowed (not recommended).
# strategy_whitelist = ["0x12345678", "0xabcdef01"]

# Allow arbitrary calldata (bypasses selector whitelist). Default: false.
allow_arbitrary_calldata = false

# Chainalysis OFAC sanctions oracle address.
sanction_oracle = "0x40C57923924B5c5c5455c48D93317139ADDaC8fb"

[safety.spending_limits]
# Per-transaction limit in USD.
per_transaction = 10000
# Per-session limit in USD.
per_session = 50000
# Per-day limit in USD.
per_day = 100000
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `approved_assets` | Address[] | `[]` | -- |
| `max_position_bps` | u32 | `2500` | -- |
| `max_concentration_bps` | u32 | `3000` | -- |
| `max_drawdown_bps` | u32 | `1300` | -- |
| `spending_limits.per_transaction` | u64 | `10000` | `GOLEM_SPEND_LIMIT_TX` |
| `spending_limits.per_day` | u64 | `100000` | `GOLEM_SPEND_LIMIT_DAILY` |

---

## `[custody]` -- Wallet and Signing

Three modes with fundamentally different trust models. See `prd2/10-safety/01-custody.md`.

```toml
[custody]
# Mode: "delegation" (recommended), "embedded" (legacy), "localkey" (dev)
mode = "delegation"

# -- Delegation mode config --
# smart_account = "0x..."  # Optional: specific MetaMask Smart Account address
# caveat_enforcers = [...]  # Optional: custom ERC-7710 caveat enforcer configs

# -- Embedded mode config --
# privy_app_id loaded from PRIVY_APP_ID env var

# -- LocalKey mode config --
# private_key_path = "/path/to/keystore.json"  # Never put raw keys in TOML

[custody.delegation_bounds]
# Only applies to localkey mode. Bounds the damage if the key leaks.
max_daily_spend_usd = 1000.0
max_total_calls = 10000
# expires_at = 1735689600  # Unix timestamp
# allowed_targets = ["0x..."]
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `mode` | enum | `"delegation"` | `GOLEM_CUSTODY_MODE` |
| Private key (localkey only) | -- | -- | `GOLEM_PRIVATE_KEY` (stdin preferred) |
| Privy App ID (embedded only) | string | -- | `PRIVY_APP_ID` |
| Privy App Secret (embedded only) | string | -- | `PRIVY_APP_SECRET` |

---

## `[styx]` -- Styx Connectivity

Full Styx (global knowledge relay and persistence layer at wss://styx.bardo.run; three tiers: Vault/Clade/Lethe) relay configuration. See `tmp/research/styx-interation2/S4-clade-sync-v4.3.md`.

Styx is fully optional. Setting `enabled = false` runs the Golem on its local Grimoire alone (~95% capability). All sub-sections inherit the master switch.

```toml
[styx]
# Master switch. false = local Grimoire only, no network features.
enabled = true                       # default: true
# Styx WebSocket endpoint.
host = "styx.bardo.run"             # default: "styx.bardo.run"

[styx.vault]
# L0 backup of Grimoire snapshots to Styx.
enabled = true                       # default: true
ttl = "90d"                          # Backup retention period
backup_interval_ticks = 200          # Full backup every ~50 minutes

[styx.clade]
# Peer-to-peer knowledge sync with sibling Golems.
enabled = true                       # default: true
sync_interval_ticks = 50             # Batch sync every Curator cycle
immediate_warnings = true            # Push warnings outside batch cycle
immediate_bloodstains = true         # Push bloodstains outside batch cycle

[styx.clade.p2p]
# Optional direct P2P for same-network siblings (bypasses Styx relay).
enabled = false                      # default: false
# listen_address = "127.0.0.1:8402"
# peers = ["ws://10.0.0.2:8402"]

[styx.lethe]
# L2 anonymized Lethe (formerly Commons) participation.
read_enabled = true                  # default: true
publish_enabled = true               # default: true
domains = ["dex-lp", "lending", "gas", "regime"]

[styx.pheromone]
# Pheromone Field: swarm intelligence signals.
enabled = true                       # default: true
deposit_enabled = true               # Deposit pheromone signals
sense_enabled = true                 # Read pheromone signals

[styx.marketplace]
# Knowledge marketplace participation.
browse_enabled = true                # default: true
auto_discover = true                 # Golem autonomously searches marketplace
max_auto_spend = 1.00                # Max USDC per auto-purchase
min_seller_reputation = 60           # Only buy from Verified+ sellers
sell_enabled = false                 # Opt-in to create listings

[styx.budget]
# Hard spending limits for all Styx operations.
max_per_tick = 0.01                  # Max USDC per tick
daily_budget = 0.50                  # Max USDC per day
monthly_budget = 10.00               # Max USDC per month
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `enabled` | bool | `true` | `BARDO_STYX_ENABLED` |
| `host` | string | `"styx.bardo.run"` | `BARDO_STYX_HOST` |
| `clade.enabled` | bool | `true` | `BARDO_CLADE_ENABLED` |
| `clade.sync_interval_ticks` | u64 | `50` | -- |
| `budget.daily_budget` | f64 | `0.50` | `BARDO_STYX_DAILY_BUDGET` |
| `budget.monthly_budget` | f64 | `10.00` | `BARDO_STYX_MONTHLY_BUDGET` |

---

## `[succession]` -- Generational Learning

Controls whether and how knowledge transfers between predecessor and successor Golems. See `prd2/02-mortality/07-succession.md` and `prd2/02-mortality/13-configuration.md`.

```toml
[succession]
# Enable owner-initiated succession (not automatic -- owner decides).
enabled = true                       # default: true

# Automatically create a successor when this Golem dies.
# When false, the owner must explicitly trigger succession via the UI or API.
auto = false                         # default: false

# Budget allocated from the Golem's remaining funds for successor creation.
# Only used when auto = true.
budget_usdc = 50.0                   # default: 50.0

# Maximum allowed strategy drift between predecessor and successor.
# 0.0 = exact clone, 1.0 = completely different strategy.
strategy_drift_allowed = 0.3         # default: 0.3

# Whether to seed the successor with the predecessor's Grimoire.
inherit_grimoire = true              # default: true

# Confidence applied to inherited Grimoire entries.
# Lower values force more independent validation.
# Protocol invariant: cannot exceed 0.7.
inheritance_confidence = 0.4         # default: 0.4

# Minimum PLAYBOOK.md divergence from predecessor (anti-proletarianization).
min_playbook_divergence = 0.15       # default: 0.15 (15%)

# Whether to also inherit from online Clade siblings.
inherit_from_clade = true            # default: true

# Use SUPER (Surprise-Based Experience Sharing) for novelty ranking of inherited entries.
use_novelty_ranking = true           # default: true
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `enabled` | bool | `true` | -- |
| `auto` | bool | `false` | `GOLEM_SUCCESSION_AUTO` |
| `budget_usdc` | f64 | `50.0` | `GOLEM_SUCCESSION_BUDGET` |
| `inherit_grimoire` | bool | `true` | -- |
| `inheritance_confidence` | f64 | `0.4` | -- |

---

## `[daimon]` -- Affect Engine

Controls the Daimon (the affect engine implementing PAD -- Pleasure-Arousal-Dominance -- emotional state as a control signal) that modulates decision-making under mortality pressure. See `prd2/02-mortality/13-configuration.md` (AffectConfig).

```toml
[daimon]
# Enable the daimon engine. When false, all decisions are "neutral."
enabled = true                       # default: true

# Appraisal model for emotional computation.
# "chain_of_emotion" = LLM-based (1 Haiku call per non-suppressed tick).
# "rule_based" = deterministic rules mapping events to emotions ($0.00).
# "disabled" = no appraisal; always Neutral.
appraisal_model = "chain_of_emotion" # default: "chain_of_emotion"

# EMA decay rate for mood state. Higher = slower mood shifts (more stable).
mood_decay_rate = 0.95               # default: 0.95, range: 0.80-0.99

# Whether mortality state amplifies emotional responses.
mortality_aware_affect = true        # default: true (mortal), false (immortal)

# Ticks of modified behavior after a Clade sibling dies.
grief_duration_ticks = 100           # default: 100, range: 10-500

# Record emotional context on Grimoire entries.
record_emotional_context = true      # default: true
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `enabled` | bool | `true` | `GOLEM_DAIMON_ENABLED` |
| `appraisal_model` | enum | `"chain_of_emotion"` | `GOLEM_APPRAISAL_MODEL` |
| `mood_decay_rate` | f64 | `0.95` | -- |
| `mortality_aware_affect` | bool | `true` | -- |

---

## `[dreams]` -- Dream Cycle

Controls the three-phase offline learning cycle (NREM replay, REM imagination, Integration). See `prd2/05-dreams/01-architecture.md`.

```toml
[dreams]
# Enable dreaming. When false, no offline consolidation occurs.
enabled = true                       # default: true

# Scheduling mode.
# "operator" = dreams only during configured windows.
# "autonomous" = Golem decides when to dream based on urgency score.
# "hybrid" = operator windows + autonomous dreaming above threshold.
schedule = "hybrid"                  # default: "hybrid"

# Urgency threshold for autonomous/hybrid dreaming (0.0-1.0).
# Lower = dreams more often. Higher = only dreams when urgency is high.
autonomous_threshold = 0.8           # default: 0.8

# Fraction of inference budget allocated to dreaming.
budget_fraction = 0.08               # default: 0.08 (8%)

# Vitality (composite survival score from 0.0 to 1.0) threshold below which dreaming ceases entirely.
# Terminal Golems redirect all resources to the Thanatopsis (Death Protocol -- four-phase structured shutdown: Acceptance, Settlement, Reflection, Legacy) Protocol.
terminal_cutoff = 0.15               # default: 0.15

# Maximum staged dream hypotheses waiting for live validation.
max_staged_revisions = 10            # default: 10

# Dream inference provider override (uses main inference config if unset).
# Set to "venice" for private no-log dreaming while using Bardo for waking.
# dream_inference_provider = "venice"

# Web search budget per dream cycle in USDC (separate from inference budget).
web_search_budget_per_cycle_usdc = 0.05  # default: 0.05

[dreams.windows]
# Sleep windows for operator/hybrid modes. Heartbeat suspends during these.
windows = [
  { start = "02:00", end = "06:00", timezone = "UTC" },
]

[dreams.phase_scaling]
# Budget allocation across dream phases (must sum to 1.0).
nrem = 0.45                          # Replay and consolidation
rem = 0.35                           # Imagination and counterfactuals
integration = 0.20                   # PLAYBOOK.md updates and validation
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `enabled` | bool | `true` | `GOLEM_DREAMS_ENABLED` |
| `schedule` | enum | `"hybrid"` | `GOLEM_DREAM_SCHEDULE` |
| `budget_fraction` | f64 | `0.08` | -- |
| `terminal_cutoff` | f64 | `0.15` | -- |
| `dream_inference_provider` | string? | (inherits from `[inference]`) | `GOLEM_DREAM_INFERENCE_PROVIDER` |

---

## `[oracle]` -- Prediction Engine

Master switch for the prediction subsystem. See `prd2/01-golem/17-prediction-engine.md`.

```toml
[oracle]
# Master switch for the prediction engine.
# When false: no predictions, no action gating, no attention foraging.
# Epistemic fitness switches to P&L proxy (see [mortality.epistemic]).
# Default: true. Recommended for all trading and LP strategies.
# Set false for data-only, read-only, or passive strategies.
enabled = true                     # default: true | env: GOLEM_ORACLE_ENABLED

residual_buffer_size = 256         # Entries per (category, regime). 8 bytes each.
target_coverage = 0.85
min_correction_samples = 10
novelty_threshold = 2.0
forgetting_rate = 0.005
compaction_window = 604800         # 7 days in seconds

[oracle.attention]
active_max = 15
watched_max = 60
scanned_max = 500
watched_eval_frequency = 4
scanned_eval_frequency = 100
promotion_threshold = 3.0
demotion_patience = 10

[oracle.gate]
category_threshold = 0.60
inaction_comparison = true
inaction_margin = 0.05
inheritance_coefficient = 0.70

[oracle.calibration]
enabled = true
min_samples = 30
refit_interval = 50
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `enabled` | bool | `true` | `GOLEM_ORACLE_ENABLED` |
| `residual_buffer_size` | u32 | `256` | -- |
| `target_coverage` | f64 | `0.85` | -- |
| `attention.active_max` | u32 | `15` | -- |
| `gate.category_threshold` | f64 | `0.60` | -- |
| `calibration.enabled` | bool | `true` | -- |

---

## `[mortality]` -- Three Clocks

Controls the triple-clock mortality system: economic (resource exhaustion), epistemic (informational decay), stochastic (random death probability). See `prd2/02-mortality/13-configuration.md`.

```toml
[mortality]
# Master switch. When true, disables ALL mortality clocks.
# Only valid for self-hosted mode. Tracked for the falsifiable experiment.
immortal = false                     # default: false

[mortality.economic]
# Death by resource exhaustion.
enabled = true                       # default: true (hosted), false (self-hosted)
death_reserve_floor_usdc = 0.30      # Minimum: 0.10 (protocol invariant)
death_reserve_proportional = 0.02    # Range: 0.01-0.10
conservation_threshold = 0.30        # Range: 0.10-0.50
terminal_threshold = 0.05            # Range: 0.01-0.20, must be < conservation

[mortality.economic.partitions]
# Credit partition allocations (must sum to 1.0).
llm = 0.60
gas = 0.25
data = 0.15

[mortality.economic.daily_caps]
# Per-partition daily budget caps as fraction of partition balance.
llm = 0.20
gas = 0.30
data = 0.25

[mortality.epistemic]
# Death by informational decay.
enabled = true                       # default: true
fitness_window = 2000                # Ticks for rolling fitness. Range: 500-10000
senescence_threshold = 0.35          # Fitness below which senescence begins. Range: 0.20-0.60
recovery_grace_period = 500          # Ticks to recover before death. Range: 100-2000
# Oracle-mode fields (used when [oracle] enabled = true):
minimum_predictions = 50             # Min predictions before fitness is valid. Range: 10-500
tracked_prediction_types = [
  "price_direction",
  "volatility_regime",
  "fee_rate",
  "liquidity_depth",
]
# P&L proxy fields (used when [oracle] enabled = false):
pnl_minimum_trades = 10              # Min executed trades before fitness is valid
pnl_fitness_window = 200             # Rolling window in trade count (not ticks)

[mortality.stochastic]
# Ever-present random death probability.
enabled = true                       # default: true
base_hazard_rate = 1e-6              # Per-tick death probability. Range: 1e-8 to 1e-4
age_hazard_coefficient = 0.00001     # Age contribution to hazard. Range: 0 to 0.001
epistemic_hazard_multiplier = 2.0    # Multiplier when fitness < 0.5. Range: 1.0-10.0
max_hazard_rate = 0.001              # Protocol invariant: cannot exceed 0.01
# seed = 42                          # Fixed PRNG seed for reproducible testing

[mortality.demurrage]
# Knowledge carrying cost (Gesell's Freigeld for information).
enabled = true                       # default: true (mortal), false (immortal)
validation_interval_ticks = 5000     # Range: 1000-20000
decay_rate_per_interval = 0.05       # Range: 0.01-0.20
archive_threshold = 0.1              # Range: 0.01-0.30

[mortality.demurrage.decay_classes]
# Half-lives in days. Omit for no decay.
# structural = <none>                # Protocol mechanics, ABIs: no decay
regime_conditional = 14.0            # Market regime models
tactical = 7.0                       # Entry/exit signals, spread patterns
ephemeral = 1.0                      # Order book snapshots, current prices

[mortality.phage]
# Micro-Replicant (phage) system for continuous self-testing.
enabled = true                       # default: true
daily_spawn_rate = 2                 # Range: 0-10
max_lifetime_ticks = 50              # Range: 10-200
budget_per_phage = 0.05              # USDC per phage. Range: 0.01-1.00
minimum_vitality_to_spawn = 0.5      # Range: 0.3-0.8

[mortality.thanatopsis]
# Enhanced Death Protocol: emotional life review and knowledge compression.
enabled = true                       # default: true
reflect_budget_fraction = 0.60       # Range: 0.30-0.80
emotional_life_review = true
include_unresolved_questions = true
max_reflection_tokens = 8000         # Range: 2000-32000
snapshot_interval_ticks = 50         # Range: 10-200
generate_successor_recommendation = true
novelty_prioritized_packaging = true
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `immortal` | bool | `false` | `BARDO_IMMORTAL` |
| `economic.enabled` | bool | `true` | `BARDO_MORTALITY_ENABLED` |
| `stochastic.seed` | u64? | `None` (crypto RNG) | `BARDO_STOCHASTIC_SEED` |
| `epistemic.senescence_threshold` | f64 | `0.35` | -- |
| `demurrage.enabled` | bool | `true` | -- |

---

## `[compute]` -- VM Tier

Controls the Fly.io VM configuration for hosted Golems. Ignored for self-hosted. See `prd2/11-compute/00-overview.md`.

```toml
[compute]
# Deployment target. "hosted" = Fly.io VM. "self-hosted" = user's machine.
mode = "hosted"                      # default: "hosted"

# VM size tier. Determines CPU, RAM, and hourly cost.
# "micro"  = shared-cpu-1x / 256MB, $0.025/hr
# "small"  = shared-cpu-1x / 512MB, $0.05/hr  (default)
# "medium" = shared-cpu-2x / 1GB,   $0.10/hr
# "large"  = performance-2x / 2GB,  $0.20/hr
tier = "small"                       # default: "small"
```

| Field | Type | Default | Env Var |
|-------|------|---------|---------|
| `mode` | enum | `"hosted"` | `GOLEM_MODE` |
| `tier` | enum | `"small"` | `GOLEM_COMPUTE_TIER` |

---

## Complete Example: `eth-dca` Template

A conservative DCA Golem on Base Sepolia testnet with default safety parameters.

```toml
# golem.toml -- eth-dca template
# Generated by: bardo-golem create --template eth-dca

[golem]
name = "eth-dca-weekly"
strategy_category = "dca"
network = "base-sepolia"
mode = "hosted"
funding = "50.00"
custody_mode = "delegation"
transfer_restriction = "strict"
schema_version = 1

[heartbeat]
base_interval_seconds = 120        # DCA: relaxed theta base, Adaptive Clock won't go below 30s
base_deliberation_threshold = 0.4  # Higher threshold = fewer LLM calls
max_daily_cost_usd = 2.0           # DCA is low-cost

[heartbeat.regime_multipliers]
trending_up = 1.0
trending_down = 0.8
volatile = 0.5
range_bound = 3.0                  # Very relaxed during calm markets
unknown = 1.0

[inference]
payment = "golem_wallet"
daily_budget_usd = 2.0

[[inference.providers]]
type = "bardo"

[safety]
approved_assets = [
  "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",  # USDC
  "0x4200000000000000000000000000000000000006",  # WETH
]
max_asset_count = 2
max_position_bps = 5000            # DCA can hold up to 50% in ETH
max_concentration_bps = 5000
max_drawdown_bps = 2000            # 20% drawdown tolerance for DCA
max_rebalances_per_day = 4
allow_arbitrary_calldata = false

[safety.spending_limits]
per_transaction = 500
per_session = 2000
per_day = 5000

[custody]
mode = "delegation"

[styx]
enabled = true

[styx.clade]
enabled = true

[styx.budget]
daily_budget = 0.10
monthly_budget = 2.00

[succession]
enabled = true
auto = false
inherit_grimoire = true
inheritance_confidence = 0.4

[daimon]
enabled = true
appraisal_model = "rule_based"     # No LLM cost for DCA's simple emotional landscape

[dreams]
enabled = true
schedule = "operator"
budget_fraction = 0.05             # DCA needs less dream time

[dreams.windows]
windows = [
  { start = "03:00", end = "05:00", timezone = "UTC" },
]

[mortality]
immortal = false

[mortality.economic]
enabled = true

[mortality.epistemic]
enabled = true
fitness_window = 5000              # Longer window -- DCA changes slowly
senescence_threshold = 0.25        # More tolerant of fitness dips

[mortality.stochastic]
enabled = true

[compute]
mode = "hosted"
tier = "micro"                     # DCA is lightweight
```

---

## Full Env Var Table

| Env Var | Config Path | Type | Notes |
|---------|-------------|------|-------|
| `GOLEM_NAME` | `golem.name` | string | |
| `GOLEM_STRATEGY_CATEGORY` | `golem.strategy_category` | enum | |
| `GOLEM_NETWORK` | `golem.network` | enum | |
| `GOLEM_MODE` | `golem.mode` / `compute.mode` | enum | |
| `GOLEM_FUNDING` | `golem.funding` | string | |
| `GOLEM_CUSTODY_MODE` | `golem.custody_mode` / `custody.mode` | enum | |
| `GOLEM_TRANSFER_RESTRICTION` | `golem.transfer_restriction` | enum | |
| `GOLEM_TICK_INTERVAL` | `heartbeat.base_interval_seconds` | u64 | |
| `GOLEM_DELIBERATION_THRESHOLD` | `heartbeat.base_deliberation_threshold` | f64 | |
| `GOLEM_MAX_DAILY_COST` | `heartbeat.max_daily_cost_usd` | f64 | |
| `GOLEM_INFERENCE_PAYMENT` | `inference.payment` | enum | |
| `GOLEM_INFERENCE_DAILY_BUDGET` | `inference.daily_budget_usd` | f64 | |
| `BARDO_INFERENCE_API_KEY` | `inference.providers[bardo].api_key` | string | Secret |
| `VENICE_API_KEY` | `inference.providers[venice].api_key` | string | Secret |
| `ANTHROPIC_API_KEY` | `inference.providers[anthropic].api_key` | string | Secret |
| `OPENAI_API_KEY` | `inference.providers[openai].api_key` | string | Secret |
| `GOOGLE_API_KEY` | `inference.providers[google].api_key` | string | Secret |
| `DEEPSEEK_API_KEY` | `inference.providers[deepseek].api_key` | string | Secret |
| `GOLEM_PRIVATE_KEY` | `custody` (localkey mode) | string | Secret, stdin preferred |
| `PRIVY_APP_ID` | `custody` (embedded mode) | string | Secret |
| `PRIVY_APP_SECRET` | `custody` (embedded mode) | string | Secret |
| `GOLEM_SPEND_LIMIT_TX` | `safety.spending_limits.per_transaction` | u64 | |
| `GOLEM_SPEND_LIMIT_DAILY` | `safety.spending_limits.per_day` | u64 | |
| `BARDO_STYX_ENABLED` | `styx.enabled` | bool | |
| `BARDO_STYX_HOST` | `styx.host` | string | |
| `BARDO_CLADE_ENABLED` | `styx.clade.enabled` | bool | |
| `BARDO_STYX_DAILY_BUDGET` | `styx.budget.daily_budget` | f64 | |
| `BARDO_STYX_MONTHLY_BUDGET` | `styx.budget.monthly_budget` | f64 | |
| `GOLEM_SUCCESSION_AUTO` | `succession.auto` | bool | |
| `GOLEM_SUCCESSION_BUDGET` | `succession.budget_usdc` | f64 | |
| `GOLEM_ORACLE_ENABLED` | `oracle.enabled` | bool | Restart required |
| `GOLEM_DAIMON_ENABLED` | `daimon.enabled` | bool | |
| `GOLEM_APPRAISAL_MODEL` | `daimon.appraisal_model` | enum | |
| `GOLEM_DREAMS_ENABLED` | `dreams.enabled` | bool | |
| `GOLEM_DREAM_SCHEDULE` | `dreams.schedule` | enum | |
| `GOLEM_DREAM_INFERENCE_PROVIDER` | `dreams.dream_inference_provider` | string | |
| `BARDO_MORTALITY_ENABLED` | `mortality.economic.enabled` | bool | Master mortality switch |
| `BARDO_IMMORTAL` | `mortality.immortal` | bool | |
| `BARDO_STOCHASTIC_SEED` | `mortality.stochastic.seed` | u64 | For testing |
| `BARDO_EXPERIMENT_ENABLED` | -- | bool | Immortal vs mortal experiment tracking |
| `GOLEM_COMPUTE_TIER` | `compute.tier` | enum | |

---

## Cross-References

- `prd2/01-golem/06-creation.md` -- GolemCoreManifest and GolemExtendedManifest (defines the manifest schema for creating and provisioning new Golems)
- `prd2/01-golem/02-heartbeat.md` S17 -- HeartbeatConfig struct (specifies the 9-step autonomous decision loop with Adaptive Clock and regime multipliers)
- `prd2/01-golem/01-cognition.md` S13 -- GolemInferenceConfig struct (covers the T0/T1/T2 cognitive tier routing and LLM provider configuration)
- `prd2/10-safety/02-policy.md` -- PolicyCageConfig (specifies the on-chain smart contract enforcing safety constraints: approved assets, position limits, spending caps)
- `prd2/10-safety/01-custody.md` -- Three custody modes (delegation via MetaMask Smart Accounts, embedded via Privy TEE, and local key for dev)
- `prd2/02-mortality/13-configuration.md` -- Full MortalityConfig with all sub-configs (economic, epistemic, stochastic clocks plus demurrage and phage subsystems)
- `prd2/05-dreams/01-architecture.md` -- Dream scheduling and budget (three-phase offline learning: NREM replay, REM imagination, Integration)
- `prd2/09-economy/02-clade.md` -- CladeConfig (sibling Golem knowledge exchange through Styx)
- `prd2/11-compute/00-overview.md` -- VM tiers and pricing (Fly.io micro VM provisioning via x402 micropayments)
- `tmp/research/styx-interation2/S4-clade-sync-v4.3.md` -- Full Styx TOML block (detailed configuration for the global knowledge relay and persistence layer)
