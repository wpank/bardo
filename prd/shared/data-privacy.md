# Data Retention and Privacy [SPEC]

> **Document Type**: SPEC (normative) | **Referenced by**: All PRDs | **Last Updated**: 2026-03-08

> **Reader orientation:** This document specifies how Bardo handles data retention, privacy, and regulatory compliance (GDPR/CCPA). It belongs to the `shared/` reference layer and applies across all Bardo products. The key concept is the on-chain vs off-chain distinction: on-chain data (wallet addresses, transactions, ERC-8004 attestations) is permanently public and pseudonymous, while off-chain data (strategy theses, Grimoire state, conversation logs) follows standard data subject rights. Golems (mortal autonomous agents) generate and store knowledge in the Grimoire (the agent's persistent knowledge base of episodes, insights, heuristics, warnings, and causal links), which has its own privacy controls. See `prd2/shared/glossary.md` for full term definitions.

---

## 1. Data Architecture

| Data Type                          | Storage Location                 | Retention                       | User Rights                  |
| ---------------------------------- | -------------------------------- | ------------------------------- | ---------------------------- |
| Wallet addresses                   | On-chain (Base/Ethereum)         | Permanent                       | Public (blockchain)          |
| Transaction history                | On-chain (Base/Ethereum)         | Permanent                       | Public (blockchain)          |
| Strategy theses (natural language) | Console database                 | Until user deletes              | Export, delete on request    |
| PLAYBOOK.md / Grimoire state       | Console database or local disk   | Configurable                    | Export (first-class feature) |
| LLM conversation logs              | LLM gateway (ephemeral)          | 24h for debugging, then purged  | Not stored long-term         |
| Performance metrics                | On-chain (ERC-8004 attestations) | Permanent                       | Public (blockchain)          |
| Golem heartbeat logs               | Bardo Compute (VM hosting service for Golems; Fly.io micro VMs provisioned via x402) or local disk   | VM lifetime + 24h export window | Export via Grimoire          |
| Insurance snapshots                | Cloudflare R2 (encrypted)        | 30 days after golem death       | Export, then auto-purge      |

---

## 2. Privacy

- **User PII**: GDPR deletion requests honored within 72 hours for off-chain data
- **Wallet addresses**: Pseudonymous -- no KYC data stored on-chain
- **Golem Grimoire**: Encrypted at rest with user-derived key
- **Session data**: No cross-session PII leakage by default
- **Clade (sibling Golems sharing a common ancestor; exchange knowledge through Styx) knowledge**: Shared knowledge strips PII before transfer (provenance metadata only)

---

## 3. GDPR / CCPA Compliance

### 3.1 On-Chain vs Off-Chain Distinction

On-chain data (wallet addresses, transactions, ERC-8004 attestations) is public and immutable by design. This data is pseudonymous and cannot be deleted from the blockchain.

Off-chain data (strategy theses, Grimoire state, conversation logs, analytics) follows standard data subject access and deletion rights:

| Right                        | Supported?           | Implementation                                        |
| ---------------------------- | -------------------- | ----------------------------------------------------- |
| Right to access              | Yes                  | Export via CLI (`npx @bardo config export`) or portal |
| Right to deletion            | Yes (off-chain only) | Delete API + 72h SLA for managed data                 |
| Right to portability         | Yes                  | JSON export of all off-chain data                     |
| Right to rectification       | Yes (off-chain only) | Edit via console/portal                               |
| Right to restrict processing | Yes                  | Pause golem, disable telemetry                        |

### 3.2 Self-Hosted Exemption

Self-hosted golems store no data with Bardo. The self-sovereign path (see `00-vision/04-trust.md`) means users who run their own agent infrastructure have no off-chain data subject to GDPR requests to Bardo. They are their own data controller.

### 3.3 PII Scanner

The LLM gateway runs a PII scanner (Presidio + crypto extensions) that strips wallet addresses and strategy details before logging. This prevents inadvertent PII leakage in LLM provider logs.

Crypto extensions include:

- Ethereum address detection and redaction
- Private key pattern detection and blocking
- Seed phrase detection and blocking
- API key pattern detection and redaction

---

## 4. Cross-Border Data

- Fly.io regions are selectable per session. Golems choose compute region at session creation.
- No cross-border data transfer occurs for self-hosted golems.
- Managed golems use the region configured at session start.
- EU users can select EU-only compute regions to comply with data residency requirements.

---

## 5. Snapshot Security

- Wallet keys and session keys MUST be stripped from compute snapshots
- Snapshots encrypted at rest with user-derived key
- Download window: 4 hours (reduced from 24h for security)
- After download window expires, snapshots are permanently deleted
- Insurance snapshots (every 6 hours during golem life) follow the same encryption and access controls

---

## 6. Telemetry

Telemetry can be fully disabled with `BARDO_TELEMETRY=false`.

When enabled:

- Events are HMAC-anonymized (no raw user identifiers)
- Funnel events track aggregate behavior, not individual actions
- No strategy content or financial data is included in telemetry
- PostHog is the analytics provider (self-hostable)
- Telemetry data is not sold or shared with third parties

---

## 7. Grimoire Privacy

The Grimoire contains potentially sensitive strategy information. Privacy controls:

| Scope   | Privacy Level      | What Is Shared                                |
| ------- | ------------------ | --------------------------------------------- |
| Private | Full encryption    | Nothing shared with Clade                     |
| Clade   | Controlled sharing | Heuristics and insights (not raw positions)   |
| Public  | Performance only   | Performance cards (returns, Sharpe, drawdown) |

Clade knowledge transfer strips:

- Specific wallet addresses
- Exact position sizes and entry/exit prices
- API keys and credentials
- Personal identifiers

What Clade knowledge transfer preserves:

- Strategy patterns and heuristics
- Market observations and causal links
- Warnings and failure cases
- Confidence scores and provenance metadata
