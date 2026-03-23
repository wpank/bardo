# 04 -- Compute security [SPEC]

**VM isolation, three-mode custody, caveat enforcers, secrets management, and threat model**

> **Reader orientation:** This document specifies the security model for Bardo Compute, covering VM isolation, custody modes, authentication, and threats specific to the compute hosting layer. It belongs to the **Compute** layer of Bardo (the Rust runtime for Golems, mortal autonomous DeFi agents). The key concept before diving in: the system assumes key compromise is possible and bounds the damage via on-chain delegation caveats rather than relying on key secrecy alone. No plaintext wallet secrets exist on VMs. Terms like Golem, PolicyCage, x402, and ERC-8004 are defined inline on first use; a full glossary lives in `00-overview.md § Terminology`.

---

## Threat model

### Attacker taxonomy

| Attacker | Capability | Goal |
| --- | --- | --- |
| **External script kiddie** | Automated scanners, known exploit payloads | Crypto mine, resource theft |
| **Sophisticated external** | Custom tooling, protocol knowledge | Fund theft, data exfiltration |
| **Malicious user** | Valid x402 payments, legitimate VM access | Abuse compute, attack other golems |
| **Compromised golem** | Full VM access, OIDC token | Lateral movement, privilege escalation |
| **Malicious extension payer** | x402 payment ability, no auth | Cost inflation, zombie creation, griefing |
| **Insider (operator)** | Infrastructure access, admin credentials | Fund theft, data exfiltration |

### Attack surfaces

#### 1. x402 payment flow

Signature replay, front-running, double-spend, balance depletion between validation and settlement.

#### 2. Provisioning pipeline

Race conditions on warm pool claims, resource exhaustion via rapid provisioning, cost inflation via failed-but-charged provisions.

#### 3. VM endpoints

Unauthorized access to auth-gated `:3001` routes, information leakage via public `:3000` routes, DoS against individual golems.

#### 4. SSH bridge

Session hijacking via ticket theft or replay, idle timeout bypass, unauthorized terminal access to another user's golem.

#### 5. Proxy layer

Cache poisoning (stale 6PN mapping routes to wrong/destroyed VM), subdomain enumeration to discover active golems, routing requests to destroyed machines.

#### 6. Custody layer

Session key compromise, delegation abuse, unauthorized sub-delegation.

---

## Top 5 threats and mitigations

### T1: Payment front-running (MEV)

**Threat**: Attacker observes a pending `receiveWithAuthorization` transaction in the mempool and front-runs it.

**Mitigation**: `receiveWithAuthorization` (EIP-3009) is inherently front-run resistant -- only the designated `to` address (Bardo treasury) can execute it. The `from`, `to`, `value`, and `nonce` are all signed by the payer.

### T2: Zombie machines (cost drain)

**Threat**: VMs continue running after TTL expiry.

**Mitigation**: Two-layer TTL enforcement limits maximum zombie duration to ~90 seconds:
- **Layer 1**: Turso poll worker (30s intervals)
- **Layer 2**: Machine-local cron (60s intervals, queries control plane)
- **Reconciliation job**: Every 5 minutes, catches machines >2 minutes past expiry

### T3: Session key compromise (fund theft)

**Threat**: Attacker extracts a session key from a compromised VM and uses it to steal funds.

**Mitigation by custody mode**:

| Mode | Exposure if key leaks | Bound |
| --- | --- | --- |
| **Delegation** | Attacker can sign UserOperations, but caveats enforce limits | DailySpendLimit, MortalityTimeWindow, GolemPhase, MaxSlippage |
| **Embedded** | Attacker has Privy API credentials, but TEE enforces policy | Privy signing policy (binary allow/deny) |
| **LocalKey** | Attacker has raw key, but on-chain delegation bounds damage | DelegationBounds: max_daily_spend, allowed_targets, expires_at |

In all modes, key compromise is **bounded**. The paradigm shift: instead of "keep the key secret," the system says "bound the damage if the key leaks."

### T4: Machine name enumeration

**Threat**: Attacker enumerates golem subdomains to discover active machines.

**Mitigation**: `nanoid(12)` with URL-safe alphabet produces 64^12 = 4.7 x 10^21 possible names. Combined with rate limiting, enumeration is computationally infeasible.

### T5: SSH session hijacking

**Threat**: Attacker intercepts or replays an SSH ticket.

**Mitigation**: Defense in depth:
- Tickets are single-use (deleted from in-memory Map on first use)
- Tickets have 30-second TTL
- SSH certificates are 5-minute validity
- WebSocket uses TLS (`wss://`)

---

## Three-mode custody security

### Delegation mode (recommended)

Funds never leave the owner's MetaMask Smart Account. The Golem holds only a disposable session key and a signed ERC-7710/7715 delegation. Every transaction executes *from* the owner's address.

**Seven custom caveat enforcers** bound what the delegation can do. Each is a deployed Solidity contract implementing `ICaveatEnforcer`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaveatEnforcer {
    /// Restricts actions by behavioral phase. Reads current phase
    /// from VitalityOracle. A dying Golem cannot open new positions.
    GolemPhase {
        vitality_oracle: Address,
        golem_address: Address,
    },

    /// Time-locked delegation. When block.timestamp > end_time,
    /// the delegation is dead. Maps to projected lifespan.
    MortalityTimeWindow {
        start_time: u64,
        end_time: u64,
    },

    /// On-chain structural atonia. Blocks writes during dream cycles.
    /// Even if a code bug fires an action during a dream, the
    /// enforcer blocks it.
    DreamMode {
        dream_oracle: Address,
        golem_address: Address,
    },

    /// Limits actions based on vault NAV percentage. Prevents
    /// a single trade from destroying the vault.
    VaultNAV {
        vault_address: Address,
        max_nav_pct: u16,
    },

    /// Caps Replicant sub-delegation spending and lifespan.
    ReplicantBudget {
        max_budget_usd: u64,
        max_lifespan_seconds: u64,
    },

    /// Bounds acceptable slippage on swap transactions.
    MaxSlippage {
        max_slippage_bps: u16,
    },

    /// Rolling 24h spending limit across all executions.
    DailySpendLimit {
        daily_limit_usd: u64,
    },
}
```

**Revocation**: One on-chain transaction disables the delegation hash in the DelegationManager. Works even if the Golem's infrastructure is offline, even if the platform is down. No cooperation from the Golem needed.

**Death settlement**: The delegation expires via MortalityTimeWindow. The owner's MetaMask Smart Account retains full control. No sweep, no race conditions, no stuck funds.

### Embedded mode (Privy)

Funds transferred to Privy server wallet in AWS Nitro Enclaves. Policy enforcement is off-chain (inside the TEE) and binary. Simpler to set up, but the owner surrenders direct custody.

| Secret | Where it lives | How Golem accesses it |
| --- | --- | --- |
| secp256k1 wallet key | Privy TEE (AWS Nitro) | Never -- Privy signs on behalf |
| P-256 session signer | Generated at provision, in-memory | Reference via `privy_config.json` |

**Death settlement**: Control plane queries balance, transfers to owner. BardoManifest records deferred positions if sweep fails.

### LocalKey mode (dev/self-hosted)

Locally generated keypair bounded by on-chain delegation. No TEE, no HSM. The key is insecure in the traditional sense. The security model: bound the damage.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationBounds {
    pub max_daily_spend_usd: f64,
    pub max_total_calls: u32,
    pub expires_at: u64,
    pub allowed_targets: Vec<Address>,
}
```

---

## Authentication

### Four auth contexts

| Context | Auth method | Endpoints |
| --- | --- | --- |
| **Public** | None | `:3000/*`, `/v1/golems`, `/v1/pricing` |
| **Owner** | Privy JWT (Bearer token) | `:3001/*` (via proxy), `/v1/machines/*` (API) |
| **Internal (VM-to-API)** | Fly OIDC token | `:3002/*`, `/internal/*` |
| **Admin** | Privy JWT with admin role | `/admin/*` |

### Fly OIDC machine authentication

Fly-issued OIDC tokens replace machine secrets for all VM-to-control-plane communication. ~10 minute validity, scoped to requesting machine. No extractable credentials on VMs.

**What OIDC eliminates**: machine secrets on VMs, secret generation at provisioning, source IP verification, the "Secret Zero" problem.

### Admin authentication

Privy JWT with admin role. Break-glass fallback via `BARDO_ADMIN_SECRET` env var (triggers extra alerting). All admin actions logged with admin identity from JWT.

---

## No plaintext secrets on VMs

| Secret | Where it lives | How Golem accesses it |
| --- | --- | --- |
| Wallet key (Delegation) | Owner's Smart Account | Session key signs UserOperations; caveats bound scope |
| Wallet key (Embedded) | Privy TEE (AWS Nitro) | Never touches VM; Privy signs on behalf |
| Wallet key (LocalKey) | Encrypted at rest on VM | Decrypted in-memory; bounded by on-chain delegation |
| SSH host certificate | Generated at boot, `/etc/ssh/` | `step-ca` signs via Fly OIDC exchange |
| Machine identity | Fly OIDC token (ephemeral, ~10 min) | Requested from Fly API at each call |
| Styx auth token | Derived from ERC-8004 identity | EIP-712 signature from session key |

---

## SSH Certificate Authority

Smallstep `step-ca` deployed on Fly. Machines get short-lived SSH certificates at boot (host certificates). Users get short-lived certificates per WebSocket session (5-minute validity). No key distribution.

---

## Grimoire import hardening

Full validation for `POST /owner/grimoire/import`:

```rust
const GRIMOIRE_IMPORT_LIMITS: ImportLimits = ImportLimits {
    max_total_size: 500 * 1024 * 1024, // 500MB
    max_file_count: 10_000,
    allowed_extensions: &[
        ".json", ".jsonl", ".lance",
        ".sqlite", ".sqlite-wal", ".sqlite-shm",
    ],
};
```

Rejects symlinks, path traversal, unexpected extensions, oversized archives. Atomic swap via temp directory.

---

## x402 payment security

| Attack | Mitigation |
| --- | --- |
| **Signature replay** | EIP-3009 nonces globally unique; Turso UNIQUE constraint rejects duplicates |
| **Front-running** | `receiveWithAuthorization` callable only by designated `to` address |
| **Double-spend** | Nonce uniqueness + `receiveWithAuthorization` atomicity |
| **Balance depletion** | Balance re-checked after health check and before settlement |
| **Signature expiry** | `validBefore` checked before settlement; minimum 300s window |
| **Overpayment** | TTL deterministically computed from amount; overpayment = longer TTL |

---

## Rate limiting matrix

In-memory token bucket per `bardo-control` instance. Optional Redis for distributed rate limiting.

| Endpoint | Per-IP | Per-user | Per-machine | Notes |
| --- | --- | --- | --- | --- |
| `POST /v1/machines` | 5/min | 5/min | -- | Provisioning |
| `POST /v1/machines/:name/extend` | 10/min | -- | 30/hr | Extension |
| `GET /v1/golems` | 30/min | -- | -- | Discovery |
| `GET /v1/machines/mine` | 30/min | 30/min | -- | User machines |
| `DELETE /v1/machines/:name` | 5/min | 5/min | -- | Destruction |
| `POST /v1/keys` | 5/min | 5/min | -- | SSH key add |
| `POST /v1/ssh/ticket` | 10/min | 10/min | -- | SSH ticket |
| Public VM endpoints | Per-route | -- | -- | Via proxy |
| Admin endpoints | 60/min | -- | -- | Privy JWT |
| Internal endpoints | -- | -- | 60/min | OIDC |

---

## Firewall rules

UFW deny all incoming by default. Allow from 6PN networks (`10.0.0.0/8` + `fdaa::/16`) on ports 22, 3000-3002. Deny database ports explicitly.

---

## Snapshot security

- **Key stripping**: Snapshots never contain wallet keys, OIDC tokens, or session signer material
- **Signed URLs**: Snapshot download URLs pre-signed with 15-minute expiry
- **Import validation**: Hardened import pipeline (symlink rejection, path traversal prevention, extension allowlist, size limits)
- **Styx backup encryption**: Grimoire backups to Styx Archive layer use the Golem's ERC-8004 identity for namespace isolation
