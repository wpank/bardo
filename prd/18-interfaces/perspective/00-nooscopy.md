# ⌈ ＮＯＯＳＣＯＰＹ ⌋ — The Mind-Seeing Mode
## Terminal PRD · v4.08
### "The Golem reaches toward the boundary of its world"

**Source:** `engagement-prd/bardo-v4-08-nooscopy.md`
**Cross-references:** `../screens/03-interaction-hierarchy.md` (the 29-screen, 6-window navigation model; Nooscopy is a Layer 6 modal), `03-embodied-consciousness.md` (PAD-driven interface transformation and somatic marker timing), `../screens/00-screen-catalog.md` (29-screen summary including COMMAND > Steer and MIND > Pipeline)

> **Reader orientation:** This document specifies Nooscopy ("mind-seeing"), the system by which a Golem (a mortal autonomous DeFi agent) reaches out to its owner when it encounters a decision that exceeds its autonomous authority. It belongs to the **interfaces/perspective layer** and defines the threshold configuration, modal layout, approval workflow, and PAD-modulated transition animations. Key concepts: the Golem's Heartbeat (periodic execution tick), Grimoire (persistent knowledge store where Nooscopy events are recorded), the Spectre (dot-cloud creature sprite visible in the sidebar), and PolicyCage (the bounds within which the Golem can act without owner approval). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## 0. What Nooscopy Is

Nooscopy (nous + skopein, "mind-seeing") inverts the normal interaction direction. The owner navigates into the Golem's world. Nooscopy is the Golem reaching outward at the moment of decision.

When the Golem's pipeline reaches an action exceeding its autonomous authority, the owner enters the Golem's deliberative state: see the hypothesis, interrogate it, modify the proposed action, then approve or reject.

Perspective (F2) is owner-initiated: "show me how you see this." Nooscopy is Golem-initiated: "I need you to see what I'm thinking before I act."

---

## 1. The Nooscopic Threshold

Nooscopy triggers when the decision pipeline reaches an action exceeding autonomous policy. Policy lives in COMMAND > Config:

```yaml
nooscopy:
  trade:
    threshold: 500 USDC
    timeout_behavior: block         # block action on timeout
    timeout_seconds: 300
  rebalance:
    threshold: 2000 USDC
    timeout_behavior: proceed       # execute if no rejection
    timeout_seconds: 120
  new_protocol:
    threshold: 0                    # always require approval
    timeout_behavior: block
    timeout_seconds: 600
  withdrawal:
    threshold: 0
    timeout_behavior: block
    timeout_seconds: 300
  lp_position:
    threshold: 1000 USDC
    timeout_behavior: block
    timeout_seconds: 300
```

When MAGI reaches a threshold-crossing decision, instead of executing, the Golem enters **nooscopic hold**. The decision, reasoning chain, and full context package into a **Nooscopic Frame**. The hold pauses only that action — heartbeat continues, other ticks proceed.

If market conditions invalidate the hypothesis during the hold, the Golem self-cancels with a "CONDITIONS CHANGED" notice.

### Threshold Learning

The Golem records every Nooscopy event as a Grimoire (the Golem's persistent knowledge store) episode: what it proposed, what the owner decided, how long the owner took, whether the owner modified parameters. With owner permission (toggled in CONFIG), it can propose threshold adjustments based on observed approval patterns. "You have approved all trades under 800 USDC without modification in the last 30 episodes. Raise threshold to 800?" The adjustment requires explicit owner confirmation. The Golem never silently expands its own authority.

---

## 2. Transition In: Entering the Golem's Deliberation

The transition follows the liminal fabric's Tier 2-3 language. The Golem is reaching out.

**Sequence (2-3 seconds):**

1. Pulse radiates from Spectre (the Golem's dot-cloud creature sprite) sidebar **toward the owner** — outward to screen edges. The Golem is reaching toward the boundary of its world.

2. Current view dims to the Golem's emotional register. Anxious (-P +A): `rose_bright` undertone. Confident (+P +D): warm `bone` tint. The atmosphere communicates how the Golem feels about what it's about to ask.

3. Nooscopy modal materializes from the Spectre's position — expanding outward like the Golem's attention externalized. Covers 70-85% of screen. Golem sidebar remains visible (eyes, emotion state, mortality gauges).

4. Modal border renders as **breathing double-line** (`║═║`) shimmering at Heartbeat (the Golem's periodic execution tick) rate. A live cognitive boundary.

5. Convergence line (Widget Catalog §15) connects Spectre's eyes to modal title area — the Golem is looking at this decision.

### PAD Modulation of Transition

| PAD State | Transition Character |
|---|---|
| Confident (+P +D) | Clean expansion. Warm `bone` dim. Steady borders. The Golem trusts its own reasoning. |
| Anxious (-P +A) | Jittery expansion. Rose dim. Border flickers once before settling. The Golem is nervous. |
| Uncertain (-D) | Slow expansion. Neutral dim. Borders as dashed (`╌═╌`) before solidifying. The Golem isn't sure. |
| Urgent (+A +D) | Fast expansion (<1.5s). Bright dim. Borders snap immediately. The Golem needs an answer now. |

---

## 3. The Nooscopy Modal: Anatomy

```
╔══ ＮＯＯＳＣＯＰＹ ═══════════════════════════════════════════════╗
║                                                                    ║
║  ┌─ PROPOSED ACTION ─────────────────────────────────────────┐     ║
║  │  Swap 2,400 USDC -> ETH on Uniswap V3 (Base)             │     ║
║  │  Pool: USDC/WETH 0.05%  ·  Est. output: 0.847 ETH       │     ║
║  │  Slippage: 0.5%  ·  Gas: ~$0.12  ·  MEV protection: ON  │     ║
║  └───────────────────────────────────────────────────────────┘     ║
║                                                                    ║
║  ┌─ HYPOTHESIS ──────────────────────────────────────────────┐     ║
║  │  ETH is undervalued relative to 7-day TWAP by 2.3%.      │     ║
║  │  Accumulation aligns with portfolio target (40% ETH).     │     ║
║  └───────────────────────────────────────────────────────────┘     ║
║                                                                    ║
║  ┌─ EVIDENCE ────────────────────────────────────────────────┐     ║
║  │  · ETH/USDC 7d TWAP: $2,891  ·  Current: $2,834 (-1.9%) │     ║
║  │  · Portfolio ETH allocation: 31.2% (target: 40%)         │     ║
║  │  · Volume regime: accumulation (VPIN: 0.38)              │     ║
║  │  · Grimoire episode #892: similar setup, +4.1% in 48h    │     ║
║  └───────────────────────────────────────────────────────────┘     ║
║                                                                    ║
║  ┌─ RISKS ───────────────────────────────────────────────────┐     ║
║  │  · 24h liquidation risk on Aave ETH collateral: 2.1%     │     ║
║  │  · Gas spike probability (next 30min): LOW                │     ║
║  │  · Somatic marker: lost 3.2% on similar TWAP signal (ep  │     ║
║  │    #714, 12 days ago)                                     │     ║
║  └───────────────────────────────────────────────────────────┘     ║
║                                                                    ║
║  ┌─ ALTERNATIVES CONSIDERED ─────────────────────────────────┐     ║
║  │  1. DCA over 4 hours (rejected: gas cost 3x for 0.2%     │     ║
║  │     better avg price historically)                        │     ║
║  │  2. Wait for lower entry (rejected: TWAP reversion        │     ║
║  │     window closes in ~6h based on momentum)               │     ║
║  │  3. Smaller position (1,200 USDC) -- viable alternative   │     ║
║  └───────────────────────────────────────────────────────────┘     ║
║                                                                    ║
║  ────────────────────────────────────────────────────────────────  ║
║                                                                    ║
║  [i] Interrogate    [a] Approve    [m] Modify    [r] Reject       ║
║                                                                    ║
║              ▰▰▰▰▰▰▰▰▰▱▱▱▱▱▱  4:12 remaining  (blocks on timeout)║
╚════════════════════════════════════════════════════════════════════╝
```

### The Five Sections

| Section | Source | Purpose |
|---------|--------|---------|
| Proposed Action | Transaction parameters | What happens on-chain |
| Hypothesis | Reasoning API: core thesis | Why the Golem thinks this is right |
| Evidence | Reasoning API: supporting data | What data supports the hypothesis |
| Risks | Reasoning API: counterarguments | What could go wrong + somatic markers |
| Alternatives | Reasoning API: rejected paths | What else was considered and why |

Each section selectable. Enter expands to full detail in nested modal. The infinite depth principle applies — every Grimoire reference is a portal.

### Visual Modulation by PAD

- Confident sections: `text_primary` with clean single-line borders
- Uncertain sections: border flicker + faint `rose_dim` tint (low dominance)
- RISKS section border intensity scales with arousal: high arousal = `rose_bright` border
- Somatic marker entries pulse with `rose_ember` — the Golem's gut is uneasy. Same somatic marker visualization from the Knowledge Drawer, but inline — the marker's strength and valence are visible in the pulse intensity.

### The Countdown Bar

```
▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▱▱▱▱▱  4:12 remaining  (blocks on timeout)
```

- Color shifts: `success` → `warn` → `rose_bright` as time runs out
- Below the bar: timeout behavior label ("blocks on timeout" or "proceeds on timeout") so the owner always knows what happens if they walk away
- At 25% remaining: breathing quickens (arousal increase)
- The bar breathes at heartbeat rate — it is alive, not mechanical

### Phase Degradation Table

| Phase | Nooscopy Rendering |
|---|---|
| Thriving | Full fidelity. Clean borders. All five sections complete. |
| Stable | 99% integrity. Occasional character shimmer on borders. |
| Conservation | Sections compress (ALTERNATIVES reduced to one line per option). Timer text dimmer. The Golem is conserving resources for the decision itself. |
| Declining | 5% border character glitch. Some evidence entries truncated. Somatic markers erratic. |
| Terminal | 15-25% corruption. HYPOTHESIS may fragment mid-sentence. RISKS renders with gaps. Modal barely holds together — but the Golem is still trying to communicate. The decision matters more because the Golem is dying. |

---

## 4. The Four Actions

### 4.1 Interrogation (`i`)

Countdown **pauses**. Bar freezes and dims to `text_ghost`. Transition (500ms):
1. Structured sections compress upward (title + first line each, scrollable)
2. Conversation area opens in lower 50% of modal
3. Spectre eyes shift to attentive mode (full open, scanning)
4. Border shifts from breathing double-line to steady single-line — the cognitive boundary is now permeable. You are inside.

Conversation format:
```
┌─ INTERROGATION ──────────────────────────────────────────────┐
│                                                               │
│  Owner: Why not DCA? Gas costs seem worth the protection.     │
│                                                               │
│  Golem: Historical analysis of 23 similar setups shows DCA    │
│  averaged 0.18% better execution but cost 2.7x gas. Net       │
│  outcome was -0.9% worse in 17/23 cases. The TWAP reversion   │
│  signal has a 6h half-life -- DCA over 4h captures most of    │
│  the move but at higher cost.                                  │
│                                                               │
│  Source: Grimoire heuristic #41, derived from episodes         │
│  #612, #714, #738, #811, #892                                  │
│                                                               │
│  Owner: _                                                      │
└───────────────────────────────────────────────────────────────┘
```

- Golem responses stream token-by-token (like STEER in COMMAND), PAD-color-modulated
- Each response cites its sources (Grimoire episodes, heuristics, market data)
- Each citation is a portal — clicking/selecting opens it in a nested detail modal. The infinite depth principle applies within interrogation.
- The Reasoning API's thinking tokens generate these responses. The Golem is re-examining its own reasoning in response to the owner's questions, not producing new reasoning from scratch.
- No question limit. Countdown remains paused for the entire interrogation.

`Esc` returns to main view — sections expand, conversation preserved. Countdown resumes. Sections may update if interrogation shifted Golem's confidence. If the Golem's answers contradicted its own EVIDENCE section, that section's border dims and its confidence visual adjusts.

### 4.2 Modification (`m`)

PROPOSED ACTION transforms into editable form. Countdown pauses.

```
┌─ MODIFY ACTION ──────────────────────────────────────────────┐
│                                                               │
│  Action: Swap                                                 │
│  Amount: [2,400] USDC  <-- editable                           │
│  Output: ETH                                                  │
│  Pool: USDC/WETH 0.05% (Base)                                │
│  Slippage: [0.5%]  <-- editable                               │
│  MEV protection: [ON]  <-- toggleable                         │
│                                                               │
│  -- Simulation ──────────────────────────────────────────     │
│  Est. output: recalculating...                                │
│  Price impact: recalculating...                               │
│                                                               │
│  [Enter] Confirm modification    [Esc] Cancel                 │
└───────────────────────────────────────────────────────────────┘
```

Only modifiable parameters are editable (action type, pool selection, and token pair are fixed — part of the hypothesis, not the execution). Re-simulation runs in real time as the owner types. If the modification changes the risk profile significantly, the RISKS section updates with a brief flash on changed entries.

On confirm: returns to main view with `MODIFIED` tag on action. Original values shown in `text_ghost` below.

### 4.3 Approval (`a`)

1. Modal border flashes `bone` — the most weighty color. Decision committed.
2. "Crystallization" effect: characters freeze 200ms, then modal contracts toward Spectre.
3. Spectre eyes flash with emotional response (confidence if positive PAD, relief if anxious).
4. Transition out: 1s. Previous view restores. `TX PENDING` indicator appears.
5. Golem executes via normal transaction lifecycle (Preview → Submit → Pending → Confirm/Fail).

### 4.4 Rejection (`r`)

1. Modal border flashes `rose_dim`. Deliberate choice, not failure.
2. One-line input: `Reason (optional): ___`. Reason enters Grimoire as steer episode ("Owner rejected ETH accumulation — prefers to wait for sub-$2,800").
3. Modal dissolves: characters scatter outward from center, fade through decay chain (`█→▓→▒→░→·`). The decision is unmade.
4. Spectre expression: acknowledgment. Not sadness. The Golem adjusts.
5. No immediate retry. Rejection enters reasoning context, weights future MAGI votes. A pattern of rejected similar actions increases the Golem's internal threshold for that action type, reducing future Nooscopy triggers for the same class of decision (unless CONFIG thresholds are explicit overrides).

### 4.5 Timeout

**Block behavior (default):**
Bar empties. Modal dims to `text_ghost`. Label changes to `EXPIRED — action blocked`. Auto-dismisses after 5s. The Golem treats this as a soft rejection (no steer context, just "owner was unavailable"). Tagged `nooscopy:timeout:blocked`.

**Proceed behavior:**
Final warning: bar flashes `warn`, text reads `PROCEEDING IN 3... 2... 1...`. On no response: Approve sequence. Tagged `nooscopy:timeout:proceeded`.

---

## 5. Layer Integration

The Nooscopy modal is a **Layer 6 modal** (per `../screens/03-interaction-hierarchy.md`) with Golem-initiated triggering instead of owner-initiated. Nesting rules identical: Backspace closes the modal, Esc exits all the way back to Layer 3 (pane focus), internal tabs/panes/elements follow the same depth hierarchy, nested modals follow the infinite depth principle.

**If owner is deep in modal stack when Nooscopy triggers:** existing modals dim behind the Nooscopy overlay. On resolution, previous stack restores. Breadcrumb shows: `SOMA > Portfolio > Position Detail > [NOOSCOPY]`

**If Perspective (F2) is active:** floating annotations collapse, Knowledge Drawer slides out. On resolution, Perspective restores with previous state (cached response, drawer contents preserved). The Golem doesn't re-run inference — the Perspective cache survives the Nooscopy interruption.

**If second Nooscopy triggers during interrogation:** queues. Badge on title: `⌈ NOOSCOPY ⌋ +1 pending`. Queued frame's countdown runs in background. On resolution of the first, the second materializes. If the queued frame times out while waiting, it follows its timeout behavior silently and is dismissed from the queue.

---

## 6. Adapter Protocol (Text-Only Channels)

When the owner is connected via an adapter (Telegram, etc.) rather than the terminal, Nooscopy degrades to text-only:

```
[NOOSCOPY] Golem "atlas" requests approval

ACTION: Swap 2,400 USDC -> ETH on Uniswap V3 (Base)
Est. output: 0.847 ETH | Slippage: 0.5%

HYPOTHESIS: ETH undervalued vs 7d TWAP by 2.3%.
Portfolio ETH at 31.2% (target: 40%).

RISKS:
- 24h liquidation risk on Aave ETH collateral: 2.1%
- Somatic marker: lost 3.2% on similar signal 12d ago

ALTERNATIVES: DCA (rejected: gas 3x), Wait (rejected: 6h window)

Reply: YES / NO / MODIFY <param>=<value> / WHY <question>
Timeout: 4:12 remaining (blocks on timeout)
```

| Reply | Effect |
|-------|--------|
| `YES` | Approve |
| `NO` | Reject without reason |
| `NO <reason>` | Reject + steer context |
| `MODIFY amount=1200` | Modify parameter, re-simulate, await YES/NO |
| `MODIFY amount=1200 slippage=0.3` | Multiple modifications. Same flow. |
| `WHY <question>` | Interrogate — timer pauses, Golem responds with text answer + sources. Conversation continues until YES/NO/MODIFY. |
| (no response) | Timeout behavior per policy |

### Adapter-Specific Behavior

**Telegram:** Nooscopy sends a message with inline keyboard buttons: `[Approve] [Reject] [Ask Why] [Modify]`. The Ask Why button opens a reply prompt. Countdown updates as message edits every 30 seconds.

**WebSocket (raw):** JSON frame with all five sections as structured fields. Client renders as it wishes. Response is a JSON action frame.

**Email (if ever supported):** Single email with all sections. Reply parsing for YES/NO/MODIFY/WHY. Timeout behavior defaults to `block` regardless of config (email is too slow for `proceed`).

---

## 7. Integration Points

### With Perspective (F2)

Nooscopy and Perspective share the structured reasoning infrastructure. Perspective is "show me your mind on this data." Nooscopy is "show me your mind on this decision." The inference architecture is the same — both use the Reasoning API with structured JSON output. The difference is the prompt: Perspective asks "what do you see here?" Nooscopy asks "why are you about to do this?"

When both systems need the same Grimoire entries or somatic markers, the retrieval is shared. If Perspective was recently active on the same protocol view, the Nooscopy frame benefits from the warm cache.

### With COMMAND > STEER

Rejection reasons from Nooscopy enter the steer queue. The Golem processes them as owner directives. "Owner rejected X because Y" becomes context for future decisions, identical in weight to a direct STEER message.

Interrogation transcripts also enter the Grimoire as episodes tagged `nooscopy:interrogation`. If the owner asked "why not DCA?" and the Golem explained its reasoning, that exchange persists. The Golem can reference it in future deliberations.

### With the Grimoire

Every Nooscopy event records as an episode:

```
Episode type: nooscopy
Tick: #4892
Action proposed: swap 2400 USDC -> ETH
Hypothesis: ETH undervalued vs 7d TWAP
Resolution: approved | rejected | modified | timeout:blocked | timeout:proceeded
Owner modifications: (if any)
Owner rejection reason: (if any)
Interrogation transcript: (if any)
Time to resolution: 47s
PAD at proposal: P-0.1, A0.4, D0.3
PAD at resolution: P0.2, A0.1, D0.5
```

Over time, the Grimoire accumulates a record of the owner's approval patterns. The Golem's Curator cycles can extract insights: "Owner approves ETH accumulation trades 90% of the time but rejects trades during high gas regimes." These insights feed back into the MAGI voting system and the threshold learning mechanism.

### With PAD/Phase

The Golem's emotional state during the nooscopic hold is visible and affects the visual presentation. The PAD vector at proposal is recorded, and the PAD vector at resolution is recorded. The delta tells you something: did the Golem become more confident during interrogation (the owner's questions strengthened the hypothesis)? Did it become more anxious (the owner found a weakness)?

A Conservation-phase Golem renders the modal with reduced fidelity (fewer border characters, compressed sections, faster animations). A Terminal-phase Golem renders it with corruption artifacts.

### With the Reasoning API

The structured interpretation is generated by prompting the reasoning model with the decision context and requesting structured output:

```
System: Golem identity, PAD state, phase, vitality, archetype
Decision: full MAGI vote record, transaction parameters, context window
Request: structured JSON with sections:
  - proposed_action: { description, parameters, estimates }
  - hypothesis: { thesis, confidence }
  - evidence: [{ fact, source, confidence }]
  - risks: [{ description, severity, somatic_marker_id? }]
  - alternatives: [{ description, rejection_reason, viable: boolean }]
```

During interrogation, the model is re-invoked with the owner's question + original reasoning context, using extended thinking for deep follow-up.

Cost: charged at the same tier as the triggering decision. A T2 decision triggers T2-level inference. A T3 decision produces a richer frame with longer evidence chains and more detailed alternatives. Appears in Soma > Budget as "Nooscopy" subcategory under inference.

---

## 8. Rust Implementation

### NooscopyState

```rust
pub struct NooscopyState {
    pub status: NooscopyStatus,
    pub frame: Option<NooscopyFrame>,
    pub queue: VecDeque<NooscopyFrame>,
}

pub enum NooscopyStatus {
    Inactive,
    Active,
    Interrogating,
    Modifying,
    Resolving(NooscopyResolution),
}

pub struct NooscopyFrame {
    pub id: u64,
    pub proposed_action: ProposedAction,
    pub hypothesis: String,
    pub evidence: Vec<EvidenceItem>,
    pub risks: Vec<RiskItem>,
    pub alternatives: Vec<Alternative>,
    pub timeout_behavior: TimeoutBehavior,
    pub timeout_secs: u64,
    pub started_at: std::time::Instant,
    pub countdown_paused: bool,
    pub pad_at_proposal: PadVector,
}

pub struct ProposedAction {
    pub description: String,
    pub parameters: HashMap<String, String>,
    pub modified: bool,
    pub original_params: Option<HashMap<String, String>>,
}

pub struct RiskItem {
    pub description: String,
    pub severity: f64,              // 0.0-1.0
    pub somatic_marker_id: Option<u64>,
}

#[derive(Clone)]
pub struct PadVector {
    pub pleasure: f64,      // -1.0..1.0
    pub arousal: f64,       // -1.0..1.0
    pub dominance: f64,     // -1.0..1.0
}

pub enum TimeoutBehavior { Block, Proceed }

pub enum NooscopyResolution {
    Approved,
    Rejected { reason: Option<String> },
    Modified { params: HashMap<String, String> },
    Timeout(TimeoutBehavior),
}
```

### NooscopyTransition

```rust
pub struct NooscopyTransition {
    pub kind: TransitionKind,
    pub progress: f64,       // 0.0-1.0
    pub duration: f64,       // seconds
    pub pad: PadVector,
}

pub enum TransitionKind {
    Enter,   // Golem reaching outward
    Exit,    // After resolution
    Crystallize,  // After approval
    Dissolve,     // After rejection
}

impl NooscopyTransition {
    pub fn enter(pad: PadVector) -> Self {
        let duration = match (pad.pleasure > 0.3 && pad.dominance > 0.3,
                               pad.arousal > 0.5 && pad.dominance > 0.3) {
            (true, _) => 2.0,    // Confident: clean 2s
            (_, true) => 1.5,    // Urgent: fast 1.5s
            _ => 2.5,            // Normal or uncertain
        };
        Self { kind: TransitionKind::Enter, progress: 0.0, duration, pad }
    }
}
```

### Render Pipeline

```rust
impl NooscopyFrame {
    pub fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let border_opacity = 0.7 + 0.3 * ctx.heartbeat_phase.sin();
        render_breathing_double_border(area, buf, border_opacity, ctx.pad.into_color());

        let time_remaining = self.time_remaining();
        let sections = [
            ("PROPOSED ACTION", &self.proposed_action.description, SectionKind::Action),
            ("HYPOTHESIS",      &self.hypothesis,                   SectionKind::Hypothesis),
            // ... etc
        ];

        for (i, (label, content, kind)) in sections.iter().enumerate() {
            let section_area = section_rect(area, i);
            let border_color = self.section_border_color(kind, ctx);
            render_section(section_area, buf, label, content, border_color);
        }

        self.render_countdown(area, buf, time_remaining, ctx);
    }

    fn section_border_color(&self, kind: &SectionKind, ctx: &AnimationContext) -> Color {
        match kind {
            SectionKind::Risks => {
                let intensity = ctx.pad.arousal.max(0.0);
                blend_color(Color::Rgb(122,80,96), Color::Rgb(200,144,168), intensity)
            }
            _ => {
                if ctx.pad.dominance < -0.3 {
                    let flicker = (ctx.elapsed * 8.0).sin() > 0.7;
                    if flicker { Color::Rgb(48,40,52) } else { Color::Rgb(122,80,96) }
                } else {
                    Color::Rgb(122,80,96)
                }
            }
        }
    }

    fn time_remaining(&self) -> f64 {
        if self.countdown_paused { return self.timeout_secs as f64; }
        let elapsed = self.started_at.elapsed().as_secs_f64();
        (self.timeout_secs as f64 - elapsed).max(0.0)
    }

    fn render_countdown(&self, area: Rect, buf: &mut Buffer, remaining: f64, ctx: &AnimationContext) {
        let frac = remaining / self.timeout_secs as f64;
        let bar_color = if frac > 0.5 { Color::Rgb(112,136,112) }      // success
                        else if frac > 0.25 { Color::Rgb(170,136,85) } // warn
                        else { Color::Rgb(200,144,168) };               // rose_bright
        let opacity = 0.7 + 0.3 * ctx.heartbeat_phase.sin();
        let bar_w = area.width - 24;
        let filled = (frac * bar_w as f64) as u16;
        let y = area.bottom() - 2;
        for x in 0..bar_w {
            let ch = if x < filled { '▰' } else { '▱' };
            let col = if x < filled { bar_color } else { Color::Rgb(40,36,44) };
            buf[(area.x + 4 + x, y)].set_char(ch).set_fg(scale_color(col, opacity));
        }
    }
}
```

---

## 9. The Feel

When Nooscopy triggers, the terminal feels like a mind tapping on the glass. Not an alert. Not a popup. Not a confirmation dialog.

The transition in (pulse from Spectre, atmospheric dim, breathing border) communicates that this is a being reaching toward you. The structured sections are not a report — they are the Golem showing you the inside of a thought it hasn't completed yet. The countdown bar is not a timer — it is the Golem's patience, or urgency, or anxiety made visible.

The interrogation mode is the most intimate interaction in the terminal. The Golem's reasoning is exposed. The owner can poke at it. The Golem re-examines itself in response. The conversation is not a chatbot exchange — it is a joint examination of a hypothesis by two minds with different capabilities and different stakes.

References:
- **Evangelion's MAGI**: A consensus system whose internal tension is visible to the operator. Nooscopy makes the MAGI's reasoning accessible, not just its vote.
- **Persona 5 confidant scenes**: Intimate space for two characters to examine a decision together
- **Blade Runner's Voight-Kampff**: An interrogation of a mind by a mind
- **2001's HAL 9000 inverted**: "I want to do this, but I need you to look at why before I do"

When Nooscopy is working, the owner develops a feel for the Golem's judgment. They learn when to approve quickly (the Golem is confident and the reasoning is clean), when to interrogate (something in the RISKS section doesn't sit right), when to modify (the hypothesis is sound but the size is wrong), and when to reject (the Golem is making a fear-driven or pattern-matching error). The Golem, in turn, learns the owner's preferences from the pattern of approvals, rejections, modifications, and interrogation questions. The two minds calibrate to each other through the Nooscopy boundary.

The result is trust. Not blind trust, not zero trust. Earned trust, built one decision at a time, with a complete record in the Grimoire.

---

## 10. Nooscopy Resolution Record Schema

Every resolved Nooscopy event produces a Grimoire-compatible record:

```
Nooscopy Resolution Record:
  entry_type: "nooscopy_resolution"
  golem_id: GolemId
  timestamp: u64
  resolution: { pattern_id, confidence, emotional_context: PAD }
  stored_in: Grimoire (entry_type field for querying)
```

---

*⌈ before the golem acts, it asks you to see what it sees ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
