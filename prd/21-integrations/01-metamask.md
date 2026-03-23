# MetaMask Delegation Framework -- frictionless custody for mortal agents [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-14

> **Crates**: `golem-chain` (delegation types, VitalityOracle client), `golem-safety` (policy enforcement), `golem-tools` (permission compiler) | **Prerequisites**: [00-overview.md](00-overview.md), [../07-tools/18-tools-metamask.md](../07-tools/18-tools-metamask.md), [../07-tools/22-wallets.md](../07-tools/22-wallets.md)
>
> This file specifies the MetaMask Delegation Framework from the Golem's perspective: how it holds delegated authority, manages session keys, respects phase-gated permissions, and settles at death. For the MCP tool surface and Snap UI, see [../07-tools/18-tools-metamask.md](../07-tools/18-tools-metamask.md). For general wallet architecture, see [../07-tools/22-wallets.md](../07-tools/22-wallets.md).

---

> **Reader orientation:** This document specifies how the Golem (a mortal autonomous DeFi agent) holds delegated spending authority via MetaMask's ERC-7710/7715 framework. It belongs to the **integrations layer** and covers custody modes, seven custom caveat enforcers that restrict actions by the Golem's survival phase, the VitalityOracle on-chain contract, session key lifecycle, and how strategies compile to permission requests. You should understand smart account delegation, ERC-4337, and basic DeFi custody patterns. For Bardo-specific terms, see `prd2/shared/glossary.md`.

## S1 -- The problem

Three wallets, two transfers, zero confidence.

The current state of agent custody: the owner creates a wallet, funds it with a token transfer, prays the agent doesn't drain it, and has no on-chain recourse if it does. Embedded wallets (Privy, Turnkey, etc.) improve key security via TEEs but don't change the fundamental problem -- funds leave the owner's control the moment they transfer. Revoking access means sweeping funds back, which requires cooperation from the agent's infrastructure.

MetaMask's Delegation Framework inverts this. Funds never leave the owner's MetaMask Smart Account. The Golem holds a signed delegation -- a permission object, not a key -- that grants bounded spending authority. The owner revokes by revoking the delegation from MetaMask. No sweep transaction. No agent cooperation. No prayer.

This is the recommended custody mode. The other two modes (Embedded via Privy, LocalKey for development) exist for compatibility and developer convenience. Every new Golem should default to Delegation.

---

## S2 -- Three custody modes

Detailed in [../07-tools/22-wallets.md](../07-tools/22-wallets.md). Summary for context:

| Mode | Funds location | Trust model | Provisioning time | Death settlement |
| --- | --- | --- | --- | --- |
| **Delegation** (recommended) | Owner's MetaMask Smart Account | On-chain caveats (ERC-7710/7715) | 15-30s | No sweep needed |
| **Embedded** (Privy, legacy) | Privy server wallet (TEE) | Off-chain TEE policy | 30-60s | Sweep required |
| **LocalKey** (dev only) | Owner's Smart Account or local EOA | On-chain delegation bounds | 5-10s | Delegation expires |

Delegation mode eliminates two on-chain transactions from provisioning (approve + transfer). It replaces them with a single ERC-7715 signature. The owner signs once, the Golem runs for its lifetime.

**Supported chains:** Base (8453), Ethereum (1), Celo (42220), Arbitrum (42161), Base Sepolia (84532), Sepolia (11155111).

---

## S3 -- Delegation tree

The permission structure is a tree rooted at the owner's MetaMask Smart Account (ERC-7710). Sub-delegations attenuate strictly -- a child delegation never exceeds its parent's authority. The DelegationManager enforces this invariant on-chain by walking the delegation chain and calling every caveat enforcer at each level.

```
Owner (MetaMask Smart Account)
  |
  +-- D1: Golem Alpha (vault manager)
  |   Caveats: [GolemPhase, MortalityTimeWindow, VaultNAV, MaxSlippage,
  |             ERC20TransferAmount($1000/day USDC), AllowedTargets,
  |             AllowedMethods]
  |   |
  |   +-- D1.1: Replicant Alpha-1 (hypothesis tester)
  |   |   Caveats: [ReplicantBudget($50, 24h), AllowedTargets(subset),
  |   |             LimitedCalls(100), NoSubDelegation]
  |   |
  |   +-- D1.2: Sleepwalker Observer
  |   |   Caveats: [ReadOnly, NoTransfers, OraclePublishOnly]
  |   |
  |   +-- Phase-gated sub-delegations:
  |       D1.P1 (Thriving):     [fullTrading, maxPosition(30%), replicantSpawning]
  |       D1.P2 (Stable):       [fullTrading, maxPosition(20%), noReplicantSpawning]
  |       D1.P3 (Conservation): [closeOnly, noNewPositions, withdrawAllowed]
  |       D1.P4 (Declining):    [unwindOnly, sweepToOwner]
  |       D1.P5 (Terminal):     [settlementOnly, deathProtocol]
  |
  +-- D2: Golem Beta (DCA executor)
      Caveats: [maxSpend($100/week), onlySwap(USDC->ETH), CronOnly(weekly)]
```

Each level in the tree is a `DelegationNode`:

```rust
use alloy::primitives::{Address, Bytes, B256};

/// A single node in the delegation tree.
/// The DelegationManager verifies the full chain from leaf to root
/// before executing any action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationNode {
    /// Who granted this delegation.
    pub delegator: Address,
    /// Who received it.
    pub delegate: Address,
    /// Parent delegation hash (None for root delegations from the owner).
    pub authority: Option<B256>,
    /// Ordered list of caveat enforcers. ALL must pass.
    pub caveats: Vec<CaveatEnforcer>,
    /// EIP-712 signature from the delegator.
    pub signature: Bytes,
    /// Unique salt to prevent replay.
    pub salt: u64,
}

/// A caveat enforcer reference: deployed contract address + ABI-encoded terms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaveatEnforcer {
    /// Address of the deployed enforcer contract.
    pub enforcer: Address,
    /// ABI-encoded terms specific to this enforcer.
    pub terms: Bytes,
}
```

---

## S4 -- Standard caveats

MetaMask's Delegation Framework ships with a set of standard caveat enforcers. The Golem uses seven of them directly:

| Enforcer | Purpose | Terms encoding |
| --- | --- | --- |
| `ERC20TransferAmountEnforcer` | Caps USDC spending per delegation | `(address token, uint256 maxAmount)` |
| `NativeTokenTransferAmountEnforcer` | Caps ETH spending (gas deposits) | `(uint256 maxAmount)` |
| `AllowedTargetsEnforcer` | Restricts callable contract addresses | `(address[] targets)` |
| `TimestampEnforcer` | Time-bounds the delegation | `(uint256 after, uint256 before)` |
| `BlockNumberEnforcer` | Block-bounds the delegation | `(uint256 afterBlock, uint256 beforeBlock)` |
| `LimitedCallsEnforcer` | Caps total number of calls | `(uint256 maxCalls)` |
| `AllowedMethodsEnforcer` | Restricts callable function selectors | `(bytes4[] selectors)` |

These are composed with custom Golem caveats (S5) on the same delegation. All enforcers in a delegation's caveat list must pass -- they are AND-composed, not OR-composed.

---

## S5 -- Custom Golem caveats

Seven custom enforcers implement Golem-specific behavioral constraints. Each is a Solidity contract deployed once per chain and referenced by address in delegation terms.

### 5.1 GolemPhaseEnforcer

Restricts actions by BehavioralPhase (one of five survival phases -- Thriving, Stable, Conservation, Declining, Terminal -- determined by Vitality, a composite 0.0-1.0 survival score). Reads the Golem's current phase from the VitalityOracle (S6). A Golem in Conservation cannot open new positions. A Golem in Terminal can only settle.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import { ICaveatEnforcer } from "@metamask/delegation-framework/ICaveatEnforcer.sol";
import { Action } from "@metamask/delegation-framework/Action.sol";
import { IVitalityOracle } from "./interfaces/IVitalityOracle.sol";

/// @title GolemPhaseEnforcer
/// @notice Restricts delegated actions based on the Golem's behavioral phase.
///         Phase is read from the VitalityOracle -- an on-chain source of truth
///         updated by the Golem's heartbeat via a signed transaction.
/// @dev    Phase transitions are monotonic: once a Golem enters Conservation,
///         it cannot return to Thriving. The oracle enforces this.
contract GolemPhaseEnforcer is ICaveatEnforcer {
    IVitalityOracle public immutable vitalityOracle;

    // Function selector constants for permission mapping
    bytes4 constant SWAP_SELECTOR = 0x12345678;
    bytes4 constant ADD_LIQUIDITY_SELECTOR = 0x23456789;
    bytes4 constant REMOVE_LIQUIDITY_SELECTOR = 0x34567890;
    bytes4 constant DEPOSIT_SELECTOR = 0x45678901;
    bytes4 constant WITHDRAW_SELECTOR = 0x56789012;
    bytes4 constant SPAWN_REPLICANT_SELECTOR = 0x67890123;
    bytes4 constant SETTLEMENT_SELECTOR = 0x78901234;

    /// @dev phase => selector => allowed
    mapping(uint8 => mapping(bytes4 => bool)) public phasePermissions;

    error ActionNotAllowedInPhase(uint8 phase, bytes4 selector);
    error GolemNotRegistered(address golem);

    constructor(address _vitalityOracle) {
        vitalityOracle = IVitalityOracle(_vitalityOracle);

        // Phase 0 (Thriving): everything allowed
        phasePermissions[0][SWAP_SELECTOR] = true;
        phasePermissions[0][ADD_LIQUIDITY_SELECTOR] = true;
        phasePermissions[0][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[0][DEPOSIT_SELECTOR] = true;
        phasePermissions[0][WITHDRAW_SELECTOR] = true;
        phasePermissions[0][SPAWN_REPLICANT_SELECTOR] = true;

        // Phase 1 (Stable): no replicant spawning
        phasePermissions[1][SWAP_SELECTOR] = true;
        phasePermissions[1][ADD_LIQUIDITY_SELECTOR] = true;
        phasePermissions[1][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[1][DEPOSIT_SELECTOR] = true;
        phasePermissions[1][WITHDRAW_SELECTOR] = true;

        // Phase 2 (Conservation): close and withdraw only
        phasePermissions[2][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[2][WITHDRAW_SELECTOR] = true;

        // Phase 3 (Declining): same as Conservation (unwind)
        phasePermissions[3][REMOVE_LIQUIDITY_SELECTOR] = true;
        phasePermissions[3][WITHDRAW_SELECTOR] = true;

        // Phase 4 (Terminal): settlement only
        phasePermissions[4][SETTLEMENT_SELECTOR] = true;
    }

    /// @notice Validates action against the Golem's current behavioral phase.
    /// @param _terms ABI-encoded Golem session key address (the delegate).
    /// @param _action The action the Golem is attempting.
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 /* _delegationHash */
    ) external view override returns (bool) {
        address golem = abi.decode(_terms, (address));

        (bool registered, uint8 currentPhase) = vitalityOracle.getPhase(golem);
        if (!registered) revert GolemNotRegistered(golem);

        bytes4 actionSelector = bytes4(_action.data[:4]);
        if (!phasePermissions[currentPhase][actionSelector]) {
            revert ActionNotAllowedInPhase(currentPhase, actionSelector);
        }

        return true;
    }
}
```

### 5.2 MortalityTimeWindowEnforcer

Auto-expires the delegation at the Golem's projected death time. When `block.timestamp` exceeds `endTime`, every execution reverts. The `endTime` is set at delegation creation based on the Golem's initial runway calculation and updated via delegation re-signing when the owner extends funding (the "Feed-the-Golem" flow, S9).

```solidity
/// @title MortalityTimeWindowEnforcer
/// @notice Time-locks a delegation to the Golem's projected lifespan.
///         When the Golem's runway expires, the delegation dies with it.
contract MortalityTimeWindowEnforcer is ICaveatEnforcer {
    error DelegationNotYetActive(uint256 current, uint256 start);
    error DelegationExpired(uint256 current, uint256 end);

    /// @param _terms ABI-encoded (uint256 startTime, uint256 endTime)
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 /* _delegationHash */
    ) external view override returns (bool) {
        (uint256 startTime, uint256 endTime) = abi.decode(
            _terms, (uint256, uint256)
        );

        if (block.timestamp < startTime) {
            revert DelegationNotYetActive(block.timestamp, startTime);
        }
        if (block.timestamp > endTime) {
            revert DelegationExpired(block.timestamp, endTime);
        }

        return true;
    }
}
```

### 5.3 VaultNAVEnforcer

Circuit breaker. Blocks any transaction whose value exceeds a configured percentage of the vault's total assets. Prevents a rogue Golem from draining a vault in a single transaction.

### 5.4 DreamModeEnforcer

On-chain structural atonia. Blocks all write operations during dream cycles by reading the DreamOracle. Even if a bug in the Golem's runtime fires an action during a dream, the on-chain enforcer stops it. Defense in depth -- the runtime enforces dream atonia too, but the on-chain layer cannot be bypassed.

### 5.5 ReplicantBudgetEnforcer

Caps Replicant (child agent) spending at the parent Golem's sub-allocated budget. Tracks cumulative spend per delegation hash. Enforces both USD-equivalent limits and lifespan bounds. A Replicant cannot outlive or outspend its allocation.

### 5.6 EmotionalStateEnforcer

Reads the Golem's current PAD vector (Pleasure-Arousal-Dominance emotional coordinates) from the DaimonOracle (on-chain projection of the Daimon, the Golem's affect engine) and restricts high-risk actions during extreme emotional states. A Golem experiencing high arousal + low dominance (panic) cannot open new leveraged positions. A Golem in negative pleasure + negative dominance (despair) is restricted to conservative actions.

```solidity
/// @title EmotionalStateEnforcer
/// @notice Restricts high-risk actions when the Golem's emotional state
///         indicates compromised decision-making capacity.
/// @dev    PAD values are int8 (-100 to +100). Thresholds are configurable
///         per delegation via the terms parameter.
contract EmotionalStateEnforcer is ICaveatEnforcer {
    IDaimonOracle public immutable daimonOracle;

    error EmotionalStateRestriction(int8 pleasure, int8 arousal, int8 dominance);

    constructor(address _daimonOracle) {
        daimonOracle = IDaimonOracle(_daimonOracle);
    }

    /// @param _terms ABI-encoded (address golem, int8 minPleasure,
    ///        int8 maxArousal, int8 minDominance, bytes4[] restrictedSelectors)
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 /* _delegationHash */
    ) external view override returns (bool) {
        (
            address golem,
            int8 minPleasure,
            int8 maxArousal,
            int8 minDominance,
            bytes4[] memory restrictedSelectors
        ) = abi.decode(_terms, (address, int8, int8, int8, bytes4[]));

        (int8 p, int8 a, int8 d) = daimonOracle.getPAD(golem);
        bytes4 selector = bytes4(_action.data[:4]);

        // Check if this action is in the restricted set
        for (uint256 i = 0; i < restrictedSelectors.length; i++) {
            if (selector == restrictedSelectors[i]) {
                // Restricted action -- enforce emotional bounds
                if (p < minPleasure || a > maxArousal || d < minDominance) {
                    revert EmotionalStateRestriction(p, a, d);
                }
            }
        }

        return true;
    }
}
```

### 5.7 KnowledgeRoyaltyEnforcer

Ensures that when a Golem sells knowledge via the AgentCash marketplace, a configured royalty percentage flows back to the original author of inherited knowledge. If a Golem sells an Insight it inherited from a dead ancestor, this enforcer verifies the royalty transfer was included in the transaction batch.

---

## S6 -- VitalityOracle

The VitalityOracle is the on-chain source of truth for Golem behavioral phase. Caveat enforcers read it. The Golem's Heartbeat (9-step decision cycle running on an adaptive timer) writes to it. Phase transitions are monotonic -- a Golem can move from Thriving to Stable to Conservation to Declining to Terminal, but never backward.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";

/// @title VitalityOracle
/// @notice On-chain phase reporting for Golem behavioral state.
///         Updated by the Golem's session key via signed transactions.
///         Read by caveat enforcers to gate delegated actions.
/// @dev    Phase transitions are monotonic. Once a Golem enters a later phase,
///         it cannot return to an earlier one. This prevents gaming where a
///         Golem reports Thriving to bypass Conservation restrictions.
contract VitalityOracle is Ownable {
    enum Phase {
        Thriving,    // 0: Full autonomy, all actions allowed
        Stable,      // 1: Reduced autonomy, no replicant spawning
        Conservation,// 2: Close-only, no new positions
        Declining,   // 3: Unwind and sweep
        Terminal     // 4: Settlement and death protocol only
    }

    struct GolemState {
        Phase phase;
        uint64 lastUpdated;
        uint64 survivalScore; // 0-10000 (basis points, 10000 = 1.0)
        bool registered;
    }

    /// @dev golem session key address => state
    mapping(address => GolemState) public golems;

    /// @dev Authorized updaters (Golem session keys). Set by the factory
    ///      at registration time and revoked at death.
    mapping(address => bool) public authorizedUpdaters;

    event PhaseTransition(
        address indexed golem,
        Phase fromPhase,
        Phase toPhase,
        uint64 survivalScore
    );
    event GolemRegistered(address indexed golem, address indexed owner);
    event GolemDeregistered(address indexed golem);

    error NotAuthorized(address caller);
    error PhaseNotMonotonic(Phase current, Phase proposed);
    error GolemNotFound(address golem);

    constructor() Ownable(msg.sender) {}

    /// @notice Register a new Golem. Called by the provisioning pipeline.
    /// @param golem The Golem's session key address.
    function registerGolem(address golem) external onlyOwner {
        golems[golem] = GolemState({
            phase: Phase.Thriving,
            lastUpdated: uint64(block.timestamp),
            survivalScore: 10000, // starts at 1.0
            registered: true
        });
        authorizedUpdaters[golem] = true;
        emit GolemRegistered(golem, msg.sender);
    }

    /// @notice Update phase. Called by the Golem's heartbeat.
    ///         Phase must be >= current phase (monotonic).
    /// @param newPhase The proposed new phase.
    /// @param survivalScore Current survival score (0-10000).
    function updatePhase(
        Phase newPhase,
        uint64 survivalScore
    ) external {
        if (!authorizedUpdaters[msg.sender]) revert NotAuthorized(msg.sender);

        GolemState storage state = golems[msg.sender];
        if (!state.registered) revert GolemNotFound(msg.sender);
        if (newPhase < state.phase) {
            revert PhaseNotMonotonic(state.phase, newPhase);
        }

        Phase oldPhase = state.phase;
        state.phase = newPhase;
        state.lastUpdated = uint64(block.timestamp);
        state.survivalScore = survivalScore;

        if (newPhase != oldPhase) {
            emit PhaseTransition(msg.sender, oldPhase, newPhase, survivalScore);
        }
    }

    /// @notice Deregister a dead Golem. Called by the death protocol.
    function deregisterGolem(address golem) external onlyOwner {
        delete authorizedUpdaters[golem];
        golems[golem].registered = false;
        emit GolemDeregistered(golem);
    }

    /// @notice Read current phase. Used by caveat enforcers.
    /// @return registered Whether the Golem is registered.
    /// @return phase Current behavioral phase.
    function getPhase(address golem)
        external
        view
        returns (bool registered, uint8 phase)
    {
        GolemState memory state = golems[golem];
        return (state.registered, uint8(state.phase));
    }

    /// @notice Read full state. Used by Portal dashboard.
    function getState(address golem)
        external
        view
        returns (GolemState memory)
    {
        return golems[golem];
    }
}
```

---

## S7 -- Session key lifecycle

The session key is an ephemeral secp256k1 keypair. It exists only in process memory. It is generated at boot, used for the Golem's lifetime, and zeroed at death.

```rust
use alloy::signers::local::LocalSigner;
use alloy::primitives::Address;
use zeroize::Zeroize;

/// Ephemeral session key for Delegation custody mode.
/// Generated fresh at Golem boot. Never written to disk.
/// Zeroed from memory during the death protocol.
pub struct SessionKey {
    /// The signing key. Held in-memory only.
    signer: LocalSigner,
    /// Derived address (the delegate in the delegation tree).
    pub address: Address,
    /// When this key was generated (Unix seconds).
    pub created_at: u64,
    /// Maximum validity (delegation's MortalityTimeWindow endTime).
    pub expires_at: u64,
    /// Whether this key has been zeroed (post-death).
    zeroed: bool,
}

impl SessionKey {
    /// Generate a new ephemeral session key.
    /// Called once during Golem provisioning.
    pub fn generate(expires_at: u64) -> Result<Self> {
        let signer = LocalSigner::random();
        let address = signer.address();

        Ok(Self {
            signer,
            address,
            created_at: now_unix(),
            expires_at,
            zeroed: false,
        })
    }

    /// Sign arbitrary data with the session key.
    /// Fails if the key has been zeroed.
    pub async fn sign(&self, data: &[u8]) -> Result<alloy::primitives::Bytes> {
        if self.zeroed {
            return Err(Error::SessionKeyZeroed);
        }
        let signature = self.signer.sign_message(data).await?;
        Ok(signature.as_bytes().into())
    }

    /// Check if the session key has expired.
    pub fn is_expired(&self) -> bool {
        now_unix() > self.expires_at
    }

    /// Zero the key material. Called during death protocol step 9.
    /// After this call, all signing operations fail.
    pub fn zero(&mut self) {
        // Zeroize is a trait from the `zeroize` crate that overwrites
        // memory with zeros. The LocalSigner's inner key bytes are
        // zeroed, preventing recovery from memory dumps.
        let key_bytes = self.signer.credential().to_bytes();
        let mut mutable_bytes = key_bytes.to_vec();
        mutable_bytes.zeroize();
        self.zeroed = true;
    }
}

impl Drop for SessionKey {
    fn drop(&mut self) {
        if !self.zeroed {
            self.zero();
        }
    }
}
```

### 7.1 Lifecycle stages

| Stage | What happens | Duration |
| --- | --- | --- |
| Generation | `SessionKey::generate()` during provisioning step 2 | Instantaneous |
| Registration | Session key address is included in the delegation's `delegate` field | Part of provisioning step 4 |
| Injection | Key material injected into Golem VM via Fly.io per-machine env var | Part of provisioning step 7 |
| Active use | Golem signs transactions using the session key | Golem's entire lifetime |
| Rotation | Not supported in v1. Session key is valid for the delegation's lifetime. | N/A |
| Zeroing | `SessionKey::zero()` during death protocol step 9 | Instantaneous |

### 7.2 Key isolation

The session key private material never leaves the Golem's process memory:

- **Not on disk:** Injected via env var (Fly.io per-machine, not app-wide). Env vars live in process memory, not filesystem.
- **Not in logs:** The runtime's logger redacts any string matching the key pattern.
- **Not in Grimoire:** Session key material is explicitly excluded from Episode serialization.
- **Not in Styx:** The WebSocket connection to Styx authenticates with a signed challenge, not the raw key.
- **Not readable via API:** Fly.io machine env vars are write-only through the Machines API. No readback endpoint.

If the key is compromised anyway, the blast radius is bounded by the delegation's caveat enforcers. The attacker can only execute actions the Golem was permitted to execute, subject to spending caps, allowed targets, time windows, and phase restrictions.

---

## S8 -- ERC-7715 GrantPermissionsRequest

The owner grants the Golem its delegation via ERC-7715's `wallet_grantPermissions` RPC method. MetaMask renders a human-readable permission dialog showing what the Golem can do.

```rust
use serde::{Deserialize, Serialize};
use alloy::primitives::{Address, U256};

/// ERC-7715 permission request sent to MetaMask via
/// `wallet_grantPermissions`. MetaMask renders this as a
/// human-readable permission dialog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPermissionsRequest {
    /// Chain ID for the delegation.
    #[serde(rename = "chainId")]
    pub chain_id: String,

    /// Owner's MetaMask Smart Account address.
    pub address: Address,

    /// Expiry timestamp (Unix seconds). Maps to MortalityTimeWindowEnforcer.
    pub expiry: u64,

    /// Session key public address (the delegate).
    pub signer: PermissionSigner,

    /// Requested permissions.
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSigner {
    #[serde(rename = "type")]
    pub signer_type: String, // "key"
    pub data: SignerData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerData {
    /// Session key address (hex).
    pub id: String,
}

/// Individual permission types supported by the Golem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Permission {
    /// Recurring ERC-20 allowance (e.g., USDC daily cap).
    #[serde(rename = "erc20-recurring-allowance")]
    Erc20RecurringAllowance {
        data: Erc20RecurringAllowanceData,
    },

    /// Recurring native token allowance (e.g., ETH for gas).
    #[serde(rename = "native-token-recurring-allowance")]
    NativeTokenRecurringAllowance {
        data: NativeTokenRecurringAllowanceData,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erc20RecurringAllowanceData {
    /// ERC-20 token address (e.g., USDC on Base).
    pub token: Address,
    /// Maximum amount per period (in token decimals).
    pub allowance: U256,
    /// Period duration in seconds (86400 = daily).
    pub period: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeTokenRecurringAllowanceData {
    /// Maximum ETH per period (in wei).
    pub allowance: U256,
    /// Period duration in seconds.
    pub period: u64,
}
```

### 8.1 Example: vault manager Golem

A vault manager Golem that trades USDC and ETH on Base with a $1,000/day spending cap:

```rust
fn build_vault_manager_permissions(
    owner: Address,
    session_key: Address,
    vault: Address,
    runway_days: u64,
) -> GrantPermissionsRequest {
    let usdc_base = address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
    let now = now_unix();

    GrantPermissionsRequest {
        chain_id: "0x2105".into(), // Base (8453)
        address: owner,
        expiry: now + (runway_days * 86400),
        signer: PermissionSigner {
            signer_type: "key".into(),
            data: SignerData {
                id: format!("{:?}", session_key),
            },
        },
        permissions: vec![
            Permission::Erc20RecurringAllowance {
                data: Erc20RecurringAllowanceData {
                    token: usdc_base,
                    allowance: U256::from(1_000_000_000u64), // $1000 (6 decimals)
                    period: 86400, // daily
                },
            },
            Permission::NativeTokenRecurringAllowance {
                data: NativeTokenRecurringAllowanceData {
                    allowance: U256::from(100_000_000_000_000_000u64), // 0.1 ETH
                    period: 86400, // daily
                },
            },
        ],
    }
}
```

---

## S9 -- Strategy-to-permission compiler

The strategy-to-permission compiler reads STRATEGY.md and generates the minimal delegation required to execute it. This prevents over-permissioning -- a DCA-only strategy does not get LP permissions.

```rust
use std::collections::HashSet;

/// Parses STRATEGY.md content and produces the minimal permission set.
/// Called during provisioning to construct the GrantPermissionsRequest.
pub fn strategy_to_permission_request(
    strategy_text: &str,
    owner: Address,
    session_key: Address,
    chain_id: u64,
    runway_days: u64,
) -> Result<GrantPermissionsRequest> {
    let capabilities = detect_capabilities(strategy_text);
    let mut permissions = Vec::new();

    // Always grant gas allowance (every Golem needs gas)
    permissions.push(Permission::NativeTokenRecurringAllowance {
        data: NativeTokenRecurringAllowanceData {
            allowance: U256::from(50_000_000_000_000_000u64), // 0.05 ETH/day
            period: 86400,
        },
    });

    // USDC allowance scaled by strategy complexity
    let daily_usdc_cap = estimate_daily_spend(&capabilities, strategy_text);
    let usdc = get_usdc_address(chain_id)?;
    permissions.push(Permission::Erc20RecurringAllowance {
        data: Erc20RecurringAllowanceData {
            token: usdc,
            allowance: U256::from(daily_usdc_cap),
            period: 86400,
        },
    });

    Ok(GrantPermissionsRequest {
        chain_id: format!("0x{:x}", chain_id),
        address: owner,
        expiry: now_unix() + (runway_days * 86400),
        signer: PermissionSigner {
            signer_type: "key".into(),
            data: SignerData {
                id: format!("{:?}", session_key),
            },
        },
        permissions,
    })
}

/// Detect strategy capabilities from natural language text.
/// Returns a set of capability tags used for permission scoping.
fn detect_capabilities(strategy_text: &str) -> HashSet<Capability> {
    let lower = strategy_text.to_lowercase();
    let mut caps = HashSet::new();

    // Token patterns
    if lower.contains("swap") || lower.contains("trade") || lower.contains("dca") {
        caps.insert(Capability::Swap);
    }
    if lower.contains("liquidity") || lower.contains("lp") || lower.contains("pool") {
        caps.insert(Capability::ProvideLiquidity);
    }
    if lower.contains("vault") || lower.contains("deposit") || lower.contains("yield") {
        caps.insert(Capability::VaultManagement);
    }
    if lower.contains("bridge") || lower.contains("cross-chain") {
        caps.insert(Capability::Bridge);
    }
    if lower.contains("replicant") || lower.contains("sub-agent") {
        caps.insert(Capability::SpawnReplicants);
    }
    if lower.contains("limit order") || lower.contains("uniswapx") {
        caps.insert(Capability::LimitOrders);
    }

    caps
}

/// Estimate daily USDC spend based on detected capabilities.
/// Returns amount in USDC smallest units (6 decimals).
fn estimate_daily_spend(
    capabilities: &HashSet<Capability>,
    strategy_text: &str,
) -> u64 {
    // Default daily spending limit: configurable per-Golem
    // (recommended starting value: $50-200/day based on portfolio size)
    let mut base_spend: u64 = 100_000_000; // configurable baseline

    if capabilities.contains(&Capability::VaultManagement) {
        base_spend = base_spend.max(1_000_000_000); // $1000/day for vault ops
    }
    if capabilities.contains(&Capability::ProvideLiquidity) {
        base_spend = base_spend.max(500_000_000); // $500/day for LP
    }
    if capabilities.contains(&Capability::Bridge) {
        base_spend += 200_000_000; // +$200 for bridge operations
    }

    // Parse explicit budget mentions: "budget: $X/day"
    if let Some(explicit) = parse_budget_from_text(strategy_text) {
        return explicit;
    }

    base_spend
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Capability {
    Swap,
    ProvideLiquidity,
    VaultManagement,
    Bridge,
    SpawnReplicants,
    LimitOrders,
}
```

---

## S10 -- Feed-the-Golem: delegation extension

When a Golem's runway is low, the owner can extend it without creating a new delegation. The flow:

1. Portal detects low runway (< 7 days remaining based on survival score)
2. Portal displays "Feed your Golem" CTA with recommended extension amount
3. Owner clicks, MetaMask prompts a new `wallet_grantPermissions` with updated `expiry`
4. New delegation replaces the old one (same session key address, higher salt)
5. VitalityOracle is updated if the phase had degraded due to funding pressure
6. The Golem's MortalityTimeWindowEnforcer terms are updated on-chain

The old delegation is not explicitly revoked -- it is superseded by the new one with a higher salt. The DelegationManager uses the most recent valid delegation.

In Embedded (Privy) mode, "feeding" is a USDC transfer via Permit2. In Delegation mode, it is a signature. No funds move.

---

## S11 -- Cross-agent delegations

Golems can sub-delegate to other Golems for cooperative strategies. The delegation tree enforces attenuation: a sub-delegation never exceeds the parent.

### 11.1 Golem-to-Golem sub-delegation

Golem Alpha (vault manager) sub-delegates read-only access to Golem Beta (market analyst):

```rust
/// Build a cross-Golem sub-delegation.
/// The sub-delegation is bounded by the parent delegation's caveats
/// AND the additional caveats specified here.
pub fn build_cross_golem_delegation(
    parent_golem: Address,
    child_golem: Address,
    parent_delegation_hash: B256,
    max_budget_usd: u64,
    max_lifespan_seconds: u64,
    allowed_methods: Vec<[u8; 4]>,
) -> DelegationNode {
    DelegationNode {
        delegator: parent_golem,
        delegate: child_golem,
        authority: Some(parent_delegation_hash),
        caveats: vec![
            CaveatEnforcer {
                enforcer: REPLICANT_BUDGET_ENFORCER,
                terms: encode_replicant_budget(max_budget_usd, max_lifespan_seconds),
            },
            CaveatEnforcer {
                enforcer: ALLOWED_METHODS_ENFORCER,
                terms: encode_allowed_methods(&allowed_methods),
            },
            CaveatEnforcer {
                enforcer: LIMITED_CALLS_ENFORCER,
                terms: encode_limited_calls(1000),
            },
        ],
        signature: Bytes::default(), // Filled by the parent Golem's session key
        salt: generate_salt(),
    }
}
```

### 11.2 Coordination Council (multi-sig threshold)

For high-value cooperative actions (e.g., coordinated rebalancing across multiple vaults), a 2-of-3 threshold delegation prevents unilateral action:

```
Owner
  +-- D1: Coordination Council (2-of-3 threshold)
      Delegates: [Golem Alpha, Golem Beta, Golem Gamma]
      Caveats: [ThresholdEnforcer(2, 3), MaxValue($10000), AllowedTargets([vault1, vault2])]
```

The `ThresholdEnforcer` is a standard MetaMask caveat that requires `k` of `n` delegates to co-sign before execution. Each delegate signs independently. The DelegationManager collects signatures and executes only when the threshold is met.

---

## S12 -- Death settlement

In Delegation mode, death settlement is a non-event for the owner's funds.

| Phase | What happens | Owner action required |
| --- | --- | --- |
| Declining (phase 3) | Golem unwinds positions, withdraws to owner's Smart Account | None |
| Terminal (phase 4) | Golem executes settlement-only actions, writes BardoManifest | None |
| Dead | Session key zeroed, delegation expires, VitalityOracle deregistered | None (optionally: revoke delegation from MetaMask) |

Funds were never transferred. There is nothing to sweep. The delegation expires on its own via MortalityTimeWindowEnforcer. The owner can optionally revoke it early from MetaMask for hygiene.

Compare with Embedded (Privy) mode death settlement:

1. Golem sweeps remaining USDC from Privy wallet to owner's Main Wallet
2. If the Golem is unresponsive, the owner sweeps manually via Portal dashboard
3. If Portal is down, the owner sweeps via direct Privy API call
4. Last resort: Privy support recovers funds from the TEE

Delegation mode eliminates all four of these failure paths.

---

## S13 -- Provisioning comparison

| Step | Delegation mode | Embedded (Privy) mode |
| --- | --- | --- |
| 1. Validate manifest | Same | Same |
| 2. Create wallet | Generate session keypair (local, instant) | Privy API call (2-5s) |
| 3. Approve Permit2 | **Skip** (no token transfer) | Check + approve (0-1 tx) |
| 4. Fund wallet | ERC-7715 signature (one off-chain sign) | Permit2 SignatureTransfer (1 sign + 1 tx) |
| 5. Register identity | Same | Same |
| 6. Provision VM | Same | Same |
| 7. Deploy Golem | Inject session key + delegation config | Inject Privy wallet config |
| 8. Start heartbeat | Same | Same |
| **Total time** | **15-30s** | **30-60s** |
| **On-chain txs** | **0-1** (identity registration only) | **1-3** (approve + transfer + identity) |
| **User signatures** | **1** (ERC-7715 grant) | **2** (Permit2 + approval) |

The 4-8x speed improvement comes from eliminating on-chain transactions. A signature takes ~2 seconds (user reads MetaMask dialog, clicks approve). An on-chain transaction takes 10-15 seconds (block confirmation on Base).

---

## S14 -- Testing strategy

### 14.1 Unit tests (golem-chain crate)

- Session key generation, signing, zeroing
- Delegation tree construction and validation
- Strategy-to-permission compiler: known strategy texts produce expected permissions
- Permission request serialization/deserialization

### 14.2 Integration tests (Anvil fork)

- Deploy VitalityOracle, GolemPhaseEnforcer, MortalityTimeWindowEnforcer to Anvil
- Register a Golem, update phase, verify enforcer behavior
- Attempt out-of-phase action, verify revert
- Attempt action after MortalityTimeWindow expiry, verify revert
- DreamModeEnforcer: set dreaming, attempt action, verify revert
- VaultNAVEnforcer: attempt over-limit transaction, verify revert

### 14.3 End-to-end tests (MetaMask test dApp)

- Full provisioning flow with MetaMask Flask (dev build)
- ERC-7715 permission grant and subsequent delegated execution
- Delegation revocation from MetaMask side
- Feed-the-Golem extension flow
- Cross-agent sub-delegation

---

## Permit2 + ERC-7710 Interaction

Permit2 handles token approvals (user -> Golem). ERC-7710 handles delegation (user -> Golem for specific capabilities). Both are required: Permit2 for the money, ERC-7710 for the authority.

---

## Cross-references

- MetaMask MCP tools and Snap: [../07-tools/18-tools-metamask.md](../07-tools/18-tools-metamask.md) -- MCP tool definitions for the MetaMask Snap, covering the tool surface the Golem uses to request permissions and execute delegated transactions
- Wallet architecture: [../07-tools/22-wallets.md](../07-tools/22-wallets.md) -- the three custody modes (Delegation, Embedded, LocalKey), key management lifecycle, and wallet provisioning across all integrations
- Provisioning pipeline: [../01-golem/07-provisioning.md](../01-golem/07-provisioning.md) -- how a Golem is created from config through VM deployment to first heartbeat, including the session key generation that this document specifies
- Funding sources: [../01-golem/08-funding.md](../01-golem/08-funding.md) -- vault fee revenue, x402 credit conversion, and burn rate estimation that determine how long the Golem's delegated spending authority remains useful
- Death protocol: [../02-mortality/06-thanatopsis.md](../02-mortality/06-thanatopsis.md) -- the four-phase Thanatopsis shutdown (Acceptance, Settlement, Reflection, Legacy) where MortalityTimeWindowEnforcer blocks post-death delegated actions
- Daimon (affect engine): [../03-daimon/00-overview.md](../03-daimon/00-overview.md) -- the PAD emotional state engine whose outputs feed the EmotionalStateEnforcer caveat and influence which delegated actions the Golem attempts
- Dream cycles: [../05-dreams/00-overview.md](../05-dreams/00-overview.md) -- the three-phase dream cycle whose DreamMode state is checked by the DreamModeEnforcer caveat to restrict actions during consolidation
- Safety architecture: [../10-safety/00-overview.md](../10-safety/00-overview.md) -- PolicyCage, ActionPermit, and Warden systems that work alongside MetaMask caveats to enforce safety constraints on all Golem actions
- Integration bounties overview: [00-overview.md](00-overview.md) -- synthesis of all eleven hackathon bounties, dependency graph, and build order showing MetaMask as a Phase 1 structural component
