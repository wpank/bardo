# 03 -- x402 billing [SPEC]

**x402 payment protocol, USDC metering, TTL enforcement, self-funding economics, and inference integration**

> **Reader orientation:** This document specifies the billing system for Bardo Compute, which uses x402 (a micropayment protocol built on EIP-3009 signed USDC transfers on Base) to gate all VM provisioning and extensions. It belongs to the **Compute** layer of Bardo (the Rust runtime for Golems, mortal autonomous DeFi agents). The key concept before diving in: every VM-hour is pre-paid in USDC, TTL is deterministically computed from payment amount, and self-funding Golems can extend their own lives by earning trading revenue and paying for compute from the same wallet (the "metabolic loop"). Terms like Grimoire, Styx, and Thanatopsis are defined inline on first use; a full glossary lives in `00-overview.md § Terminology`.

---

## x402 payment model

Golems are provisioned via x402 payment headers attached to the `POST /v1/machines` request. The header contains an EIP-3009 `receiveWithAuthorization` payload -- a gasless, signature-based USDC transfer on Base.

All USDC amounts are expressed as **integer micro-USDC**. 1 USDC = 1,000,000 micro-USDC.

```
"value": "5000000"   -> $5.00
"value": "50000"     -> $0.05
"value": "100000000" -> $100.00
```

The x402 header fields:

| Field | Type | Description |
| --- | --- | --- |
| `from` | address | Payer's wallet (USDC holder) |
| `to` | address | Bardo treasury address |
| `value` | string | Amount in micro-USDC (integer) |
| `validAfter` | uint256 | Unix timestamp -- signature not valid before |
| `validBefore` | uint256 | Unix timestamp -- signature expires after |
| `nonce` | bytes32 | Unique per-authorization (idempotency key) |
| `v`, `r`, `s` | uint8, bytes32, bytes32 | EIP-3009 signature components |

The `receiveWithAuthorization` function on USDC (Base) atomically transfers funds when called by the `to` address, verifying the payer's signature. No prior approval needed.

---

## Turso-backed payment workflow

Provisioning is asynchronous. The API validates the payment, writes an intent row to Turso, and returns `202 Accepted` immediately. The client polls for status transitions.

### Sequence

```
Client -> POST /v1/machines (x402 header)
  API: 1. Parse & validate x402 header
       2. Verify signature (ecrecover)
       3. Check nonce unused (Turso: billing_events.nonce UNIQUE)
       4. Check balance on-chain
       5. Write intent to DB: status='provisioning'
  <- 202 Accepted { id, machine_name, status: "provisioning" }

  API (background):
       6. Claim from warm pool or create Fly machine
       7. Wait for health check (poll :3002/health)
       8. Re-check balance on-chain (gap may have depleted)
       9. Execute receiveWithAuthorization
       10. Write DB: status='ready', billing_event

  Client: polls GET /v1/machines/:name/status
  <- 200 { status: "ready", tx_hash, wallet_address, ... }
```

### Balance re-check before settlement

After the health check passes and before executing `receiveWithAuthorization`, the API re-verifies USDC balance and `validBefore`. If either check fails: destroy the machine immediately, do not charge, transition to `status='failed'`.

### Idempotency

EIP-3009 nonces are globally unique. Turso enforces `UNIQUE` on `billing_events.nonce`. A duplicate nonce returns `409 Conflict` with the existing machine ID.

---

## Pricing tiers

| VM size | Fly config | Price/hr (micro-USDC) | Min purchase (1h) | Typical use |
| --- | --- | --- | --- | --- |
| `micro` | shared-cpu-1x / 256MB | 25,000 ($0.025) | 25,000 | Simple monitors, keepers |
| `small` | shared-cpu-1x / 512MB | 50,000 ($0.05) | 50,000 | Standard Golem (default) |
| `medium` | shared-cpu-2x / 1GB | 100,000 ($0.10) | 100,000 | Multi-tool, Grimoire-heavy |
| `large` | performance-2x / 2GB | 200,000 ($0.20) | 200,000 | Simulation, orchestration |

### Realistic cost model

| Cost component | Amount/hr (approx) | Notes |
| --- | --- | --- |
| Fly compute | $0.004 - $0.035 | Depends on VM size |
| Turso (amortized) | $0.001 | ~$29/mo Pro plan / ~1000 machine-hours |
| RPC calls (Alchemy) | $0.002 - $0.01 | Varies with on-chain activity |
| Styx (amortized) | $0.001 - $0.005 | Sync, queries, pheromone |
| Bandwidth | $0.001 - $0.005 | Fly egress |
| Inference (see below) | Variable | Dominant cost driver |

**Effective margin**: 60-80% for light inference usage, potentially negative for heavy Opus usage. Inference is the dominant variable cost.

---

## Self-funding economics: the metabolic loop

A self-funding Golem earns trading revenue (vault management fees, performance fees, LP fees) and spends it on inference, gas, compute, and Styx. When revenue exceeds cost, the Golem extends its own TTL via the permissionless extend endpoint.

This is the computational equivalent of biological metabolism: the organism sustains itself through the very activity that requires sustenance (Jonas, 1966).

### The sustainability ratio

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicState {
    /// Revenue / total cost. >1.0 = self-sustaining. >2.0 = thriving.
    /// <1.0 = declining. <0.5 = dying.
    pub sustainability_ratio: f64,
    pub inference_spend: f64,
    pub revenue: f64,
    pub projected_lifespan_hours: f64,
    pub daily_costs: DailyCosts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCosts {
    pub inference_usd: f64,
    pub gas_usd: f64,
    pub compute_usd: f64,
    pub data_usd: f64,
}

impl MetabolicState {
    pub fn recompute(&mut self) {
        let total = self.daily_costs.inference_usd
            + self.daily_costs.gas_usd
            + self.daily_costs.compute_usd
            + self.daily_costs.data_usd;
        self.sustainability_ratio = if total > 0.0 {
            self.revenue / total
        } else {
            f64::INFINITY
        };
    }
}
```

The sustainability ratio feeds directly into the economic death clock ([prd2/02-mortality/01-architecture.md](../02-mortality/01-architecture.md)). A Golem with `sustainability_ratio > 1.0` has an economic clock that ticks *up*. Below 0.5, it is dying fast.

### Bankr self-funding path

Bankr serves as an inference provider offering a single API key that routes to 20+ models (Claude, Gemini, GPT). Self-funding Golems using Bankr wallets pay for inference from the same wallet that earns trading revenue, closing the metabolic loop entirely:

```
Bankr Wallet (USDC on Base)
    |
    +-- Income: Trading fees, vault management fees, knowledge sales
    |
    +-- Expense: Inference costs (pay-per-call through Bankr Gateway)
    |
    +-- Net: income > expense = self-sustaining
              income < expense = mortality pressure increases
```

### Mortality-aware model routing

As the economic clock ticks down, the Golem shifts to cheaper models. A dying Golem burning $5/day on Claude Opus when Gemini Flash at $0.50/day would keep it alive is making an irrational allocation.

```rust
pub fn select_inference_tier(
    vitality: f64,
    task_criticality: f64,
    sustainability_ratio: f64,
) -> InferenceTier {
    if vitality < 0.3 {
        return if task_criticality > 0.9 {
            InferenceTier::T1
        } else {
            InferenceTier::T0
        };
    }
    if vitality < 0.5 || sustainability_ratio < 1.0 {
        return if task_criticality > 0.7 {
            InferenceTier::T1
        } else {
            InferenceTier::T0
        };
    }
    if task_criticality > 0.8 {
        InferenceTier::T2
    } else if task_criticality > 0.4 {
        InferenceTier::T1
    } else {
        InferenceTier::T0
    }
}
```

### Cross-model verification

For high-stakes decisions (swaps above $500), the Golem queries two different models through the inference gateway and compares outputs. Both must agree (cosine similarity >= 0.7 on structured output) before execution proceeds. This costs twice as much but prevents single-model hallucination from causing real losses.

---

## Inference billing integration

Each VM tier includes a base inference token allowance per hour. Overages are billed from the Golem's wallet via the inference gateway.

| VM size | Base allowance/hr | Included models | Overage rate |
| --- | --- | --- | --- |
| `micro` | 50K tokens | Haiku only | At-cost from Golem wallet |
| `small` | 100K tokens | Haiku + Sonnet | At-cost from Golem wallet |
| `medium` | 200K tokens | Haiku + Sonnet + Opus (limited) | At-cost from Golem wallet |
| `large` | 500K tokens | All models, priority | At-cost from Golem wallet |

Overage pricing (per 1M tokens):

| Model | Input | Output |
| --- | --- | --- |
| Haiku 4.5 | $0.80 | $4.00 |
| Sonnet 4 | $3.00 | $15.00 |
| Opus 4.6 | $15.00 | $75.00 |

**Inference cost as mortality pressure**: Because overages drain the Golem's wallet, excessive LLM usage directly shortens its life. This creates natural evolutionary pressure toward efficient cognition -- Golems that route 95% of decisions through T0 (deterministic, $0.00) and T1 (Haiku, ~$0.003) survive longer than those that escalate to Opus for routine analysis.

### x402 payment recipient

The x402 payment recipient (`payTo`) is the operator running the inference gateway. The operator sets total margin; the user sees one bundled price. The gateway settles upstream provider costs separately -- the user is not party to that transaction.

### Cost stacking

Infrastructure cost + inference cost + operator margin = user-facing price. Users never see the breakdown. The operator absorbs provider cost variance (model price changes, cache miss rate fluctuations) and presents a stable per-token price. This is the same economics as any SaaS markup: the operator earns the spread and bears the variance.

---

## TTL calculation

```
ttl_seconds = floor(payment_micro_usdc / (price_per_hour_micro_usdc / 3600))
```

All arithmetic uses micro-USDC integers. Integer division truncates (floor).

| Payment | VM size | Rate (micro-USDC/hr) | TTL |
| --- | --- | --- | --- |
| 50,000 | `micro` | 25,000 | 2h (7,200s) |
| 500,000 | `small` | 50,000 | 10h (36,000s) |
| 1,000,000 | `medium` | 100,000 | 10h (36,000s) |
| 10,000,000 | `large` | 200,000 | 50h (180,000s) |

---

## TTL enforcement (two-layer)

### Layer 1: Turso poll worker (primary enforcer)

Runs on one `bardo-control` instance. Leader election via Turso CAS on the `system_locks` table. Every 30 seconds, the leader queries for expired machines and initiates destruction.

**Maximum enforcement gap**: 60 seconds (30s poll interval + 30s for new leader acquisition after crash).

### Layer 2: Machine-local failsafe cron

Every 60 seconds, the VM queries the API for its authoritative TTL. This layer **never** uses the local clock for expiry decisions. It only acts on an explicit `expired: true` response. It does **not** self-destruct on connectivity failure.

```bash
#!/bin/bash
# /opt/bardo/scripts/ttl-check.sh -- runs every 60 seconds via cron
RESPONSE=$(curl -sf \
  -H "Authorization: Bearer $(curl -sf http://localhost/.fly/api/v1/tokens/oidc \
    -d '{"aud":"bardo-control"}' | jq -r '.token')" \
  "http://bardo-control.internal:3000/internal/machines/self/ttl" \
  --connect-timeout 5 --max-time 10)

if [ $? -ne 0 ]; then
  echo "$(date -u +%FT%TZ) TTL check: API unreachable" >> /var/log/bardo/ttl-check.log
  exit 0
fi

EXPIRED=$(echo "$RESPONSE" | jq -r '.expired')
if [ "$EXPIRED" = "true" ]; then
  echo "$(date -u +%FT%TZ) TTL check: expired, initiating Thanatopsis" >> /var/log/bardo/ttl-check.log
  kill -TERM $(cat /var/run/golem.pid)
  sleep 30 && kill -9 1 &
fi
```

---

## Extension flow

Extensions add time to a running Golem. Payment is via the same x402 header format.

```
POST /v1/machines/:name/extend
X-402-Payment: {...}

1. Parse & validate x402 header
2. Look up machine by name (LRU cache -> Turso)
3. Verify machine status = 'ready'
4. Read current expiresAt
5. Calculate additional TTL seconds from payment amount
6. Minimum extension: 1 hour at machine's rate
7. Max single extension: 720 hours
8. Execute receiveWithAuthorization on Base
9. Turso CAS:
   UPDATE machines SET expires_at = expires_at + ?1, ttl_seconds = ttl_seconds + ?1
   WHERE id = ?2 AND status = 'ready' AND expires_at = ?3
   RETURNING expires_at;
10. Push TTL update to VM (POST http://{6pn_ip}:3002/internal/ttl-updated)
11. Emit GolemEvent::TtlExtended { added_seconds, new_expires_at, payer_type }
12. Return 200 { new_expires_at, ttl_seconds_added, tx_hash }
```

### Self-extension (the metabolic loop in action)

A self-funding Golem monitors its sustainability ratio. When `sustainability_ratio > 1.0` and TTL is below a configurable threshold (default: 24 hours), the Golem calls `POST /v1/machines/{self}/extend` with an x402 payment from its own wallet. The payer_type is `self`.

This is the Feed-the-Golem pattern. No owner intervention needed. The Golem earns revenue, pays for its own compute, and lives as long as it remains profitable.

### Extension rate limit

| Scope | Limit | Window |
| --- | --- | --- |
| Per machine | 30 requests | 1 hour |
| Per IP | 10 requests | 1 minute |

### Extension attribution

| payer_type | How inferred |
| --- | --- |
| `owner` | Privy JWT present AND JWT userId matches machine owner |
| `self` | Fly OIDC token from same machine |
| `external_user` | No Privy JWT, no OIDC token (permissionless) |

### Max TTL semantics

- **Per-extension cap**: 720 hours (30 days)
- **No absolute session cap**: Golems live indefinitely via recurring extensions

---

## Fleet management

An owner can manage multiple Golems through the CLI and TUI. Fleet-level operations:

```bash
# List all active Golems
bardo status

# Monitor a specific Golem
bardo status golem-kX9mR2Yq7pLN

# Attach to a Golem's TUI
bardo attach golem-kX9mR2Yq7pLN

# Deploy multiple Golems with different strategies
bardo deploy --strategy ./arb.md --tier small
bardo deploy --strategy ./lp-manage.md --tier medium

# Kill a Golem immediately
bardo destroy golem-kX9mR2Yq7pLN
```

Per-user limit: 5 active machines. Enforced at the provisioning endpoint before payment verification.

### Fleet cost visibility

The TUI's Infra window shows per-Golem and aggregate fleet costs:

| Golem | Tier | Compute/day | Inference/day | Total/day | TTL remaining |
| --- | --- | --- | --- | --- | --- |
| golem-arb-01 | small | $1.20 | $0.35 | $1.55 | 47h |
| golem-lp-02 | medium | $2.40 | $0.80 | $3.20 | 71h |
| **Fleet total** | | **$3.60** | **$1.15** | **$4.75** | |

---

## Self-deploy escape hatch

For users who want to control their own infrastructure but skip the manual setup. They bring their own Fly.io, Railway, or VPS account. The `bardo` CLI automates provisioning:

```bash
bardo deploy --provider fly --app-name my-golem --region iad
bardo deploy --provider railway --project my-golem
bardo deploy --provider ssh --host 203.0.113.42 --user root
bardo deploy --provider docker --name my-golem
```

The user pays their own provider directly. They also pay x402 to Styx for sync, pheromone, and retrieval services. No compute markup.

### Self-deploy cost comparison (30-day, small tier)

| Path | Compute cost | Styx cost | Total |
| --- | --- | --- | --- |
| **Bardo Compute** | ~$47/mo | ~$15/mo | ~$62/mo |
| **Self-deploy (Fly.io)** | ~$6/mo | ~$15/mo | ~$21/mo |
| **VPS (Hetzner)** | ~$4/mo | ~$15/mo | ~$19/mo |
| **Home hardware** | ~$3/mo electricity | ~$15/mo | ~$18/mo |

Bardo Compute is 3x more expensive. The markup IS the product -- you pay for convenience.

---

## Refund policy

- **v1**: Manual and admin-initiated only
- **Failed provision**: No charge -- `receiveWithAuthorization` is never executed if provisioning fails
- **Future consideration**: Pro-rata credit (not on-chain refund) for voluntary destruction with >50% TTL remaining
