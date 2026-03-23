# Terminal Rack Editor [SPEC]

**TUI Integration, Live Patching, Presets, and Persistence**

> **Version**: 2.0 | **Status**: Draft | **Type**: SPEC (normative)
>
> **Crate**: `golem-sonification` (tui.rs), `golem-surfaces` (rack_pane.rs)
>
> **Cross-references**: [00-overview.md](./00-overview.md), [01-module-system.md](./01-module-system.md), Design System v3-interaction (window/tab/pane hierarchy)

> **Reader orientation**: This document specifies how users interact with the sonification engine through the terminal UI. The Bardo terminal is a ratatui-based TUI organized into 6 windows (HEARTH, MIND, VAULT, WORLD, FATE, COMMAND), each with tabs and panes. The sonification rack editor lives as a tab within the MIND window. Users can add, remove, and reconnect modules, adjust parameters with keyboard controls, save and load presets, and hear changes in real time. The rack editor is optional -- the default rack produces sound without any user interaction. This document is for users who want to customize.

---

## Placement in the TUI hierarchy

The Bardo terminal has a strict navigation hierarchy:

```
Windows (Tab/Shift-Tab) -> Tabs (number keys 1-9) -> Panes (arrow keys) -> Elements (Enter)
```

The sonification rack editor is:

```
MIND window -> Tab 4: SOUND -> Rack Editor pane
```

The MIND window already contains tabs for the Golem's cognitive state (inference history, context governor, prediction ledger). SOUND is a new tab added after the existing ones. Within the SOUND tab, there are four panes:

| Pane | Purpose |
|------|---------|
| **Rack** | The module graph -- visual layout of all modules and their connections. The main editing surface. |
| **Inspector** | Detail view of the selected module -- all parameters as editable knobs, port list, description. |
| **CV Map** | The CorticalState -> CV mapping table. Reassign sources, adjust scaling, add new outputs. |
| **Visualizer** | Real-time signal displays: waveform, envelope follower, active-notes piano roll, clock phase. |

Navigation between panes uses arrow keys (standard Bardo interaction). Enter on a module in the Rack pane opens it in the Inspector. Backspace returns to the Rack.

---

## Two UI modes

The rack editor has two modes. Composer Mode exposes the full modular routing. Listener Mode collapses it into five high-level controls. Both modes produce sound from the same underlying rack -- Listener Mode just changes the interface, not the synthesis graph.

### Composer Mode

This is the default and the mode described in most of this document. It shows the module graph, cable routing, all parameters as individual knobs, and the CV map. Every synthesizer parameter is exposed.

Composer Mode is for users who know modular synthesis, or want to learn it. The full module picker is accessible. Cable patching with `c` works here. The Inspector and CV Map panes are available.

### Listener Mode

Press `L` from the Rack pane (or from anywhere in Composer Mode) to enter Listener Mode. Press `L` again or `Esc` to return to Composer Mode.

Listener Mode replaces the module graph with five mood sliders. Each slider is a 0-10 scale displayed as a horizontal bar. Arrow Up/Down selects a slider, Arrow Left/Right adjusts its value.

The five sliders and their synthesis mappings:

**Brightness** (0-10): How open or muffled the sound is. At 0 the filter cutoff drops to 200 Hz and oscillators shift toward darker timbres (Plaits engine 0, sine-like). At 10 the filter opens fully and oscillators move to brighter harmonics (Plaits engine 4+, FM territory). Internally this is an additive offset to `filter_brightness` CV and `plaits.engine` parameter.

**Density** (0-10): How busy the texture is. At 0, trigger probability drops to 5% and only one voice sounds at a time. At 10, all probability gates fire and up to four voices overlap. Maps to `beads_gate.density`, `beads_gate.seed`, and the mixer voice-count parameter.

**Motion** (0-10): How restless or still the piece feels. At 0, the clock BPM multiplier drops to 0.25x and LFO rates halve. At 10, clock multiplier hits 4x and LFO rates double. Maps to `clock_bpm` CV multiplier and all `lfo.rate` parameters.

**Depth** (0-10): How much space surrounds the sound. At 0, reverb mix is 0% and delay feedback is 0% -- bone dry. At 10, reverb mix is 95% and delay feedback is 85% -- cathedral wash. Maps to `dattorro.mix`, `dattorro.decay`, and any delay module's `feedback` parameter.

**Warmth** (0-10): Timbral character from smooth to sharp. At 0, oscillators favor FM and additive modes, envelopes have fast attacks and short decays -- percussive, bright, transient. At 10, oscillators favor sine and analog-modeled modes, envelopes round out with slow attacks and long sustains. Maps to `plaits.engine` selection bias, `plaits.morph`, and `stages.attack`/`stages.decay`.

These slider values are additive offsets applied on top of whatever Composer Mode has configured. They shift the sound relative to the current patch, not replace it. Setting all sliders to 5 (the default) produces no offset -- the rack sounds exactly as patched in Composer Mode.

```
+-----------------------------------------------------------+
|  LISTENER MODE                                    [L] exit |
|                                                            |
|  Brightness  [=========>        ]  7                       |
|  Density     [===>              ]  3                       |
|  Motion      [=====>            ]  5                       |
|  Depth       [========>         ]  8                       |
|  Warmth      [======>           ]  6                       |
|                                                            |
|  Arrow Up/Down: select   Left/Right: adjust                |
+-----------------------------------------------------------+
```

---

## The Rack pane

The Rack pane displays the module graph as a text-mode schematic. Each module is a box with its name, ports listed on the sides (inputs left, outputs right), and patch cables drawn as Unicode line characters between connected ports.

### Visual layout

```
+--------------+          +--------------+          +--------------+
| Turing Mach. |          |   Quantizer  |          |    Plaits    |
|              |          |              |          |              |
|     cv_out --+----------+-- in         |          |              |
|    gate_out -+----+     |       out ---+----------+-- pitch      |
|              |    |     |   trigger ---+--+       |              |
|  *change 0.3|    |     |              |  |       |      out ----+-->
|  *length  16|    |     |  *scale  min |  |       |      aux ----+-->
+--------------+    |     |  *root   C3  |  |       |              |
                    |     +--------------+  |       |  *engine   0 |
                    |                       |       |  *decay  0.5 |
                    |     +--------------+  |       +--------------+
                    +-----+-- gate       |  |
                          |  Stages Env  |  |
                          |       out ---+--+-->
                          |              |  |
                          |  *attack 0.1 |  |
                          |  *decay  2.0 |  |
                          +--------------+  |
                                            |
                          +--------------+  |
                          | Beads Gate   +--+
                          |              |
                          |  *density 0.5|
                          |  *seed   0.4 |
                          +--------------+
```

Modules are positioned on a grid. The layout algorithm places modules left-to-right in topological order (signal sources on the left, effects on the right, output on the far right). Users can manually reposition modules with Shift+Arrow keys.

### Rendering details

| Element | Unicode | Color (ROSEDUST palette) |
|---------|---------|------------------------|
| Module box border | `+-+|` | `GHOST_40` (dim border) |
| Selected module border | `+-+|` | `ROSE_PRIMARY` (bright rose) |
| Module name | plain text | `GHOST_80` (bright) |
| Parameter (*) | `*` prefix | `GHOST_60` |
| Patch cable | `--+--` | `ACCENT_TEAL` |
| Selected cable | `--+--` | `ROSE_PRIMARY` |
| Audio signal flow | `-->` | `ACCENT_TEAL` |
| CV signal flow | `..>` | `ACCENT_GOLD` |
| Gate signal flow | `--->` | `ACCENT_VIOLET` |
| Disconnected port | dim text | `GHOST_20` |

The entire rack pane re-renders at 10fps (sufficient for a text-mode schematic that changes only on user input). This is separate from the audio processing, which runs at 48kHz independently.

---

## Keyboard controls

All controls follow the existing Bardo interaction model: arrow keys navigate, Enter selects/enters, Backspace goes up one level, number keys switch tabs.

### Rack pane (module graph) -- Composer Mode

| Key | Action |
|-----|--------|
| Arrow keys | Move selection between modules |
| Enter | Open selected module in Inspector pane |
| `a` | **Add module** -- opens module picker modal |
| `d` | **Delete module** -- removes selected module and all its cables |
| `c` | **Connect** -- start cable patching mode (see below) |
| `x` | **Disconnect** -- remove cable from selected port |
| Shift+Arrow | Reposition selected module on the grid |
| `p` | **Preset** -- open preset picker modal |
| `s` | **Save** -- save current rack as preset (opens naming dialog) |
| `m` | **Mute/unmute** -- toggle master audio output |
| `+` / `-` | Master volume up/down |
| `L` | **Listener Mode** -- switch to mood sliders |
| `o` | **Oblique Strategy** -- draw a random creative constraint |
| `C` | **Capture Moment** -- save a named snapshot of the current state |
| `v` | **Visualizer** -- toggle the Visualizer pane |

### Listener Mode

| Key | Action |
|-----|--------|
| Arrow Up/Down | Select slider |
| Arrow Left/Right | Adjust selected slider (-0.5 / +0.5 per press, hold Shift for -0.1 / +0.1) |
| `L` or `Esc` | Exit Listener Mode, return to Composer Mode |
| `m` | **Mute/unmute** |
| `+` / `-` | Master volume up/down |
| `o` | **Oblique Strategy** |
| `C` | **Capture Moment** |
| `v` | **Visualizer** toggle |
| `p` | **Preset** picker |
| `s` | **Save** preset |

### Cable patching mode

When `c` is pressed on a module:

1. The module's output ports highlight.
2. Arrow keys select which output port to patch from.
3. Enter confirms the source port.
4. Arrow keys now navigate to the destination module.
5. Enter on the destination highlights its input ports.
6. Arrow keys select which input port to patch to.
7. Enter confirms -- the cable is created.
8. Escape cancels at any step.

If the signal types are incompatible (e.g., Audio -> Gate), the destination port shows in red and Enter is blocked. If the connection would create a cycle, same treatment.

### Inspector pane (module detail)

| Key | Action |
|-----|--------|
| Arrow Up/Down | Select parameter |
| Arrow Left/Right | Adjust parameter value (fine: +/-0.01, hold Shift: +/-0.1) |
| Enter | Open parameter for numeric entry (type a value, Enter to confirm) |
| `r` | Reset parameter to default |
| `l` | **Link to CV** -- bind this parameter to a CV output (opens CV picker) |
| `u` | **Unlink** -- remove CV binding |
| Backspace | Return to Rack pane |

When a parameter is linked to a CV output, it shows the CV source name next to the value, and the value updates in real time as CorticalState changes. The parameter's knob value becomes an offset added to the CV.

### CV Map pane

| Key | Action |
|-----|--------|
| Arrow Up/Down | Select CV mapping row |
| Enter | Edit the selected mapping (opens editor modal) |
| `a` | Add new CV output |
| `d` | Delete selected CV output |
| `r` | Reset all mappings to defaults |
| Tab | Cycle between columns (source, output range, curve, smoothing) |

The CV map pane shows a table:

```
 CV Output           Source                  Range           Curve    a
----------------------------------------------------------------------
 clock_bpm           regime (0-3)            4.0 - 12.0     linear   0.10
 master_level        composite_vitality      0.0 - 1.0      log(3)   0.03
 filter_brightness   pleasure (-1-+1)        0.1 - 0.9      exp(2)   0.08
 event_density       arousal (-1-+1)         0.05 - 0.95    linear   0.10
 sustain_time        dominance (-1-+1)       0.1 - 0.9      linear   0.08
 noise_level         surprise_rate (0-1)     0.0 - 0.7      exp(2.5) 0.10
 root_pitch          comp_momentum (0-1)     36.0 - 60.0    linear   0.01
 reverb_depth        epistemic_vit (0-1)     0.8 - 0.1      linear   0.03  < inverted
 emotion_index       primary_emotion (0-7)   0.0 - 7.0      linear   0.05
```

---

## Visualizer pane

The Visualizer is the 4th pane in the SOUND tab. Toggle it with `v` from any mode. When hidden, the Rack/Inspector/CV Map panes expand to fill the space. When visible, the Visualizer takes the bottom third of the SOUND tab area.

It has four displays, stacked vertically:

### Mini waveform strip

A 256-sample rolling window of the final stereo output, rendered as two rows of ASCII braille characters. The top row is the left channel, the bottom row is the right channel. Each braille cell encodes a 2x4 dot grid, so 64 braille characters span the 256-sample window.

```
 L ⣿⣾⣹⣏⡇⣀⣀⣠⣤⣶⣿⣿⣷⣤⣀⡀⢀⣠⣴⣶⣿⣿⣿⣶⣤⣀⡀⢀⣀⣤⣶⣿⣿⣷⣶⣤⣀⡀⢀⣀⣠⣤⣶⣿⣿⣷⣶⣤⣄⡀⢀⣀⣠⣴⣶⣿⣿⣷⣶⣤⣀⡀⢀⣀⣤
 R ⣿⣷⣶⣤⣀⡀⢀⣀⣠⣴⣶⣿⣿⣿⣶⣤⣄⡀⢀⣀⣤⣶⣿⣿⣷⣶⣤⣀⡀⢀⣀⣠⣴⣶⣿⣿⣷⣶⣤⣀⡀⢀⣀⣤⣶⣿⣿⣿⣶⣤⣀⡀⢀⣀⣠⣤⣶⣿⣿⣷⣶⣤⣀⡀⢀
```

Updates at 10fps. The amplitude maps to vertical dot density -- silence is empty space, full scale is solid braille blocks.

### Envelope follower display

A single bar showing the current amplitude envelope, using Unicode block elements:

```
 ENV  ▁▂▃▅▆█████▇▆▅▃▂▁▁▁▁▁▂▃▅▇████▇▅▃▂▁
```

The bar is 40 characters wide and scrolls left as time passes, so you see roughly 1.3 seconds of envelope history. Updates at 30fps. When the sound is loud, the rightmost characters are tall blocks. As notes decay, the blocks shrink. Silence is `▁`.

### Active-notes piano roll

A horizontal strip showing 3 octaves (36 keys, C2 through B4). Each key is one character wide. Active notes show as filled blocks, recently released notes fade through shading.

```
 KEYS C2                  C3                  C4
      ░░░█░░░░░░░░░░░▓░░░░░░█░░░░░░░░░░░░░░░░
```

`█` = currently sounding, `▓` = released within the last 500ms, `▒` = released within the last second, `░` = released within the last 2 seconds, blank = silent. This gives a ghosting effect where you can see the melodic trail of recent notes.

### Clock phase display

Three small phase indicators for the gamma, theta, and delta clocks. Each clock is represented as a 5-character ASCII arc showing where it is in its cycle:

```
 CLOCKS   gamma (12s)   theta (60s)   delta (40m)
              /|            --|             |
             / |              |            /
```

The arc rotates clockwise. A full rotation is one complete clock cycle. The cycle duration is shown in parentheses. When a clock fires (completes a cycle), the indicator flashes bright for one frame.

---

## Oblique Strategies

Brian Eno and Peter Schmidt's Oblique Strategies are a deck of creative constraint cards. Each card introduces a forced constraint that breaks comfortable habits. The terminal has its own deck, tuned to modular synthesis.

Press `o` from any SOUND mode (Composer or Listener). A modal appears with a randomly selected strategy:

```
+-----------------------------------------------------------+
|  OBLIQUE STRATEGY                                          |
|                                                            |
|  "Mute the melodic layer for 45 seconds"                  |
|                                                            |
|  Activating in 3... 2... 1...                              |
|                                                            |
|  [Enter] Accept   [Esc] Dismiss   [n] Draw another        |
+-----------------------------------------------------------+
```

If the user presses Enter (or waits 3 seconds), the constraint activates. A countdown timer appears in the status bar:

```
 VOL 0.6  |  OBLIQUE: "Mute melodic" 0:38  |  MIND  |  GOLEM: stable
```

When the timer expires, the constraint lifts. Parameters don't snap back to their previous values -- they interpolate back over 5 seconds using one-pole smoothing, so the transition is a gradual release rather than a hard cut.

Press `Esc` during the countdown to dismiss the constraint early. Parameters still interpolate back.

### Built-in strategies

| # | Strategy | Duration | What it does to the rack |
|---|----------|----------|--------------------------|
| 1 | "Mute the melodic layer for 45 seconds" | 45s | Sets Plaits output VCA to 0. Drone and percussion continue. |
| 2 | "Transpose the root by a tritone" | 60s | Adds +6 semitones to `root_pitch` CV offset. |
| 3 | "Frozen grains: Clouds buffer freezes" | 60s | Sets `clouds.freeze` to 1.0. New audio stops entering the buffer. |
| 4 | "Remove all probability gates -- everything fires" | 60s | Sets all `beads_gate.density` params to 1.0. |
| 5 | "Maximum reverb" | 30s | Sets `dattorro.mix` to 1.0 and `dattorro.decay` to 0.99. |
| 6 | "Invert the velocity curve" | 60s | Flips the velocity mapping: quiet events become loud, loud become quiet. |
| 7 | "Highest register only" | 60s | Adds +24 semitones to pitch CV. Low notes disappear. |
| 8 | "Half speed" | 60s | Sets clock BPM multiplier to 0.5x. |
| 9 | "Maximum clock jitter" | 45s | Sets jitter parameter on all clock sources to 1.0. |
| 10 | "Mute the Drone Foundation layer" | 60s | Zeros the output of the drone oscillator and its filter chain. Melodic and percussive layers continue. |
| 11 | "Force Whole Tone scale" | 90s | Overrides the quantizer's scale to Whole Tone (0, 2, 4, 6, 8, 10). |
| 12 | "Reverse emotional valence" | 60s | Inverts the pleasure axis in the CV mapper: positive emotion maps to dark parameters, negative to bright. |
| 13 | "Solo the noise floor" | 30s | Mutes all oscillators. Only the noise source and effects tails remain. |
| 14 | "Double the LFO rates" | 45s | Multiplies all LFO rate parameters by 2.0. |

Users can add custom strategies through the config file at `{data_dir}/oblique_strategies.json`. Custom strategies define a name, duration, and a list of parameter overrides.

---

## Preset naming and tagging

When the user presses `s` to save a preset, a dialog opens:

```
+-----------------------------------------------------------+
|  SAVE PRESET                                               |
|                                                            |
|  Name: dark_cathedral_v2________________                   |
|        (max 32 characters)                                 |
|                                                            |
|  Tags: [ambient] [drone] [dark] [+add]                    |
|                                                            |
|  Context (auto):                                           |
|    Phase: mature   Emotion: melancholy                     |
|    Vitality: 0.72  Regime: 2 (volatile)                    |
|                                                            |
|  [Enter] Save   [Esc] Cancel                               |
+-----------------------------------------------------------+
```

The name field accepts up to 32 characters. After typing the name and pressing Enter, a fuzzy-search tag picker appears. The user can select up to 5 tags from a predefined list:

`ambient`, `drone`, `melodic`, `percussive`, `bright`, `dark`, `dream`, `terminal`, `sparse`, `dense`, `warm`, `cold`, `rhythmic`, `still`

Type to filter, Enter to select, Tab to confirm tag selection and return to the save dialog.

The preset is saved with three kinds of metadata:

1. **User metadata**: name + tags (what the user chose)
2. **CorticalState context**: the current phase, primary emotion, composite vitality, regime, and the raw CorticalState vector (32 floats) at the time of save
3. **Rack state**: the full module graph, cables, parameters, CV mappings

### Preset picker

Press `p` to open the preset picker. It shows all presets (built-in and user-created) with their tags and context:

```
+-----------------------------------------------------------+
|  LOAD PRESET                                   [Tab] Sort  |
|                                                            |
|  > ambient_default    [ambient] [drone]                    |
|    minimal_drone      [drone] [sparse]                     |
|    granular_texture   [ambient] [dense]                    |
|    dark_cathedral_v2  [ambient] [drone] [dark]             |
|    percussion_dust    [percussive] [sparse]                |
|    deep_dream         [dream] [ambient]                    |
|    terminal_requiem   [terminal] [dark]                    |
|                                                            |
|  Sorting: name                                             |
|  [Enter] Load   [Esc] Cancel   [Tab] Toggle sort          |
+-----------------------------------------------------------+
```

Tab cycles between two sort modes:

- **By name**: alphabetical
- **By similarity**: ranked by cosine similarity between the preset's saved CorticalState vector and the Golem's current CorticalState. The preset that best matches the organism's current emotional and cognitive state floats to the top.

The similarity sort means the preset picker is context-aware. If the Golem is anxious and volatile, presets saved during similar states appear first.

---

## Capture Moment

Press `C` (capital C, distinct from cable-patching `c`) to capture a named snapshot of the current sonic state. This is a personal archive feature -- no blockchain, no NFT, just a local record of a memorable moment.

A naming dialog opens:

```
+-----------------------------------------------------------+
|  CAPTURE MOMENT                                            |
|                                                            |
|  Name: first_profitable_trade______________                |
|        (max 64 characters)                                 |
|                                                            |
|  Capturing:                                                |
|    - Current knob values (all parameters)                  |
|    - CorticalState snapshot (32 signals)                   |
|    - 10-second audio render (48kHz WAV)                    |
|                                                            |
|  [Enter] Capture   [Esc] Cancel                            |
+-----------------------------------------------------------+
```

The user types a name (up to 64 characters -- longer than presets, because these names tend to be descriptive: "the moment of dread before crisis", "three hours into the dream cycle"). Press Enter to capture.

What gets saved:

- **Parameter snapshot**: all knob values across all modules. Not the module graph or cables (that's the preset's job) -- just the raw parameter state at that instant.
- **CorticalState snapshot**: all 32 atomic signals, the current phase, primary emotion, composite vitality, regime.
- **10-second audio render**: the engine renders 10 seconds of audio from the current state into a 48kHz 16-bit stereo WAV file. This is a non-real-time render -- it runs as fast as the CPU allows, typically finishing in under a second. The render captures what the rack sounded like at that exact moment.

Captured moments are stored in `{data_dir}/moments/` as individual directories:

```
moments/
  2026-03-18T14-32-01_first_profitable_trade/
    params.json       # all knob values
    cortical.json     # CorticalState snapshot
    audio.wav         # 10-second render
    meta.json         # name, timestamp, golem generation
```

### Browsing captured moments

Captured moments appear in a "Moments" section at the bottom of the preset picker (`p`). They are visually distinct from presets -- shown with a timestamp and the captured name, not tags. Loading a moment restores the parameter values but does not change the module graph or cables. It's like recalling a knob position from memory.

---

## Module picker modal

Pressing `a` in the Rack pane opens a modal listing all available modules, organized by category:

```
+--------------------------------------------------+
|  Add Module                                       |
|                                                   |
|  OSCILLATORS                                      |
|    Plaits Oscillator    24-engine macro oscillator |
|    Noise Source         White / pink / dust        |
|    LFO                 Low-frequency oscillator    |
|                                                   |
|  SEQUENCERS                                       |
|    Turing Machine       Shift register sequencer   |
|    Quantizer           Scale quantizer             |
|    Beads Gate Gen      Stochastic triggers         |
|    Clock Divider       Clock divider/multiplier    |
|                                                   |
|  FILTERS                                          |
|    Ripples SVF         State variable filter       |
|                                                   |
|  ENVELOPES                                        |
|    Stages Envelope     Segment generator           |
|                                                   |
|  AMPLIFIERS                                       |
|    VCA                 Voltage-controlled amp      |
|    Mixer               4-input summing mixer       |
|                                                   |
|  EFFECTS                                          |
|    Clouds Granular     Granular processor           |
|    Dattorro Reverb     Algorithmic reverb           |
|    Sample & Hold       Sample and hold             |
|                                                   |
|  [Enter] Add   [Esc] Cancel                       |
+--------------------------------------------------+
```

Arrow keys navigate, Enter adds the selected module to the rack with a default position and auto-generated ID (e.g., `plaits_2` if `plaits_1` already exists).

---

## Preset system

Rack configurations are saved as JSON presets. Each preset contains:

```json
{
  "name": "dark_cathedral_v2",
  "description": "Deep reverb, slow motion, dark filter. Saved during volatile regime.",
  "version": 2,
  "tags": ["ambient", "drone", "dark"],
  "context": {
    "phase": "mature",
    "primary_emotion": "melancholy",
    "composite_vitality": 0.72,
    "regime": 2,
    "cortical_vector": [0.31, -0.44, 0.12, "... (32 floats)"]
  },
  "modules": [
    {
      "id": "turing_1",
      "type": "TuringMachine",
      "position": [0, 2],
      "params": { "change": 0.3, "length": 16, "scale": 24.0 },
      "state": { "register": 42531 }
    }
  ],
  "cables": [
    { "from": "turing_1/cv_out", "to": "quantizer_1/in", "attenuation": 1.0 }
  ],
  "cv_map": [
    { "source": "Regime", "output": "clock_bpm", "range": [4.0, 12.0], "curve": "linear", "alpha": 0.10 }
  ],
  "listener_sliders": {
    "brightness": 5.0,
    "density": 5.0,
    "motion": 5.0,
    "depth": 5.0,
    "warmth": 5.0
  },
  "master_level": 0.6
}
```

### Built-in presets

| Name | Description |
|------|-------------|
| `ambient_default` | The default rack from `00-overview.md`. Turing Machine -> Quantizer -> Plaits -> Stages -> Ripples -> Reverb. Works with no configuration. |
| `minimal_drone` | Single oscillator with filter and reverb. Simplest possible patch. Good for testing. |
| `granular_texture` | Plaits -> Clouds granular processor. Dense grain clouds from the organism's voice. |
| `percussion_dust` | Beads gate gen -> noise source + Plaits percussion engines. Sparse, rhythmic, metallic. |
| `deep_dream` | Optimized for dream cycles: extra reverb, slow LFOs, wide stereo. Activates when `creative_mode=1`. |
| `terminal_requiem` | For the dying phase: whole-tone quantizer, particle noise, near-infinite reverb, fading to silence. |

Users can save their own presets and share them. Presets are stored in the Golem's data directory alongside the Grimoire.

---

## Persistence

The current rack state (modules, cables, parameters, CV map, Listener Mode slider positions) is saved automatically on every delta tick (~40-100 minutes) and on shutdown. On boot, the last saved rack is restored. If no saved rack exists, the `ambient_default` preset is loaded.

The rack state is stored in the Golem's redb database alongside other persistent state. The key is `sonification/rack`. The value is the serialized preset JSON.

### Generational inheritance

When a Golem dies and spawns a successor, the sonification rack preset is included in the genomic inheritance (alongside Grimoire entries, PLAYBOOK heuristics, and configuration). The successor starts with the parent's rack, including any customizations the user made. The sound evolves across generations just like the knowledge does.

Captured moments are also inherited. The successor's moment library includes every moment the parent (and grandparent, and so on) ever captured. The archive grows across generations.

---

## Live audio in the terminal

### Audio output architecture

The terminal application (bardo-terminal) runs as a separate process from the Golem container. The Golem runs on a micro VM (Fly.io); the terminal runs on the user's local machine and connects over WebSocket.

Audio cannot stream over the WebSocket connection (too much bandwidth, too much latency). Instead, the sonification engine runs locally in the terminal process, consuming CorticalState snapshots and EventFabric events that arrive over the WebSocket. The terminal has a local copy of the rack that processes audio on the user's machine.

```
+---------------------+     WebSocket      +--------------------------+
|  GOLEM (Fly.io VM)  |  --------------->  |  TERMINAL (local)         |
|                     |  CorticalState     |                          |
|  CorticalState      |  snapshots +       |  CorticalState cache     |
|  EventFabric        |  EventFabric       |  EventFabric replay      |
|                     |  events            |                          |
|  NO audio           |                    |  Sonification Engine     |
|  processing here    |                    |  (local cpal output)     |
|                     |                    |                          |
+---------------------+                    |  +------------------+   |
                                           |  |  Module Rack     |   |
                                           |  |  (processes at   |   |
                                           |  |   48kHz locally) |   |
                                           |  +--------+---------+   |
                                           |           |              |
                                           |           v              |
                                           |     speakers / DAC      |
                                           +--------------------------+
```

This architecture means:

- The Golem VM has zero audio CPU overhead. All synthesis happens on the user's machine.
- Audio latency is independent of network latency. The rack processes locally.
- The user hears sound even if the WebSocket connection is temporarily interrupted (the rack continues processing with the last known CorticalState values, parameters drifting slowly via smoothing).
- Multiple users watching the same Golem each get their own local audio with their own rack customizations.

### CorticalState synchronization

The terminal receives CorticalState snapshots at gamma frequency (every 5-15 seconds) over WebSocket. Between snapshots, the local CV mapper interpolates using the same one-pole smoothing. This means the audio parameters move smoothly even though updates arrive infrequently.

EventFabric events are forwarded in real time over the WebSocket. The event mapper processes them immediately, producing gates and triggers at their natural timing. The WebSocket latency (typically 20-100ms) adds a small delay to trigger timing -- acceptable for ambient music where precise rhythmic sync is not the point.

---

## Spectre integration

The Spectre (the Golem's visual creature in the sidebar) already animates based on CorticalState. The sonification engine can optionally sync visual and audio events:

- When a note plays (Plaits trigger fires), the Spectre's dot cloud briefly brightens -- a visual "ping" synchronized with the audio.
- When the scale changes (emotion shift), the Spectre's eye glyph transitions at the same time.
- When a trade executes, both the Spectre and the audio produce a simultaneous event.

This synchronization is one-directional: the sonification engine emits `sonification:note_played` events to the Event Fabric, and the Spectre renderer subscribes to them. The audio drives the visual, not the other way around. This prevents feedback loops.

---

## Mute and volume

The user can mute audio at any time with `m` in the SOUND tab or via the COMMAND window (`/sound mute`, `/sound unmute`, `/sound volume 0.5`). Muting does not stop the rack processing -- it only zeros the output. This means unmuting produces instant sound at the current state, with no warm-up delay.

The master volume is always visible in the terminal status bar:

```
 VOL 0.6  |  MIND  |  GOLEM: stable  |  tick #4,281
```

Or when muted:

```
 MUTE     |  MIND  |  GOLEM: stable  |  tick #4,281
```

When an Oblique Strategy is active, the status bar also shows the constraint countdown:

```
 VOL 0.6  |  OBLIQUE: "Half speed" 0:38  |  MIND  |  GOLEM: stable  |  tick #4,281
```

---

## References

- [RATATUI] "ratatui: A Rust library for building terminal user interfaces." -- *The TUI framework the terminal is built on.*
- [BARDO-DESIGN-V3-INTERACTION] "Interaction Architecture v3." -- *The window/tab/pane hierarchy this document extends.*
- [ENO-SCHMIDT-1975] Brian Eno and Peter Schmidt, "Oblique Strategies: Over One Hundred Worthwhile Dilemmas." -- *The creative constraint card deck that inspired the Oblique Strategy feature.*
