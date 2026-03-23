# ⌈ ＢＡＲＤＯ ＤＥＳＩＧＮ ＳＹＳＴＥＭ ⌋
## Generative Protocol Views · Interface Spec · v1.0
### Auto-Generated UI for Discovered Protocols

**Cross-references:** `14-chain/07-generative-views.md` (data pipeline, trigger logic, ABI classification for protocol discovery), `00-sanctum-protocol-layer.md` (common protocol view chrome and Sanctum layer architecture), `01-protocol-view-catalog.md` (hand-authored views for top protocols that override generated ones), `../screens/02-widget-catalog.md` (33-widget TUI component library used for rendering generated views)

> **Reader orientation:** This document specifies the generative view system that auto-creates TUI layouts for DeFi protocols that lack hand-authored views. It belongs to the **interfaces/protocol layer**. Hand-authored views cover the top 5-10 protocols; this system fills in the long tail of hundreds of discovered protocols using a declarative schema mapped to the widget catalog. Key concepts: Golem (a mortal autonomous DeFi agent), the Sanctum layer (the DeFi protocol interaction system inside the terminal), and `bardo-chain-scope` (the protocol discovery service). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## Overview

Hand-authored protocol views (`01-protocol-view-catalog.md`) cover the top 5-10 protocols by usage. bardo-chain-scope can watch hundreds of discovered protocols. This spec covers the UI layer that renders auto-generated views for protocols without hand-authored layouts.

Generative views are a floor, not a ceiling. Anything auto-generated can be replaced by a hand-authored view. The system fills in the long tail.

The data pipeline (classification, ABI parsing, schema generation, LLM refinement) lives in `14-chain/07-generative-views.md`. This document covers what the user sees: the ProtocolViewSpec schema as it maps to widgets, the template library that determines layout, widget composition rules, and the dynamic layout engine.

---

## 1. ProtocolViewSpec (PVS) Widget Rendering

The PVS is a declarative schema stored in redb alongside the ProtocolDef. The TUI consumes it to render a protocol view. Each field in the PVS maps to a widget from the widget catalog.

### PVS-to-Widget Mapping

```rust
pub enum WidgetHint {
    /// Inline numeric value. Maps to FlashNumber (widget #1).
    /// Used for prices, balances, rates, counts.
    FlashNumbers,

    /// Braille-resolution inline chart. Maps to Sparkline (widget #8).
    /// Used for time-series of any numeric field with history.
    Sparkline,

    /// Double-height fill bar. Maps to MortalityGauge variant (widget #2).
    /// Used for utilization rates, health factors, ratios.
    Gauge,

    /// Static or slowly-updating text. No special widget — rendered as
    /// styled text with staleness decay.
    Text,

    /// Horizontal segmented bar. Maps to a simplified UnitArray (widget #6).
    /// Used for token distribution, reserve ratios, fee tiers.
    BarChart,
}
```

### Field Rendering Rules

Each `DisplayField` in the PVS renders according to its `WidgetHint` and `FieldFormat`:

| WidgetHint | Widget | Chrome | Interaction |
|---|---|---|---|
| `FlashNumbers` | FlashNumber (#1) | Label left, value right, flash on change | Enter → value history modal |
| `Sparkline` | Sparkline (#8) | Label left, chart right, current value inline | Enter → full waveform view |
| `Gauge` | MortalityGauge (#2) variant | Label above, bar below, percentage right | Enter → breakdown modal |
| `Text` | Styled text cell | Label left, value right | Enter → copy to clipboard |
| `BarChart` | UnitArray (#6) horizontal | Label above, segments below with per-segment labels | Enter → segment detail |

### Format Rules

| FieldFormat | Rendering |
|---|---|
| `Usd` | `$` prefix, comma-separated thousands, 2 decimal places. FlashNumber color: `success` (green) for value increases, `rose` for decreases |
| `Eth` | `Ξ` prefix, up to 6 decimal places. Compact mode: abbreviate with K/M suffixes |
| `Gwei` | Integer + ` gwei` suffix |
| `Percent` | Value × 100, `%` suffix, 2 decimal places. Gauge fill maps 0-100% to bar width |
| `Address` | First 6 + `…` + last 4 characters. Full on hover or Enter |
| `Raw` | Display as-is. Integer formatting with comma separation |

### Visual Grouping

Fields with matching `group` labels render inside a shared bordered pane. The group label becomes the pane header in `text_dim`. Fields within a group stack vertically. Empty groups (all fields null/zero) collapse to a single-line summary.

```
┌─ Token Reserves ──────────────────────┐
│ Reserve A (USDC)        42,100,000.00 │
│ Reserve B (WETH)              12,847  │
│ Total Liquidity       $156,200,000.00 │
└───────────────────────────────────────┘
```

---

## 2. Template Library

Four base templates control layout and field grouping. The template is selected by the protocol family classifier (see `14-chain/07-generative-views.md` §2). Each template specifies a layout hint, default field groupings, and which fields get sparklines vs. static display.

### DEX Template

**Applies to:** UniswapV2Pool, UniswapV3Pool, UniswapV4Pool, CurvePool, DEX_GENERIC

**Layout:** `TwoColumn`

```
┌────────────────────────────────────────────────────────────────┐
│ ⌈ USDC/WETH 0.3% ⌋                           [DEX · Generated]│
├──────────────────────────────┬─────────────────────────────────┤
│ Price                        │ Reserves                        │
│                              │                                 │
│   $3,412.87           +1.2%  │ USDC    42,100,000   ████████▓ │
│   ▁▂▃▅▇█▇▅▃▂▁▁▂▃▅▇          │ WETH        12,847   ████████  │
│   [60-tick price sparkline]  │                                 │
│                              │ Liquidity                       │
├──────────────────────────────┤ $156.2M              ████████▓ │
│ Fee Metrics                  │                                 │
│                              │ 24h Volume                      │
│ Fee Tier     0.30%           │ $8.4M                           │
│ Fee Growth   ▁▂▃▃▂▁▁▂       │                                 │
├──────────────────────────────┴─────────────────────────────────┤
│ ⌈ ACTIONS ⌋                                                    │
│ [Swap]                                                         │
│ Token In:  [________]  Amount: [________]  Min Out: [________] │
└────────────────────────────────────────────────────────────────┘
```

**V3/V4 additions:** Current tick, sqrtPriceX96, active liquidity concentration. Tick rendered as FlashNumber with raw format.

**Field groupings:**
- "Price": price field + price sparkline
- "Reserves": reserve0, reserve1, totalLiquidity
- "Fee Metrics": fee, feeGrowth (sparkline)
- "Volume": 24h volume if available

### Lending Template

**Applies to:** AaveV3Market, CompoundMarket, LENDING_GENERIC

**Layout:** `TwoColumn`

```
┌────────────────────────────────────────────────────────────────┐
│ ⌈ AAVE USDC Market ⌋                      [Lending · Generated]│
├──────────────────────────────┬─────────────────────────────────┤
│ Supply APY         3.42%     │ Borrow APY          5.18%       │
│ ▁▂▃▅▇▅▃▂▁▁▂▃▅               │ ▁▁▂▃▅▇█▇▅▃▂▁▁                  │
│                              │                                 │
│ Total Supplied               │ Total Borrowed                  │
│ $892.4M                      │ $341.2M                         │
├──────────────────────────────┴─────────────────────────────────┤
│ Utilization                                                    │
│ ████████████████████████████████▓░░░░░░░░░░░░░  38.2%          │
├────────────────────────────────────────────────────────────────┤
│ ⌈ ACTIONS ⌋  [Supply]  [Withdraw]  [Borrow]  [Repay]          │
└────────────────────────────────────────────────────────────────┘
```

**Field groupings:**
- "Supply": supply APY (sparkline), total supplied
- "Borrow": borrow APY (sparkline), total borrowed
- "Health": utilization rate (gauge), liquidation threshold

### Vault Template

**Applies to:** ERC4626Vault

**Layout:** `Stacked`

```
┌────────────────────────────────────────────────────────────────┐
│ ⌈ Yearn USDC Vault ⌋                       [Vault · Generated]│
├────────────────────────────────────────────────────────────────┤
│ Share Price          $1.047                                    │
│ TVL                  $24.8M                                    │
│ APY                  4.7%     ▁▂▃▅▇▅▃▂▁▁▂▃                    │
├────────────────────────────────────────────────────────────────┤
│ ⌈ ACTIONS ⌋  [Deposit]  [Withdraw]                             │
└────────────────────────────────────────────────────────────────┘
```

**Field groupings:**
- "Performance": share price, APY (sparkline)
- "Capacity": TVL, deposit cap (if available)

### Generic/Unknown Template

**Applies to:** UNKNOWN, any unmatched family

**Layout:** `Stacked`

All read functions → display fields, sorted alphabetically. All write functions → action forms. No layout intelligence. No sparklines (insufficient context to know which fields are time-series). Every field gets FlashNumbers or Text based on type.

```
┌────────────────────────────────────────────────────────────────┐
│ ⌈ 0x4e23…8f91 ⌋                          [Unknown · Generated]│
├────────────────────────────────────────────────────────────────┤
│ balances(0)          42,100,000                                │
│ balances(1)          41,800,000                                │
│ A                    2,000                                     │
│ fee                  4,000,000                                 │
│ owner                0xAb5…3d2                                 │
├────────────────────────────────────────────────────────────────┤
│ ⌈ ACTIONS ⌋  [exchange]  [add_liquidity]  [remove_liquidity]  │
└────────────────────────────────────────────────────────────────┘
```

---

## 3. Widget Composition Rules

### Precedence

1. Hand-authored view (`01-protocol-view-catalog.md`) takes absolute priority. If a hand-authored view exists for a protocol address, the generated view is never shown.
2. LLM-refined PVS (generation_method: `LlmRefined`) overrides template-only PVS.
3. Template PVS (generation_method: `Template`) is the fallback.

### Chrome

All generated views share the common protocol view chrome from `00-sanctum-protocol-layer.md` §3:
- Header bar with protocol name/address, family badge, generation method indicator
- F2 Golem Perspective overlay (from CorticalState) works on generated views
- Standard navigation: Tab cycles between data view and action forms, Esc returns to Sanctum
- The generation method badge (`[DEX · Generated]`, `[Lending · Refined]`) distinguishes auto-generated from hand-authored

### Action Form Rendering

Each `ActionForm` in the PVS renders as a form widget using the param-to-InputWidget mapping:

| ABI Type | Input Widget | Rendering |
|---|---|---|
| `uint256` | NumberInput | Decimal text input with wei conversion on submit. Label from `FormParam.label` |
| `address` | AddressInput | Text input with ENS resolution and checksum validation. Truncated display |
| `bool` | Toggle | Checkbox-style on/off |
| `bytes` / `bytes32` | BytesInput | Hex input with length validation |
| `int256` | NumberInput (signed) | Allows negative values |
| Tuple/struct | NestedForm | Recursively render struct fields, indented |

**Form submission:** Forms never submit directly. They construct a `TransactionRequest` that routes through the Golem's T1 approval and the permit system. The form is encoding infrastructure; the Golem decides whether the action is appropriate.

### Sparkline Eligibility

A field receives a sparkline (instead of static FlashNumber) when any of:
- The field name contains `rate`, `price`, `apy`, `yield`, `fee`, `growth`
- The LLM refinement pass flagged it as time-series
- The field has been observed to change across 3+ consecutive state fetches

Sparklines store the last 60 values (one per gamma tick that included a state fetch for this protocol).

---

## 4. Dynamic Layout Engine

### Layout Hints

Three layout modes, selected by template or LLM refinement:

| Layout | When | Behavior |
|---|---|---|
| `TwoColumn` | DEX, Lending | Left column: primary metrics (price, APY). Right column: secondary (reserves, utilization). Action forms below both columns. |
| `Grid` | Multi-asset protocols (future) | 2×N grid of field groups. Used when a protocol has many independent field groups. |
| `Stacked` | Vault, Unknown | Single column. Fields stack top to bottom. Simplest layout. |

### Responsive Behavior

```
Terminal width ≥ 120 cols:
  TwoColumn layout renders as designed (two 50% columns)

Terminal width 80-119 cols:
  TwoColumn collapses to Stacked with groups interleaved
  Sparklines shrink to 30 chars

Terminal width < 80 cols:
  All layouts → Stacked
  Sparklines hidden (FlashNumber only)
  Action form params stack vertically
```

### Field Visibility

When a protocol has many fields (>12 display fields), the layout engine applies priority filtering:

1. Fields in the template's default groupings are always shown
2. Fields flagged by LLM refinement as high-importance are shown
3. Remaining fields are collapsed into an expandable "More" section (`m` key)
4. Fields with null/zero values are hidden unless the user explicitly expands

### Live Reload

When `GolemEvent::ProtocolViewGenerated` or `GolemEvent::ProtocolViewRefined` arrives while the view is open:

1. Current scroll position and form state are preserved
2. New PVS is loaded and diffed against the current render
3. Added fields fade in from `text_phantom` over 300ms
4. Removed fields fade out over 500ms
5. Changed labels update with a brief `bone` flash
6. Layout reflows if the layout hint changed

The user sees the view improve in real time as the LLM refinement completes. No navigation required.

---

## 5. Golem Perspective (F2) on Generated Views

F2 works on generated views the same as hand-authored ones. The CorticalState processes the protocol's state data and generates floating annotations. For generated views, the annotations may be less specific (the Golem has less domain knowledge about unknown protocols), but the mechanism is identical.

Additional generated-view-specific annotations:

| Situation | Annotation |
|---|---|
| Low-confidence classification | "I'm not sure what this protocol does. The ABI looks like a DEX but the state fields don't match typical patterns." |
| First interaction | "I've never interacted with this protocol before. Proceeding with baseline caution." |
| State anomaly | "This value changed by 3x since last observation. That's unusual for this field type." |
| Similar to known protocol | "This looks like a Curve fork based on the ABI structure. Treating it similarly." |

---

## 6. Staking and Bridge Templates (Future)

Two additional templates are planned but not yet specified:

**Staking Template:** For staking protocols (validator deposits, delegation, rewards). Primary fields: staked amount, reward rate, unbonding period, validator count. Layout: `TwoColumn`.

**Bridge Template:** For cross-chain bridges. Primary fields: locked amount per chain, bridge fee, transfer time, supported chains. Layout: `TwoColumn` with a chain-selector widget.

These templates will be added when the protocol family classifier gains the selectors needed to detect them reliably.

---

*⌈ every protocol gets a face. hand-authored for the known. generated for the rest. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
