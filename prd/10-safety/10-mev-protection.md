# MEV Detection and Protection [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-18
>
> **Crates**: `bardo-mev`, `bardo-chain`, `bardo-safety`
>
> **Depends on**: [00-defense.md](00-defense.md) (defense layers), [05-threat-model.md](05-threat-model.md) (threat taxonomy), [06-adaptive-risk.md](06-adaptive-risk.md) (Layer 5 DeFi threats), `../14-chain/02-triage.md` (triage pipeline)

> **Reader orientation:** This document specifies how a Golem (mortal autonomous DeFi agent) detects and protects itself from MEV (Maximal Extractable Value) extraction. It belongs to the **Safety** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: MEV is both a threat (your transactions get sandwiched, front-run, or censored) and a signal (MEV patterns reveal market microstructure that informs trading decisions). The Golem uses detection algorithms, private mempool routing, and slippage bounds to minimize extraction. Terms like PolicyCage, Heartbeat, and CorticalState are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

Maximal Extractable Value (MEV) is profit that block proposers and searchers extract by reordering, inserting, or censoring transactions within a block. For an autonomous trading agent, MEV is both a threat (your transactions get sandwiched) and a signal (MEV patterns reveal market microstructure). This document covers the foundational research, detection algorithms, protection strategies, and Rust implementations for integrating MEV awareness into a chain intelligence pipeline.

The numbers are large. Daian et al. ("Flash Boys 2.0," IEEE S&P 2020) documented Priority Gas Auctions consuming significant network resources. Zust et al. (ETH Zurich, 2021) identified 525,004 sandwich attacks over 12 months extracting 57,493 ETH (~$189M). An agent that ignores MEV is donating money to searchers.

---

## The foundational model: Flash Boys 2.0

[SPEC] Daian et al. ("Flash Boys 2.0: Frontrunning in Decentralized Exchanges, Miner Extractable Value, and Consensus Instability," IEEE S&P 2020, arXiv:1904.05234) established the theoretical framework. The paper documents three key phenomena:

**Priority Gas Auctions (PGAs).** When a profitable opportunity appears in the mempool (e.g., a large swap that creates an arbitrage), searchers compete by bidding up gas prices. The paper observed auctions lasting hundreds of milliseconds with bots adjusting gas prices in real-time, consuming block space with failed replacement transactions.

**Front-running.** A searcher sees a pending transaction that will move a price, and submits their own transaction ahead of it (with higher gas) to profit from the price movement. The victim's transaction executes at a worse price.

**Miner/proposer extractable value.** Block producers can reorder transactions within a block to extract value directly. Post-merge, this manifests through the proposer-builder separation (PBS) architecture where specialized builders construct blocks that maximize MEV.

The paper's lasting contribution is framing MEV as a systemic property of transparent mempools, not an anomaly. Any system that broadcasts pending transactions to a public network creates an information asymmetry that sophisticated actors will exploit.

---

## MEV attack taxonomy

### Sandwich attacks

[SPEC] The most common MEV attack against DEX traders. The attacker observes a pending swap, places a buy order before it (front-run) and a sell order after it (back-run), profiting from the price impact the victim's trade creates.

**Structure:**
1. Victim submits: swap X tokens for Y on Uniswap
2. Attacker front-runs: buy Y tokens, pushing the price up
3. Victim's swap executes at a worse price (higher cost for Y)
4. Attacker back-runs: sell Y tokens at the inflated price

The attacker's profit equals the price impact of the victim's trade minus gas costs and the attacker's own price impact. Qin et al. ("Quantifying Blockchain Extractable Value: How dark is the forest?" IEEE S&P 2022) formalized the profit calculation and showed that sandwich attacks account for the majority of DEX-related MEV.

### Front-running

Pure front-running without a corresponding back-run. The attacker copies a profitable transaction (e.g., a liquidation or arbitrage) and submits it with higher gas priority. The original transaction either fails or executes at a worse price.

### Back-running

The attacker places a transaction immediately after a large trade to capture the arbitrage opportunity it creates. Less adversarial than sandwiching -- the victim's transaction executes unmodified, and the attacker only captures the resulting price discrepancy across venues.

### JIT (Just-In-Time) liquidity

A more subtle form. The attacker observes a pending large swap on a concentrated liquidity AMM (Uniswap V3/V4), provides concentrated liquidity in the exact tick range the swap will traverse, earns fees from the swap, and removes liquidity in the same block. The existing liquidity providers earn less in fees, and the JIT provider captures a disproportionate share.

---

## Detecting MEV patterns in Rust

[SPEC] The following implementation provides pattern detectors for each MEV type. These detectors operate on block-level transaction data -- either from mempool monitoring or post-execution block analysis.

```rust
use alloy::primitives::{Address, U256, TxHash, B256};
use std::collections::HashMap;

/// A simplified transaction representation for MEV analysis.
/// In production, this would include full trace data.
#[derive(Debug, Clone)]
pub struct MevTransaction {
    pub hash: TxHash,
    pub from: Address,
    pub to: Address,
    pub block_number: u64,
    pub tx_index: u32,
    pub gas_price: U256,
    pub value: U256,
    /// Decoded swap parameters, if this is a DEX trade.
    pub swap: Option<SwapData>,
    /// Decoded liquidity parameters, if this is an LP action.
    pub liquidity: Option<LiquidityData>,
    /// The pool address involved, if any.
    pub pool: Option<Address>,
}

#[derive(Debug, Clone)]
pub struct SwapData {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out: U256,
    pub pool: Address,
    /// Direction: true if token0->token1, false if token1->token0.
    pub zero_for_one: bool,
}

#[derive(Debug, Clone)]
pub struct LiquidityData {
    pub pool: Address,
    pub action: LiquidityAction,
    pub amount0: U256,
    pub amount1: U256,
    pub tick_lower: Option<i32>,
    pub tick_upper: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiquidityAction {
    Add,
    Remove,
}

/// The type of MEV pattern detected.
#[derive(Debug, Clone)]
pub enum MevPattern {
    Sandwich(SandwichBundle),
    Frontrun(FrontrunBundle),
    Backrun(BackrunBundle),
    JitLiquidity(JitBundle),
    Arbitrage(ArbitrageBundle),
}

#[derive(Debug, Clone)]
pub struct SandwichBundle {
    pub attacker: Address,
    pub frontrun_tx: TxHash,
    pub victim_tx: TxHash,
    pub backrun_tx: TxHash,
    pub pool: Address,
    pub estimated_profit: U256,
    /// Price impact imposed on the victim, in basis points.
    pub victim_impact_bps: u32,
}

#[derive(Debug, Clone)]
pub struct FrontrunBundle {
    pub attacker: Address,
    pub attacker_tx: TxHash,
    pub victim_tx: TxHash,
    pub pool: Address,
}

#[derive(Debug, Clone)]
pub struct BackrunBundle {
    pub attacker: Address,
    pub attacker_tx: TxHash,
    pub target_tx: TxHash,
    pub pool: Address,
    pub estimated_profit: U256,
}

#[derive(Debug, Clone)]
pub struct JitBundle {
    pub provider: Address,
    pub add_liquidity_tx: TxHash,
    pub target_swap_tx: TxHash,
    pub remove_liquidity_tx: TxHash,
    pub pool: Address,
    pub fees_earned: U256,
}

#[derive(Debug, Clone)]
pub struct ArbitrageBundle {
    pub arbitrageur: Address,
    pub tx: TxHash,
    pub path: Vec<Address>,  // pools traversed
    pub profit: U256,
}

/// Core MEV detection engine.
/// Operates on a single block's worth of transactions.
pub struct MevDetector {
    /// Minimum profit threshold (in wei) to report a pattern.
    min_profit_threshold: U256,
    /// Known MEV bot addresses for attribution.
    known_bots: HashMap<Address, String>,
}

impl MevDetector {
    pub fn new(min_profit_threshold: U256) -> Self {
        Self {
            min_profit_threshold,
            known_bots: HashMap::new(),
        }
    }

    pub fn register_known_bot(&mut self, address: Address, name: String) {
        self.known_bots.insert(address, name);
    }

    /// Scan a block's transactions for all MEV patterns.
    /// Transactions must be ordered by their index within the block.
    pub fn detect_all(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        let mut patterns = Vec::new();

        patterns.extend(self.detect_sandwiches(txs));
        patterns.extend(self.detect_jit_liquidity(txs));
        patterns.extend(self.detect_backruns(txs));
        // Arbitrage detection requires trace-level data;
        // included as a structural placeholder.
        patterns.extend(self.detect_arbitrage(txs));

        patterns
    }

    /// Detect sandwich attacks.
    ///
    /// Pattern: three consecutive-ish transactions on the same pool where:
    ///   1. tx_a is a swap by address X in direction D
    ///   2. tx_b is a swap by address Y (victim) in direction D
    ///   3. tx_c is a swap by address X in direction !D
    /// and tx_a.index < tx_b.index < tx_c.index.
    ///
    /// The attacker (X) buys before the victim and sells after.
    pub fn detect_sandwiches(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        let mut patterns = Vec::new();

        // Group swaps by pool address.
        let mut swaps_by_pool: HashMap<Address, Vec<&MevTransaction>> = HashMap::new();
        for tx in txs {
            if let Some(ref swap) = tx.swap {
                swaps_by_pool
                    .entry(swap.pool)
                    .or_default()
                    .push(tx);
            }
        }

        for (_pool, pool_swaps) in &swaps_by_pool {
            if pool_swaps.len() < 3 {
                continue;
            }

            // Check all triples where the first and third share a sender
            // and have opposite directions, with a different sender in between.
            for i in 0..pool_swaps.len() {
                let front = pool_swaps[i];
                let front_swap = front.swap.as_ref().unwrap();

                for j in (i + 1)..pool_swaps.len() {
                    let victim = pool_swaps[j];
                    let victim_swap = victim.swap.as_ref().unwrap();

                    // Victim must be a different address, same direction.
                    if victim.from == front.from {
                        continue;
                    }
                    if victim_swap.zero_for_one != front_swap.zero_for_one {
                        continue;
                    }

                    for k in (j + 1)..pool_swaps.len() {
                        let back = pool_swaps[k];
                        let back_swap = back.swap.as_ref().unwrap();

                        // Back-run must be same attacker, opposite direction.
                        if back.from != front.from {
                            continue;
                        }
                        if back_swap.zero_for_one == front_swap.zero_for_one {
                            continue;
                        }

                        // Confirm ordering within the block.
                        if front.tx_index >= victim.tx_index
                            || victim.tx_index >= back.tx_index
                        {
                            continue;
                        }

                        // Estimate profit: attacker's output on back-run
                        // minus attacker's input on front-run.
                        let profit = if back_swap.amount_out > front_swap.amount_in {
                            back_swap.amount_out - front_swap.amount_in
                        } else {
                            U256::ZERO
                        };

                        if profit >= self.min_profit_threshold {
                            // Estimate victim impact in basis points.
                            let impact_bps = if !victim_swap.amount_out.is_zero() {
                                // Rough estimate: front-run amount / pool reserves
                                // This is simplified; real calculation needs
                                // pre-trade and post-trade prices.
                                10u32 // placeholder
                            } else {
                                0
                            };

                            patterns.push(MevPattern::Sandwich(SandwichBundle {
                                attacker: front.from,
                                frontrun_tx: front.hash,
                                victim_tx: victim.hash,
                                backrun_tx: back.hash,
                                pool: front_swap.pool,
                                estimated_profit: profit,
                                victim_impact_bps: impact_bps,
                            }));
                        }
                    }
                }
            }
        }

        patterns
    }

    /// Detect JIT liquidity provision.
    ///
    /// Pattern: within the same block on the same pool:
    ///   1. Address X adds concentrated liquidity
    ///   2. A large swap executes through that tick range
    ///   3. Address X removes liquidity
    /// The JIT provider earns swap fees without bearing directional risk.
    pub fn detect_jit_liquidity(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        let mut patterns = Vec::new();

        // Index liquidity events by pool.
        let mut liq_by_pool: HashMap<Address, Vec<&MevTransaction>> = HashMap::new();
        let mut swaps_by_pool: HashMap<Address, Vec<&MevTransaction>> = HashMap::new();

        for tx in txs {
            if let Some(ref liq) = tx.liquidity {
                liq_by_pool.entry(liq.pool).or_default().push(tx);
            }
            if let Some(ref swap) = tx.swap {
                swaps_by_pool.entry(swap.pool).or_default().push(tx);
            }
        }

        for (pool, liq_txs) in &liq_by_pool {
            let Some(pool_swaps) = swaps_by_pool.get(pool) else {
                continue;
            };

            // Find add/remove pairs from the same address.
            let adds: Vec<&&MevTransaction> = liq_txs
                .iter()
                .filter(|tx| {
                    tx.liquidity
                        .as_ref()
                        .map_or(false, |l| l.action == LiquidityAction::Add)
                })
                .collect();

            let removes: Vec<&&MevTransaction> = liq_txs
                .iter()
                .filter(|tx| {
                    tx.liquidity
                        .as_ref()
                        .map_or(false, |l| l.action == LiquidityAction::Remove)
                })
                .collect();

            for add_tx in &adds {
                for remove_tx in &removes {
                    // Same provider.
                    if add_tx.from != remove_tx.from {
                        continue;
                    }
                    // Add must come before remove.
                    if add_tx.tx_index >= remove_tx.tx_index {
                        continue;
                    }

                    // Check for a swap between them on this pool.
                    let sandwiched_swap = pool_swaps.iter().find(|swap_tx| {
                        swap_tx.tx_index > add_tx.tx_index
                            && swap_tx.tx_index < remove_tx.tx_index
                    });

                    if let Some(swap_tx) = sandwiched_swap {
                        // Estimate fees earned from the removal amounts
                        // exceeding the addition amounts.
                        let add_liq = add_tx.liquidity.as_ref().unwrap();
                        let remove_liq = remove_tx.liquidity.as_ref().unwrap();

                        let fees_0 = remove_liq.amount0.saturating_sub(add_liq.amount0);
                        let fees_1 = remove_liq.amount1.saturating_sub(add_liq.amount1);
                        let fees = fees_0 + fees_1; // simplified

                        patterns.push(MevPattern::JitLiquidity(JitBundle {
                            provider: add_tx.from,
                            add_liquidity_tx: add_tx.hash,
                            target_swap_tx: swap_tx.hash,
                            remove_liquidity_tx: remove_tx.hash,
                            pool: *pool,
                            fees_earned: fees,
                        }));
                    }
                }
            }
        }

        patterns
    }

    /// Detect back-running patterns.
    ///
    /// A back-run follows a large trade and captures the arbitrage
    /// opportunity it creates. Look for: large swap on pool P, then
    /// a swap on the same pool (or a related pair) by a known bot
    /// or high-gas-priority sender in the opposite direction.
    pub fn detect_backruns(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        let mut patterns = Vec::new();

        let swaps: Vec<&MevTransaction> = txs
            .iter()
            .filter(|tx| tx.swap.is_some())
            .collect();

        for i in 0..swaps.len() {
            let target = swaps[i];
            let target_swap = target.swap.as_ref().unwrap();

            // Look at the next few swaps on the same pool.
            for j in (i + 1)..swaps.len().min(i + 4) {
                let candidate = swaps[j];
                let candidate_swap = candidate.swap.as_ref().unwrap();

                if candidate_swap.pool != target_swap.pool {
                    continue;
                }
                if candidate.from == target.from {
                    continue;
                }
                // Back-run trades in the opposite direction.
                if candidate_swap.zero_for_one == target_swap.zero_for_one {
                    continue;
                }
                // Must be immediately after (within 2 tx index positions).
                if candidate.tx_index > target.tx_index + 2 {
                    continue;
                }

                let is_known_bot = self.known_bots.contains_key(&candidate.from);
                let high_gas = candidate.gas_price > target.gas_price;

                if is_known_bot || high_gas {
                    patterns.push(MevPattern::Backrun(BackrunBundle {
                        attacker: candidate.from,
                        attacker_tx: candidate.hash,
                        target_tx: target.hash,
                        pool: candidate_swap.pool,
                        estimated_profit: candidate_swap.amount_out
                            .saturating_sub(candidate_swap.amount_in),
                    }));
                }
            }
        }

        patterns
    }

    /// Detect cyclic arbitrage.
    ///
    /// An arbitrage transaction touches multiple pools in sequence,
    /// ending with more of the starting token than it began with.
    /// Detection requires trace-level data to reconstruct the token flow.
    pub fn detect_arbitrage(&self, txs: &[MevTransaction]) -> Vec<MevPattern> {
        let mut patterns = Vec::new();

        // Group transactions by sender.
        let mut by_sender: HashMap<Address, Vec<&MevTransaction>> = HashMap::new();
        for tx in txs {
            by_sender.entry(tx.from).or_default().push(tx);
        }

        for (sender, sender_txs) in &by_sender {
            for tx in sender_txs {
                let Some(ref swap) = tx.swap else {
                    continue;
                };

                // A single transaction that is part of a multi-hop path
                // looks like: multiple internal swap events in one tx.
                // With only top-level data, we check if the sender
                // has a net positive token balance change.
                //
                // Full detection requires transaction traces, which would
                // parse internal calls and log events. The struct below
                // is a placeholder for trace-based detection.

                // Check for known arbitrage bot patterns:
                // - transaction touches 2+ pools
                // - starts and ends with the same token
                // - profit = final_balance - initial_balance > 0
                //
                // This is left as a structural marker. Real implementation
                // needs the trace decoder from ../14-chain/02-triage.md
                let _ = (sender, swap);
            }
        }

        patterns
    }
}
```

---

## Integration with the triage pipeline

[SPEC] The MEV detector plugs into the chain intelligence triage pipeline (see [../14-chain/02-triage.md](../14-chain/02-triage.md)). Triage already processes every transaction in a block, scoring by curiosity and relevance. MEV detection adds a classification layer on top.

```rust
use std::time::Instant;

/// MEV classification result attached to a triage event.
#[derive(Debug, Clone)]
pub struct MevClassification {
    pub patterns: Vec<MevPattern>,
    pub is_mev_target: bool,
    pub risk_score: f64,
    pub recommended_action: MevAction,
    pub detection_latency_us: u64,
}

/// What the agent should do in response to MEV detection.
#[derive(Debug, Clone)]
pub enum MevAction {
    /// No MEV risk detected. Proceed normally.
    Proceed,
    /// MEV risk detected. Route through private submission.
    UsePrivateSubmission,
    /// High MEV risk. Delay or restructure the transaction.
    DelayAndRestructure {
        suggested_delay_blocks: u32,
        split_into: u32,
    },
    /// The transaction itself is MEV (the agent is the searcher).
    ExecuteAsSearcher,
    /// Abort. The MEV cost exceeds the expected trade profit.
    Abort { reason: String },
}

/// Wraps the MEV detector for use within the triage pipeline.
/// Called once per block after basic triage scoring completes.
pub struct MevTriageStage {
    detector: MevDetector,
    /// Threshold above which MEV risk triggers private submission.
    private_submission_threshold: f64,
    /// Threshold above which MEV risk triggers abort.
    abort_threshold: f64,
}

impl MevTriageStage {
    pub fn new(
        min_profit_threshold: U256,
        private_submission_threshold: f64,
        abort_threshold: f64,
    ) -> Self {
        Self {
            detector: MevDetector::new(min_profit_threshold),
            private_submission_threshold,
            abort_threshold,
        }
    }

    /// Classify MEV risk for a set of block transactions.
    /// Returns per-transaction classifications.
    pub fn classify_block(
        &self,
        txs: &[MevTransaction],
    ) -> HashMap<TxHash, MevClassification> {
        let start = Instant::now();
        let patterns = self.detector.detect_all(txs);
        let detection_time = start.elapsed().as_micros() as u64;

        let mut classifications: HashMap<TxHash, MevClassification> = HashMap::new();

        // Tag victim transactions.
        for pattern in &patterns {
            match pattern {
                MevPattern::Sandwich(bundle) => {
                    classifications.insert(
                        bundle.victim_tx,
                        MevClassification {
                            patterns: vec![pattern.clone()],
                            is_mev_target: true,
                            risk_score: 0.9,
                            recommended_action: MevAction::UsePrivateSubmission,
                            detection_latency_us: detection_time,
                        },
                    );
                }
                MevPattern::Frontrun(bundle) => {
                    classifications.insert(
                        bundle.victim_tx,
                        MevClassification {
                            patterns: vec![pattern.clone()],
                            is_mev_target: true,
                            risk_score: 0.8,
                            recommended_action: MevAction::UsePrivateSubmission,
                            detection_latency_us: detection_time,
                        },
                    );
                }
                MevPattern::JitLiquidity(bundle) => {
                    classifications.insert(
                        bundle.target_swap_tx,
                        MevClassification {
                            patterns: vec![pattern.clone()],
                            is_mev_target: true,
                            risk_score: 0.4, // JIT is less harmful to the swapper
                            recommended_action: MevAction::Proceed,
                            detection_latency_us: detection_time,
                        },
                    );
                }
                _ => {}
            }
        }

        classifications
    }

    /// Assess MEV risk for the agent's own pending transaction.
    /// Call this before submitting to decide on routing strategy.
    pub fn assess_own_transaction(
        &self,
        pending_tx: &MevTransaction,
        recent_block_txs: &[MevTransaction],
    ) -> MevAction {
        let swap = match &pending_tx.swap {
            Some(s) => s,
            None => return MevAction::Proceed,
        };

        // Heuristic: estimate sandwichability based on trade size
        // relative to pool liquidity. This is a simplified model.
        // A production implementation would simulate the sandwich
        // using revm to compute exact profit.
        let trade_size_eth = swap.amount_in;

        // Check if recent blocks show active sandwiching on this pool.
        let recent_sandwiches = recent_block_txs.iter().filter(|tx| {
            tx.swap
                .as_ref()
                .map_or(false, |s| s.pool == swap.pool)
        }).count();

        let risk_score = if recent_sandwiches > 3 {
            0.8 // pool is actively targeted
        } else if trade_size_eth > U256::from(10u64).pow(U256::from(18u64)) {
            0.6 // large trade on any pool
        } else {
            0.2 // small trade, low risk
        };

        if risk_score >= self.abort_threshold {
            MevAction::Abort {
                reason: format!(
                    "MEV risk score {:.2} exceeds abort threshold {:.2}",
                    risk_score, self.abort_threshold
                ),
            }
        } else if risk_score >= self.private_submission_threshold {
            MevAction::UsePrivateSubmission
        } else {
            MevAction::Proceed
        }
    }
}
```

---

## Triage rules for MEV patterns

[SPEC] MEV patterns generate triage alerts at different severity levels, depending on whether the agent is the target or just observing.

```rust
/// Severity levels for MEV triage alerts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MevAlertSeverity {
    /// Informational: observed MEV activity on monitored pools.
    Info,
    /// Warning: MEV activity detected on pools the agent trades on.
    Warning,
    /// High: the agent's pending transaction is at risk of MEV extraction.
    High,
    /// Critical: confirmed MEV extraction against the agent's transaction.
    Critical,
}

#[derive(Debug, Clone)]
pub struct MevAlert {
    pub severity: MevAlertSeverity,
    pub pattern: MevPattern,
    pub block_number: u64,
    pub message: String,
    pub recommended_action: MevAction,
}

/// Rule engine that converts raw MEV detections into prioritized alerts.
pub struct MevAlertEngine {
    /// Pools the agent actively trades on.
    watched_pools: Vec<Address>,
    /// The agent's own addresses.
    agent_addresses: Vec<Address>,
}

impl MevAlertEngine {
    pub fn new(watched_pools: Vec<Address>, agent_addresses: Vec<Address>) -> Self {
        Self {
            watched_pools,
            agent_addresses,
        }
    }

    /// Process detected MEV patterns into severity-ranked alerts.
    pub fn generate_alerts(
        &self,
        patterns: &[MevPattern],
        block_number: u64,
    ) -> Vec<MevAlert> {
        let mut alerts = Vec::new();

        for pattern in patterns {
            let alert = match pattern {
                MevPattern::Sandwich(bundle) => {
                    let severity = if self.is_agent_address(&bundle.attacker) {
                        // Should not happen, but flag it.
                        MevAlertSeverity::Critical
                    } else if self.is_agent_victim(&bundle.victim_tx, &bundle.pool) {
                        MevAlertSeverity::Critical
                    } else if self.is_watched_pool(&bundle.pool) {
                        MevAlertSeverity::Warning
                    } else {
                        MevAlertSeverity::Info
                    };

                    MevAlert {
                        severity,
                        pattern: pattern.clone(),
                        block_number,
                        message: format!(
                            "Sandwich on pool {:?}: attacker {:?}, ~{} wei profit",
                            bundle.pool, bundle.attacker, bundle.estimated_profit
                        ),
                        recommended_action: MevAction::UsePrivateSubmission,
                    }
                }

                MevPattern::JitLiquidity(bundle) => {
                    let severity = if self.is_watched_pool(&bundle.pool) {
                        MevAlertSeverity::Warning
                    } else {
                        MevAlertSeverity::Info
                    };

                    MevAlert {
                        severity,
                        pattern: pattern.clone(),
                        block_number,
                        message: format!(
                            "JIT liquidity on pool {:?}: provider {:?}, {} wei fees",
                            bundle.pool, bundle.provider, bundle.fees_earned
                        ),
                        recommended_action: MevAction::Proceed,
                    }
                }

                MevPattern::Backrun(bundle) => {
                    let severity = if self.is_watched_pool(&bundle.pool) {
                        MevAlertSeverity::Info
                    } else {
                        MevAlertSeverity::Info
                    };

                    MevAlert {
                        severity,
                        pattern: pattern.clone(),
                        block_number,
                        message: format!(
                            "Backrun on pool {:?}: searcher {:?}",
                            bundle.pool, bundle.attacker
                        ),
                        recommended_action: MevAction::Proceed,
                    }
                }

                MevPattern::Frontrun(bundle) => {
                    let severity = if self.is_watched_pool(&bundle.pool) {
                        MevAlertSeverity::Warning
                    } else {
                        MevAlertSeverity::Info
                    };

                    MevAlert {
                        severity,
                        pattern: pattern.clone(),
                        block_number,
                        message: format!(
                            "Frontrun on pool {:?}: attacker {:?}",
                            bundle.pool, bundle.attacker
                        ),
                        recommended_action: MevAction::UsePrivateSubmission,
                    }
                }

                MevPattern::Arbitrage(bundle) => {
                    MevAlert {
                        severity: MevAlertSeverity::Info,
                        pattern: pattern.clone(),
                        block_number,
                        message: format!(
                            "Arbitrage by {:?} across {} pools, {} wei profit",
                            bundle.arbitrageur,
                            bundle.path.len(),
                            bundle.profit
                        ),
                        recommended_action: MevAction::Proceed,
                    }
                }
            };

            alerts.push(alert);
        }

        // Sort by severity, highest first.
        alerts.sort_by(|a, b| b.severity.cmp(&a.severity));
        alerts
    }

    fn is_watched_pool(&self, pool: &Address) -> bool {
        self.watched_pools.contains(pool)
    }

    fn is_agent_address(&self, addr: &Address) -> bool {
        self.agent_addresses.contains(addr)
    }

    /// Check whether a victim transaction belongs to the agent.
    /// In practice, this would look up the tx sender from a cache.
    fn is_agent_victim(&self, _victim_tx: &TxHash, pool: &Address) -> bool {
        // Simplified: flag if it's a watched pool.
        // Full implementation checks tx sender against agent addresses.
        self.is_watched_pool(pool)
    }
}
```

---

## Protection strategies

Detection without protection is just watching yourself get robbed. Four protection mechanisms exist, each with different tradeoffs.

### Private transaction submission

[SPEC] **Flashbots Protect** (rpc.flashbots.net) routes transactions through a private mempool. Transactions are never broadcast publicly; they go directly to registered block builders who include them only if they don't revert. The transaction is invisible to searchers scanning the public mempool.

**MEV Blocker** (CoW Protocol) takes a different approach: it mixes real transactions with AI-generated decoy transactions, making it harder for searchers to identify targets. It refunds 90% of back-run bid value to users.

**MEV-Share** (Flashbots) implements an order flow auction where users selectively share transaction "hints" (partial information) and receive MEV rebates. Users control what information they expose. Paradigm maintains a Rust client (mev-share-rs) for programmatic access.

```rust
/// Transaction submission routing based on MEV risk assessment.
pub struct SubmissionRouter {
    /// Standard public RPC endpoint.
    public_rpc: String,
    /// Flashbots Protect RPC endpoint.
    flashbots_rpc: String,
    /// MEV-Share endpoint for order flow auctions.
    mev_share_rpc: String,
    /// MEV Blocker endpoint.
    mev_blocker_rpc: String,
}

#[derive(Debug, Clone)]
pub enum SubmissionRoute {
    /// Standard public mempool. No protection.
    Public,
    /// Flashbots Protect. Private submission, no rebate.
    FlashbotsProtect,
    /// MEV-Share. Private submission with partial hint sharing and rebates.
    MevShare { hints: MevShareHints },
    /// MEV Blocker. Private submission with decoy mixing and rebates.
    MevBlocker,
}

#[derive(Debug, Clone)]
pub struct MevShareHints {
    /// Share the target contract address.
    pub share_contract: bool,
    /// Share the function selector.
    pub share_selector: bool,
    /// Share log topics.
    pub share_logs: bool,
    /// Share the calldata.
    pub share_calldata: bool,
}

impl SubmissionRouter {
    pub fn new(
        public_rpc: &str,
        flashbots_rpc: &str,
        mev_share_rpc: &str,
        mev_blocker_rpc: &str,
    ) -> Self {
        Self {
            public_rpc: public_rpc.to_string(),
            flashbots_rpc: flashbots_rpc.to_string(),
            mev_share_rpc: mev_share_rpc.to_string(),
            mev_blocker_rpc: mev_blocker_rpc.to_string(),
        }
    }

    /// Select a submission route based on MEV risk assessment.
    pub fn select_route(&self, action: &MevAction) -> SubmissionRoute {
        match action {
            MevAction::Proceed => SubmissionRoute::Public,

            MevAction::UsePrivateSubmission => {
                // Default to MEV-Share for rebate potential.
                // Share minimal hints to maximize rebate while
                // limiting searcher information.
                SubmissionRoute::MevShare {
                    hints: MevShareHints {
                        share_contract: true,
                        share_selector: false,
                        share_logs: false,
                        share_calldata: false,
                    },
                }
            }

            MevAction::DelayAndRestructure { .. } => {
                // For restructured transactions, use Flashbots Protect
                // since the split trades are smaller and less likely
                // to generate meaningful MEV-Share rebates.
                SubmissionRoute::FlashbotsProtect
            }

            MevAction::ExecuteAsSearcher => {
                // The agent is the searcher. Submit as a Flashbots bundle
                // directly to builders. This is separate from Protect.
                SubmissionRoute::FlashbotsProtect
            }

            MevAction::Abort { .. } => {
                // Don't submit at all. Caller handles this.
                SubmissionRoute::Public // unreachable in practice
            }
        }
    }

    /// Get the RPC endpoint for the selected route.
    pub fn endpoint(&self, route: &SubmissionRoute) -> &str {
        match route {
            SubmissionRoute::Public => &self.public_rpc,
            SubmissionRoute::FlashbotsProtect => &self.flashbots_rpc,
            SubmissionRoute::MevShare { .. } => &self.mev_share_rpc,
            SubmissionRoute::MevBlocker => &self.mev_blocker_rpc,
        }
    }
}
```

### Time-weighted average pricing

For large trades that cannot avoid price impact, splitting the trade across multiple blocks with time-weighted average pricing (TWAP) reduces MEV exposure. Each individual trade is smaller and less profitable to sandwich.

Uniswap V3's built-in TWAP oracle provides on-chain time-weighted price feeds. The agent can compare execution price against TWAP to detect whether it's getting a fair price. A significant deviation between spot price and TWAP signals either high volatility or active MEV manipulation.

### Commit-reveal schemes

The agent can protect specific high-value transactions by committing a hash of the transaction parameters on-chain, waiting for the commitment to be included in a block, then revealing the actual parameters in a subsequent transaction. Searchers cannot front-run what they cannot decode.

The cost is latency -- at least two blocks (24+ seconds) instead of one. This is acceptable for position management but too slow for time-sensitive arbitrage.

### Pre-trade simulation for MEV vulnerability

[SPEC] Before submitting any transaction, the agent can simulate whether a profitable sandwich exists. Using revm (see mirage-rs), simulate:

1. The agent's pending transaction against current state
2. An optimal front-run transaction by a hypothetical attacker
3. The agent's transaction after the front-run
4. An optimal back-run transaction

If step 4 is profitable for the attacker, the transaction is sandwichable. The agent can then either reduce trade size, use private submission, or abort.

```rust
/// Pre-trade MEV vulnerability assessment using revm simulation.
/// This integrates with the mirage-rs fork simulation layer.
pub struct MevSimulator {
    /// Minimum attacker profit (in wei) to consider a trade vulnerable.
    sandwich_profit_threshold: U256,
}

/// Result of pre-trade MEV simulation.
#[derive(Debug)]
pub struct MevSimulationResult {
    /// Is this trade vulnerable to a sandwich attack?
    pub is_sandwichable: bool,
    /// Estimated maximum attacker profit from sandwiching.
    pub max_sandwich_profit: U256,
    /// Price impact of the trade without sandwich (basis points).
    pub clean_impact_bps: u32,
    /// Price impact of the trade with sandwich (basis points).
    pub sandwiched_impact_bps: u32,
    /// The additional cost to the trader from the sandwich.
    pub additional_cost: U256,
}

impl MevSimulator {
    pub fn new(sandwich_profit_threshold: U256) -> Self {
        Self {
            sandwich_profit_threshold,
        }
    }

    /// Simulate whether a pending swap is vulnerable to sandwich MEV.
    ///
    /// Uses the revm fork simulation to:
    /// 1. Execute the swap cleanly and record the output
    /// 2. Find the optimal front-run amount
    /// 3. Execute front-run + swap + back-run and compute attacker profit
    /// 4. Compare victim's output in both scenarios
    ///
    /// The actual revm invocation depends on the mirage-rs fork layer.
    /// This function shows the algorithmic structure.
    pub fn assess_sandwich_risk(
        &self,
        swap: &SwapData,
        _block_number: u64,
    ) -> MevSimulationResult {
        // Step 1: Simulate clean execution.
        // let clean_result = mirage.simulate_swap(swap, block_number);
        // let clean_output = clean_result.amount_out;
        // let clean_impact = compute_impact(swap.amount_in, clean_output, ...);

        // Step 2: Binary search for optimal front-run amount.
        // The attacker maximizes: backrun_output - frontrun_input - gas_cost.
        // Search between 0 and some maximum (e.g., 10x the victim's trade).
        //
        // for each candidate front-run amount:
        //   a) simulate front-run swap
        //   b) simulate victim's swap (now at worse price)
        //   c) simulate back-run swap (attacker sells)
        //   d) compute attacker profit = backrun_out - frontrun_in - gas
        //   e) track maximum

        // Step 3: Compare victim outputs.
        // sandwiched_impact = victim's output with sandwich / without sandwich

        // Placeholder return -- real implementation calls revm.
        MevSimulationResult {
            is_sandwichable: false,
            max_sandwich_profit: U256::ZERO,
            clean_impact_bps: 0,
            sandwiched_impact_bps: 0,
            additional_cost: U256::ZERO,
        }
    }
}
```

---

## ArbiNet: arbitrage network analysis

Arbitrage is the most benign form of MEV -- it corrects price discrepancies across venues without directly harming other traders. But the network of arbitrage activity reveals market structure.

Research on arbitrage networks (see Daian et al. and subsequent work) shows that arbitrage paths form a directed graph where nodes are liquidity pools and edges represent profitable trade routes. The topology of this graph changes with market conditions: during calm markets, arbitrage paths are short (2-3 hops) and well-known. During volatility events, longer paths open up temporarily as price dislocations cascade across protocols.

For chain intelligence, tracking arbitrage network topology provides:

- **Market health signals**: drying up of arbitrage opportunities signals price convergence (stable markets) or liquidity withdrawal (dangerous markets)
- **New opportunity discovery**: novel arbitrage paths through newly deployed pools indicate emerging DeFi activity worth monitoring
- **Bot behavior mapping**: tracking which addresses execute which paths over time builds a behavioral fingerprint of active MEV searchers

The Artemis framework (github.com/paradigmxyz/artemis) from Paradigm provides a Rust MEV bot architecture with Collector, Strategy, and Executor stages. Its architecture maps onto the triage pipeline: Collectors ingest mempool and block data, Strategies identify opportunities, and Executors route transactions. The built-in MEV-Share arbitrage example demonstrates the pattern end-to-end.

---

## Real-time MEV monitoring with the witness system

[SPEC] The witness/triage system described in [../14-chain/](../14-chain/) processes every block in real-time. MEV detection adds a classification layer that runs after basic transaction decoding:

1. **Block arrives** via Reth ExEx or WebSocket subscription
2. **Triage stage 1**: Decode transactions, identify swaps and liquidity events
3. **Triage stage 2**: Run `MevDetector::detect_all()` on the decoded block
4. **Triage stage 3**: Run `MevAlertEngine::generate_alerts()` to prioritize findings
5. **Triage stage 4**: For the agent's own pending transactions, run `MevSimulator::assess_sandwich_risk()` and route through `SubmissionRouter`

This adds microseconds to per-block processing. The sandwich detection is O(n^2) in the number of swaps per pool per block, which in practice rarely exceeds a few dozen. The JIT detection is similarly bounded.

For historical analysis, the same detectors can run over archived block data to build MEV activity heatmaps -- which pools get sandwiched most, which time windows see peak MEV activity, and how the MEV landscape shifts around major protocol events.

---

## References

- Daian, P., Goldfeder, S., Kell, T., Li, Y., Zhao, X., Bentov, I., Breidenbach, L., & Juels, A. (2020). "Flash Boys 2.0: Frontrunning in Decentralized Exchanges, Miner Extractable Value, and Consensus Instability." IEEE Symposium on Security and Privacy (S&P) 2020. arXiv:1904.05234. The foundational MEV paper: frames MEV as a systemic property of transparent mempools, documents Priority Gas Auctions, and introduces miner extractable value as a concept. The theoretical framework this entire document builds on.
- Qin, K., Zhou, L., & Gervais, A. (2022). "Quantifying Blockchain Extractable Value: How dark is the forest?" IEEE Symposium on Security and Privacy (S&P) 2022. Quantifies total extractable value across Ethereum DeFi protocols. Provides the economic context for why MEV protection is a first-order concern for autonomous agents.
- Zust, P., Nadler, M., & Livshits, B. (2021). "Analyzing and Preventing Sandwich Attacks in Ethereum." ETH Zurich Technical Report. Identified 525,004 sandwich attacks over 12 months extracting 57,493 ETH (~$189M). The empirical basis for the sandwich detection algorithms implemented here.
- Qin, K., Zhou, L., Gamito, B., Jovanovic, P., & Gervais, A. (2021). "An Empirical Study of DeFi Liquidations: Incentives, Risks, and Instabilities." ACM IMC 2021. Analyzes DeFi liquidation mechanics and their exploitation by MEV searchers. Relevant because liquidation-triggered MEV is a distinct risk for leveraged strategies.
- Babel, K., Daian, P., Kelkar, M., & Juels, A. (2023). "Clockwork Finance: Automated Analysis of Economic Security in Smart Contracts." IEEE S&P 2023. Automates economic security analysis by modeling DeFi as composable financial primitives. Provides the formal framework for reasoning about MEV as an economic property.
- Flashbots. "MEV-Share: Programmable Privacy." github.com/flashbots/mev-share. The specification for Flashbots' MEV-Share protocol, allowing users to share MEV with searchers in exchange for execution guarantees. One of the private mempool strategies evaluated for Golem transaction protection.
- Paradigm. "Artemis: A Framework for Writing MEV Bots in Rust." github.com/paradigmxyz/artemis. Rust-based MEV bot framework. Relevant as a reference implementation for understanding searcher behavior and as potential infrastructure for defensive MEV strategies.
- Weintraub, B., Torres, C. F., Roos, S., & State, R. (2022). "A Flash(bot) in the Pan: Measuring Maximal Extractable Value in Private Transaction Ordering." ACM IMC 2022. Measures MEV extraction in private ordering systems (Flashbots). Shows that private mempools reduce but do not eliminate MEV extraction.
- Park, S., Figueiredo, D. R., & Bahrak, B. (2024). "Just-In-Time Liquidity on the Uniswap Protocol." DeFi Workshop 2024. Analyzes JIT liquidity provision as a form of MEV on Uniswap. Relevant because JIT attacks target LP positions that Golems may manage.
