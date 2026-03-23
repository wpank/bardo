# 02 -- VM provisioning and lifecycle [SPEC]

**Warm pool, allocation strategy, TTL enforcement, and machine lifecycle**

> **Reader orientation:** This document specifies how Golem (mortal autonomous DeFi agent) VMs are provisioned, configured, and destroyed on Bardo Compute. It belongs to the **Compute** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: VMs are pre-warmed in a stopped pool for sub-5-second provisioning, configured via Fly's native file injection (no SSH bootstrapping), and destroyed deterministically when USDC credit expires. The Golem binary boots in ~2 seconds and contains the full cognitive architecture. Terms like Grimoire, Daimon, Thanatopsis, and x402 are defined inline on first use; a full glossary lives in `00-overview.md § Terminology`.

---

## Golem binary deployment

The Golem VM runs a single Rust binary compiled from the `bardo-golem-rs` Cargo workspace. The binary includes the full cognitive architecture: Heartbeat (9-step decision cycle) pipeline, Grimoire (persistent knowledge store), Daimon (affect/emotion engine), mortality engine, dream scheduler, custody manager, and the `bardo-tools` DeFi toolkit. Configuration is injected via Fly's native env var and file injection at machine creation time -- no SSH bootstrapping required.

The Docker image wraps the binary with lifecycle scripts, health servers, and the SSH daemon for terminal access:

```dockerfile
FROM debian:bookworm-slim

# System packages
RUN apt-get update && apt-get install -y \
    openssh-server ufw curl cron jq logrotate step-cli \
    && rm -rf /var/lib/apt/lists/*

# Golem binary (statically compiled Rust, ~30MB)
COPY --from=golem-build /opt/golem/golem-binary /opt/bardo/golem-binary

# Configuration template
COPY golem-default.toml /opt/bardo/golem-default.toml

# Lifecycle scripts
COPY scripts/ /opt/bardo/scripts/
RUN chmod +x /opt/bardo/scripts/*.sh

# Health server (lightweight Rust binary, ~2MB)
COPY health-server /opt/bardo/health-server

# TTL failsafe cron
COPY bardo-ttl-check /etc/cron.d/bardo-ttl-check
RUN chmod 0644 /etc/cron.d/bardo-ttl-check && crontab /etc/cron.d/bardo-ttl-check

# Log rotation config
COPY bardo-logrotate /etc/logrotate.d/bardo
RUN chmod 0644 /etc/logrotate.d/bardo

# SSH config (key-only, no password)
RUN mkdir -p /run/sshd && \
    sed -i 's/#PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config && \
    sed -i 's/#PubkeyAuthentication yes/PubkeyAuthentication yes/' /etc/ssh/sshd_config

# Config directory (populated via Fly file injection at machine creation)
RUN mkdir -p /etc/bardo/strategy /etc/bardo/wallet

# Grimoire data directory
RUN mkdir -p /var/lib/bardo/grimoire

EXPOSE 22 3000 3001 3002

ENTRYPOINT ["/opt/bardo/scripts/entrypoint.sh"]
```

No Node.js. No V8 runtime. The Golem binary starts in ~100ms and uses ~20MB resident memory at idle -- a fraction of the TypeScript-era stack.

---

## Entrypoint script

```bash
#!/bin/bash
# /opt/bardo/scripts/entrypoint.sh
set -euo pipefail

# 1. Request SSH host certificate from CA
/opt/bardo/scripts/setup-ssh-cert.sh

# 2. Start SSH daemon (for user terminal access only)
/usr/sbin/sshd -D &

# 3. Start cron (TTL failsafe)
cron

# 4. Apply firewall rules
/opt/bardo/scripts/ufw-rules.sh

# 5. Start health server (immediately reports 'booting')
/opt/bardo/health-server --port 3002 &

# 6. Initialize and start Golem (via process supervisor)
exec /opt/bardo/scripts/supervisor.sh
```

The Golem binary handles its own public (`:3000`) and auth-gated (`:3001`) endpoints internally. No separate endpoint server processes.

---

## Configuration via Fly API

At machine creation time, the control plane passes config via Fly's native env var and file injection:

```rust
/// Configuration injected into each Golem VM at provisioning time.
/// The control plane constructs this and passes it via Fly Machine API.
pub struct MachineProvisionConfig {
    pub machine_id: String,
    pub machine_name: String,   // golem-{nanoid(12)}
    pub user_id: String,
    pub vm_size: VmTier,
    pub expires_at: u64,        // epoch seconds
    pub tool_profile: ToolProfile,
    pub custody_mode: CustodyMode,
    pub styx_url: String,       // "wss://styx.bardo.run/v1/styx/ws"
    pub inference_url: String,  // "https://inference.bardo.money"
    pub default_model: String,  // "claude-haiku-4-5"
    pub escalation_model: String, // "claude-sonnet-4"
    pub critical_model: String, // "claude-opus-4-6"
    pub grimoire_path: PathBuf, // "/var/lib/bardo/grimoire"
}
```

These values are written to `golem.toml` and injected as Fly file:

```toml
# /etc/bardo/golem.toml (generated at provisioning time)
golem_id = "golem-V1StGXR8_Z5j"
owner_address = "0x..."
styx_url = "wss://styx.bardo.run/v1/styx/ws"
heartbeat_interval_secs = 15
tool_profile = "full"
vm_tier = "small"

[custody]
mode = "delegation"
smart_account = "0x..."

[inference]
default_model = "claude-haiku-4-5"
escalation_model = "claude-sonnet-4"
critical_model = "claude-opus-4-6"
gateway_url = "https://inference.bardo.money"
```

### Three-mode wallet provisioning

The wallet setup varies by custody mode:

| Mode | What's injected | On-chain setup |
| --- | --- | --- |
| **Delegation** | Session key address, delegation hash, caveat enforcer addresses | Owner has already signed ERC-7715 grant; Golem verifies delegation on-chain |
| **Embedded** | Privy `appId`, `walletId`, `sessionSignerKeyId`, `chainId` | Privy server wallet already created; secp256k1 key never touches VM |
| **LocalKey** | Encrypted keypair path, delegation bounds | Key generated during `bardo init`; delegation signed before deployment |

In Delegation mode, `wallet_config.json` contains only the session key address and the delegation hash. No private keys, no Privy credentials. The Golem verifies its delegation is still active on-chain at boot and on every `should_rotate()` check.

### Styx connection setup

During provisioning, the control plane:

1. Registers ERC-8004 (on-chain agent identity standard) identity on Base L2 (gas from Golem wallet)
2. Injects the Styx (global knowledge relay at `wss://styx.bardo.run`) URL into `golem.toml`
3. The Golem binary connects to Styx within its first heartbeat tick
4. Styx verifies the ERC-8004 registration during WebSocket authentication

If the Golem cannot reach Styx, it continues operating on its local Grimoire at ~95% capability. The `bardo-styx` extension retries with exponential backoff.

---

## Warm machine pool

Pre-created stopped Fly machines for sub-5s provisioning. Pool manager runs every 5 minutes, maintaining 5 stopped machines per region.

### Warm Pool Mechanics

The warm pool maintains pre-warmed stopped machines in each active region. These machines have the Docker image already pulled and the base filesystem ready. At provision time, the control plane only needs to inject per-Golem configuration and start the machine, which takes ~300ms compared to 15-30s for a cold create.

Pool replenishment runs as a background Tokio task on `bardo-control`. Every 5 minutes, it checks each region's pool count and creates replacement machines for any that were claimed. The pool target is 5 machines per region at v1 scale. At higher scale, the target adjusts based on provisioning rate (1-hour rolling average of deploys per region).

### Tier System

Four tiers, each mapping to a specific Fly.io machine configuration:

| Tier | CPU | RAM | Inference Allowance/hr | Typical Use |
| --- | --- | --- | --- | --- |
| `micro` | 1 shared | 256MB | 50K tokens (Haiku only) | Price monitors, keeper bots |
| `small` | 1 dedicated | 512MB | 100K tokens (Haiku + Sonnet) | Standard Golem (default) |
| `medium` | 2 CPU | 1GB | 200K tokens (Haiku + Sonnet + limited Opus) | Multi-chain, Grimoire-heavy |
| `large` | 4 CPU | 2GB | 500K tokens (all models, priority) | Full observatory + trading |

The small tier is the default. A context-engineered Golem burning $0.40/day in total costs (compute + inference + gas) lives 2,500 days on $1,000 at this tier.

### Claim flow

At provision time:

1. Query warm pool for stopped machines in target region matching requested tier
2. If available: update machine config (env + files) then start machine (~300ms)
3. If pool empty: create new machine (15-30s cold fallback)

### Cost analysis

Stopped machines cost ~$0.15/GB/month for rootfs storage. 5 `small` machines x 2 regions = 10 machines x 512MB = 5.12GB. Monthly cost: ~$0.77. Negligible.

### Provisioning time comparison

| Path | Time | When |
| --- | --- | --- |
| Warm pool | 3-8s | Normal (pool has machines) |
| Warm pool + Styx setup | 5-8s | Including ERC-8004 registration and Styx handshake |
| Cold fallback | 15-30s | Pool exhausted or non-default region |
| Cold + first-time ERC-8004 | 20-35s | New machine + on-chain identity registration |

---

## Process supervisor

```bash
#!/bin/bash
# /opt/bardo/scripts/supervisor.sh
MAX_RESTARTS=5
RESTART_WINDOW=3600  # 1 hour
RESTART_COUNT=0
WINDOW_START=$(date +%s)

while true; do
  NOW=$(date +%s)
  ELAPSED=$((NOW - WINDOW_START))

  if [ $ELAPSED -gt $RESTART_WINDOW ]; then
    RESTART_COUNT=0
    WINDOW_START=$NOW
  fi

  /opt/bardo/golem-binary --config /etc/bardo/golem.toml &
  GOLEM_PID=$!
  wait $GOLEM_PID
  EXIT_CODE=$?

  if [ $EXIT_CODE -eq 0 ]; then
    break  # Clean exit (graceful shutdown / Thanatopsis)
  fi

  RESTART_COUNT=$((RESTART_COUNT + 1))
  echo "$(date -u +%FT%TZ) Golem crashed (exit=$EXIT_CODE), restart $RESTART_COUNT/$MAX_RESTARTS" \
    >> /var/log/bardo/supervisor.log

  # Report crash to control plane via Fly OIDC
  OIDC_TOKEN=$(curl -sf http://localhost/.fly/api/v1/tokens/oidc \
    -d '{"aud":"bardo-control"}' | jq -r '.token' 2>/dev/null || echo "")
  if [ -n "$OIDC_TOKEN" ]; then
    curl -sf -X POST "http://bardo-control.internal:3000/internal/machines/self/crash" \
      -H "Authorization: Bearer $OIDC_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"exit_code\":$EXIT_CODE,\"restart_count\":$RESTART_COUNT}" || true
  fi

  if [ $RESTART_COUNT -ge $MAX_RESTARTS ]; then
    echo "$(date -u +%FT%TZ) Max restarts exceeded, entering crashed state" \
      >> /var/log/bardo/supervisor.log
    sleep infinity
  fi

  sleep 2
done
```

Max 5 restarts per hour. On max exceeded, machine enters `crashed` state. Control plane can force-destroy or admin can investigate.

---

## Golem startup sequence

```
1. SSH CA certificate request                     ~1-2s
2. SSH daemon starts                               ~instant
3. Cron starts                                     ~instant
4. Firewall rules applied                          ~500ms
5. Health server starts (reports 'booting')          ~100ms
6. Golem binary starts:
   a. Parse golem.toml                             ~instant
   b. Initialize Grimoire (LanceDB + SQLite)       ~1-3s
      +-- Or restore from Styx backup if available
   c. Load tool profile, register bardo-tools      ~500ms
   d. Initialize custody (verify delegation/wallet) ~1-2s
   e. Connect to Styx (outbound WebSocket)          ~500ms-2s
   f. Register ERC-8004 identity (if not yet done) ~2-5s
   g. Start heartbeat pipeline (first tick)         ~instant
7. Health server reports 'ready'                    ~instant
```

**Total boot time (warm pool)**: 3-8 seconds from machine start to first heartbeat.
**Total boot time (cold)**: 15-30 seconds (image pull + boot).

The Rust binary's startup is dramatically faster than the TypeScript-era stack. No V8 warmup, no `npm` dependency resolution, no JIT compilation. The Golem is cognitively active within 2 seconds of binary start.

---

## Machine lifecycle states

```
payment_verified --> provisioning --> booting --> ready --> draining --> destroyed
                         |                         |         |
                         | (Fly create fails)      |         |
                         v                         |         v
                     destroyed                     |      destroyed
                  (provision_failed)               |
                                                   |
                                             crashed (via supervisor)
                                             (admin can force-destroy)
```

### State transitions

| From | To | Trigger | DB action |
| --- | --- | --- | --- |
| (new) | `provisioning` | x402 payment verified | INSERT (before Fly API call) |
| `provisioning` | `booting` | Fly machine started | UPDATE status |
| `booting` | `ready` | Health check passes | UPDATE status |
| `provisioning` | `destroyed` | Fly creation fails | UPDATE status, reason='provision_failed' |
| `booting` | `destroyed` | Health check timeout (120s) | UPDATE status, reason='boot_timeout' |
| `ready` | `ready` | Extension payment | UPDATE expiresAt (CAS) |
| `ready` | `draining` | TTL expired / user DELETE / admin | UPDATE status |
| `ready` | `crashed` | Supervisor max restarts | UPDATE status (reported by VM) |
| `draining` | `destroyed` | Graceful shutdown complete | UPDATE status, destroyedAt |
| `crashed` | `destroyed` | Admin force-destroy | UPDATE status |

---

## Graceful shutdown (Thanatopsis integration)

When TTL expires, the Golem runs Thanatopsis -- the four-phase death protocol. The VM has a 30-second budget for phases I-III. Phase IV (Legacy) runs asynchronously on the control plane.

```
On VM (30s budget):
1. Signal SIGTERM to Golem process                          ~instant
2. Golem receives shutdown, enters Terminal phase            ~instant
3. Phase I: Acceptance
   - Close all open positions                               ~5-10s
   - Cancel pending orders                                  ~1-2s
4. Phase II: Settlement
   - Flush Grimoire to persistent storage                   ~3-5s
   - Upload death testament to Styx (Vault layer)           ~2-5s
5. Phase III: Reflection
   - Generate death reflection (T1 inference, frugal)       ~3-5s
   - Deposit bloodstain warnings to Styx                    ~1-2s
6. Health server reports 'draining'                          ~instant
7. Endpoint servers stop accepting new requests              ~instant
8. Wait for in-flight requests (10s max)                     ~0-10s
9. Fly machine stopped                                       ~instant

On control plane (after VM drain, no time pressure):
10. Phase IV: Legacy
    - Sweep wallet (Embedded mode only; Delegation expires)  variable
    - Record death event, final stats                        ~instant
    - Trigger successor provisioning if configured            variable
    - Update DB: status='destroyed', destroyedAt             ~instant
```

In Delegation mode, there is no wallet sweep -- the delegation expires, and the owner retains full control of their funds. The "no sweep" death eliminates stuck funds, failed sweeps, and gas estimation errors during teardown.

---

## Golem creation flow

Whether triggered by TUI, web dashboard, or API call, the creation flow is identical:

1. Owner configures strategy, custody mode, VM tier
2. Owner submits x402 payment
3. Control plane verifies payment, writes intent to Turso
4. Claim warm pool machine or create cold
5. Inject `golem.toml`, `STRATEGY.md`, wallet config
6. Start machine, poll health check
7. Golem binary boots, connects to Styx, registers ERC-8004
8. Health check passes -> status = `ready`
9. Return tracking link and TUI connection command

---

## Region selection

7 regions with capacity checked via direct Turso query. Per-region cap: 100. Global cap: 500.

---

## Machine limit

5 active machines per user. Enforced at provisioning endpoint before payment verification.

---

## Wallet lifecycle by custody mode

### Delegation mode

1. **At provision**: Session key generated in-memory. Delegation hash verified on-chain.
2. **During operation**: Session key signs UserOperations against the owner's Smart Account. Caveat enforcers validate each operation.
3. **At key rotation**: New session key, fresh delegation grant from owner.
4. **At death**: Delegation expires (MortalityTimeWindow). No sweep needed.

### Embedded mode (Privy)

1. **At provision**: Privy API creates server wallet.
2. **During operation**: Golem signs via Privy API.
3. **At death**: Control plane queries balance, sweeps to owner. BardoManifest records deferred positions.
4. **If sweep fails**: Reconciliation job retries; admin can trigger manually.

### LocalKey mode

1. **At provision**: Keypair generated by `bardo init`, encrypted at rest.
2. **During operation**: Local signing, bounded by on-chain delegation.
3. **At death**: Delegation expires. Key material zeroized from memory.

---

## Periodic Grimoire sync

Every 6 hours via cron. Uses OIDC auth. Grimoire snapshots stored on Cloudflare R2. Latest 5 snapshots retained per machine. Restore on boot from latest snapshot.

Grimoire also syncs to Styx Archive layer continuously via the outbound WebSocket. The periodic R2 snapshot is a belt-and-suspenders backup for the case where both Styx and the VM die simultaneously.

---

## Log rotation

```
/var/log/bardo/*.log {
    daily
    rotate 7
    maxsize 100M
    compress
    missingok
    notifempty
    copytruncate
    su root root
}
```

Total log volume cap: 700MB (7 days x 100MB).
