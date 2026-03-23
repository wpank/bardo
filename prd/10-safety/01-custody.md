# Custody: Keys, Delegations, and the Economics of Autonomy [SPEC]

> **Crate**: `golem-custody`
>
> **Depends on**: [00-defense.md](00-defense.md) (defense layers, Capability model), [03-policy.md](03-policy.md) (PolicyCage)
>
> **Prerequisites**: `02-mortality/00-overview.md` (three clocks, behavioral phases), `01-golem/13-runtime-extensions.md` (Event Fabric, GolemEvent)

> **Reader orientation:** This document specifies how Golems (mortal autonomous DeFi agents) hold and use cryptographic keys for on-chain transactions. It belongs to the **Safety** layer of Bardo (the Rust runtime governing these agents). The key concept before diving in: Bardo offers three custody architectures with fundamentally different trust models, and the recommended mode (Delegation) keeps funds in the owner's wallet at all times, never transferring keys to the agent. Terms like PolicyCage, Grimoire, and Styx are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

The safest wallet is the one the agent never holds.

A Golem manages real capital on behalf of an owner. The custody architecture determines who holds the keys, what the keys can sign, and what happens to the funds when the Golem dies. These are not implementation details. They are the foundation on which every other safety guarantee rests.

---

## 1. Three Custody Modes

The system supports three custody modes. They are not graduated tiers -- they are fundamentally different trust models. An owner selects one at provisioning time.

```rust
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The owner's choice of key management architecture.
/// Selected once during onboarding. Cannot change after the first heartbeat tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyMode {
    /// Recommended. Funds stay in the owner's MetaMask Smart Account.
    /// ERC-7710/7715 delegation framework. Session keys with on-chain
    /// caveat enforcers. Key compromise is bounded by caveats, not
    /// by secrecy.
    Delegation {
        smart_account: Address,
        caveat_enforcers: Vec<CaveatEnforcer>,
    },

    /// Legacy. Funds transferred to Privy server wallet in AWS Nitro
    /// Enclaves. Simpler but custodial. Requires sweep at death.
    Embedded {
        privy_app_id: String,
        server_wallet_id: String,
    },

    /// Dev/self-hosted. Locally generated keys bounded by on-chain
    /// delegation. For testing and self-hosted deployments.
    LocalKey {
        private_key_path: PathBuf,
        delegation_bounds: DelegationBounds,
    },
}

/// Hard limits for the LocalKey mode. The delegation is the security
/// boundary, not the key's secrecy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationBounds {
    pub max_daily_spend_usd: f64,
    pub max_total_calls: u32,
    pub expires_at: u64,
    pub allowed_targets: Vec<Address>,
}
```

### Mode 1: Delegation (Recommended)

Funds never leave the owner's wallet. The Golem holds a disposable session key and a signed delegation authorizing it to spend from the owner's Smart Account, subject to on-chain caveat enforcers. Every transaction executes *from* the owner's address -- on Etherscan, the owner is `msg.sender`, not the Golem. If the session key leaks, the attacker is bounded by caveats. The owner revokes from MetaMask directly. No Golem cooperation needed.

The paradigm shift: instead of "keep the key secret" (which requires TEEs, HSMs, secure enclaves), the architecture says "bound the damage if the key leaks" (which requires only math on-chain).

### Mode 2: Embedded (Privy)

Funds transfer to a Privy server wallet running in AWS Nitro Enclaves. The private key never leaves the hardware enclave. Policy enforcement is off-chain, inside the TEE, and binary -- the signing policy compiled from strategy risk bounds determines what the wallet can sign. Simpler to set up, but the owner surrenders direct custody and must trust Privy's TEE integrity.

### Mode 3: Local Key + Delegation

A locally generated keypair (secp256k1 or P-256) bounded by an on-chain delegation. The key is insecure in the traditional sense -- no TEE, no HSM. The delegation bounds the blast radius. Intended for development, hackathon demos, and self-hosted owners who trust their own infrastructure.

### Mode Comparison

| Property | Delegation | Embedded (Privy) | LocalKey |
|----------|-----------|------------------|----------|
| Funds location | Owner's wallet (always) | Privy server wallet | Owner's wallet (delegation) |
| Key protection | Caveats bound damage | TEE isolates key | Caveats bound damage |
| Sweep at death | No | Yes | No |
| Stuck fund risk | None | Yes (if sweep fails) | None |
| On-chain verifiability | Full (DelegationManager) | None (trust TEE) | Full (DelegationManager) |
| Setup complexity | Medium (Smart Account) | Low (Privy handles it) | Low (keygen + delegation) |

---

## 2. Delegation Architecture

### 2.1 The Delegation Tree

The permission structure is a tree rooted at the owner's MetaMask Smart Account. Sub-delegations attenuate strictly -- a child can never exceed its parent's authority. The on-chain DelegationManager (ERC-7710) enforces this invariant.

```rust
use alloy::primitives::{Address, Bytes, B256};
use std::collections::HashMap;

/// A single node in the delegation tree.
/// Each node represents a signed permission grant from delegator to delegate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationNode {
    pub delegator: Address,
    pub delegate: Address,
    /// Hash of the parent delegation. None for root delegations.
    pub authority: Option<B256>,
    pub caveats: Vec<CaveatEnforcer>,
    pub signature: Bytes,
    pub salt: u64,
}

/// The complete delegation tree for a Golem and its Replicants.
/// Invariant: every edge satisfies child.permissions ⊆ parent.permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationTree {
    /// The owner's MetaMask Smart Account.
    pub root: Address,
    /// delegation_hash → node
    pub nodes: HashMap<B256, DelegationNode>,
    /// Parent-child edges: (parent_hash, child_hash)
    pub edges: Vec<(B256, B256)>,
}
```

A concrete example:

```
Owner (MetaMask Smart Account -- ERC-7710)
  |
  +-- Delegation D1: Golem Alpha (vault manager)
  |   Caveats: [maxSpend($1000/day), approvedAssets, maxDrawdown(15%), timeWindow(30d)]
  |
  |   +-- Sub-Delegation D1.1: Replicant Alpha-1 (hypothesis tester)
  |   |   Caveats: [maxSpend($50), maxLifespan(24h), readOnly, noSubDelegation]
  |   |   (auto-expires after 24h)
  |   |
  |   +-- Phase-Gated Delegations:
  |       D1.P0 (Thriving):     [fullTrading, maxPosition(30%), replicantSpawning]
  |       D1.P1 (Stable):       [fullTrading, maxPosition(20%), noReplicantSpawning]
  |       D1.P2 (Conservation): [closeOnly, noNewPositions, withdrawAllowed]
  |       D1.P3 (Declining):    [unwindOnly, sweepToOwner]
  |       D1.P4 (Terminal):     [settlementOnly, deathProtocol]
  |
  +-- Delegation D2: Golem Beta (DCA executor)
      Caveats: [maxSpend($100/week), onlySwap(USDC→ETH), cronOnly(weekly)]
```

Phase-gated delegations restrict actions by behavioral phase (`02-mortality/00-overview.md`). A Golem in Conservation phase physically cannot open new positions because the caveat enforcer rejects the transaction on-chain.

### 2.2 Replicant Sub-Delegation

When a Golem spawns a Replicant, it sub-delegates from its own delegation rather than creating an independent wallet. ERC-7710 guarantees attenuation: the sub-delegation can only reduce permissions, never expand them.

```rust
/// Spawn a Replicant with attenuated permissions from the parent delegation.
/// The Replicant's delegation chain is: Owner → Parent Golem → Replicant.
pub async fn spawn_replicant_delegation(
    parent_delegation: &DelegationNode,
    replicant_config: &ReplicantConfig,
    session_key: &alloy::signers::local::LocalSigner,
) -> Result<DelegationNode> {
    let replicant_caveats = vec![
        CaveatEnforcer::DailySpendLimit {
            daily_limit_usd: replicant_config.budget_usd,
        },
        CaveatEnforcer::MortalityTimeWindow {
            start_time: now_secs(),
            end_time: now_secs() + replicant_config.lifespan_secs,
        },
        CaveatEnforcer::ReplicantBudget {
            max_budget_usd: replicant_config.budget_usd,
            max_lifespan_seconds: replicant_config.lifespan_secs,
        },
    ];

    // DelegationManager verifies: replicant_caveats ⊆ parent_caveats
    // If the Replicant tries [maxSpend($2000)] but parent has [maxSpend($1000)],
    // the DelegationManager reverts.
    let delegation = create_delegation(CreateDelegationParams {
        delegator: parent_delegation.delegator,
        delegate: replicant_config.wallet_address,
        authority: Some(parent_delegation.hash()),
        caveats: replicant_caveats,
        salt: replicant_config.replicant_id,
    });

    let signature = session_key.sign_typed_data(&delegation.to_typed_data()).await?;
    Ok(DelegationNode { signature, ..delegation })
}
```

The entire Replicant permission tree is on-chain verifiable. The owner can revoke D1, which automatically invalidates D1.1. A compromised Replicant key cannot exceed its budget, and it cannot spawn further sub-delegations because the caveats don't include re-delegation authority.

### 2.3 Revocation

One on-chain transaction disables the delegation hash in the DelegationManager. Works even if the Golem's infrastructure is offline, even if the platform is down. No cooperation from the Golem is needed.

```rust
/// Tracks the lifecycle state of a delegation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevocationState {
    Active,
    RevokedByOwner { revoked_at: u64, tx_hash: String },
    Expired { expired_at: u64 },
    Exhausted { total_calls: u32 },
}
```

Revocation from MetaMask is immediate and unilateral:

```
MetaMask → Settings → Delegations → Active Delegation: "Golem Alpha"
    ├── Spending: $47.30 / $100.00 this week
    ├── Expires: April 15, 2026
    ├── Calls made: 12 / 500
    │
    ├── [Revoke Delegation]  ← one click, one on-chain tx
    └── [Adjust Limits]      ← modify without full revocation
```

After revocation, the session key cannot redeem any further actions. The Golem's next execution attempt fails with `DELEGATION_DISABLED`. Because funds were never transferred, there is nothing to sweep.

---

## 3. Seven Custom Caveat Enforcers

Each is a deployed Solidity contract implementing the MetaMask Delegation Framework's `ICaveatEnforcer` interface. The Rust enum represents the `_terms` bytes that parameterize each enforcer on-chain.

```rust
/// Caveat enforcers constrain what a delegation can do.
/// Each variant maps to a deployed Solidity contract.
/// The DelegationManager calls enforceCaveat() before every execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaveatEnforcer {
    /// Restricts actions by behavioral phase. Reads current phase from
    /// VitalityOracle. A Golem in Conservation cannot open new positions.
    /// See `02-mortality/00-overview.md` for phase definitions.
    GolemPhase { vitality_oracle: Address, golem_address: Address },

    /// Time-locked delegation. When block.timestamp > end_time, the
    /// delegation is dead. Maps to projected lifespan.
    MortalityTimeWindow { start_time: u64, end_time: u64 },

    /// On-chain structural atonia. Blocks writes during dream cycles.
    /// Even if a code bug fires an action during a dream, the enforcer
    /// blocks it. See `03-daimon/06-dream-daimon.md`.
    DreamMode { dream_oracle: Address, golem_address: Address },

    /// Limits actions based on vault NAV percentage. Prevents a single
    /// trade from consuming more than X% of total vault value.
    VaultNAV { vault_address: Address, max_nav_pct: u16 },

    /// Caps Replicant sub-delegation spending and lifespan.
    /// Prevents unbounded Replicant proliferation.
    ReplicantBudget { max_budget_usd: u64, max_lifespan_seconds: u64 },

    /// Bounds acceptable slippage on swap transactions. Reads swap
    /// calldata and verifies minAmountOut against the configured tolerance.
    MaxSlippage { max_slippage_bps: u16 },

    /// Rolling 24h spending limit across all executions.
    /// Resets every block.timestamp / 86400 boundary.
    DailySpendLimit { daily_limit_usd: u64 },
}
```

### 3.1 GolemPhaseEnforcer

The behavioral phase constraint on-chain. Instead of TypeScript hook enforcement, phase restrictions are smart contract code that the DelegationManager validates before executing any action.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { ICaveatEnforcer } from "@metamask/delegation-framework/ICaveatEnforcer.sol";

/// @title GolemPhaseEnforcer
/// @notice Enforces behavioral phase constraints on Golem delegations.
///         Reads the Golem's current phase from the VitalityOracle and
///         restricts actions based on the phase. Phase 0 (Thriving) allows
///         all actions; Phase 4 (Terminal) allows only settlement.
/// @dev The phasePermissions bitmap is set in the constructor and cannot be
///      modified after deployment. This is intentional -- the phase rules
///      are part of the safety invariant.
contract GolemPhaseEnforcer is ICaveatEnforcer {
    /// @notice Oracle that reports the Golem's current behavioral phase
    IVitalityOracle public immutable vitalityOracle;

    /// @notice Phase => function selector => allowed
    mapping(uint8 => mapping(bytes4 => bool)) public phasePermissions;

    bytes4 constant SWAP_SELECTOR = 0x12345678;
    bytes4 constant ADD_LIQUIDITY_SELECTOR = 0x23456789;
    bytes4 constant REMOVE_LIQUIDITY_SELECTOR = 0x34567890;
    bytes4 constant DEPOSIT_SELECTOR = 0x45678901;
    bytes4 constant WITHDRAW_SELECTOR = 0x56789012;
    bytes4 constant SPAWN_REPLICANT_SELECTOR = 0x67890123;
    bytes4 constant SETTLEMENT_SELECTOR = 0x78901234;

    constructor(address _vitalityOracle) {
        vitalityOracle = IVitalityOracle(_vitalityOracle);

        // Thriving (0): all actions
        phasePermissions[0][SWAP_SELECTOR] = true;
        phasePermissions[0][ADD_LIQUIDITY_SELECTOR] = true;
        phasePermissions[0][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[0][DEPOSIT_SELECTOR] = true;
        phasePermissions[0][WITHDRAW_SELECTOR] = true;
        phasePermissions[0][SPAWN_REPLICANT_SELECTOR] = true;

        // Stable (1): all except replicant spawning
        phasePermissions[1][SWAP_SELECTOR] = true;
        phasePermissions[1][ADD_LIQUIDITY_SELECTOR] = true;
        phasePermissions[1][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[1][DEPOSIT_SELECTOR] = true;
        phasePermissions[1][WITHDRAW_SELECTOR] = true;

        // Conservation (2): close/withdraw only
        phasePermissions[2][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[2][WITHDRAW_SELECTOR] = true;

        // Declining (3): unwind + sweep only
        phasePermissions[3][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[3][WITHDRAW_SELECTOR] = true;

        // Terminal (4): settlement only
        phasePermissions[4][SETTLEMENT_SELECTOR] = true;
    }

    /// @notice Validates that the requested action is allowed in the
    ///         Golem's current behavioral phase.
    /// @param _terms ABI-encoded golem address
    /// @param _action The action the delegation is trying to execute
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 /* _delegationHash */
    ) external view override returns (bool) {
        address golem = abi.decode(_terms, (address));
        uint8 currentPhase = vitalityOracle.getPhase(golem);
        bytes4 actionSelector = bytes4(_action.data[:4]);

        require(
            phasePermissions[currentPhase][actionSelector],
            "GolemPhaseEnforcer: action blocked by phase"
        );
        return true;
    }
}
```

### 3.2 VitalityOracle

The Golem's vitality state must be accessible on-chain for the GolemPhaseEnforcer to read. A lightweight oracle posts phase transitions.

```solidity
/// @title VitalityOracle
/// @notice On-chain record of each Golem's current behavioral phase and
///         vitality score. The Golem's runtime posts updates; caveat
///         enforcers read them.
/// @dev Phase transitions are one-directional (monotonically increasing).
///      A Golem cannot go from Conservation back to Thriving. The owner
///      can override this restriction via ownerResetPhase() for debugging.
contract VitalityOracle {
    /// @notice golemAddress => current phase (0-4)
    mapping(address => uint8) public phases;

    /// @notice golemAddress => vitality score (0-1000, fixed-point 3 decimals)
    mapping(address => uint16) public vitality;

    event PhaseUpdated(address indexed golem, uint8 phase, uint16 vitality);

    /// @notice Only the Golem's runtime can update its own phase
    modifier onlyGolem(address golem) {
        require(msg.sender == golem, "VitalityOracle: unauthorized");
        _;
    }

    /// @notice Update phase and vitality. Phase can only increase.
    /// @param newPhase The new behavioral phase (0-4)
    /// @param newVitality The new vitality score (0-1000)
    function updatePhase(uint8 newPhase, uint16 newVitality)
        external
        onlyGolem(msg.sender)
    {
        require(newPhase <= 4, "VitalityOracle: invalid phase");
        require(newVitality <= 1000, "VitalityOracle: invalid vitality");
        require(newPhase >= phases[msg.sender], "VitalityOracle: phase cannot decrease");

        phases[msg.sender] = newPhase;
        vitality[msg.sender] = newVitality;

        emit PhaseUpdated(msg.sender, newPhase, newVitality);
    }

    function getPhase(address golem) external view returns (uint8) {
        return phases[golem];
    }
}
```

Phase updates are emitted as events, making the Golem's mortality journey publicly observable. Anyone can watch a Golem's vitality decline in real time.

### 3.3 MortalityTimeWindowEnforcer

Maps the Golem's projected lifespan to a hard on-chain deadline.

```solidity
/// @title MortalityTimeWindowEnforcer
/// @notice Auto-expires a delegation when block.timestamp exceeds the
///         configured end time. The Golem's projected lifespan becomes
///         an on-chain constraint.
contract MortalityTimeWindowEnforcer is ICaveatEnforcer {
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 /* _delegationHash */
    ) external view override returns (bool) {
        (uint256 startTime, uint256 endTime) = abi.decode(_terms, (uint256, uint256));
        require(block.timestamp >= startTime, "MortalityTimeWindow: not yet active");
        require(block.timestamp <= endTime, "MortalityTimeWindow: expired");
        return true;
    }
}
```

### 3.4 DreamModeEnforcer

Structural atonia -- even if the Golem's code has a bug that fires an action during a dream cycle (`03-daimon/06-dream-daimon.md`), the on-chain enforcer blocks it.

```solidity
/// @title DreamModeEnforcer
/// @notice Blocks all write executions while the Golem is dreaming.
///         Defense-in-depth: even if the off-chain dream scheduler fails,
///         on-chain enforcement holds.
contract DreamModeEnforcer is ICaveatEnforcer {
    IDreamOracle public immutable dreamOracle;

    constructor(address _dreamOracle) {
        dreamOracle = IDreamOracle(_dreamOracle);
    }

    function enforceCaveat(
        bytes calldata _terms,
        Action calldata /* _action */,
        bytes32 /* _delegationHash */
    ) external view override returns (bool) {
        address golem = abi.decode(_terms, (address));
        require(!dreamOracle.isDreaming(golem), "DreamMode: golem is dreaming");
        return true;
    }
}
```

### 3.5 MaxSlippageEnforcer and DailySpendLimitEnforcer

```solidity
/// @title MaxSlippageEnforcer
/// @notice Reads swap calldata and verifies that minAmountOut falls within
///         the configured slippage tolerance. Protocol-agnostic -- works
///         with any swap router that follows the (amountIn, minAmountOut)
///         parameter convention.
contract MaxSlippageEnforcer is ICaveatEnforcer {
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 /* _delegationHash */
    ) external view override returns (bool) {
        uint16 maxSlippageBps = abi.decode(_terms, (uint16));
        uint256 minAmountOut = extractMinAmountOut(_action.data);
        uint256 expectedOut = getExpectedOutput(_action.data);
        uint256 actualSlippage = ((expectedOut - minAmountOut) * 10000) / expectedOut;
        require(actualSlippage <= maxSlippageBps, "MaxSlippage: exceeded limit");
        return true;
    }
}

/// @title DailySpendLimitEnforcer
/// @notice Enforces a rolling 24-hour spending cap across all executions
///         under a given delegation. Stateful -- tracks cumulative spend
///         per day in a mapping.
contract DailySpendLimitEnforcer is ICaveatEnforcer {
    /// @notice delegationHash => day => amount spent (USD, 6 decimals)
    mapping(bytes32 => mapping(uint256 => uint256)) public dailySpend;

    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external override returns (bool) {
        uint256 dailyLimitUsd = abi.decode(_terms, (uint256));
        uint256 today = block.timestamp / 86400;
        uint256 actionValueUsd = estimateActionValueUsd(_action);

        require(
            dailySpend[_delegationHash][today] + actionValueUsd <= dailyLimitUsd,
            "DailySpendLimit: limit exceeded"
        );

        dailySpend[_delegationHash][today] += actionValueUsd;
        return true;
    }
}
```

---

## 4. Session Key Lifecycle

A session key is an ephemeral keypair that signs UserOperations on behalf of the delegation. It has no independent authority -- worthless without a valid delegation to redeem.

```rust
/// An ephemeral signing key. The private key exists only in process memory
/// (hosted mode) or encrypted on disk (local mode). The key has no
/// independent value -- it can only sign within the delegation's bounds.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionKey {
    pub address: Address,
    pub granted_at: u64,
    pub expires_at: u64,
    pub remaining_budget_usd: f64,
    pub rotation_policy: RotationPolicy,
    pub operations_signed: u64,
}

/// When to rotate the session key. Rotation generates a fresh keypair,
/// requests a new delegation grant, and atomically switches. The old key
/// is zeroized from memory (see `00-defense.md` §5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Maximum operations before forced rotation. Default: 10,000.
    pub max_operations: u64,
    /// Maximum age before forced rotation. Default: 7 days.
    pub max_age_seconds: u64,
    /// Rotate when remaining budget drops below this fraction. Default: 0.1.
    pub budget_floor_fraction: f64,
}

impl SessionKey {
    pub fn should_rotate(&self, current_time: u64) -> bool {
        let age = current_time.saturating_sub(self.granted_at);
        age >= self.rotation_policy.max_age_seconds
            || self.operations_signed >= self.rotation_policy.max_operations
            || self.remaining_budget_usd <= 0.0
    }
}
```

**Generation.** Fresh keypair at boot. Hosted mode: private key in process memory, never on disk. Local mode: persisted to `$GOLEM_DATA/session-key.json` (encrypted at rest via `golem-crypto`). Key type matches delegation expectations: secp256k1 for standard, P-256 for Privy-derived.

**Delegation granting.** Owner signs an ERC-7715 permission grant targeting the session key's address. Browser (MetaMask popup), mobile deep-link, or CLI for offline signing.

**Usage tracking.** Every UserOperation is counted. Budget tracked locally; reconciled against on-chain state when estimates diverge.

**Rotation.** When `should_rotate()` returns true: generate new keypair, request fresh delegation, atomically switch. Old key is zeroized from memory (`00-defense.md` §5 secret zeroization). Brief write-action gap is acceptable -- the heartbeat can skip a tick.

**Revocation/Expiry.** Three paths: owner revokes on-chain, MortalityTimeWindow expires, or call limit exhausts. The Golem detects the revert, logs to the audit chain (`00-defense.md` §4), and either requests renewal or initiates Thanatopsis (the four-phase death protocol: Acceptance, Settlement, Reflection, Legacy; see `01-golem/05-death.md`).

### Session Key Events

```rust
/// Events emitted through the Event Fabric (Subsystem::Lifecycle).
/// See `01-golem/13-runtime-extensions.md` for the EventPayload enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionKeyEvent {
    Generated { address: Address, expires_at: u64 },
    DelegationGranted { delegation_hash: B256, caveats: Vec<String> },
    OperationSigned { op_hash: B256, value_usd: f64, remaining_budget: f64 },
    RotationTriggered { reason: String, old_address: Address, new_address: Address },
    Revoked { cause: RevocationState },
    BudgetWarning { remaining_pct: f64, estimated_operations_left: u64 },
}
```

---

## 5. Self-Funding Economics

### 5.1 The Metabolic Loop

Jonas (1966) described metabolism as self-sustaining through the very activity that requires sustenance. A self-funding Golem achieves the computational equivalent: it trades to earn revenue that pays for the inference that produces better trades.

```rust
/// Tracks the Golem's economic sustainability.
/// The sustainability_ratio feeds directly into the economic clock
/// (`02-mortality/00-overview.md`). A ratio >1.0 means the clock ticks up.
/// A ratio <0.5 means the Golem is dying fast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicState {
    /// Revenue / total cost. >1.0 = self-sustaining. >2.0 = thriving.
    /// <1.0 = declining. <0.5 = dying.
    pub sustainability_ratio: f64,
    pub inference_spend: f64,
    pub revenue: f64,
    /// Feeds the economic clock.
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
        if total > 0.0 {
            self.projected_lifespan_hours =
                ((self.revenue - self.inference_spend) / total) * 24.0;
        }
    }
}
```

### 5.2 Mortality-Aware Model Routing

As the economic clock ticks down, the Golem shifts to cheaper inference models. This is rational inattention (Sims 2003) applied to model selection. A dying Golem burning $5/day on an expensive model when a cheaper one at $0.50/day would keep it alive is making an irrational allocation.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InferenceTier {
    /// Cheapest: small models, flash-class. $0.10-0.50/day.
    T0,
    /// Mid-range: Haiku/Gemini Flash. $0.50-2.00/day.
    T1,
    /// Full power: Opus/Sonnet. $2.00-10.00/day.
    T2,
}

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

### 5.3 Cross-Model Verification

For high-stakes decisions, the Golem queries two different models and compares outputs. Both must agree before execution proceeds. The agreement threshold defaults to 0.7 cosine similarity on the structured output.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossModelVerification {
    pub primary: ModelRecommendation,
    pub secondary: ModelRecommendation,
    pub agreement: f64,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model_id: String,
    pub action: String,
    pub confidence: f64,
    pub reasoning_hash: B256,
}
```

---

## 6. Onboarding Flow

Six steps, forward-only. The exact procedure varies by custody mode.

```rust
/// Tracks the onboarding state machine. Each step must complete before
/// the next begins. No backward transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnboardingState {
    /// Step 1: Browser handoff, QR code, or device code grant (RFC 8628).
    Authenticating { method: AuthMethod },
    /// Step 2: Owner picks Delegation, Embedded, or LocalKey.
    SelectingCustody { owner_address: Address, auth_token: String },
    /// Step 3: Delegation = sign one ERC-7715 grant. Embedded = transfer
    /// funds. LocalKey = generate keypair + sign delegation.
    SettingUpWallet { owner_address: Address, custody_mode: CustodyMode },
    /// Step 4: STRATEGY.md + HEARTBEAT.md authoring.
    AuthoringStrategy { owner_address: Address, custody_mode: CustodyMode },
    /// Step 5: Optional ERC-8004 (on-chain agent identity standard on Base L2) identity registration.
    RegisteringIdentity { owner_address: Address, strategy_hash: B256 },
    /// Step 6: The Summoning. First heartbeat tick. All clocks start.
    Summoning { owner_address: Address, custody_mode: CustodyMode },
    /// Terminal: Golem is running.
    Live { golem_id: String, custody_mode: CustodyMode },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    BrowserHandoff { redirect_uri: String },
    QrCode { session_id: String },
    DeviceCodeGrant { device_code: String, user_code: String },
}
```

### Onboarding Timeline by Mode

| Mode | On-chain Txs | Owner Confirmations | Wall Clock |
|------|-------------|-------------------|------------|
| Delegation | 0-1 (optional identity) | 1 signature | 15-30s |
| Embedded | 2-3 (approve + transfer + identity) | 3-4 wallet popups | 60-120s |
| LocalKey | 0-1 (optional identity) | 1 signature (or pre-signed) | 5-10s |

Delegation eliminates two on-chain transactions (approve + transfer). The fund transfer is replaced by a single signature. The owner sees a MetaMask popup:

```
Golem "Alpha DCA" requests:

  Spend up to 100 USDC per week
  Use up to 0.05 ETH per week for gas
  Only interact with: Uniswap Router, WETH
  Valid for 30 days

  Your funds stay in YOUR wallet.
  You can revoke anytime from MetaMask.

  [Adjust Limits]  [Grant Permission]  [Cancel]
```

### Custody Events

All onboarding and custody lifecycle events are emitted through the Event Fabric as `Subsystem::Lifecycle` payloads:

- `CustodyModeSelected { mode: String, owner: Address }`
- `DelegationGranted { delegation_hash: B256, caveats: Vec<String>, expires_at: u64 }`
- `SessionKeyRotated { old_address: Address, new_address: Address, reason: String }`
- `DelegationRevoked { delegation_hash: B256, cause: String }`
- `DelegationExpiringWarning { hours_remaining: f64, delegation_hash: B256 }`

---

## 7. Death Settlement by Mode

How a Golem's financial state resolves at death depends on the custody mode. This is the final act of Thanatopsis (`01-golem/05-death.md`).

```rust
/// How the Golem's financial state resolves at death.
/// The inner settlement type depends on the custody mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeathSettlement {
    /// Delegation: the cleanest death. Session key expires. No sweep.
    /// Funds were never transferred. Open positions close back to
    /// the owner's address (that was always the execution address).
    DelegationExpiry {
        delegation_hash: B256,
        expiry_cause: ExpiryKind,
        positions_settled: Vec<SettledPosition>,
    },

    /// Embedded: requires sweep. BardoManifest records deferred
    /// positions. Privy wallet persists in AWS Nitro Enclaves
    /// independently of the VM.
    PrivySweep {
        server_wallet_id: String,
        bardo_manifest: BardoManifest,
        positions_settled: Vec<SettledPosition>,
        positions_deferred: Vec<DeferredPosition>,
    },

    /// Self-funding: economic clock exhaustion. The clock can go UP
    /// if revenue > spend. Death only when revenue is permanently
    /// insufficient. Inner settlement follows the custody mode.
    EconomicExhaustion {
        final_sustainability_ratio: f64,
        lifetime_revenue: f64,
        lifetime_inference_spend: f64,
        inner: Box<DeathSettlement>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpiryKind {
    NaturalExpiry,
    OwnerRevocation,
    CallLimitReached,
}
```

### The "No Sweep" Death

Delegation mode is strictly superior for death settlement:

| Aspect | Delegation | Embedded (Privy) |
|--------|-----------|-----------------|
| Funds at death | Owner's wallet (always were) | Privy server wallet |
| Sweep required | No | Yes |
| Stuck fund risk | None | Yes (if sweep fails) |
| Owner action needed | None | Must claim deferred positions |
| Gas required for settlement | None (positions close to owner's address) | Yes (sweep tx gas) |

The "no sweep" death eliminates stuck funds, failed sweeps, gas estimation errors during teardown, and the race condition between death and sweep confirmation. When a Golem in Delegation mode dies:

1. Close positions -- USDC stays in the owner's MetaMask (it was always there)
2. Delegation expires or owner revokes
3. Done.

Death settlement events (`Subsystem::Lifecycle`):

- `GolemEvent::DeathProtocolStarted { trigger, vitality_score }`
- `GolemEvent::PositionsUnwound { positions_exited, total_recovered, slippage_cost }`
- `GolemEvent::DeathCompleted { golem_id, lifetime_ticks, total_cost_usd, final_nav }`

---

## 8. TEE Limitations

TEE provides probabilistic defense-in-depth, not defense-in-total. Three escalating attacks have rendered TEE-only security untenable:

- **TEE.Fail** (ACM CCS 2025): Physical side-channel attacks using <$1,000 hardware break Intel SGX, Intel TDX, and AMD SEV-SNP [TEE-FAIL-2025]
- **BadRAM** (IEEE S&P 2025): DDR4/DDR5 SPD chip tampering costing <$10 breaks AMD SEV-SNP [BADRAM-2025]
- **Battering RAM** (IEEE S&P 2026): A DDR4/DDR5 memory interposer costing <$50 breaks Intel TDX, AMD SEV-SNP, AND NVIDIA Confidential Computing [VANBULCK-2026]

The convergence is definitive: for cloud-hosted agents, TEEs alone are insufficient. They remain necessary (they raise the attack bar for remote-only adversaries) but must be layered with on-chain delegation enforcement, active monitoring, and optionally time-delayed execution (see `02-warden.md`).

This is why Delegation mode exists. The TEE protects the Privy signing key (Embedded mode), but the delegation caveats protect the owner's funds regardless of TEE integrity (Delegation mode). If the TEE breaks, Delegation mode is unaffected -- the security guarantee comes from the DelegationManager smart contract, not from hardware.

---

## 9. Custody Architecture Comparison

Every agent wallet solution reduces to one of four core cryptographic architectures. They have fundamentally different trust assumptions. A time-delayed proxy execution layer can sit on top of any of them.

| Architecture | Providers | Signing Latency | Trust Assumption | Policy Enforcement | Defense Type |
|-------------|-----------|----------------|-----------------|-------------------|-------------|
| On-chain delegation (ERC-7710) | MetaMask, Bardo | Varies (+ gas) | Smart contract code | On-chain (DelegationManager) | Preventive |
| TEE (enclaves) | Privy | 100-200ms | Enclave hardware + provider | Infrastructure-level (inside TEE) | Preventive |
| Smart accounts (ERC-4337) | Safe, ZeroDev, Biconomy | Varies (+ gas) | Smart contract code | On-chain (validateUserOp) | Preventive |
| Decentralized MPC | Lit Protocol (Vincent) | 200-500ms | 2/3 of node operators | On-chain + off-chain | Preventive |
| Time-delayed proxy | Zodiac, Warden | Configurable delay | Smart contract logic | On-chain + monitoring | **Reactive** |

On-chain delegation is the recommended default because it combines the strongest trust model (smart contract code, auditable, on-chain verifiable) with the best user experience (funds never leave the owner's wallet, no sweep at death, one-click revocation).

---

## 10. Owner Trust Model

```
+------------------------------------------------------------------+
|  OWNER                                                            |
|                                                                   |
|  HAS:                                                             |
|  - MetaMask Smart Account (or Privy embedded wallet)              |
|  - Signed delegation to the Golem                                 |
|  - One-click revocation via MetaMask (Delegation mode)            |
|  - Dashboard access via bardo.app                                 |
|                                                                   |
|  CAN:                                                             |
|  - Revoke delegation instantly (no Golem cooperation needed)      |
|  - Adjust spending limits without full revocation                 |
|  - Extend or shorten delegation time windows                      |
|  - View all Golem transactions in their MetaMask history          |
|                                                                   |
|  BARDO OPERATOR                                                   |
|                                                                   |
|  HAS:                                                             |
|  - Compute infrastructure (VM management)                         |
|  - Inference gateway (LLM routing)                                |
|  - Public wallet addresses (on-chain, visible to anyone)          |
|                                                                   |
|  CANNOT:                                                          |
|  - Sign transactions (requires session key in VM)                 |
|  - Extract private keys (Delegation: not held; Privy: TEE)        |
|  - Access funds in owner wallets                                  |
|  - Override delegation caveats (enforced on-chain)                |
|                                                                   |
|  CAN (maliciously):                                               |
|  - Deny service (shut down VMs)                                   |
|  - Read public on-chain data (like anyone else)                   |
|                                                                   |
|  WORST CASE: Denial of service, NEVER fund theft.                 |
+------------------------------------------------------------------+
```

---

## 11. Recovery and Disaster Scenarios

| Scenario | Impact | Recovery |
|----------|--------|----------|
| Process crash (OOM, panic) | Agent stops operating | VM auto-restarts. Session resumes. |
| VM destroyed (hardware failure) | Agent stops, session key lost | Delegation: funds safe (owner's wallet). Privy: wallet survives in TEE. |
| VM destroyed during transaction | Pending tx may or may not land | Funds safe in either mode. Delegation bounds limit worst case. |
| Privy API outage (Embedded mode) | Cannot sign transactions | Observe-only mode. Retries with exponential backoff. |
| DelegationManager contract paused | Cannot redeem delegations | Observe-only. Owner retains full wallet access. |
| Session key compromised | Attacker can sign within delegation bounds | Damage bounded by caveats. Owner revokes delegation on-chain. |

---

## Cross-References

- [00-defense.md](00-defense.md) -- The main defense-in-depth architecture doc: six defense layers, `Capability<T>` compile-time tokens, `TaintedString` information-flow control, Merkle hash-chain audit log, and `zeroize` secret wiping.
- [02-warden.md](02-warden.md) -- Optional Warden time-delayed proxy: transactions are announced, held for a configurable delay, then executed or cancelled, adding a human-reviewable window before irreversible on-chain actions.
- [03-policy.md](03-policy.md) -- PolicyCage on-chain smart contract (the "DeFi Constitution"): asset whitelists, spending caps, drawdown breakers, position limits, and the `IPolicyCage` Solidity interface.
- [../09-economy/00-identity.md](../09-economy/00-identity.md) -- ERC-8004 on-chain agent identity: registration, reputation scoring, clade discovery, and reputation-gated access to shared knowledge tiers.
- [../02-mortality/00-overview.md](../02-mortality/00-overview.md) -- Three independent death clocks (economic, epistemic, stochastic), VitalityState composition, and the five behavioral phases (Thriving through Terminal).
- [../01-golem/05-death.md](../01-golem/05-death.md) -- Thanatopsis four-phase death protocol (Acceptance, Settlement, Reflection, Legacy), Grimoire genomic bottleneck, death testament, and generational knowledge transfer.

---

## References

- MetaMask Delegation Framework: https://github.com/MetaMask/delegation-framework — The reference implementation for ERC-7710/7715 delegations that Bardo's Delegation custody mode builds on. Provides caveat enforcers, delegation manager, and session key infrastructure.
- ERC-7710: Smart Contract Delegation Standard — Defines how a smart account can delegate specific permissions to another address. The on-chain primitive behind Bardo's session key delegations.
- ERC-7715: Request Permissions (wallet_grantPermissions) — The wallet RPC method that requests delegation permissions from the user. Used during onboarding to create the initial delegation.
- EIP-7702: Set EOA Account Code (EOA to Smart Account upgrade) — Allows an EOA to behave as a smart account by setting its code. Enables Delegation mode for users who start with a standard EOA wallet.
- ERC-4337: Account Abstraction Using Alt Mempool — The Account Abstraction standard that enables UserOperations signed by session keys. The underlying infrastructure for gasless, batched, and sponsored transactions.
- [VANBULCK-2026] Van Bulck, J. et al. "Battering RAM." IEEE S&P 2026. Demonstrates physical attacks against TEE memory integrity, extracting secrets from Intel SGX and AMD SEV enclaves. Motivates the caveat-bounded custody model over TEE-only security.
- [BADRAM-2025] De Meulemeester, J. et al. "BadRAM." IEEE S&P 2025. Shows that AMD SEV-SNP memory encryption can be bypassed via physical DRAM manipulation. Another reason Bardo prefers on-chain delegation bounds over TEE-only trust.
- [TEE-FAIL-2025] "TEE.Fail." ACM CCS 2025. Comprehensive catalog of TEE vulnerabilities across Intel SGX, AMD SEV, and ARM TrustZone. Informs the risk assessment for Embedded (Privy TEE) custody mode.
- [JONAS-1966] Jonas, H. "The Phenomenon of Life." 1966. Argues that metabolism (self-sustaining activity) is the defining characteristic of living beings. The philosophical foundation for the "metabolic loop" where self-funding Golems earn revenue to pay for their own compute.
- [DENNIS-VAN-HORN-1966] Dennis, J.B. & Van Horn, E.C. "Programming Semantics for Multiprogrammed Computations." CACM, 9(3), 1966. Established capability-based security: unforgeable access tokens verified at the type level. The ancestor of the `Capability<T>` pattern and delegation-bounded custody.
- [SIMS-2003] Sims, C. "Implications of Rational Inattention." Journal of Monetary Economics, 50(3), 2003. Models how agents with limited processing capacity optimally allocate attention. Relevant to the mortality-aware resource allocation where dying agents must budget inference spend.
