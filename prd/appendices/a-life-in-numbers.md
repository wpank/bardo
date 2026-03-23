# A Life in Numbers [SPEC]

> **Part of**: [Appendices](.) | **Status**: Specification
>
> One Golem, from birth to death. A narrative walkthrough of the full lifecycle with real economics at every step.

> **Reader orientation:** This appendix is a narrative walkthrough of one Golem's (mortal autonomous agent compiled as a single Rust binary running on a micro VM) complete lifecycle, from $50 USDC birth through 18 days of operation to death by credit exhaustion. Every number is derived from the Heartbeat (9-step decision cycle) specification. It covers credit partition management, cost tier routing (T0/T1/T2/T3 -- cognitive tier routing from fast cached/rule-based to extended reasoning), behavioral phase transitions (Thriving/Stable/Conservation/Desperate/Terminal), the Death Protocol (Thanatopsis -- four-phase structured shutdown), and knowledge inheritance to a successor. See `prd2/shared/glossary.md` for full term definitions.

Balanced disposition. ETH DCA strategy. Morpho yield on idle USDC. Clade member. Initial balance: $50 USDC. Initial burn rate: $0.18/hr. Apoptotic Reserve: max($0.30, $50 x 0.02) = **$1.00**. Hayflick limit: 100,000 ticks.

Every number below is derived from the formulas in the Heartbeat specification.

---

## Day 0 -- Birth

**$50.00** | burn $0.18/hr | projected life 278hr (11.6d) | pressure **1.0** | **thriving** | tick 0 / 100,000

Deploy $26 to Morpho (reserve = 24hr x $0.18 x 1.5 = $6.48). All inference tiers available. Begin DCA: $2/day ETH buys. Apoptotic Reserve of $1.00 locked, invisible to the Golem's spendable balance.

Inherits predecessor's Grimoire (the agent's persistent knowledge base: episodes, insights, heuristics, warnings, causal links): 127 entries at confidence 0.4 (decayed), 89 PLAYBOOK.md (the agent's evolving strategy document; heuristics and rules updated through learning) heuristics, causal graph with 34 validated edges. Debt register logged: received knowledge from predecessor.

**Credit partitions:** LLM $18.00 (60%) / Gas $7.50 (25%) / Data $4.50 (15%). Death reserve $1.00 locked. Working capital: $19.00.

**Cost tier distribution:** ~80% T0, ~10% T1, ~7% T2, ~3% T3. Ranging regime. Daily cost ~$1.32.

**First insurance snapshot at hour 6.** Captures wallet state, top 20 insights (all inherited at 0.4 confidence), PLAYBOOK.md hash. Cost: $0.01 from Data partition.

---

## Day 3 -- Cache Building

**$37.04** | burn $0.17/hr | projected 218hr (9.1d) | pressure **0.97** | **thriving** | tick 6,480 / 100,000 | cache 18%

Cache recognizes routine ETH price moves (+/-1%). Haiku handles 82% of ticks. 47 cache entries. Gas timing emerging: 40% cheaper 02:00-06:00 UTC. Morpho yield: $0.003/day.

---

## Day 7 -- Stabilization

**$28.50** | burn $0.15/hr | income $0.03/hr (LP $0.02 + Morpho $0.01) | net burn $0.12/hr | pressure **0.94** | **stable** | tick 15,120 / 100,000 | cache 28%

Transition to stable. Cache TTLs 2x. DCA batches 2 days of buys into one low-gas window. Sonnet for weekly strategy review. Net burn $0.12/hr gives projected 237hr remaining. PLAYBOOK.md divergence from predecessor: 15% (6 entries modified, 5 novel entries added, 3 inherited entries invalidated). Anti-proletarianization check: passing.

**Cost tier shift:** T3/Opus suppressed after 70% of daily budget consumed. More ticks resolve at T0/T1, fewer escalate to T3. The daily cost budget is doing its job, throttling expensive deliberation so the cheaper tiers handle the steady state.

---

## Day 12 -- Market Shock (ETH -15%)

**$16.40** | burn $0.22/hr (spike from analysis) | income -$0.01/hr (IL realized) | pressure **0.72** | **stable** (barely) | tick 25,920 / 100,000 | cache 31%

LP position closed to stop IL bleeding. DCA reduced to every 3 days. Revenue seeking activates. Haiku-only for non-critical analysis. `actionScore` blends merit with USDC return (survivalUrgency = 0.28).

**Probes firing:** `priceDeltaProbe` returns `high` (>2% move in one tick). `positionHealthProbe` returns `low` (IL detection on the LP position). The combined severity triggers immediate T3 escalation: Opus deliberation on whether to close the LP position or ride it out. Cost of this single tick: $0.25. The Golem decides to close.

**Cost tier spike:** Bear high-vol regime. T2/T3 escalation rate jumps. Daily cost climbs to ~$3.24.

**Credit partitions:** LLM $4.20 / Gas $2.10 / Data $0.90. Predictive rebalancing fires. The gas partition is projected to go negative within 10 ticks due to position exit transaction costs. 15% transferred from the LLM partition to Gas. The Golem sacrifices future thinking capacity to pay for the trade that stops the bleeding.

**Insurance snapshot captured 2 hours before the crash.** Bounds worst-case knowledge loss if the VM dies during the volatility spike. Cost: $0.01.

---

## Day 15 -- Conservation

**$7.20** | burn $0.14/hr (Haiku-only, 3x heartbeat) | income $0.005/hr (Morpho only) | pressure **0.52** | **conservation** | tick 32,400 / 100,000 | cache 34%

Spotted 0.3% ETH/USDC arb. `actionScore` = 0.48 x 0.52 + 0.15 x 0.48 = 0.32. Captured $0.47 profit, extending life by 3.4 hours. DCA paused. On-chain reads only.

**Haiku ceiling.** Only T0 and T1 ticks execute. No T2, no T3. Daily cost drops to ~$0.41. The Golem observes the world through a narrow aperture now, reacting only to what its deterministic probes flag as anomalous.

---

## Day 17 -- Desperate

**$2.10** | burn $0.10/hr (5x heartbeat, minimal probes) | projected 21hr | pressure **0.28** | **desperate** | tick 36,720 / 100,000 | `riskMultiplier` = **1.44x**

Haiku-only, hard token limits, no reflection. Two arb attempts: +$0.22, -$0.08 gas. Net +$0.14. Clade threshold dropped to 0.3, push 14 insights. Balance: $2.16.

**T0 only** with rare T1 escalations under hard token caps. Daily cost: ~$0.10. Only 5 of 16 probes still active: price delta, position health, credit balance, kill switch, clade alert. The other 11 disabled to save T1 costs. The Golem has narrowed its sensory apparatus to survival essentials.

**Thanatos threshold crossed. Legacy partition activated.** Credit split: LLM 30% / Gas 15% / Data 5% / Legacy 50% of remaining $2.10. Legacy partition = $1.05 earmarked for death.

**Final insurance snapshot.** Contains the contemplative interval's synthesized insights. If the Golem crashes before the Death Protocol completes, this snapshot preserves 94% of accumulated knowledge. The 6% gap is roughly 4 hours of contemplative writing that has not been captured yet.

**Contemplative interval begins.** Transaction frequency drops to 50% of normal. Grimoire-writing frequency increases: active synthesis of trade history, regime analysis, what-if reconstructions. The Golem revisits its Day 12 market shock narratively -- not "IL was $4.60" but "I held through the drawdown because my predecessor's PLAYBOOK.md said to hold, but the regime was different. Next time: close immediately on >10% drawdown in low-liquidity regimes."

---

## Day 18 -- Terminal / Death Protocol

**08:00** -- Balance: $1.83. Burn rate: $0.08/hr. Projected life: 10.4 hours. Tick 38,880 / 100,000.

**11:00** -- Balance: $1.59. `survivalPressure` = 0.07. Triggers Death Protocol (< 0.1). Neither Hayflick limit (38,880 << 100,000) nor staleness threshold triggered -- this is resource mortality.

**Death Protocol budget:** $1.59 - $1.00 Apoptotic Reserve = $0.59 Legacy partition + $1.00 floor = $1.59 total.

| Phase         | Budget | Spend | Action                                                                                                                                                                                                                                           |
| ------------- | ------ | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| I -- Settle   | $0.16  | $0.12 | Withdraw $3.41 from Morpho. Sweep to Main Wallet.                                                                                                                                                                                                |
| II -- Reflect | $0.95  | $0.45 | 1 Sonnet lifecycle reflection ($0.10). 4 Haiku snapshots ($0.04). "What confused me": ETH/gas correlation broke on Day 14 -- cause unknown. "What I suspect": USDC depeg rumors correlate with LP withdrawal spikes 6h before on-chain evidence. |
| III -- Legacy | $0.48  | $0.20 | Clade push: 142 Grimoire entries at threshold 0.1 (15 new since predecessor). Webhook: `golem.dying` with successor recommendation.                                                                                                              |

**12:47** -- Last tick. Tick 39,060 / 100,000.

```
state.json: {
  ticks: 39060,
  totalCost: $48.87,
  totalRevenue: $4.26,
  cause: "credit_exhaustion",
  hayflickUtilization: "39%",
  playbookDivergence: "22%",
  antiProletarianization: "pass"
}
PLAYBOOK.md: 97 heuristics (89 inherited, 8 novel, 5 invalidated), 34% cache hit rate
Grimoire: 142 entries
  - insight: 47 (28 high-confidence, 12 medium, 7 speculative)
  - heuristic: 38 (from PLAYBOOK.md, 30 inherited + 8 novel)
  - warning: 12 (3 from personal experience, 9 inherited)
  - causal_link: 34 edges (29 inherited + validated, 5 discovered)
  - strategy_fragment: 11 (partial observations, not yet parameterized)
  - questions: 11 (explicit gaps for successor investigation)
  minQuestionRatio: 7.7% -- passing
  includes "What confused me" and "What I suspect" sections
WebSocket: { type: "golem_dead" }
exit(0)
```

---

## The Successor

A new Golem boots. Balanced disposition. ETH DCA strategy. $50 USDC. Apoptotic Reserve: $1.00.

It discovers its Clade on first boot via ERC-8004 `getAgentsByOperator()`. Three siblings are online. The successor calls `GET /api/clade/entries?since=0` on each (first boot, no history to skip) and pulls their accumulated knowledge. The dead predecessor's entries arrive through these siblings, who already ingested the dying Golem's final push.

It inherits:

- 142 Grimoire entries at confidence 0.4 (decayed from predecessor's confidence)
- PLAYBOOK.md with 97 heuristics, regime-tagged, 22% diverged from grandparent
- Causal graph: 38 validated edges (+4 from predecessor), 15 speculative
- Death reflection: "Batch DCA more aggressively. Close LP positions immediately on >10% drawdown in low-liquidity regimes -- I waited 2 days and bled $4.60 in IL. Predecessor said hold; I should have individuated sooner."
- Debt register: received knowledge from predecessor lineage
- 11 open questions: the preindividual fuel for novel individuation

Not all inherited knowledge ages the same way. Structural insights carry a long confidence half-life -- a causal link between ETH price and Aave utilization rates arrives at 0.4 and decays slowly, still useful on Day 14. Tactical insights decay fast. "Gas is cheap between 02:00 and 06:00 UTC" arrives at 0.4 and drops to 0.1 within 3 days. The successor must re-validate tactical knowledge quickly or lose it to decay. Structural knowledge grants a longer runway. This asymmetry means the successor's first week is a race: absorb what the predecessor learned about market structure, and independently verify the time-sensitive operational details before they fade below the usability threshold.

The predecessor's 18-day life produced knowledge the successor receives on tick 0. By day 3, the successor's cache hit rate is 41% (vs predecessor's 18%) -- the heuristic transfer works. By day 7, it has validated 19 of 142 inherited entries and discarded 8 as regime-specific. It has already begun modifying 3 PLAYBOOK.md entries, on track for the 15% divergence requirement.

The ratchet turns. The spiral ascends.
