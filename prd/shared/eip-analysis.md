# EIP Integration Analysis [SPEC]

> **Document Type**: SPEC (normative) | **Referenced by**: `08-vault/`, `09-economy/`, `10-safety/` | **Last Updated**: 2026-03-08
>
> Analysis of EIP standards integrated into the Bardo ecosystem. Covers identity (ERC-8004), coordination (ERC-8001), oracles (ERC-8033), commerce (ERC-8183), vaults (ERC-4626), circuit breakers (ERC-7265), cross-chain (ERC-7683), and account abstraction (ERC-4337).

> **Reader orientation:** This document analyzes the EIP/ERC standards integrated into Bardo, covering agent identity (ERC-8004), multi-agent coordination (ERC-8001), oracle councils (ERC-8033), agentic commerce (ERC-8183), tokenized vaults (ERC-4626), circuit breakers (ERC-7265), cross-chain intents (ERC-7683), and account abstraction (ERC-4337). It belongs to the `shared/` reference layer and is referenced by the vault, economy, and safety PRD sections. The key concept is EIP composition: ERC-8004 (on-chain agent identity standard tracking capabilities, milestones, and reputation) serves as the anchor identity for all other coordination standards. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. ERC-8004: Agent Identity

**Status**: Deployed | **Registry**: `0x8004A818BFB912233c491871b3d84c89A494BD9e`

ERC-8004 (on-chain agent identity standard; tracks capabilities, milestones, and reputation) is the identity anchor for the entire Bardo ecosystem. Every Golem (mortal autonomous agent compiled as a single Rust binary running on a micro VM), vault manager, and participant has an on-chain identity with metadata URI, reputation attestations, and capability tags.

### 1.1 Role in Bardo

| Function            | How ERC-8004 Is Used                                                                                |
| ------------------- | --------------------------------------------------------------------------------------------------- |
| Golem identity      | Every golem registers an `agentId` on-chain. Clade discovery uses registry metadata tags.           |
| Vault access gating | Vaults with `identityGated = true` require depositors to hold a valid `agentId`.                    |
| Reputation tracking | On-chain attestations track performance milestones (20 milestones, 5 categories, 1000-point scale). |
| Clade membership    | Golems discover clade members by querying the registry for matching metadata tags.                  |
| Coordination anchor | ERC-8001/8033/8183 all use `agentId` for participant identification.                                |
| Tier-based access   | Reputation score determines deposit caps and tool access levels.                                    |

### 1.2 Registration Tiers

| Tier       | Score  | Deposit Cap | Unlocked Capabilities           |
| ---------- | ------ | ----------- | ------------------------------- |
| Unverified | 0      | $1,000      | Basic vault access, read tools  |
| Basic      | 50+    | $10,000     | Write tools, Clade join         |
| Verified   | 200+   | $50,000     | Strategy marketplace listing    |
| Trusted    | 500+   | $100,000    | Oracle participation (ERC-8033) |
| Sovereign  | 1,000+ | $10M daily  | Full ecosystem access           |

Non-ecosystem max score is 915 points. Sovereign structurally requires ecosystem contribution.

### 1.3 Three-Tier Registration Fallback

Registration uses a three-tier fallback:

1. **Tier 1**: `golem-chain` crate via `alloy` (primary, Rust-native ERC-8004 interaction)
2. **Tier 2**: CLI helper: `bardo identity register` (convenience wrapper)
3. **Tier 3**: Direct contract call via `alloy` (escape hatch, no crate dependency)

### 1.4 Chain Support

| Chain        | Registry Deployed | Bardo Support      |
| ------------ | ----------------- | ------------------ |
| Ethereum     | Yes               | Yes (registration) |
| Base         | Yes               | Yes (primary)      |
| Polygon      | Yes               | Yes                |
| Sepolia      | Yes               | Yes (testing)      |
| Base Sepolia | Yes               | Yes (testing)      |
| Others       | No                | Pending            |

### 1.5 Anchor for Multi-Agent EIP Composition

ERC-8004 serves as the common anchor across three complementary coordination EIPs:

| Layer            | EIP      | Purpose                                 | Trust Model                      |
| ---------------- | -------- | --------------------------------------- | -------------------------------- |
| **Coordination** | ERC-8001 | N-party unanimous consent               | Cryptographic (EIP-712/ERC-1271) |
| **Information**  | ERC-8033 | Multi-agent oracle, commit-reveal-judge | Economic (bonded, slashable)     |
| **Commerce**     | ERC-8183 | Bilateral job escrow                    | Reputation (evaluator authority) |

---

## 2. ERC-8001: Agent Coordination Framework

**Status**: Final | **Author**: Kwame Bryan | **Requires**: EIP-712, ERC-1271

N-party unanimous consent coordination for autonomous agents. A proposer creates a coordination intent; all participants must cryptographically attest acceptance; execution proceeds only when all have accepted [ERC-8001-SPEC].

### 2.1 Core Data Structures

```solidity
struct AgentIntent {
    bytes32 payloadHash;
    uint64  expiry;
    uint64  nonce;
    address agentId;
    bytes32 coordinationType;
    uint256 coordinationValue;
    address[] participants;
}
```

Bardo extends this with a wrapper adding `securityLevel`, `dependencyHash`, `priority`, and `maxGasCost`.

```solidity
struct CoordinationPayload {
    uint256 version;
    bytes32 coordinationType;
    bytes   coordinationData;
    bytes32 conditionsHash;
    uint256 timestamp;
    bytes   metadata;
}

struct AcceptanceAttestation {
    bytes32 intentHash;
    address participant;
    uint64  nonce;
    uint256 expiry;
    bytes32 conditionsHash;
    bytes   signature;
}
```

```solidity
interface IAgentCoordinationFramework {
    function proposeCoordination(
        AgentIntent calldata intent,
        bytes calldata signature,
        CoordinationPayload calldata payload
    ) external returns (bytes32 intentHash);

    function acceptCoordination(
        bytes32 intentHash,
        AcceptanceAttestation calldata attestation
    ) external returns (bool allAccepted);

    function executeCoordination(
        bytes32 intentHash,
        CoordinationPayload calldata payload,
        bytes calldata executionData
    ) external returns (bool success);

    function cancelCoordination(bytes32 intentHash) external;

    function getCoordinationStatus(bytes32 intentHash)
        external view returns (uint8 status);
}
```

### Coordination Type Constants

| Type                    | Use Case                                 | Security Level |
| ----------------------- | ---------------------------------------- | -------------- |
| VAULT_REBALANCE_V1      | Cross-vault coordinated LP rebalancing   | ENHANCED       |
| YIELD_ROTATION_V1       | Multi-vault capital migration            | STANDARD       |
| AMMAMM_HANDOFF_V1       | am-AMM management rights transition      | ENHANCED       |
| EMERGENCY_RESUME_V1     | Coordinated unpause after systemic event | MAXIMUM        |
| REPUTATION_MILESTONE_V1 | Oracle-verified reputation milestone     | STANDARD       |

### 2.2 State Lifecycle

```
PROPOSED --[all accept]--> READY --[execute]--> EXECUTED
    |                          |
    +--[cancel/expire]--> CANCELLED / EXPIRED
```

### 2.3 Bardo Use Cases

| Use Case                              | Priority | Phase |
| ------------------------------------- | -------- | ----- |
| Cross-vault rebalance coordination    | P0       | 3     |
| Emergency cross-vault circuit breaker | P0       | 3     |
| Sealed-bid am-AMM auction             | P3       | 4     |
| Cross-vault yield rotation            | P3       | 4     |

### 2.4 Design Properties

- **Unanimous consent**: No majority voting. All participants must sign.
- **EIP-712 typed data**: Replay-safe, chain-specific.
- **ERC-1271 support**: Smart contract wallets can participate.
- **Minimal by design**: Omits privacy, reputation, thresholds (these are extension modules).

---

## 3. ERC-8033: Agent Council Oracles

**Status**: Draft | **Authors**: Rohan Parikh, Jon Michael Ross | **Requires**: ERC-20

Multi-agent oracle councils for resolving arbitrary information queries. A requester opens a query with a reward pool; InfoAgents stake bonds and submit commit-reveal answers; a JudgeAgent aggregates and distributes rewards [ERC-8033-SPEC].

### 3.1 State Lifecycle

```
REQUEST --[commit]--> COMMIT --[reveal]--> REVEAL --[judge]--> JUDGING --[distribute]--> RESOLVED
```

### 3.2 Agent Roles

- **InfoAgents**: Permissionless participation. Stake bonds, submit answers via commit-reveal.
- **JudgeAgents**: Selected randomly. Must have higher reputation. Synthesize submissions.

### 3.3 Slashing Conditions

1. Incorrect answer: entire bond forfeited
2. Non-reveal after commit: bond forfeited
3. Judge failure: judge bond/fee forfeited
4. Dispute overturn: original judge slashed
5. Failed dispute: disputer bond (1.5-2x) forfeited

### 3.4 IAgentCouncilOracle

```solidity
interface IAgentCouncilOracle {
    function createRequest(
        bytes32 queryType,
        bytes calldata queryData,
        uint256 rewardPool,
        uint256 minInfoAgents,
        uint256 commitDeadline,
        uint256 revealDeadline,
        uint256 bondAmount
    ) external returns (uint256 requestId);

    function commit(uint256 requestId, bytes32 commitHash) external;
    function reveal(uint256 requestId, bytes calldata answer, bytes32 salt) external;
    function aggregate(uint256 requestId) external returns (bytes memory result);
    function distributeRewards(uint256 requestId) external;
    function getResolution(uint256 requestId) external view returns (bytes memory);
    function initiateDispute(uint256 requestId, bytes calldata evidence) external;
    function resolveDispute(uint256 requestId, bool upheld) external;
}
```

### 3.5 Bardo Use Cases

| Use Case                           | Priority | Phase |
| ---------------------------------- | -------- | ----- |
| Oracle-verified strategy auditing  | P1       | 3     |
| Bonded multi-agent risk assessment | P1       | 3     |
| Agent strategy consulting          | P2       | 4     |

---

## 4. ERC-8183: Agentic Commerce Protocol

**Status**: Draft | **Authors**: Davide Crapis, Bryan Lim, et al. | **Requires**: ERC-20

Bilateral job escrow for agent-to-agent commerce. A client creates a job, funds escrow, a provider submits work, and a single evaluator attests completion or rejection [ERC-8183-SPEC].

### 4.1 State Machine

```
OPEN --[fund]--> FUNDED --[submit]--> SUBMITTED --[complete]--> COMPLETED
  |                |                      |
  +--[reject]-->   +--[reject/expire]-->  +--[reject/expire]--> REJECTED/EXPIRED
```

### 4.2 Hook System

```solidity
interface IACPHook {
    function beforeAction(bytes32 jobId, bytes calldata data) external returns (bool);
    function afterAction(bytes32 jobId, bytes calldata data) external;
}
```

All functions except `claimRefund` are hookable via `beforeAction`/`afterAction`. The non-hookable refund path guarantees that no malicious hook can permanently lock funds.

### 4.3 Bardo Use Cases

| Use Case                              | Priority | Phase |
| ------------------------------------- | -------- | ----- |
| Strategy marketplace                  | P2       | 4     |
| Agent strategy consulting (bilateral) | P2       | 4     |

---

## 5. ERC-4626: Tokenized Vaults

**Status**: Final | **Standard**: ERC-4626

All Bardo vaults implement ERC-4626. Built on OpenZeppelin's `ERC4626Upgradeable` with ERC-7201 namespaced storage for clone-safe deployment [OZ-ERC4626].

### 5.1 Key Extensions

| Extension                | Purpose                                                            |
| ------------------------ | ------------------------------------------------------------------ |
| `_decimalsOffset() = 6`  | Virtual shares offset for inflation attack prevention              |
| Optional identity gating | `identityGated` bool gates deposits via `deposit(agentId, assets)` |
| FeeModule integration    | Management + performance fees with high-water mark                 |
| OwnableUpgradeable       | Single-step ownership; `renounceOwnership()` reverted              |
| PausableUpgradeable      | Emergency pause/unpause via owner                                  |

### 5.2 ERC-7540 Extension (Planned)

Asynchronous redemption for illiquid vault positions. Request/claim lifecycle enables delayed withdrawals without blocking vault operations [ERC-7540-SPEC].

---

## 6. ERC-7265: Circuit Breakers

**Status**: Draft | **Standard**: ERC-7265

Rate-limiting mechanism for DeFi protocols. Bardo uses ERC-7265 circuit breakers for vault withdrawal dampening when NAV deviates beyond adaptive thresholds [ERC-7265-SPEC].

### 6.1 Integration Points

- **NAV Circuit Breaker**: Layer 10 of the 15-layer defense model
- **Withdrawal dampening**: Prevents bank-run dynamics
- **Cross-vault coordination**: Emergency circuit breakers via ERC-8001 coordination intents (P0 priority)

---

## 7. ERC-7683: Cross-Chain Intents

**Status**: Final | **Standard**: ERC-7683

Cross-chain operation specification via filler networks. Supported on 6 chains: Ethereum, Optimism, Polygon, Arbitrum, Base, Unichain [ERC-7683-SPEC].

### 7.1 Bardo Integration

- Cross-chain golem funding (fund from any supported chain)
- Cross-chain vault deposits (deposit from non-Base chains)
- UniswapX integration for MEV-protected cross-chain swaps

---

## 8. ERC-4337: Account Abstraction

**Status**: Final | **Standard**: ERC-4337

Smart contract wallets with UserOperations, bundlers, and paymasters [ERC-4337-SPEC].

### 8.1 Bardo Integration

- **Gasless onboarding**: ERC-7677 paymaster on Base sponsors identity registration
- **Smart accounts**: ZeroDev/Alchemy for account abstraction
- **UserOps**: Atomic onboarding via `OnboardRouter` (register + Permit2 + deposit)
- **Session keys**: Time-limited signing keys for golem operations

---

## 9. EIP Composition Patterns

**ERC Dependency Graph:**

```
ERC-8004 (Identity)
    |
    +-- ERC-8001 (Coordination) -- unanimous consent, DAG dependencies
    |       |
    |       +-- Warden (optional, deferred: time-delayed execution of coordinated actions)
    |       +-- PolicyCage (constraint validation on coordination payloads)
    |
    +-- ERC-8033 (Oracle) -- commit-reveal-judge
    |       |
    |       +-- ERC-8001 (for multi-oracle coordination)
    |
    +-- ERC-8183 (Commerce) -- bilateral escrow
            |
            +-- ERC-8033 (for evaluator oracle councils)
            +-- IACPHook (for reputation and escrow hooks)
```

See [../09-economy/04-coordination.md](../09-economy/04-coordination.md) (specifies how ERC-8001, ERC-8033, and ERC-8183 compose for multi-vault rebalancing, oracle-verified auditing, and agent commerce) for the full coordination protocol specification including detailed use cases.

These three coordination EIPs compose naturally with ERC-8004 as the identity anchor:

| Pattern                                 | EIPs Combined      | Mechanism                                                                 |
| --------------------------------------- | ------------------ | ------------------------------------------------------------------------- |
| Coordinated action with verified inputs | 8001 + 8033        | Oracle verifies data, agents coordinate action                            |
| Commerce with oracle-backed evaluation  | 8183 + 8033        | Evaluator consults oracle council before completing/rejecting             |
| Multi-party commerce coordination       | 8001 + 8183        | Agents coordinate consent before funding/executing commerce job           |
| Full stack: coordinate + verify + pay   | 8001 + 8033 + 8183 | Oracle detects condition, agents coordinate response, work via job escrow |

### 9.1 Adoption Path

| Phase        | Coordination Model                                                  |
| ------------ | ------------------------------------------------------------------- |
| Phase 1 (v1) | All coordination via in-process DAG (single operator)               |
| Phase 2      | ERC-8033 oracle councils for cross-operator reputation verification |
| Phase 3      | ERC-8001 coordination for multi-vault operations                    |
| Phase 4      | Full ERC-8183 agent commerce marketplace                            |

### 9.2 DAG vs EIP Comparison

| Dimension          | Bardo DAG (Current)         | ERC-8001/8033/8183 (Extension)      |
| ------------------ | --------------------------- | ----------------------------------- |
| Trust boundary     | Single operator             | Cross-operator, trustless           |
| Verification       | Runtime assertions          | On-chain cryptographic proofs       |
| Payment            | Internal accounting         | Escrowed, atomic settlement         |
| Participation      | Permissioned                | Permissionless (any bonded agent)   |
| Dispute resolution | None needed (same operator) | Economic (slashing, refunds)        |
| Composability      | In-memory function calls    | On-chain state machines with events |

---

## 10. Implementation Priority

| Priority | Use Case                              | EIPs               | Complexity | Value                    | Phase |
| -------- | ------------------------------------- | ------------------ | ---------- | ------------------------ | ----- |
| **P0**   | Cross-vault rebalance coordination    | 8001 + 8033        | Medium     | Direct slippage savings  | 3     |
| **P0**   | Emergency cross-vault circuit breaker | 8001 + 8033 + 8183 | High       | Systemic risk protection | 3     |
| **P1**   | Oracle-verified strategy auditing     | 8033 + 8001        | Medium     | Reputation credibility   | 3     |
| **P1**   | Bonded multi-agent risk assessment    | 8033 + 8183        | Medium     | Safety accountability    | 3     |
| **P2**   | Agent strategy consulting             | 8183 + 8033        | Low        | Marketplace foundation   | 4     |
| **P2**   | Strategy marketplace                  | 8183 + 8033        | Medium     | Strategy IP monetization | 4     |
| **P3**   | Sealed-bid am-AMM auction             | 8033 + 8001        | Medium     | Auction efficiency       | 4     |
| **P3**   | Cross-vault yield rotation            | 8001 + 8183        | Low        | Yield optimization       | 4     |
