# Simulation Validation: Mirage vs On-Chain [SPEC]

**Version:** 1.0.0
**Last Updated:** 2026-03-18
**Companion to:** `16-testing/11-mirage-v2-testing.md` (engine correctness)

> Validates that mirage-rs simulation outputs match actual on-chain results. Takes N historical blocks where the golem acted, replays them through mirage, and compares simulated vs actual outcomes. This is the sim-to-real accuracy test.

> **Reader orientation:** This document validates that mirage-rs simulation outputs match actual on-chain results for DeFi operations (swaps, LP positions, lending, vaults). It belongs to Section 16 (Testing) and defines the sim-to-real accuracy test: take N historical blocks where a Golem (mortal autonomous agent) acted, replay them through Mirage (live fork infrastructure), and compare simulated vs actual outcomes within defined tolerances. Engine correctness is a prerequisite, covered in `./11-mirage-v2-testing.md` (structural tests for HybridDB, DirtyStore, CoW branching). See `prd2/shared/glossary.md` for full term definitions.

---

## Document Map

| Section | Topic |
|---------|-------|
| Overview | Why simulation accuracy matters, what "match" means |
| Methodology | Block selection, replay procedure, comparison metrics |
| Regression Thresholds | Tolerances for swaps, multi-hop, lending, LP operations |
| Swap Accuracy | Single-hop swap output validation |
| Multi-Hop Accuracy | Complex routes through aggregators |
| LP Position Accuracy | Fee accrual, tick crossing, liquidity changes |
| Lending Protocol Accuracy | Borrow rates, health factors, liquidation thresholds |
| Shadow Execution Validation | Modified bytecode divergence tracking |
| Continuous Regression Suite | Automated nightly validation against recent blocks |

---

## Overview

mirage-rs v2 makes a core promise: the simulation is accurate enough for a golem to make real capital allocation decisions based on it. "Accurate enough" means the simulated outcome of a DeFi operation matches the actual on-chain outcome within defined tolerances.

The tolerances are tight for simple operations (single swaps) and looser for complex ones (multi-hop routes, lending with accruing interest). The gap comes from two sources: (1) state divergence between mirage's view and actual chain state at execution time, and (2) gas and ordering differences between simulation and actual block inclusion.

This document specifies the validation methodology, tolerance thresholds, and the continuous regression suite that runs against recent historical blocks.

### What this does NOT test

- Engine correctness (read priority, branching isolation, etc.) -- covered in `16-testing/11-mirage-v2-testing.md`
- Triage pipeline accuracy -- covered in `16-testing/13-triage-evaluation.md`
- Golem decision quality -- a correct simulation can still inform a bad decision

### Dependencies

- `16-testing/11-mirage-v2-testing.md` -- engine correctness (prerequisite)
- Research: `02-mirage-rs/00-architecture.md` -- HybridDB, targeted replay
- Research: `02-mirage-rs/03-historical-mode.md` -- historical mode, PnL attribution

---

## 1. Methodology [SPEC]

### 1.1 Block selection criteria [SPEC]

**Source:** Archived golem transaction history. Each entry contains:
- Block number where the golem submitted a transaction
- Transaction hash
- Expected output (token amounts, position changes, events emitted)
- On-chain receipt (actual gas used, actual events, actual state changes)

**Selection:** Stratified sample across:
- Operation type: swap, multi-hop swap, LP mint/burn, borrow/repay, vault deposit/withdraw
- Market condition: low volatility, high volatility, during oracle update, during liquidation cascade
- Block position: early in block, mid-block, late in block (ordering matters for MEV exposure)

**Minimum sample:** 500 blocks per operation type for statistical significance. Updated monthly with fresh blocks.

### 1.2 Replay procedure [SPEC]

For each selected block B where the golem acted:

1. **Fork at B-1.** Start mirage-rs in historical mode: `--from-block (B-1) --to-block B`.
2. **Apply state at B-1.** All upstream reads pin to block B-1. The state before the golem's transaction is the baseline.
3. **Execute the golem's transaction.** Submit the exact same transaction (same calldata, same gas limit, same value) via `eth_sendTransaction`.
4. **Capture simulated output.** Record: output amounts, gas used, events emitted, final storage state of affected contracts.
5. **Compare against on-chain receipt.** Compute deltas for each metric.

```rust
pub struct ValidationResult {
    pub block_number: u64,
    pub tx_hash: B256,
    pub operation_type: OperationType,

    /// Simulated vs actual output token amounts.
    pub amount_delta_bps: f64,

    /// Simulated vs actual gas used.
    pub gas_delta_pct: f64,

    /// Events that appeared in simulation but not on-chain (or vice versa).
    pub event_mismatches: Vec<EventMismatch>,

    /// Storage slots where simulated final value differs from on-chain.
    pub storage_divergences: Vec<StorageDivergence>,

    /// Pass/fail against the threshold for this operation type.
    pub passed: bool,
}

pub struct StorageDivergence {
    pub address: Address,
    pub slot: U256,
    pub simulated: U256,
    pub actual: U256,
    pub delta_pct: f64,
}
```

### 1.3 Comparison normalization [SPEC]

Token amounts are compared in basis points (bps) relative to the actual output:

```
delta_bps = abs(simulated - actual) / actual * 10_000
```

For zero-output transactions (reverts), comparison is binary: did both revert, and if so, with the same reason?

---

## 2. Regression Thresholds [SPEC]

### 2.1 Threshold table [SPEC]

| Operation Type | Tolerance | Metric | Notes |
|---------------|-----------|--------|-------|
| Single-hop swap | 0.1% (10 bps) | Output token amount | Tightest threshold. State at B-1 is exact. |
| Multi-hop swap (2-3 hops) | 0.5% (50 bps) | Final output amount | Intermediate pools may have different state. |
| Complex multi-hop (4+ hops) | 1.0% (100 bps) | Final output amount | Aggregator routing is path-dependent. |
| LP mint (concentrated) | 0.1% (10 bps) | Liquidity token amount | Deterministic given tick range and amounts. |
| LP burn + collect fees | 0.5% (50 bps) | Collected fee amounts | Fee accumulation depends on intervening swaps. |
| Borrow/repay | 0.1% (10 bps) | Debt token amount | Interest accrual adds minor variance. |
| Health factor check | 0.01 absolute | Health factor value | Tight: 1.01 vs 1.02 matters for liquidation proximity. |
| Vault deposit/withdraw | 0.5% (50 bps) | Share amount | Share price depends on all prior deposits. |
| Gas used | 5% | Gas units | Minor EVM opcode cost differences are acceptable. |
| Transaction revert | Exact match | Reverted yes/no | If actual reverted, simulation must also revert. |

### 2.2 Threshold rationale [SPEC]

The 0.1% swap tolerance accounts for: rounding in Uniswap v3's `SqrtPriceMath`, potential 1-wei differences in token transfer amounts, and the fact that mirage reads state at B-1 while the actual transaction executed after prior transactions in block B.

The 1.0% multi-hop tolerance is wider because each hop compounds small differences. A 3-hop route through pools with slightly different reserves can produce a 0.3-0.5% aggregate difference even with correct per-pool math.

The health factor tolerance is absolute (0.01) rather than relative because health factors near 1.0 are the danger zone. A relative tolerance would be too permissive at low health factors.

### 2.3 Failure classification [SPEC]

| Classification | Criteria | Response |
|---------------|----------|----------|
| Pass | All metrics within tolerance | No action |
| Soft fail | 1-2 metrics exceed tolerance by < 2x | Log warning, investigate at next review |
| Hard fail | Any metric exceeds tolerance by > 2x | Block release, investigate immediately |
| Catastrophic fail | Simulation succeeds but on-chain reverted (or vice versa) | Block release, root-cause required |

---

## 3. Swap Accuracy Tests [SPEC]

### 3.1 Single-hop Uniswap v3 swap [SPEC]

**Test:** Replay 500 historical single-hop swaps (ETH/USDC, WBTC/ETH, etc.) through mirage.

**Properties:**
- Output amount within 10 bps of actual in >= 99% of cases
- Tick after swap matches actual tick in 100% of cases
- sqrtPriceX96 after swap matches actual within 1 wei in >= 99% of cases

```rust
#[test]
fn validate_uniswap_v3_single_hop_swaps() {
    let test_blocks = load_labeled_blocks("swap_single_hop", 500);

    let mut results = Vec::new();
    for block in &test_blocks {
        let result = replay_and_compare(block);
        results.push(result);
    }

    let pass_rate = results.iter()
        .filter(|r| r.amount_delta_bps <= 10.0)
        .count() as f64 / results.len() as f64;

    assert!(
        pass_rate >= 0.99,
        "single-hop swap pass rate {:.2}% < 99%",
        pass_rate * 100.0
    );

    // Report the worst-case delta.
    let worst = results.iter()
        .max_by(|a, b| a.amount_delta_bps.partial_cmp(&b.amount_delta_bps).unwrap())
        .unwrap();
    println!(
        "worst case: block {} delta {:.1} bps",
        worst.block_number, worst.amount_delta_bps
    );
}
```

### 3.2 Swap during oracle update [SPEC]

**Test:** Select blocks where a Chainlink oracle update and a swap occur in the same block. The oracle update changes the price reference mid-block. Verify mirage handles the ordering correctly (oracle update before swap, or after, matching actual ordering).

### 3.3 Swap with slippage revert [SPEC]

**Test:** Select blocks where the golem's swap reverted on-chain due to slippage. Verify the simulation also reverts. Verify the revert reason matches (e.g., `"Too little received"` from Uniswap's router).

---

## 4. Multi-Hop Accuracy Tests [SPEC]

### 4.1 Two-hop routes through the Universal Router [SPEC]

**Test:** Replay 500 historical multi-hop swaps routed through Uniswap's Universal Router. Compare final output amount.

**Expected:** 99% within 50 bps. The extra tolerance accounts for intermediate pool state differences.

### 4.2 Complex aggregator routes (1inch, CoW Protocol) [SPEC]

**Test:** Replay 200 historical swaps routed through third-party aggregators. These routes may split across multiple pools and protocols.

**Expected:** 95% within 100 bps. Aggregator routing is path-dependent and may choose different paths when pool states differ slightly.

### 4.3 Multi-hop with MEV sandwich [SPEC]

**Test:** Select blocks where the golem's swap was sandwiched (frontrun + backrun). Verify that mirage replays the sandwich transactions in the correct order and the golem's output matches the (degraded) on-chain result.

This is a failure mode test: the golem should see the same bad outcome in simulation that it experienced on-chain, so it can learn from it.

---

## 5. LP Position Accuracy Tests [SPEC]

### 5.1 Fee accrual over N blocks [SPEC]

**Test:** Fork at block B. The golem has an active Uniswap v3 LP position. Replay blocks B through B+1000 in `hybrid` mode. At B+1000, collect fees via simulation. Compare against actual fees collected at B+1000.

**Expected:** Within 50 bps. Fee accrual depends on every swap that crosses the position's tick range, so small state divergences compound.

```rust
#[test]
fn validate_fee_accrual_over_1000_blocks() {
    let test_cases = load_labeled_blocks("lp_fee_accrual", 50);

    for case in &test_cases {
        let mirage = start_historical(case.from_block, case.to_block);
        mirage.replay_range().unwrap();

        let simulated_fees = mirage.collect_fees(case.position_id).unwrap();
        let actual_fees = case.actual_fees;

        let delta_bps = (simulated_fees - actual_fees).abs()
            / actual_fees * 10_000.0;

        assert!(
            delta_bps <= 50.0,
            "fee accrual delta {:.1} bps > 50 bps at block range {}-{}",
            delta_bps, case.from_block, case.to_block
        );
    }
}
```

### 5.2 Tick crossing accuracy [SPEC]

**Test:** During historical replay, verify that the simulated pool tick matches the actual on-chain tick at each block where a swap occurred. A tick mismatch means the simulation's price is wrong, which compounds into incorrect fee attribution.

**Expected:** Exact match in 99% of blocks. The remaining 1% tolerance accounts for blocks where mirage's targeted replay missed a transaction (the intentional imprecision of the matching heuristics).

### 5.3 LP position out-of-range detection [SPEC]

**Test:** Replay blocks where an LP position went out of range. Verify that mirage detects the same block where the position left the active tick range.

---

## 6. Lending Protocol Accuracy Tests [SPEC]

### 6.1 Health factor tracking [SPEC]

**Test:** Fork with an active Aave position. Replay 500 blocks including oracle updates. At each oracle update block, compare simulated health factor against actual.

**Expected:** Within 0.01 absolute. Health factor computation is deterministic given the oracle price and position state, so the only source of error is state divergence.

### 6.2 Interest rate accrual [SPEC]

**Test:** Replay 1000 blocks of an active borrow position. Compare simulated accrued interest against actual.

**Expected:** Within 10 bps. Interest accrues per-block based on the utilization rate, which depends on all deposits and borrows in the pool.

### 6.3 Liquidation trigger accuracy [SPEC]

**Test:** Select blocks where a liquidation occurred on-chain for a position the golem was tracking. Verify that the simulation also triggers the liquidation at the same block (or within 1 block).

**Expected:** Same block in 95% of cases. 1-block offset acceptable if the mirage missed a single oracle update due to targeted replay filtering.

---

## 7. Shadow Execution Validation [SPEC]

Shadow execution replays historical blocks with modified contract bytecode. Validation checks that the comparison between original and shadow executions is internally consistent.

### 7.1 Property: Identical bytecode produces zero divergence [SPEC]

**Statement:** When `ShadowConfig.bytecode_overrides` maps an address to its own existing bytecode, the shadow execution produces zero divergent slots and zero PnL delta.

```rust
#[test]
fn shadow_noop_produces_zero_divergence() {
    let config = ShadowConfig {
        bytecode_overrides: HashMap::from([(
            pool_address,
            get_deployed_bytecode(pool_address),
        )]),
        emit_shadow_events: true,
        diff_output: None,
    };

    let result = run_shadow_replay(19_400_000, 19_400_100, config);

    assert_eq!(result.block_diffs.iter()
        .map(|d| d.divergent_slots.len())
        .sum::<usize>(), 0);
    assert!(result.total_pnl_delta.values()
        .all(|&d| d == 0));
}
```

### 7.2 Property: Divergence is monotonically tracked [SPEC]

**Statement:** Once a shadow execution diverges from the original at block B, `ShadowResult.first_divergence_block` is `Some(B)`. Subsequent identical blocks do not reset it.

### 7.3 Shadow PnL attribution [SPEC]

**Test:** Replace a pool's fee tier bytecode (e.g., 0.3% -> 0.05%) and replay 1000 blocks. Verify that the total PnL delta equals the sum of per-block PnL deltas. No accounting gaps.

---

## 8. Scenario Runner Validation [SPEC]

### 8.1 LHS sample coverage [SPEC]

**Test:** Generate 32 LHS samples across 4 dimensions. Verify that each dimension's samples fall in exactly 32 distinct strata (one per equal-probability interval). No two samples share a stratum in any dimension.

```rust
#[test]
fn lhs_stratification() {
    let space = ParameterSpace {
        dimensions: vec![
            ("size".into(), ParameterDimension::LogNormal {
                mu: 0.0, sigma: 1.0, min: 0.01, max: 100.0,
            }),
            ("slippage".into(), ParameterDimension::Uniform {
                min: 0.001, max: 0.05,
            }),
            ("gas".into(), ParameterDimension::Uniform {
                min: 5.0, max: 200.0,
            }),
            ("fee_tier".into(), ParameterDimension::Discrete(
                vec![100.0, 500.0, 3000.0, 10000.0],
            )),
        ],
    };

    let mut rng = StdRng::seed_from_u64(42);
    let samples = latin_hypercube(&space, 32, &mut rng);

    assert_eq!(samples.len(), 32);
    assert!(samples.iter().all(|row| row.len() == 4));

    // Verify stratification: for each dimension, map each
    // sample back to its stratum index. All 32 strata should
    // appear exactly once.
    for col in 0..4 {
        let mut strata: Vec<usize> = samples.iter()
            .map(|row| {
                // Approximate inverse: which of the 32 equal-probability
                // intervals does this value fall in?
                let u = cdf_of_dimension(&space.dimensions[col].1, row[col]);
                (u * 32.0).floor() as usize
            })
            .collect();
        strata.sort();
        strata.dedup();
        assert_eq!(strata.len(), 32, "dimension {} has duplicate strata", col);
    }
}
```

### 8.2 Bayesian optimization convergence [SPEC]

**Test:** Optimize a known quadratic objective function `f(x) = -(x - 3)^2 + 10` using the BayesianOptimizer. After 20 evaluations (8 LHS + 12 GP-guided), the best observed x should be within 0.5 of x=3.

### 8.3 CFR regret computation [SPEC]

**Test:** Provide a CfrAnalyzer with known counterfactual PnL values. Verify `total_regret = max(counterfactual_PnL) - actual_PnL` exactly. Verify `per_exit_regret[i] = max(counterfactual_PnL) - counterfactual_PnL[i]` for all i.

---

## 9. Continuous Regression Suite [SPEC]

### 9.1 Nightly validation pipeline [SPEC]

Runs every night against the most recent 24 hours of golem transactions:

1. **Collect** all golem transactions from the last 24 hours.
2. **Stratify** by operation type (at least 10 per type).
3. **Replay** each through mirage in historical mode.
4. **Compare** against on-chain receipts.
5. **Report** pass/fail per threshold, plus aggregate statistics.

### 9.2 Alerting [SPEC]

| Condition | Severity | Action |
|-----------|----------|--------|
| Pass rate drops below 95% for any operation type | Warning | Notify dev channel |
| Any catastrophic fail (sim succeeds, on-chain reverts) | Error | Block deployment, page on-call |
| Aggregate worst-case delta > 2x tolerance | Warning | Investigate state divergence source |
| 3 consecutive nights with soft fails | Error | Root-cause investigation required |

### 9.3 Regression dataset management [SPEC]

**Storage:** Labeled blocks are stored as compressed JSON files, one per operation type. Each file contains 500+ block entries with transaction data and expected outcomes.

**Refresh:** Monthly. Old blocks are archived, new blocks from the last month are added. The dataset should always contain recent blocks to catch Ethereum protocol changes (hardforks, EIP implementations).

**Versioning:** Dataset version is pinned in `Cargo.toml` for reproducibility. A new dataset version requires a full regression run before promotion.

---

## Cross-references

- `16-testing/11-mirage-v2-testing.md` -- engine correctness tests (HybridDB read priority, DirtyStore isolation, CoW branching, Block-STM determinism); prerequisite for this document
- `16-testing/04-mirage.md` -- v1 mirage testing (Anvil-based fork infrastructure and regime scenario tests)
- Research: `02-mirage-rs/00-architecture.md` -- HybridDB three-tier read model, targeted replay, and DirtyStore design
- Research: `02-mirage-rs/02-scenario-runner.md` -- Latin Hypercube Sampling, Bayesian optimization, and Counterfactual Regret analysis
- Research: `02-mirage-rs/03-historical-mode.md` -- historical block replay, shadow execution with modified bytecode, and PnL attribution

## References

- Gelashvili, R. et al. (2023). Block-STM: Scaling blockchain execution. *PPoPP*. — *Parallel execution algorithm used in mirage-rs historical replay; determinism is validated by the simulation validation tests.*
- McKay, M.D. et al. (1979). A comparison of three methods for selecting values of input variables. *Technometrics*, 21(2). — *Introduces Latin Hypercube Sampling (LHS), the stratified sampling method used by the scenario runner to cover the parameter space with fewer samples than grid search.*
- Snoek, J. et al. (2012). Practical Bayesian optimization of machine learning algorithms. *NeurIPS*. — *Gaussian Process-based optimization approach adapted for the scenario runner's GP-guided phase after initial LHS exploration.*
- Zinkevich, M. et al. (2007). Regret minimization in games with incomplete information. *NeurIPS*. — *Counterfactual regret framework used by the CFR analyzer to compute per-exit regret scores and total regret against optimal timing.*
