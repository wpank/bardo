# Bardo Tools -- MetaMask deep integration [SPEC]

**Version:** 4.0.0
**Last Updated:** 2026-03-14

> **Crate**: `bardo-tools` (permission tools), `bardo-snap` (MetaMask Snap) | **Prerequisites**: [01-architecture.md](01-architecture.md) (ToolDef pattern, trust tiers, capability tokens, safety hooks, and profiles), [22-wallets.md](22-wallets.md) (wallet architecture, three custody modes, capability token flow), [20-config.md](20-config.md) (environment variables, TOML config schema, API key tiers)
>
> Deep MetaMask integration across three layers: SDK-based connection, ERC-7715 Advanced Permissions for Golem runway subscriptions, and a MetaMask Snap that surfaces every on-chain action inside the wallet. MetaMask is the Delegation custody mode's primary wallet surface. The integration completes Bardo's "consent boundary / execution boundary" architecture -- MetaMask is the human-facing consent surface; Bardo runs the machine.

---

> **Reader orientation:** This document specifies the MetaMask deep integration in `bardo-tools`, covering three layers: SDK-based wallet connection, ERC-7715 advanced permissions for Golem runway subscriptions, and a MetaMask Snap that surfaces on-chain actions inside the wallet UI. MetaMask is the primary wallet surface for the Delegation custody mode. You should be familiar with MetaMask Snaps, ERC-7715, and smart account delegation patterns. Bardo-specific terms are defined inline on first use; for a full glossary see [`prd2/shared/glossary.md`](../shared/glossary.md).

## Design principle

MetaMask is the consent boundary. Bardo is the execution boundary.

The owner never hands their signing keys to an agent. They hand a bounded permission object -- a specific USDC daily cap, a specific duration, a specific receiver -- and MetaMask's UI enforces that those parameters are legible before the owner signs. Once signed, the Golem operates within those bounds autonomously. The Warden (optional, deferred) and PolicyCage enforce Bardo-side limits; the Advanced Permission enforces the wallet-side limit. Both have to agree before any funds move.

This means the integration is optional and gracefully degradable. A Golem can run on an Embedded (Privy) wallet or a LocalKey with no MetaMask connection. MetaMask adds the human consent layer for the Delegation custody mode. Portal detects MetaMask presence via EIP-6963 and activates the connection flow; if absent, nothing breaks.

The full demo arc: owner funds a Golem via a Runway subscription (USDC/day via ERC-7715 -> MetaMask signs a legible permission dialog) -> Snap installs and shows Golem status in-wallet -> every Warden announcement (when Warden is deployed; optional, deferred) appears as a transaction insight -> when the Golem enters terminal phase, `metamask_revoke_permissions` fires, the Snap emits a native notification, and Portal shows a succession CTA. Human-legible from start to death.

---

## Three custody modes

MetaMask Delegation is the recommended custody mode. It coexists with two alternatives. The owner selects one at provisioning time.

| Mode | Trust Model | Funds Location | Death Settlement |
| --- | --- | --- | --- |
| **Delegation** (recommended) | On-chain caveats via ERC-7710/7715 | Owner's MetaMask Smart Account | No sweep needed -- funds were never transferred |
| **Embedded** (Privy, legacy) | Off-chain TEE policy | Privy server wallet | Sweep required |
| **LocalKey** (dev only) | On-chain delegation bounds | Owner's Smart Account or local EOA | Delegation expires |

Delegation mode eliminates two on-chain transactions (approve + transfer) from provisioning. The fund transfer is replaced by a single signature.

---

## Delegation tree

The permission structure is a tree rooted at the owner's MetaMask Smart Account. Sub-delegations attenuate strictly -- a child can never exceed its parent's authority. The DelegationManager enforces this invariant on-chain.

```
Owner (MetaMask Smart Account -- ERC-7710)
  |
  +-- Delegation D1: Golem Alpha (vault manager)
  |   +-- Caveats: [maxSpend($1000/day), approvedAssets([USDC,WETH,WBTC]),
  |   |             maxDrawdown(15%), timeWindow(30d)]
  |   |
  |   +-- Sub-Delegation D1.1: Replicant Alpha-1 (hypothesis tester)
  |   |   +-- Caveats: [maxSpend($50), maxLifespan(24h), readOnly, noSubDelegation]
  |   |   +-- (auto-expires after 24h)
  |   |
  |   +-- Sub-Delegation D1.2: Sleepwalker Observer
  |   |   +-- Caveats: [readOnly, noTransfers, oraclePublishOnly]
  |   |   +-- (no expiry -- observation is always safe)
  |   |
  |   +-- Phase-Gated Delegations:
  |       +-- D1.P1 (Thriving):     [fullTrading, maxPosition(30%), replicantSpawning]
  |       +-- D1.P2 (Stable):       [fullTrading, maxPosition(20%), noReplicantSpawning]
  |       +-- D1.P3 (Conservation): [closeOnly, noNewPositions, withdrawAllowed]
  |       +-- D1.P4 (Declining):    [unwindOnly, sweepToOwner]
  |       +-- D1.P5 (Terminal):     [settlementOnly, deathProtocol]
  |
  +-- Delegation D2: Golem Beta (DCA executor)
      +-- Caveats: [maxSpend($100/week), onlySwap(USDC->ETH), cronOnly(weekly)]
      +-- (simpler strategy, simpler delegation)
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationNode {
    pub delegator: Address,
    pub delegate: Address,
    pub authority: Option<[u8; 32]>,
    pub caveats: Vec<CaveatEnforcer>,
    pub signature: Bytes,
    pub salt: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationTree {
    pub root: Address,
    pub nodes: HashMap<[u8; 32], DelegationNode>,
    /// Every edge satisfies: child.permissions is a subset of parent.permissions.
    pub edges: Vec<([u8; 32], [u8; 32])>,
}
```

---

## Seven caveat enforcers

Each is a deployed Solidity contract implementing `ICaveatEnforcer`. The DelegationManager calls `enforceCaveat()` before executing any delegated action.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import { ICaveatEnforcer } from "@metamask/delegation-framework/ICaveatEnforcer.sol";

/// @title ICaveatEnforcer
/// @notice Interface that all Bardo caveat enforcers implement.
///         The DelegationManager calls enforceCaveat() before executing
///         any delegated action. Revert = action blocked.
interface ICaveatEnforcer {
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external view returns (bool);
}
```

### GolemPhaseEnforcer

Restricts actions by behavioral phase. Reads current phase from the VitalityOracle. A Golem in Conservation cannot open new positions; a Golem in Terminal can only settle.

```solidity
/// @title GolemPhaseEnforcer
/// @notice Enforces behavioral phase constraints on Golem delegations.
///         Reads the Golem's current phase from the VitalityOracle and
///         restricts actions based on the phase.
contract GolemPhaseEnforcer is ICaveatEnforcer {
    /// @notice Oracle that reports the Golem's current behavioral phase
    IVitalityOracle public immutable vitalityOracle;

    /// @notice Allowed action types per phase (bitmap)
    mapping(uint8 => mapping(bytes4 => bool)) public phasePermissions;

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

    /// @notice Validates action against the Golem's current behavioral phase
    /// @param _terms ABI-encoded Golem address
    /// @param _action The action being executed
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external view override returns (bool) {
        uint8 currentPhase = vitalityOracle.getPhase(
            abi.decode(_terms, (address))
        );
        bytes4 actionSelector = bytes4(_action.data[:4]);

        require(
            phasePermissions[currentPhase][actionSelector],
            "Action not allowed in current phase"
        );

        return true;
    }
}
```

### MortalityTimeWindowEnforcer

Time-locked delegation. When `block.timestamp > endTime`, the delegation is dead. Maps to the Golem's projected lifespan.

```solidity
/// @title MortalityTimeWindowEnforcer
/// @notice Auto-expires delegation at the Golem's projected death time.
///         When block.timestamp exceeds endTime, all executions revert.
contract MortalityTimeWindowEnforcer is ICaveatEnforcer {
    /// @param _terms ABI-encoded (uint256 startTime, uint256 endTime)
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external view override returns (bool) {
        (uint256 startTime, uint256 endTime) = abi.decode(_terms, (uint256, uint256));

        require(block.timestamp >= startTime, "Delegation not yet active");
        require(block.timestamp <= endTime, "Delegation expired -- Golem is dead");

        return true;
    }
}
```

### DreamModeEnforcer

On-chain structural atonia. Blocks all write operations during dream cycles. Even if a code bug fires an action during a dream, the enforcer blocks it. Defense in depth.

```solidity
/// @title DreamModeEnforcer
/// @notice Blocks execution during Golem dream cycles (structural atonia).
///         Even if the Golem's runtime has a bug that fires an action during
///         a dream, this on-chain enforcer blocks it.
contract DreamModeEnforcer is ICaveatEnforcer {
    IDreamOracle public immutable dreamOracle;

    constructor(address _dreamOracle) {
        dreamOracle = IDreamOracle(_dreamOracle);
    }

    /// @param _terms ABI-encoded Golem address
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external view override returns (bool) {
        address golem = abi.decode(_terms, (address));
        require(!dreamOracle.isDreaming(golem), "Golem is dreaming -- no execution");
        return true;
    }
}
```

### VaultNAVEnforcer

Limits actions based on vault NAV percentage. Blocks transactions if the requested value exceeds a configured fraction of the vault's net asset value.

```solidity
/// @title VaultNAVEnforcer
/// @notice Blocks actions if the value exceeds a percentage of vault NAV.
///         Acts as an on-chain circuit breaker for vault operations.
contract VaultNAVEnforcer is ICaveatEnforcer {
    /// @param _terms ABI-encoded (address vaultAddress, uint16 maxNavPct)
    ///        maxNavPct is in basis points (e.g., 1000 = 10%)
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external view override returns (bool) {
        (address vault, uint16 maxNavPct) = abi.decode(_terms, (address, uint16));
        uint256 totalAssets = IERC4626(vault).totalAssets();
        uint256 maxValue = (totalAssets * maxNavPct) / 10000;

        require(_action.value <= maxValue, "Action value exceeds vault NAV limit");
        return true;
    }
}
```

### ReplicantBudgetEnforcer

Caps Replicant sub-delegation spending and lifespan. Prevents child delegations from exceeding parent-allocated budgets.

```solidity
/// @title ReplicantBudgetEnforcer
/// @notice Caps Replicant spending at the parent's sub-allocated budget.
///         Enforces both USD-equivalent limits and lifespan bounds.
contract ReplicantBudgetEnforcer is ICaveatEnforcer {
    /// delegationHash => cumulative spend in USD (18 decimals)
    mapping(bytes32 => uint256) public cumulativeSpend;

    /// @param _terms ABI-encoded (uint64 maxBudgetUsd, uint64 maxLifespanSeconds)
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external override returns (bool) {
        (uint64 maxBudgetUsd, uint64 maxLifespanSeconds) = abi.decode(
            _terms, (uint64, uint64)
        );
        uint256 actionValueUsd = estimateActionValueUsd(_action);

        require(
            cumulativeSpend[_delegationHash] + actionValueUsd <= uint256(maxBudgetUsd) * 1e18,
            "Replicant budget exceeded"
        );

        cumulativeSpend[_delegationHash] += actionValueUsd;
        return true;
    }
}
```

### MaxSlippageEnforcer

Bounds acceptable slippage on swap transactions. Reads the swap calldata and verifies `minAmountOut` against a configured maximum slippage tolerance.

```solidity
/// @title MaxSlippageEnforcer
/// @notice Ensures swap transactions maintain minimum output amounts.
/// @dev Reads the swap calldata and verifies minAmountOut against
///      a configured maximum slippage tolerance.
contract MaxSlippageEnforcer is ICaveatEnforcer {
    /// @param _terms ABI-encoded uint16 maxSlippageBps
    function enforceCaveat(
        bytes calldata _terms,
        Action calldata _action,
        bytes32 _delegationHash
    ) external view override returns (bool) {
        uint16 maxSlippageBps = abi.decode(_terms, (uint16));

        bytes4 selector = bytes4(_action.data[:4]);
        require(
            selector == UNISWAP_EXACT_INPUT_SELECTOR ||
            selector == UNISWAP_EXACT_INPUT_SINGLE_SELECTOR,
            "Only swap functions allowed"
        );

        uint256 minAmountOut = extractMinAmountOut(_action.data);
        uint256 expectedOut = getExpectedOutput(_action.data);
        uint256 actualSlippage = ((expectedOut - minAmountOut) * 10000) / expectedOut;

        require(actualSlippage <= maxSlippageBps, "Slippage exceeds delegation limit");
        return true;
    }
}
```

### DailySpendLimitEnforcer

Rolling 24-hour spending cap across all executions under a delegation.

```solidity
/// @title DailySpendLimitEnforcer
/// @notice Enforces a rolling 24-hour spending cap across all executions.
/// @dev Tracks cumulative spend in a mapping, resets every 24 hours.
contract DailySpendLimitEnforcer is ICaveatEnforcer {
    /// delegationHash => day => amount spent
    mapping(bytes32 => mapping(uint256 => uint256)) public dailySpend;

    /// @param _terms ABI-encoded uint256 dailyLimitUsd
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
            "Daily spend limit exceeded"
        );

        dailySpend[_delegationHash][today] += actionValueUsd;
        return true;
    }
}
```

### Caveat enforcer summary

| Enforcer | Purpose | State |
| --- | --- | --- |
| `GolemPhaseEnforcer` | Restrict actions by behavioral phase | View (reads VitalityOracle) |
| `MortalityTimeWindowEnforcer` | Auto-expire delegation at death | View (reads block.timestamp) |
| `DreamModeEnforcer` | Block execution during dream cycles | View (reads DreamOracle) |
| `VaultNAVEnforcer` | Cap actions by vault NAV percentage | View (reads vault totalAssets) |
| `ReplicantBudgetEnforcer` | Cap Replicant spending + lifespan | Stateful (tracks cumulative spend) |
| `MaxSlippageEnforcer` | Bound swap slippage tolerance | View (reads calldata) |
| `DailySpendLimitEnforcer` | Rolling 24h spending cap | Stateful (tracks daily totals) |

The Rust-side enum representation for `_terms` encoding:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaveatEnforcer {
    GolemPhase { vitality_oracle: Address, golem_address: Address },
    MortalityTimeWindow { start_time: u64, end_time: u64 },
    DreamMode { dream_oracle: Address, golem_address: Address },
    VaultNAV { vault_address: Address, max_nav_pct: u16 },
    ReplicantBudget { max_budget_usd: u64, max_lifespan_seconds: u64 },
    MaxSlippage { max_slippage_bps: u16 },
    DailySpendLimit { daily_limit_usd: u64 },
}
```

---

## Session key lifecycle

A session key is an ephemeral keypair that signs UserOperations on behalf of the delegation. It has no independent authority -- worthless without a valid delegation to redeem.

### Boot

1. Generate fresh secp256k1 or P-256 keypair
2. Hosted mode: store in process memory (never to disk). Local mode: persist to `$GOLEM_DATA/session-key.json` (encrypted at rest)
3. Load delegation from provisioning config
4. Register session key address with bundler

### Operation

- Session key signs UserOps that redeem the delegation
- DelegationManager validates caveats on every execution
- If key is compromised: attacker bounded by caveats
- Budget tracked locally; reconciled against on-chain state when estimates diverge
- Every UserOperation is counted against the rotation policy

### Compromise

If the session key leaks, the blast radius is bounded:
- Attacker can spend at most the remaining daily cap
- Attacker can only interact with approved target contracts
- Attacker cannot grant new permissions (requires owner signature)
- Attacker cannot access the owner's full wallet (only the delegation scope)
- Owner revokes the delegation from MetaMask -- no Golem cooperation needed

### Crash

If the VM crashes, the key is lost but the delegation remains valid. A new Golem instance boots, generates a new key, and the owner re-delegates (or the platform auto-delegates if pre-authorized).

### Death

Session key is zeroized from memory. Delegation auto-expires via MortalityTimeWindowEnforcer. No fund sweep needed -- funds were never transferred. Open positions close back to the owner's address (that is where execution always happened).

### Rotation policy

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionKey {
    pub address: Address,
    pub granted_at: u64,
    pub expires_at: u64,
    pub remaining_budget_usd: f64,
    pub rotation_policy: RotationPolicy,
    pub operations_signed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    pub max_operations: u64,       // default: 10,000
    pub max_age_seconds: u64,      // default: 7 days
    pub budget_floor_fraction: f64, // default: 0.1
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

When `should_rotate()` returns true: generate new keypair, request fresh delegation, atomically switch. Old key is zeroized from memory. Brief write-action gap is acceptable -- the heartbeat can skip a tick.

### Session key events

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionKeyEvent {
    Generated { address: Address, expires_at: u64 },
    DelegationGranted { delegation_hash: [u8; 32], caveats: Vec<String> },
    OperationSigned { op_hash: [u8; 32], value_usd: f64, remaining_budget: f64 },
    RotationTriggered { reason: String, old_address: Address, new_address: Address },
    Revoked { cause: RevocationState },
    BudgetWarning { remaining_pct: f64, estimated_operations_left: u64 },
}
```

---

## Delegation execution flow

When the Golem needs to execute an action (e.g., a swap), the transaction is sent FROM the owner's Smart Account but SIGNED BY the Golem's session key. The DelegationManager validates the delegation chain and caveats before execution proceeds.

```
Session Key signs UserOp
    |
    v
Bundler submits to EntryPoint (ERC-4337)
    |
    v
EntryPoint calls Owner's Smart Account
    |
    v
Smart Account calls DelegationManager.redeemDelegation()
    |
    v
DelegationManager validates:
    1. Delegation signature is valid
    2. Delegate matches session key address
    3. All caveat enforcers pass (phase, time window, spend limit, etc.)
    4. Sub-delegation chain is valid (attenuation invariant)
    |
    v
Execution proceeds FROM owner's address
    |
    v
On Etherscan: owner's address is msg.sender (Golem is invisible)
```

```rust
/// Execute an action by redeeming a delegation.
/// The transaction executes FROM the owner's Smart Account.
pub async fn execute_with_delegation(
    action: &ValidatedAction,
    delegation: &SignedDelegation,
    session_key: &LocalSigner,
    bundler_client: &BundlerClient,
) -> Result<TransactionReceipt> {
    // 1. Build the execution calldata (e.g., Uniswap swap)
    let execution_calldata = encode_swap_calldata(action)?;

    // 2. Build the UserOp that redeems the delegation.
    //    Sent FROM the owner's Smart Account, SIGNED BY session key.
    let user_op = build_delegation_redemption_user_op(DelegationRedemptionParams {
        delegator: delegation.delegator,   // Owner's MetaMask Smart Account
        delegate: session_key.address(),   // Golem's session key
        delegation: delegation.clone(),
        execution: Execution {
            target: action.target,         // e.g., Uniswap Router
            value: U256::ZERO,
            call_data: execution_calldata,
        },
        mode: ExecutionMode::Single,
    })
    .await?;

    // 3. Sign the UserOp with the session key
    let signed_user_op = session_key.sign_user_operation(&user_op).await?;

    // 4. Submit via bundler (ERC-4337)
    let user_op_hash = bundler_client.send_user_operation(&signed_user_op).await?;

    // 5. Wait for confirmation
    let receipt = bundler_client
        .wait_for_user_operation_receipt(user_op_hash)
        .await?;

    Ok(receipt)
}
```

---

## Three-identity architecture

Three accounts participate in every Golem lifecycle. They have distinct roles and custody models.

```
Owner Account (MetaMask Smart Account / EIP-7702 upgraded EOA)
  |-- grants Advanced Permissions (ERC-7715) via wallet_grantPermissions
  |-- holds primary capital
  |-- sub-delegates to Session Account via EIP-7710

Session Account (Bardo-managed ephemeral key, rotated per session)
  |-- redeems permissions for execution
  |-- re-delegates bounded subset to Replicant wallets
  |-- marked invalid on Golem terminal phase

Execution Wallet (Golem's wallet -- mode-dependent)
  |-- Delegation mode: session key signs UserOps against owner's Smart Account
  |-- Embedded mode: Privy TEE wallet with PolicyCage enforcement
  |-- LocalKey mode: local keypair with on-chain delegation bounds
```

The owner grants bounded autonomy without handing full signing authority. The session account can redeem up to the remaining daily cap. It cannot exceed it because the permission object is enforced at the ERC-7715 verifier level before any calldata executes. PolicyCage then further constrains what the session account can do with those funds. Two independent enforcement layers, neither of which trusts the other.

Smart Account requirement: `wallet_grantPermissions` with `erc20-token-transfer` type requires the owner account to support ERC-7715, which requires ERC-4337 or EIP-7702. Portal detects whether the connected account is already a Smart Account; if not, it prompts the EIP-7702 upgrade path via the MetaMask Smart Accounts Kit.

---

## Connection layer

### MetaMask SDK

Portal uses `@metamask/sdk` as the primary connector. The SDK handles mobile deep links, browser extension detection, and the EIP-6963 multi-wallet discovery protocol simultaneously.

On mount, `MetaMaskProvider` initializes the SDK with Bardo's dApp metadata and begins polling for connected accounts. If MetaMask is not installed, the provider exposes `is_installed: false` and Portal renders an "Install MetaMask" CTA -- no crash, no empty state.

EIP-6963 discovery runs in parallel as a fallback. When multiple wallets are detected, Portal presents a wallet selector. MetaMask is listed first if present; its EIP-6963 `rdns` is `io.metamask`.

### Smart Account upgrade

When the owner connects a plain EOA, Portal shows an upgrade prompt: "Upgrade to Smart Account to enable Runway subscriptions." The flow calls MetaMask Smart Accounts Kit to sign an EIP-7702 authorization tuple that delegates the EOA to a canonical delegation contract. The resulting account can call `wallet_grantPermissions`.

Portal does not block on this upgrade -- all non-permission features work with a plain EOA. The upgrade is gated behind the "Fund Golem" CTA in the RunwaySubscriptionCard.

### Agent identity alignment

When an owner connects MetaMask, Portal triggers the identity alignment flow:

1. Owner signs an EIP-712 attestation binding their MM address to a Golem ID.
2. ERC-8004 `register()` call creates the on-chain identity record with the owner address as the controlling key.
3. Portal renders the identity card showing MM address, Golem name, phase, and permission health.

The owner address becomes the `owner` in the ERC-8004 record. The Execution Wallet address is recorded as the `agent` key. This mapping is what the Snap uses to associate incoming transactions with known Golems.

### Mobile deep link

Portal exposes a deep link pattern (`metamask://dapp/bardo.ai`) for mobile owners. The deep link is the required channel for urgent interventions: kill switch, permission revocation, emergency halt. Any Snap notification that involves a terminal Golem includes a deep link button.

---

## Advanced Permissions -- Runway subscriptions

### ERC-7715 permission objects

A Runway subscription is a signed ERC-7715 permission object stored in the Golem's Grimoire. Its structure:

```json
{
  "type": "erc20-token-transfer",
  "data": {
    "address": "<USDC_ADDRESS_ON_BASE>",
    "allowance": "<DAILY_CAP_IN_USDC_WEI>"
  },
  "required": true,
  "policies": [
    {
      "type": "native-token-recurring-allowance",
      "data": {
        "allowance": "<DAILY_CAP_IN_USDC_WEI>",
        "startTime": "<UNIX_TS>",
        "period": 86400
      }
    }
  ]
}
```

The permission receiver is the Session Account. The owner never sends USDC directly -- they sign a permission object that authorizes the Session Account to pull up to the daily cap per 24-hour period for the subscription duration.

### Permission tools (4 tools)

#### `metamask_request_runway`

Constructs the ERC-7715 permission params and calls `wallet_grantPermissions`. Returns the signed permission object. Before calling, checks: owner account is a Smart Account (EIP-7702 upgraded), chain is Base (where Advanced Permissions are live), and no conflicting active permission exists for the same Golem.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `golems` | `Vec<String>` | Yes | Golem IDs to grant runway |
| `daily_cap_usdc` | `f64` | Yes | USDC/day cap |
| `duration_days` | `u32` | Yes | Subscription duration in days |

```rust
#[derive(Debug, Deserialize)]
pub struct RequestRunwayParams {
    pub golems: Vec<String>,
    pub daily_cap_usdc: f64,
    pub duration_days: u32,
}

#[derive(Debug, Serialize)]
pub struct RequestRunwayResult {
    pub permission_id: String,
    pub stored_in_grimoire: bool,
    pub daily_cap: String,
    pub expires_at: u64,
}
```

On success, stores the permission object in Grimoire under key `metamask.permission.<golems[0]>`.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "metamask_request_runway",
    description: concat!(
        "Constructs an ERC-7715 permission object and calls wallet_grantPermissions on MetaMask. ",
        "Grants a USDC/day runway subscription for a Golem. ",
        "Requires owner's Smart Account on Base. Stores permission in Grimoire.",
    ),
    category: Category::Wallet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Slow,
    progress_steps: &[
        "Checking Smart Account status",
        "Validating chain (Base)",
        "Checking conflicting permissions",
        "Constructing ERC-7715 params",
        "Calling wallet_grantPermissions",
    ],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Constructs an ERC-7715 permission object and calls wallet_grantPermissions on MetaMask. Grants a USDC/day runway subscription for a Golem. Requires owner's Smart Account on Base.",
    prompt_guidelines: &[
        "thriving: Request permissions with standard daily cap and 7-30 day windows. Verify no conflicting active permission.",
        "cautious: Do not request new permissions. Existing permissions continue.",
        "declining: Do not request. Begin revocation planning.",
        "terminal: Do not request. Trigger revocation instead.",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ golems, daily_cap_usdc, duration_days }` |
| `tool:end` | `{ permission_id, stored_in_grimoire }` |

**Pi hooks:** `tool_call` (verify owner account is Smart Account, verify Base chain, check no conflicting permission). `tool_result` (store permission object in Grimoire under `metamask.permission.<golem_id>`).

---

#### `metamask_redeem_runway`

Called by the heartbeat FSM at each tick interval when the Golem needs operating funds. Reads the current permission object from Grimoire, computes the redeemable amount (daily cap minus cumulative spend today), constructs the ERC-7715 execution call, and submits via the Session Account.

The redeemable amount is phase-aware:

| Phase | Redemption multiplier |
| --- | --- |
| Thriving | 1.0x daily cap |
| Conservation | 0.5x daily cap |
| Declining | 0.1x daily cap (inference only) |
| Terminal | 0 -- triggers `metamask_revoke_permissions` instead |

```rust
#[derive(Debug, Deserialize)]
pub struct RedeemRunwayParams {
    pub golem_id: String,
}

#[derive(Debug, Serialize)]
pub struct RedeemRunwayResult {
    pub redeemed: String,
    pub remaining: String,
    pub phase: String,
}
```

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "metamask_redeem_runway",
    description: concat!(
        "Redeems operating funds from a MetaMask runway subscription. ",
        "Phase-aware: thriving = 1.0x daily cap, conservation = 0.5x, ",
        "declining = 0.1x (inference only), terminal = 0 (triggers revocation). ",
        "Called by heartbeat FSM at each tick interval.",
    ),
    category: Category::Wallet,
    capability: CapabilityTier::Write,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Medium,
    progress_steps: &[
        "Reading permission from Grimoire",
        "Computing redeemable amount",
        "Submitting ERC-7715 execution",
    ],
    sprite_trigger: SpriteTrigger::Executing,
    prompt_snippet: "Redeems operating funds from a MetaMask runway subscription. Phase-aware multiplier. Called by heartbeat FSM.",
    prompt_guidelines: &[
        "thriving: Redeem full daily cap as needed for operations.",
        "cautious: Redeem 50% of daily cap. Conserve runway.",
        "declining: Redeem 10% only -- inference costs only.",
        "terminal: Redemption multiplier is 0. Triggers metamask_revoke_permissions instead.",
    ],
};
```

**Ground truth:** `expected_outcome` = redeemable amount based on phase multiplier. `actual_outcome` = amount actually transferred. `ground_truth_source` = `"erc7715_verifier"`.

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ golem_id, phase, daily_cap }` |
| `tool:end` | `{ redeemed, remaining, phase }` |

**Pi hooks:** `tool_call` (verify permission exists and is valid, compute redeemable amount from phase multiplier). `tool_result` (update Grimoire cumulative spend, update permission health).

---

#### `metamask_revoke_permissions`

The revocation cascade. Called on: terminal phase entry, PolicyCage violation, owner kill-switch, anomaly in heartbeat (missed N consecutive ticks). Calls `wallet_revokePermissions` for all active permission IDs associated with the Golem. Marks the Session Account as invalid in the Bardo control plane. Emits a `PERMISSION_REVOKED` event on the Pi event bus. The Snap's next cron tick detects the state change and emits a native notification.

| Parameter | Type | Required | Description |
| --- | --- | --- | --- |
| `golems` | `Vec<String>` | Yes | Golem IDs to revoke |
| `reason` | `String` | Yes | `"terminal"`, `"policy_violation"`, `"owner_kill_switch"`, `"heartbeat_anomaly"` |

```rust
#[derive(Debug, Deserialize)]
pub struct RevokePermissionsParams {
    pub golems: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RevokePermissionsResult {
    pub revoked_count: u32,
    pub session_account_invalidated: bool,
}
```

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "metamask_revoke_permissions",
    description: concat!(
        "Revocation cascade: revokes all active ERC-7715 permissions for specified Golems. ",
        "Called on terminal phase, PolicyCage violation, owner kill-switch, or heartbeat anomaly. ",
        "Marks Session Account invalid. Emits PERMISSION_REVOKED on Pi event bus.",
    ),
    category: Category::Wallet,
    capability: CapabilityTier::Privileged,
    risk_tier: RiskTier::Layer2,
    tick_budget: TickBudget::Medium,
    progress_steps: &[
        "Loading active permissions",
        "Calling wallet_revokePermissions",
        "Invalidating Session Account",
        "Emitting PERMISSION_REVOKED",
    ],
    sprite_trigger: SpriteTrigger::Failure,
    prompt_snippet: "Revocation cascade: revokes all active ERC-7715 permissions for specified Golems. Called on terminal phase, PolicyCage violation, owner kill-switch, or heartbeat anomaly.",
    prompt_guidelines: &[
        "thriving: Only revoke on explicit owner kill-switch or policy violation.",
        "cautious: Revoke only on owner kill-switch or detected anomaly.",
        "declining: Prepare for revocation. Do not revoke proactively -- wait for terminal transition.",
        "terminal: Revoke ALL permissions immediately. This is mandatory.",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ golems, reason }` |
| `tool:end` | `{ revoked_count, session_account_invalidated }` |

**Pi hooks:** `tool_call` (validate reason, verify permissions exist). `tool_result` (emit `PERMISSION_REVOKED` on Pi event bus, invalidate Session Account, notify Snap via next cron tick).

---

#### `metamask_check_permission_health`

Reads the current permission state: remaining allowance for the current 24-hour window, total remaining across the subscription duration, last redemption timestamp, and permission status from the ERC-7715 verifier. Returns a health classification.

| Status | Meaning |
| --- | --- |
| `healthy` | >50% daily cap remaining, >7 days remaining |
| `low` | <50% daily cap remaining, or <3 days remaining |
| `critical` | <10% daily cap remaining, or <1 day remaining |
| `revoked` | Permission object has been revoked |

```rust
#[derive(Debug, Deserialize)]
pub struct CheckPermissionHealthParams {
    pub golem_id: String,
}

#[derive(Debug, Serialize)]
pub struct PermissionHealthResult {
    pub health: String,
    pub remaining_today: String,
    pub days_remaining: u32,
    pub last_redemption_at: u64,
    pub total_remaining: String,
}
```

Used by the Snap cron job and the RunwaySubscriptionCard.

**ToolDef fields:**

```rust
pub static TOOL_DEF: ToolDef = ToolDef {
    name: "metamask_check_permission_health",
    description: concat!(
        "Reads current permission state: remaining daily allowance, total remaining across subscription, ",
        "last redemption, health classification (healthy/low/critical/revoked). Read-only. ",
        "Used by Snap cron and RunwaySubscriptionCard.",
    ),
    category: Category::Wallet,
    capability: CapabilityTier::Read,
    risk_tier: RiskTier::Layer1,
    tick_budget: TickBudget::Fast,
    progress_steps: &["Reading permission state"],
    sprite_trigger: SpriteTrigger::Thinking,
    prompt_snippet: "Reads current permission state: remaining daily allowance, total remaining, health classification. Read-only.",
    prompt_guidelines: &[
        "thriving: Check periodically. Renew when status is low.",
        "cautious: Check every tick. Alert owner when critical.",
        "declining: Check every tick. Prepare for revocation when critical.",
        "terminal: Check once to confirm revocation succeeded (status should be revoked).",
    ],
};
```

**Event Fabric events:**

| Event | Payload |
| --- | --- |
| `tool:start` | `{ golem_id }` |
| `tool:end` | `{ health, remaining_today, days_remaining }` |

**Pi hooks:** `tool_call` (none). `tool_result` (if health is "critical", emit Snap notification; if "revoked", confirm clean state).

---

### Re-delegation for Clade and Replicant flows

When a Golem spawns a Replicant, the parent Golem sub-delegates a fraction of its runway:

1. Parent Golem holds a 10 USDC/day permission.
2. Replicant creation calls `wallet_grantPermissions` with a child permission: 2 USDC/day, scoped to the Replicant's Session Account, with an explicit expiry tied to the Replicant's expected lifespan.
3. Parent's effective remaining capacity: 8 USDC/day.
4. When the Replicant enters terminal phase, `metamask_revoke_permissions` fires on the child permission only. Parent's 8 USDC/day is unaffected.

This is EIP-7710 sub-delegation. The parent's permission object carries a `delegationChain` that the ERC-7715 verifier validates -- preventing a Replicant from granting permissions wider than what it received.

---

## Bardo Snap

### Package and distribution

New workspace package: `packages/snap/` (`@bardo/snap`). MetaMask Flask required for hackathon because `endowment:transaction-insight` and `endowment:signature-insight` are protected permissions not available in production MetaMask without allowlisting. Post-hackathon: submit for production allowlisting. Distribution via `npm:@bardo/snap`.

### Manifest permissions

```json
{
  "initialPermissions": {
    "endowment:page-home": {},
    "endowment:cronjob": {
      "jobs": [
        { "expression": "*/5 * * * *", "request": { "method": "heartbeat" } }
      ]
    },
    "endowment:network-access": {},
    "endowment:rpc": { "dapps": true, "snaps": false },
    "endowment:transaction-insight": { "allowTransactionOrigin": true },
    "endowment:signature-insight": { "allowSignatureOrigin": true },
    "snap_notify": {},
    "snap_manageState": {},
    "snap_dialog": {}
  }
}
```

Protected permissions requiring Flask: `endowment:transaction-insight`, `endowment:signature-insight`.

### Handlers

**`onHomePage()`** -- Golem console panel.

Reads cached Golem state from `snap_manageState`. Renders a structured panel (no React -- MetaMask Snaps use the `@metamask/snaps-sdk` JSX subset):

```
BARDO GOLEM CONSOLE
------------------------------------
* Atlas         [THRIVING]   98 vitality
  Permission: healthy - 8.2 USDC/day remaining - 4d left
  Last heartbeat: 3m ago

* Echo          [CONSERVATION]   61 vitality
  Permission: low - 2.1 USDC/day remaining - 1d left
  Last heartbeat: 8m ago
------------------------------------
[Revoke all]
```

Unicode * (U+25C6) for Golem bullet. No emojis. "Revoke all" triggers `snap_dialog` confirm -> calls `triggerRevoke` RPC to Portal.

**`onCronjob({ request: { method: "heartbeat" } })`**

Fires every 5 minutes. Fetches `GET /api/snap/status` from the Bardo control plane (Bearer token stored in `snap_manageState`). Updates cached state. Sends `snap_notify` (type `native`) if:

- Any Golem in `declining` or `terminal` phase
- Any permission health `critical` or `revoked`
- Missed heartbeat: last heartbeat timestamp > 2x expected interval

**`onTransaction({ transaction, chainId })`**

Decodes calldata against known Bardo ABI signatures and returns a `TransactionInsightResponse`:

| Detected calldata | Display | Severity |
| --- | --- | --- |
| `Warden.announce(bytes32, uint256)` (requires optional Warden module) | "Bardo Warden announcement -- cancel window: {delay}min, action hash: {short}" | `warning` |
| `Warden.execute(bytes32)` (requires optional Warden module) | "Warden execution -- delay elapsed, action committed" | `critical` |
| EIP-712 `ActionPermit` typed data | Risk tier, policy hash, simulation hash present/absent | `warning` |
| UniswapX order submission | "Agent LP/trade action -- policy tier: {tier}" | `loading` |
| Unknown | No annotation -- pass through | (none) |

Severity maps to MetaMask's `TransactionInsightComponent` severity colors: `critical` = red, `warning` = yellow, `loading` = grey.

**`onSignature({ signature, signatureOrigin })`**

Detects EIP-712 typed data with Bardo domain separator (`name: "Bardo"` or `name: "BardoWarden"`). Returns annotation:

- `ActionPermit`: "This is a Bardo action authorization -- risk tier {tier}, policy hash {short}"
- `PermissionRequest`: "This is a Bardo runway subscription -- {amount} USDC/day for {days} days"
- Unknown Bardo domain: "Unrecognized Bardo signature -- verify the origin before signing"
- Unknown origin signing Bardo domain: flag as suspicious, severity `critical`

**`onRpcRequest({ origin, request })`**

Origin-restricted to Bardo Portal (`https://bardo.ai`, `http://localhost:3003` in dev). Methods:

| Method | Action |
| --- | --- |
| `showApprovalDialog({ title, body })` | `snap_dialog` with approve/reject buttons |
| `showPlanSummary({ plan })` | Formatted plan display in dialog |
| `triggerRevoke({ golems })` | Confirm dialog -> calls Portal revoke endpoint |
| `getGolemsStatus()` | Returns cached state from `snap_manageState` |
| `updateAuthToken({ token })` | Updates Bearer token for control plane polling |

### State management

`snap_manageState` stores encrypted state up to 100MB. Schema:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SnapState {
    pub golems: HashMap<String, GolemStateEntry>,
    pub permissions: HashMap<String, PermissionEntry>,
    pub alert_history: Vec<AlertEntry>,
    pub auth_token: Option<String>,
    pub last_fetch_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GolemStateEntry {
    pub name: String,
    pub phase: String,  // "thriving", "conservation", "declining", "terminal"
    pub vitality: u32,
    pub last_heartbeat_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub golems: String,
    pub daily_cap: String,
    pub remaining_today: String,
    pub expires_at: u64,
    pub health: String,  // "healthy", "low", "critical", "revoked"
}
```

### Portal <-> Snap bridge

`packages/portal/src/lib/snap_client.ts` wraps all Snap RPC calls:

```rust
// Rust equivalent for type reference
pub struct SnapClient {
    snap_id: String,
}

impl SnapClient {
    pub async fn invoke(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value>;
    pub async fn is_installed(&self) -> Result<bool>;
    pub async fn prompt_install(&self) -> Result<()>;
}
```

Portal calls `is_installed()` on connect. If false, shows `SnapStatusBadge` with "Install Snap" CTA. First install triggers MetaMask's Snap installation confirmation dialog.

---

## Wallet API plumbing

Beyond Advanced Permissions and Snap RPC, the integration uses:

**`eth_signTypedData_v4`** for:

- x402 payment authorization (owner signing compute invoices)
- ERC-8004 identity attestation during Golem registration
- `ActionPermit` creation for Warden-controlled high-risk actions (requires optional Warden module)

**`wallet_getPermissions` / `wallet_requestPermissions`** for classic EIP-2255 capabilities. Used on first connect to request `eth_accounts` and verify account access.

**`eth_accounts` polling** -- Portal watches for owner account changes (MetaMask account switch). On account change, tears down the active Golem session and prompts re-authentication.

**Chain switching** -- Portal calls `wallet_switchEthereumChain` to suggest Base (chain ID 8453). Advanced Permissions and x402 are live on Base; Warden contracts (optional, deferred) would be deployed on Base and Ethereum mainnet. If the owner is on an unsupported chain, a chain-switch banner appears before any transaction is initiated.

---

## Security model

**Two independent enforcement layers:**

MetaMask enforces the permission object: the owner sees a legible USDC/day cap in the MetaMask UI, signs it, and the ERC-7715 verifier enforces it on every redemption call. No Bardo-side logic can override this -- the verifier is a contract the owner doesn't control.

Bardo enforces PolicyCage: even within the granted permission, the Golem can only act within its configured policy tiers. A phase-declining Golem cannot initiate new positions even if the permission object allows the spend.

**Risk zones and mitigations:**

Over-broad permissions: default to 7-day windows, daily caps explicitly displayed, expiry required. The `metamask_request_runway` tool rejects `duration_days > 90` or `daily_cap_usdc > 100` without an explicit owner override flag.

EIP-7702 phishing: use Smart Accounts Kit only. Never prompt owners to "sign an authorization tuple" directly -- the Kit provides a reviewed UI flow. Portal never constructs raw EIP-7702 auth blobs.

Snap protected permissions: Flask for hackathon. Production submission requires MetaMask allowlisting review for `endowment:transaction-insight` and `endowment:signature-insight`. Until approved, the Snap degrades -- home page and cron still work, transaction/signature insights are absent.

Metadata leakage via Snap: all `/api/snap/status` calls are authenticated with a rotating Bearer token stored in `snap_manageState`. The token rotates on every heartbeat. The Snap RPC handler validates origin against the Portal allowlist before responding to any `onRpcRequest` call.

Session Account compromise: if the Session Account key is compromised, the attacker can redeem up to the remaining daily cap. They cannot exceed it (ERC-7715 verifier), cannot grant new permissions (requires owner signature), and cannot access the Golem's Execution Wallet (separate Privy TEE custody). Worst case: attacker drains one day's USDC budget. Owner can revoke the permission immediately via MetaMask or Portal.

---

## Death Protocol integration

When the heartbeat FSM transitions to Terminal:

1. **`metamask_revoke_permissions`** fires with reason `"terminal"`. All active ERC-7715 permissions for the Golem are revoked. Session Account marked invalid.

2. **Pi event bus** emits `GOLEM_TERMINAL` with permission IDs and Golem identity.

3. **Snap `onCronjob`** detects `terminal` phase on next tick (within 5 minutes). Emits `snap_notify`:

   > "Golem {name} has reached terminal phase. All MetaMask permissions revoked. Knowledge transferred to Clade."
   > The notification includes a deep link to Portal succession flow.

4. **Portal** surfaces the succession CTA: "Fund a new Golem with Runway subscription." The CTA pre-fills the new permission request with the deceased Golem's last-used daily cap.

5. **Snap home page** shows the dead Golem entry with phase `terminal` until the owner dismisses it. The Grimoire transfer record is linked.

---

## Distribution

**Hackathon**: MetaMask Flask (developer build). No allowlisting needed for protected Snap permissions. Flask is distributed as a separate browser extension; testers install it alongside or instead of production MetaMask.

**Post-hackathon**: Submit `@bardo/snap` to MetaMask Snap registry for production allowlisting. The allowlisting review covers `endowment:transaction-insight` and `endowment:signature-insight` -- both require MetaMask's security review. Expected timeline: 2-4 weeks from submission.

**Production distribution URL**: `npm:@bardo/snap` (once published to npm). Portal passes this as the `snapId` in `wallet_invokeSnap` calls.

---

## UI touchpoints

**`MetaMaskConnectButton`** -- Primary connection entry point in Portal header. SDK-first with EIP-6963 fallback. Shows "Upgrade to Smart Account" inline if connected account is a plain EOA. Exported from `packages/ui/` for reuse.

**`RunwaySubscriptionCard`** -- Dashboard card showing current permission health: USDC/day cap, days remaining, amount redeemed today, health badge (healthy / low / critical / revoked). "Revoke" button calls `metamask_revoke_permissions` with reason `"owner_kill_switch"`. "Renew" button opens the `metamask_request_runway` flow.

**`SnapStatusBadge`** -- Header badge: "Bardo Snap installed *" or "Install Snap ->". Clicking "Install Snap" calls `prompt_install()` from the Snap client. Once installed, shows last heartbeat time.

**`GolemPermissionsPanel`** -- Per-Golem drawer showing all active permissions, their health, redemption history (last 7 days), and per-Replicant sub-permissions. Includes a "Revoke all" button with a confirmation modal.

**`DeathCountdownOverlay`** -- Full-screen overlay when a Golem is in terminal phase. Shows the revocation countdown (time until `metamask_revoke_permissions` fires), the Snap notification status, and the succession CTA. Disappears once revocation is confirmed.

---

## Tool summary

| Tool | Category | Capability | Risk | Budget |
| --- | --- | --- | --- | --- |
| `metamask_request_runway` | wallet | Write | Layer 2 | Slow |
| `metamask_redeem_runway` | wallet | Write | Layer 2 | Medium |
| `metamask_revoke_permissions` | wallet | Privileged | Layer 2 | Medium |
| `metamask_check_permission_health` | wallet | Read | Layer 1 | Fast |

All 4 permission tools are in the `wallet` category and included in profiles: `identity`, `golem`, `full`, `dev`.
