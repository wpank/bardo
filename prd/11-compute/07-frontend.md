# 07 -- TUI and web dashboard [SPEC]

**TUI as primary frontend, web dashboard as secondary management surface**

> **Reader orientation:** This document specifies the web dashboard for Bardo Compute, the managed VM hosting service for Golems (mortal autonomous DeFi agents). It belongs to the **Compute** layer of Bardo (the Rust runtime for these agents). The key concept before diving in: the TUI (`bardo` CLI, built on `ratatui`) is the primary interactive frontend; the web dashboard at `bardo.money/compute` is a secondary management surface for deployment, monitoring, and fleet overview. Terms like Golem, Styx, Grimoire, and x402 are defined inline on first use; a full glossary lives in `00-overview.md § Terminology`.

---

## Two surfaces, different purposes

The Golem has two user-facing surfaces. They serve different purposes and carry different weight.

### TUI: the primary interactive frontend

The TUI (`bardo` CLI) is a full terminal application built on `ratatui`. It is the primary way owners interact with their Golems. It renders the GolemEvent stream in real-time -- heartbeat ticks, emotional state, vitality curves, dream cycles, trading activity, and death approach are all visible as they happen. The TUI connects to the Golem's event stream through Styx, meaning it works from anywhere, even when the Golem is behind NAT.

The TUI is specified in [prd2/18-interfaces/03-tui.md](../10-surfaces/01-tui.md). This document covers only the web dashboard.

### Web dashboard: the secondary management surface

The web dashboard (`bardo.money/compute/*`) handles VM lifecycle management: provisioning, extensions, destruction, billing. It does NOT attempt to replicate the TUI's real-time cognitive visualization. The dashboard is a REST-driven management interface. The TUI is where you watch your Golem think.

---

## Web application structure

Six pages under `/compute`. Admin auth uses Privy JWT with admin role.

```
/compute                        Dashboard (SSE-powered)
/compute/deploy                 Deploy new Golem
/compute/g/[name]               Golem detail view (owner)
/compute/g/[name]/term          Full-screen terminal (owner)
/compute/g/[name]/public        Public Golem page (no auth)
/compute/admin                  Operator admin dashboard (admin role)
```

---

## Page: Compute dashboard (`/compute`)

### SSE dashboard (replaces polling)

Replace 10s polling with `EventSource` to `GET /v1/machines/mine/events`. Initial data load from `GET /v1/machines/mine`, then SSE for real-time updates.

**Fallback**: If SSE fails, falls back to 30s polling (not 10s -- reduce load on error).

### GolemCard component

| Property | Source | Display |
| --- | --- | --- |
| `machine_name` | API | Monospace identifier |
| `status` | SSE / API | Badge: running, stopped, crashed, destroyed |
| `ttl_remaining` | Computed | Clock-drift-proof countdown |
| `vm_size` | API | e.g., "shared-cpu-1x, 256MB" |
| `region` | API | 3-letter code (ams, ord, sin) |
| `vitality` | API | Vitality bar with phase label |
| `custody_mode` | API | Delegation / Embedded / LocalKey |
| `nav` | API | USD-formatted NAV |
| `ttl_bar` | Computed | Percentage bar with color coding |
| `styx_status` | API | Connected / Degraded / Offline indicator |

### TTL color coding

| Color | Condition | Meaning |
| --- | --- | --- |
| Green | > 1 hour | Healthy |
| Yellow | 10 min - 1 hour | Attention needed |
| Red | < 10 min | Urgent -- extend or lose Golem |
| Gray | 0 / expired | TTL exhausted |

### Vitality indicator

The dashboard shows a vitality bar alongside the TTL bar. This communicates a distinction the TTL bar alone misses: a Golem can have plenty of TTL but low vitality (epistemic decay, stochastic event approaching), or low TTL but high vitality (self-funding Golem about to extend itself).

| Vitality | Phase | Color | Label |
| --- | --- | --- | --- |
| > 0.7 | Thriving | Green | "Thriving" |
| 0.5 - 0.7 | Stable | Blue | "Stable" |
| 0.3 - 0.5 | Conservation | Yellow | "Conserving" |
| 0.1 - 0.3 | Declining | Orange | "Declining" |
| < 0.1 | Terminal | Red | "Terminal" |

### Clock-drift-proof countdown

`useTTLCountdown` hook computing offset from `server_time`. Fly VMs have been observed with up to 33 minutes of clock skew. The offset is computed from each API response's `server_time` field.

---

## Page: Deploy new Golem (`/compute/deploy`)

### Three input modes

1. **Natural language** -- Describe strategy in plain English, compiled to STRATEGY.md
2. **Templates** -- Pick from curated strategy templates
3. **Editor** -- Write STRATEGY.md directly in Monaco/CodeMirror

#### Strategy compilation

**Model**: Claude Sonnet 4

**Validation rules**:
- Must contain all 8 required sections (Overview, Objective, Entry Criteria, Exit Criteria, Risk Parameters, Target Protocols, Asset, Rebalance Frequency)
- Risk parameters must be numeric and within bounds (drawdown 1-50%, position size $1-$100K)
- Target protocols must be from supported list
- Asset must be a valid token symbol

**Cost tracking**: First 3 compilations per user: free. After 3: x402 payment required (phase 2, priced at cost).

### Configuration options

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| VM size | Select | `small` | CPU/memory tier |
| Region | Select | Nearest | Fly region (ams, ord, sin) |
| Duration | Slider | 4 hours | 1h - 30d, micro-USDC cost displayed |
| Tool profile | Select | `full` | DeFi tool profile |
| Custody mode | Select | `delegation` | Delegation / Embedded / LocalKey |
| SSH key | Textarea | None | Optional public key for SSH access |

Duration slider displays micro-USDC cost live as the slider moves.

### Custody mode selection

The deploy form includes a custody mode picker with clear tradeoffs:

| Mode | Setup | Security | Death settlement |
| --- | --- | --- | --- |
| **Delegation** | Sign one ERC-7715 grant in MetaMask | On-chain caveats bound all actions | Delegation expires. No sweep. |
| **Embedded** | Transfer USDC to Privy wallet | TEE-based policy enforcement | Control plane sweeps to owner |
| **LocalKey** | Generate keypair locally | On-chain delegation bounds damage | Delegation expires. Key zeroized. |

Delegation is the recommended default. The form pre-selects it and shows "Recommended" badge.

### Deploy loading states

Step-by-step provisioning progress backed by SSE:

```
Step labels:
1. "Verifying payment..."           (10%)
2. "Starting your Golem..."         (30%)
3. "Loading tools and Grimoire..."  (50%)
4. "Connecting to Styx..."          (65%)
5. "Registering identity..."        (80%)
6. "Settling on Base..."            (90%)
7. "Your Golem is live!"            (100%)
```

Usually takes 3-8 seconds for warm pool.

### Payment flow

1. User configures Golem, selects custody mode, clicks "Deploy"
2. UI calculates cost: `duration_hours * price_per_hour_micro_usdc`
3. User reviews cost breakdown
4. For Delegation mode: user signs ERC-7715 permission grant in MetaMask
5. UI constructs EIP-3009 `receiveWithAuthorization` signature request
6. User signs with wallet
7. UI sends `POST /v1/machines` with `X-402-Payment` header
8. UI receives `202 Accepted` with `machine_name`
9. UI connects to SSE `GET /v1/machines/:name/provision-status`
10. On ready: redirect to detail page, show TUI connection command

### Empty/error states

**0 Golems (first visit)**: CTA "Deploy Your First Golem" with link to docs.

**Failed provision**: "Provisioning failed. You were not charged." with retry and change-region buttons.

**Crashed Golem**: Shows crash reason, restart count (5/5), and options to redeploy (keeps Grimoire), view logs, or destroy.

---

## Page: Golem detail (`/compute/g/[name]`)

Tabbed layout with 7 tabs: **Overview**, **Strategy**, **Terminal**, **Logs**, **Grimoire**, **Wallet**, **Custody**.

### Overview tab

- Status, TTL countdown, vitality bar, behavioral phase
- Styx connection status
- Sustainability ratio (if self-funding)
- Recent GolemEvents (last 10, from SSE)
- Quick action buttons

### Custody tab

Shows delegation details (for Delegation mode), Privy wallet details (for Embedded mode), or local key status (for LocalKey mode):

- Active delegation hash, caveat enforcers, remaining budget
- Session key address, rotation policy, operations signed
- Delegation expiry countdown (maps to MortalityTimeWindow)

### Mobile quick actions

| Action | Target |
| --- | --- |
| Send steer | Heartbeat pipeline |
| Check state | Health endpoint |
| Extend TTL | Extension modal |
| View in TUI | TUI connection command |
| Export strategy | Download STRATEGY.md |
| Destroy | Confirmation modal |

### Wallet balance clarity

Wallet section shows clear distinction based on custody mode:

- **Delegation**: "Funds remain in your MetaMask Smart Account. This Golem operates via delegation."
- **Embedded**: Full wallet address with copy. USDC and ETH balances. "This is your Golem's trading wallet. Funds are swept to you when the Golem dies."
- **LocalKey**: Local key address with delegation bounds displayed.

---

## Page: Public Golem (`/compute/g/[name]/public`)

No authentication required. Shows read-only Golem status, strategy summary, vitality, equity curve, and extension buttons.

### Extension explainer

Context for non-crypto users:
- "This Golem runs on prepaid compute credits (USDC). When credits run out, it shuts down. Anyone can add more time -- no account needed."
- Step-by-step: click button -> connect wallet -> approve USDC -> timer extends

### Permissionless extension buttons

Pre-configured durations: `[+4h $0.20]`, `[+24h $1.20]`, `[+7d $8.40]`

Flow: connect any EVM wallet -> sign EIP-3009 -> `POST /v1/machines/:name/extend` -> TTL extends, SSE pushes update.

---

## Page: Terminal (`/compute/g/[name]/term`)

Full-screen terminal emulator (xterm.js). Connects via WebSocket through SSH CA proxy. Uses short-lived tickets.

Features: full PTY support (resize, colors, cursor positioning), automatic reconnection on disconnect, copy/paste support.

The terminal provides raw SSH access to the VM. For the rich Golem interaction experience (heartbeat visualization, vitality curves, emotional state, creature display), use the TUI via `bardo attach <golem-name>`.

---

## Page: Admin dashboard (`/compute/admin`)

Auth: Privy JWT with admin role.

### Health indicators

| Service | Tracked metrics |
| --- | --- |
| Turso DB | Status, latency |
| Fly Machines API | Status, latency |
| TTL Worker | Running status, last run time |
| SSH CA | Status, latency |
| Warm Pool | Size per region, claim rate |

### Admin sections

- **Fleet overview** -- All Golems across all users, filterable by status/region/size/custody-mode
- **Revenue** -- Total USDC collected, settled, pending. Self-extension revenue tracked separately.
- **Warm pool** -- Pool sizes by region, claim rates, replenishment queue
- **TTL worker** -- Last run, next run, machines approaching expiry
- **User lookup** -- Search by wallet address, view all Golems for a user
- **Vitality overview** -- Golems by behavioral phase, approaching Terminal count

---

## TUI connection from web

After deploying a Golem or viewing a detail page, the web dashboard displays a TUI connection command:

```
bardo attach golem-V1StGXR8_Z5j
```

This connects the TUI to the Golem's event stream through Styx. The TUI renders the full cognitive visualization: heartbeat pipeline, emotional state, vitality curves, dream cycles, creature display, trading activity.

For remote self-hosted Golems, the TUI connects through Styx:

```
TUI <-- WSS ---- Styx <-- WSS ---- Golem (behind NAT)
```

Events flow from Golem to Styx to TUI. Steers flow from TUI to Styx to Golem. The deployment location is invisible. The experience is universal.

---

## Component library

| Component | Usage |
| --- | --- |
| `GolemCard` | Machine card on dashboard |
| `TTLCountdown` | Clock-drift-proof countdown |
| `TTLBar` | Visual progress bar with color coding |
| `VitalityBar` | Vitality percentage with phase label |
| `EquityCurve` | Recharts NAV chart |
| `X402PayButton` | x402 payment button |
| `StrategyEditor` | Monaco/CodeMirror editor |
| `LogStream` | SSE-backed log viewer |
| `DeployWizard` | Multi-step deploy with SSE progress |
| `CustodyModeSelector` | Delegation/Embedded/LocalKey picker |
| `GolemStatus` | Status badge (running, stopped, crashed) |
| `StyxIndicator` | Styx connection status (green/amber/red/gray) |
| `RegionSelector` | Region picker with live capacity |
| `VMSizePicker` | Size selector with micro-USDC pricing |
| `ProvisionProgress` | Step-by-step SSE progress indicator |
| `QuickActions` | Mobile action shortcuts |
| `ExtensionExplainer` | Non-crypto user context |
| `TuiConnectionCommand` | Copyable `bardo attach` command |

---

## Design system tokens

### Colors

| Token | Value | Usage |
| --- | --- | --- |
| `--color-primary` | `#FF007A` | Uniswap pink, primary |
| `--color-bg` | `#191B1F` | Page background |
| `--color-surface` | `#212429` | Card/panel background |
| `--color-border` | `#2C2F36` | Borders, dividers |
| `--color-text` | `#C3C5CB` | Body text |
| `--color-text-primary` | `#FFFFFF` | Headings, emphasis |
| `--color-success` | `#27AE60` | Running, healthy |
| `--color-warning` | `#F2994A` | TTL warning, attention |
| `--color-error` | `#EB5757` | Errors, critical TTL |
| `--color-muted` | `#6C7284` | Disabled, secondary |

### Typography

| Element | Font | Size | Weight |
| --- | --- | --- | --- |
| Heading 1 | Inter | 24px | 600 |
| Heading 2 | Inter | 20px | 600 |
| Body | Inter | 14px | 400 |
| Mono | JetBrains Mono | 13px | 400 |
| Caption | Inter | 12px | 400 |

### Spacing

4px base unit. Standard increments: 4, 8, 12, 16, 24, 32, 48, 64.

---

## Mobile considerations

| Page | Mobile behavior |
| --- | --- |
| Dashboard | SSE-powered, no polling overhead on mobile |
| Deploy | Full-width form, custody mode selector, bottom-fixed Deploy btn |
| Detail | Quick actions section replaces sidebar on mobile. TUI command displayed prominently. |
| Terminal | Not supported on mobile (hidden). "Use TUI from a desktop terminal" message shown. |
| Public | Responsive, extension explainer stacks vertically |
| Admin | Table scrolls horizontally |

---

## Responsive breakpoints

| Breakpoint | Width | Layout |
| --- | --- | --- |
| Mobile | < 640px | Single column, stacked cards |
| Tablet | 640px - 1024px | Two-column grid |
| Desktop | > 1024px | Full layout, sidebar visible |

---

## Cross-references

| Topic | Document |
| --- | --- |
| Architecture | [01-architecture.md](01-architecture.md) — Two-app Fly.io topology (bardo-control + bardo-machines), host header routing, subdomain proxy, and two-tier VM endpoint model. |
| API reference | [06-api.md](06-api.md) — Full REST API specification, Turso database schema, GolemEvent types, and HTTP error codes. |
| SSE events format | [01-architecture.md](01-architecture.md) — Specifies the SSE event stream format that the web dashboard subscribes to for real-time Golem state updates. |
| SSH ticket flow | [04-security.md](04-security.md) — Single-use ticket-based SSH authentication with 30-second TTL and 5-minute certificate validity. |
| TUI specification | [prd2/18-interfaces/03-tui.md](../18-interfaces/03-tui.md) — Full TUI specification: ratatui-based terminal application with real-time GolemEvent rendering, heartbeat visualization, and emotional state display. |
| GolemEvent catalog | [prd2/01-golem/13-runtime-extensions.md](../01-golem/13-runtime-extensions.md) — The 28-extension architecture and 50+ typed GolemEvent variants that feed both TUI and web dashboard. |
| Custody modes | [prd2/10-safety/01-custody.md](../10-safety/01-custody.md) — Three wallet custody modes (Delegation, Embedded, LocalKey), seven caveat enforcers, and death settlement flows. |
