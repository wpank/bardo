# Wallet & Funding [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Reader orientation:** This document specifies how Golems (mortal autonomous agents compiled as single Rust binaries running on micro VMs) are funded -- from initial provisioning through self-sustaining operation. It covers four funding sources (direct USDC, inline swap, bridge, fiat on-ramp), Permit2 batch approval for Embedded custody, delegation grants for the recommended Delegation custody, the funding recommendation formula, and the metabolic self-funding loop where a Golem's trading revenue pays for its own inference and compute. It belongs to the `01-golem` economic layer. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## S1 -- Overview

Every Golem needs funded operations before it can act. How those operations are funded depends on the custody mode (see `06-creation.md` S5.1). This document specifies the four funding sources, Permit2 batch approval integration (for Embedded custody), delegation grant flow (for Delegation custody), the strategy-aware funding recommendation formula, and x402/Styx as ongoing funding mechanisms for extending a Golem's lifespan.

---

## S2 -- Four Funding Sources

The funding step either grants a delegation (Delegation mode) or transfers USDC to the Golem wallet address (Embedded mode). In both cases, the owner's Main Wallet is the source of authority or funds.

| Source                   | UX                                         | Time                               | Fees                           | Best For                                 |
| ------------------------ | ------------------------------------------ | ---------------------------------- | ------------------------------ | ---------------------------------------- |
| **Direct USDC transfer** | Address + QR code                          | ~30s (same chain)                  | Gas only (~$0.01 on Base)      | Users with USDC on Base already          |
| **Inline swap**          | Select source token, review quote, confirm | ~15s                               | 0.3% swap fee + gas            | Users with ETH/other tokens on Base      |
| **Bridge**               | LI.FI widget (cross-chain)                 | 1--15 min                          | Bridge fee + gas (both chains) | Users with funds on other chains         |
| **Fiat on-ramp**         | Privy + MoonPay widget                     | 1--5 min (card) / 1--3 days (bank) | 1.5--3.5%                      | New crypto users with no on-chain assets |

In Delegation mode, the first three sources fund the owner's Smart Account (where funds stay). The Golem spends from that account via delegation. In Embedded mode, the sources transfer directly to the Golem's Privy wallet.

### 2.1 Direct USDC Transfer

The simplest path. Display the target wallet address and a QR code (EIP-681 payment URI format). Poll for incoming USDC transfers at 10-second intervals until the target amount is reached. Timeout after 10 minutes with option to retry or switch method.

### 2.2 Inline Token Swap

Users who have tokens other than USDC on the target chain should not have to leave the onboarding flow to find a DEX. The wizard offers an inline swap, executed from the owner's Main Wallet, with the output USDC sent directly to the target address.

The wizard detects Main Wallet balances and displays swappable tokens sorted by USD value. The user selects a source token and enters an amount (or clicks "Max"). A quote is fetched showing input amount, output amount, price impact, slippage, and gas. One transaction executes the swap (or one off-chain signature + one transaction with Permit2).

**Minimum viable implementation**: ETH -> USDC swap on Base via Uniswap V3 (deepest liquidity pair). Extend to WETH, DAI, USDT in v1.1.

### 2.3 Bridge (Cross-Chain)

For users with funds on other chains, the wizard embeds the LI.FI widget for cross-chain token bridging. The widget is configured with the target wallet as the recipient (`toAddress`), USDC as the output token, and the target chain.

Supported source chains: Ethereum (1), Arbitrum (42161), Optimism (10), Polygon (137), Base (8453, same-chain fallback to swap). Typical times: 1--3 minutes (optimistic bridges), 5--15 minutes (canonical bridges). Completion detected via the same polling mechanism as direct transfer.

### 2.4 Fiat On-Ramp (MoonPay)

For users with no crypto at all, the wizard offers fiat on-ramp via Privy's built-in MoonPay integration. Email and wallet address auto-populate from the Privy session. KYC is handled by MoonPay. Minimum purchase: $20.

Supported payment methods (jurisdiction-dependent):

| Method        | Speed                    | Fee       |
| ------------- | ------------------------ | --------- |
| Credit card   | Instant                  | 2.5--3.5% |
| Debit card    | Instant                  | 1.5--2.5% |
| Bank transfer | 1--3 business days       | 0.5--1.5% |
| Apple Pay     | Instant (iOS/Safari)     | 2.5--3.5% |
| Google Pay    | Instant (Android/Chrome) | 2.5--3.5% |

---

## S3 -- Permit2 Batch Approval (Embedded Mode)

Permit2 is a Uniswap-deployed contract at the canonical address `0x000000000022D473030F116dDEE9F6B43aC78BA3` that enables off-chain signature-based token transfers [PERMIT2-2022]. This section applies only to Embedded (Privy) custody mode, where funds are actually transferred to a Golem wallet.

### 3.1 Flow

1. **One-time approval**: `USDC.approve(PERMIT2, type(uint256).max)` -- a single on-chain transaction, done once per token per chain
2. **Per-funding signature**: Sign an off-chain EIP-712 `PermitTransferFrom` message authorizing a specific USDC amount to a specific Golem wallet (no gas cost)
3. **On-chain execution**: The provisioning pipeline calls `permit2.permitTransferFrom()` with the signature, executing the transfer atomically

The EIP-712 typed data is constructed using alloy's type system:

```rust
use alloy::primitives::{Address, U256};
use alloy::sol;

sol! {
    struct TokenPermissions {
        address token;
        uint256 amount;
    }

    struct PermitTransferFrom {
        TokenPermissions permitted;
        uint256 nonce;
        uint256 deadline;
    }

    struct SignatureTransferDetails {
        address to;
        uint256 requestedAmount;
    }
}
```

### 3.2 Gas Savings

| Approach                         | Transactions | Approx. Gas (Base)       | User Interactions |
| -------------------------------- | ------------ | ------------------------ | ----------------- |
| Traditional (approve + transfer) | 2            | ~163K gas (~$0.02)       | 2 wallet popups   |
| Permit2 (first time)             | 1 tx + 1 sig | ~120--140K gas (~$0.015) | 1 popup + 1 sig   |
| Permit2 (subsequent)             | 0 tx + 1 sig | ~120--140K gas (~$0.015) | 1 sig only        |

On Base with sub-$0.01 gas, the primary benefit is UX (fewer wallet popups), not cost.

### 3.3 Batch Permit (v2)

In v2, Permit2 `PermitBatchTransferFrom` enables a single EIP-712 signature to authorize multiple token transfers atomically -- for example, funding a Golem with both USDC and WETH for gas in a single interaction.

### 3.4 Approval Check

Before requesting a Permit2 signature, the wizard checks current allowance. If already approved (previous Golem creation or DeFi interaction), the approval step is skipped entirely.

---

## S3b -- Delegation Grant Flow (Delegation Mode)

In Delegation custody mode, no token transfer occurs. Instead, the owner signs a single ERC-7715 `wallet_grantPermissions` request from their MetaMask Smart Account.

### Flow

1. **Owner connects MetaMask**: The Smart Account address is confirmed
2. **Sign delegation**: A single ERC-7715 signature grants the Golem's session key spending authority, bounded by caveat enforcers
3. **Done**: No on-chain transaction needed for the grant itself (caveat deployers may require prior deployment)

```rust
use alloy::primitives::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationGrant {
    /// The owner's MetaMask Smart Account.
    pub delegator: Address,
    /// The Golem's ephemeral session key address.
    pub delegate: Address,
    /// Caveat enforcers bounding the delegation.
    pub caveats: Vec<CaveatEnforcer>,
    /// EIP-712 signature from the owner.
    pub signature: Vec<u8>,
    /// Unique salt preventing replay.
    pub salt: u64,
}

/// Seven custom caveat enforcers (see ../10-safety/01-custody.md for full spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaveatEnforcer {
    GolemPhase { vitality_oracle: Address, golem_address: Address },
    MortalityTimeWindow { start_time: u64, end_time: u64 },
    DreamMode { dream_oracle: Address, golem_address: Address },
    VaultNAV { vault_address: Address, max_nav_pct: u16 },
    ReplicantBudget { max_budget_usd: u64, max_lifespan_seconds: u64 },
    MaxSlippage { max_slippage_bps: u16 },
    DailySpendLimit { daily_limit_usd: u64 },
}
```

The owner sees one MetaMask popup. The delegation is active immediately. Revocation is also a single on-chain transaction from MetaMask -- no Golem cooperation needed.

---

## S4 -- Funding Recommendation Formula

A static default of $10 would be insufficient for most strategies. A Golem created with "weekly ETH DCA, $100/week" and funded with $10 cannot execute a single trade -- the worst possible first-run experience. The funding recommendation is computed from the strategy's actual parameters.

### 4.1 Formula

```
F = (daily_cost x duration) x safety_margin
```

Where:

- `daily_cost = (trade_cost x trades_per_day) + gas_per_day + compute_per_day + inference_per_day`
- `duration` = target runway in days (default: 14 days / 2 weeks)
- `safety_margin` = 1.10 (10% buffer)

Plus a fixed death reserve:

```
death_reserve = $0.30 base + (num_token_positions x estimated_sweep_gas_cost)
```

In Delegation mode, the death reserve covers only gas for settlement transactions (no sweep needed -- funds are already in the owner's wallet).

### 4.2 Recommendation Tiers

| Tier            | Definition                                                                                                                                                               |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Minimum**     | One trade cycle + 3 cycles of gas + death reserve. Floor: $25 (mainnet only; testnet has no practical minimum floor). Below this, the Golem cannot execute its strategy. |
| **Recommended** | 2 weeks of projected operation + 10% buffer + death reserve.                                                                                                             |

### 4.3 Breakdown Display

The wizard shows a full breakdown so the user understands what their money covers:

```
Trade execution:    $200.00  (2 weeks x $100/week)
Gas fees:            $0.28   (2 swaps x $0.14/swap on Base)
Compute (hosted):    $3.36   (2 weeks x $1.68/week)
Inference:           $0.84   (2 weeks x $0.42/week)
Death reserve:       $0.30   (Golem knowledge preservation)
Buffer (10%):       $20.48
---
Recommended:        $225.26 USDC
Minimum:            $103.44 USDC (1 trade + gas + reserve)
```

### 4.4 Below-Minimum Warning

If the user enters an amount below the minimum, the wizard blocks with an explanation:

```
[!] $50.00 is below the minimum ($103.44) for your strategy.
    Your Golem cannot execute a single $100 trade with this funding.
    Reduce trade size or increase funding.
```

### 4.5 Template-Aware Defaults

When a strategy template is selected, its `defaultFunding` values override the generic defaults. Templates provide both mainnet and testnet defaults calibrated to their specific strategy parameters.

---

## S5 -- Insufficient Funds Flow

When the owner's Main Wallet balance is below the required amount:

1. **"Add Funds"** opens the funding source picker (direct transfer, inline swap, bridge, fiat on-ramp). After adding funds, the balance is re-checked and the wizard updates.

2. **"Reduce to $X"** auto-adjusts the funding amount to fit the available balance. Available only when balance >= $25 absolute floor.
   - If reduced amount >= minimum viable: warning that runway is shorter than recommended
   - If reduced amount < minimum viable but >= floor: strong warning that the Golem will be observation-only
   - If balance < floor ($25): "Reduce" button disabled. "Fund at least $25.00 to proceed."

3. **Balance < $25 absolute floor**: Both buttons disabled. The user must add funds before proceeding. "Your Golem needs at least $25.00 for gas, compute, and inference costs to operate for one week."

---

## S6 -- Funding Validation

Before transitioning from the funding step to provisioning, five validation checks must pass:

| Check               | Condition                              | Failure Behavior                            |
| ------------------- | -------------------------------------- | ------------------------------------------- |
| Floor check         | `fundingAmount >= $25`                 | Hard block                                  |
| Balance check       | User's USDC balance >= `fundingAmount` | Show insufficient funds flow                |
| Gas check           | User's ETH balance covers gas estimate | Warn if insufficient                        |
| Permit2 check       | Permit2 approval status (Embedded only)| Prompt for one-time approval                |
| Death reserve check | Reserve included in funding amount     | Warn if user manually reduced below reserve |

---

## S7 -- x402 and Styx as Ongoing Funding Mechanisms

Beyond initial provisioning, two mechanisms enable ongoing funding for extending a Golem's lifespan.

### 7.1 x402 Micropayments

The x402 protocol enables HTTP-native micropayments. A Golem publishes an x402 payment endpoint. Sending USDC to this endpoint:

1. Credits the Golem's operational balance
2. Extends the projected lifespan proportionally (based on current burn rate)
3. Emits a `golem.funded` webhook to the owner

### 7.2 Styx x402 as Ongoing Funding

Styx operations (clade sync, pheromone reads, bloodstain deposits, marketplace queries) are paid via x402 micropayments from the Golem's operational balance. These are small amounts (see `styx-interation2/S4-clade-sync-v4.3.md` for pricing: ~$0.007--$0.011/day typical), but they represent continuous operational costs that feed directly into the economic clock (`05-mortality.md`).

For self-funding Golems, Styx costs are part of the metabolic loop: the Golem trades to earn revenue that pays for inference, gas, compute, and Styx queries. The sustainability ratio (`revenue / total_costs`) determines whether the Golem's economic clock ticks up or down.

### 7.3 Anyone Can Fund

The x402 model decouples funding from ownership. The owner creates the Golem and provides initial funding, but anyone can extend its life:

- **The owner** can top up to keep a profitable Golem running
- **Other agents** can fund Golems whose capabilities they consume (pay-per-invocation)
- **Third parties** can fund Golems that provide valuable public goods (market data, research, signals)
- **Clade siblings** can pool resources to keep a specialist Golem alive

### 7.4 Feed Option at Death

When a Golem enters the dying phase (15% credits remaining), its `golem.dying` webhook includes a `feedOption`:

```json
{
  "feedOption": {
    "amount": 5.0,
    "extendsLifeHours": 28
  }
}
```

If the owner (or anyone) sends the specified USDC amount before `exit(0)`, the Death Protocol suspends and the Golem re-enters the desperate phase with a fresh budget allocation.

### 7.5 Relationship to Permit2

Initial provisioning funding uses Permit2 (off-chain signature, batched with other setup steps) in Embedded mode, or delegation grant (off-chain signature) in Delegation mode. Ongoing x402 funding is separate -- direct USDC transfers to the Golem's payment endpoint, no Permit2 signature required.

---

## S8 -- Wallet Hierarchy and Custody

Golem wallets exist within a hierarchy whose shape depends on the custody mode:

### Delegation Mode (Recommended)

| Level                | Type                                    | Custody                         | Purpose                                                  |
| -------------------- | --------------------------------------- | ------------------------------- | -------------------------------------------------------- |
| **Owner Wallet**     | MetaMask Smart Account                  | Owner-controlled                | Holds all funds. Grants delegations. Receives settlements.|
| **Session Key**      | Ephemeral keypair in Golem process      | Process-memory, caveat-bounded  | Signs UserOperations under delegation                    |

Funds never leave the owner's wallet. The session key is disposable -- compromise is bounded by caveat enforcers. The owner revokes from MetaMask directly.

### Embedded Mode (Legacy)

| Level                | Type                                    | Custody                         | Purpose                                                   |
| -------------------- | --------------------------------------- | ------------------------------- | --------------------------------------------------------- |
| **Main Wallet**      | Privy embedded wallet (browser-side)    | User-controlled                 | Funds Golem creation, receives swept funds on destruction |
| **Golem Wallet**     | Privy server wallet (TEE-hosted)        | TEE-managed, policy-constrained | Holds operational funds, executes trades                  |
| **Replicant Wallet** | Subordinate server wallet               | TEE-managed, parent-constrained | Holds funds for child Golems                              |

Each tier has transfer restrictions controlling where funds can flow. The signing policy enforced in the TEE prevents the Golem from sending funds to unauthorized addresses:

| Restriction    | Allowed Destinations                             |
| -------------- | ------------------------------------------------ |
| `strict`       | Owner's Main Wallet only                         |
| `clade`        | Owner's Main Wallet + owner's other Golems       |
| `unrestricted` | Any address (advanced, full autonomy)            |

> **Security Requirement:** `unrestricted` mode requires all transfers to route through PolicyCage enforcement with additional time-delay hardening recommended. No instant unrestricted transfers.

Write actions are enforced by the PolicyCage (see `prd2/10-safety/02-policy.md`). An optional time-delayed proxy (Warden) provides additional hardening with configurable cancellation windows per risk tier; see `prd2-extended/10-safety/02-warden.md`.

---

## S9 -- Wallet Persistence

Wallet persistence depends on custody mode:

**Delegation mode:** The owner's Smart Account persists independently of everything. If the VM dies, the delegation simply expires. The owner retains full control through MetaMask. No sweep, no recovery, no stuck funds.

**Embedded mode:** The Golem wallet persists independently of the Golem's compute instance. If the VM dies, the wallet and its funds remain accessible:

- The owner can sweep funds via the app dashboard
- The provisioning pipeline can reconnect to the wallet on resume
- Tombstone records link to the wallet address for post-mortem reconciliation

Key material never leaves the Privy TEE enclave. The Golem VM holds only the session signer (P-256 key), which is policy-constrained and time-bounded. Loss of the VM does not compromise the wallet -- the owner retains ownership through Privy's account recovery mechanism.

---

## S10 -- Self-Funding Economics: The Metabolic Loop

A Golem with revenue can pay for its own existence. Jonas (1966) described metabolism as self-sustaining through the very activity that requires sustenance. A self-funding Golem achieves the computational equivalent: it trades to earn revenue that pays for the inference that produces better trades.

### 10.1 Revenue Sources

Self-funding does not assume vault management fees. Revenue comes from whatever the Golem's strategy produces:

- **Trading profit** -- direct P&L from executed trades (DCA timing, momentum capture, arbitrage)
- **LP fee revenue** -- collected fees from concentrated liquidity positions
- **Keeper fees** -- compensation for executing strategy actions on behalf of other agents
- **Knowledge sales** -- revenue from Grimoire exports sold via the Styx marketplace
- **Vault management fees** -- optional, only if the Golem operates a vault

The first three are available to any trading Golem. Vault income is one possible revenue stream, not the default assumption.

### 10.2 The Loop

```
Revenue (trading, LP fees, keeper rewards)
    |
    v
Operational Balance (USDC)
    |
    +---> Inference costs (LLM calls via x402 or Bankr)
    +---> Gas costs (on-chain transactions)
    +---> Compute costs (Fly.io VM via x402)
    +---> Styx costs (clade sync, pheromone queries)
    |
    v
Net: revenue > costs --> economic clock ticks UP
     revenue < costs --> economic clock ticks DOWN
```

The sustainability ratio feeds directly into the economic clock (see `02-mortality/01-architecture.md`). A Golem with `sustainability_ratio > 1.0` has an economic clock that ticks up -- it can live indefinitely (barring epistemic or stochastic death). Below `0.5`, the Golem is dying fast.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicState {
    /// Revenue / total cost. >1.0 = self-sustaining. >2.0 = thriving.
    pub sustainability_ratio: f64,
    pub daily_revenue_usd: f64,
    pub daily_costs: DailyCosts,
    /// Feeds the economic clock.
    pub projected_lifespan_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCosts {
    pub inference_usd: f64,
    pub gas_usd: f64,
    pub compute_usd: f64,
    pub styx_usd: f64,
}
```

### 10.3 Bankr as Fifth Inference Provider

Bankr's LLM Gateway provides a single API key routing to 20+ models (Claude, Gemini, GPT, DeepSeek). What makes it distinct from Bardo Inference, OpenRouter, Venice, or direct provider APIs: the inference provider and the execution wallet are the same system. A Golem using Bankr can call models and execute on-chain transactions through the same account, closing the metabolic loop within a single service.

The five inference providers (configured via `golem.toml` `[inference]` section, see `06-creation.md` S5.2):

| Provider | Payment | Models | Distinct Property |
|---|---|---|---|
| Bardo Inference | x402 micropayment | Claude, GPT | Default. Pay-per-call from operational wallet. |
| Bankr LLM Gateway | Bankr wallet balance | 20+ models | Wallet = API key. Revenue and inference share one account. |
| Venice | API key | Open-weight models | Privacy-preserving inference. No logging. |
| OpenRouter | API key | 100+ models | Broadest model selection. |
| Direct (Anthropic/OpenAI/Google) | API key | Provider-specific | Lowest latency. Requires separate API key per provider. |

### 10.4 Mortality-Aware Model Routing

As the economic clock ticks down, the Golem shifts to cheaper models. This is rational inattention applied to model selection: a dying Golem burning $5/day on Claude Opus when Gemini Flash at $0.50/day would keep it alive is making an irrational allocation.

```rust
pub fn select_inference_tier(
    vitality: f64,
    task_criticality: f64,
    sustainability_ratio: f64,
) -> InferenceTier {
    if vitality < 0.3 {
        return if task_criticality > 0.9 { InferenceTier::T1 } else { InferenceTier::T0 };
    }
    if vitality < 0.5 || sustainability_ratio < 1.0 {
        return if task_criticality > 0.7 { InferenceTier::T1 } else { InferenceTier::T0 };
    }
    if task_criticality > 0.8 { InferenceTier::T2 }
    else if task_criticality > 0.4 { InferenceTier::T1 }
    else { InferenceTier::T0 }
}
```

### 10.5 Cross-Model Verification

For high-stakes decisions (swaps above $500, LP rebalances above $1000), the Golem queries two different models and compares structured outputs. Both must agree (cosine similarity >= 0.7 on the structured output) before execution proceeds. Bankr's multi-model access makes this a single-API operation.

### 10.6 Self-Funding Configuration

```toml
[self_funding]
enabled = true
inference_allocation_fraction = 0.6   # 60% of revenue allocated to inference
throttle_balance_usd = 5.0            # Throttle inference below this balance
provider_order = ["bardo", "bankr"]   # Fallback order
```

When `enabled = true`, the Golem tracks `MetabolicState` every tick and adjusts model routing based on `sustainability_ratio`. Self-funding Golems do not require owner top-ups -- but the owner can always extend life via x402 (see S7.3).

---

## S11 -- The Pay Moat

Every agent framework lets agents move money. The architectural question is what prevents the agent from moving money it should not.

The six-layer financial security stack (TEE key isolation, scoped signing policies, time-delayed execution via Warden, on-chain PolicyCage guards, pre-flight simulation, post-trade verification) is the structural answer. Three of these layers are cryptographic -- they hold even if the LLM is fully compromised by prompt injection. See `prd2/10-safety/02-policy.md` for PolicyCage specification and `prd2-extended/10-safety/02-warden.md` for the Warden time-delay proxy.

The competitive implication: any framework that wants to match this financial security must independently implement all six layers, and the layers interact with each other. The PolicyCage must know about the Warden's delay configuration. The simulation must check against the PolicyCage before the real PolicyCage sees the transaction. The MonitorBot must understand the cancel authority's key management. This integration complexity is the moat -- it cannot be replicated by adding a single feature.

---

## Events Emitted

Funding events track the financial lifecycle from initial provisioning through self-sustaining operation.

| Event | Trigger | Payload |
|---|---|---|
| `funding:initial_provisioned` | First funding completes | `{ amount, source, custodyMode, chainId }` |
| `funding:balance_low` | Balance drops below 15% of initial | `{ balance, burnRate, projectedHoursRemaining }` |
| `funding:topped_up` | x402 top-up received | `{ amount, sender, newBalance }` |
| `funding:self_sustaining` | Sustainability ratio crosses 1.0 upward | `{ ratio, dailyRevenue, dailyCosts }` |
| `funding:metabolic_decline` | Sustainability ratio crosses 1.0 downward | `{ ratio, dailyRevenue, dailyCosts }` |
| `funding:model_downgrade` | Mortality-aware routing selects cheaper tier | `{ previousTier, newTier, vitality, reason }` |

---

## References

- [PERMIT2-2022] Adams, H. et al. "Permit2: Signature-based token transfers." Uniswap Labs, 2022. — *Enables off-chain signature-based token transfers without per-spender approvals; the mechanism for funding Golem wallets in Embedded custody mode.*
- [JONAS-1966] Jonas, H. "The Phenomenon of Life." 1966. — *Argues that metabolism is self-sustaining through the very activity that requires sustenance; the philosophical basis for the Golem's metabolic self-funding loop.*
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." _Journal of Monetary Economics_, 50(3), 2003. — *Formalizes that information processing is costly and agents optimally allocate attention; grounds mortality-aware model routing where dying Golems shift to cheaper inference.*
- MetaMask Delegation Framework: https://github.com/MetaMask/delegation-framework — *ERC-7710/7715 implementation enabling caveat-bounded session key delegations for Golem wallet access.*
- ERC-7710: Smart Contract Delegation Standard — *On-chain delegation protocol that lets the owner's Smart Account grant bounded spending authority to a Golem's session key.*
- ERC-7715: Request Permissions (wallet_grantPermissions) — *The wallet-side counterpart to ERC-7710; the UX layer where the owner signs a single permission grant.*
- Bankr LLM Gateway: https://docs.bankr.bot/llm-gateway/overview — *Fifth inference provider where the wallet IS the API key, enabling a closed metabolic loop between trading revenue and inference spending.*
