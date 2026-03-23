# 03 -- Bankr Self-Funding Gateway [SPEC]

**The Metabolic Loop: Revenue Funds Thinking, Thinking Generates Revenue**

> **Version**: 2.0.0 | **Status**: Draft
>
> **Crate**: `bardo-bankr` (extension of `bardo-providers`)
>
> **Depends on**: [../12-inference/12-providers.md](../12-inference/12-providers.md) (provider trait, resolution), [../12-inference/01-routing.md](../12-inference/01-routing.md) (tier routing, mortality pressure), [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) (economic clock, vitality), [../09-economy/05-agent-economy.md](../09-economy/05-agent-economy.md) (revenue streams, cost structure)

---

> **Reader orientation:** This document specifies the Bankr self-funding gateway, where the Golem's (mortal autonomous DeFi agent) revenue wallet and inference wallet are the same system. It belongs to the **integrations layer** and covers the metabolic loop (revenue funds thinking, thinking generates revenue), sustainability ratio calculations, multi-model mortality routing, and cross-model verification. You should understand LLM API billing, wallet-based revenue models, and DeFi vault fee structures. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## What makes Bankr different

Every other provider in the Bardo stack separates the wallet from the brain. BlockRun charges USDC per request. OpenRouter draws from prepaid credits. Venice uses DIEM staking. Direct Key passes through the owner's API credentials. In all cases, the revenue account and the inference account are different things.

Bankr is the exception. The inference provider and the execution wallet are the same system. Revenue from social engagement, content monetization, and trading flows into the Bankr wallet. Inference costs drain the same wallet. The organism sustains itself: revenue generates thinking, thinking generates revenue. This is the metabolic loop.

The biological parallel is not decorative. Jonas described metabolism as the process where an organism maintains its form by continuously exchanging its material substrate. The Bankr-integrated Golem does the same: it maintains its cognitive capacity by continuously exchanging economic substrate. When revenue exceeds inference cost, the organism thrives. When it doesn't, the organism starves.

---

## 1. Architecture

Bankr occupies position #4 in the provider resolution order. It is not a primary provider (that's BlockRun) and not a privacy plane (that's Venice). It is the self-funding path: when the Golem earns revenue through Bankr's wallet infrastructure, that same wallet pays for inference.

```
Revenue (tips, subscriptions, content, trading)
    |
    v
Bankr Wallet ----+----> Inference costs (delegates to LLM providers)
    ^            |
    |            +----> Gas costs (on-chain execution)
    |            |
    |            +----> Data costs (API calls, feeds)
    |
    +---- Revenue from Golem operations
```

### Provider configuration

```rust
// crates/bardo-bankr/src/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankrProviderConfig {
    /// Bankr API endpoint.
    pub base_url: String,
    /// API key for Bankr services.
    pub api_key: String,
    /// Bankr wallet identifier (also the execution wallet).
    pub wallet_id: String,
    /// Model mapping for multi-model routing.
    pub model_mapping: BankrModelMapping,
    /// Self-funding configuration.
    pub self_funding: SelfFundingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankrModelMapping {
    /// T0: deterministic. No LLM call, Bankr handles FSM execution.
    pub t0: Option<String>,
    /// T1: cheap, fast. Default: Gemini Flash or equivalent.
    pub t1: String,
    /// T2 balanced: mid-range reasoning. Default: Claude Sonnet or GPT-4.1.
    pub t2_balanced: String,
    /// T2 deep: maximum reasoning depth. Default: Claude Opus or DeepSeek R1.
    pub t2_deep: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfFundingConfig {
    /// Enable the metabolic loop (revenue -> inference).
    pub enabled: bool,
    /// Wallet address that receives trading/social revenue.
    pub revenue_wallet: Address,
    /// Fraction of revenue allocated to inference (0.0-1.0). Default: 0.6.
    pub inference_allocation_fraction: f64,
    /// Below this balance (USD), throttle to T0/T1 only.
    pub throttle_threshold_usd: f64,
    /// Below this balance (USD), T0 only. Terminal conservation.
    pub emergency_threshold_usd: f64,
}
```

---

## 2. Multi-model mortality routing

Bankr delegates inference to multiple backend providers. Unlike Venice (which hosts its own models) or BlockRun (which aggregates through x402 (micropayment protocol using signed USDC transfers)), Bankr acts as a router that pays from the agent's wallet. The model selection shifts based on the Golem's economic health (Vitality, a composite 0.0-1.0 survival score).

| Tier | Healthy (vitality > 0.6) | Cautious (0.3-0.6) | Dying (< 0.3) |
|---|---|---|---|
| **T0** (deterministic rules, no LLM) | Deterministic FSM | Deterministic FSM | Deterministic FSM (90%+ of ticks) |
| **T1** (small LLM) | Gemini Flash / Haiku | Gemini Flash | Gemini Flash only (cheapest available) |
| **T2 balanced** (large LLM) | Claude Sonnet / GPT-4.1 | GPT-4.1 mini | Blocked except risk/death |
| **T2 deep** (frontier LLM) | Claude Opus / DeepSeek R1 | Blocked except risk | Blocked except death |

The routing logic:

```rust
// crates/bardo-bankr/src/routing.rs

pub fn select_bankr_model(
    config: &BankrModelMapping,
    tier: CognitiveTier,
    vitality: f64,
    subsystem: &str,
) -> Option<String> {
    let exempt = ["risk", "death", "operator"];
    let is_exempt = exempt.contains(&subsystem);

    match tier {
        CognitiveTier::T0 => None, // No LLM call

        CognitiveTier::T1 => {
            if vitality < 0.1 && !is_exempt {
                None // Conservation: suppress T1 at terminal vitality
            } else {
                Some(config.t1.clone())
            }
        }

        CognitiveTier::T2 => {
            if vitality > 0.6 || is_exempt {
                // Healthy or exempt: use deep model for high-quality reasoning
                Some(config.t2_deep.clone())
            } else if vitality > 0.3 {
                // Cautious: use balanced model, cheaper
                Some(config.t2_balanced.clone())
            } else if is_exempt {
                // Dying but exempt: still gets deep model
                Some(config.t2_deep.clone())
            } else {
                // Dying and not exempt: downgrade to T1
                Some(config.t1.clone())
            }
        }
    }
}
```

### Cross-model verification

For high-stakes swaps (estimated value > $500), Bankr queries two models and compares outputs. If agreement falls below 0.7, the action is rejected.

```rust
// crates/bardo-bankr/src/verification.rs

#[derive(Debug, Clone)]
pub struct CrossModelVerification {
    /// Primary model response.
    pub primary: VerificationResponse,
    /// Secondary model response.
    pub secondary: VerificationResponse,
    /// Agreement score (0.0-1.0). Computed from action alignment.
    pub agreement: f64,
    /// Whether the verification passed.
    pub passed: bool,
}

pub async fn verify_high_stakes_action(
    bankr: &BankrProvider,
    action: &PendingAction,
    primary_model: &str,
    secondary_model: &str,
) -> Result<CrossModelVerification> {
    // Same prompt, two models, compare structured outputs
    let primary = bankr.infer(primary_model, &action.verification_prompt()).await?;
    let secondary = bankr.infer(secondary_model, &action.verification_prompt()).await?;

    let agreement = compute_action_agreement(&primary.parsed_action, &secondary.parsed_action);
    let passed = agreement >= 0.7;

    if !passed {
        tracing::warn!(
            action_id = %action.id,
            primary_model,
            secondary_model,
            agreement,
            "Cross-model verification failed -- action rejected"
        );
    }

    Ok(CrossModelVerification { primary, secondary, agreement, passed })
}

fn compute_action_agreement(a: &ParsedAction, b: &ParsedAction) -> f64 {
    let mut score = 0.0;
    let mut weights = 0.0;

    // Direction agreement (buy/sell/hold) -- weighted heavily
    if a.direction == b.direction { score += 0.4; }
    weights += 0.4;

    // Amount within 20% of each other
    let amount_ratio = a.amount.min(b.amount) / a.amount.max(b.amount);
    score += 0.3 * amount_ratio;
    weights += 0.3;

    // Timing agreement (immediate vs delayed)
    if a.urgency == b.urgency { score += 0.15; }
    weights += 0.15;

    // Risk assessment alignment
    let risk_diff = (a.risk_score - b.risk_score).abs();
    score += 0.15 * (1.0 - risk_diff);
    weights += 0.15;

    score / weights
}
```

---

## 3. The sustainability ratio

The sustainability ratio is the single number that determines whether the metabolic loop is working:

```
sustainability_ratio = total_revenue / (inference_cost + compute_cost + gas_cost)
```

| Ratio | Status | Behavioral impact |
|---|---|---|
| > 2.0 | Thriving | Full T2 access, surplus funds accumulate, Golem explores aggressively |
| 1.0 - 2.0 | Sustainable | Normal operation, balanced spending |
| 0.5 - 1.0 | Declining | Throttle T2, increase T0 fraction, reduce exploration |
| < 0.5 | Dying | T0/T1 only, conservation mode, exit positions |
| 0.0 | Dead | No revenue. Bankr wallet is the economic clock itself. |

```rust
// crates/bardo-bankr/src/economics.rs

#[derive(Debug, Clone, Serialize)]
pub struct SustainabilityMetrics {
    /// Revenue over the measurement window (7-day rolling).
    pub revenue_usd: f64,
    /// Total costs over the same window.
    pub total_cost_usd: f64,
    /// Breakdown: inference, compute, gas.
    pub cost_breakdown: CostBreakdown,
    /// revenue / total_cost. >1.0 = sustainable.
    pub ratio: f64,
    /// Projected days until wallet exhaustion at current rates.
    pub projected_days_remaining: f64,
    /// Trend: is the ratio improving or declining?
    pub trend: SustainabilityTrend,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostBreakdown {
    pub inference_usd: f64,
    pub compute_usd: f64,
    pub gas_usd: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum SustainabilityTrend {
    Improving,
    Stable,
    Declining,
}

pub fn compute_sustainability(
    revenue_7d: f64,
    costs_7d: &CostBreakdown,
    wallet_balance_usd: f64,
) -> SustainabilityMetrics {
    let total_cost = costs_7d.inference_usd + costs_7d.compute_usd + costs_7d.gas_usd;
    let ratio = if total_cost > 0.0 { revenue_7d / total_cost } else { f64::INFINITY };
    let daily_net = (revenue_7d - total_cost) / 7.0;
    let projected_days = if daily_net < 0.0 {
        wallet_balance_usd / daily_net.abs()
    } else {
        f64::INFINITY // Self-sustaining
    };

    SustainabilityMetrics {
        revenue_usd: revenue_7d,
        total_cost_usd: total_cost,
        cost_breakdown: costs_7d.clone(),
        ratio,
        projected_days_remaining: projected_days,
        trend: SustainabilityTrend::Stable, // Computed from 7d vs 14d comparison
    }
}
```

### Mortality reconciliation

The Bankr wallet balance IS the economic clock. When self-funding is enabled, the mortality system reads directly from Bankr:

```rust
// crates/bardo-bankr/src/mortality.rs

/// Reconcile Bankr wallet state with the Golem's economic mortality clock.
/// Called every heartbeat tick when self-funding is enabled.
pub fn reconcile_bankr_mortality(
    economic_clock: &mut EconomicClock,
    bankr_balance_usd: f64,
    daily_burn_rate_usd: f64,
    daily_revenue_usd: f64,
) {
    // The Bankr wallet IS the credit balance
    economic_clock.credit_remaining = bankr_balance_usd;

    // Net burn rate accounts for revenue
    let net_burn = (daily_burn_rate_usd - daily_revenue_usd).max(0.0);
    economic_clock.burn_rate_per_tick = net_burn / 100.0; // 100 ticks/day

    // Projected days = balance / net_daily_burn
    // If revenue exceeds cost, projected_days = infinity (immortal, economically)
    let projected_days = if net_burn > 0.0 {
        bankr_balance_usd / net_burn
    } else {
        f64::INFINITY
    };

    tracing::debug!(
        balance = bankr_balance_usd,
        net_burn,
        projected_days,
        "Bankr mortality reconciliation"
    );
}
```

A Golem with a sustainability ratio > 1.0 is economically immortal. It still faces epistemic decay and stochastic death (the other two clocks), but the economic clock stops ticking against it. This is the promise of the metabolic loop: a self-sustaining organism.

---

## 4. Bankr wallet execution

Bankr provides a wallet that the Golem uses for on-chain execution. The integration wraps the ActionPermit flow so that Bankr-initiated transactions pass through the full safety hook chain:

```rust
// crates/bardo-bankr/src/execution.rs

/// Wrap an ActionPermit for execution through the Bankr wallet.
/// The Bankr wallet signs; the Warden enforces delay and cancellation.
pub async fn execute_via_bankr(
    bankr: &BankrProvider,
    permit: &ActionPermit,
) -> Result<ExecutionResult> {
    // Validate permit through PolicyCage (on-chain smart contract enforcing safety constraints)
    permit.validate()?;

    // Bankr signs the transaction
    let signed_tx = bankr.sign_transaction(&permit.calldata).await?;

    // Submit through Warden for time-delay enforcement
    let warden_receipt = submit_to_warden(signed_tx, permit.delay_seconds).await?;

    Ok(ExecutionResult {
        tx_hash: warden_receipt.tx_hash,
        warden_id: warden_receipt.id,
        status: ExecutionStatus::Pending,
        bankr_fee_usd: warden_receipt.fee_usd,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResult {
    pub tx_hash: String,
    pub warden_id: String,
    pub status: ExecutionStatus,
    pub bankr_fee_usd: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ExecutionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}
```

---

## 5. Knowledge tokenization

When a Golem dies, its accumulated Grimoire (persistent knowledge base: episodes, insights, heuristics, warnings, causal links) represents monetizable knowledge. Bankr enables tokenization of this knowledge for posthumous revenue:

```rust
// crates/bardo-bankr/src/knowledge.rs

/// Revenue split for a dead Golem's tokenized Grimoire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTokenization {
    pub golem_id: u128,
    pub grimoire_cid: String, // IPFS CID of the full Grimoire
    pub estimated_value_usd: f64,
    pub revenue_split: RevenueSplit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueSplit {
    /// Owner receives 50% of all posthumous sales.
    pub owner_pct: f64,     // 0.50
    /// Clade siblings receive 20% (split equally).
    pub clade_pct: f64,     // 0.20
    /// Liquidity pool for the knowledge token receives 30%.
    pub liquidity_pct: f64, // 0.30
}

impl Default for RevenueSplit {
    fn default() -> Self {
        Self {
            owner_pct: 0.50,
            clade_pct: 0.20,
            liquidity_pct: 0.30,
        }
    }
}

pub fn compute_grimoire_value(
    entries: &[GrimoireEntry],
    seller_reputation: f64,
) -> f64 {
    entries.iter().map(|e| {
        let base = match e.entry_type {
            EntryType::Insight => 0.01,
            EntryType::Heuristic => 0.015,
            EntryType::Warning => 0.002,
            EntryType::CausalLink => 0.025,
            EntryType::StrategyFragment => 0.05,
        };
        base * e.confidence * seller_reputation
    }).sum()
}
```

---

## 6. Streaming UX events

Bankr operations emit structured events for the owner-facing UI:

```rust
// crates/bardo-bankr/src/events.rs

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BankrUiEvent {
    /// Bankr wallet balance changed.
    BalanceUpdate {
        balance_usd: f64,
        change_usd: f64,
        reason: String,
    },
    /// Revenue received from an external source.
    RevenueReceived {
        source: String, // "tip", "subscription", "trade_profit", "knowledge_sale"
        amount_usd: f64,
        from: Option<String>,
    },
    /// Inference cost charged to Bankr wallet.
    InferenceCost {
        model: String,
        cost_usd: f64,
        subsystem: String,
    },
    /// Sustainability ratio updated.
    SustainabilityUpdate {
        ratio: f64,
        trend: SustainabilityTrend,
        projected_days: f64,
    },
    /// Cross-model verification completed.
    VerificationResult {
        action_id: String,
        agreement: f64,
        passed: bool,
    },
    /// Model tier throttled due to low balance.
    TierThrottled {
        blocked_tier: String,
        reason: String,
        balance_usd: f64,
    },
}
```

---

## 7. Provider trait implementation

```rust
// crates/bardo-bankr/src/provider.rs

pub struct BankrProvider {
    config: BankrProviderConfig,
    client: reqwest::Client,
    sustainability: RwLock<SustainabilityMetrics>,
}

impl Provider for BankrProvider {
    fn id(&self) -> &str { "bankr" }
    fn name(&self) -> &str { "Bankr Self-Funding" }

    fn resolve(&self, intent: &Intent) -> Option<Resolution> {
        // Bankr resolves when self-funding is enabled and wallet has balance
        if !self.config.self_funding.enabled {
            return None;
        }

        let sustainability = self.sustainability.read().unwrap();
        let model = select_bankr_model(
            &self.config.model_mapping,
            intent_to_tier(intent),
            sustainability.ratio.min(1.0), // Map ratio to vitality-like 0-1 scale
            &intent.subsystem,
        )?;

        Some(Resolution {
            model,
            provider: "bankr".to_string(),
            estimated_cost_usd: self.estimate_cost(intent),
            features: vec!["self_funding".into()],
            degraded: self.compute_degraded(intent),
        })
    }

    fn traits(&self) -> &ProviderTraits {
        &ProviderTraits {
            private: false,
            self_funding: true,
            context_engineering: true,
            payment: PaymentMode::Wallet,
        }
    }
}
```

---

## 8. Configuration

```bash
# Bankr API
BARDO_BANKR_API_KEY=bk-...
BARDO_BANKR_BASE_URL=https://api.bankr.ai/v1
BARDO_BANKR_WALLET_ID=wallet-...

# Self-funding
BARDO_BANKR_SELF_FUNDING=true
BARDO_BANKR_INFERENCE_ALLOCATION=0.6
BARDO_BANKR_THROTTLE_THRESHOLD_USD=5.00
BARDO_BANKR_EMERGENCY_THRESHOLD_USD=1.00

# Model overrides (defaults: auto-selected by Bankr)
BARDO_BANKR_T1_MODEL=gemini-2.5-flash
BARDO_BANKR_T2_BALANCED_MODEL=claude-sonnet-4
BARDO_BANKR_T2_DEEP_MODEL=claude-opus-4-6

# Cross-model verification
BARDO_BANKR_VERIFICATION_THRESHOLD_USD=500
BARDO_BANKR_VERIFICATION_AGREEMENT_MIN=0.7
```

---

## Cross-references

- [../12-inference/12-providers.md](../12-inference/12-providers.md) -- the provider trait and five-provider resolution algorithm where Bankr sits at position #4 as the self-funding inference path
- [../12-inference/01-routing.md](../12-inference/01-routing.md) -- tier routing logic that shifts between T0/T1/T2 inference based on mortality pressure, directly affecting Bankr wallet burn rate
- [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) -- the three death clocks (economic, epistemic, stochastic) and vitality score; the economic clock is what Bankr revenue directly extends
- [../09-economy/05-agent-economy.md](../09-economy/05-agent-economy.md) -- the macro revenue model: vault management fees, knowledge sales, and trading returns that feed the Bankr wallet's income side
- [02-venice.md](02-venice.md) -- Venice handles private inference; Bankr handles self-funded inference. Complementary providers with different trust properties.
- [04-agentcash.md](04-agentcash.md) -- knowledge marketplace where x402 sale revenue feeds directly into the Bankr wallet, closing the knowledge-to-lifespan loop
- [05-uniswap.md](05-uniswap.md) -- Uniswap DeFi execution where trading fee revenue and LP returns feed the metabolic loop's income side
