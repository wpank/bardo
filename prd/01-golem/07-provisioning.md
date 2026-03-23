# Infrastructure Provisioning [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Reader orientation:** This document specifies how a validated GolemExtendedManifest becomes a running Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM). It covers the 8-step type-state provisioning pipeline: validate, create wallet, approve Permit2, fund, register ERC-8004 (on-chain agent identity standard tracking capabilities, milestones, and reputation) identity, provision VM via Bardo Compute (VM hosting service for Golems; Fly.io micro VMs provisioned via x402), deploy, and start heartbeat. The pipeline is implemented as a Rust type-state machine with per-step recovery and memoized results. See `prd2/shared/glossary.md` (canonical Bardo term definitions) for full term definitions.

---

## S1 -- Overview

> **Billing Model:** Bardo Compute is a wrapper service that bills users via x402 and pays Fly.io directly. Users never interact with Fly.io billing.

The provisioning pipeline transforms a validated `GolemExtendedManifest` into a running Golem with a funded wallet (or delegation grant), configured signing policy, optional ERC-8004 identity, and active compute session. It is implemented as a Rust type-state machine where each step transitions to the next state, with per-step recovery and memoized results. If the browser crashes after step 3, resuming the pipeline skips steps 1--3 and continues from step 4.

```
GolemExtendedManifest
        |
        v
  8-Step Type-State Pipeline
        |
        v
  Running Golem (wallet + identity + policy + compute + heartbeat)
```

---

## S2 -- The 8-Step Pipeline

Eight steps, executed sequentially. Each step is a state transition in the type-state machine -- results are memoized to persistent storage, failed steps retry with exponential backoff.

| Step | Name                | Purpose                                                              | Retries |
| ---- | ------------------- | -------------------------------------------------------------------- | ------- |
| 1    | `validate_manifest` | Schema validation, network contract checks, funding calculation      | 1       |
| 2    | `create_wallet`     | Wallet provisioning per custody mode                                 | 3       |
| 3    | `approve_permit2`   | One-time USDC approval to Permit2 contract (Embedded mode only)      | 3       |
| 4    | `fund_wallet`       | Delegation grant or Permit2 SignatureTransfer                        | 3       |
| 5    | `register_identity` | ERC-8004 registration (skippable if contracts not deployed)          | 2       |
| 6    | `provision_vm`      | Fly.io VM from warm pool (hosted) or skip (self-hosted)              | 3       |
| 7    | `deploy_golem`      | Inject session signer, signing policy, strategy config into runtime  | 2       |
| 8    | `start_heartbeat`   | Health check (heartbeat response within 30s), webhook, telemetry     | 3       |

### Type-State Pipeline

The pipeline is expressed as a Rust type-state machine. Each step consumes the previous state and produces the next, making invalid step sequences a compile-time error.

```rust
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};

// Step marker types (zero-sized)
pub struct StepValidate;
pub struct StepWallet;
pub struct StepPermit2;
pub struct StepFund;
pub struct StepIdentity;
pub struct StepProvisionVm;
pub struct StepDeploy;
pub struct StepHeartbeat;
pub struct StepComplete;

/// The provisioning pipeline in a specific step.
/// Each step transition consumes `self` and produces the next step.
pub struct Pipeline<S> {
    pub session_id: String,
    pub manifest: GolemExtendedManifest,
    pub results: PipelineResults,
    _step: PhantomData<S>,
}

/// Accumulated results from completed steps. Memoized to disk.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PipelineResults {
    pub wallet: Option<WalletProvisionResult>,
    pub permit2_approved: Option<bool>,
    pub funding: Option<FundingResult>,
    pub identity: Option<IdentityResult>,
    pub vm: Option<VmProvisionResult>,
    pub deploy: Option<DeployResult>,
}

impl Pipeline<StepValidate> {
    /// Entry point. Validates the manifest and produces a pipeline
    /// ready for wallet creation.
    pub async fn validate(
        manifest: GolemExtendedManifest,
        session_id: String,
    ) -> Result<Pipeline<StepWallet>> {
        validate_manifest(&manifest).await?;
        Ok(Pipeline {
            session_id,
            manifest,
            results: PipelineResults::default(),
            _step: PhantomData,
        })
    }
}

impl Pipeline<StepWallet> {
    /// Create the wallet per the selected custody mode.
    pub async fn create_wallet(self) -> Result<Pipeline<StepPermit2>> {
        let result = match &self.manifest.custody {
            Some(CustodyModeConfig::Delegation { .. }) => {
                provision_delegation_wallet(&self.manifest).await?
            }
            Some(CustodyModeConfig::LocalKey { .. }) => {
                provision_local_key_wallet(&self.manifest).await?
            }
            _ => {
                // Default: Embedded (Privy) for backward compatibility
                provision_privy_wallet(&self.manifest).await?
            }
        };
        let mut results = self.results;
        results.wallet = Some(result);
        Ok(Pipeline { results, session_id: self.session_id, manifest: self.manifest, _step: PhantomData })
    }
}

// Each subsequent step follows the same pattern:
// consume self, execute step, produce next state.
```

### 2.1 Step 1: Validate Manifest

Schema validation, network contract availability checks, and funding recommendation computation. Deterministic -- 1 retry for transient network issues only. Non-retryable failures return the user to the wizard with highlighted field errors.

### 2.2 Step 2: Create Wallet

Wallet creation varies by custody mode:

**Delegation mode (recommended):** No server-side wallet creation needed. The Golem generates an ephemeral session keypair (secp256k1). The owner's MetaMask Smart Account address is recorded. No Privy dependency.

**Embedded mode (Privy):** Privy server wallets are standard EOAs whose secp256k1 private keys are generated inside and never leave a Privy AWS Nitro Enclave (TEE). The wallet address is only known after calling the Privy creation API. The pipeline creates the wallet first, then uses the returned address for all subsequent on-chain operations.

**Local Key mode:** A secp256k1 keypair is generated locally, bounded by an on-chain delegation. The private key is stored at the configured path (encrypted at rest). A delegation is signed granting this key bounded spending authority.

```rust
/// Result of wallet provisioning. Structure varies by custody mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletProvisionResult {
    Delegation {
        /// The owner's MetaMask Smart Account address.
        owner_smart_account: Address,
        /// Ephemeral session key address (Golem-side).
        session_key_address: Address,
        /// Chain the delegation targets.
        chain_id: u64,
    },
    Embedded {
        /// Privy wallet identifier (e.g., "wl_abc123"). Used as dedup key.
        wallet_id: String,
        /// EOA address derived from the TEE-generated secp256k1 key.
        address: Address,
        /// Chain the wallet is configured for.
        chain_id: u64,
        /// P-256 session signer public key (compressed, hex-encoded).
        session_signer_pubkey: String,
        /// When the session signer expires (Unix seconds).
        session_signer_expires_at: u64,
    },
    LocalKey {
        /// Address derived from the generated keypair.
        address: Address,
        /// Chain the key targets.
        chain_id: u64,
    },
}
```

### 2.3 Step 3: Approve Permit2

Only applies to Embedded (Privy) custody mode. Check whether the user has already approved Permit2 to spend USDC. If not, request a one-time `USDC.approve(PERMIT2, type(uint256).max)` transaction. In Delegation mode, this step is skipped -- funds never leave the owner's wallet. In Local Key mode with delegation, this step is also skipped.

### 2.4 Step 4: Fund Wallet

Funding varies by custody mode:

**Delegation mode:** The owner signs an ERC-7715 `wallet_grantPermissions` request, granting the session key address spending authority subject to caveat enforcers. No token transfer. One off-chain signature.

**Embedded mode:** Execute a Permit2 `SignatureTransfer` to move USDC from the owner's Main Wallet to the Golem wallet. Idempotent via Permit2 nonces -- if the transfer was already executed (pipeline retry after completion), the on-chain call reverts with `InvalidNonce` and the existing transfer receipt is returned instead.

**Local Key mode:** The owner signs a delegation granting the local key bounded spending authority. Optionally funds a separate on-chain account if the delegation requires a gas deposit.

### 2.5 Step 5: Register Identity

When ERC-8004 contracts are available on the target network, register the Golem's identity. Metadata is generated fresh at registration time -- not cached from wizard editing minutes earlier. Keyword matching identifies DeFi domains, protocols, and capabilities from the strategy text.

If ERC-8004 is not available (e.g., not yet deployed on Base mainnet), the step is skipped. Deferred registration uses runtime-observed behavior (actual tool usage, contract interactions) instead of manifest-derived inference when it eventually executes.

### 2.6 Step 6: Provision VM

For hosted Golems, provision a Fly.io VM from a warm pool of pre-created stopped machines. The VM runs the Rust Golem binary (not a Node.js runtime):

| Path                                       | Time    |
| ------------------------------------------ | ------- |
| Warm pool (pre-allocated stopped machines) | 3--8s   |
| Cold fallback (create from scratch)        | 15--30s |

The warm pool maintains 5 stopped machines per region (ams, ord), replenished every 5 minutes. Cost: ~$0.77/month for 10 stopped machines. At provision time: query pool for a stopped machine, update config (env + files), start machine. If the pool is empty, fall back to cold creation.

The VM image contains the statically-linked `golem-binary` Rust executable. Configuration is injected via Fly's native env var and file injection at machine creation time. The VM receives `golem.toml`, STRATEGY.md, custody config, and the signing policy. No SSH config injection, no wait loops. Config is present from first boot.

For self-hosted Golems, this step is skipped. The owner runs the Golem binary directly on their own infrastructure.

### 2.7 Step 7: Deploy Golem

Inject configuration into the runtime environment:

- Session key material (Fly.io per-machine env var for hosted; returned to owner for self-hosted)
- `golem.toml` configuration
- STRATEGY.md content
- Custody config (mode + parameters -- no private keys for Delegation/LocalKey; Privy config for Embedded)
- Signing policy (compiled from transfer restriction tier + caveat enforcers)

The session key is an ephemeral keypair granting the Golem the ability to sign transactions through its delegation (Delegation mode) or Privy-managed wallet (Embedded mode). Time-bounded (default: 1 week), wallet-scoped, and policy-constrained. The private key reaches the Golem's runtime without being written to disk.

### 2.8 Step 8: Start Heartbeat

The Golem binary boots, opens a persistent outbound WebSocket to Styx (`wss://styx.bardo.run/v1/styx/ws`), and fires its first heartbeat tick. The provisioner waits for a heartbeat response within 30 seconds. On success, emits `golem.created` webhook and `golem_created` telemetry event.

The Styx connection is outbound-only -- no inbound ports, no tunnels, no port forwarding. A Golem behind double-NAT works identically to one on a public IP. Styx verifies ERC-8004 registration during WebSocket authentication; an unregistered Golem is rejected.

For self-hosted Golems, this step performs a local process health check or is skipped if the owner starts manually. The Golem still connects to Styx on its own at boot.

---

## S3 -- Idempotency

### 3.1 Session Identity

Each provisioning attempt gets a deterministic session ID:

```rust
use alloy::primitives::keccak256;

pub fn compute_session_id(
    manifest: &GolemExtendedManifest,
    user_address: Address,
) -> String {
    let canonicalized = serde_json::to_string(manifest).expect("manifest serializable");
    let input = format!("{}{}", canonicalized, user_address);
    let hash = keccak256(input.as_bytes());
    format!("{:x}", hash)
}
```

Two attempts with identical manifests from the same wallet produce the same session ID, enabling resume-from-checkpoint. If the manifest changes (user edits strategy between attempts), a new session ID is generated and the old session is abandoned.

### 3.2 Resume Semantics

On resume (browser refresh, retry after failure, network recovery):

1. Client sends `session_id` to the server
2. Server reads memoized `PipelineResults` from disk
3. Reconstructs the pipeline at the appropriate type-state step
4. Re-executes from the first incomplete step
5. Sessions expire after 1 hour
6. If the attached manifest differs from the stored manifest, a new session is created

### 3.3 Per-Step Idempotency

| Step                 | Idempotent | Mechanism                                                    |
| -------------------- | ---------- | ------------------------------------------------------------ |
| 1. Validate          | Yes        | Pure function (no side effects)                              |
| 2. Create Wallet     | Yes        | Dedup by session ID (Delegation); Privy dedup by wallet ID (Embedded) |
| 3. Approve Permit2   | Yes        | Allowance check before approval                              |
| 4. Fund Wallet       | Yes        | Delegation signature is idempotent; Permit2 nonces are single-use |
| 5. Register Identity | Yes        | Registry lookup before attempt                               |
| 6. Provision VM      | Yes        | Fly.io machine ID as dedup key                               |
| 7. Deploy Golem      | No\*       | Pipeline-level dedup via memoized step result                |
| 8. Start Heartbeat   | Yes        | Read-only health check + idempotent webhook                  |

\*Step 7 (session key registration) calls register multiple signers if called multiple times. The pipeline tracks the signer ID in the memoized step result to prevent duplicates.

---

## S4 -- Step Compensation on Failure

The pipeline is forward-only. On-chain operations (steps 4, 5) cannot be rolled back. The design compensates for failures rather than attempting undo.

### 4.1 Per-Step Recovery

| Step | Failure Mode                 | Recovery                              | Retryable |
| ---- | ---------------------------- | ------------------------------------- | --------- |
| 1    | Invalid manifest             | Show errors, return to wizard         | No        |
| 2    | Wallet provisioning failed   | Retry with backoff (3 attempts)       | Yes       |
| 3    | Approval rejected by user    | Return to funding step                | No        |
| 4    | Insufficient balance         | Show insufficient funds flow          | No        |
| 4    | Permit2 signature expired    | Request new signature                 | No        |
| 4    | Delegation signature failed  | Request new signature                 | No        |
| 4    | Transaction reverted         | Retry with fresh nonce                | Yes       |
| 4    | Already funded (nonce reuse) | Detect `InvalidNonce`, skip           | Yes       |
| 5    | ERC-8004 not deployed        | Skip step (deferred registration)     | N/A       |
| 5    | Agent already registered     | Detect, skip                          | Yes       |
| 6    | Fly.io capacity unavailable  | Retry; offer alternative region       | Yes       |
| 6    | VM boot timeout (>60s)       | Retry; after 3 failures, manual debug | Yes       |
| 7    | Policy rejected              | Surface to user                       | No        |
| 7    | Signer revoked before deploy | Re-generate signer, retry             | Yes       |
| 8    | Health check timeout         | Retry 3x at 10s intervals             | Yes       |

### 4.2 Fund Compensation

If the pipeline fails after step 4 (wallet is funded but subsequent steps fail permanently):

**Delegation mode:** No compensation needed. The delegation can be revoked by the owner from MetaMask. Funds were never transferred.

**Embedded mode:** Funds are swept back to the owner's Main Wallet. If step 7 completed (session signer available), the sweep uses the session signer. Otherwise, the owner sweeps manually via the app dashboard. Key invariant: **the owner never loses funds**.

---

## S5 -- Hosted vs Self-Hosted Differences

| Step                 | Hosted (Bardo Compute) | Self-Hosted                                       |
| -------------------- | ---------------------- | ------------------------------------------------- |
| 1. Validate          | Full validation        | Same                                              |
| 2. Create Wallet     | Per custody mode       | Same                                              |
| 3. Approve Permit2   | If Embedded mode       | Same, or skip if Delegation/LocalKey              |
| 4. Fund Wallet       | Per custody mode       | Same                                              |
| 5. Register Identity | On-chain ERC-8004      | Same                                              |
| 6. Provision VM      | Fly.io from warm pool  | **Skip**                                          |
| 7. Deploy Golem      | Inject config into VM  | Return config to owner                            |
| 8. Start Heartbeat   | Remote health check    | **Skip** (owner runs `golem-binary` directly)     |

---

## S6 -- Warm Pool for Sub-5s Provisioning

The warm pool eliminates cold start latency for hosted Golems. Pre-created stopped Fly.io machines sit ready in each region, awaiting configuration injection and start.

| Parameter              | Value                                         |
| ---------------------- | --------------------------------------------- |
| Pool size per region   | 5 stopped machines                            |
| Regions                | ams (Amsterdam), ord (Chicago)                |
| Replenishment interval | Every 5 minutes                               |
| Machine specs          | shared CPU, 1 vCPU, 512MB RAM                 |
| Monthly cost           | ~$0.77 (10 machines x 512MB x $0.15/GB/month) |
| Warm boot time         | 3--8 seconds (config update + start)          |
| Cold fallback time     | 15--30 seconds (create from scratch)          |

At provision time:

1. Query pool for a stopped machine in the target region
2. If available: update machine config (env + strategy files), start machine
3. If pool empty: create new machine from scratch (cold fallback)

Configuration is injected via Fly's native env var and file injection at machine creation time. The VM receives `golem.toml`, STRATEGY.md, custody config JSON (no private keys in Delegation mode), and the signing policy. No SSH config injection, no wait loops. Config is present from first boot.

---

## S7 -- Session Key Transmission

The session key material must reach the Golem's runtime without being written to disk or exposed to intermediate infrastructure.

### 7.1 Hosted Mode

The key is injected as a Fly.io per-machine env var (not app-wide secrets). Per-machine env vars ensure one Golem cannot read another's key. Fly.io security properties:

- In-transit: TLS 1.3 between provisioner and Fly API
- At-rest: Fly infrastructure encrypts machine config
- Process isolation: Each Fly machine runs in its own Firecracker microVM
- No disk persistence: Env vars live in process memory only
- No API readback: Machine env vars are write-only via the Machines API
- Golem isolation: Per-machine env vars ensure cross-Golem key isolation

In Delegation mode, the session key private material is ephemeral -- compromise is bounded by the caveat enforcers on the delegation, and the owner can revoke from MetaMask at any time.

### 7.2 Self-Hosted Mode

The private key is returned to the owner in the provisioning result. The owner injects it into their runtime:

```bash
export GOLEM_SESSION_KEY="<key from provisioning>"
./golem-binary --config ./golem.toml
```

The key is read from `process.env` and held in memory for the process lifetime. Never written to disk by Bardo code. In Delegation mode, key compromise is bounded by caveats. In Embedded mode, the key is policy-constrained within the TEE.

---

## S8 -- Fresh Metadata Generation

Identity metadata for ERC-8004 registration is always generated fresh from the current state at the moment of registration -- never cached from wizard editing.

**At creation time**: Metadata is derived from the finalized manifest. Keyword matching on the strategy text identifies DeFi domains, protocols, and capabilities.

**At deferred registration**: When the Golem migrates to a network with ERC-8004, metadata is derived from runtime-observed behavior (actual tool usage, contract interactions, chain activity) -- more accurate because it reflects what the Golem actually does, not what the strategy text says it should do.

---

## S9 -- Graceful Shutdown

> **Graceful Shutdown:** Full 10-phase shutdown sequence in `12-teardown.md` Section S11 and `rewrite4/01b-runtime-infrastructure.md` Section 6. Summary: stop new work > cancel in-flight tools > flush Grimoire > settlement triage > execute critical settlements > write BardoManifest > seal audit chain > sync to Styx > zero secrets > exit.

---

## S10 -- Telemetry

Every step completion and failure emits a telemetry event to PostHog for creation funnel tracking. Events use privacy-preserving identifiers (server-generated HMAC of wallet address, never the raw address).

Key funnel metrics:

- **Completion rate**: `provisioning_started` to `golem_created`
- **Step drop-off**: which step has the highest failure/abandonment rate
- **Step duration P50/P95**: identify slow steps for optimization
- **Retry rate per step**: identify flaky infrastructure
- **Mode split**: Bardo Compute vs self-hosted adoption
- **Custody split**: Delegation vs Embedded vs LocalKey adoption
- **Network distribution**: which networks users target

---

## S11 -- Client-Side Session Tracking

The wizard UI tracks provisioning progress via polling (2-second intervals) or server-sent events (SSE). Each step displays a status message:

| Step | In Progress                   | Completed                   |
| ---- | ----------------------------- | --------------------------- |
| 1    | "Validating configuration..." | "Configuration valid"       |
| 2    | "Setting up wallet..."        | "Wallet ready: 0xab...cd"  |
| 3    | "Checking token approval..."  | "Permit2 approved"          |
| 4    | "Granting delegation..."      | "Delegation active"         |
| 5    | "Registering identity..."     | "Identity registered"       |
| 6    | "Booting compute..."          | "Compute instance running"  |
| 7    | "Deploying Golem..."          | "Golem deployed"            |
| 8    | "Verifying heartbeat..."      | "Your Golem is live!"       |

Step 4 message adapts to custody mode: "Granting delegation..." (Delegation), "Funding Golem..." (Embedded), "Signing delegation..." (LocalKey).

Available recovery actions include retry (for retryable failures), cancel, add funds (for insufficient balance in Embedded mode), change region (for Fly.io capacity issues), and manual sweep (for fund recovery after permanent failure in Embedded mode).

---

## Events Emitted

Provisioning events track each pipeline step for funnel analysis and debugging.

| Event | Trigger | Payload |
|---|---|---|
| `provisioning:started` | Pipeline begins | `{ sessionId, custodyMode, network, mode }` |
| `provisioning:step_completed` | Any step succeeds | `{ sessionId, step, durationMs }` |
| `provisioning:step_failed` | Any step fails | `{ sessionId, step, error, retryable }` |
| `provisioning:wallet_created` | Step 2 completes | `{ walletAddress, custodyMode, chainId }` |
| `provisioning:funded` | Step 4 completes | `{ amount, custodyMode }` |
| `provisioning:identity_registered` | Step 5 completes | `{ identityId, txHash }` |
| `provisioning:styx_connected` | Golem connects to Styx | `{ golemId, styxEndpoint }` |
| `provisioning:first_heartbeat` | Step 8 completes | `{ golemId, bootDurationMs }` |
