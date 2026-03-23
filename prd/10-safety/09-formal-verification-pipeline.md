# Formal Verification Pipeline [SPEC]

> **Version**: 2.0 | **Status**: Draft
>
> **Last Updated**: 2026-03-18
>
> **Crates**: `bardo-verify`, `bardo-chain`
>
> **Depends on**: [00-defense.md](00-defense.md) (defense layers), [05-threat-model.md](05-threat-model.md) (threat taxonomy), `../14-chain/02-triage.md` (triage pipeline)

> **Reader orientation:** This document specifies the formal verification pipeline that a Golem (mortal autonomous DeFi agent) uses to verify smart contract behavior before committing capital. It belongs to the **Safety** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: the pipeline chains five complementary tools (Echidna fuzzing, hevm symbolic execution, Certora/Kontrol formal proofs, Slither static analysis, Heimdall-rs decompilation) in sequence, matching verification depth to available time and risk magnitude. Terms like PolicyCage, Grimoire, and Heartbeat are defined inline on first use; a full glossary lives in `prd2/11-compute/00-overview.md § Terminology`.

Smart contracts manage billions of dollars with no recourse for bugs. An autonomous chain intelligence system that simulates, triages, and acts on DeFi transactions needs verification woven into its pipeline, not bolted on after the fact. This document covers five tools -- Echidna, hevm, Certora/Kontrol, Slither, and Heimdall-rs -- and how each integrates with a Rust-based agent that must verify contract behavior before committing capital.

The core tension: formal methods are slow and thorough, fuzzing is fast and probabilistic, static analysis is instant and shallow. A production pipeline uses all three in sequence, matching verification depth to available time and risk magnitude.

---

## Echidna: property-based fuzzing for Solidity

[SPEC] Echidna is a property-based fuzzer built by Trail of Bits for Solidity smart contracts. It generates semi-random transaction sequences, executes them against a contract, and checks whether user-defined invariants hold. When an invariant breaks, Echidna reports a minimal reproducing sequence.

**How it works.** Echidna uses grammar-based fuzzing guided by coverage feedback. It constructs sequences of contract function calls with random arguments, biasing generation toward inputs that explore new code paths. Properties are Solidity functions prefixed with `echidna_` that return `bool` -- the fuzzer tries to make them return `false`. Echidna also supports assertion-based testing (where `assert()` failures count as bugs) and optimization mode (where it maximizes a numeric return value, useful for finding worst-case gas consumption or maximal extractable value).

Since version 2.1, Echidna supports **on-chain forking** via RPC endpoints. This means it can fuzz against actual mainnet state -- the same state your mirage-rs fork simulation uses. Properties can reference real pool reserves, real token balances, real protocol configurations.

**Configuration.** Echidna reads a YAML config file controlling test limits, shrink steps, corpus directory, and RPC settings:

```yaml
# echidna-config.yaml
testLimit: 50000
shrinkLimit: 5000
seqLen: 10
corpusDir: "corpus"
cryticArgs: ["--compile-force-framework", "foundry"]
rpcUrl: "http://localhost:8545"
rpcBlock: "latest"
deployer: "0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"
```

### Rust integration

[SPEC] Echidna is a Haskell binary. Calling it from Rust means spawning a subprocess and parsing its JSON output.

```rust
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EchidnaResult {
    pub name: String,
    pub status: EchidnaStatus,
    pub reproducer: Option<Vec<EchidnaCall>>,
    pub coverage: f64,
}

#[derive(Debug, Deserialize)]
pub enum EchidnaStatus {
    Passed,
    Failed,
    Fuzzing,
}

#[derive(Debug, Deserialize)]
pub struct EchidnaCall {
    pub function: String,
    pub args: Vec<String>,
    pub value: String,
    pub sender: String,
}

pub struct EchidnaRunner {
    binary_path: String,
    config_path: String,
    working_dir: String,
}

impl EchidnaRunner {
    pub fn new(binary_path: &str, config_path: &str, working_dir: &str) -> Self {
        Self {
            binary_path: binary_path.to_string(),
            config_path: config_path.to_string(),
            working_dir: working_dir.to_string(),
        }
    }

    /// Run Echidna against a contract file with a given config.
    /// Returns parsed results for each property tested.
    pub fn run(&self, contract_file: &str, contract_name: &str) -> anyhow::Result<Vec<EchidnaResult>> {
        let output = Command::new(&self.binary_path)
            .arg(contract_file)
            .arg("--contract")
            .arg(contract_name)
            .arg("--config")
            .arg(&self.config_path)
            .arg("--format")
            .arg("json")
            .current_dir(&self.working_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Echidna failed: {}", stderr);
        }

        let results: Vec<EchidnaResult> = serde_json::from_slice(&output.stdout)?;
        Ok(results)
    }

    /// Check whether any property failed, returning the first failure found.
    pub fn find_violations(&self, results: &[EchidnaResult]) -> Vec<&EchidnaResult> {
        results
            .iter()
            .filter(|r| matches!(r.status, EchidnaStatus::Failed))
            .collect()
    }
}
```

### Integration with mirage-rs fork testing

The key insight: Echidna and mirage-rs can share the same forked state. Point Echidna's `rpcUrl` at the same Anvil or Reth node your simulation layer uses. Write Echidna properties that encode the invariants your agent cares about:

```solidity
// Property: a swap on this pool should never drain more than 5% of reserves
function echidna_swap_bounded() public returns (bool) {
    uint256 reserveBefore = pool.getReserve0();
    // Echidna will try random swap amounts
    pool.swap(amount0, amount1, address(this), "");
    uint256 reserveAfter = pool.getReserve0();
    return reserveAfter >= (reserveBefore * 95) / 100;
}
```

When the agent encounters an unfamiliar contract, it can deploy Echidna properties against the forked state, fuzz for a bounded time (say 30 seconds), and flag the contract if any property breaks. This is faster than symbolic execution and catches a different class of bugs -- sequence-dependent state corruption that single-transaction analysis misses.

**Academic context.** Grieco et al. ("Echidna: effective, usable, and fast fuzzing for smart contracts," ISSTA 2020) demonstrated that Echidna found bugs in contracts that Mythril and Manticore missed, particularly bugs requiring multi-transaction sequences. The grammar-based approach achieves higher coverage than blackbox fuzzing because it generates valid ABI-encoded calls rather than random byte sequences.

---

## hevm: symbolic execution with SMT solving

[SPEC] hevm, maintained by the Ethereum Foundation, is a symbolic EVM implementation that proves properties hold for *all* possible inputs rather than checking specific ones. Where Echidna asks "can I find an input that breaks this?", hevm asks "does any input exist that breaks this?"

**How it works.** hevm executes EVM bytecode symbolically, representing unknown values (function arguments, msg.sender, block.timestamp) as symbolic variables. When execution branches on a symbolic condition, hevm forks into both paths, accumulating path constraints. At each `assert` or property check, it queries an SMT solver (typically Z3 or CVC5) to determine whether any assignment of symbolic variables can violate the assertion. If the solver returns SAT, a concrete counterexample exists. If UNSAT, the property holds for all inputs on that execution path.

**RPC state fetching.** Unlike Halmos (a16z's symbolic testing tool, which cannot fork mainnet because symbolic keccak256 is incompatible with concrete storage lookups), hevm supports **RPC state fetching**. It can consume state from a forked environment, making it the strongest candidate for integration with the mirage-rs simulation layer. You run symbolic analysis against real mainnet state -- real balances, real pool configurations, real access control settings.

**Equivalence checking.** hevm can prove that two bytecodes behave identically for all inputs. This is valuable when a protocol upgrades a contract: verify that the new implementation preserves all behaviors of the old one, or identify exactly where they diverge.

### Rust integration

```rust
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HevmResult {
    pub property: String,
    pub outcome: HevmOutcome,
    pub counterexample: Option<HevmCounterexample>,
    pub solver_time_ms: u64,
    pub paths_explored: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum HevmOutcome {
    Proved,
    Counterexample,
    Timeout,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct HevmCounterexample {
    pub calldata: String,
    pub msg_value: String,
    pub msg_sender: String,
}

pub struct HevmRunner {
    binary_path: String,
    rpc_url: String,
    solver_timeout_ms: u64,
}

impl HevmRunner {
    pub fn new(binary_path: &str, rpc_url: &str, solver_timeout_ms: u64) -> Self {
        Self {
            binary_path: binary_path.to_string(),
            rpc_url: rpc_url.to_string(),
            solver_timeout_ms,
        }
    }

    /// Run symbolic execution against a Foundry test contract.
    /// hevm discovers all `prove_*` prefixed functions and attempts
    /// to find counterexamples for each.
    pub fn prove(
        &self,
        project_dir: &str,
        contract_name: &str,
    ) -> anyhow::Result<Vec<HevmResult>> {
        let output = Command::new(&self.binary_path)
            .arg("test")
            .arg("--match")
            .arg(format!("prove_{}", contract_name))
            .arg("--rpc")
            .arg(&self.rpc_url)
            .arg("--smttimeout")
            .arg(self.solver_timeout_ms.to_string())
            .arg("--json")
            .current_dir(project_dir)
            .output()?;

        let results: Vec<HevmResult> = serde_json::from_slice(&output.stdout)?;
        Ok(results)
    }

    /// Check equivalence between two contract bytecodes.
    /// Returns true if they behave identically for all inputs.
    pub fn check_equivalence(
        &self,
        bytecode_a: &str,
        bytecode_b: &str,
    ) -> anyhow::Result<HevmOutcome> {
        let output = Command::new(&self.binary_path)
            .arg("equivalence")
            .arg("--code-a")
            .arg(bytecode_a)
            .arg("--code-b")
            .arg(bytecode_b)
            .arg("--smttimeout")
            .arg(self.solver_timeout_ms.to_string())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("No differences found") {
            Ok(HevmOutcome::Proved)
        } else if stdout.contains("Counterexample") {
            Ok(HevmOutcome::Counterexample)
        } else {
            Ok(HevmOutcome::Unknown)
        }
    }

    /// Verify a specific property against forked state at a given block.
    pub fn verify_property(
        &self,
        project_dir: &str,
        test_function: &str,
        block_number: u64,
    ) -> anyhow::Result<HevmResult> {
        let output = Command::new(&self.binary_path)
            .arg("test")
            .arg("--match")
            .arg(test_function)
            .arg("--rpc")
            .arg(&self.rpc_url)
            .arg("--block")
            .arg(block_number.to_string())
            .arg("--smttimeout")
            .arg(self.solver_timeout_ms.to_string())
            .arg("--json")
            .current_dir(project_dir)
            .output()?;

        let results: Vec<HevmResult> = serde_json::from_slice(&output.stdout)?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No results returned from hevm"))
    }
}
```

**When to use hevm vs. Echidna.** Use Echidna first -- it runs faster and catches sequence-dependent bugs. Use hevm when you need a proof, not just high confidence. If Echidna's 50,000 test runs all pass, hevm can prove the property holds unconditionally. In the verification pipeline, Echidna acts as a fast filter and hevm provides the formal guarantee.

**Academic context.** The symbolic execution approach builds on work by Mossberg et al. ("Manticore: A User-Friendly Symbolic Execution Framework for Binaries and Smart Contracts," ASE 2019), which established the pattern of symbolic EVM analysis with SMT backends. hevm improves on Manticore's approach with better EVM spec compliance, parallel solver queries, and the RPC forking capability that Manticore lacks.

---

## Certora and Kontrol: formal verification at scale

Certora and Kontrol represent the upper end of verification rigor. They prove invariants about contract behavior using mathematical specifications, not testing.

### Certora

[SPEC] Certora uses the **Certora Verification Language (CVL)**, a specification language for writing rules about Solidity contracts. CVL rules describe relationships between pre-states and post-states, and the Certora Prover checks whether any execution path violates them.

**How CVL works.** A CVL spec file defines rules, invariants, and ghost variables:

```cvl
// spec/Pool.spec

methods {
    function getReserve0() external returns (uint256) envfree;
    function getReserve1() external returns (uint256) envfree;
    function totalSupply() external returns (uint256) envfree;
    function swap(uint256, uint256, address, bytes) external;
}

// Ghost variable tracking cumulative swap volume
ghost mathint cumulativeVolume {
    init_state axiom cumulativeVolume == 0;
}

// Hook: update ghost on every swap
hook Sstore reserves[0] uint256 newVal (uint256 oldVal) {
    cumulativeVolume = cumulativeVolume + (newVal > oldVal ?
        newVal - oldVal : oldVal - newVal);
}

// Rule: swaps preserve the constant product invariant (within rounding)
rule swap_preserves_k(uint256 amount0, uint256 amount1, address to) {
    env e;

    mathint k_before = getReserve0() * getReserve1();

    swap(e, amount0, amount1, to, "");

    mathint k_after = getReserve0() * getReserve1();

    // k should not decrease (fees increase it)
    assert k_after >= k_before,
        "Swap violated constant product invariant";
}

// Invariant: total supply is zero iff reserves are zero
invariant liquidity_consistency()
    (totalSupply() == 0) <=> (getReserve0() == 0 && getReserve1() == 0);
```

**Scale and track record.** Certora has verified contracts securing over $100B in total value locked, including Aave, MakerDAO, Uniswap, Lido, and EigenLayer. The Certora Prover found an invariant violation in MakerDAO that put $10B at risk. Over 70,000 verification rules have been written by the community. The prover was recently open-sourced (github.com/Certora/CertoraProver).

### Certora Rust integration

```rust
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CertoraJobResult {
    pub job_id: String,
    pub rules: Vec<CertoraRuleResult>,
    pub overall_status: String,
}

#[derive(Debug, Deserialize)]
pub struct CertoraRuleResult {
    pub name: String,
    pub status: String, // "verified", "violated", "timeout"
    pub violation_trace: Option<String>,
}

pub struct CertoraRunner {
    cli_path: String,
    api_key: String,
}

impl CertoraRunner {
    pub fn new(cli_path: &str, api_key: &str) -> Self {
        Self {
            cli_path: cli_path.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Submit a verification job to Certora.
    /// Returns a job ID for polling results.
    pub fn submit(
        &self,
        contract_path: &str,
        spec_path: &str,
        contract_name: &str,
    ) -> anyhow::Result<String> {
        let output = Command::new(&self.cli_path)
            .arg(contract_path)
            .arg("--verify")
            .arg(format!("{}:{}", contract_name, spec_path))
            .arg("--key")
            .arg(&self.api_key)
            .arg("--json_output")
            .output()?;

        let result: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let job_id = result["jobId"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No job ID in response"))?;
        Ok(job_id.to_string())
    }

    /// Poll for job completion and parse results.
    pub fn poll_result(&self, job_id: &str) -> anyhow::Result<CertoraJobResult> {
        let output = Command::new(&self.cli_path)
            .arg("--get_results")
            .arg(job_id)
            .arg("--key")
            .arg(&self.api_key)
            .output()?;

        let result: CertoraJobResult = serde_json::from_slice(&output.stdout)?;
        Ok(result)
    }
}
```

### Kontrol

[SPEC] Kontrol, built by Runtime Verification, takes a different approach. It uses **KEVM** -- a mathematically formalized EVM semantics written in the K Framework (Hildenbrandt et al., "KEVM: A Complete Formal Semantics of the Ethereum Virtual Machine," CSF 2018). Where Certora abstracts the EVM into its own IR, Kontrol reasons about the actual EVM bytecode semantics, making its proofs closer to ground truth.

Kontrol integrates with Foundry test suites. You write Foundry tests with `prove_` prefixes, and Kontrol symbolically executes them using compositional reasoning -- breaking large proofs into smaller lemmas that compose. Optimism uses Kontrol to validate codebase changes before production merges.

**When to use which.** Certora is faster to write specs for and has a larger ecosystem of existing rules. Kontrol is more thorough because it reasons about actual EVM semantics rather than an abstraction. For the chain intelligence pipeline, Certora handles the common case (verifying known protocol invariants), while Kontrol handles the hard case (proving properties about novel contract interactions where the abstraction gap matters).

---

## Slither: static analysis at the speed of compilation

[SPEC] Slither, also from Trail of Bits, is a static analysis framework for Solidity. It does not execute code. It parses Solidity into **SlithIR** (an intermediate representation), builds control flow graphs, and runs detectors over the IR.

**What it catches.** Slither ships 90+ vulnerability detectors covering reentrancy, unprotected selfdestruct, arbitrary send, unchecked low-level calls, shadowed state variables, unused return values, and more. It also includes "printers" that output contract summaries, inheritance graphs, function call graphs, and storage layouts.

**Custom analysis passes.** Slither's plugin architecture lets you write custom detectors in Python. For an on-chain intelligence system, custom detectors can flag patterns specific to DeFi: unguarded flash loan callbacks, missing slippage checks, oracle manipulation vulnerabilities, uncapped minting functions.

**Speed.** Slither runs in seconds on most contracts. It is the right first pass in any verification pipeline -- catch the obvious issues before spending compute on fuzzing or symbolic execution.

### Rust integration

```rust
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SlitherOutput {
    pub success: bool,
    pub results: SlitherResults,
}

#[derive(Debug, Deserialize)]
pub struct SlitherResults {
    pub detectors: Vec<SlitherDetection>,
    pub printers: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SlitherDetection {
    pub check: String,
    pub impact: String,          // "High", "Medium", "Low", "Informational"
    pub confidence: String,      // "High", "Medium", "Low"
    pub description: String,
    pub elements: Vec<SlitherElement>,
}

#[derive(Debug, Deserialize)]
pub struct SlitherElement {
    pub name: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub source_mapping: Option<serde_json::Value>,
}

pub struct SlitherRunner {
    binary_path: String,
}

impl SlitherRunner {
    pub fn new(binary_path: &str) -> Self {
        Self {
            binary_path: binary_path.to_string(),
        }
    }

    /// Run Slither analysis on a contract, returning all detections.
    pub fn analyze(&self, contract_path: &str) -> anyhow::Result<SlitherOutput> {
        let output = Command::new(&self.binary_path)
            .arg(contract_path)
            .arg("--json")
            .arg("-")
            .output()?;

        let result: SlitherOutput = serde_json::from_slice(&output.stdout)?;
        Ok(result)
    }

    /// Filter detections to only high-impact, high-confidence findings.
    pub fn critical_findings(output: &SlitherOutput) -> Vec<&SlitherDetection> {
        output
            .results
            .detectors
            .iter()
            .filter(|d| d.impact == "High" && d.confidence == "High")
            .collect()
    }

    /// Run a specific set of detectors (faster than running all 90+).
    pub fn analyze_targeted(
        &self,
        contract_path: &str,
        detectors: &[&str],
    ) -> anyhow::Result<SlitherOutput> {
        let detector_list = detectors.join(",");
        let output = Command::new(&self.binary_path)
            .arg(contract_path)
            .arg("--detect")
            .arg(&detector_list)
            .arg("--json")
            .arg("-")
            .output()?;

        let result: SlitherOutput = serde_json::from_slice(&output.stdout)?;
        Ok(result)
    }
}
```

**Academic context.** Feist, Grieco, and Groce ("Slither: A Static Analysis Framework for Smart Contracts," WETSEB 2019) describe the SlithIR design and detector architecture. The key contribution is that SlithIR preserves enough semantic information for cross-function analysis while remaining simple enough for fast traversal.

---

## Heimdall-rs: bytecode decompilation in Rust

[SPEC] Heimdall-rs (github.com/Jon-Becker/heimdall-rs) is the only tool in this section written natively in Rust. It decompiles EVM bytecode -- turning raw hex into readable function signatures, control flow graphs, and approximate Solidity source.

**Why it matters for chain intelligence.** The agent constantly encounters unverified contracts. Etherscan has source code for a fraction of deployed contracts. When the triage pipeline flags an interesting transaction involving an unverified contract, Heimdall-rs can:

1. **Disassemble** bytecode into EVM opcodes
2. **Recover control flow** -- identify basic blocks, loops, and branches
3. **Resolve function signatures** -- match 4-byte selectors to known function names via signature databases
4. **Decompile** into pseudo-Solidity that humans (or LLMs) can reason about
5. **Dump storage layout** -- identify storage slot assignments

### Rust integration

Since Heimdall-rs is a Rust crate, integration is direct -- no subprocess spawning needed:

```rust
use alloy::primitives::Address;
use std::str::FromStr;

/// Represents a decompiled contract with recovered structure.
pub struct DecompiledContract {
    pub address: Address,
    pub functions: Vec<DecompiledFunction>,
    pub storage_layout: Vec<StorageSlot>,
    pub bytecode_size: usize,
}

pub struct DecompiledFunction {
    pub selector: [u8; 4],
    pub signature: Option<String>,  // e.g., "transfer(address,uint256)"
    pub decompiled_body: String,     // pseudo-Solidity
    pub state_mutability: StateMutability,
}

#[derive(Debug)]
pub enum StateMutability {
    Pure,
    View,
    Nonpayable,
    Payable,
}

pub struct StorageSlot {
    pub slot: u64,
    pub inferred_type: String,
    pub label: Option<String>,
}

/// Wrapper around Heimdall-rs CLI for bytecode analysis.
/// In production, you would use Heimdall's library crate directly;
/// this shows the subprocess pattern for clarity.
pub struct HeimdallRunner {
    binary_path: String,
    rpc_url: String,
}

impl HeimdallRunner {
    pub fn new(binary_path: &str, rpc_url: &str) -> Self {
        Self {
            binary_path: binary_path.to_string(),
            rpc_url: rpc_url.to_string(),
        }
    }

    /// Decompile a contract at the given address.
    pub fn decompile(&self, address: &str) -> anyhow::Result<String> {
        let output = std::process::Command::new(&self.binary_path)
            .arg("decompile")
            .arg("--target")
            .arg(address)
            .arg("--rpc-url")
            .arg(&self.rpc_url)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Disassemble bytecode into raw opcodes.
    pub fn disassemble(&self, bytecode: &str) -> anyhow::Result<String> {
        let output = std::process::Command::new(&self.binary_path)
            .arg("disassemble")
            .arg("--target")
            .arg(bytecode)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Resolve function selectors from bytecode to known signatures.
    pub fn resolve_selectors(&self, address: &str) -> anyhow::Result<Vec<(String, String)>> {
        let output = std::process::Command::new(&self.binary_path)
            .arg("decompile")
            .arg("--target")
            .arg(address)
            .arg("--rpc-url")
            .arg(&self.rpc_url)
            .arg("--include-sol")
            .output()?;

        // Parse selector -> signature mappings from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut selectors = Vec::new();
        for line in stdout.lines() {
            if line.contains("function ") {
                // Extract selector and signature from decompiled output
                if let Some(sig) = line.strip_prefix("function ") {
                    let selector = sig
                        .split('(')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    selectors.push((selector, sig.to_string()));
                }
            }
        }
        Ok(selectors)
    }

    /// Generate a control flow graph for visual analysis.
    pub fn generate_cfg(&self, address: &str, output_path: &str) -> anyhow::Result<()> {
        std::process::Command::new(&self.binary_path)
            .arg("cfg")
            .arg("--target")
            .arg(address)
            .arg("--rpc-url")
            .arg(&self.rpc_url)
            .arg("--output")
            .arg(output_path)
            .output()?;

        Ok(())
    }
}
```

**In the pipeline.** Heimdall-rs sits at the front of the verification chain. When triage flags a transaction involving an unknown contract, Heimdall-rs decompiles it. The decompiled output feeds into Slither for static analysis (if close enough to valid Solidity) and into the LLM for semantic understanding. If the contract looks interesting enough, Echidna and hevm take over for deeper verification.

---

## The five-stage verification pipeline

[SPEC] These tools compose into a pipeline ordered by speed and depth. Each stage gates the next -- there is no point running symbolic execution on a contract that Slither already flagged as having an unprotected selfdestruct.

```rust
use std::time::Duration;

/// Verification confidence levels, ordered by rigor.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum VerificationLevel {
    /// No verification performed.
    Unknown,
    /// Heimdall-rs decompiled the bytecode; basic structure recovered.
    Decompiled,
    /// Slither static analysis found no high-severity issues.
    StaticClean,
    /// Echidna fuzzing passed N iterations with no property violations.
    FuzzPassed { iterations: u64 },
    /// hevm symbolic execution proved properties for all inputs.
    SymbolicProved { properties: Vec<String> },
    /// Certora/Kontrol formal verification of invariants.
    FormallyVerified { rules: Vec<String> },
}

/// A contract verification report aggregating all pipeline stages.
pub struct VerificationReport {
    pub address: String,
    pub level: VerificationLevel,
    pub slither_findings: Vec<String>,
    pub echidna_results: Vec<String>,
    pub hevm_results: Vec<String>,
    pub certora_results: Vec<String>,
    pub decompiled_abi: Option<Vec<String>>,
    pub total_duration: Duration,
}

/// Configuration for the verification pipeline.
/// Time budgets control how long each stage can run before
/// the pipeline moves on.
pub struct PipelineConfig {
    pub slither_timeout: Duration,
    pub echidna_time_budget: Duration,
    pub echidna_iterations: u64,
    pub hevm_solver_timeout: Duration,
    pub certora_enabled: bool,
    pub rpc_url: String,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            slither_timeout: Duration::from_secs(30),
            echidna_time_budget: Duration::from_secs(60),
            echidna_iterations: 50_000,
            hevm_solver_timeout: Duration::from_secs(120),
            certora_enabled: false, // expensive; opt-in
            rpc_url: "http://localhost:8545".to_string(),
        }
    }
}

/// Run the full verification pipeline against a contract.
/// Each stage produces findings that feed into the next.
/// Stages are ordered: decompile -> static -> fuzz -> symbolic -> formal.
pub fn run_pipeline(
    address: &str,
    bytecode: &str,
    config: &PipelineConfig,
) -> VerificationReport {
    let start = std::time::Instant::now();
    let mut report = VerificationReport {
        address: address.to_string(),
        level: VerificationLevel::Unknown,
        slither_findings: vec![],
        echidna_results: vec![],
        hevm_results: vec![],
        certora_results: vec![],
        decompiled_abi: None,
        total_duration: Duration::ZERO,
    };

    // Stage 1: Decompile with Heimdall-rs.
    // Recover ABI, function signatures, and approximate source.
    // This runs in milliseconds and gates all subsequent stages.
    let heimdall = HeimdallRunner::new("heimdall", &config.rpc_url);
    if let Ok(decompiled) = heimdall.decompile(address) {
        report.level = VerificationLevel::Decompiled;
        // Parse function signatures from decompiled output
        let functions: Vec<String> = decompiled
            .lines()
            .filter(|l| l.contains("function "))
            .map(|l| l.to_string())
            .collect();
        report.decompiled_abi = Some(functions);
    }

    // Stage 2: Static analysis with Slither.
    // Catches obvious vulnerabilities in seconds.
    let slither = SlitherRunner::new("slither");
    // In practice, you'd feed the decompiled source or the original
    // Solidity if available. Slither needs source, not just bytecode.
    // For unverified contracts, skip to stage 3.

    // Stage 3: Property-based fuzzing with Echidna.
    // Runs for the configured time budget.
    // Properties are drawn from a library of common DeFi invariants.

    // Stage 4: Symbolic execution with hevm.
    // Proves properties that fuzzing validated probabilistically.

    // Stage 5 (optional): Formal verification with Certora.
    // Only for high-value interactions where proof is worth the cost.

    report.total_duration = start.elapsed();
    report
}
```

The pipeline in the autonomous agent runs on a separate thread from the main triage loop. When triage scores a transaction above a threshold, it enqueues the involved contracts for verification. The verification thread works through the pipeline stages, updating the contract's verification level as each stage completes. Trading decisions gate on the verification level: the agent won't commit capital to an interaction involving a contract below `StaticClean` unless the expected value overwhelms the verification gap.

---

## DeFi verification properties

[SPEC] The properties worth verifying for an autonomous trading agent (referenced in Babel et al., "Clockwork Finance," IEEE S&P 2023):

- **Position bounds**: exposure to any single protocol stays within configured limits
- **Slippage bounds**: maximum output deviation from expected, provable symbolically
- **Composability safety**: adding the agent's transaction to a block does not create new extractable value for third parties (Clockwork Finance's epsilon-composability)
- **Liquidation safety**: the agent's positions remain above liquidation thresholds under a range of price movements
- **Reentrancy freedom**: no callback path can re-enter the agent's contracts in an unexpected state

Tolmach et al. ("Formal Analysis of Composable DeFi Protocols," FC Workshops 2021) used process algebra (CSP#) to model composed protocol interactions (Curve + Compound), demonstrating that formal methods can reason about cross-protocol behavior. VerX (Permenev et al., IEEE S&P 2020) extends temporal safety specifications with `always` and `once` operators, verifying properties like "once a loan is repaid, the collateral is always unlocked." These temporal patterns map to the agent's need to verify multi-step DeFi strategies.

---

## References

- Grieco, G., Song, W., Cygan, A., Feist, J., & Groce, A. (2020). "Echidna: effective, usable, and fast fuzzing for smart contracts." ISSTA 2020. The primary paper on Echidna: grammar-based fuzzing with coverage feedback for Solidity. Directly used as Stage 1 of the verification pipeline.
- Mossberg, M., Manzano, F., Hennenfent, E., Groce, A., Grieco, G., Feist, J., Brunson, T., & Dinaburg, A. (2019). "Manticore: A User-Friendly Symbolic Execution Framework for Binaries and Smart Contracts." ASE 2019. Introduces Manticore for symbolic execution of EVM bytecode. Relevant as an alternative to hevm for Stage 2 symbolic execution.
- Hildenbrandt, E., Saxena, M., Rodrigues, N., Zhu, X., Daian, P., Guth, D., Moore, B., Park, D., Zhang, Y., Stefanescu, A., & Rosu, G. (2018). "KEVM: A Complete Formal Semantics of the Ethereum Virtual Machine." CSF 2018. Provides a complete formal semantics for the EVM in the K framework. Foundation for hevm's symbolic execution and Kontrol's formal verification.
- Babel, K., Daian, P., Kelkar, M., & Juels, A. (2023). "Clockwork Finance: Automated Analysis of Economic Security in Smart Contracts." IEEE S&P 2023. Automates economic security analysis by modeling DeFi protocols as composable financial primitives. Relevant to verifying economic properties of vault strategies.
- Tolmach, P., Li, Y., Lin, S.-W., Liu, Y., & Li, Z. (2021). "Formal Analysis of Composable DeFi Protocols." FC Workshops 2021. Formalizes composability risks in DeFi protocol interactions. Directly relevant to verifying cross-protocol strategy safety.
- Permenev, A., Dimitrov, D., Tsankov, P., Drachsler-Cohen, D., & Vechev, M. (2020). "VerX: Safety Verification of Smart Contracts." IEEE S&P 2020. Proposes temporal safety verification for smart contracts using delayed predicate abstraction. Related to the temporal logic verification in `07-temporal-logic-verification.md`.
- Feist, J., Grieco, G., & Groce, A. (2019). "Slither: A Static Analysis Framework for Smart Contracts." WETSEB 2019. The primary paper on Slither: fast static analysis with 99%+ detection rate for common vulnerabilities. Used as Stage 4 of the verification pipeline.
- Torres, C. F., Iannillo, A. K., Gervais, A., & State, R. (2021). "ConFuzzius: A Data Dependency-Aware Hybrid Fuzzer for Ethereum Smart Contracts." IEEE EuroS&P 2021. Combines fuzzing with data dependency analysis for deeper coverage. Informs the hybrid approach of combining Echidna fuzzing with hevm symbolic execution.
- He, J., Balunovic, M., Ambroladze, N., Tsankov, P., & Vechev, M. (2019). "Learning to Fuzz from Symbolic Execution with Application to Smart Contracts." CCS 2019. Uses symbolic execution to guide fuzzer input generation. Relevant to the fuzzing-then-symbolic-execution pipeline ordering.
- Kim, Y., Lee, S., Yoo, H., Yun, J., & Kang, S. (2021). "An Off-The-Chain Execution Environment for Scalable Testing and Profiling of Smart Contracts." USENIX ATC 2021. Proposes off-chain execution environments for efficient contract testing. Conceptually related to the revm fork simulation used for pre-execution verification.
