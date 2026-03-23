# ⌈ ＳＡＮＣＴＵＭ ⌋ — The Protocol Interaction Layer
## Terminal Interface · v4.04

**Source:** `engagement-prd/bardo-v4-04-sanctum-protocol-layer.md`

**Cross-references:** `../perspective/01-golem-perspective.md` (F2 Perspective overlay that adds the Golem's inner thoughts to any data view), `01-protocol-view-catalog.md` (per-protocol view specs for Uniswap, Aave, Morpho, Aerodrome, etc.), `../screens/02-widget-catalog.md` (33-widget TUI component library; FlashNumber §1, health factor gauge §2)

> **Reader orientation:** This document specifies the Sanctum layer, the system that lets owners interact with DeFi protocols (swapping tokens, managing LP positions, supplying collateral) directly inside the Bardo terminal. It belongs to the **interfaces/protocol layer**. The Sanctum layer renders protocol UIs with keyboard navigation, the ROSEDUST aesthetic, and the ability to overlay the Golem's cognitive perspective on any protocol data. Key concepts: Golem (a mortal autonomous DeFi agent), `@bardo/sanctum` (the tool library exposing 166+ DeFi protocol adapters), and Styx (Bardo's social and connectivity layer). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. What This Is

The Sanctum layer is how the terminal user interacts with DeFi protocols directly — swapping tokens, managing LP positions, supplying collateral, browsing pools, viewing token data — all without leaving the Bardo terminal. Every protocol view renders through the lens of the selected Golem's worldview, using the Golem's authenticated endpoints, querying through the Golem's tools. When no Golem is selected, the terminal's own wallet and RPC connections serve as the data source.

This is the equivalent of visiting uniswap.org, app.aave.com, or morpho.org — but inside the terminal, with keyboard navigation, the ROSEDUST aesthetic, real-time data via WebSocket, and the ability to invoke the Golem's cognitive perspective on anything you're looking at.

The name comes from `@bardo/sanctum`, the tool library that exposes 166+ DeFi protocol adapters. The Sanctum layer is the visual surface of those tools. It renders through the lens of the selected Golem's (a mortal autonomous DeFi agent) worldview.

---

## 1. Placement in the Hierarchy

### Two Entry Points

**Entry Point A: Contextual (from existing positions)**

Inside the **Soma** window (Tab 1: Portfolio), the owner sees position cards — LP ranges, lending supplies, token balances. Each card shows which protocol it belongs to (Uniswap, Aave, Morpho, Aerodrome, etc.). When you lock a position card and press Enter, the **Position Detail Modal** opens. Within that modal, a new action is available:

```
Tab 5: Protocol → Opens the full Sanctum view for this protocol,
                   pre-navigated to the relevant pool/market/position.
```

This is the "I'm looking at my Uniswap LP and want to manage it in the full protocol interface" flow.

Similarly, in **Soma > Tab 2: Trades**, entering a trade's detail modal and selecting the Protocol tab opens the Sanctum view for the protocol that trade executed on, at the relevant pair/pool.

**Entry Point B: General (Protocol Browser)**

A new tab is added to the **Soma** window:

> Sanctum is SOMA tab 5. See `../screens/00-screen-catalog.md` for the canonical tab listing.

The Sanctum tab contains the **Protocol Browser** — a navigable list of all supported protocols, organized by category, filterable by chain. This is the "I want to explore Morpho markets on Base" flow, independent of existing positions.

### Without a Golem Selected

When no Golem is selected (the "all" view or no Golems exist), the Sanctum layer still works:

- Data flows through the terminal's own RPC connections (configured in `~/.bardo/config.toml`)
- The chain selector uses the terminal's configured chains
- Protocol views show real-time data as normal
- You CAN execute transactions from the terminal wallet (if configured)
- You CANNOT invoke the Golem Perspective overlay (F2)
- You CANNOT send directives to a Golem
- The Steer integration and paper trading modes are unavailable

This means a user who hasn't created any Golems yet can still browse protocols, view pool data, check token prices, and even make trades from their terminal wallet.

### Navigation Model

The Sanctum view opens as a **full-screen modal** (Layer 6+), following the existing modal hierarchy from the interaction hierarchy PRD. It has its own internal tab/pane structure. Backspace exits back one level (to whatever triggered the current view). Esc jumps to Layer 3 (pane focus), consistent with the global Esc behavior defined in the interaction hierarchy.

```
SOMA > Sanctum Tab > Protocol Browser (list of protocols)
                        │
                        Enter on protocol
                        ↓
                      Protocol View (full-screen modal)
                        │
                        Has its own tabs: varies by protocol
                        Each tab has panes
                        Each pane can lock, select, drill into modals
                        │
                        Backspace → back to Protocol Browser
                        Esc → jump to Layer 3 (pane focus)
```

---

## 2. The Protocol Browser

### Layout

**Protocol List** (left, 40%) — All supported protocols, grouped by category.

```
⌈ DEX ⌋
  ◆ Uniswap        Base, Ethereum, Arbitrum, Celo
  ◆ Aerodrome       Base

⌈ LENDING ⌋
  ◆ Aave            Base, Ethereum, Arbitrum
  ◆ Morpho          Base, Ethereum
  ◆ Compound        Ethereum, Base
  ◆ Spark           Ethereum

⌈ YIELD ⌋
  ◆ Pendle          Ethereum, Arbitrum
  ◆ Beefy           Base, Arbitrum
  ◆ Yearn           Ethereum

⌈ STAKING ⌋
  ◆ Lido            Ethereum
  ◆ EigenLayer      Ethereum
  ◆ Rocket Pool     Ethereum

⌈ BRIDGE ⌋
  ◆ Across          Multi-chain
  ◆ Stargate        Multi-chain
  ◆ Hop             Multi-chain

⌈ STABLECOIN ⌋
  ◆ MakerDAO / Sky  Ethereum
  ◆ Curve           Ethereum, Base, Arbitrum

⌈ IDENTITY ⌋
  ◆ ENS             Ethereum

⌈ DERIVATIVES ⌋
  ◆ GMX             Arbitrum
```

Each protocol entry shows:
- Name and icon glyph
- Supported chains (dimmed if the current Golem's selected chain doesn't match)
- Active positions indicator: a count badge if the selected Golem has open positions on this protocol (e.g., `◆ Uniswap [3]`)
- Status dot: green if the Golem has tools loaded for this protocol, dim if the protocol is available but no tools are currently active

**Protocol Detail** (right, 60%) — Preview of the selected protocol.
- Protocol name, description, category
- Chain availability
- Current Golem's positions on this protocol (summary: count, total value)
- Recent transactions on this protocol
- Key stats: TVL, 24h volume, current APY ranges (fetched via the Golem's data tools or terminal RPC)

### Filtering and Sorting

- `f` → Filter by: chain, category, "has positions", "has tools loaded"
- `s` → Sort by: name, TVL, position count, recent activity
- `/` → Search: fuzzy search across protocol names

### Chain Selector

The chain selector is persistent in the Sanctum tab's top bar:

```
⌈ CHAIN ⌋  ◉ Base  ○ Ethereum  ○ Arbitrum  ○ Celo  ○ Testnet
```

- When a Golem is selected, the chain defaults to the Golem's configured primary chain
- `Alt+C` cycles chains (distinct from `Ctrl+↑/↓` which cycles Golems)
- Protocols not deployed on the selected chain are greyed out but still visible
- Changing chains refreshes all data in the Protocol Browser and any open Protocol View

---

## 3. Protocol View Architecture

When you Enter on a protocol in the browser, the **Protocol View** modal opens. Each protocol has a unique set of tabs and panes, but they all share a common chrome:

### Common Protocol View Chrome

**Header Bar:**
```
⌈ ＵＮＩＳＷＡＰ ⌋  V3  Base  ◉ Connected  │ Wallet: 0x1a2b...3c4d │ Balance: 4.21 ETH
```

- Protocol name in fullwidth
- Protocol version (if applicable)
- Selected chain
- Connection status
- Active wallet address (Golem's or terminal's)
- Relevant balance

**Bottom Bar:**
```
1-5:tab  ←→↑↓:navigate  Enter:select  F2:perspective  F3:paper  F5:refresh  Backspace:back
```

The bottom bar includes the standard navigation keys plus:
- `F2` — Toggle Golem Perspective overlay (see `../perspective/01-golem-perspective.md`)
- `F3` — Toggle Paper Trading mode (see `../perspective/01-golem-perspective.md` §9)
- `F5` — Force refresh all data
- `F8` — Execute / Submit (context-dependent)

### Protocol View Modal Anatomy

```
┌─ ⌈ ＵＮＩＳＷＡＰ ⌋  V3  Base  ◉ Connected  │ Wallet: 0x1a2b...3c4d ──────────────────┐
│                                                                                     │
│  [Tab 1: Swap] [Tab 2: Pools] [Tab 3: Positions] [Tab 4: Tokens]                   │
│                                                                                     │
│  ┌─────────────────────────────────────┐  ┌──────────────────────────────────────┐ │
│  │                                     │  │                                      │ │
│  │   Primary Pane (60%)                │  │   Secondary Pane (40%)               │ │
│  │                                     │  │                                      │ │
│  │   Main interaction area             │  │   Chart / details / context          │ │
│  │   (form, list, etc.)                │  │                                      │ │
│  │                                     │  │                                      │ │
│  └─────────────────────────────────────┘  └──────────────────────────────────────┘ │
│                                                                                     │
│  ┌──────────────────────────────────────────────────────────────────────────────┐  │
│  │  Bottom strip: recent trades, status, supplementary data                     │  │
│  └──────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                     │
│  1-4:tab  ←→↑↓:navigate  Enter:select  F2:perspective  F3:paper  F5:refresh  Esc  │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Golem Context Strip

When a Golem is selected, a persistent strip shows the Golem's current exposure relevant to this protocol:

```
⌈ GOLEM CONTEXT ⌋  Position: 0.5 ETH LP @ 1800-2200  │  Exposure: $847  │  PnL: +$23.4 (2.8%)
```

This strip appears below the tab bar. It updates in real-time via the Golem's observation pipeline. When no position exists on this protocol, it shows: `⌈ GOLEM ⌋  No position on this protocol`.

### Data Flow

All data in Protocol Views flows through one of two paths:

**Path A: Golem Selected**
```
Protocol View → Golem's tool endpoints (authenticated) → Chain RPC
                     │
                     WebSocket subscription for real-time updates
                     │
                     Golem's Grimoire and context available for F2 overlay
```

The Golem's tools (`query_state`, `preview_action`, `commit_action`) are the data layer. The terminal subscribes to the Golem's Event Fabric for real-time updates. Token prices, pool states, and position data arrive via the Golem's observation pipeline.

**Path B: No Golem Selected**
```
Protocol View → Terminal's direct RPC connections → Chain RPC
                     │
                     Polling at configurable interval (default 5s)
                     │
                     F2 overlay unavailable
```

The terminal uses its own `@bardo/chain` RPC client configured in `config.toml`. Data is polled rather than streamed (unless the terminal has its own WebSocket subscriptions configured).

### Real-Time Updates

Protocol Views should update in real-time where possible:

- **Token prices**: WebSocket subscription via the Golem's observation pipeline (or terminal polling)
- **Pool state**: Subscribe to relevant on-chain events (Swap, Mint, Burn, etc.)
- **Position health**: Continuous monitoring via the Golem's probe system
- **Order book / trade flow**: Stream from the Golem's market data tools

When a value changes, it uses the **FlashNumber** widget (Widget Catalog §1) — green flash on increase, rose flash on decrease, phosphor persistence on the old value.

### Keyboard Navigation Within Protocol Views

Protocol Views follow the standard Bardo navigation model:

- Number keys (`1-9`): switch tabs within the protocol view
- Arrow keys: navigate between panes
- Enter: lock pane → select element → open detail modal
- Backspace: exit protocol view (back one level)
- Esc: jump to Layer 3 (pane focus)

### Form Input

Protocol Views contain form elements — amount inputs, token selectors, slippage settings. These follow a consistent pattern:

- **Amount Input**: When an input field is focused, typing numbers fills the value. `Tab` moves to the next field. Percentage buttons (`25%`, `50%`, `75%`, `Max`) are navigable with arrow keys.
- **Token Selector**: Enter on a token field opens a **Token Search Modal** with fuzzy search, showing token name, symbol, balance, and price.
- **Slippage / Settings**: Displayed in a collapsible pane within the protocol view (no dedicated hotkey). Persistent per-protocol.
- **Confirmation**: Before any write action, a **Confirmation Modal** appears showing the full transaction details, estimated gas, estimated cost, and the action buttons (Cancel / Confirm / Send to Golem).

---

## 4. Execution Modes

When a user fills out a form (e.g., a swap on Uniswap) and presses the execute key (`F8`), three execution paths are available:

### Mode 1: Direct Execute (Owner Bypass)

```
F8 → Confirmation Modal → Confirm
```

The transaction executes immediately through the Golem's wallet (or terminal wallet if no Golem). This bypasses the Golem's cognitive pipeline entirely — no MAGI vote, no T2 deliberation, no emotional appraisal. The Golem records the transaction as an externally-directed action but does not reason about it beforehand.

Use case: The owner knows exactly what they want and doesn't need the Golem's opinion.

The Golem still observes the outcome and records it as an episode. It learns from the result even though it didn't deliberate.

### Mode 2: Golem Deliberation

```
F8 → Confirmation Modal → "Let Golem Think" (Shift+Enter)
```

The full transaction parameters are packaged as a Steer directive and sent to the Golem's heartbeat pipeline. The Golem runs a full T2 deliberation cycle:

1. The transaction parameters enter the Cognitive Workspace
2. Relevant Grimoire entries, PLAYBOOK heuristics, and somatic markers are retrieved
3. The MAGI council votes
4. The Golem decides: execute as proposed, modify parameters, or decline

The result appears in the Protocol View as a response overlay:

```
┌─ GOLEM DELIBERATION ──────────────────────────┐
│                                                │
│  MAGI: ██APPROVE██  APPROVE  MODIFY            │
│                                                │
│  The Golem suggests modifying slippage from    │
│  0.5% to 0.3% based on current pool depth.    │
│  Grimoire entry #412 (gas-timing heuristic)   │
│  indicates lower gas in ~4 minutes.            │
│                                                │
│  [Accept] [Accept Modified] [Override] [Cancel]│
└────────────────────────────────────────────────┘
```

Use case: The owner wants the Golem's perspective before executing. This helps the Golem learn because it engages the full cognitive pipeline.

### Mode 3: Free-Form Delegation

```
F8 → Confirmation Modal → "Let Golem Decide" (Ctrl+Enter)
```

Instead of sending specific parameters, the current screen state (protocol, pool, balances, market conditions) is packaged as context, and the Golem receives an open-ended directive: "Optimize this position as you see fit." The Golem handles everything on its next tick cycle — it may execute the proposed trade, do something different, or decide to wait.

Use case: The owner trusts the Golem and wants to give it autonomy within the current context.

### Mode 4: Paper Trade

```
F3 (toggle Paper Mode) → F8 → Confirmation Modal → Confirm
```

When Paper Mode is active (indicated by a `⌈ PAPER ⌋` badge in the header and a subtle amber border on all panes), all writes go to a simulated environment instead of mainnet:

**Mirage Mode** (if `mirage-rs` is available and configured):
- Spins up a forked chain at the current block height
- Continues receiving mainnet blocks, so the market keeps moving
- The Golem interacts with the forked network as if it were real
- The chain selector shows `Base (Mirage Fork @ Block #12345678)`
- You can freeze the fork at any point with `F3+F` (stop receiving new blocks)
- You can resume with `F3+R`
- All positions, trades, and outcomes are tagged as `paper:mirage` in the Grimoire

**Estimation Mode** (fallback, always available):
- The Golem uses its own cost estimation and simulation tools (`simulate_price_impact`, `calculate_health_factor`, etc.)
- Results are displayed as projected outcomes, not executed transactions
- The Golem records the hypothetical as a `paper:estimated` episode
- Useful for "what if" scenarios: "what would happen if I swapped 10 ETH right now?"

Paper trading is explicitly tagged in the Grimoire so the Golem can distinguish real experience from simulated experience. Both types contribute to learning, but paper episodes carry a lower confidence weight (0.6x real episodes) during Curator cycles.

---

## 5. Atmosphere

The Sanctum layer uses the **Analytical** atmosphere register (same as Soma > Positions, Trades):

- Cool/neutral palette
- Sharp borders
- Noise floor at 0.2%
- Scanlines at 0.08
- FlashNumbers throughout
- Green/rose tinting on profit/loss
- Fragments suppressed (this is a precision environment)

When Paper Mode is active, the atmosphere shifts slightly:
- Amber undertone (warm, not cool)
- A faint amber border on all panes
- The `⌈ PAPER ⌋` badge pulses slowly
- Data values render with a subtle amber tint to remind the user this isn't real

When the Golem Perspective overlay is active (F2), the atmosphere adds:
- Rose undertone (the Golem's color)
- Ghost text traces of the Golem's reasoning appear at the margins
- The Spectre sidebar becomes more active (eyes shift to scanning mode)

---

## 6. Persistent State

Protocol Views remember their state per-Golem:

- **Last viewed tab** per protocol
- **Form values** (token selections, amounts) persist until the next session
- **Slippage and settings** per protocol per chain
- **Column sort orders** and **filter states** in list views
- **Scroll positions** in logs and lists

State is stored in `~/.bardo/golems/<name>/sanctum/` as TOML files, one per protocol.

---

## 7. Error Handling

### RPC Failures
When the data source (Golem or terminal RPC) fails:
- Stale data is shown with a `STALE` indicator and timestamp of last successful fetch
- A reconnection indicator appears in the header bar
- Write operations are blocked until connection is restored
- The error is logged in the terminal's error log (Hearth > Status tab)

### Tool Failures
When a Golem tool call fails (e.g., `get_pool_info` returns an error):
- The affected pane shows the error message in `rose_dim`
- Other panes continue to function
- `F5` retries the failed call
- Repeated failures trigger a toast notification suggesting the user check the Golem's Status tab

### Transaction Failures
When a write operation fails:
- The Confirmation Modal updates to show the failure reason
- The Golem records the failure as an episode (if a Golem is active)
- Gas spent on the failed transaction is tracked in the Soma > Budget tab
- The user can retry, modify parameters, or cancel

---

*⌈ the sanctum is where the golem's tools become the owner's hands ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
