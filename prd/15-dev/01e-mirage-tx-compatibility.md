# mirage-rs -- Transaction Compatibility [SPEC]

**Version:** 2.0.0
**Last Updated:** 2026-03-18
**Parent:** [01-mirage-rs.md](./01-mirage-rs.md)

---

> **Reader orientation:** This document covers transaction compatibility edge cases for mirage-rs, Bardo's in-process EVM fork (section 15). It is a companion to the RPC reference (`01b-mirage-rpc.md`); where that doc covers what methods exist, this doc covers what can go wrong and why. The key concept is that EVM fork simulation introduces subtle divergences from mainnet behavior: signature verification semantics change, gas costs depend on hardfork configuration, and DeFi protocols embed assumptions about timestamps, price feeds, and nonce ordering that break in fork state. See `prd2/shared/glossary.md` for full term definitions.

This document covers transaction validity, signature verification, gas edge cases, nonce semantics, state injection pitfalls, EVM hardfork compatibility, and DeFi-specific concerns. It is a companion to [01b-mirage-rpc.md](./01b-mirage-rpc.md), which covers the RPC surface; this doc covers what can go wrong and why.

## Transaction Formats

EIP-2718 defines a typed transaction envelope. mirage-rs accepts all four current types.

### Type 0 -- Legacy

Plain RLP encoding, no leading type byte. Signature fields are `v`, `r`, `s`. Pre-EIP-155 transactions use `v` in {27, 28}. Post-EIP-155 (replay protection) uses `v = 2 * chainId + 35` or `v = 2 * chainId + 36`.

Required fields: `nonce`, `gasPrice`, `gas`, `to` (optional for contract creation), `value`, `data`.

### Type 1 -- EIP-2930 Access List

Encoding: `0x01 || RLP([chainId, nonce, gasPrice, gas, to, value, data, accessList])`

Adds `accessList`: an array of `{address, storageKeys[]}` used to pre-warm slots. The hash includes the access list; a tx signed without the correct list is invalid. Useful for reducing gas on known-hot paths (see the gas section below).

### Type 2 -- EIP-1559 Fee Market

Encoding: `0x02 || RLP([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gas, to, value, data, accessList])`

Replaces `gasPrice` with `maxFeePerGas` and `maxPriorityFeePerGas`. Do not set `gasPrice` in a type 2 tx -- it is ignored. `maxFeePerGas` must be >= the current block's `baseFeePerGas` or the tx is rejected. Actual fee paid: `min(maxFeePerGas, baseFeePerGas + maxPriorityFeePerGas)`.

### Type 3 -- EIP-4844 Blob Transaction

Encoding: `0x03 || RLP([chainId, nonce, maxPriorityFeePerGas, maxFeePerGas, gas, to, value, data, accessList, maxFeePerBlobGas, blobVersionedHashes])`

Adds `blobVersionedHashes` (array of `bytes32`) and `maxFeePerBlobGas`. Blob data (`blobs`, `commitments`, `proofs`) is not stored on-chain; it is transmitted separately to the consensus layer and expires after ~18 days. mirage-rs does not validate blob data by default; it accepts the tx envelope and ignores blob fields unless `--validate-blobs` is set.

### mirage-rs Behavior

- `eth_sendTransaction`: auto-selects type based on fields provided. If `maxFeePerGas` is present, sends type 2. If `accessList` is present without fee market fields, sends type 1. Otherwise type 0.
- `eth_sendRawTransaction`: accepts all four types; validates the type byte and decodes accordingly.
- Signature verification: skipped by default; enabled via `--verify-signatures`.
- Impersonation: bypasses signature check entirely, regardless of `--verify-signatures`.

---

## Signature Verification

### ECDSA / EIP-155

By default, mirage-rs skips ECDSA verification. Fork simulation gives you control over all state, so verifying every signature is usually unnecessary overhead.

With `--verify-signatures` enabled:

- Verifies `v`, `r`, `s` fields against the transaction hash.
- Enforces EIP-155 chain ID: `v` must encode the correct chain ID or the tx is rejected.
- Rejects signature malleability: if `s > secp256k1.n/2`, the signature is high-s and rejected.
- `ecrecover` returning `address(0)` means the signature is invalid -- rejected.

### EIP-1271 -- Contract Signatures

For contract wallets (Safe, Argent, etc.), signature validity is determined by calling `isValidSignature(bytes32 hash, bytes signature)` on the contract and checking the return value against the magic bytes `0x1626ba7e`.

mirage-rs behavior:

- Detects contract accounts by checking for non-empty code at the `from` address.
- Calls `isValidSignature` with a gas limit of 100,000 by default; configurable via `--eip1271-gas-limit`.
- If the contract is absent from fork state (no code at the address), verification fails with `"EIP-1271: no code at address"`.
- Impersonation bypasses this check.

### EIP-712 -- Typed Structured Data

EIP-712 signatures commit to a domain separator:

```
domainSeparator = keccak256(
  EIP712Domain{name, version, chainId, verifyingContract}
)
```

The chain ID in the domain separator must match the chain ID of the fork. Two cases:

1. **Contract reads `chainId` dynamically via the `CHAINID` opcode**: fork-safe, because mirage-rs returns the configured chain ID and the contract recomputes the domain separator at runtime.
2. **Contract hardcodes the domain separator or chain ID at deploy time**: the stored value reflects the chain at deployment time. If you fork with a different `--chain-id`, all EIP-712 signatures for that contract are invalid.

`mirage_computeDomainSeparator(address)` calls the contract and returns the domain separator it currently uses, so you can verify what the contract will validate against.

### EIP-2612 -- Permit

Token permits extend EIP-712. The domain separator includes the token's contract address, so the domain is fork-safe as long as `--chain-id` matches the deployment chain.

Common failure: **deadline expiry**. The permit struct includes a `deadline` field checked against `block.timestamp`. If the fork's block timestamp has advanced past the deadline (stale fork, or test advancing time), the permit reverts with `ERC20Permit: expired deadline`.

Fix: call `evm_setNextBlockTimestamp` with a timestamp before the deadline, then execute.

Permit nonces are tracked per-account inside the token contract, separate from the EOA transaction nonce. To reset a permit nonce: `mirage_setStorageAt` on the token's nonce mapping slot for that account.

### Permit2 (Uniswap)

Permit2 is deployed at a fixed address on every major chain:

```
0x000000000022D473030F116dDEE9F6B43aC78BA3
```

Chains: Ethereum, Base, Arbitrum, Optimism, Polygon, and others. If you fork one of these chains, Permit2 is present in fork state automatically.

If forking Ethereum mainnet before block 15,968,266 (Permit2 deployment block), Permit2 does not exist. Protocols that require it will revert.

Two modes:

- **SignatureTransfer**: one-time transfer authorized by a signature. Nonce is tracked per `(owner, word, bit)` in a bitmap; once used, cannot be replayed.
- **AllowanceTransfer**: persistent allowance with an expiry timestamp. Stored as `(amount, expiration, nonce)` per `(owner, token, spender)`.

If fork state has stale Permit2 allowances (expired or wrong amounts), override them directly via `mirage_setStorageAt` on Permit2's allowance mapping.

Signature format: both EIP-2098 compact signatures and standard `(v, r, s)` are accepted. For contract signers, Permit2 calls `isValidSignature()` -- ensure the contract code is present in fork state.

---

## Gas

### EIP-1559 Base Fee

At fork startup, `baseFeePerGas` is inherited from the forked block header. For each subsequent locally mined block, `baseFeePerGas` evolves per the EIP-1559 adjustment formula: up or down by at most 12.5% depending on whether the previous block was above or below the gas target (half of `gasLimit`).

To override the next block's base fee: `hardhat_setNextBlockBaseFeePerGas(quantity)`.

For type 2 transactions, `maxFeePerGas` must be >= the current block's `baseFeePerGas`. The error is: `max fee per gas less than block base fee`. Fix: call `eth_gasPrice` or `eth_feeHistory` to get the current fee, then set `maxFeePerGas` accordingly.

Setting `gasPrice` on a type 2 transaction is silently ignored. Always set `maxFeePerGas`.

### Gas Estimation Pitfalls

`eth_estimateGas` runs the transaction against the current state and returns the gas used plus a 20% buffer. It can still underestimate in several cases:

- **Conditional code paths**: the execution path during estimation hits a branch that costs less gas than the real execution. Common in multicalls and routers that take different paths based on state.
- **Delegatecall depth**: if estimation does not fully expand delegatecall trees, gas for inner calls may be undercounted.
- **Re-entrancy or callback patterns**: callbacks that execute late in the tx may see different state than during estimation.

When in doubt: use `eth_estimateGas` as a floor, set an explicit limit at 1.5x the estimate, and treat `out of gas` reverts as a signal to investigate rather than a cue to bump the limit.

### Warm/Cold Storage -- EIP-2929

Introduced in the Berlin hardfork. Every storage access (SLOAD, SSTORE) is classified:

- **Cold access** (first access in the tx): 2100 gas for SLOAD, 2100 gas for SSTORE.
- **Warm access** (already accessed in this tx): 100 gas for SLOAD.

Type 1 transactions include an `accessList` that pre-warms specific `(address, storageKey)` pairs at the start of the transaction. Including known-hot slots in the access list reduces per-call cost and can make gas estimation more accurate.

mirage-rs tracks warm/cold state per transaction. `eth_estimateGas` runs a full trace and accounts for warm/cold costs correctly, but only for the initial call -- it does not simulate a transaction where the warm/cold state was already modified by a preceding transaction in the same block. Use `evm_mine` between transactions if you need accurate per-tx estimates.

### Gas Stipend (2300 Gas)

Solidity's `.transfer()` and `.send()` forward exactly 2300 gas to the recipient's fallback or receive function. After EIP-2929 (Berlin), a cold SLOAD in the fallback costs 2100 gas, leaving only 200 gas for everything else. Contracts that relied on `.transfer()` to contract addresses with storage reads in their fallback break after Berlin.

This is correct EVM behavior, not a mirage-rs bug. mirage-rs enforces the 2300 stipend exactly; there is no flag to bypass it.

### Gas Refunds -- EIP-3529

Before EIP-3529 (London): gas refund for clearing storage (SSTORE to 0) was uncapped, allowing transactions to effectively "pay for themselves."

After EIP-3529: max refund = `gasUsed / 5`. SELFDESTRUCT no longer gives a gas refund at all.

Transactions that were gas-neutral or profitable pre-London due to large refunds are now net-cost transactions. mirage-rs applies refund rules based on `--hardfork`; set it to match the fork block.

---

## Nonce Management

### Default Mode (Lenient)

Without `--strict-nonce`, any nonce is accepted including out-of-order, duplicate, and skipped nonces. Useful for scenario testing where execution order matters but nonce tracking is overhead.

### Strict Nonce Mode

With `--strict-nonce`, mirage enforces sequential nonces starting from the account's current confirmed nonce.

- **Nonce too low**: `eth_sendTransaction` returns `-32003 NONCE_TOO_LOW` with message `"expected N, got M"`.
- **Nonce gap** (too high): returns `-32004 NONCE_TOO_HIGH` with message `"expected N, got M"`.

After each successful transaction, mirage-rs increments the account's nonce in DirtyStore. To get the current expected nonce: `eth_getTransactionCount(address, "pending")`.

### Impersonation and Nonces

Impersonated accounts start at their upstream nonce (fetched from the forked chain at the fork block). If `nonce` is omitted from an `eth_sendTransaction` for an impersonated account, mirage-rs auto-fills it with the current expected value, regardless of `--strict-nonce`.

To explicitly set an account's nonce: `anvil_setNonce(address, nonce)`.

### Scenario Runner and Nonces

When replaying a scenario with multiple transactions from the same sender, nonces must be consistent. Options:

1. Omit `nonce` in every tx object -- mirage-rs auto-increments if `--strict-nonce` is off, or uses the correct sequence if on.
2. Explicitly set nonces starting from `eth_getTransactionCount(address, "pending")`.
3. Use `anvil_setNonce` before the scenario starts to reset to a known value.

---

## State Injection Edge Cases

### Storage Slot Calculation

EVM storage is a `bytes32 -> bytes32` map. Slot locations for Solidity types:

| Type | Slot formula |
|------|-------------|
| State variable at position `p` | `p` |
| Mapping `key -> value` at position `p` | `keccak256(abi.encode(key, p))` |
| Nested mapping `key1 -> key2 -> value` at `p` | `keccak256(abi.encode(key2, keccak256(abi.encode(key1, p))))` |
| Dynamic array at position `p` | Length at `p`; element `i` at `keccak256(p) + i` |
| Struct member at slot `p`, offset `o` | Read slot `p`, extract bytes at offset `o` |

**Packed slots**: two or more variables that together fit in 32 bytes share a single slot. `mirage_setStorageAt` writes the full 32-byte slot. Writing a packed slot overwrites all co-packed variables. Use `mirage_mintERC20` or read the slot first, mask the target bytes, then write the modified value.

### `mirage_mintERC20` vs Raw `setStorageAt`

`mirage_mintERC20(tokenAddress, recipientAddress, amount)` finds the correct balance slot by probing known ERC-20 storage layouts and updates the balance and total supply atomically. Prefer it over raw slot manipulation for standard tokens.

Limitations:

- **Rebasing tokens** (stETH, aTokens): balance is derived from shares + a rebase factor. Setting the raw balance slot may not produce the expected `balanceOf` result. Override the shares slot instead.
- **Custom storage layouts**: tokens that don't match ERC-20 conventions require direct slot override. Verify with `eth_call` to `balanceOf` after.
- **Upgradeable ERC-20s**: probe the implementation contract's layout, not the proxy's.

### Proxy and Upgradeable Contracts

**EIP-1967 proxies** store the implementation address in a well-known slot:

```
implementation slot: 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc
admin slot:          0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103
```

To replace the implementation in a fork:

```
mirage_setStorageAt(
  proxyAddress,
  "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc",
  paddedNewImplAddress
)
```

Do not call `mirage_setCode` on the proxy address unless you intend to replace the proxy contract itself. `setCode` replaces bytecode at the address, turning the proxy into a plain contract with the injected code.

**UUPS proxies**: upgradeability logic is in the implementation, not the proxy. The slot layout is the same as EIP-1967; override the implementation slot the same way.

**Transparent proxies**: the admin slot controls who can call `upgradeTo`. You may need to override it to simulate admin actions.

**Diamond proxies (EIP-2535)**: the facet routing table is complex internal storage. Rather than overriding facet mapping slots directly, call the diamond contract normally and let the `DiamondCut` facet handle routing. If you need to add/replace a facet in simulation, call `diamondCut` via an impersonated owner address.

### Code Injection

`mirage_setCode(address, bytecode)` injects arbitrary bytecode. Notes:

- No code size limit enforced by default. EIP-170 limits deployed code to 24,576 bytes; `--strict-eip170` enables this limit.
- Injected code does not initialize storage. Set required slots via `mirage_setStorageAt` after injecting.
- Nonce is not set by `setCode`. A contract address normally has nonce 1 after deployment; if nonce matters for your test, set it via `anvil_setNonce`.

### Account State Initialization

Creating an account by setting balance or code does not auto-populate other fields:

- `mirage_setBalance` on a new address: creates the account with balance, nonce 0, no code.
- `mirage_setCode` on a new address: creates the account with code, balance 0, nonce 0.
- An address with code and nonce 0 is a valid contract.
- An address with nonce > 0 and no code is a valid EOA that has sent transactions.

If you need a fresh EOA with a specific nonce to simulate prior transaction history, use `anvil_setNonce` after `mirage_setBalance`.

---

## EVM Hardfork Compatibility

Set the hardfork via `--hardfork`. The default is `cancun`. If the fork block is from a chain running a different hardfork, silent behavioral divergence occurs: opcodes behave differently, gas costs are wrong, and state transitions may not match what the real chain would produce.

Hardfork progression: `frontier` -> `homestead` -> `tangerine` -> `spurious` -> `byzantium` -> `constantinople` -> `istanbul` -> `berlin` -> `london` -> `paris` -> `shanghai` -> `cancun`

### Critical Transitions

| Hardfork | EIP | Change | mirage impact |
|----------|-----|--------|---------------|
| Berlin | EIP-2929 | Cold/warm storage access costs | Gas estimation must track access list per-tx |
| London | EIP-1559 | `baseFeePerGas` in block header | Type 2 txs required on London+ chains |
| London | EIP-3529 | Gas refund cap at `gasUsed / 5` | High-refund patterns no longer self-fund |
| Paris | EIP-4399 | `DIFFICULTY` opcode -> `PREVRANDAO` | `DIFFICULTY` now returns CL randomness, not PoW difficulty |
| Shanghai | EIP-3855 | `PUSH0` opcode added | Solidity >= 0.8.20 emits `PUSH0`; pre-Shanghai EVM rejects it |
| Cancun | EIP-1153 | `TSTORE`/`TLOAD` transient storage | Wiped at end of tx; not persisted to state |
| Cancun | EIP-5656 | `MCOPY` memory copy opcode | Used by modern Solidity for array/struct copies |
| Cancun | EIP-4844 | Blob transactions, `BLOBHASH` opcode | Blob data not on-chain; type 3 tx envelope |
| Cancun | EIP-4788 | Beacon block root contract | Beacon root available at `0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02` |
| Cancun | EIP-6780 | `SELFDESTRUCT` restricted | Code/storage only deleted if contract was created in same tx |

### Common Hardfork Mismatch Errors

- `"invalid opcode"` -- bytecode uses an opcode unavailable in the configured hardfork. Most often `PUSH0` (0x5f) in bytecode compiled with Solidity >= 0.8.20 running against a pre-Shanghai hardfork.
- `"max fee per gas less than block base fee"` -- type 2 transaction on a pre-London config. Either switch to type 0/1 with `gasPrice`, or set `--hardfork london` or later.
- **Silent gas divergence** -- warm/cold cost tracking is wrong when `--hardfork` is set to pre-Berlin while the contract was deployed and optimized for Berlin+.

### Transient Storage (TSTORE/TLOAD)

Transient storage is scoped to a single transaction. It persists across internal calls (calls, delegatecalls, staticcalls) within the same top-level transaction, but is cleared at the end of that transaction.

Interaction with snapshots: `evm_snapshot` saves the persistent state (storage, balances, code) but not transient storage. After `evm_revert`, transient storage state from the reverted tx is already gone -- it was never persisted -- so there is nothing to restore. This is correct behavior.

mirage-rs clears transient storage at the end of each `eth_sendTransaction` and `eth_call`.

### SELFDESTRUCT Semantics Change -- EIP-6780

Before EIP-6780 (Cancun): `SELFDESTRUCT` deletes the contract's code and storage and sends its balance to the beneficiary.

After EIP-6780: `SELFDESTRUCT` only performs full deletion if the contract was deployed in the same transaction. If the contract was deployed in a prior transaction, `SELFDESTRUCT` only sends the balance; code and storage remain.

Contracts that relied on `SELFDESTRUCT` for one-time cleanup across transactions are silently broken on Cancun. mirage-rs applies EIP-6780 semantics when `--hardfork cancun` is set.

### CREATE2 Collision

`CREATE2` deploys to a deterministic address based on `(deployer, salt, initcode hash)`. If that address already has code -- either from a prior deployment or from `mirage_setCode` -- the deployment reverts with `"contract address collision"`.

To simulate re-deployment to a CREATE2 address, clear the code first: `mirage_setCode(address, "0x")` and set nonce to 0 via `anvil_setNonce`.

---

## Block Environment

### Block Timestamp

On startup, the fork inherits the forked block's timestamp. Subsequent local blocks increment by 12 seconds by default (configurable with `--block-time`).

Time manipulation:

- `evm_setNextBlockTimestamp(ts)` -- sets the exact timestamp for the next mined block.
- `evm_increaseTime(seconds)` -- adds delta to the current block's timestamp baseline.

**Deadline failures**: many DeFi functions take a `deadline` parameter and check `block.timestamp <= deadline`. If the fork's timestamp has advanced past the deadline -- or if the fork started from a stale block -- the transaction reverts. Fix: call `evm_setNextBlockTimestamp` before sending the transaction.

### BLOCKHASH

`BLOCKHASH(n)` returns the hash of block `n`, but only for the most recent 256 blocks. Older blocks return `bytes32(0)`.

In a fork:

- Blocks before the fork point (within the 256-block window): mirage-rs fetches the hash from the upstream RPC on first access and caches it.
- Blocks before the fork point but outside the 256-block window: returns `bytes32(0)`.
- Blocks produced locally after the fork: hash is computed from local block headers.

Contracts that use `BLOCKHASH` for randomness or to anchor proofs to recent blocks may behave incorrectly if the expected block is more than 256 blocks before the current one.

### PREVRANDAO (Post-Merge)

After the Paris hardfork (EIP-4399), the `DIFFICULTY` opcode was repurposed to return the consensus layer's randomness value, now called `PREVRANDAO`.

mirage-rs generates a deterministic pseudo-random value per block by default. To set a specific value: `--prevrandao VALUE` or `anvil_setPrevRandao(value)`.

Contracts that use `PREVRANDAO` for randomness will behave deterministically in simulation, unlike mainnet where the value is beacon-randomness. This is a known simulation limitation.

### COINBASE

The fork inherits the forked block's coinbase by default. To override: `hardhat_setCoinbase(address)` / `anvil_setCoinbase(address)`.

Contracts that grant special permissions or rewards to `block.coinbase` (MEV-related logic) are affected. If you need to simulate a specific builder's behavior, set coinbase to that address.

---

## Upstream RPC and State Fetch

### Lazy Fetch Semantics

mirage-rs uses a copy-on-write state model. On startup, only the block header and any explicitly prefetched slots are loaded. Other storage slots are fetched from the upstream RPC on first access and cached in the clean store.

If the upstream RPC is unreachable when a storage fetch is triggered during transaction execution, the fetch fails and the transaction reverts with:

```
"upstream RPC fetch failed: <address> slot <slot>"
```

To avoid mid-execution fetch failures: call `mirage_prefetchSlots(address, [slot, ...])` or `mirage_prefetchAccount(address)` before executing complex transactions.

### Rate Limiting

Public RPC endpoints (Infura, Alchemy) rate-limit at roughly 100-500 requests per second. Complex transactions can trigger many concurrent slot reads.

mirage-rs configuration:

- `--upstream-rps 100` -- maximum requests per second (default 100).
- `--upstream-burst 200` -- burst budget above the sustained rate (default 200).

If the rate limit is hit, mirage-rs retries with exponential backoff up to 3 times. If all retries fail, the RPC call returns `-32099: "upstream RPC rate limit exceeded"`.

Use a private RPC node or an archive node endpoint with higher rate limits for heavy fork workloads.

### RPC Provider Compatibility

mirage-rs normalizes upstream responses: addresses are lowercased, numbers are decoded from hex, missing optional fields (e.g., `accessList` absent from receipts on some Infura responses) are defaulted to empty.

Known divergences:

- Some providers omit `accessList` from `eth_getTransactionByHash` responses; mirage-rs treats this as an empty access list.
- Erigon and Geth may return slightly different fields in `eth_getBlockByNumber`; mirage-rs handles both.
- `eth_getProof` (EIP-1186, used when `--verify-state-roots` is set) requires an archive node. Light nodes and pruned nodes do not support this endpoint.

---

## DeFi-Specific Concerns

### Chainlink Price Feeds

Chainlink aggregators update on a heartbeat (typically 1 hour on mainnet) or when price deviation exceeds a threshold (typically 0.5%). In a fork, feeds are frozen at fork-block values.

Many contracts check feed freshness:

```solidity
require(block.timestamp - updatedAt < stalenessThreshold, "stale price");
```

If the fork's block timestamp advances far beyond the fork point, this check fails. Two fixes:

1. Override the aggregator's storage to inject a fresh price and `updatedAt` timestamp. Find the `latestRoundData` storage slots in the aggregator contract and use `mirage_setStorageAt` to write current values.
2. Keep the fork timestamp close to the fork block by not advancing time unnecessarily.

### Uniswap V3 TWAP

TWAP is computed from the `observations` array in a Uniswap V3 pool. The array is only updated when ticks cross (when a swap moves price past a tick boundary). In a long-lived fork, the observations are stale and `observe()` returns stale accumulated values.

If you need a specific TWAP price in simulation, override the pool's `observations` array directly via `mirage_setStorageAt`. The array starts at the slot for the `observations` mapping; each entry is a packed struct of `(blockTimestamp, tickCumulative, secondsPerLiquidityCumulativeX128, initialized)`.

Alternatively, advance local blocks with `evm_mine` after swapping to allow the TWAP to accumulate from your simulated state.

### Pool Reserve Drift

Every block the fork runs without syncing to mainnet, pool reserves drift further from the live state. Swap simulations in a fork that is many blocks old produce quotes that don't match what would happen on-chain.

For accurate DeFi simulation: fork from the latest available block, execute immediately, and avoid mining extra blocks unless testing time-dependent logic.

### Permit2 in DeFi Protocols

Most Uniswap-ecosystem protocols (Universal Router, v3 position manager, etc.) use Permit2 for token approvals. Checklist for fork simulation:

1. Permit2 is deployed at `0x000000000022D473030F116dDEE9F6B43aC78BA3`. Verify it is present by calling `eth_getCode` at that address after fork startup.
2. If the fork block predates Permit2's deployment (Ethereum mainnet block 15,968,266), protocols requiring Permit2 revert with no code at that address.
3. If the user's Permit2 allowance from mainnet is stale or expired, override the allowance slot via `mirage_setStorageAt`.
4. `SignatureTransfer` nonce bitmaps are in storage; if you need a fresh nonce, override the bitmap slot to unset the bit.

---

## Error Reference

| Error message | Code | Root cause | Fix |
|--------------|------|-----------|-----|
| `cannot estimate gas; transaction may fail` | -32000 | `eth_estimateGas` hit a different code path than actual execution | Set manual gas limit at 1.5x estimate; inspect call trace |
| `max fee per gas less than block base fee` | -32000 | Type 2 tx `maxFeePerGas` below current `baseFeePerGas` | Call `eth_feeHistory`; set `maxFeePerGas >= baseFeePerGas` |
| `invalid chain id` | -32000 | EIP-155 `v` encodes wrong chain ID | Match `--chain-id` to the value used when signing |
| `nonce too low` | -32003 | Tx nonce < account's current confirmed nonce | Call `eth_getTransactionCount(addr, "pending")` for correct nonce |
| `nonce too high` | -32004 | Gap in nonce sequence (with `--strict-nonce`) | Submit missing nonces first, or omit nonce to auto-fill |
| `invalid from` | -32010 | From address not impersonated and no private key loaded | Call `hardhat_impersonateAccount(addr)` before sending |
| `execution reverted` | -32015 | EVM hit `revert`, `require`, or custom error | Decode revert data; use `eth_call` to isolate the failure |
| `out of gas` | -32015 | Gas limit too low for actual execution | Increase gas limit; check for cold SLOAD costs in fallback |
| `invalid opcode` | -32015 | Opcode not available at configured hardfork | Set `--hardfork` to match the fork block's actual hardfork |
| `contract address collision` | -32015 | CREATE2 target address already has code | Clear code via `mirage_setCode(addr, "0x")` and reset nonce |
| `EIP-1271: no code at address` | -32015 | Contract wallet signer has no code in fork state | Inject contract code via `mirage_setCode`, or use impersonation |
| `upstream RPC fetch failed` | -32099 | State slot fetch triggered during execution, upstream unreachable | Prefetch slots before execution; check RPC connectivity |
| `upstream RPC rate limit exceeded` | -32099 | Upstream rate limit hit after 3 retries | Use private/archive RPC node; reduce `--upstream-rps` |
| `transaction deadline exceeded` | revert | `block.timestamp > deadline` in contract logic | Call `evm_setNextBlockTimestamp` before sending |
| `stale price feed` | revert | Chainlink `block.timestamp - updatedAt >= staleness threshold` | Override aggregator storage with fresh price + timestamp |
| `allowance too low` | revert | ERC-20 or Permit2 allowance insufficient | Set approval or override allowance storage slot directly |
| `ERC20Permit: expired deadline` | revert | Permit deadline < current `block.timestamp` | Call `evm_setNextBlockTimestamp` to a time before deadline |
| `SELFDESTRUCT did not delete code` | (behavior) | EIP-6780: cross-tx SELFDESTRUCT no longer clears code | Expected on Cancun; clear code manually if needed |

---

## RPC Methods Referenced

The following RPC methods are documented or referenced in this specification:

- `eth_sendTransaction` -- submit a transaction for local execution
- `eth_sendRawTransaction` -- submit a pre-signed transaction
- `eth_estimateGas` -- estimate gas for a transaction
- `eth_gasPrice` -- get current gas price
- `eth_feeHistory` -- get historical fee data
- `eth_getTransactionCount` -- get account nonce

---

## Cross-References

- [01-mirage-rs.md](./01-mirage-rs.md) -- Core architecture: HybridDB three-tier database, DirtyStore dirty tracking, upstream fetch model, CoW state layers
- [01b-mirage-rpc.md](./01b-mirage-rpc.md) -- Full JSON-RPC method catalog with parameter specs for all methods referenced in this document
- [01c-mirage-scenarios.md](./01c-mirage-scenarios.md) -- Scenario runner, historical replay modes, targeted follower classification rules
- [01d-mirage-integration.md](./01d-mirage-integration.md) -- CorticalState integration, golem sidecar lifecycle, F6 fork workflow, resource pressure gating

## References

- Gelashvili, R. et al. (2023). "Block-STM: Scaling Blockchain Execution by Turning Ordering Curse to a Performance Blessing." arXiv (Aptos Labs). -- *Introduces optimistic parallel transaction execution with per-slot version tracking; referenced for the parallel replay mode's conflict detection semantics.*
- Saraph, V. & Herlihy, M. (2019). "An Empirical Study of Speculative Concurrency in Ethereum Smart Contracts." arXiv:1901.01376. -- *Measures read-write conflict rates across real Ethereum blocks; the <5% conflict rate on typical DeFi blocks justifies parallel execution assumptions in the transaction compatibility model.*
