# Bardo Tools -- Tool Profiles [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` | **Prerequisites**: [01-architecture.md](01-architecture.md)
>
> Profile-based tool loading for the `bardo-tools` Pi extension.

---

> **Reader orientation:** This document specifies the profile-based tool loading system for the `bardo-tools` crate, part of Bardo's DeFi tool library. Profiles control which tool adapters a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) loads at boot, determining what actions it can take. Understanding the two-layer tool model from `01-architecture.md` is a prerequisite. See `prd2/shared/glossary.md` for full term definitions.

## Profile system

Set `BARDO_PROFILE` in the environment to control which tool adapters the `bardo-tools` extension loads at boot. Profiles determine which `actionType` values are valid for `preview_action`/`commit_action`. Profiles are composable -- `BARDO_PROFILE=trader,vault` activates both.

| Profile          | Categories                                                                                                      | Use case                                                               |
| ---------------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `active`         | data, trading, lp, lending, staking, restaking, derivatives, yield, safety, intelligence, memory, identity, wallet, streaming | **Default.** Full trading capability for standard active Golems        |
| `observatory`    | data, streaming                                                                                                 | **Sleepwalker phenotype.** Read-only observation, no wallet needed. Observes, dreams, publishes, never trades |
| `conservative`   | data, trading (limited), safety, streaming                                                                      | Limited writes -- no leverage, no complex LP, no flashloans. Risk-averse owner configuration |
| `trader`         | data, trading, safety, streaming                                                                                | Swap execution, quotes, approvals, MEV assessment                      |
| `lp`             | data, trading, lp, safety, streaming                                                                            | Liquidity provision, position management, fee collection, optimization |
| `vault`          | data, vault, safety                                                                                             | ERC-4626 vault operations, proxy management                            |
| `vault-curator`  | data, vault, lp, trading, safety, intelligence, streaming                                                       | Full vault management -- trading + LP + vault operations               |
| `intelligence`   | data, intelligence                                                                                              | MEV scoring, IL calculation, venue comparison, token discovery         |
| `learning`       | data, intelligence, memory                                                                                      | Memory management, episodic/semantic queries, self-improvement         |
| `identity`       | data, identity                                                                                                  | Agent identity, registration, validation registry queries              |
| `full`           | all except testnet, bootstrap                                                                                   | All tools registered (power users)                                     |
| `development`    | all                                                                                                             | Full + testnet + bootstrap tools                                       |
| `evaluation`     | data, safety, intelligence                                                                                      | Eval harness -- read + safety checks + intelligence analysis           |
| `minimal`        | data                                                                                                            | Bare minimum -- data reads only, no streaming                          |

### 17 tool categories

| Category     | Tool count | Description                                                        |
| ------------ | ---------- | ------------------------------------------------------------------ |
| data         | ~40        | Pool info, token prices, positions, portfolio, historical queries  |
| trading      | ~20        | Swap execution, quotes, approvals, UniswapX, limit orders          |
| lending      | ~15        | Aave/Compound supply, borrow, repay, flash loans                   |
| staking      | ~10        | Liquid staking (Lido stETH/wstETH), native staking                |
| restaking    | ~8         | EigenLayer restaking, operator delegation                          |
| derivatives  | ~12        | Perpetuals, options, structured products                           |
| yield        | ~10        | Yield aggregation, auto-compounding, strategy vaults               |
| lp           | ~21        | LP position management, fee collection, migration, optimization   |
| vault        | ~12        | ERC-4626 vault operations, am-AMM bidding, proxy management       |
| safety       | ~7         | Transaction simulation, risk assessment, token validation          |
| intelligence | ~10        | MEV scoring, IL calculation, venue comparison, token discovery     |
| memory       | ~16        | Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links) operations, episodic/semantic store, consolidation        |
| identity     | ~8         | ERC-8004 agent identity, reputation, on-chain registration        |
| wallet       | ~12        | Wallet status, policy config, session keys, funding, migration    |
| streaming    | ~7         | Real-time event subscriptions (pool events, prices, alerts)       |
| testnet      | ~5         | Local testnet setup, time travel, mock deployments                |
| bootstrap    | ~3         | Initial setup, RPC config, wallet provisioning, identity creation |

Total: ~210 tools across 17 categories. 12 categories have existing implementations; 5 (lending, staking, restaking, derivatives, yield) are specified but not yet implemented.

### Profile composition

Multiple profiles can be combined:

```bash
# Activate both trader and vault profiles
BARDO_PROFILE=trader,vault

# Activate data + intelligence
BARDO_PROFILE=data,intelligence
```

The `data` category is implicitly included in all profiles. The `full` profile includes everything except testnet and bootstrap. The `development` profile extends `full` with testnet and bootstrap tools.

### Observatory profile detail

The `observatory` profile is the Sleepwalker phenotype's tool configuration. It loads only read tools (~72 total). No wallet is needed -- the Sleepwalker never signs transactions. It observes the market, dreams about what it sees, and publishes structural understanding to the Lethe (formerly Commons). Because it never trades, it has no alpha to leak and can publish freely.

### Conservative profile detail

The `conservative` profile loads all read tools plus a restricted set of ~40 write tools. Excluded: leverage operations, complex LP (concentrated liquidity below 50-tick ranges), flashloan tools, cross-chain bridge tools. Write operations are rate-limited to 10 per hour (vs 20 for standard profiles). Intended for risk-averse owners who want their Golem to trade but within tight bounds.

### Profile filtering

Profile filtering uses the `ToolDef.category` field. Filtering happens once at extension initialization, not per-request:

```rust
let allowed = resolve_profile_categories(profile);
let tools: Vec<&ToolDef> = ALL_TOOL_DEFS
    .iter()
    .filter(|t| allowed.contains(&t.category))
    .collect();
```

### Fine-grained overrides

The config file supports per-tool enable/disable that takes precedence over profiles:

```toml
[tools]
profile = "trader"
enable = ["intel_compute_vpin", "intel_compute_lvr"]
disable = ["uniswap_submit_uniswapx_order"]
```

---

## Profile registry

The `ProfileRegistry` loads tool adapters based on the active profile at boot. Each adapter wraps a `ToolDef` and registers its action type with the session's action system. Profile filtering uses category-based matching against `ToolDef.category`.

```rust
/// Profile registry: resolves profile names to allowed tool categories.
pub struct ProfileRegistry {
    profiles: HashMap<String, HashSet<Category>>,
}

impl ProfileRegistry {
    /// Load tools for the given profile. Called once at startup.
    pub fn load_profile(&self, profile: &str) -> Vec<&'static ToolDef> {
        let categories = self.profiles.get(profile)
            .expect("Unknown profile");

        ALL_TOOL_DEFS
            .iter()
            .filter(|t| categories.contains(&t.category))
            .collect()
    }

    /// Load tools for composite profiles (e.g., "trader,vault").
    pub fn load_composite(&self, profiles: &[&str]) -> Vec<&'static ToolDef> {
        let mut categories = HashSet::new();
        for profile in profiles {
            if let Some(cats) = self.profiles.get(*profile) {
                categories.extend(cats.iter());
            }
        }

        ALL_TOOL_DEFS
            .iter()
            .filter(|t| categories.contains(&t.category))
            .collect()
    }
}
```

The two-layer tool model (8 Pi-facing tools wrapping ~423 underlying adapters) means context window cost is fixed regardless of how many adapters are loaded. The inference gateway's L4 layer prunes these down to ~10-15 per request via semantic search.

Profile filtering happens once at extension initialization. All adapters matching the profile are available immediately. No per-request filtering, no session caps, no activation sequences.

---

## Profile-to-category mapping

| Profile        | data | trading | lending | staking | restaking | derivatives | yield | lp  | vault | safety | intelligence | memory | identity | wallet | streaming | testnet | bootstrap |
| -------------- | ---- | ------- | ------- | ------- | --------- | ----------- | ----- | --- | ----- | ------ | ------------ | ------ | -------- | ------ | --------- | ------- | --------- |
| active         | Yes  | Yes     | Yes     | Yes     | Yes       | Yes         | Yes   | Yes | Yes   | Yes    | Yes          | Yes    | Yes      | Yes    | Yes       | --      | Yes       |
| observatory    | Yes  | --      | --      | --      | --        | --          | --    | --  | --    | --     | --           | --     | --       | --     | Yes       | --      | --        |
| conservative   | Yes  | Yes*    | --      | --      | --        | --          | --    | --  | --    | Yes    | --           | --     | --       | --     | Yes       | --      | --        |
| trader         | Yes  | Yes     | --      | --      | --        | --          | --    | --  | --    | Yes    | --           | --     | --       | --     | Yes       | --      | --        |
| lp             | Yes  | Yes     | --      | --      | --        | --          | --    | Yes | --    | Yes    | --           | --     | --       | --     | Yes       | --      | --        |
| vault          | Yes  | --      | --      | --      | --        | --          | --    | --  | Yes   | Yes    | --           | --     | --       | --     | --        | --      | --        |
| vault-curator  | Yes  | Yes     | --      | --      | --        | --          | --    | Yes | Yes   | Yes    | Yes          | --     | --       | --     | Yes       | --      | --        |
| intelligence   | Yes  | --      | --      | --      | --        | --          | --    | --  | --    | --     | Yes          | --     | --       | --     | --        | --      | --        |
| learning       | Yes  | --      | --      | --      | --        | --          | --    | --  | --    | --     | Yes          | Yes    | --       | --     | --        | --      | --        |
| identity       | Yes  | --      | --      | --      | --        | --          | --    | --  | --    | --     | --           | --     | Yes      | --     | --        | --      | --        |
| full           | Yes  | Yes     | Yes     | Yes     | Yes       | Yes         | Yes   | Yes | Yes   | Yes    | Yes          | Yes    | Yes      | Yes    | Yes       | --      | Yes       |
| development    | Yes  | Yes     | Yes     | Yes     | Yes       | Yes         | Yes   | Yes | Yes   | Yes    | Yes          | Yes    | Yes      | Yes    | Yes       | Yes     | Yes       |
| evaluation     | Yes  | --      | --      | --      | --        | --          | --    | --  | --    | Yes    | Yes          | --     | --       | --     | --        | --      | --        |
| minimal        | Yes  | --      | --      | --      | --        | --          | --    | --  | --    | --     | --           | --     | --       | --     | --        | --      | --        |

\* `conservative` includes ~40 restricted write tools (no leverage, no complex LP, no flashloans).

### Profile design notes

The `active` profile is the default for standard Golems. It includes nearly everything except testnet tools (not used in production). The `full` profile is identical in category coverage but is the canonical name for non-Golem power users.

The `observatory` profile is the Sleepwalker's configuration: ~72 read-only tools, no wallet needed. The `conservative` profile sits between `observatory` (read-only) and `active` (full write) -- it allows trades but blocks high-risk operations.

The `data` and `observatory` profiles include `streaming` because real-time price feeds and pool event subscriptions are read-only operations. Any profile that needs market awareness gets streaming automatically.

The `trader` and `lp` profiles both include `safety` because any profile that can execute on-chain writes must have safety middleware available for simulation, risk assessment, and token validation.

The `learning` profile includes `intelligence` in addition to `memory`. Self-improvement tools need intelligence tools (prediction comparison, regime classification) to function. It remains narrow -- no trading, no LP, no write capability.

### Approximate tool counts per profile

| Profile        | Read tools | Write tools | Total |
| -------------- | ---------- | ----------- | ----- |
| active         | ~250       | ~150        | ~400  |
| observatory    | ~72        | 0           | ~72   |
| conservative   | ~250       | ~40         | ~290  |
| trader         | ~47        | ~27         | ~74   |
| lp             | ~47        | ~48         | ~95   |
| vault          | ~47        | ~12         | ~59   |
| vault-curator  | ~47        | ~60         | ~107  |
| intelligence   | ~50        | 0           | ~50   |
| learning       | ~66        | ~6          | ~72   |
| identity       | ~48        | ~8          | ~56   |
| full           | ~250       | ~173        | ~423  |
| development    | ~250       | ~178        | ~428  |
| evaluation     | ~57        | 0           | ~57   |
| minimal        | ~40        | 0           | ~40   |

Counts are approximate because new tools are added per batch. The canonical source is `ALL_TOOL_DEFS.len()` at build time.
