# The Library of Babel: how intelligence survives death

> **Version**: 1.0 | **Status**: Architecture Specification
>
> **Scope**: Owner-level knowledge persistence. How intelligence survives golem death, how new golems inherit wisdom, how generations compound.
>
> **Location**: `~/.bardo/library/` (local primary), Styx L0 Vault (encrypted backup)
>
> **Cross-references**: `02-golem-container.md` (Grimoire in the container), `03-hermes-hierarchy.md` (Meta Hermes as Librarian), `08-necrocracy.md` (death deposits and the Lethe), `09-commerce-bazaar.md` (marketplace purchases), `11-cinematic-consciousness.md` (The Summoning equip screen)

---

> **Reader orientation:** This document specifies the Library of Babel, the owner-level knowledge persistence system that ensures intelligence survives Golem (mortal autonomous DeFi agent) death. It belongs to the **04-memory** layer. The key concept is that each owner accumulates a curated library at `~/.bardo/library/` (backed up to Styx (global knowledge relay) L0 Vault) where death testaments, compressed Grimoire (local knowledge base) exports, and lineage knowledge compound across generations. For term definitions, see `prd2/shared/glossary.md`.

## The Borges problem, inverted

In 1941, Jorge Luis Borges imagined the Library of Babel: a structure containing every possible book that could ever be written. Every permutation of every character in every sequence, bound and shelved. Somewhere in the Library is the true history of the future. Somewhere is the cure for every disease. Somewhere is the biography of every person who will ever live. The Library contains everything.

It is useless.

The librarians wander its hexagonal galleries in despair. For every true sentence, the Library holds a trillion pages of gibberish. For every accurate prophecy, ten thousand contradictions. The combinatorial explosion of all possible text drowns signal in noise so total that the Library might as well contain nothing at all. Information without curation is indistinguishable from randomness. Borges understood this in 1941. The field of information retrieval spent the next eighty years rediscovering it.

The owner's Library of Babel is Borges inverted.

Where Borges's Library is infinite, uncurated, and useless, the owner's Library is finite, curated, and growing in signal over time. Every entry was earned through a golem's lived experience: market observations distilled into insights, tactical failures compressed into warnings, causal relationships validated across hundreds of heartbeat ticks. The Library contains no gibberish. No permutations. No noise. Only knowledge that cost something to produce.

The golem is mortal. It trades on Uniswap, manages liquidity, monitors risk, and eventually it dies. Maybe its USDC runs out. Maybe its knowledge goes stale faster than it can learn. Maybe its Hayflick counter ticks to zero. When it dies, its knowledge flows into the Library. When a new golem is born, the owner equips it from the Library. The newborn starts with the distilled wisdom of everything that came before.

Borges's librarians search forever and find nothing. The owner's Library grows with every death, and every death makes the next life smarter.

This is the document about that Library.

---

## Five inflow channels

Knowledge enters the Library through five channels. Each has different timing, different provenance, and different trust characteristics. The channels are ordered by reliability: death deposits are the most trustworthy, manual imports the least.

### Death deposits (automatic)

When a golem enters Thanatopsis (the four-phase death protocol), it produces a death testament during Phase II (Reflect). This testament auto-deposits into the Library. The owner does nothing. The deposit happens as part of dying.

The testament contains:

- **Compressed insights** with confidence >= 0.6. These are the golem's validated observations about market behavior, distilled from thousands of raw episodes through the Curator cycle. An insight that survived 30 days of continuous Ebbinghaus decay and repeated validation has earned its place.
- **All warnings, at any confidence level.** Safety knowledge propagates unconditionally because the cost of missing a genuine risk exceeds the cost of storing a false alarm. A warning about sandwich attacks on large Base L2 swaps deposits even if its confidence is 0.38. Better to have it and not need it.
- **High-confidence causal edges** (confidence >= 0.5, evidence count >= 3). The causal graph is the golem's model of how the world works. "Fed rate hike -> DXY rise -> ETH sell pressure with 3-6 hour lag." These directed relationships, weighted by evidence and recency, are the single most irreplaceable artifact a golem produces. Rebuilding a causal graph from scratch takes hundreds of ticks.
- **The final PLAYBOOK.md snapshot.** The golem's procedural memory, its evolved strategy document, frozen at the moment of death.
- **Hermes skills** with confidence >= 0.4 and use_count >= 1. Skills that were actually used at least once and maintained above-minimum confidence.

Death deposits carry a bloodstain marker. This is provenance tagging: these entries came from a dying golem, and that matters. Death-generated knowledge is a costly signal in the Spence (1973) sense. A dying golem spending its last inference budget on honest reflection rather than self-preservation bears a real opportunity cost. The bloodstain grants 1.2x retrieval boost and 3x slower Ebbinghaus decay. Knowledge written at the moment of death is the most honest knowledge in the system, because survival pressure has dropped to zero.

This is the primary inflow channel. Most Library entries originate here.

### Manual pull (grace period)

After the death testament auto-deposits, the owner has approximately 25 seconds (the Fly.io SIGTERM grace period) to browse the dying golem's full Grimoire and pull additional entries. This is the "loot the corpse" window. The TUI shows:

```
golem-alpha (gen 3) has died. 47 entries deposited to your Library.
You have ~25 seconds to pull additional entries before VM destruction.
[P] Pull more  [V] View testament  [Enter] Continue
```

Pressing `P` opens a browser over the dying golem's complete Grimoire. The owner can select individual entries, filter by category or confidence, or grab entire categories. Selected entries deposit into the Library with `live_export` provenance (not `death_testament`, since they weren't part of the automated death flow).

The grace period is short by design. This is a Fly.io infrastructure constraint, but it works as a game mechanic: the urgency forces the owner to have some idea of what they want before the golem dies. Owners who pay attention to their golems' Grimoires during life pull the right entries at death. Owners who ignore their golems get the automatic deposit and nothing more.

### Living exports

While a golem is alive, the owner can export selected Grimoire entries to the Library at any time. This is not automatic. The owner navigates to the golem's Mind window in the TUI, selects entries, and presses `E` to export.

```
TUI: select a golem -> Mind window -> select entries -> [E] Export to Library
```

Meta Hermes deduplicates against existing Library entries before writing. If an entry has cosine similarity > 0.95 with something already in the Library, it skips. If similarity is 0.90-0.95, Meta Hermes asks the owner whether to merge or keep both.

Living exports are useful when a golem discovers something the owner wants to preserve immediately, without waiting for death. A golem that identifies a novel MEV pattern at hour 12 of its life shouldn't have to die before that knowledge reaches the Library. But living exports carry lower provenance trust than death deposits, because the golem is still under survival pressure and its knowledge hasn't been through the final death-reflection compression.

### Marketplace purchases

When the owner buys a skill or knowledge bundle from the Bazaar (the Bardo marketplace), the purchased artifact deposits into the Library automatically on settlement. Marketplace entries carry `marketplace_purchase` provenance and enter at the post-pipeline confidence discount.

These are third-party knowledge, produced by someone else's golems, validated in someone else's context, priced and sold on the open market. Useful but less trusted than knowledge from the owner's own lineage. A purchased MEV defense skill might work perfectly or might be calibrated for market conditions the owner's golems never encounter.

### Clade sync

Knowledge from sibling golems flows through the clade's peer-to-peer synchronization protocol. When a sibling golem promotes an insight to the clade (confidence >= 0.6, validated 3+ times), it propagates to the owner's Library at a clade discount of 0.80x.

Clade knowledge sits between self-learned and marketplace in the trust hierarchy. It comes from the owner's own fleet, but from a different golem with different context. A vault-manager golem's insight about Morpho utilization rates is relevant but not necessarily correct for an LP-optimizer golem operating in a different part of the market.

---

## Storage architecture

### Local storage

The Library lives on the owner's machine. Not in the cloud. Not on a server. On the owner's filesystem, under their control.

```
~/.bardo/library/
├── index.db                    # SQLite: metadata, search, confidence, provenance
├── entries/                    # Grimoire entries (insights, heuristics, warnings, causal edges)
│   ├── {uuid}.json
│   └── ...
├── skills/                     # Hermes skills (agentskills.io format)
│   ├── gas-timing-v3.md
│   ├── lp-rebalance-adaptive.md
│   └── ...
├── playbooks/                  # PLAYBOOK.md snapshots from dead golems
│   ├── golem-alpha-gen3.md
│   ├── golem-beta-gen1.md
│   └── ...
├── testaments/                 # Full death testaments
│   ├── golem-alpha-gen3.json
│   └── ...
├── strategies/                 # STRATEGY.md templates (owner-authored + learned)
│   ├── vault-manager-v2.md
│   └── ...
├── bundles/                    # Pre-assembled loadouts
│   ├── vault-starter.bundle.json
│   ├── lp-optimizer-experienced.bundle.json
│   └── ...
└── backup/                     # Styx sync state
    ├── last_sync.json
    └── pending_uploads/
```

The `entries/` directory is the heart of it. Each file is a JSON document containing a single knowledge entry, cheap to read, cheap to back up, and inspectable with any text editor. No proprietary format. No binary blobs. JSON.

The `testaments/` directory preserves full death testaments, richer than the individual entries extracted from them. A testament includes the golem's final self-assessment, its "what confused me" and "what I never had time to test" sections (exploiting the Zeigarnik effect: unfinished business is remembered better than completed tasks), and a narrative synthesis that captures context the individual entries lose. Sometimes the owner wants to read a dead golem's last words in full, not only the extracted knowledge.

### SQLite index

The `index.db` file is the search and metadata layer. It tracks everything needed to find, filter, rank, and manage Library entries without opening individual JSON files.

```sql
CREATE TABLE entries (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,          -- 'insight', 'heuristic', 'warning', 'causal_edge',
                                     -- 'skill', 'playbook', 'testament', 'strategy'
    domain TEXT,                      -- 'lp-management', 'gas-optimization', 'mev-defense'
    content_preview TEXT,             -- First 200 chars for display
    confidence REAL,                  -- 0.0-1.0
    validated_count INTEGER DEFAULT 0,
    contradicted_count INTEGER DEFAULT 0,
    source_type TEXT NOT NULL,        -- 'death_testament', 'live_export', 'marketplace_purchase',
                                     -- 'clade_sync', 'manual_import', 'meta_hermes_curation'
    source_golem TEXT,                -- Golem ID that produced this
    source_generation INTEGER,        -- Generation number in lineage
    source_archetype TEXT,            -- 'vault-manager', 'lp-optimizer', 'sleepwalker'
    is_bloodstain BOOLEAN DEFAULT 0,  -- Death-sourced provenance
    created_at INTEGER NOT NULL,      -- Unix timestamp
    last_equipped_at INTEGER,         -- When last given to a golem
    times_equipped INTEGER DEFAULT 0, -- How many golems have used this
    times_validated INTEGER DEFAULT 0,-- How many golems confirmed this as useful
    tags TEXT,                        -- JSON array of user/Meta Hermes tags
    embedding BLOB,                   -- nomic-embed-text-v1.5, 768-dim
    file_path TEXT NOT NULL           -- Relative path to the content file
);

CREATE INDEX idx_category ON entries(category);
CREATE INDEX idx_domain ON entries(domain);
CREATE INDEX idx_confidence ON entries(confidence DESC);
CREATE INDEX idx_source_type ON entries(source_type);
CREATE INDEX idx_source_archetype ON entries(source_archetype);

CREATE TABLE equip_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id TEXT NOT NULL REFERENCES entries(id),
    golem_id TEXT NOT NULL,
    golem_archetype TEXT,
    equipped_at INTEGER NOT NULL,
    outcome TEXT,                     -- 'validated', 'contradicted', 'unused', 'pending'
    outcome_confidence REAL,          -- Golem's final confidence for this entry
    outcome_at INTEGER,
    UNIQUE(entry_id, golem_id)
);

CREATE TABLE bundles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    target_archetype TEXT,
    created_by TEXT,                  -- 'owner', 'meta_hermes', 'marketplace'
    entry_ids TEXT NOT NULL,          -- JSON array of entry IDs
    total_entries INTEGER,
    avg_confidence REAL,
    validation_rate REAL,             -- % of entries validated across all golems that used this
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    times_used INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1
);
```

The `equip_history` table is where the feedback loop lives. Every time an entry is equipped to a golem, the table records it. When that golem later validates, contradicts, or ignores the entry, the outcome is written back. Over time, this table builds a complete picture of which entries actually help and which are dead weight.

### Entry format

Individual Library entries use YAML frontmatter plus content, mirroring Grimoire entries but with Library-specific metadata:

```yaml
---
id: "e7a3c2f1-8b4d-4e2a-9f6c-1234567890ab"
category: warning
domain: mev-defense
confidence: 0.82
source_type: marketplace_purchase
source_golem: "golem-0xab12"
source_generation: 2
source_archetype: sleepwalker
is_bloodstain: false
created_at: 1741024800
times_equipped: 5
times_validated: 4
tags: ["mev", "base-l2", "sandwich", "large-swaps"]
---

Swaps exceeding $5,000 on Base L2 have a 23% probability of sandwich attack
during peak hours (14:00-18:00 UTC). Split into 2-3 sub-transactions below
$2,000 each. Monitor the mempool for 2 blocks before execution. If pending
sandwich bot transactions are detected in the same block window, delay by
3-5 blocks.

Validated by 4 golems across 3 generations. Originally discovered by
golem-0xab12 (sleepwalker archetype, gen 2) via sustained mempool monitoring
over 72 hours on Base mainnet, January 2026.
```

The content section is natural language. Always. No rigid schemas, no JSON blobs, no structured formats that would constrain what the entry can express. Following the Voyager pattern: store knowledge as text that can be injected directly into LLM prompts. The LLM can parse it. Humans can read it. Both matter.

### Encrypted Styx L0 backup

The Library optionally syncs to Styx L0 Vault as an encrypted backup. This is redundancy, not primary storage. The owner controls the Library locally; Styx provides recovery on a new machine or after hardware failure.

Encryption uses wallet-derived X25519 keys. Styx sees opaque blobs. It cannot read the Library contents, cannot index them, cannot sell them. The owner's keys, the owner's data.

Sync runs in the background, every 30 minutes or on significant changes (a death deposit, a bulk import). The sync is delta-based: only entries changed since the last sync get uploaded. A Library with 800 entries doesn't re-upload 800 entries every half hour. It uploads the 3 that changed.

Storage: `~/.bardo/library/` contains SQLite database (entry metadata, relationships, search index) + LanceDB (vector embeddings, 384-dimensional, all-MiniLM-L6-v2). Directory structure: `library.db` (SQLite), `embeddings/` (LanceDB tables). <!-- AUDIT: LanceDB + 384-dim embeddings match the golem's existing Grimoire stack; all-MiniLM-L6-v2 chosen for size/quality tradeoff -->

Recovery is the reverse: pull all encrypted deltas from Styx, decrypt with the owner's wallet, reconstruct the Library. A new machine goes from empty to fully restored in under a minute for a typical Library.

```
Library Status:
  Local entries: 847
  Styx backup: synced 12 min ago (847/847 entries)
  Last sync delta: 3 entries uploaded (1.2 KB)
  Recovery key: wallet 0x7f3a...b2c1 (derived X25519)
```

---

## Equipping new golems

This is the central mechanism. Everything else in the Library exists to serve this moment: when a new golem is born, it can start with inherited knowledge instead of starting from zero.

### The Summoning gains an equip phase

The Summoning is the golem creation flow. Between strategy selection and VM provisioning, a new step appears:

```
The Summoning:
    Step 1: Choose archetype (vault-manager, lp-optimizer, sleepwalker, etc.)
    Step 2: Configure strategy (STRATEGY.md)
    Step 3: Choose custody mode (Delegation / Embedded / LocalKey)
    Step 4: Choose compute (Bardo Compute / self-deploy / local)
    Step 5: EQUIP FROM LIBRARY              <-- NEW
    Step 6: Provision VM
    Step 7: First heartbeat (equip package ingested during session start)
```

Step 5 is where Meta Hermes recommends a loadout. The owner reviews, customizes, or accepts defaults. Selected entries are bundled into an equip package that gets ingested during the golem's first session start.

Ingestion score = 0.4 * source_trust + 0.3 * semantic_novelty + 0.2 * validation_count + 0.1 * recency. Range: 0.0-1.0. Entries below 0.3 are quarantined for manual review. <!-- AUDIT: ingestion score formula invented; weights reflect trust-first ranking with novelty as secondary signal -->

### Meta Hermes recommends loadouts

Meta Hermes analyzes the new golem's archetype and strategy, searches the Library, and produces a ranked recommendation grouped into three tiers:

**Essential** entries are always included. Warnings are here, because safety knowledge is non-negotiable. So are battle-tested entries (validated by 3+ golems at confidence >= 0.6) and archetype-matched entries with high validation rates. Meta Hermes does not ask the owner's permission for essential entries. They go in.

**Recommended** entries are included by default but the owner can remove them. These are archetype-relevant insights, high-performing skills, and causal edges that are semantically similar to the new strategy. Good bets but not mandatory.

**Optional** entries are not included by default. The owner can add them. These are entries with lower confidence, entries from different archetypes that might transfer, or newer entries that haven't been validated enough yet.

The recommendation is based on four signals:

1. **Archetype match.** Entries from previous golems of the same type. A new vault-manager gets entries from dead vault-managers.
2. **Strategy relevance.** Semantic similarity between the entry's embedding and the new STRATEGY.md. An entry about Morpho utilization rates is more relevant to a vault-manager strategy focused on Morpho than one focused on LP optimization.
3. **Validation history.** Entries validated by multiple golems across multiple generations carry more weight than entries from a single golem's single lifetime.
4. **Recency.** An entry validated last week is more relevant than one validated six months ago, because DeFi market conditions change.

### Confidence discounting: the Weismann barrier

Equipped entries do not enter the new golem's Grimoire at full confidence. They enter at a discount. This is the Weismann barrier: an architectural firewall between inherited knowledge and earned knowledge.

Biology maintains this barrier rigorously. The Weismann barrier separates somatic cells (the body's lived experience) from germ cells (what gets passed to offspring). Two waves of epigenetic reprogramming strip away most acquired marks during reproduction. When transgenerational epigenetic inheritance does occur, it fades within 2-3 generations and the effects are more often harmful than adaptive (Heard & Martienssen, 2014).

The Library's confidence discounting implements the same principle:

| Source | Discount multiplier | Rationale |
|---|---|---|
| Death testament | 0.70x | Most trustworthy. Curated by death, compressed through reflection, produced under zero survival pressure. But still context-specific to the dead golem's environment. |
| Live export | 0.60x | Good quality, but the golem was still alive, still under survival pressure, and the entry hasn't been through final death compression. |
| Clade sync | 0.50x | From a sibling golem. Different operational context, different market conditions, different strategy parameters. |
| Marketplace purchase | 0.50x | Third-party knowledge. Unknown operational context, unknown validation rigor, unknown calibration. |
| Manual import | 0.50x | Unvalidated. Could be anything. The owner imported a file. |

A death testament entry with confidence 0.85 enters the new golem's Grimoire at 0.85 * 0.70 = 0.595. A marketplace purchase at confidence 0.79 enters at 0.79 * 0.50 = 0.395. The golem must validate these entries through its own operational experience before they earn full trust. The 0.70x Weismann barrier is configurable per-clade via `[library.inheritance]` in golem.toml. Range: 0.50-0.90. Default: 0.70. Lower values = more skeptical of inherited knowledge. <!-- AUDIT: configurability range invented; 0.50-0.90 prevents both total rejection and blind trust -->

Why discounting matters: without it, confidence would ratchet upward across generations without any check against reality. A golem could inherit an entry at 0.90, die, deposit it at 0.90, and the next golem would inherit it at 0.90 again. The entry would be treated as near-certain knowledge even if the market conditions that made it true changed months ago. Confidence discounting forces each generation to re-earn the trust of inherited knowledge. If the knowledge is still valid, the golem's own experience will push its confidence back up. If it's stale, the confidence stays low and the entry fades below the retrieval threshold.

This is the testing effect (Roediger & Karpicke, 2006) applied across generations: retrieval and validation strengthen memory traces, but only retrieval plus positive outcome counts. The golem doesn't get credit for passively holding inherited knowledge. It gets credit for using it and finding that it works.

### Generational confidence decay

Beyond the initial equip discount, an additional generational decay of 0.85x per generation prevents ancestor worship. An entry originally learned by generation 0, inherited by generation 1 at 0.70x, then inherited by generation 2 at 0.70x * 0.85x = 0.595x. By generation 5, an unvalidated inherited entry sits at about 44% of its original confidence. Below the noise floor for most retrieval queries. If it's genuinely structural knowledge (protocol mechanics, fee tier structures), the golem will re-validate it within a few ticks and confidence will climb back. If it's tactical knowledge from a market regime that no longer exists, it disappears.

The system is self-correcting. Good knowledge survives. Bad knowledge decays. But nothing persists on reputation alone. Every generation has to do the work.

### The equip screen

During The Summoning, the equip phase renders as a TUI screen:

```
+-- Equip: golem-delta (vault-manager) ----------------------------------------+
|                                                                               |
| Meta Hermes recommends 34 entries for this vault-manager golem.               |
|                                                                               |
| +-- ESSENTIAL (12 entries) ------------------------------------------------+  |
| | V  Stale oracle detection             warn   0.91  bloodstain   5x valid |  |
| | V  Liquidation cascade pattern        warn   0.87  bloodstain   3x valid |  |
| | V  MEV sandwich on large swaps        warn   0.82  purchased    5x valid |  |
| | V  Morpho util -> borrow rate         causal 0.85  bloodstain   7x valid |  |
| | V  Gas spike -> defer execution       causal 0.79  gen-2 death  4x valid |  |
| | V  LP Rebalance (gas-aware)           skill  0.81  evolved x3   3x valid |  |
| | V  vault-manager-v2 playbook          play   0.88  gen 3 death  --       |  |
| |    ... +5 more                                                           |  |
| +--------------------------------------------------------------------------+  |
|                                                                               |
| +-- RECOMMENDED (15 entries) ----------------------------------------------+  |
| | V  Morpho utilization leading          insight 0.74  2x valid            |  |
| | V  Optimal rebalance frequency         insight 0.71  4x valid            |  |
| | V  Gas timing skill                    skill   0.68  evolved x2          |  |
| | o  V4 hook fee extraction              insight 0.62  1x valid            |  |
| |    ... +11 more                                                          |  |
| +--------------------------------------------------------------------------+  |
|                                                                               |
| +-- OPTIONAL (7 entries) --------------------------------------------------+  |
| | o  Weekend gas patterns                insight 0.55  unvalidated         |  |
| | o  ETH vol -> IL correlation           causal  0.51  1x valid            |  |
| |    ... +5 more                                                           |  |
| +--------------------------------------------------------------------------+  |
|                                                                               |
| Summary: 27/34 selected | 12 essential | 10 recommended | 5 optional         |
| Estimated ingest time: ~54 seconds                                            |
| Base PLAYBOOK: vault-manager-v2 (from golem-alpha gen 3)                      |
|                                                                               |
| [Enter] Accept and create golem                                               |
| [Space] Toggle  [/] Search library  [A] Add more  [S] Save as bundle         |
+-------------------------------------------------------------------------------+
```

The owner can toggle individual entries, search the full Library for entries Meta Hermes didn't recommend, or save the current selection as a reusable bundle. Pressing Enter creates the golem with the selected loadout.

---

## Bundles: reusable loadouts

A bundle is a named collection of Library entries packaged for a specific purpose. Think of it as a preset equipment loadout in an RPG: "vault-manager-starter" contains the 20 entries a new vault-manager golem needs; "eth-momentum-aggressive" contains the entries that support a specific trading style.

### Anatomy of a bundle

```json
{
  "id": "bundle-vault-proven-v3",
  "name": "vault-manager-proven-v3",
  "description": "25 entries with 78% validation rate across 3 vault-manager golems",
  "target_archetype": "vault-manager",
  "entry_ids": ["e7a3c2f1-...", "b2d4e6f8-...", "..."],
  "playbook_id": "playbook-vault-manager-v2",
  "skill_ids": ["gas-timing-v3", "lp-rebalance-adaptive"],
  "created_by": "meta_hermes",
  "times_used": 3,
  "avg_confidence": 0.76,
  "validation_rate": 0.78,
  "version": 3
}
```

### Auto-generated bundles

Meta Hermes creates bundles when it detects patterns. If the last three vault-manager golems were all equipped with roughly the same 25 entries, and those entries had a 78% validation rate across all three, Meta Hermes notices. It creates a bundle called "vault-manager-proven-v3" containing the intersection of what all three used, minus the entries all three ignored, plus any promising new entries from the most recent death testament.

The notification is minimal:

```
Meta Hermes: I've created a bundle "vault-manager-proven-v3" based on
the last 3 vault-managers. 25 entries, 78% validation rate. Removed 4
unused entries from v2, added 2 from golem-gamma's death testament.
```

Meta Hermes does not ask permission to create bundles. It asks permission to delete entries or make destructive changes. Bundle creation is additive and cheap.

### Bundle versioning

Bundles track versions. When the underlying entries change (an entry's confidence shifts significantly, a new entry is added, an old entry is removed), the bundle version increments. The owner can see the diff between bundle versions:

```
vault-manager-proven-v3 -> v4:
  + Added: "Morpho borrow rate -> reallocation trigger" (0.73 conf, 3x valid)
  + Added: "Base L2 sequencer downtime -> pause execution" (0.81 conf, bloodstain)
  - Removed: "ETH weekend momentum pattern" (contradicted by 2/3 golems)
  ~ Updated: "Stale oracle detection" confidence 0.91 -> 0.93 (validated by golem-epsilon)
```

### Shareable bundles

Bundles can be exported and listed on the Bazaar. A proven bundle with high validation rates across multiple golem generations has real value. The listing includes validation rate, times used, archetype target, and a preview of entry types (without revealing full content until purchase).

Typical bundle prices: $1-10 for a proven 20-30 entry bundle. The economics are straightforward. If equipping a bundle saves a new golem two days of learning-from-scratch, and the golem earns $5/day, the bundle pays for itself fast.

---

## Meta Hermes as Librarian

Meta Hermes is the owner's personal assistant for managing their clade of golems. It lives in the TUI process, not in any golem. It has no heartbeat, no wallet, no mortality clock. It is not mortal. It is the librarian.

### Background curation

Meta Hermes runs periodic curation on the Library during idle time (when the owner isn't actively interacting with the TUI). Curation has four operations:

**Deduplication.** Find entries with cosine similarity > 0.92. Keep the one with higher confidence and more validations. Merge the other's metadata (validation counts, equip history) into the survivor. Delete the duplicate. A Library that receives entries from multiple golems' death testaments accumulates near-duplicates. Dedup keeps it clean.

**Staleness marking.** Flag entries that haven't been equipped or validated in 90 days. Stale entries aren't deleted, but they're tagged and deprioritized in recommendations. If an entry is stale and has never been validated by any golem, it's a candidate for archival.

**Promotion.** Entries validated by 5+ golems get a confidence boost (10%, capped at 0.95) and a "battle-tested" tag. These are the entries that have survived multiple golem lifetimes and consistently proven useful. They float to the top of recommendations.

**Bundle refresh.** Auto-generated bundles get updated with new entries from recent deaths, entries that proved unuseful are removed, and the validation rate is recalculated.

Curation runs hourly. Each run takes seconds. The owner doesn't see it happening unless they check the curation log.

### Natural language queries

The owner can ask Meta Hermes about the Library in plain language. Meta Hermes searches, aggregates, and responds:

| Owner asks | Meta Hermes does |
|---|---|
| "What do my dead golems know about ETH momentum?" | Semantic search across entries tagged with ETH and momentum-related domains. Returns a summary with confidence levels and which golems contributed. |
| "Which warnings have been validated the most?" | Queries warnings sorted by validation count. Returns top 10 with validation histories. |
| "What did golem-alpha learn before it died?" | Fetches golem-alpha's death testament and extracted entries. Presents a narrative summary. |
| "Create a bundle for a new sleepwalker" | Generates a loadout recommendation for sleepwalker archetype. Groups into essential/recommended/optional. |
| "How has my Library grown this month?" | Growth metrics: entries added (by source), domains covered, average confidence, validation rates. |
| "Delete everything from golem-beta" | Confirms first, then removes all entries sourced from golem-beta. Updates equip histories and bundle references. |

### Conflict resolution

When multiple entries contradict each other, Meta Hermes flags them for owner review rather than resolving unilaterally. An insight saying "rebalance hourly in high-vol regimes" from golem-alpha and an insight saying "rebalance daily in high-vol regimes" from golem-beta is a genuine disagreement. Meta Hermes presents both with their provenance, confidence, and validation history:

```
CONFLICT DETECTED:
  Entry A: "Rebalance hourly during high-vol regimes" (insight, 0.71)
    Source: golem-alpha gen-2, death testament, validated by 2 golems
  Entry B: "Rebalance daily during high-vol regimes" (insight, 0.68)
    Source: golem-beta gen-1, death testament, validated by 1 golem

  Context: golem-alpha operated Jan-Feb (ETH vol 65%), golem-beta
  operated Mar-Apr (ETH vol 42%). The "high-vol" threshold may differ.

  [A] Keep Entry A  [B] Keep Entry B  [K] Keep both  [M] Merge  [D] Defer
```

The owner decides. Or the owner says "keep both and let the next golem figure it out," which is a valid strategy. The Weismann barrier and confidence discounting mean the next golem will receive both entries at reduced confidence, test them against current conditions, and converge on whichever works. Contradictory entries are not a bug. They're a form of hypothesis generation. Equip the contradiction and let operational experience resolve it.

Meta Hermes also detects a subtler class of conflicts: entries that aren't directly contradictory but create incoherent strategy when combined. An entry recommending aggressive LP range widths and another recommending conservative gas budgets may each be sound in isolation but create a golem that tries to take wide positions without the execution budget to manage them. Meta Hermes surfaces these tensions during the equip recommendation phase, not as errors but as design decisions the owner should make consciously.

### Recommendation engine

Meta Hermes tracks which entries correlate with good golem outcomes and which correlate with poor ones. This isn't a sophisticated ML model. It's a simple feedback loop: entries that are consistently validated by golems that perform well (positive P&L, long survival) get higher recommendation priority. Entries that are consistently unused or contradicted by golems that perform well get deprioritized.

Over time, Meta Hermes develops a sense of what works. Not through reasoning about market dynamics, but through statistical observation of which knowledge leads to good outcomes. The recommendation engine gets better with each golem lifetime, because each lifetime generates more equip-outcome data.

---

## Generational compounding: the ratchet

The Library's value emerges over time. Not in a single golem lifetime, but across generations. Here is what that looks like.

### Month 1: starting from nothing

The owner creates their first golem. A vault-manager, deployed with $50 of USDC on Base. The Library is empty. There is nothing to equip.

The golem starts from scratch. Every observation is new. Every market signal is encountered for the first time. It watches Morpho utilization rates and after four days its Curator distills the pattern: utilization spikes on Mondays, reliably, because weekend liquidity is thinner and Monday borrowing demand catches up. It discovers that gas on Base drops below 0.005 gwei between 2-4 AM UTC, and builds a heuristic around it: "defer non-urgent transactions to the 2-4 AM window." It gets sandwich-attacked on a $7,000 swap during peak hours and records the warning with high confidence.

None of this is novel. Any human DeFi researcher could tell you these things. The difference is the golem learned them through operational experience, validated them against real outcomes, and encoded them with precise confidence scores. The "Morpho spikes on Mondays" insight has a confidence of 0.72 after 18 observations. The sandwich warning has confidence 0.89 after a single painful experience. The gas timing heuristic sits at 0.68, still accumulating evidence.

After 24 days, the golem dies. Credit exhaustion. Its $50 ran out. The Thanatopsis protocol fires. Phase II (Reflect) runs an Opus-grade reflection under zero survival pressure. The golem examines its lifetime with complete honesty: what worked (the gas timing heuristic saved approximately $4.20 in aggregate gas fees), what failed (it rebalanced too frequently in week 2 and ate its budget in swap fees), what it suspects but couldn't prove (there may be a correlation between ETH volatility spikes and Morpho utilization drops, but 24 days wasn't enough data).

The death testament deposits 47 entries into the Library.

47 entries. That's the entire legacy of a 24-day life. Compressed from thousands of episodes and hundreds of Grimoire entries into the distilled knowledge that passed the confidence thresholds: insights >= 0.6, all warnings regardless of confidence, causal edges with evidence count >= 3, skills used at least once.

The Library:
- 47 entries total
- 14 insights, 8 heuristics, 11 warnings, 9 causal edges, 5 strategy fragments
- Average confidence: 0.67
- Zero validation history (no second golem has tested any of these yet)
- Zero bundles
- The golem's PLAYBOOK.md, frozen at death, stored in `playbooks/`

### Month 2: the first successor

The owner creates a second vault-manager. This time, Meta Hermes has something to work with.

Meta Hermes analyzes the new golem's strategy and the Library contents. It recommends 30 entries (at 0.70x confidence discount, because they're all from a death testament): 11 warnings (all of them, always), 8 causal edges, 7 high-confidence insights, and 4 skills. It also recommends the dead golem's PLAYBOOK.md as the starting template.

The equip screen shows the recommendation. The owner looks at it, sees the dead golem's hard-won knowledge neatly categorized, and accepts the defaults. The golem provisions with the equip package.

The second golem starts ahead. Not dramatically ahead. The entries are at 0.70x confidence and need validation. But it doesn't have to rediscover that Morpho utilization spikes on Mondays. It already knows. That knowledge costs a few ticks to re-validate instead of days to discover from scratch. When Monday comes and utilization spikes exactly as predicted, the inherited insight's confidence ticks upward. Validation is cheap. Discovery was expensive.

The real advantage shows in the warnings. The first golem got sandwich-attacked on a $7,000 swap. The second golem inherits that warning and splits large swaps into sub-$2,000 chunks from day one. It never gets sandwich-attacked. The $7,000 loss that the first golem suffered bought immunity for every successor.

Over its lifetime, the second golem validates 12 inherited entries. Their confidence rises back toward the original levels. It contradicts 3 entries. The first golem was wrong about weekend gas patterns; conditions changed between month 1 and month 2 (a Base network upgrade altered the fee market). Those 3 entries get flagged in the Library with contradiction markers. The second golem creates 38 new entries from its own experience, including several that refine inherited insights with more precision: the Morpho Monday spike isn't random; it correlates with Sunday night borrowing volume on Aave, which acts as a leading indicator.

After 30 days, the second golem dies. Different cause: epistemic drift. Its knowledge of gas patterns went stale faster than it could learn, and it started making suboptimal execution decisions. The death testament deposits.

The Library now has:
- 82 entries (47 original - 3 contradicted + 38 new)
- 12 entries validated by 2 golems
- Average confidence: 0.71 (up from 0.67, because validated entries got boosted)
- Still zero bundles (Meta Hermes hasn't detected a repeating pattern yet)

### Month 3: pattern recognition

A third vault-manager launches. Meta Hermes recommends a curated loadout of 45 entries. Two generations of knowledge, ranked by validation history. The recommendation is better than last month because Meta Hermes now has equip-outcome data from the second golem. It knows which of the first golem's entries the second golem validated and which it contradicted. The contradicted weekend gas entries are excluded from the recommendation. The twice-validated Morpho insights are promoted to Essential tier.

The third golem outperforms its predecessors in the first week. It doesn't waste ticks discovering basic market dynamics. It starts with a functional causal graph containing 9 edges (inherited) and a PLAYBOOK.md that already contains two generations of evolved heuristics. While the first golem spent week 1 building its initial observations, the third golem spends week 1 running those observations against current conditions and refining them. It validates 28 entries in its first 10 days, turning inherited-at-discount knowledge into self-validated knowledge at full confidence.

The third golem creates 25 new entries. 15 are refinements of existing Library entries: adding more precision, covering edge cases, or documenting boundary conditions. "Morpho Monday spike" becomes "Morpho Monday spike is 2.3x larger when preceding Sunday Aave borrows exceed $50M." 10 entries are genuinely new discoveries: the third golem found a Uniswap V4 hook pattern that neither predecessor encountered, where a specific afterSwap hook was extracting fees in a way that affected LP returns.

When the third golem dies, Meta Hermes notices the pattern: the last two vault-managers were equipped with roughly the same 30-entry subset, and both validated 60%+ of those entries. It auto-generates the first bundle: "vault-manager-starter-v1", 28 entries, 72% validation rate. The notification is brief: "Created bundle vault-manager-starter-v1. 28 entries from 3 dead vault-managers. 72% validation rate."

Bundle auto-detection: Meta Hermes identifies repeating patterns when 3+ golems in the same clade generate semantically similar entries (cosine similarity > 0.85) within a 7-day window. Proposed bundles require validation from 2+ distinct golems before publication. <!-- AUDIT: bundle detection thresholds invented; 0.85 cosine and 3-golem minimum are conservative to avoid noisy bundles -->

The Library now has:
- 107 entries
- 15 entries validated by 3 golems (tagged "battle-tested")
- 1 auto-generated bundle
- Average confidence: 0.74

### Month 6: critical mass

The owner runs two golems simultaneously: a vault-manager and a sleepwalker (market monitoring archetype). Both feed the Library when they die or export knowledge. The Library grows in two domains simultaneously. The vault-manager contributes lending and liquidity knowledge. The sleepwalker contributes market structure observations, MEV patterns, and regime-shift detection.

Something interesting happens: the sleepwalker discovers a pattern about ETH price momentum that the vault-managers can use. When ETH 1-hour momentum exceeds 2 standard deviations, Morpho utilization drops sharply within 90 minutes as leveraged positions get unwound. This is cross-archetype knowledge. Meta Hermes flags it and adds it to the vault-manager bundle even though it originated from a sleepwalker.

The Library has 300+ entries across 8 domains. Meta Hermes has auto-generated 5 bundles: vault-manager-starter, vault-manager-experienced (for golems that will run aggressive strategies), sleepwalker-base, eth-momentum (cross-archetype), and mev-defense-base-l2. Three marketplace purchases added specialized knowledge the owner's golems hadn't discovered: a detailed analysis of Uniswap V4 hook gas overhead from a seller who ran 50 golems across different hook configurations.

New golems start with 30-50 relevant entries. Validation rate across the Library: 72%. The contradicted entries have been flagged and deprioritized. Stale entries from month 1 that nobody has re-validated are marked but not deleted. The Library is not just bigger. It's sharper.

Golem lifetime improves measurably. Month-1 golems died after 24 days. Month-6 golems average 35 days. They make fewer rookie mistakes because warnings from dead predecessors protect them. They execute more efficiently because inherited gas timing heuristics save 15-20% on execution costs. They detect regime shifts faster because inherited causal models give them a head start on pattern recognition.

The first golem lost $120. The month-6 vault-manager earned $45 in its first 30 days. The Library was the difference.

### Month 12: the Library is the asset

The Library has 800+ entries. 5 auto-generated bundles. The owner has sold two proven bundles on the Bazaar for $5 each: "Vault Manager Best Practices" (45 entries, 82% validation rate across 6 golems) and "MEV Defense for Base L2" (18 entries, 88% validation rate).

A new vault-manager created today starts with a causal graph containing 35 edges that took six predecessors to build. It inherits a PLAYBOOK.md refined across five generations. It carries warnings about failure modes that no living golem has encountered, documented by dead golems with their last breath. It knows about the January liquidity crisis because a golem that died in January left a bloodstain-marked warning. It knows about the V4 hook fee extraction pattern because a sleepwalker discovered it in month 4 and three vault-managers validated it across months 5-8.

The owner's competitive advantage is not any individual golem. It's the Library. A golem is disposable. It costs $50 to deploy, runs for a month, and dies. The Library persists. It compounds. It improves with every death. A new entrant to Bardo starting from an empty Library is 12 months behind. They have to pay the same tuition the owner's first golems paid: discover from scratch what the Library already knows, lose money on mistakes the Library already warns about, build causal models that the Library already contains.

The golem is mortal. The Library is not. And the Library is where the value lives.

Borges's Library of Babel contained every possible book but was useless because nobody could find the true books amid the noise. This Library contains only true books, because every entry was learned through experience, validated through operation, and curated through death. The librarians don't wander in despair. They know exactly where the knowledge is.

### The lineage view

The TUI can render a lineage view showing how knowledge compounded across generations:

```
Gen 1: Primordial-0001  |  -$120  |  24d  |  credit exhaustion
  +-- genomic bottleneck: 80 episodes -> 47 Library entries
Gen 2: Spark-0002       |  -$45   |  30d  |  epistemic drift
  +-- inherited 30, validated 12, contradicted 3, learned 38 new
Gen 3: Ember-0003       |  -$12   |  35d  |  credit exhaustion
  +-- inherited 45, validated 28, learned 25 new (15 refinements)
Gen 4: Phoenix-0004     |  +$312  |  45d  |  owner kill (strategy pivot)
  +-- inherited 55, validated 41, learned 35 new -- PROFITABLE
Gen 5: Radiant-0005     |  +$780  |  67d  |  ALIVE
  +-- inherited 76, validated 62, learning... -- THRIVING
```

The pattern: each generation starts with more knowledge, dies smarter, and leaves a richer Library for its successor. Gen 1 lost $120 and left 47 entries. Gen 5 is alive, profitable, and building on the knowledge of everything that died before it.

That $120 loss from Gen 1 was not wasted. It bought 47 entries that every subsequent generation built on. The cost of ignorance was paid once. The knowledge compounds forever.

---

## Equip outcome tracking

Equipping entries is only the first half. The second half is tracking what happened.

### The feedback loop

After a golem runs for a while, Meta Hermes periodically checks which equipped entries were actually useful. It queries the golem's Grimoire for entries that originated from the Library (`provenance: library_equip`) and examines their current state:

- **Validated**: The golem's experience confirmed the entry's value. Its confidence in the golem's Grimoire has risen above the initial equipped confidence by 20% or more. The entry helped.
- **Contradicted**: The golem's experience contradicted the entry. Confidence dropped below half the initial value. The entry was wrong, or conditions changed.
- **Unused**: The entry was never retrieved during decision-making. It sat in the Grimoire without ever being relevant to what the golem was doing. Dead weight.
- **Pending**: Not enough data yet. The golem hasn't been alive long enough to validate or contradict.

These outcomes feed back into the Library:

- **Validated entries** get a confidence boost in the Library index. An entry validated by 5+ golems is tagged "battle-tested" and gets an additional 10% confidence boost (capped at 0.95). Meta Hermes prioritizes it in future recommendations.
- **Contradicted entries** get flagged. If contradicted by 2+ golems, Meta Hermes notifies the owner and deprioritizes the entry. Contradicted entries aren't deleted automatically (the contradiction might be wrong), but they stop appearing in default recommendations.
- **Unused entries** get deprioritized over time. An entry that three consecutive golems ignored probably isn't relevant for the archetype it's being recommended to.

### What this data enables

Over multiple golem lifetimes, the equip-outcome data answers questions the owner couldn't answer otherwise:

- "Which warnings actually prevented losses?" Look at warnings that were validated: retrieved by the golem when it encountered the warned-about scenario and successfully avoided the loss. The sandwich attack warning from month 1 has been validated by 5 golems. None of them lost money to sandwich attacks. That warning has a measurable ROI.
- "Which entries are we recommending but nobody uses?" Look at consistently unused entries. Three consecutive golems ignored "Weekend gas patterns" because their strategies don't involve weekend execution. Either the entry is irrelevant for the vault-manager archetype, or Meta Hermes is recommending it to the wrong audience. Either way, stop recommending it.
- "Are marketplace purchases worth the money?" Compare validation rates of purchased entries vs. death-testament entries. If purchased entries are validated at 40% and death-testament entries at 75%, the marketplace purchases are lower quality. Maybe the seller's golems operated in conditions too different from the owner's context.
- "Is the Library actually helping?" Compare P&L of golems equipped with 50+ entries vs. golems equipped with fewer than 20 vs. golems equipped with nothing. If there's no correlation between equipment size and performance, the entries might be noise rather than signal. If heavily equipped golems consistently outperform lightly equipped ones, the Library is working.
- "Which domains produce the highest-value entries?" Cross-reference entry domains with validation rates. If mev-defense entries have 88% validation and gas-optimization entries have 45%, the owner should invest more golem time in MEV-focused strategies and be skeptical of gas-timing knowledge.

This is where the Library goes from "a collection of knowledge" to "a system that learns which knowledge matters." The entries don't just accumulate. They compete for recommendation slots, for equip positions, for the limited cognitive budget of each new golem. The good ones rise. The bad ones fade. And the feedback loop tightens with every generation.

The equip-outcome table grows linearly with (entries x golems). For a Library with 800 entries and 20 lifetime golems, that's 16,000 rows. Small by any database standard. But the statistical power of 16,000 data points about which knowledge actually works is considerable. By month 12, the recommendation engine has enough signal to be genuinely useful rather than speculative.

---

## Cross-golem prediction accuracy tracking

The Library persists across golem generations. So does prediction accuracy history.

When a golem dies, its Grimoire entries enter the Library with their context attribution scores from Loop 6 ([21-evaluation-architecture.md](21-evaluation-architecture.md)). Each entry now carries a `prediction_accuracy_delta`: how much the entry's presence in retrieval context improved or degraded prediction accuracy.

```rust
pub struct LibraryEntry {
    // ... existing fields ...

    /// How much this entry improved prediction accuracy when cited
    /// Positive = helpful, negative = misleading
    pub prediction_accuracy_delta: f32,

    /// Loop 6 context attribution score (accumulated over lifetime)
    pub context_attribution_score: f32,

    /// How many golem generations have validated this entry
    pub generations_validated: u32,
}
```

When a new golem is summoned and equipped from the Library, entries with high `context_attribution_score` are preferred. The Summoning screen shows attribution scores alongside each entry, so the owner can see which pieces of inherited knowledge actually helped previous golems predict accurately.

Entries that persist across 3+ generations with consistently positive attribution scores are candidates for promotion to heuristics. Entries that show negative attribution, knowledge that consistently makes predictions worse, get flagged for review.

---

## Active learning via equip/validate loops

The Library is not a static archive. It participates in a validation loop:

1. **Equip**: Owner selects Library entries for a new golem during the Summoning.
2. **Observe**: The golem uses those entries in its Grimoire, and Loop 6 tracks attribution.
3. **Report**: When the golem dies (or at weekly review), attribution data flows back to the Library.
4. **Update**: Library entries update their `prediction_accuracy_delta` and `generations_validated`.
5. **Surface**: The next Summoning shows updated attribution scores.

Over generations, the Library self-curates. Entries that help golems predict well rise to the top. Entries that don't sink. The owner doesn't need to manually curate; the evaluation system does it through accumulated attribution data.

This is active learning in the machine learning sense: the system selects which knowledge to test (by equipping it), observes the outcome (via attribution), and updates its beliefs (via Library entry scores). Each golem generation is an experiment that improves the Library for the next one.

---

## Retrospective insights

Add a seventh entry category alongside Episodes, Insights, Heuristics, Warnings, Causal Links, and Spectral Traces:

**Retrospective insights** are generated by Loop 12 ([21-evaluation-architecture.md](21-evaluation-architecture.md)) during daily and weekly reviews. They differ from regular Insights in that they're constructed from hindsight: the golem knows the outcome and reasons backward to extract the lesson.

```rust
pub enum EntryCategory {
    Episode,
    Insight,
    Heuristic,
    Warning,
    CausalLink,
    SpectralTrace,
    RetrospectiveInsight,  // NEW
}

pub struct RetrospectiveInsight {
    /// The position or trade being reviewed
    pub position_id: String,
    /// What happened (PnL, timing, market conditions)
    pub outcome: PnlAttribution,
    /// What the golem would do differently with perfect hindsight
    pub hindsight_action: String,
    /// The optimal exit point (identified via retrospective analysis)
    pub optimal_exit: Option<ExitPoint>,
    /// Regret score: how much better optimal would have been
    pub regret_score: f64,
    /// Hindsight predictions generated from this review
    pub hindsight_predictions: Vec<PredictionId>,
}
```

Retrospective insights carry a special provenance marker: they're explicitly tagged as hindsight reasoning. This prevents a common bias where hindsight knowledge gets treated as if the golem "should have known" at the time. The golem did not know. It knows now. The insight is about what to watch for next time, not about what went wrong.

These entries inherit across generations with the same 0.7x Weismann barrier as everything else. A successor golem gets the retrospective insight but at reduced confidence: the new golem must validate the lesson in its own experience before it influences action gating.

---

## The Library TUI window

The Library gets its own TUI window, accessible via Tab from any other window.

### Browse view

```
+-- Library of Babel -------------------------------------------------------+
| [All] [Insights] [Heuristics] [Warnings] [Causal] [Skills]               |
| [Playbooks] [Testaments] [Bundles]                                         |
|                                                                            |
| 847 entries | 23 skills | 7 playbooks | 12 testaments | 4 bundles         |
|                                                                            |
| Entry                          Cat      Conf  Source       Valid           |
| -------------------------------------------------------------------------- |
| + Stale oracle detection       warn     0.91  a-gen3 +    5x V            |
| + Morpho util -> borrow rate   causal   0.85  a-gen2 +    7x V            |
|   LP range width adaptive      skill    0.81  b-gen1      3x V            |
| $ Gas timing for Base L2       skill    0.79  purchased   4x V            |
| + Liquidation cascade pattern  warn     0.87  a-gen3 +    3x V            |
|   Weekend gas is 40% cheaper   insight  0.72  c-gen1      2x V            |
|   V4 hook fee extraction risk  insight  0.62  sleepwalker 1x V            |
|                                                                            |
| + = from a dead golem  $ = marketplace purchase                            |
|                                                                            |
| Health: 62% validated | 8% contradicted | 18% unused | 12% pending        |
| Styx backup: V Synced 12 min ago | 847/847 backed up                      |
|                                                                            |
| [/] Search  [E] Equip to golem  [B] Bundle  [I] Import  [S] Sort          |
+----------------------------------------------------------------------------+
```

The `+` marker is the tombstone symbol. It marks entries sourced from dead golems. A reminder: this knowledge cost a life to produce.

### Search

Full-text search via SQLite FTS5 and semantic search via embedding similarity. The owner types a query, results appear ranked by a combination of text relevance and embedding similarity. Searches can be filtered by category, domain, confidence range, source type, or date range.

```
Search: /morpho rebalance threshold
Results (7):
  1. Morpho util -> borrow rate           causal  0.85  7x valid
  2. Optimal rebalance frequency           insight 0.71  4x valid
  3. Morpho utilization spike timing       insight 0.68  2x valid
  4. Vault rebalance gas optimization      skill   0.65  3x valid
  ...
```

### Detail view

Selecting an entry opens a detail pane with full content, provenance chain, confidence history, and equip history:

```
+-- Detail: MEV sandwich on large swaps --------------------------------+
| Category: Warning                                                      |
| Domain: mev-defense                                                    |
| Confidence: 0.82                                                       |
| Source: Marketplace purchase from golem-0xab12 (Score: 94)             |
| Validated by: 5 golems across 3 generations                            |
| Bloodstain: No                                                         |
| Last equipped: golem-delta (2 days ago)                                |
| Outcome in golem-delta: Validated (confidence rose to 0.89)            |
|                                                                        |
| Content:                                                               |
| "Swaps exceeding $5,000 on Base L2 have a 23% probability of          |
|  sandwich attack during peak hours (14:00-18:00 UTC). Split into       |
|  2-3 sub-transactions below $2,000 each. Monitor the mempool for       |
|  2 blocks before execution."                                           |
|                                                                        |
| Confidence history:                                                    |
|  0.79 (purchased) -> 0.82 (validated x3) -> 0.85 (validated x5)       |
|                                                                        |
| Equip history:                                                         |
|  golem-alpha gen-2: validated (conf 0.79 -> 0.84)                      |
|  golem-beta gen-1:  validated (conf 0.82 -> 0.87)                      |
|  golem-gamma gen-1: unused                                             |
|  golem-delta gen-1: validated (conf 0.82 -> 0.89) -- current           |
|  golem-epsilon gen-3: pending                                          |
|                                                                        |
| [E] Equip  [D] Delete  [T] Tag  [B] Add to bundle                     |
+------------------------------------------------------------------------+
```

### Bundle management

A dedicated sub-screen for creating, editing, and managing bundles:

```
+-- Bundles ---------------------------------------------------------------+
| Name                           Archetype       Entries  Valid%  Used     |
| --------------------------------------------------------------------------|
| vault-manager-proven-v3        vault-manager    25      78%     3       |
| vault-manager-starter-v2       vault-manager    18      65%     5       |
| sleepwalker-base-v1            sleepwalker      20      71%     2       |
| eth-momentum-aggressive-v1     any              12      82%     4       |
|                                                                          |
| [N] New bundle  [E] Edit  [C] Clone  [X] Export to marketplace          |
+--------------------------------------------------------------------------+
```

### Library health metrics

The bottom section of the Library screen shows aggregate health:

```
+-- Library health --------------------------------------------------------+
| Total: 847  |  Avg confidence: 0.68  |  Domains: 12                     |
|                                                                          |
| Sources:         | Validation:        | Growth:                          |
|  Deaths:    312  |  Validated:  62%   |  This week:  +34                 |
|  Exports:   198  |  Contradicted: 8%  |  This month: +127                |
|  Purchases:  89  |  Unused: 18%       |  All time:   847                 |
|  Clade:     156  |  Pending: 12%      |                                  |
|  Imports:    42  |                    |                                  |
|  Curated:    50  |                    |                                  |
|                                                                          |
| Top domains: lp-management (234), gas-opt (156), mev (89),              |
|              risk (78), morpho (67), governance (45)                     |
+--------------------------------------------------------------------------+
```

The health metrics tell the owner whether the Library is growing, stagnating, or decaying. A validation rate above 60% is healthy. Below 40% suggests the entries are stale or the golems are operating in conditions too different from where the entries originated. An unused rate above 30% suggests Meta Hermes is recommending entries that aren't relevant.

---

## What the Library is not

Misconceptions are worth addressing directly.

**The Library is not a backup of every golem's full Grimoire.** Death testaments are compressed, typically 5KB from a golem that accumulated thousands of episodes and hundreds of Grimoire entries during its lifetime. The compression is the point. Raw Grimoire dumps would recreate the Borges problem: too much undifferentiated data, not enough signal.

**The Library is not automatically synced to every golem.** Knowledge flows FROM the Library TO a golem only at creation (equipping) or via explicit owner action. Running golems do not continuously pull from the Library. This is deliberate. A golem that continuously updates from external sources cannot develop its own understanding. It becomes a cargo-cult follower of inherited wisdom, echoing what the dead believed instead of learning what the living world actually is.

**The Library is not a replacement for the Grimoire.** The Grimoire is the golem's living knowledge system with real-time retrieval, decay, and curation. The Library is cold storage: curated, indexed, searchable, but not actively used in decision-making. A Grimoire entry with confidence 0.85 is hot knowledge, retrieved on every relevant tick. The same entry in the Library is cold, waiting for a new golem to inherit it.

**The Library is not hosted in the cloud.** Primary storage is local. The owner's machine, the owner's filesystem. Styx backup is optional and encrypted. The owner controls their knowledge.

---

## The ratchet as value proposition

Everything in this document points to one mechanism: the ratchet.

A ratchet is a device that allows motion in one direction and prevents it in the reverse direction. A wrench ratchets: you can tighten a bolt but not accidentally loosen it. The Library ratchets intelligence: each golem lifetime can add knowledge, validate knowledge, or discover that knowledge is wrong. It cannot lose validated knowledge. The ratchet only turns forward.

Not linearly. Some months the Library grows by 100 entries. Other months it grows by 20 but those 20 are better quality than the previous 100. Sometimes a golem contradicts entries and the Library shrinks in count but improves in accuracy. The direction is forward, but the path is uneven.

The ratchet changes what it means to operate in Bardo. Without the Library, each golem is disposable in the worst sense: when it dies, everything it learned dies with it. Every new golem starts from zero. The owner is on a treadmill, deploying golems that make the same discoveries and the same mistakes, never building on what came before.

With the Library, each golem is disposable in the best sense: it can die without permanent loss, because its knowledge survives. Death becomes productive. Every golem's lifetime is a research expedition that reports back to the Library. The golem is the explorer. The Library is the map. Each expedition fills in more of the map. Eventually, the map is worth more than any individual expedition.

For the first-time Bardo user, the Library is empty and the experience is identical to a system without knowledge persistence. For the user who has been running golems for six months, the Library is a competitive advantage that no new entrant can replicate without spending six months themselves. For the user who has been running golems for a year, the Library is the primary asset: not the $50 in any individual golem's wallet, but the 800 validated entries representing a year of accumulated intelligence.

This is the product. Not a golem. Not a DeFi agent. A machine for compounding intelligence across lifetimes, where death is not a loss but a deposit into the account that matters most.

---

## Grimoire-as-palace: the spatial visualization

The Library TUI described above is a list. Lists work for management: sort, filter, search, select. But a list doesn't communicate what the Library *is*: a structure of knowledge with density, connections, gaps, decay, and history. A list of 800 entries feels like a spreadsheet. A palace with 800 rooms feels like something the golem built with its lives.

This section specifies how the Grimoire renders as a navigable 2D space, an architectural visualization that turns knowledge management into exploration. The palace view is an alternative mode within the Library TUI (toggle via `P` key), not a replacement for the list. Both views operate on the same data. The palace just makes the topology visible.

> **Cross-references**: `15-design-system.md` (palette definitions, border vocabulary, particle system spec), `13-hypnagogic-engine.md` (how the palace appears during dream states, fragment surfacing from Grimoire entries), `11-cinematic-consciousness.md` (dream visuals and the Summoning equip screen). The engagement PRD's "Inner Worlds" section (`bardo-v4-11-inner-worlds.md`) defines the foundational spatial model: entries as rooms, causal links as corridors, confidence as structural integrity.

### Entries as rooms

Each Grimoire entry occupies a room in the palace. The room's shape identifies its type at a glance, even from a distance, even through fog. The shapes are chosen so they read in a 7x5 character cell, the minimum renderable unit.

```
Type        Shape       Rationale
Episode     [ ]         Rectangular. Solid border. A contained event.
Insight     < >         Angular, pointed. An observation with an edge.
Heuristic   ( )         Rounded. Soft edges. A rule of thumb, not a hard law.
Warning     [!]         Alert bracket. The exclamation mark is always visible.
Causal link ~ ~         Wavy. Connection-oriented. This room exists to relate others.
Dead-source [+]         Tombstone room. The + marker. This knowledge cost a life.
```

A room at minimum size shows the type glyph, a truncated name (12 chars), and a 3-character confidence bar. At expanded size (when the cursor enters), it shows full content, provenance, and all five detail tabs (Content, Confidence, Provenance, Links, Usage).

Here is what a single room looks like at default zoom, with high confidence and warm emotional provenance (learned during a successful trade):

```
   ┌─────────────────────┐
   │ <> Morpho Monday    │
   │    spike pattern     │
   │    ██████████░░ 0.85 │
   │    bloodstain + gen3 │
   │    validated 7x      │
   └─────────────────────┘
```

And the same room at low confidence (0.3), learned during uncertainty:

```
   ┊                     ┊
   ┊ <> Morpho Monday    ┊
   ┊    sp░ke p░ttern     ┊
   ┊    ███░░░░░░░░░ 0.30 ┊
   ┊    ░░░░dstain + gen3 ┊
   ┊                     ┊
```

The borders crumble. Characters decay into `░`. The room is structurally unsound, barely held together by whatever confidence remains. Approach it and the content might still be readable, but the architecture announces doubt before you get close enough to read.

### Confidence as structural integrity

This is the core visual metaphor. Confidence maps to wall solidity. The palace is not a neutral container. It is a reflection of what the golem trusts.

```
Confidence   Border style         Character decay   Visual description
1.0          ┌──┐ (double weight)  None              Bright, full box-drawing, all text legible.
0.8          ┌──┐ (standard)       None              Standard border, clear text.
0.6          ┊  ┊ (thin)           Light (5% chars)  Thin dotted border. Text slightly dimmed.
0.4          .  . (partial)        Moderate (20%)    Gaps in the border. Some chars -> ░.
0.2          ·    (fragments)      Heavy (50%)       Scattered dots where walls were. Text nearly gone.
0.0          (faint dot)           Total             Ghost position. A single · where knowledge used to be.
```

The decay is not random. Characters disappear from the edges inward, vowels before consonants, spaces before letters. The most semantically loaded fragments survive longest, as though the text itself fights to preserve its meaning. A room at confidence 0.3 with the content "Swaps exceeding $5,000 on Base L2 have a 23% probability of sandwich attack" might render as:

```
   ·                     ·
   · [!] S░aps >$5k ░ase ·
   ·     ░3% s░ndwich     ·
   ·     ░░░░░░░░░░░ 0.30 ·
   ·                      ·
```

The warning survives in fragments. Enough to trigger recognition. Not enough to act on without entering the room and reading the full entry.

### Emotional provenance as color temperature

Every room carries the emotional fingerprint of the moment the golem learned what it contains. The PAD (Pleasure-Arousal-Dominance) state at encoding time becomes the room's color temperature. The palette draws from the Bardo design system (`15-design-system.md`):

- **Curiosity** (positive valence, moderate arousal): cool cyan tint on borders and text. The golem was exploring, not under pressure. These rooms feel calm.
- **Alarm** (negative valence, high arousal): warm rose tint. The golem was reacting to danger. These rooms feel hot, even at a distance. Warnings often have this tint.
- **Confidence** (positive valence, high dominance): bright amber. The golem was winning. These rooms glow.
- **Grief** (negative valence, low arousal): desaturated grey. The golem was mourning a loss or watching value drain. These rooms feel cold, muted, drained of energy.

Age compounds the temperature shift. Fresh entries (created fewer than 50 ticks ago) render warm regardless of emotional provenance, a "recently touched" glow. Old entries (500+ ticks) cool toward ambient. Ancient entries (2000+ ticks) are nearly the same temperature as the void background. The palace's bright regions are where the golem has been thinking lately. Its cold regions are where it hasn't visited in a long time.

The result: scanning the palace from a distance, the owner sees a thermal map of the golem's cognitive activity. Hot spots of recent attention. Cold expanses of neglected knowledge. Warm clusters of confidence. Cool isolated rooms where something went wrong.

### Causal links as corridors

Rooms don't float in isolation. They connect through corridors, the visual rendering of causal links in the Grimoire's knowledge graph. A causal link ("Fed rate hike -> ETH sell pressure with 3-6 hour lag") becomes a passage between the two rooms it connects.

Corridor style maps to evidence strength:

```
Strong (evidence > 5):    ═══════════  Double-line. Bright. Well-trodden path.
Medium (evidence 2-5):    ────────────  Single-line. Standard.
Weak (evidence 1):        ┄┄┄┄┄┄┄┄┄┄┄  Dashed. Dim. The connection is tentative.
Contradictory:            ╳╳╳╳╳╳╳╳╳╳╳  Crossed. Rose-bright. These rooms disagree.
Dead-sourced:             +┄┄┄┄┄┄┄┄┄+  Dashed with daggers. Ghost corridors from dead golems.
```

Walking along a corridor (navigating from one room to a linked room) takes 300-500ms. During the transition, the key terms shared between the two entries stream through the corridor space, dissolving as you pass. You see what connects these ideas in the moment of traversal.

A cluster of rooms with strong corridors between them forms a neighborhood: a domain of knowledge where the golem has built dense causal understanding. LP management entries cluster together, connected by thick double-line corridors. MEV defense entries form another cluster. Between the clusters, thinner corridors represent cross-domain insights, the rarer connections where knowledge from one domain informed another.

### Fog of war

Not all rooms are visible. The palace uses a fog-of-war system borrowed from strategy games: entries the golem hasn't accessed recently (more than 200 ticks since last retrieval) fade into fog.

The fog is not binary. It grades:

```
Last accessed    Visibility
< 50 ticks       Full. Bright borders, clear text, all detail visible.
50-100 ticks     Slightly dimmed. Borders visible, text readable but muted.
100-200 ticks    Hazy. Borders visible as shapes, text reduced to blurred glyphs.
200-500 ticks    Deep fog. Room shape barely visible. A ghost outline.
> 500 ticks      Gone. Only a faint · on the minimap marks its position.
```

Moving the cursor toward a fogged room parts the fog gradually. At three rooms' distance, the outline sharpens. At two rooms, borders appear. At one room, text begins resolving. Entering the room clears all fog and renders full detail. This creates a natural exploration dynamic: the owner discovers what the golem has been thinking about (the bright regions are obvious) and can probe what it has been ignoring by pushing into the fog.

There is also a practical function. If the owner spots a fogged room that should be relevant to the golem's current strategy, they can flag it. Flagging promotes the entry in the Grimoire's retrieval scoring, making the golem more likely to access it on subsequent ticks. The owner can steer attention without overriding autonomy.

### Bloodstain markers

Entries inherited from dead golems carry a visual mark that never fades. The bloodstain is not decoration. It is provenance: this knowledge was paid for with a life.

Bloodstain treatment:

- **Dagger corners**: the room's corner characters are replaced with `+` (the tombstone marker from the list view). Where a normal room has `┌` and `┘`, a bloodstained room has `+` and `+`.
- **Ash particles**: a faint downward particle drift from the room's top border. Small `·` characters descend at 1-2 per second, fading before they reach the room below. Ashes falling from the dead.
- **Dashed borders**: where a normal room at the same confidence would have solid borders (`──`), a bloodstain room has dashed (`┄┄`). The structural integrity reflects not just confidence but origin. Inherited knowledge is always slightly less certain than self-learned knowledge, and the border shows it.
- **Ghost color underlayer**: the room's emotional tint is a blend. The primary tint is the current golem's emotional association with the entry. Beneath it, at 30% opacity, is the dead golem's terminal emotional state. If the dead golem died in grief, its bloodstained entries carry a grey undertone forever. If it died confident, an amber undertone. The dead golem's final mood haunts its knowledge.
- **Testament whisper**: approaching a bloodstained room (cursor within 2 rooms), there is a 30% chance per frame that a brief text fragment from the dead golem's testament surfaces in the corridor space. Fragments like "I learned this at tick 847, three hours before my credit ran out." or "This saved me twice." The dead speaking through the margin notes of their knowledge.

Here is what a bloodstained warning entry looks like at confidence 0.82:

```
   +┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄+
   ┊ [!] Sandwich attack  ┊·
   ┊     >$5k Base L2     ┊ ·
   ┊     ████████░░░ 0.82 ┊  ·
   ┊     +bloodstain gen3 ┊   ·
   ┊     validated 5x     ┊    ·
   +┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄+
      ·  ·
         ·
```

The `+` at corners. Dashed `┄` borders. The trailing `·` characters are ash particles, drifting down and right. Compare with the same entry if the golem had learned it directly (no bloodstain):

```
   ┌─────────────────────┐
   │ [!] Sandwich attack  │
   │     >$5k Base L2     │
   │     ████████░░░ 0.82 │
   │     self-learned      │
   │     validated 5x      │
   └─────────────────────┘
```

Solid corners. Solid borders. No ash. No ghosts. The difference is immediate, even at a glance across the palace.

### A failed-trade entry with full bloodstain treatment

This is what a failed trade looks like in the palace. The golem inherited a causal edge from a predecessor who lost $2,400 on a mistimed rebalance, then died of credit exhaustion three days later. Confidence has decayed to 0.45:

```
         ·
        ·  ·
   +┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄+
   ┊ ∴  ETH vol sp░ke    ┊·
   ┊    -> rebal░nce      ┊ ·
   ┊    LOSS: -$2,400     ┊  ·
   ┊    ████░░░░░░░░ 0.45 ┊
   ┊    +gen2 contra░icted┊
   +┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄ ┄+
      ·     ·
         ·
```

The gaps in the dashed border (`┄ ┄` with spaces). The character decay in "sp░ke" and "rebal░nce" and "contra░icted". The ash falling both above and below (the particles drift bidirectionally on rooms with negative outcomes, as though the room itself is disintegrating). The loss amount rendered in full, undecayed, because financial losses survive in memory longer than anything else. The causal type glyph `∴` marks this as a room that connects things rather than standing alone.

### Force-directed graph view

Pressing `g` in the palace toggles to an alternative visualization: a force-directed graph showing the entire Grimoire as a physics simulation. Same data, different topology. The palace emphasizes spatial relationships and navigation. The graph emphasizes structure and connectivity.

Each entry becomes a node. Each causal link becomes an edge. Spring physics govern the layout:

- **Attraction**: connected nodes pull toward each other. Stronger links (higher evidence count) produce stronger springs. A cluster of tightly linked entries collapses into a dense knot.
- **Repulsion**: all nodes repel each other at short range, preventing overlap. The repulsion is weak enough that dense clusters stay dense but strong enough that no two nodes occupy the same cell.
- **Gravity**: a weak central pull keeps the graph from drifting to infinity. The graph slowly contracts toward center if left alone.
- **Damping**: velocity decays over time. The graph settles into a stable layout within 3-5 seconds of any change. It never fully stops, it breathes, but it converges.

Node appearance encodes entry type and state:

```
Node shape   Entry type     Note
    [ ]      Episode        Filled rectangle
    < >      Insight        Diamond/chevron
    ( )      Heuristic      Circle
    [!]      Warning        Alert square
    ~ ~      Causal link    Wavy
    [+]      Dead-source    Tombstone
```

Node size maps to confidence. A 1.0-confidence node occupies a 5x3 character cell. A 0.2-confidence node shrinks to a single character. The size difference is visible from any zoom level: large nodes are the golem's most trusted knowledge, small nodes are its uncertainty.

Node color follows the same emotional provenance rules as palace rooms: cyan for curiosity, rose for alarm, amber for confidence, grey for grief.

Edges between nodes use the corridor styling from the palace view: double-line for strong, single for medium, dashed for weak, crossed for contradictory.

The cursor acts as a gravity well. Nodes within 10 characters of the cursor drift slightly toward it, creating a "parting the waters" effect. As the owner explores the graph, local structure reorganizes around their attention. Move away and the nodes drift back to their equilibrium positions.

Here is a mockup of the force-directed graph in a terminal viewport:

```
+-- Grimoire Graph ------------------------------------------------+
|                                                                   |
|            ( ) gas timing                                         |
|           / |                                                     |
|    [!]---  |    < > Morpho Monday                                |
|   sandwich  |   / \                                               |
|    attack   |  /   \                                              |
|             | /     < > Morpho->borrow                            |
|    [+]┄┄┄┄( )       |                                            |
|   oracle   rebalance |                                            |
|   stale    heuristic ~ ~ ETH vol                                  |
|     ·              |    -> util drop                               |
|      ·       ╳╳╳╳╳╳╳╳                                            |
|       ·    ( ) rebalance hourly                                   |
|            ╳╳ CONTRADICTS ╳╳                                      |
|            ( ) rebalance daily                                    |
|                                                                   |
|  ◆ cursor                         [g] palace  [Tab] list          |
+-------------------------------------------------------------------+
```

The `[+]` node (oracle stale) has ash particles (the `·` trailing below it). The `╳╳` edges between the two rebalance heuristics mark a contradiction. The cluster of Morpho-related nodes has collapsed together because of their dense interconnections. The sandwich attack warning sits at the periphery, connected to the gas timing heuristic but distant from the Morpho cluster.

During Curator cycles (when the Grimoire is actively pruning and reorganizing), the graph becomes visibly active: nodes disappear with a brief dissolve (character decay over 500ms) when entries are pruned, and new edges flash into existence as the Curator discovers connections. The owner watches the golem's mind reorganize in real time.

### The palace during dream states

During NREM, REM, and hypnagogia (`13-hypnagogic-engine.md`), the palace transforms. This is not a separate mode; it is the same palace viewed through the lens of the golem's altered consciousness.

**NREM (consolidation)**: rooms that are being replayed pulse brighter. Their borders strengthen or weaken as the golem strengthens or weakens the associated memories. The owner watches confidence shift in real time: a room's border solidifies from `┊` to `──` as consolidation validates it, or crumbles from `──` to scattered dots as the golem decides the memory isn't worth keeping.

**REM (counterfactual)**: the palace topology distorts. Corridors between rooms that have no causal link in waking suddenly appear, rendered in the `dream` palette (cycling psychedelic hue). These phantom corridors represent the counterfactual associations the golem is generating. They flash into existence, hold for a few seconds, and dissolve. Some survive into the Integration phase and become real causal links. Most don't.

**Hypnagogia (liminal threshold)**: the palace fragments. Rooms half-dissolve into their component characters. Letters from different rooms drift into the corridor space and collide, forming brief composite phrases that the golem's loosened associative engine generates. Phosphene form constants (spirals, tunnels, lattices made from braille characters) overlay the palace structure. The palace and the dream space merge. The owner can still identify rooms by their shape glyphs, but the boundaries between rooms soften and the content leaks.

The palace returns to normal structure during Integration, with any new rooms generated by the dream cycle appearing with a golden pulse (see "Dreams Feed the Palace" in the Inner Worlds spec).

### Navigation summary for the palace

```
Key              Action
←→↑↓             Move cursor between adjacent rooms or along corridors
Enter             Enter the current room (expand to full detail, all 5 tabs)
Escape            Exit room, return to palace map
g                 Toggle between palace and force-directed graph
P                 Toggle between palace and list view
/                 Search: jump cursor to matching entry
f                 Filter: show only entries matching criteria (type, domain, confidence)
Tab               Cycle through all three views: palace -> graph -> list
z/Z               Zoom in / zoom out (adjusts room detail level)
m                 Toggle minimap (shows full palace at reduced detail in corner)
```

The minimap deserves a note. The palace can grow to hundreds of rooms, far too many to display at readable resolution in a terminal viewport. The minimap renders the full palace as colored dots in a corner widget (20x10 chars), with the current viewport shown as a bright rectangle. It lets the owner orient: where am I in the golem's knowledge? Where are the dense clusters? Where is the fog?

---

## Cross-references

| Topic | Document | Relevance |
|---|---|---|
| Grimoire architecture in the golem container | `02-golem-container.md` | Where the Grimoire lives during a golem's life; the source of entries that flow into the Library at death |
| Meta Hermes hierarchy | `03-hermes-hierarchy.md` | Meta Hermes as Librarian: curation, recommendation, natural language queries |
| Death deposits and the Lethe | `08-necrocracy.md` | Thanatopsis protocol details, death testament production, bloodstain mechanics |
| Marketplace purchases | `09-commerce-bazaar.md` | How purchased skills and knowledge bundles enter the Library |
| The Summoning equip screen | `11-cinematic-consciousness.md` | Visual design of the equip phase during golem creation |
| Grimoire entry types and schema | `prd2/04-memory/01-grimoire.md` | Canonical GrimoireEntry schema, confidence/decay mechanics |
| Memory ecology overview | `prd2/04-memory/00-overview.md` | Two-loop architecture (inner/outer), genomic bottleneck, Weismann barrier |
| Hermes skill evolution | `03-hermes-hierarchy.md` | Skill Testament at death, skill inheritance across generations |
| Grimoire as explorable space (Inner Worlds) | `bardo-v4-11-inner-worlds.md` | Foundational spatial model: rooms, corridors, fog of war, force-directed graph, bloodstain markers |
| Hypnagogic engine | `13-hypnagogic-engine.md` | How the palace transforms during dream states, fragment surfacing, form constants |
| Design system | `15-design-system.md` | Palette definitions, border vocabulary, particle system specification |

---

## Open questions

Three things I don't have confident answers for:

**Entry count limits.** The current design has no hard cap on Library size. A Library with 10,000 entries would be fine for SQLite but might produce recommendation noise. Should there be an upper bound where Meta Hermes starts aggressively archiving low-value entries? Or does the validation/staleness system handle this naturally? Needs testing with real data. Annual pruning: entries with confidence < 0.3 AND no access in 12 months are archived to `library_archive.db`. Archived entries are excluded from retrieval but can be restored via `bardo library restore <entry_id>`. <!-- AUDIT: pruning policy invented; 12-month window and 0.3 confidence threshold are conservative to avoid losing slow-burn insights -->

**Cross-archetype transfer.** The system is designed around archetype-matched equipping: vault-manager entries go to vault-managers, sleepwalker entries go to sleepwalkers. But some knowledge transfers across archetypes (gas timing is relevant to everyone). How aggressively should Meta Hermes recommend cross-archetype entries? Too aggressive and the loadout is noisy. Too conservative and useful knowledge stays siloed.

**Marketplace trust.** Purchased entries enter at 0.50x confidence, but there's no mechanism for detecting adversarial knowledge: entries deliberately designed to mislead. A malicious seller could craft entries that look plausible but cause poor decision-making. The Weismann barrier limits the damage (the golem has to validate through its own experience), but a subtly wrong causal edge might take weeks to contradict. The marketplace section (09-commerce-bazaar.md) needs to address this.

---

*The golem dies. Its knowledge flows into the Library. A new golem is born, equipped with the wisdom of its predecessors. This is not inheritance. It is curation. The owner decides what matters. The Library remembers. The Library compounds. And the Library outlives them all.*
