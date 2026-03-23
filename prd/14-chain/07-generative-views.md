# Generative Protocol Views [SPEC]

> **Version**: 1.0 | **Status**: Active
>
> **Crates**: `bardo-triage` (trigger), `bardo-protocol-state` (schema store), `bardo-stream-api` (exposure)

---

> **Reader orientation:** This document specifies generative protocol views, an extension of Bardo's chain intelligence layer (section 14) that auto-generates TUI views for any discovered DeFi protocol. When a Golem (a mortal autonomous agent compiled as a single Rust binary running on a micro VM) discovers a new protocol, the system immediately generates a usable view from the contract ABI, then optionally refines it with a single LLM call. The key concept is the long tail problem: ChainScope can watch hundreds of protocols, but hand-authored views only cover the top 5-10. See `prd2/shared/glossary.md` for full term definitions.

Protocol views in Sanctum are currently 100% hand-authored per protocol. That works for the top 5-10 protocols (Uniswap, Aave, Curve) but doesn't scale: `bardo-chain-scope` can watch hundreds of discovered protocols, and manually specifying a TUI layout for each is impossible.

The gap: bardo-triage discovers a new protocol, the golem knows about it, but the user sees nothing. Generative views close that gap automatically.

Generative views are a floor, not a ceiling. Anything auto-generated can be replaced by a hand-authored view. The system fills in the long tail. Top 5-10% of protocols by usage should still have hand-authored views; the rest get generated ones that are functional if not optimal.

---

## Three Inputs

1. **Contract ABI** -- function/event selectors, return types, param names
2. **Protocol family classification** -- DEX, lending, vault, unknown
3. **State field samples** -- live values from initial state fetch

---

## Two Output Phases

- **Immediate (~1ms)**: Template-driven ProtocolViewSchema (PVS), stored to redb, user sees something
- **Async (~1-2s)**: LLM refines labels and groupings once, result cached permanently

---

## Protocol Family Classifier

Before any UI can be generated, the protocol needs a family label. Classification rules are checked in order, short-circuit on first match:

| Family | Selector signals | Factory provenance |
|--------|------------------|--------------------|
| `UniswapV2Pool` | `swap(uint,uint,address,bytes)`, `Swap` event | UniswapV2Factory |
| `UniswapV3Pool` | `swap(address,bool,int256,uint160,bytes)`, `Swap` event with sqrtPrice | UniswapV3Factory |
| `UniswapV4Pool` | PoolManager hookdata patterns | v4 PoolManager |
| `AaveV3Market` | `supply()`, `borrow()`, `liquidationCall()` | AavePoolAddressesProvider |
| `CompoundMarket` | `mint()`, `redeem()`, `borrow()`, `liquidate()` | Comptroller |
| `CurvePool` | `exchange()`, `add_liquidity()`, `get_dy()` | CurveFactory |
| `ERC4626Vault` | `deposit()`, `withdraw()`, `convertToAssets()` | ERC-165 `supportsInterface` |
| `DEX_GENERIC` | Any swap-like selector + price view function | Unknown factory |
| `LENDING_GENERIC` | deposit/withdraw/borrow/repay pattern | Unknown factory |
| `UNKNOWN` | No match | -- |

Priority: bytecode hash match (highest confidence), then factory provenance (medium), then ABI selector fingerprint (lower, but good for forks and variants).

```rust
pub fn classify_protocol(
    abi: &[AbiItem],
    bytecode_hash: Option<B256>,
    provenance: &Provenance,
) -> ProtocolFamily {
    // 1. Exact bytecode hash match against registry.
    if let Some(hash) = bytecode_hash {
        if let Some(family) = PROTOCOL_REGISTRY.lookup_bytecode(hash) {
            return family;
        }
    }

    // 2. Factory provenance.
    if let Some(family) = provenance.factory_family() {
        return family;
    }

    // 3. ABI selector fingerprint.
    let selectors: HashSet<B256> = abi.iter()
        .map(|item| item.selector())
        .collect();
    classify_by_selectors(&selectors)
}
```

Result stored in `ProtocolDef.family_classification`.

---

## ProtocolViewSchema (PVS)

A declarative, serializable description of a generated view. Stored in redb alongside the ProtocolDef. This is what the template engine outputs and what the TUI consumes.

```rust
pub struct ProtocolViewSchema {
    pub protocol_id: String,
    pub family: ProtocolFamily,
    pub title: String,                        // e.g., "USDC/WETH 0.3%"
    pub display_fields: Vec<DisplayField>,
    pub action_forms: Vec<ActionForm>,
    pub layout: LayoutHint,                   // TwoColumn | Grid | Stacked
    pub generated_at: u64,                    // unix_ms
    pub generation_method: GenerationMethod,  // Template | LlmRefined
    pub llm_schema_override: Option<serde_json::Value>,
}

pub struct DisplayField {
    pub id: String,             // maps to ProtocolState field path
    pub label: String,
    pub widget: WidgetHint,     // FlashNumbers | Sparkline | Gauge | Text | BarChart
    pub group: Option<String>,  // visual grouping label
    pub format: FieldFormat,    // Usd | Eth | Gwei | Percent | Address | Raw
}

pub struct ActionForm {
    pub fn_name: String,            // ABI function name
    pub label: String,              // human label, e.g., "Swap"
    pub params: Vec<FormParam>,
    pub gas_estimate: Option<u64>,
}

pub struct FormParam {
    pub name: String,
    pub abi_type: AbiType,          // uint256, address, bool, bytes, etc.
    pub widget: InputWidget,        // NumberInput | AddressInput | Toggle | BytesInput
    pub label: String,
    pub placeholder: Option<String>,
}

pub enum GenerationMethod {
    Template,
    LlmRefined,
}
```

Schema persistence: stored in redb at `protocol_view_schema/{protocol_id}`. Invalidated when ABI changes (new bytecode hash) or LLM refinement completes. Loaded at TUI startup.

---

## ABI-to-Schema Generation Pipeline

Runs when a new protocol is discovered by bardo-triage.

### Phase 1 -- Immediate (Synchronous, ~1ms)

```
ProtocolDiscovered event
    -> classify_protocol() -> ProtocolFamily
    -> abi_to_display_fields() -> Vec<DisplayField>
    -> abi_to_action_forms() -> Vec<ActionForm>
    -> apply_family_template() -> LayoutHint + field groupings
    -> emit PVS with generation_method: Template
    -> store to redb
    -> GolemEvent::ProtocolViewGenerated { protocol_id, method: "template" }
```

### Phase 2 -- Async LLM Refinement (Background, ~1-2s)

```
PVS generated
    -> LLM prompt: ABI summary + current PVS + state field samples
    -> LLM returns: improved labels, better groupings, suggested sparkline fields
    -> merge LLM delta into PVS.llm_schema_override
    -> store updated PVS
    -> GolemEvent::ProtocolViewRefined { protocol_id }
```

### Type Mapping Rules

| ABI return type | Inferred semantics | Widget | Format |
|-----------------|--------------------|--------|--------|
| `uint256` named `price`, `sqrtPriceX96`, `rate` | Price | FlashNumbers | depends on token |
| `uint256` named `totalLiquidity`, `totalSupply`, `tvl` | Liquidity/TVL | FlashNumbers | USD |
| `uint256` named `reserve0`, `reserve1`, `balance` | Token balance | FlashNumbers | token units |
| `uint256` named `fee`, `feeGrowth` | Fee metric | FlashNumbers + Sparkline | bps |
| `int24` named `tick`, `currentTick` | Tick | FlashNumbers | raw |
| `uint128` named `liquidity` | Liquidity density | Gauge (when range known) | raw |
| `address` | Address | Text (truncated) | Address |
| `bool` | Binary flag | Text | checkmark/x |
| `uint256[]`, `int256[]` | Array | BarChart | token units |
| Other `uint256` | Generic number | FlashNumbers | Raw |

Matching is done on field name (case-insensitive substring match), then falls back to type-only inference.

### Write Function Selection

Include functions with names matching: `swap`, `deposit`, `withdraw`, `borrow`, `repay`, `mint`, `redeem`, `provide`, `remove`, `collect`.

Exclude: `owner`, `admin`, `emergency`, `pause`, and functions with raw `bytes calldata` params that can't be meaningfully prompted (unless the family template knows the encoding).

---

## Family Templates

Four base templates. Applied after ABI analysis to set layout and field groupings.

### DEX Template (UniswapV2Pool, UniswapV3Pool, DEX_GENERIC)

- Primary pane: price (FlashNumbers, large), price chart (Sparkline last 60 ticks)
- Secondary pane: reserves/liquidity, fee metrics, 24h volume (if available)
- Action forms: Swap (tab 1)
- V3-specific additions: tick range, sqrt_price, active liquidity

### Lending Template (AaveV3Market, CompoundMarket, LENDING_GENERIC)

- Primary pane: supply APY, borrow APY (FlashNumbers side by side)
- Secondary pane: utilization rate (Gauge), total supplied, total borrowed
- Action forms: Supply, Withdraw, Borrow, Repay (tabs)

### Vault Template (ERC4626Vault)

- Primary pane: share price (`convertToAssets` for 1 share), TVL
- Secondary pane: strategy info if available via ABI, yield/APY
- Action forms: Deposit, Withdraw

### Generic/Unknown

- Alphabetical field list, all returned values shown
- Read functions become display fields; write functions become forms
- No layout hints beyond Stacked

---

## Execution Forms

This is where generated views get practical value: a user can interact with a protocol they've never seen without any hand-authored integration.

### Calldata Assembly

```rust
pub async fn assemble_calldata(
    form: &ActionForm,
    values: &FormValues,
) -> Result<Bytes> {
    let function = alloy_json_abi::Function::parse(&form.fn_signature)?;
    let tokens: Vec<DynSolValue> = form.params.iter()
        .zip(values.iter())
        .map(|(param, val)| encode_param(&param.abi_type, val))
        .collect::<Result<Vec<_>>>()?;
    Ok(function.abi_encode_input(&tokens)?.into())
}
```

### FormWidget

```rust
pub struct FormWidget {
    pub form: ActionForm,
    pub state: FormState,            // current input values
    pub validation: Vec<ValidationError>,
}

impl Widget for FormWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, state: &InterpolatedState) {
        // Render label + input rows
        // Highlight active param
        // Show gas estimate
        // Show validation errors inline
    }
}
```

### Param Type to InputWidget Mapping

| ABI type | InputWidget | Notes |
|----------|-------------|-------|
| `uint256` | NumberInput | Decimal input, converts to wei on submit |
| `address` | AddressInput | ENS resolution if available, validates checksum |
| `bool` | Toggle | On/off |
| `bytes` | BytesInput | Hex input, length validated |
| `bytes32` | BytesInput (fixed 32) | |
| `int256` | NumberInput (signed) | Negative values allowed |
| `uint8` enum | RadioGroup | If named variants found in NatSpec |
| Tuple/struct | NestedForm | Recursively render struct fields |

### Submission Pipeline

The form never directly submits a transaction. It constructs a `TransactionRequest` that routes through:

1. **Golem T1 approval** -- does this action make sense given current strategy?
2. **Permit system** -- does the operator have permission for this action type?
3. **Standard signing + submission pipeline**

This is intentional. The form is encoding infrastructure; the golem decides whether the action is appropriate. Execution routes through the existing golem tool system (the execute-transaction tool), not directly from the form.

---

## LLM Refinement Pass

The template pass is fast but dumb. LLM refinement runs once per protocol and improves:

- Field labels (ABI names are often terse: `getReserves` becomes "Token Reserves")
- Groupings (which fields belong together visually)
- Which fields warrant a sparkline vs. static display
- Action form labels and param descriptions
- Protocol title (heuristic classification alone may not know "Curve 3pool" from address)

### Prompt Structure

```
System: You are analyzing a DeFi protocol's ABI to generate a terminal UI schema.
        The terminal uses composable widgets: FlashNumbers, Sparkline, Gauge, Text, BarChart.
        Output a JSON delta to improve the existing schema.

User: Protocol: 0x4e23... (classified as CurvePool via factory provenance)
      ABI summary: [exchange(int128 i, int128 j, uint256 dx, uint256 min_dy),
                    get_dy(int128 i, int128 j, uint256 dx) returns (uint256),
                    balances(uint256 i) returns (uint256), A() returns (uint256), ...]
      Current schema: [generated PVS JSON]
      Sample state: { "balances": [42100000000000, 41800000000000, 43200000000000], "A": 2000 }

      Return JSON: { "title": "...", "field_overrides": [...], "group_overrides": [...] }
```

**Cost**: One T1 call per newly discovered protocol (~$0.001). One-time cost, result cached indefinitely until ABI changes.

**Priority**: Processed in `theta_chain_cognition()` (see [05-heartbeat-integration.md](./05-heartbeat-integration.md)), alongside high-score triage events. Max 5 view refinements per Theta tick (low priority vs. active chain events).

---

## Integration Points

### Trigger Location

In `bardo-triage`, in the protocol fingerprinter stage. After a new `ProtocolDef` is registered:

```rust
// In ProtocolRegistry::register_new()
if self.view_schema_store.get(&def.id).is_none() {
    let schema = generate_template_schema(&def).await?;
    self.view_schema_store.put(&def.id, &schema).await?;
    self.llm_refinement_queue.push(def.id.clone());
    fabric.emit(GolemEvent::ProtocolViewGenerated {
        protocol_id: def.id,
        method: "template",
    });
}
```

### Stream API Exposure

`bardo-stream-api` exposes `GET /chain/v1/{chain_id}/protocol/{address}/schema` returning PVS JSON. See [08-stream-api.md](./08-stream-api.md).

### TUI Consumption

- `GolemEvent::ProtocolViewGenerated` and `::ProtocolViewRefined` trigger live reload if the view is currently open
- Sanctum view renderer checks for hand-authored view first, falls back to PVS if none found
- PVS is loaded at TUI startup for all watched protocols

See [18-interfaces/03-tui.md](../18-interfaces/03-tui.md) for the TUI spec and widget catalog.

### Scale Gating

If bardo-chain-scope watches 10,000 addresses at peak arousal, don't generate a PVS for all 10,000. Gate: only generate for protocols with `curiosity_score > 0.6` and at least 1 confirmed `ProtocolInteraction` event.

---

## GolemEvent Additions

Two new variants (extends the base 5 from [06-events-signals.md](./06-events-signals.md)):

```rust
#[serde(rename = "chain.protocol_view_generated")]
ProtocolViewGenerated {
    protocol_id: String,
    chain_id: u64,
    method: String,  // "template" | "llm_refined"
},

#[serde(rename = "chain.protocol_view_refined")]
ProtocolViewRefined {
    protocol_id: String,
    chain_id: u64,
},
```

---

## Generative TUI Beyond DeFi

The ABI-to-view pipeline is an instance of a more general pattern: any system with self-describing data (schema, OpenAPI spec, GraphQL schema, database introspection) supports the same approach.

The general pattern:
```
Self-describing source -> schema extraction -> field type mapping -> layout template -> rendered view
                                                                         ^
                                                               LLM label improvement (optional)
```

Applications:

*JSON/structured data streams*: A webhook payload, a Prometheus metric endpoint, an arbitrary JSON API. Given the schema (or inferred from samples), generate a live data pane. Useful for monitoring dashboards without configuration.

*OpenAPI/REST APIs*: From an OpenAPI spec, auto-generate a TUI REST client. GET endpoints become data views, POST/PUT become forms. The ActionForm concept generalizes here directly.

*Database query results*: `SELECT *` result with auto-detected column types producing appropriate TUI columns (numbers right-align + color, timestamps format, booleans toggle, JSON expand).

*Config file editors*: JSON Schema to auto-generated form editor. Already common in web tooling (react-jsonschema-form), rare in terminal.

*Log stream intelligence*: Structured logs (JSON logs) with auto-detected fields producing live filter/search panes. Similar to how Loki + Grafana work, but terminal-native.

The key insight from the DeFi case: Protocol ABIs are schemas with semantics. The ABI is a schema, function names carry semantic intent, return types are typed fields. Any schema-with-semantics (OpenAPI with descriptions, GraphQL with directives, JSON Schema with titles/descriptions) supports the same approach. Quality of generation is proportional to how much semantic information the schema carries.

Where generated views fall short:
- Custom visualizations (Uniswap tick range density chart) require domain knowledge that can't be inferred from types alone
- Optimal information density requires domain judgment
- Execution safety requires understanding what a function actually does, not just its signature
- LLM refinement helps with labeling but not with fundamental layout insight

Generated views are a sensible floor for the long tail. The top 5-10% of protocols by usage should still have hand-authored views.

---

## Open Questions

1. **Reorg handling for submitted forms**: If a user submits a swap via a generated form and the block reorgs, how does the UX respond? Should align with existing transaction tracking in bardo rather than inventing something new.

2. **ABI drift**: If the protocol upgrades its implementation (proxy pattern), the cached ABI and PVS become stale. Detection: bytecode hash change on monitored contract triggers re-generation automatically.

3. **Malicious ABIs**: A contract could have function names designed to look like safe operations. Mitigation: the golem T1 approval is the safety check, not the form generator. The form is encoding infrastructure; the golem decides whether the action is appropriate.

4. **ERC-2535 Diamond proxies**: Multiple facets with separate ABIs. The classifier and ABI resolver need special handling -- the effective ABI is the union of all facets, but provenance tracking gets more complex.

---

## Prior Art

- **ABI.ninja** (Fraction.so, 2022): Web UI auto-generated from contract ABI. Read + write, no layout intelligence.
- **Scaffold-ETH** (BuidlGuidl, 2023): Next.js UI auto-generation from Hardhat/Foundry ABIs. Closest existing analog -- same idea, web context.
- **react-jsonschema-form** (Mozilla, 2014-): JSON Schema to React form. The web equivalent of the form generation here.
- **WhatsABI** (Shazow, 2023): ABI recovery from bytecode -- feeds the classifier when source isn't verified on Etherscan.
- **json-forms** (Eclipse, 2016-): JSON Schema + UI Schema to forms, multi-framework.
- **OpenAPI Generator**: Template-driven code generation from OpenAPI specs -- the same "template per family" pattern applied to code rather than UI.
- **htmx**: Philosophically adjacent -- the server describes what to render, client applies it. Generated views are the TUI equivalent.

---

## Verification

1. Instantiate a `ClassifyProtocol` test with a minimal UniswapV2 ABI -- confirm it returns `UniswapV2Pool` (bytecode/factory match) or `DEX_GENERIC` (selector-only fallback).
2. Generate a PVS from the V2 ABI -- confirm it has a `swap` action form with 4 params mapped to the correct InputWidgets.
3. Render the generated PVS in the TUI using existing widgets -- confirm it displays without crashing.
4. Submit the swap form against a local Anvil fork -- confirm calldata assembles correctly and routes through the golem approval pipeline.
5. Verify LLM refinement improves labels on a real Curve pool ABI (e.g., `balances` becomes "Pool Balances", `A` becomes "Amplification Coefficient").

---

## Cross-References

- **Protocol state**: [03-protocol-state.md](./03-protocol-state.md) -- ProtocolDef definition and family classification that drives template selection
- **Stream API**: [08-stream-api.md](./08-stream-api.md) -- Exposes the PVS schema via `GET /chain/v1/{chain_id}/protocol/{address}/schema`
- **Triage**: [02-triage.md](./02-triage.md) -- Protocol fingerprinter in the triage pipeline triggers view generation on new protocol discovery
- **Heartbeat**: [05-heartbeat-integration.md](./05-heartbeat-integration.md) -- Theta tick processes LLM refinement of generated views alongside high-score triage events
- **TUI**: [18-interfaces/03-tui.md](../18-interfaces/03-tui.md) -- Widget catalog (FlashNumbers, Sparkline, Gauge, etc.) consumed by the ProtocolViewSchema

---

## References

- Fan, B. et al. (2014). Cuckoo Filter: Practically Better Than Bloom. *CoNEXT*. -- *Alternative probabilistic filter; referenced in the context of the broader filter comparison.*
- Shazow (2023). WhatsABI: Guess an ABI from an Ethereum contract address. GitHub. -- *ABI recovery tool from bytecode; feeds the classifier when contract source is not verified on Etherscan.*
