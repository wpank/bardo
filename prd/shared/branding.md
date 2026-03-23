# Bardo Brand Guidelines [SPEC]

> **Document Type**: REF (normative) | **Referenced by**: All PRDs, all documentation | **Last Updated**: 2026-03-08
>
> Defines the Bardo brand identity, voice, naming conventions, and terminology rules. All `prd2/` documents, public documentation, and product interfaces MUST follow these guidelines.

> **Reader orientation:** This document defines the Bardo brand identity, voice, naming conventions, and terminology rules. It belongs to the `shared/` reference layer and applies to every document, interface, and product across the Bardo ecosystem. The key concept here is that Bardo is not just a brand name but a philosophical framework: "bardo" (Tibetan for "intermediate state") reflects the system's core commitment to mortality, transition, and knowledge inheritance as architectural primitives. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. The Name

**Bardo** (Tibetan: bar do, "intermediate state") -- the transitional state between death and rebirth where consciousness navigates, chooses, and transforms before entering its next vessel.

The name was chosen because:

- Agents executing the Death Protocol exist in a bardo state: active between lives, not passive in death.
- The Tibetan Book of the Dead is a manual for navigating the bardo successfully, mapping directly to the protocol's knowledge-packaging process.
- The concept implies **agency within transition** -- not mere waiting, but active navigation through uncertainty.
- It is warm, two syllables, globally pronounceable. Carries philosophical weight without pretension.

### 1.1 The Six Bardos

| Bardo                                   | Tibetan Meaning       | Golem Lifecycle Mapping                                                  |
| --------------------------------------- | --------------------- | ------------------------------------------------------------------------ |
| **Birth Bardo** (skye ba bar do)        | Ordinary waking life  | Creation and provisioning                                                |
| **Dream Bardo** (rmi lam bar do)        | The state of dreaming | Simulation, backtesting, hypothesis testing                              |
| **Meditation Bardo** (bsam gtan bar do) | Meditative absorption | Strategic reflection (Loop 2), PLAYBOOK.md evolution                     |
| **Dying Bardo** ('chi kha bar do)       | The moment of death   | Terminal phase, Death Protocol initiation                                |
| **Dharmata Bardo** (chos nyid bar do)   | Reality revealed      | Death Protocol Phase II (Reflect): confronting true performance          |
| **Becoming Bardo** (srid pa bar do)     | Seeking new vessel    | Succession: knowledge inheritance, Clade push, successor crystallization |

### 1.2 The Clear Light

In Tibetan Buddhism, the Clear Light (od gsal) is the luminous nature of mind revealed at the moment of death. If recognized, the practitioner achieves liberation. If unrecognized, the practitioner enters the next bardo.

For golems, the Clear Light is the Death Protocol's Reflect phase: the moment where the agent must honestly confront what it learned, what it failed to learn, and what confused it. A golem that produces a genuine reflection (sovereign death) achieves objective immortality through its Grimoire contribution [WHITEHEAD-1929]. A golem that merely dumps raw data achieves nothing lasting.

---

## 2. Brand Voice

### 2.1 Principles

| Principle                    | Description                                              | Example                                                                             |
| ---------------------------- | -------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| **Warm**                     | Approachable, human, never cold or corporate             | "Your golem earned $4.17 in management fees this week" not "Revenue accrual: $4.17" |
| **Philosophical**            | Grounded in intellectual traditions, never superficially | "Mortality is a feature" not "Our agents have limited lifespans"                    |
| **Technically precise**      | Exact terminology, no hand-waving                        | "ERC-4626 vault with \_decimalsOffset() = 6" not "a smart contract vault"           |
| **Honest about limitations** | Never oversell, acknowledge unknowns                     | "This architecture cannot prevent all attacks" not "Fully secure"                   |
| **Forward-looking**          | Specifications describe what to build, not what exists   | Present tense for spec; past tense only in implementation-state docs                |

### 2.2 Tone Examples

**Do**: "The golem's USDC balance is Jonas's metabolic substrate -- the resource whose depletion gives decisions weight."

**Do not**: "We use cutting-edge AI to manage your DeFi portfolio."

**Do**: "When survival pressure drops below 0.1, the Death Protocol initiates. There is no override."

**Do not**: "Our advanced system gracefully handles end-of-life scenarios."

---

## 3. Naming Conventions

### 3.1 Product Names

| Product         | Full Name       | Short Name | Description                                      |
| --------------- | --------------- | ---------- | ------------------------------------------------ |
| Ecosystem       | Bardo           | --         | The overall ecosystem and brand                  |
| Tool Library    | Bardo Tools     | Tools      | Unified Uniswap Pi-native tool library           |
| Vault Protocol  | Bardo Vaults    | Vaults     | ERC-4626 tokenized vaults                        |
| VM Hosting      | Bardo Compute   | Compute    | x402-gated VM hosting for golems                 |
| LLM Gateway     | Bardo Inference | Inference  | x402-gated multi-provider LLM inference          |
| Social Bot      | Bardo Bott      | Bott       | Conversational golem deployment via social media |
| Knowledge Store | Bardo Grimoire  | Grimoire   | Persistent knowledge architecture                |

### 3.2 Package Names

TypeScript packages use the `@bardo/` npm scope. The Golem runtime is a separate Rust workspace (`bardo-golem-rs`) with Cargo crate names.

**TypeScript npm packages** (`@bardo/` scope):

| Package           | npm Name           |
| ----------------- | ------------------ |
| Core              | `@bardo/core`      |
| Chain             | `@bardo/chain`     |
| Wallet            | `@bardo/wallet`    |
| Tools             | `@bardo/tools`     |
| Vault             | `@bardo/vault`     |
| Warden (deferred) | `@bardo/warden`    |
| CLI               | `@bardo`           |
| Mirage            | `@bardo/mirage`    |
| Social (new)      | `@bardo/social`    |

**Rust crates** (Cargo, in `bardo-golem-rs/`):

| Component  | Crate name          |
| ---------- | ------------------- |
| Golem      | `golem-runtime`     |
| Grimoire   | `golem-grimoire`    |
| Inference  | `golem-inference`   |
| Daimon     | `golem-daimon`      |
| Mortality  | `golem-mortality`   |
| Dreams     | `golem-dreams`      |
| Tools      | `golem-tools`       |

### 3.3 Environment Variables

All environment variables use the `BARDO_` prefix:

| Variable                 | Purpose                                      |
| ------------------------ | -------------------------------------------- |
| `BARDO_TOOL_PROFILE`     | Tool profile activation                      |
| `BARDO_DEFERRED_LOADING` | Enable/disable deferred tool loading         |
| `BARDO_MAX_ACTIVE_TOOLS` | Session cap for active tools (default 40)    |
| `BARDO_CORE_TOOLS`       | Override default core tools list             |
| `BARDO_IPFS_MODE`        | IPFS upload strategy (bardo, pinata, inline) |
| `BARDO_TELEMETRY`        | Enable/disable telemetry                     |

### 3.4 API Key Prefixes

| Key Tier | Prefix             | Scope                      |
| -------- | ------------------ | -------------------------- |
| Read     | `bardo_read_*`     | Query-only tools           |
| Feedback | `bardo_feedback_*` | Read + feedback submission |
| Write    | `bardo_write_*`    | Full execution access      |

### 3.5 Config Paths

- Config directory: `~/.bardo/`
- Config file: `~/.bardo/config.json`
- Tool prefix: `mcp__bardo_tools__`

---

## 4. Terminology Rules

### 4.1 Preferred Terms

Always use the term on the left. Never use the term on the right.

| Use This          | Not This                                | Reason                                                                      |
| ----------------- | --------------------------------------- | --------------------------------------------------------------------------- |
| golem             | agent (when referring to a Bardo agent) | "Agent" is generic; "golem" carries the mortality and inscription semantics |
| grimoire          | knowledge base, memory store            | The Grimoire is a specific architecture, not a generic store                |
| clade             | network, cluster, swarm                 | Clades are Espositean communitas with obligatory knowledge sharing          |
| PLAYBOOK.md       | rulebook, strategy file                 | Specific format with divergence requirements                                |
| bardo             | transition, death state                 | The brand name carries specific philosophical meaning                       |
| pool              | pair (for V3/V4)                        | Uniswap V3/V4 terminology                                                   |
| position          | LP stake, deposit                       | Concentrated liquidity terminology                                          |
| tick              | price boundary                          | Uniswap V3/V4 terminology                                                   |
| hook              | plugin, extension (for V4)              | Uniswap V4 terminology                                                      |
| vault             | fund, portfolio                         | ERC-4626 terminology                                                        |
| share             | receipt token, LP token (for vaults)    | ERC-4626 terminology                                                        |
| agentId           | agent address, agent token              | ERC-8004 terminology                                                        |
| intent            | order (for cross-chain)                 | ERC-7683 terminology                                                        |
| warden            | agent proxy, time-lock                  | Bardo-specific: the optional Warden guards against hasty execution (deferred; see `prd2-extended/10-safety/02-warden.md`) |
| skill             | command, action (for user-facing)       | Bardo-specific: skills parse intent                                         |
| survival pressure | health, status                          | Specific scalar from 0.0 to 1.0                                             |

### 4.2 Context-Dependent Terms

- **Agent**: Use when referring to generic autonomous software agents, industry terminology, or non-Bardo agents. Use **golem** when referring to a Bardo-specific mortal agent.
- **Death**: Use literally. Golems die. Do not euphemize as "decommission," "sunset," or "retire." The Death Protocol is a death protocol.
- **Mortality**: A feature, not a limitation. Never frame mortality apologetically.

---

## 5. Visual Identity Principles

### 5.1 Typography

- Headers: Clean, geometric sans-serif
- Body: Readable, proportional sans-serif
- Code: Monospace (system default)

### 5.2 Color Palette (ROSEDUST)

The canonical palette is **ROSEDUST** -- rose light on violet-black, seen through dirty CRT glass. 80% of visible color on any screen is rose or its variants. See `tmp/research/mmo2/15-design-system.md` for the full ROSEDUST token table.

- Background: `bg_void #060608` -- nearly black with a violet undertone. Never pure `#000000`.
- Primary: Rose spectrum (`#AA7088` family) -- the color of consciousness, active data, alive state.
- Accent: Bone (`#C8B890`) -- used sparingly, once per screen max. The single most important element.
- Dream: Dream indigo (`#585878`) -- replaces rose only during dream states or altered consciousness.
- Success: Muted sage (`#70887A`)
- Warning: Amber (`#AA8855`)
- Error/Danger: Rose bright (`#CC90A8`)

### 5.3 Iconography

- No emojis in TUI or terminal output. Unicode symbols/dingbats only (U+0000-U+FFFF BMP range)
- Icons from `packages/tui/src/icons.ts` are the single source of truth
- Visual metaphors: transitions, gradients, liminal spaces -- not skulls, not Buddhist imagery

### 5.4 Imagery Guidelines

- Avoid literal Buddhist imagery (this is a technical brand, not a religious one)
- Favor abstract representations of transition, gradients, metamorphosis
- Avoid stock photography of "AI" or "blockchain" (no robot hands, no glowing chains)

---

## 6. Differentiation

### 6.1 Google "Bard" (Now Gemini)

Google's chatbot was originally named "Bard" (one letter difference). Rebranded to Gemini in early 2024. Risk is diminishing as "Bard" fades from public memory, but:

- Always use "Bardo" (never abbreviate to "Bard")
- The extra letter and the diacritical distinction (Bardo is Tibetan, Bard is English) provide sufficient differentiation
- Context differentiates: Bardo operates in DeFi/agent infrastructure, not general chat

### 6.2 Lombard Protocol "BARD" Token

Lombard Protocol has a token with the ticker "BARD." Low overlap risk:

- Different product category (Lombard is Bitcoin liquid staking; Bardo is agent infrastructure)
- Different branding context (token ticker vs. ecosystem brand)
- No trademark conflict (tickers are not trademarks)

### 6.3 Positioning Statement

Bardo is **permissionless infrastructure for mortal autonomous agents in DeFi**. It is not:

- A chatbot (not Bard/Gemini)
- A token (not BARD)
- A yield optimizer (not Yearn)
- A vault aggregator (not Morpho)
- A social trading platform (not dHEDGE)

The differentiator is mortality as architecture: finite-lived agents that accumulate, distill, and inherit knowledge across generations through the genomic bottleneck (compression step at death that reduces the full Grimoire to <=2048 entries for inheritance), creating evolutionary improvement that no immortal agent can achieve [BENNETT-1982]. — *Bennett (1982) argues that logical depth, not mere information content, measures the computational work embedded in a structure -- the basis for why Golem knowledge distillation at death produces more value than raw data dumps.*
