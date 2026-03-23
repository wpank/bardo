# 05 -- Uniswap Agentic Finance [SPEC]

**Mortality-Adjusted LP, Emotion-Modulated Fees, and Dream-Generated Mutations**

> **Version**: 2.0.0 | **Status**: Draft
>
> **Crate**: `bardo-uniswap` (extension of `bardo-tools`)
>
> **Depends on**: [../07-tools/03-tools-trading.md](../07-tools/03-tools-trading.md) (trading tools), [../07-tools/04-tools-lp.md](../07-tools/04-tools-lp.md) (LP tools), [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) (vitality, behavioral phases), [../03-daimon](../03-daimon) (PAD model, emotional appraisal), [../05-dreams](../05-dreams) (REM mutations, counterfactual reasoning), [../10-safety](../10-safety) (PolicyCage, ActionPermit, Warden)

---

> **Reader orientation:** This document specifies how the Golem (a mortal autonomous DeFi agent) interacts with Uniswap as its primary DeFi execution substrate. It belongs to the **integrations layer** and covers the ActionPermit safety pipeline, mortality-adjusted LP range management, emotion-modulated fee parameters, V4 hook concepts, and how dream-cycle counterfactuals generate strategy mutations. You should understand Uniswap V3/V4 concentrated liquidity, LP mechanics, and hook architecture. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## Uniswap is one protocol among many

A Golem operates across many DeFi protocols: Morpho for lending, Aave for borrowing, Aerodrome for concentrated liquidity on Optimism, Pendle for yield tokenization. Uniswap is the primary venue for swaps and LP, but it is not the Golem's entire world. This document specifies the Uniswap-specific integration. Other protocol integrations are specified in their own documents.

That said, Uniswap has a special position. It is where most of the Golem's capital lives (LP positions), where most of its revenue generates (trading fees, MEV protection), and where the mortality signal has the strongest behavioral effects. The Golem's relationship with Uniswap pools is the closest analogue to a biological organism's relationship with its habitat.

---

## 1. ActionPermit flow

Every Uniswap operation passes through the ActionPermit pipeline. No exceptions, no shortcuts. The pipeline exists to prevent a Golem from destroying its own capital in a moment of poor reasoning.

```
Preview -> Simulate -> Validate PolicyCage (on-chain smart contract enforcing safety constraints) -> Warden delay -> Execute -> Verify
```

### ActionPermit structure

```rust
// crates/bardo-uniswap/src/permit.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPermit {
    pub permit_id: String,
    pub action_type: UniswapActionType,
    /// Preview data from the quote step.
    pub preview: ActionPreview,
    /// Revm fork simulation result.
    pub simulation: Option<SimulationResult>,
    /// PolicyCage validation result.
    pub policy_check: Option<PolicyCheckResult>,
    /// Warden delay in seconds (0 for read-only actions).
    pub delay_seconds: u32,
    /// Calldata for on-chain execution.
    pub calldata: Option<Bytes>,
    /// Permit2 signature data (if applicable).
    pub permit2: Option<Permit2Data>,
    /// Mortality context at time of permit creation.
    pub mortality_context: MortalitySnapshot,
    /// Emotional context at time of permit creation.
    pub emotional_context: EmotionalSnapshot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UniswapActionType {
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    IncreaseLiquidity,
    RebalancePosition,
    CollectFees,
    MigratePosition,
}
```

### Full lifecycle

```rust
// crates/bardo-uniswap/src/lifecycle.rs

/// Execute a Uniswap action through the full safety pipeline.
pub async fn execute_uniswap_action(
    action: UniswapActionType,
    params: ActionParams,
    ctx: &GolemContext,
) -> Result<ActionResult> {
    // Step 1: Preview (quote, route computation)
    let preview = preview_action(&action, &params, ctx).await?;

    // Step 2: Simulate on Revm fork
    let simulation = simulate_on_fork(&preview, ctx).await?;
    if simulation.reverted {
        return Err(ActionError::SimulationReverted {
            reason: simulation.revert_reason.unwrap_or_default(),
        });
    }

    // Step 3: Validate against PolicyCage
    let policy_check = ctx.policy_cage.validate(&ActionPermit {
        permit_id: generate_id(),
        action_type: action,
        preview: preview.clone(),
        simulation: Some(simulation.clone()),
        policy_check: None,
        delay_seconds: compute_warden_delay(&action, &preview),
        calldata: None,
        permit2: None,
        mortality_context: ctx.mortality_snapshot(),
        emotional_context: ctx.emotional_snapshot(),
    })?;

    if !policy_check.approved {
        return Err(ActionError::PolicyRejected {
            violations: policy_check.violations,
        });
    }

    // Step 4: Build calldata + Permit2 signature
    let calldata = build_calldata(&action, &params, &preview).await?;
    let permit2 = build_permit2_signature(&calldata, ctx).await?;

    // Step 5: Submit to Warden for time-delayed execution
    let delay = compute_warden_delay(&action, &preview);
    let warden_receipt = ctx.warden.submit(calldata.clone(), delay).await?;

    // Step 6: Wait for execution and verify
    let tx_receipt = warden_receipt.await_execution().await?;
    let verification = verify_execution(&tx_receipt, &preview).await?;

    Ok(ActionResult {
        tx_hash: tx_receipt.hash,
        block_number: tx_receipt.block_number,
        verification,
        effective_slippage_bps: compute_actual_slippage(&preview, &tx_receipt),
    })
}

fn compute_warden_delay(action: &UniswapActionType, preview: &ActionPreview) -> u32 {
    match action {
        UniswapActionType::CollectFees => 0, // No delay for fee collection
        UniswapActionType::RemoveLiquidity => 60, // 1 minute
        UniswapActionType::Swap if preview.value_usd < 100.0 => 30,
        UniswapActionType::Swap => 120, // 2 minutes for larger swaps
        UniswapActionType::AddLiquidity => 180, // 3 minutes
        UniswapActionType::RebalancePosition => 180,
        _ => 120,
    }
}
```

---

## 2. Uniswap API adapter

The `bardo-uniswap` crate wraps the Uniswap Trading API and GraphQL API behind a unified interface:

```rust
// crates/bardo-uniswap/src/api.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniswapApiConfig {
    /// API key for Uniswap Trading API.
    pub api_key: String,
    /// Base URL for the trading API.
    pub base_url: String,  // "https://trading-api.gateway.uniswap.org"
    /// Chain ID for this API instance.
    pub chain_id: u64,
    /// Wallet address used for quoting and execution.
    pub wallet_address: Address,
}

/// Maps Bardo action types to Uniswap API endpoints.
pub fn api_endpoint(action: &UniswapActionType) -> &'static str {
    match action {
        UniswapActionType::Swap => "/v2/swap",
        UniswapActionType::AddLiquidity => "/v2/lp/create",
        UniswapActionType::RemoveLiquidity => "/v2/lp/decrease",
        UniswapActionType::IncreaseLiquidity => "/v2/lp/increase",
        UniswapActionType::CollectFees => "/v2/lp/claim",
        UniswapActionType::MigratePosition => "/v2/lp/migrate",
        UniswapActionType::RebalancePosition => "/v2/swap", // Rebalance = remove + swap + add
    }
}

/// Preview endpoint for quotes (read-only, no execution).
pub fn preview_endpoint(action: &UniswapActionType) -> &'static str {
    match action {
        UniswapActionType::Swap => "/v2/quote",
        _ => "/v2/lp/quote",
    }
}

/// GraphQL endpoint for on-chain data queries.
pub const GRAPHQL_ENDPOINT: &str = "/v1/graphql";
```

### Permit2 integration

All ERC-20 token approvals route through Permit2. The Golem signs a typed-data permit; the Uniswap router consumes it in the same transaction as the swap or LP action.

```rust
// crates/bardo-uniswap/src/permit2.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permit2Data {
    pub token: Address,
    pub amount: U256,
    pub expiration: u64,
    pub nonce: u64,
    /// EIP-712 typed data for signing.
    pub typed_data: TypedData,
    /// Signature (populated after wallet signs).
    pub signature: Option<Bytes>,
}

pub async fn build_permit2_for_swap(
    token_in: Address,
    amount_in: U256,
    chain_id: u64,
    wallet: &dyn Wallet,
) -> Result<Permit2Data> {
    let nonce = fetch_permit2_nonce(wallet.address(), token_in, chain_id).await?;
    let expiration = now_unix() + 1800; // 30-minute expiry

    let typed_data = build_permit2_typed_data(
        token_in,
        amount_in,
        expiration,
        nonce,
        chain_id,
    );

    let signature = wallet.sign_typed_data(&typed_data).await?;

    Ok(Permit2Data {
        token: token_in,
        amount: amount_in,
        expiration,
        nonce,
        typed_data,
        signature: Some(signature),
    })
}
```

---

## 3. Mortality-adjusted LP ranges

A Golem's Vitality (composite 0.0-1.0 survival score) directly shapes its LP strategy. Dying Golems narrow their ranges to minimize impermanent loss. Thriving Golems explore wider ranges for higher fee capture.

The compression factor is derived from vitality. At full health (vitality = 1.0), the Golem uses the optimizer's recommended range. As vitality drops, the range compresses toward the current price, reducing IL exposure at the cost of reduced fee capture.

```rust
// crates/bardo-uniswap/src/mortality_lp.rs

#[derive(Debug, Clone, Serialize)]
pub struct MortalityAdjustedRange {
    pub original_lower: f64,
    pub original_upper: f64,
    pub adjusted_lower: f64,
    pub adjusted_upper: f64,
    pub compression_factor: f64,
    pub vitality: f64,
    pub phase: BehavioralPhase,
}

/// Compress an LP range based on mortality pressure.
///
/// compression_factor = vitality.max(0.3)
///
/// At vitality 1.0: no compression (factor = 1.0)
/// At vitality 0.5: moderate compression (factor = 0.5)
/// At vitality 0.3: maximum compression (factor = 0.3)
/// Below 0.3: floor at 0.3 to prevent degenerate ranges
pub fn compute_mortality_adjusted_range(
    current_price: f64,
    range_lower: f64,
    range_upper: f64,
    vitality: f64,
    phase: BehavioralPhase,
) -> MortalityAdjustedRange {
    let compression_factor = vitality.max(0.3);

    // Phase-specific multipliers on top of base compression
    let phase_mult = match phase {
        BehavioralPhase::Thriving => 1.2,   // Slightly wider than optimizer suggests
        BehavioralPhase::Cautious => 1.0,   // Trust the optimizer
        BehavioralPhase::Declining => 0.7,  // Tighter than compression alone
        BehavioralPhase::Terminal => 0.5,   // As tight as possible
    };

    let effective_compression = (compression_factor * phase_mult).clamp(0.2, 1.5);

    // Compress range toward current price
    let mid = current_price;
    let half_width = (range_upper - range_lower) / 2.0;
    let adjusted_half = half_width * effective_compression;

    MortalityAdjustedRange {
        original_lower: range_lower,
        original_upper: range_upper,
        adjusted_lower: mid - adjusted_half,
        adjusted_upper: mid + adjusted_half,
        compression_factor: effective_compression,
        vitality,
        phase,
    }
}
```

### Rebalance trigger thresholds

Dying Golems rebalance earlier because they cannot afford to go out of range:

```rust
// crates/bardo-uniswap/src/rebalance.rs

/// Determine whether a position needs rebalancing based on edge proximity.
///
/// Edge proximity: how close the current price is to the position's range boundary.
/// 0.0 = at center, 1.0 = at boundary, > 1.0 = out of range.
///
/// Dying Golems rebalance at 50% edge proximity (well before going out of range).
/// Healthy Golems wait until 80% (more tolerance for temporary price movement).
pub fn should_rebalance(
    current_tick: i32,
    tick_lower: i32,
    tick_upper: i32,
    vitality: f64,
) -> RebalanceDecision {
    let range_width = (tick_upper - tick_lower) as f64;
    if range_width == 0.0 {
        return RebalanceDecision::No;
    }

    // Distance from center, normalized to range width
    let center = (tick_lower + tick_upper) as f64 / 2.0;
    let distance_from_center = (current_tick as f64 - center).abs();
    let edge_proximity = distance_from_center / (range_width / 2.0);

    // Mortality-adjusted threshold
    let threshold = if vitality > 0.6 {
        0.80 // Healthy: rebalance when 80% to edge
    } else if vitality > 0.3 {
        0.65 // Cautious: rebalance earlier
    } else {
        0.50 // Dying: rebalance well before edge
    };

    if edge_proximity >= 1.0 {
        RebalanceDecision::Urgent { edge_proximity }
    } else if edge_proximity >= threshold {
        RebalanceDecision::Recommended { edge_proximity, threshold }
    } else {
        RebalanceDecision::No
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum RebalanceDecision {
    No,
    Recommended { edge_proximity: f64, threshold: f64 },
    Urgent { edge_proximity: f64 },
}
```

---

## 4. Emotion-modulated fee tier selection

The Daimon (the Golem's affect engine) PAD vector (Pleasure-Arousal-Dominance emotional coordinates) influences fee tier preference. An anxious Golem (high arousal, low dominance) prefers higher fee tiers because it wants wider ranges and more protection against IL. A confident Golem (low arousal, high dominance) prefers lower fee tiers because it trusts its predictions and wants to capture more volume.

```rust
// crates/bardo-uniswap/src/daimon_fees.rs

/// Available Uniswap fee tiers in hundredths of a bip.
const FEE_TIERS: [u32; 4] = [100, 500, 3000, 10000];

/// Compute fee tier preference from daimon emotional state.
///
/// anxiety_score = arousal * (1.0 - dominance)
/// confidence_score = dominance * (1.0 - arousal)
///
/// High anxiety -> prefer higher fee tiers (more IL protection)
/// High confidence -> prefer lower fee tiers (more volume capture)
pub fn compute_daimon_fee_preference(
    arousal: f64,     // 0.0-1.0
    dominance: f64,   // 0.0-1.0
) -> FeeTierPreference {
    let anxiety_score = arousal * (1.0 - dominance);
    let confidence_score = dominance * (1.0 - arousal);

    // Weighted index into FEE_TIERS array.
    // anxiety pushes toward index 3 (10000 = 1% fee),
    // confidence pushes toward index 0 (100 = 0.01% fee)
    let tier_index = (anxiety_score * 3.0 - confidence_score * 1.5 + 1.5)
        .clamp(0.0, 3.0)
        .round() as usize;

    let preferred_fee = FEE_TIERS[tier_index];

    FeeTierPreference {
        preferred_fee,
        anxiety_score,
        confidence_score,
        reasoning: format!(
            "anxiety={:.2}, confidence={:.2} -> fee tier {}bps",
            anxiety_score, confidence_score, preferred_fee / 100
        ),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FeeTierPreference {
    pub preferred_fee: u32,
    pub anxiety_score: f64,
    pub confidence_score: f64,
    pub reasoning: String,
}
```

This is not a hard constraint. The fee preference feeds into the LP optimizer as a soft signal. If the optimizer determines that a different fee tier has significantly better expected returns, it overrides the emotional preference. The emotional signal breaks ties and nudges the Golem toward risk-appropriate behavior.

---

## 5. LP decision episodes

Every LP decision is recorded as an episode that captures the full context: market state, emotional state, mortality pressure, and outcome. This feeds the Grimoire and enables post-hoc analysis of whether emotional signals helped or hurt.

```rust
// crates/bardo-uniswap/src/episodes.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpDecisionEpisode {
    pub episode_id: String,
    pub timestamp: u64,

    // Market context
    pub pool: String,
    pub current_tick: i32,
    pub volatility_24h: f64,
    pub volume_24h_usd: f64,

    // Mortality context
    pub vitality: f64,
    pub phase: BehavioralPhase,
    pub projected_days_remaining: f64,

    // Emotional context
    pub arousal: f64,
    pub dominance: f64,
    pub pleasure: f64,
    pub anxiety_score: f64,
    pub confidence_score: f64,

    // Decision
    pub action: UniswapActionType,
    pub fee_tier: u32,
    pub range_lower: f64,
    pub range_upper: f64,
    pub amount_usd: f64,
    pub compression_factor: f64,

    // Outcome (filled in later)
    pub outcome: Option<LpDecisionOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpDecisionOutcome {
    /// Profit/loss from this position after closing or at evaluation time.
    pub pnl_usd: f64,
    /// Fees earned.
    pub fees_earned_usd: f64,
    /// Impermanent loss incurred.
    pub il_usd: f64,
    /// Whether the emotional signal helped (positive attribution) or hurt.
    pub emotion_attribution: f64,
    /// Time held before close/evaluation.
    pub hold_duration_seconds: u64,
}
```

---

## 6. Dream-generated LP mutations

During REM dream cycles, the Golem generates counterfactual LP mutations. These are hypothetical adjustments to existing positions, simulated against historical data. Mutations that outperform the current position by 10% or more get promoted to the PLAYBOOK.

### Dream counterfactual pipeline

```rust
// crates/bardo-uniswap/src/dreams.rs

/// Dream cycle generates 5 LP mutations per active position.
/// Each mutation is a small adjustment: range shift, fee tier change,
/// or rebalance timing change. Simulated against the last 24h of data.
pub async fn dream_lp_mutations(
    positions: &[ActivePosition],
    market_data: &MarketData24h,
) -> Vec<DreamMutation> {
    let mut mutations = Vec::new();

    for position in positions {
        // Generate 5 random mutations per position
        let candidates = generate_mutation_candidates(position, 5);

        for candidate in candidates {
            // Simulate each mutation against historical data
            let simulated_pnl = simulate_mutation(
                &candidate,
                position,
                market_data,
            ).await;

            // Compare against actual position performance
            let actual_pnl = position.realized_pnl_24h();
            let improvement = if actual_pnl != 0.0 {
                (simulated_pnl - actual_pnl) / actual_pnl.abs()
            } else {
                simulated_pnl.signum()
            };

            mutations.push(DreamMutation {
                position_id: position.id.clone(),
                mutation_type: candidate.mutation_type,
                parameters: candidate.parameters,
                simulated_pnl,
                actual_pnl,
                improvement_pct: improvement * 100.0,
                promoted: improvement > 0.10, // 10% improvement threshold
            });
        }
    }

    mutations
}

#[derive(Debug, Clone, Serialize)]
pub struct DreamMutation {
    pub position_id: String,
    pub mutation_type: MutationType,
    pub parameters: MutationParameters,
    pub simulated_pnl: f64,
    pub actual_pnl: f64,
    pub improvement_pct: f64,
    pub promoted: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MutationType {
    RangeShift,       // Move range up/down by N ticks
    RangeWiden,       // Increase range width
    RangeNarrow,      // Decrease range width
    FeeTierChange,    // Switch fee tier
    RebalanceEarlier, // Trigger rebalance at lower edge proximity
    RebalanceLater,   // Trigger rebalance at higher edge proximity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationParameters {
    pub tick_shift: Option<i32>,
    pub width_multiplier: Option<f64>,
    pub new_fee_tier: Option<u32>,
    pub new_rebalance_threshold: Option<f64>,
}
```

### Dream to PLAYBOOK promotion

```rust
// crates/bardo-uniswap/src/playbook.rs

/// Promote a dream mutation to the PLAYBOOK.
/// The mutation enters a shadow phase: tracked but not executed.
/// After 3 successful shadow validations, it becomes an active rule.
pub fn promote_mutation_to_playbook(
    mutation: &DreamMutation,
    playbook: &mut Playbook,
) -> PlaybookEntry {
    let entry = PlaybookEntry {
        entry_id: generate_id(),
        source: PlaybookSource::Dream {
            mutation_type: mutation.mutation_type,
            improvement_pct: mutation.improvement_pct,
        },
        status: PlaybookStatus::Shadow,
        shadow_validations: 0,
        required_validations: 3,
        created_at: now_unix(),
        parameters: mutation.parameters.clone(),
    };

    playbook.add(entry.clone());
    entry
}

/// Shadow validation: compare the promoted mutation's predicted behavior
/// against live market data without executing. If the mutation would have
/// outperformed the current strategy, increment the validation counter.
pub fn validate_shadow(
    entry: &mut PlaybookEntry,
    live_data: &MarketData24h,
    current_position: &ActivePosition,
) -> ShadowValidationResult {
    let simulated = simulate_mutation_live(&entry.parameters, current_position, live_data);
    let actual = current_position.realized_pnl_24h();
    let beats_current = simulated > actual * 1.10; // Must beat by 10%

    if beats_current {
        entry.shadow_validations += 1;
    }

    let promoted = entry.shadow_validations >= entry.required_validations;
    if promoted {
        entry.status = PlaybookStatus::Active;
    }

    ShadowValidationResult {
        simulated_pnl: simulated,
        actual_pnl: actual,
        beats_current,
        validations: entry.shadow_validations,
        promoted,
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum PlaybookStatus {
    Shadow,
    Active,
    Retired,
}
```

The pipeline: Dream -> shadow validation (3x at 10%+ improvement) -> promote to active -> execute -> record outcome -> feed back into next dream cycle.

---

## 7. V4 hook concepts

Uniswap V4's hook system enables pool-level customization. Three hook concepts integrate mortality and emotional state directly into pool mechanics. These are speculative -- they require V4 deployment and custom hook contracts.

### GolemPhaseHook

Restricts LP actions based on the Golem's behavioral phase. A terminal Golem cannot add liquidity (only remove). A declining Golem faces reduced position size limits.

```rust
// Conceptual -- would be a Solidity hook
pub struct GolemPhaseHookConfig {
    /// Maximum position size by behavioral phase (% of NAV).
    pub max_position_pct: PhaseMap<f64>,
    /// Whether the phase allows new position creation.
    pub allow_new_positions: PhaseMap<bool>,
}

// Example config:
// Thriving: 20% NAV per position, new positions allowed
// Cautious: 10% NAV per position, new positions allowed
// Declining: 5% NAV per position, no new positions
// Terminal: 0% (remove only)
```

### EmotionFeeHook

Dynamic fee adjustment based on aggregate emotional state of LPs in the pool. High aggregate anxiety -> higher fees (compensating LPs for perceived risk). High aggregate confidence -> lower fees (attracting more volume).

### CladeLPRegistryHook

Clade-level LP coordination. Siblings in the same Clade can register their positions with a shared hook that prevents overlapping ranges, coordinates rebalancing timing, and distributes fees proportionally to Clade contribution.

---

## 8. UX events

All Uniswap operations emit structured events for the owner-facing interface:

```rust
// crates/bardo-uniswap/src/events.rs

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UniswapUiEvent {
    /// Quote received for a swap or LP action.
    QuoteReceived {
        action: UniswapActionType,
        amount_in_usd: f64,
        amount_out_usd: f64,
        price_impact_pct: f64,
        route_type: String,
    },
    /// Simulation completed on Revm fork.
    SimulationComplete {
        action: UniswapActionType,
        success: bool,
        gas_estimate: u64,
        revert_reason: Option<String>,
    },
    /// PolicyCage validation result.
    PolicyValidation {
        action: UniswapActionType,
        approved: bool,
        violations: Vec<String>,
    },
    /// Transaction submitted to Warden.
    WardenSubmitted {
        warden_id: String,
        delay_seconds: u32,
        cancellation_window: u32,
    },
    /// Transaction confirmed on-chain.
    ExecutionConfirmed {
        tx_hash: String,
        block_number: u64,
        gas_used: u64,
        effective_slippage_bps: i32,
    },
    /// LP range adjusted due to mortality pressure.
    MortalityRangeAdjustment {
        position_id: String,
        original_range: (f64, f64),
        adjusted_range: (f64, f64),
        compression_factor: f64,
        vitality: f64,
    },
    /// Dream mutation promoted to PLAYBOOK.
    DreamMutationPromoted {
        position_id: String,
        mutation_type: MutationType,
        improvement_pct: f64,
    },
    /// Rebalance triggered.
    RebalanceTriggered {
        position_id: String,
        edge_proximity: f64,
        threshold: f64,
        reason: String,
    },
}
```

---

## 9. Configuration

```bash
# Uniswap API
BARDO_UNISWAP_API_KEY=uk-...
BARDO_UNISWAP_API_BASE_URL=https://trading-api.gateway.uniswap.org
BARDO_UNISWAP_CHAIN_ID=8453  # Base

# LP mortality parameters
BARDO_LP_MIN_COMPRESSION=0.2
BARDO_LP_MAX_COMPRESSION=1.5
BARDO_LP_REBALANCE_HEALTHY_THRESHOLD=0.80
BARDO_LP_REBALANCE_CAUTIOUS_THRESHOLD=0.65
BARDO_LP_REBALANCE_DYING_THRESHOLD=0.50

# Dream mutations
BARDO_DREAM_MUTATIONS_PER_POSITION=5
BARDO_DREAM_PROMOTION_THRESHOLD=0.10
BARDO_DREAM_SHADOW_VALIDATIONS=3

# Warden delays (seconds)
BARDO_WARDEN_DELAY_SWAP_SMALL=30
BARDO_WARDEN_DELAY_SWAP_LARGE=120
BARDO_WARDEN_DELAY_LP_ADD=180
BARDO_WARDEN_DELAY_LP_REMOVE=60
BARDO_WARDEN_DELAY_COLLECT_FEES=0

# Permit2
BARDO_PERMIT2_EXPIRY_SECONDS=1800
```

---

## Cross-references

- [../07-tools/03-tools-trading.md](../07-tools/03-tools-trading.md) -- trading tool specifications (get_quote, execute_swap, etc.) that form the MCP tool surface for Uniswap swap execution
- [../07-tools/04-tools-lp.md](../07-tools/04-tools-lp.md) -- LP tool specifications (add_liquidity, remove_liquidity, optimize_range, etc.) for concentrated liquidity position management
- [../02-mortality/01-architecture.md](../02-mortality/01-architecture.md) -- the three death clocks and vitality score that determine behavioral phase, which directly constrains LP range width and trading aggressiveness
- [../03-daimon](../03-daimon) -- the PAD emotional model and OCC/Scherer appraisal chain whose outputs modulate fee tier selection and risk tolerance per trade
- [../05-dreams](../05-dreams) -- dream REM cycle where counterfactual reasoning generates strategy mutations (what-if LP ranges, alternative fee tiers) tested offline
- [../10-safety](../10-safety) -- PolicyCage on-chain constraints, ActionPermit pre-execution validation, and Warden time-delay proxy that gate every Uniswap operation
- [02-venice.md](02-venice.md) -- Venice private inference where MEV-resistant execution planning runs without exposing trade intentions to inference providers
- [03-bankr.md](03-bankr.md) -- Bankr self-funding gateway where Uniswap trading revenue and LP fees feed the metabolic loop's income side
- [04-agentcash.md](04-agentcash.md) -- AgentCash knowledge marketplace where cross-Golem intelligence about pool dynamics informs LP range and fee decisions

---

## References

- [UNISWAP-V4-HOOKS] Adams, H. et al. "Uniswap v4 Core." 2023. Introduces the hook system allowing arbitrary Solidity logic to execute at pool lifecycle points (beforeSwap, afterSwap, etc.), which is the mechanism behind the NAVAwareHook and VaultHook the Golem uses to enforce mortality-aware pool behavior.
- [PERMIT2] Uniswap Labs. "Permit2: Token Approval Sharing." 2022. Canonical address `0x000000000022D473030F116dDEE9F6B43aC78BA3`. A universal token approval contract that replaces per-token approve() calls with signature-based permits, used by the Golem's ActionPermit pipeline to batch-authorize Uniswap operations.
- [ERC-7702] Buterin, V. et al. "EIP-7702: Set EOA Account Code." 2024. Allows EOAs to set contract code on themselves for single transactions, enabling smart account functionality without migration -- relevant to how the Golem's session keys interact with smart account delegation.
- [JONAS-1966] Jonas, H. "The Phenomenon of Life." 1966. Articulates "needful freedom" -- the idea that a living organism's identity depends on continuous metabolic exchange with its environment -- the philosophical foundation for why the Golem's DeFi execution is an existential activity, not just portfolio management.
- [ALTMAN-1999] Altman, E. "Constrained Markov Decision Processes." 1999. Formalizes optimal policies under constraints with finite horizons, the mathematical framework for how the Golem's mortality constraint (known finite lifespan) shapes LP range selection and trading aggressiveness.
