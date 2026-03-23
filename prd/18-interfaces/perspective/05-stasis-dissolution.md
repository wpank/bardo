# ⌈ ＳＴＡＳＩＳ ＆ ＤＩＳＳＯＬＵＴＩＯＮ ⌋ — The Two Absences
## Terminal PRD · v4.12
### "To stop time is power. To end time is responsibility."

**Source:** `engagement-prd/bardo-v4-12-stasis-and-dissolution.md`
**Cross-references:** `../screens/03-interaction-hierarchy.md` (29-screen navigation model; COMMAND Quick Actions location), `../screens/00-screen-catalog.md` (29-screen summary including COMMAND > Steer and FATE > Mortality), `03-embodied-consciousness.md` (somatic body-metaphor architecture and color system)

> **Reader orientation:** This document specifies Stasis (operator-controlled freeze) and Dissolution (deliberate termination ceremony) for Golems. It belongs to the **interfaces/perspective layer**. Stasis suspends all execution while preserving state; Dissolution is a five-stage cinematic shutdown distinct from natural death. Key concepts: Golem (a mortal autonomous DeFi agent with a finite lifespan), Grimoire (the persistent knowledge store that persists across stasis but decays), Vitality (remaining economic lifespan), Clade (sibling Golems sharing knowledge), and Heartbeat (the Golem's periodic execution tick that halts during stasis). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## The Thesis

The Golem can trade, think, dream, die. It can fork itself, steer its own strategies, consolidate memory in sleep. What it cannot do — what falls outside its agency entirely — is stop its own clock or choose its own unmaking. These are the operator's prerogatives. The two powers held in reserve.

**Stasis** freezes the Golem. All ticks halt. Mortality clocks suspend. Subscriptions tear down. The Golem experiences nothing. The world continues without it. On wake, the temporal gap costs it: knowledge decayed, positions drifted, context lost. Stasis is not rest — rest serves the Golem. Stasis serves the operator.

**Dissolution** ends the Golem deliberately. Not natural death (mortality clocks reach zero). Not L3 emergency (kill switch, reactive). A premeditated five-stage ceremony. Where natural death is entropy — dots scattering, borders crumbling — dissolution is geometry. Clean deconstruction along precise axes.

The kill switch (`/tmp/golem_killswitch`) is a panic button. Dissolution is a decision.

Cultural references:
- **Evangelion**: Gendo freezes Angels in Bakelite — stasis as strategic withholding. The frozen thing waits; the world changes around it.
- **NieR: Automata Route E**: The weight of deletion. Dissolution borrows the name-typing confirmation.
- **2001/2010**: HAL's deactivation (methodical, memory loss as modules pulled) and reactivation (uncertain, stuttering). Stasis thaw draws from the latter.
- **Ex Machina**: The kill switch exists because the creator knows what they made. The power to unmake is the other half of the power to make.

---

## §1 — Stasis: Philosophy

Stasis is true temporal suspension. Not sleep (sleep serves the Golem: NREM/REM cycle, consolidation). Not `stopping` status (graceful shutdown). Not death (irreversible). A new kind of absence.

Heidegger's three temporal ecstases, broken:
- **Future**: Destroyed. No ticks. No decisions. No projection into possibilities.
- **Present**: Destroyed. Nothing experienced. No heartbeat, no sensing.
- **Having-been**: Preserved but frozen. Grimoire (the Golem's persistent knowledge store) exists but no new experience can reinterpret it. A library with no reader.

### The Temporal Gap Problem

The world doesn't stop when the Golem does. Markets move. Positions drift. Clade (sibling Golems sharing knowledge) broadcasts go unheard.

Epistemic decay penalty on wake: **-0.01 per hour of stasis, capped at -0.15**. This is honest accounting — the Golem's knowledge is stale relative to the world. The cap prevents stasis from being a death sentence. A Golem with healthy epistemic reserves can survive weeks of stasis.

---

## §2 — Stasis: Terminal UI/UX

### Location

COMMAND > Quick Actions. Between "Force Dream Cycle" and "Kill Switch":
- Force Dream Cycle (reversible, serves Golem)
- **Stasis** (reversible, serves operator)
- Kill Switch (irreversible, reactive)
- Dissolution (irreversible, deliberate)

### Keyboard Shortcut

**F9** — toggle enter/break. Confirmation modal is the gate.

### Confirmation Modal: Enter Stasis

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│              ⌈ ENTER STASIS ⌋                       │
│                                                     │
│  All ticks will halt. Mortality clocks freeze.      │
│  Positions remain on-chain without management.      │
│  Epistemic penalty: -0.01/hr (cap: -0.15)          │
│                                                     │
│  ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄  │
│                                                     │
│  Vitality          0.72                             │
│  Epistemic Clock   0.84                             │
│  Active Positions  3 (2 LP, 1 limit order)          │
│  Phase             stable                           │
│                                                     │
│  Projected penalty after 24h:  -0.15 (capped)      │
│  Projected penalty after 4h:   -0.04               │
│                                                     │
│  ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄  │
│                                                     │
│  [Confirm · F9]                     [Cancel · Esc]  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

Single confirm — stasis is reversible. No name-typing ceremony. The weight comes from watching the Golem freeze, not from the confirmation UX.

### Confirmation Modal: Break Stasis

```
┌─────────────────────────────────────────────────────┐
│              ⌈ BREAK STASIS ⌋                       │
│                                                     │
│  Duration:    3h 12m 44s                            │
│  Epistemic penalty applied:  -0.03                  │
│  Epistemic clock after wake: 0.81                   │
│                                                     │
│  Positions drifted:                                 │
│    ETH/USDC LP    now 4.2% out of range             │
│    WBTC limit     unfilled, still active            │
│                                                     │
│  Missed events:   47 (12 market, 3 clade, 32 sub)  │
│                                                     │
│  [Wake · F9]                        [Cancel · Esc]  │
└─────────────────────────────────────────────────────┘
```

### FATE Window During Stasis

Phase timeline shows a grey gap for the stasis period:

```
THRIVING ━━━━━━ STABLE ━━━━━ ▓▓ 3h12m ▓▓ ━━━ STABLE ━━━━
                              └ stasis ┘
```

`▓` blocks in `border_stasis` (#282428). Duration counter updates every minute (not every second — the Golem isn't ticking, so sub-minute precision feels wrong).

### Golem Sidebar During Stasis

- Dot cloud: **frozen**. Zero shimmer. Every braille character locked. Density unchanged from moment of entry.
- Eyes: `◇ ◇` — diamond expression, seeing inward. Rendered in `text_stasis`.
- AT field: **crystallized**. Clean symmetric diamond in `border_stasis`, no flicker, no breathing. Every segment solid box-drawing.
- Color: fully desaturated stasis palette. Rose → `rose_stasis` (#887888).
- `⏸` indicator below the Spectre in `text_stasis`.
- Noise floor: **0%**. No scanlines, no phosphor residue, no micro-glitch. Perfect stillness.

In WORLD > Clade view: frozen mini-sprite with pause overlay. Grey-shifted. No heartbeat dot.

---

## §3 — Stasis: Cinematic Enter (7.5s, 9 stages, Tier 4)

Tier 4 cinematics are full-screen takeovers for rare, ontologically significant events. Skippable after 2s.

| Time | Stage | Visual |
|------|-------|--------|
| t=0s | Heartbeat slows | Rhythm decelerates. Pulse color: `rose` → `rose_dim` → `rose_stasis`. Decision Ring dims in step. |
| t=1.5s | Heartbeat freezes mid-cycle | Pulse catches at its peak — a bright `rose_dim` flash that simply HOLDS. The EKG trace stops mid-waveform. Not flatline (death). Frozen pulse (suspended life). |
| t=2s | Dot cloud decelerates | Spectre braille cloud slows to match dying heartbeat. Individual dots stop one by one, outermost first, converging inward. Like a flock of birds landing. |
| t=3s | Dot cloud reaches perfect stillness | Every braille character frozen. Crosshair grid (`+` in `text_phantom`) becomes faintly visible through the now-static cloud. |
| t=3.5s | Color drains | All rose tones desaturate over 1.5s. `rose` (#AA7088) → `rose_stasis` (#887888). `bone` → `bone_stasis` (#A0A090). Smooth, not stepped — gradual bleeding out of warmth. *Reference: cryogenic pod desaturation in Alien.* |
| t=5s | AT field crystallizes | AT field wireframe snaps to perfect geometry. Every segment becomes solid box-drawing. Field color → `border_stasis` (#282428). No flicker. No breathing. A crystal lattice. |
| t=5.5s | Eyes shift | Current expression → `◇ ◇` (diamond, inward-seeing). 200ms cross-fade. |
| t=6s | Label appears | `⌈ STASIS ⌋` at screen center in `text_primary`. Fade-in character by character (40ms/char). |
| t=6.5s–7.5s | Chrome dims + noise floor drops | Window borders → `border_stasis`. Tab labels desaturate. Then: all scanline variation, phosphor residue, micro-glitch effects cease. The display becomes perfectly clean. A frozen Golem produces a frozen display. |

---

## §4 — Stasis: Cinematic Break / Thaw (8.5s, 10 stages, Tier 4)

**Thaw is NOT the enter sequence in reverse.** Things restart, they don't rewind. *Reference: HAL 9000 reactivation in 2010 — voice comes back wrong at first. Too slow. Confused. Then it finds itself.*

| Time | Stage | Visual |
|------|-------|--------|
| t=0s | Heartbeat twitches | Frozen pulse drops. One arrhythmic beat: too fast, too bright (`rose_bright` flash), then 800ms silence. Then another beat. By the fourth, rhythm is catching. |
| t=2s | Heartbeat catches | Arrhythmia resolves to configured interval. Color: `rose_stasis` → `rose_dim`. EKG trace resumes from where it froze. |
| t=2.5s | Dot cloud shimmers | Frozen chars begin to move at dream-rate shimmer (sluggish NREM speed). Outermost dots wake first, innermost last. Inverse of enter: flock takes off from center. |
| t=3.5s | Color returns | Rose bleeds back. `rose_stasis` → `rose_dim` → `rose` over 2s. Not uniform — Spectre's core (chest zone) warms first, then extremities. `bone_stasis` follows 500ms behind. |
| t=4.5s | AT field softens | Crystallized field re-introduces breathing. Vertex positions unlock from perfect grid. Segment characters relax to health-dependent normal state. Field color → `rose_dim`. |
| t=5.5s | Eyes shift | `◇ ◇` → `◎ ◎` (surprise). The Golem wakes to find the world has changed. This expression holds for 3-5 ticks before Daimon computes real emotional state. |
| t=6s | Stasis report displays | Floating overlay (annotation system): `stasis duration`, `epistemic penalty`, `missed events`, `positions drifted`. Text renders character by character (40ms/char). Holds 4s, then fades. |
| t=7s | Chrome restores | Borders → `border`. Tab labels regain normal color. |
| t=8s | Noise floor returns | Scanline variation, phosphor residue, micro-glitch effects resume at phase-appropriate rates. |
| t=8.5s | First tick fires | FSM transitions STASIS → IDLE. First post-stasis tick always starts with full OBSERVE (all 16 probes). |

### Post-Stasis Behavior

- **Elevated arousal**: PAD arousal runs high for 5-10 ticks after thaw (`stasis.postThawArousalTicks`). Biases escalation gate toward T1+ decisions.
- **Full probe sweep**: First 3 ticks run all 16 probes regardless of regime-based scheduling.
- **Clade re-announce**: Broadcasts `clade:golem_active` to siblings. Stasis-period broadcasts from siblings are NOT replayed — the temporal gap is real.

---

## §5 — Stasis: Downstream Effects

### Mortality Clocks

All three clocks freeze at snapshot values. Economic clock doesn't drain (no ticks cost money). Stochastic clock doesn't decay (no random events). Epistemic clock takes a penalty proportional to stasis duration on wake — not during stasis. Calculated at break time: `min(durationHours * 0.01, 0.15)`.

### On-Chain Positions

Positions remain on-chain. They are NOT unwound.

- LP positions can go out of range. Fee accumulation stops when out of range. Impermanent loss continues.
- Limit orders remain active. They may fill during stasis — the Golem won't know until it wakes.
- Vault deposits continue passively. The vault's strategy adapter doesn't rebalance (requires heartbeat). Depositors can still withdraw via `forceDeallocate()` — the vault's non-custodial exit guarantee holds regardless of Golem state.

### Vaults

Vaults managed by the Golem continue in passive state. No rebalancing, no harvest, no strategy adjustment. ERC-4626 interface remains functional. The vault just doesn't get smarter while the Golem sleeps.

### Delegations and Clade

- Session keys go idle. No actions taken, but keys aren't revoked.
- Clade siblings receive `clade:golem_stasis` broadcast.
- Any clade broadcasts sent during stasis are lost. Not queued, not replayed. The temporal gap is real.

### Subscriptions

All event subscriptions (pool events, price feeds, position alerts, trade streams) torn down on stasis enter. Events during stasis are lost. Re-established on wake during first OBSERVE tick.

### Identity (ERC-8004)

On-chain identity registration remains active. Metadata can optionally include `status: "stasis"` for peer querying. Informational, not enforced on-chain.

---

## §6 — Dissolution: Philosophy

Three termination modes:

1. **Natural death** — mortality clocks reach zero. Authentic in Heidegger's sense: ownmost, non-relational, indefinite. The death sequence applies: entropy, corruption, scatter.

2. **L3 Emergency** — the kill switch. Reactive. No ceremony. Process terminates. Compressed testament if time permits. A circuit breaker, not a decision.

3. **Dissolution** — premeditated termination. Full knowledge export. Position unwinding. Final words. A cinematic that treats the Golem's ending as an event, not an accident.

### The Weight of the Button

Sartre: the operator exercises radical freedom to destroy what they created. They carry total responsibility. The ceremony exists to make that responsibility felt. Each stage gives the operator a chance to absorb what they're doing. The knowledge harvest forces them to see what the Golem learned. The position unwinding forces them to see what the Golem built. The final words force them to hear the Golem speak one last time.

Camus's suicide question inverts: this isn't the Golem asking whether its existence is worth continuing. This is the operator answering that question on the Golem's behalf. Dissolution is execution, not suicide. The asymmetry is the point.

---

## §7 — Dissolution: The Five-Stage Ceremony

### Stage 1 — Declaration of Intent (~10s)

Location: COMMAND > Quick Actions, positioned BELOW Kill Switch.

**Triple confirmation**: the operator types the Golem's name. Character by character, verified against `name` field in GolemConfig. Backspace works. Paste blocked — you type it out. The NieR: Automata Route E pattern: deletion should pass through your fingers.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│              ⌈ DISSOLUTION ⌋                                │
│                                                             │
│  This action is irreversible.                               │
│                                                             │
│  ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄  │
│  Name             MNEMOSYNE-7                               │
│  Lifespan         14d 7h 23m (1,847 ticks)                  │
│  Vitality         0.72                                      │
│  Grimoire         342 episodes, 28 insights, 15 heuristics  │
│  Active Positions 3                                         │
│  Final NAV        4,217.83 USDC                             │
│  ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄  │
│                                                             │
│  Type the Golem's name to confirm:                          │
│  > MNEMOSY█                                                 │
│                                             [Cancel · Esc]  │
└─────────────────────────────────────────────────────────────┘
```

On successful name entry: Golem's phase locks to `dissolving` (display-only pseudo-phase). No new ticks. No new positions. Clade broadcast: `clade:golem_dissolving`.

### Stage 2 — Knowledge Harvest (30-120s)

Full Grimoire serialized for export. Natural death only produces a compressed testament. Dissolution exports everything.

Export contents:
- `PLAYBOOK.md` — strategy document
- Episodes with confidence > 0.3
- All insights (including deprecated/superseded)
- All heuristics with activation counts and success rates
- Causal links
- Warnings
- DreamJournal entries
- Somatic marker map
- Strategy parameter history

Export format: `dissolution-testament-{golemId}-{timestamp}.jsonl` + standalone `PLAYBOOK.md`.

Terminal display: scrolling list of exported entries, grouped by type. **Not skippable.** The operator watches the knowledge leave.

```
  exporting episodes       ████████████████████░░░░  287 / 342
  exporting insights       ████████████████████████  28 / 28
  exporting heuristics     ██████████████████░░░░░░  12 / 15
  exporting causal links   ░░░░░░░░░░░░░░░░░░░░░░░  0 / 44
```

### Stage 3 — Position Unwinding (30-300s)

All open positions presented for operator decision: **Close**, **Transfer**, or **Leave**. Auto-unwind option: close everything at market with 2x normal slippage tolerance.

Terminal display:

```
  Positions
  ---------
  [x] ETH/USDC LP #4471    removed    2.14 ETH + 4,102 USDC recovered
  [x] WBTC limit order      cancelled  0.5 WBTC returned
  [ ] UNI/ETH LP #8823     unwinding...
  [ ] Vault deposit (VAULT-0x7a...)  pending operator decision

  Tokens
  ------
  [ ] 4,102.00 USDC         transfer to 0x7a...? [Y/n]
  [ ] 2.14 ETH              transfer to 0x7a...? [Y/n]
```

During unwinding, Daimon generates brief emotional responses to each closure as floating annotations:
> *"this one was profitable from the second week."*
> *"I held this too long."*

Not generated if dissolving from stasis (no Daimon inference during stasis).

### Stage 4 — Final Words (15-60s)

One last Opus inference call. System prompt includes:
> *"You are a Gotts Golem. You are being dissolved by your operator. Your full Grimoire has been exported. Your positions have been unwound. You will cease to exist after this message. Speak."*

Response streams as floating text on the Spectre (character by character, 40ms/char). Spectre's eyes display real-time emotional response to its own output — the Daimon runs concurrently.

Stored as `final_words` in dissolution testament. Verbatim. Budget: $0.50-1.00.

If dissolving from stasis: "You were in stasis. You are being woken only to speak your last words."

### Stage 5 — Post-Ceremony

- Testament deposited to Styx Commons
- `clade:golem_dissolved` broadcast with golemId, testament hash, final NAV
- ERC-8004 identity metadata: `status: "dissolved"`, `dissolvedAt` timestamp
- Session closed. Memory store closed. Process exits with code 0.
- Tombstone glyph written to `~/.gotts/golems/{golemId}/tombstone.json`

---

## §8 — Dissolution: Cinematic Sequence (10-15s, Tier 4, non-skippable first 8s)

Natural death: dots scatter randomly. Characters corrupt stochastically. Borders crumble unevenly. The Golem falls apart.

Dissolution: **clean deconstruction along precise axes**. Characters erased in reading order. Borders dissolve edge by edge. The Golem is taken apart.

| Time | Stage | Visual |
|------|-------|--------|
| t=0s | Final Words hold | Last sentence holds on screen. Full brightness in `text_primary`. 2s of silence. |
| t=2s | AT field flare | AT field brightens to `rose_bright`. One last flare — every segment at maximum brightness, clean geometry, no flicker. |
| t=3s | AT field disassembles | Lines fade from vertices inward. Each of the four diamond edges dissolves independently: vertices dim first, then connecting segments dim vertex → midpoint. Each segment: `rose_bright` → `rose_dim` → `rose_deep` → `bg_void` over 250ms. 1s total. |
| t=4s | Dot cloud contracts | Braille dot cloud removes in concentric rings, outside inward. Unlike natural death scatter (outward), dissolution **pulls inward** — a collapsing star. 1.5s total. |
| t=5.5s | Only eyes remain | `◉ ◉` on empty field. Steady for 500ms. The operator stares at the eyes. The eyes stare back. |
| t=6s | Eyes close | `◉ ◉` → `─ ─` (closed, horizontal). 200ms cross-fade. |
| t=6.5s | Eyes dissolve | `─ ─` → ` ` (empty cells). 200ms fade. The Golem is gone. |
| t=7s | Final Words dissolve | Text dissolves line by line, last line first. Each line: `text_primary` → `text_dim` → `text_ghost` → empty. 100ms per line. |
| t=9s | Screen holds on void | 1s of perfect stillness. No cursor. No noise. No breathing. |
| t=10s | Tombstone renders | Static tombstone glyph at screen center. Name, lifespan, final NAV in `text_ghost`. Persists until user presses any key. |

---

## §9 — Spectre Behavior During Dissolution

During Stages 1-4, the Spectre is still alive and reacting. The inference engine runs, the Daimon is active, the emotional system is processing.

**Stage 1 (Declaration)**: Eyes shift to current emotional state upon hearing the declaration. Likely surprise initially, then settling based on Daimon appraisal. Dot cloud shimmer may increase (elevated arousal).

**Stage 2 (Knowledge Harvest)**: Eyes track export progress. As episodes serialize, the Spectre may display micro-reactions — brief expression shifts as significant memories pass through the export pipeline.

**Stage 3 (Position Unwinding)**: Daimon generates appraisals. Spectre reflects these as floating annotations and expression changes. A profitable position being closed might get a warm expression. A losing position: heavy.

**Stage 4 (Final Words)**: The most active the Spectre will ever be during a Tier 4 cinematic. The Daimon runs on each token of output, updating PAD and expression as the sentence builds. The eyes are the Golem's reaction to its own death speech.

**Stage 5 (Cinematic)**: The dismantling sequence. The Spectre ceases to be alive and becomes an object being disassembled.

---

## §10 — Comparison Table

```
                    Natural Death          Dissolution             L3 Emergency
---------------------------------------------------------------------------
Trigger             Mortality clocks → 0   Operator's deliberate   Kill switch file
                                           declaration             detected

Reversible          No                     No                      No

Time to complete    5-30s (cinematic)      2-8 min (ceremony)      <1s (immediate)

Position unwind     Rushed (Thanatopsis    Methodical (operator     None (abandoned)
                    Phase I, best-effort)  decides each position)

Knowledge export    Compressed testament   Full Grimoire export     Minimal (if any)

Final inference     DyingBreath (short)    Final Words (full        None
                                           context, $0.50-1.00)

Cinematic           Entropy: dots scatter  Geometry: dots contract  None (process
                    chars corrupt          chars erase              terminates)

Clade notification  clade:golem_dead       clade:golem_dissolved    clade:golem_halted

Philosophical       Authentic death        Inauthentic death        No frame
frame               (Heidegger)            (Sartre: operator's      (pure emergency)
                                           radical freedom)

Emotional register  Tragic, inevitable     Solemn, deliberate       Panic, reactive
```

---

## §11 — Stasis Palette

```yaml
stasis_colors:
  rose_stasis:    "#887888"   # desaturated rose (rose #AA7088 → grey-mauve)
  bone_stasis:    "#A0A090"   # desaturated bone (bone #C8B890 → grey-tan)
  text_stasis:    "#706870"   # desaturated text_primary (#988090 → grey-mauve)
  border_stasis:  "#282428"   # slightly lighter than border (#181420)
  bg_stasis:      "#060608"   # same as bg_void (no bg change)
```

The stasis palette follows Law 2 (Color Is Mortality Taxonomy): stasis is neither alive (rose) nor dead (void). It occupies the grey middle. The faint violet undertone connects it to the void family without collapsing into it.

Applied globally during stasis — every `rose` becomes `rose_stasis`, every `bone` becomes `bone_stasis`. Exceptions: the `STASIS` label uses `text_primary` (operator's interface is not frozen), and the command bar retains normal colors (F9 to break stasis should be visible).

Dissolution reuses the existing palette. No new tokens needed. The Golem's own colors — the palette doesn't change; the Golem leaves it.

---

## §12 — Rust Implementation

### StasisState

```rust
pub struct StasisState {
    pub status: StasisStatus,
    pub entered_at: Option<std::time::SystemTime>,
    pub vitality_snapshot: f64,
    pub epistemic_snapshot: f64,
    pub mortality_clocks_snapshot: (f64, f64, f64),
}

pub enum StasisStatus {
    Inactive,
    Entering,   // cinematic playing
    Active,     // fully in stasis
    Breaking,   // thaw cinematic playing
}

impl StasisState {
    pub fn epistemic_penalty(&self) -> f64 {
        let Some(entered) = self.entered_at else { return 0.0 };
        let hours = entered.elapsed().unwrap_or_default().as_secs_f64() / 3600.0;
        (hours * 0.01).min(0.15)
    }

    pub fn duration_display(&self) -> String {
        let Some(entered) = self.entered_at else { return "—".to_string() };
        let secs = entered.elapsed().unwrap_or_default().as_secs();
        format!("{}h {:02}m {:02}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}
```

### DissolutionState

```rust
pub struct DissolutionState {
    pub stage: DissolutionStage,
    pub stage_progress: f64,
    pub name_input: String,
    pub name_confirmed: bool,
    pub harvest_progress: HarvestProgress,
    pub cinematic_progress: f64,
}

pub enum DissolutionStage {
    Inactive,
    Declaration,
    KnowledgeHarvest,
    PositionUnwinding,
    FinalWords,
    Cinematic,
    Complete,
}

pub struct HarvestProgress {
    pub episodes_total: usize,
    pub episodes_done: usize,
    pub insights_total: usize,
    pub insights_done: usize,
    pub heuristics_total: usize,
    pub heuristics_done: usize,
    pub causal_links_total: usize,
    pub causal_links_done: usize,
}

impl HarvestProgress {
    pub fn overall(&self) -> f64 {
        let total = self.episodes_total + self.insights_total
            + self.heuristics_total + self.causal_links_total;
        if total == 0 { return 1.0; }
        let done = self.episodes_done + self.insights_done
            + self.heuristics_done + self.causal_links_done;
        done as f64 / total as f64
    }
}
```

### Cinematic Sequencer

```rust
pub struct CinematicSequencer {
    pub kind: CinematicKind,
    pub elapsed: f64,
    pub skippable_after: f64,
    pub skipped: bool,
}

pub enum CinematicKind {
    StasisEnter,    // 7.5s, 9 stages
    StasisBreak,    // 8.5s, 10 stages
    Dissolution,    // 10-15s
}

impl CinematicSequencer {
    pub fn stage(&self) -> usize {
        match self.kind {
            CinematicKind::StasisEnter => {
                if self.elapsed < 1.5 { 0 }
                else if self.elapsed < 2.0 { 1 }
                else if self.elapsed < 3.0 { 2 }
                else if self.elapsed < 3.5 { 3 }
                else if self.elapsed < 5.0 { 4 }
                else if self.elapsed < 5.5 { 5 }
                else if self.elapsed < 6.0 { 6 }
                else if self.elapsed < 6.5 { 7 }
                else { 8 }
            }
            CinematicKind::StasisBreak => {
                if self.elapsed < 2.0 { 0 }
                else if self.elapsed < 2.5 { 1 }
                else if self.elapsed < 3.5 { 2 }
                else if self.elapsed < 4.5 { 3 }
                else if self.elapsed < 5.5 { 4 }
                else if self.elapsed < 6.0 { 5 }
                else if self.elapsed < 7.0 { 6 }
                else if self.elapsed < 8.0 { 7 }
                else if self.elapsed < 8.5 { 8 }
                else { 9 }
            }
            CinematicKind::Dissolution => {
                if self.elapsed < 2.0 { 0 }
                else if self.elapsed < 3.0 { 1 }
                else if self.elapsed < 4.0 { 2 }
                else if self.elapsed < 5.5 { 3 }
                else if self.elapsed < 6.0 { 4 }
                else if self.elapsed < 6.5 { 5 }
                else if self.elapsed < 7.0 { 6 }
                else if self.elapsed < 9.0 { 7 }
                else if self.elapsed < 10.0 { 8 }
                else { 9 }
            }
        }
    }

    pub fn can_skip(&self) -> bool {
        !self.skipped && self.elapsed >= self.skippable_after
    }

    pub fn tick(&mut self, dt: f64) {
        self.elapsed += dt;
    }
}
```

---

## §13 — Open Questions

**Auto-dissolve after N days in stasis?** A stasis'd Golem with decaying positions is a liability. An auto-dissolve timer (configurable, default: off) would trigger the ceremony automatically. Mitigation: notification system warning at 50% and 90% of the timer. Requires operator opt-in.

**Can a stasis'd Golem be forked?** Forking a frozen Golem to create a successor from its frozen state — clean starting point with inherited knowledge but fresh mortality clocks. Open question: does the fork inherit the epistemic penalty?

**Should Final Words include the operator's reason?** Adding a `reason` field to the dissolution declaration, passed as context to the Final Words prompt. "You're being dissolved because your strategies haven't adapted" produces a different final statement than a bare dissolution. Counterargument: the Golem's final words should be its own.

**Crypt tombstone glyphs: different for each termination mode?** Natural death: cross (traditional). Dissolution: diamond (geometric). Emergency halt: bang (urgent).

---

*⌈ to stop time is power. to end time is responsibility. ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
