# Documentation Standards [SPEC]

> **Document Type**: REF (normative) | **Referenced by**: All PRDs | **Last Updated**: 2026-03-08
>
> Standards for all documents in `prd2/`. All contributors MUST follow these conventions.

> **Reader orientation:** This document defines the writing and formatting standards for all `prd2/` documents in the Bardo ecosystem. It covers file naming, document structure, citation format, cross-referencing, terminology rules, and canonical counts. The key concept is that Bardo documentation distinguishes between SPEC (normative, must-be-built) and REF (informative, advisory) document types. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. File Naming

### 1.1 Numbered Prefixes

Files within each directory use two-digit numeric prefixes for ordering:

```
00-overview.md
01-architecture.md
02-tools-data.md
...
```

Shared reference files (in `prd2/shared/`) use descriptive names without numeric prefixes:

```
glossary.md
citations.md
chains.md
```

### 1.2 Naming Rules

- Lowercase with hyphens: `agent-economy.md` not `AgentEconomy.md`
- Descriptive: `tools-trading.md` not `section-3.md`
- No spaces, no underscores in filenames
- Appendix files in `prd2/appendices/` use descriptive names without prefixes

### 1.3 Directory Structure

The full `prd2/` directory has 19 numbered sections plus shared and appendix directories:

```
prd2/
├── 00-vision/          # Brand, thesis, architecture, philosophy
├── 01-golem/           # Agent lifecycle, cognition, heartbeat, creation
├── 02-mortality/       # Three clocks, behavioral phases, death protocol
├── 03-daimon/          # Affect engine, PAD vectors, appraisal, behavior
├── 04-memory/          # Grimoire, Crypt, Oracle, Lethe, forgetting
├── 05-dreams/          # Offline optimization, replay, imagination
├── 07-tools/         # Tool library, profiles, wallets
├── 08-vault/           # ERC-4626 vaults, contracts, SDK, fees
├── 09-economy/         # Identity, reputation, clade, marketplace
├── 10-safety/          # Defense layers, custody, warden, policy
├── 11-compute/         # Compute infrastructure, provisioning
├── 12-inference/       # LLM routing, caching, sessions, memory
├── 13-runtime/         # User interaction, state model, creature system
├── 20-styx/            # Styx knowledge services (Vault, Clade, Lethe (formerly Commons), API, infrastructure)
├── 15-dev/             # Local dev stack, debug UI, testing
├── 16-testing/         # Thesis validation, Gauntlet, Mirage
├── 17-monorepo/        # Package structure, build, conventions
├── 18-interfaces/      # Portal, CLI, UI system, TUI
├── 19-agents-skills/   # Agent/skill definitions, composition
├── shared/             # Glossary, chains, branding, citations
└── appendices/         # Market context, implementation state
```

---

## 2. Document Structure

### 2.1 Header Block

Every document starts with:

```markdown
# Title

> **Document Type**: SPEC (normative) | REF (informative)
> **Scope**: Which packages/products this covers
> **Prerequisites**: Links to required reading
> **Last Updated**: YYYY-MM-DD
>
> Brief description of the document's purpose and scope.
```

### 2.2 Document Types

| Type                  | Meaning                                                   | Authority     |
| --------------------- | --------------------------------------------------------- | ------------- |
| **SPEC** (normative)  | Defines what MUST be built. Implementations MUST conform. | Authoritative |
| **REF** (informative) | Reference material. Useful context but not binding.       | Advisory      |

### 2.3 Section Structure

- Use `##` for major sections (numbered: `## 1. Overview`)
- Use `###` for subsections (numbered: `### 1.1 Design Principles`)
- Use `####` sparingly for sub-subsections
- Tables for structured data (capabilities, parameters, comparisons)
- Code blocks for interfaces, configurations, and examples
- Horizontal rules (`---`) between major sections

### 2.4 RFC 2119 Keywords

Use RFC 2119 keywords in normative sections:

| Keyword        | Meaning                        |
| -------------- | ------------------------------ |
| **MUST**       | Absolute requirement           |
| **MUST NOT**   | Absolute prohibition           |
| **SHOULD**     | Recommended but not required   |
| **SHOULD NOT** | Discouraged but not prohibited |
| **MAY**        | Truly optional                 |

---

## 3. Citation Format

### 3.1 In-Text Citations

Use `[CITATION-KEY]` format in-text. Every citation key MUST have an entry in `prd2/shared/citations.md`.

**Example**:

```markdown
As demonstrated by Milionis et al. [AMM-LVR-2023], loss-versus-rebalancing is ~5x lower on Base than mainnet.
```

### 3.2 Citation Keys

- Format: `AUTHOR-YEAR` or `ACRONYM-YEAR` (e.g., `[JONAS-1966]`, `[AMM-LVR-2023]`)
- Keys are uppercase with hyphens
- Unique across the entire `prd2/` corpus
- Full bibliographic entry in `prd2/shared/citations.md`

### 3.3 Citation Checklist

- Every in-text `[KEY]` has an entry in `citations.md`
- Every entry in `citations.md` is referenced in at least one `prd2/` document
- No orphaned citations in either direction
- Market data citations include access date

---

## 4. Cross-Reference Format

### 4.1 Internal Links

Use relative paths for cross-references within `prd2/`:

```markdown
See [contracts specification](../08-vault/01-contracts.md) for Solidity details.
```

### 4.2 Glossary References

When first using a defined term in a section, optionally link to the glossary:

```markdown
The [golem](../shared/glossary.md#g) executes its heartbeat cycle...
```

After first use, plain text is fine (no need to link every occurrence).

### 4.3 External Links

Use full URLs for external references:

```markdown
Source of truth: [Uniswap V4 Deployments](https://docs.uniswap.org/contracts/v4/deployments)
```

---

## 5. Terminology Rules

Follow the terminology rules in `prd2/shared/branding.md` (defines brand identity, voice, naming conventions, and preferred terminology for all Bardo documentation). Key rules:

- Use **golem** not "agent" for Bardo-specific agents
- Use **grimoire** not "knowledge base"
- Use **clade** not "network" or "swarm"
- Use **pool** not "pair" for V3/V4
- Use **vault** not "fund"
- Use **share** not "receipt token" for ERC-4626
- Use **warden** not "agent proxy"
- No emojis in any document

---

## 6. Canonical Counts (Normative)

When numeric counts are referenced across documents, this table is authoritative. Documents MUST NOT contradict these counts.

| Metric                   | Count    | Canonical Source                         |
| ------------------------ | -------- | ---------------------------------------- |
| Safety layers            | 15       | `10-safety/00-defense.md`                |
| Tools (total)            | 158+     | `07-tools/00-overview.md`              |
| Vault tools (core v1)    | 24       | `07-tools/05-tools-vault.md`           |
| Vault tools (proxy)      | 6        | `07-tools/05-tools-vault.md`           |
| Vault tools (am-AMM)     | 5        | `07-tools/05-tools-vault.md`           |
| Agents (core)            | 25       | `19-agents-skills/00-agents-overview.md` |
| Agents (vault)           | 7        | `19-agents-skills/10-vault-agents.md`    |
| Agents (total)           | 32+      | `19-agents-skills/00-agents-overview.md` |
| Skills (core)            | 61       | `19-agents-skills/04-skills-overview.md` |
| Skills (vault)           | 7        | `19-agents-skills/04-skills-overview.md` |
| Skills (total)           | 68       | `19-agents-skills/04-skills-overview.md` |
| Slash commands           | 68       | `19-agents-skills/04-skills-overview.md` |
| Composition patterns     | 14       | `19-agents-skills/11-composition.md`     |
| Tool profiles            | 11       | `07-tools/08-profiles.md`              |
| Wallet types             | 7        | `07-tools/09-wallets.md`               |
| Supported chains         | 12       | `shared/chains.md` (11 + Anvil)          |
| Delay tiers              | 5        | `prd2-extended/10-safety/02-warden.md` (optional, deferred) |
| Workspace packages       | 20+      | `17-monorepo/00-packages.md`             |
| Terminal agent nodes     | 8        | `19-agents-skills/11-composition.md`     |
| Max delegation depth     | 3        | `19-agents-skills/11-composition.md`     |
| Golem survival phases    | 5        | `02-mortality/01-architecture.md`        |
| Reputation milestones    | 20       | `09-economy/01-reputation.md`            |
| Reputation max score     | 1000     | `09-economy/01-reputation.md`            |
| Onboarding tiers         | 5        | `09-economy/00-identity.md`              |
| Golem extensions         | 14       | `01-golem/00-overview.md`                |
| Philosophical traditions | 20+      | `00-vision/03-philosophy.md`             |
| Total citations          | 265+     | `shared/citations.md`                    |
| Fee cap (management)     | 500 bps  | `08-vault/04-fees.md`                    |
| Fee cap (performance)    | 5000 bps | `08-vault/04-fees.md`                    |
| Virtual shares offset    | 6        | `08-vault/01-contracts.md`               |

---

## 7. Code Examples

### 7.1 Language Tags

Always specify the language for code blocks:

````markdown
```typescript
const client = getClient(chainId);
```

```solidity
function deposit(uint256 assets) external returns (uint256 shares);
```

```yaml
profile: vault
tools: 24
```
````

### 7.2 Conventions

- TypeScript examples use `@bardo/` package names
- Solidity examples include NatSpec for public functions
- All code examples should be syntactically valid
- Use `// ...` for elided code, not `...`

---

## 8. Tables

### 8.1 Format

- Use markdown tables with header row and alignment
- Left-align text columns, right-align numeric columns where sensible
- Keep tables readable at 120-character line width
- For wide tables, prefer splitting into multiple narrower tables

### 8.2 Normative Tables

Tables in normative sections that define behavior (e.g., risk tiers, fee caps, chain capabilities) MUST include `(Normative)` in the section header.

---

## 9. Diagrams

- Use ASCII art or Mermaid syntax for inline diagrams
- Label all transitions and states
- Keep diagrams simple; complex architectures should be separate figures

---

## 10. Review Process

- All `prd2/` changes require review
- Golden baseline changes (counts, addresses, fee caps) require explicit acknowledgment
- Citation additions must include both the `citations.md` entry and at least one in-text reference
- Terminology changes must update `glossary.md` and `branding.md` simultaneously
