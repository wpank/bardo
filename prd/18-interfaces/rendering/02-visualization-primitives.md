# ⌈ ＢＡＲＤＯ ＤＥＳＩＧＮ ＳＹＳＴＥＭ ⌋
## Visualization Primitives · v4.0
### Implementation Reference for Terminal Data Displays

**Cross-references:** `04-nerv-aesthetic.md` (NERV institutional register addendum with B10-B15 visual specs), `00-design-system.md` (ROSEDUST visual identity spec for palette and atmospheric layers), `../screens/02-widget-catalog.md` (33-widget TUI component library), `../19-spatial-grammar.md` (spatial layout grammar with zone architecture and depth planes)

> **Reader orientation:** This document specifies 13 visualization primitives for the Bardo terminal's data displays (waveforms, density fields, radial gauges, force graphs, etc.). It belongs to the **interfaces/rendering layer** and provides Rust struct definitions, render skeletons, and character/color mappings for each primitive. These are the building blocks composing the 29 screens. Key concepts: PAD vector (Pleasure-Arousal-Dominance emotional coordinates modulating visual properties), Heartbeat (the Golem's periodic execution tick driving animation phase), and the ROSEDUST palette (the terminal's rose-on-violet-black color system). For unfamiliar terms, see `prd2/shared/glossary.md`.

---

## Overview

Thirteen visualization primitives extracted from image research and witness-research reconciliation (Evangelion screenshots, oscilloscope references, tactical radar UIs, retro tracker interfaces, geodesic displays). Each primitive follows the same structure: description, visual spec, character/color mapping, Rust struct + trait, and `render()` skeleton.

All primitives implement `DataPrimitive`:

```rust
pub trait DataPrimitive {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext);
    fn tick(&mut self, dt: f64);
}
```

`AnimationContext` carries elapsed time, PAD vector, heartbeat phase, and ROSEDUST palette reference.

---

## 1. WaveformDisplay

Rolling time-series oscilloscope trace. Single-trace or paired-comparison mode. Phosphor decay chain makes old samples dim.

### Visual Spec

```
SINGLE TRACE:
  AROUSAL  ▁▂▃▃▂▁▁▂▃▅▇█▇▅▃▂▁▁▂▃▃▂▁▁▂▃▅▆▅▃▂  +0.18

DUAL TRACE (Signal Injection mode):
  ┌──────────────────────┐   ┌──────────────────────┐
  │ ⌈ CHANNEL-A ⌋        │   │ ⌈ CHANNEL-B ⌋        │
  │ ▁▂▃▅▇█▇▅▃▂▁▁▂▃▅▇    │   │ ▁▁▂▃▃▂▁▁▂▃▅▆▅▃▂▁▁    │
  └──────────────────────┘   └──────────────────────┘
  DELTA: +0.37 ↑  DIVERGING
```

### Rust Struct

```rust
pub struct WaveformDisplay {
    data: VecDeque<f64>,
    window: usize,
    pub palette: &'static [char],
    pub color: Color,
    pub phosphor_decay: f64,    // 0.0-1.0, applied per sample per frame
    pub mode: WaveformMode,
    label: String,
}

pub enum WaveformMode {
    Single { label: String },
    Dual { label_a: String, label_b: String, data_b: VecDeque<f64> },
    MultiChannel { channels: Vec<(String, Color, VecDeque<f64>)> },
}

impl WaveformDisplay {
    pub fn push(&mut self, v: f64) {
        if self.data.len() >= self.window {
            self.data.pop_front();
        }
        self.data.push_back(v.clamp(0.0, 1.0));
    }

    fn phosphor_intensity(&self, age: usize) -> f64 {
        (1.0 - self.phosphor_decay).powi(age as i32)
    }
}

impl DataPrimitive for WaveformDisplay {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        // Render label at left
        // Map most recent data[window-1] to right edge
        // Scale data 0..1 to palette index
        // Apply phosphor_intensity as brightness modifier
        let blocks: &[char] = &['▁','▂','▃','▄','▅','▆','▇','█'];
        for (i, &val) in self.data.iter().rev().take(area.width as usize).enumerate() {
            let age = i;
            let intensity = self.phosphor_intensity(age);
            let idx = (val * 7.0).round() as usize;
            let ch = blocks[idx.min(7)];
            let col = scale_color(self.color, intensity);
            let x = area.right() - 1 - i as u16;
            buf[(x, area.bottom() - 1)].set_char(ch).set_fg(col);
        }
        // Current value at top-right
        if let Some(&v) = self.data.back() {
            let s = format!("{:+.2}", v * 2.0 - 1.0);
            buf.set_string(area.right() - s.len() as u16, area.top(), &s,
                Style::default().fg(self.color));
        }
    }

    fn tick(&mut self, _dt: f64) {}
}
```

### Character Mapping

| Value | Character | Meaning |
|-------|-----------|---------|
| 0.0 | `▁` | Floor |
| 0.5 | `▄` | Mid |
| 1.0 | `█` | Peak |

Phosphor chain: current frame 100% → -1 frame 80% → -2 frame 60% → -4 frame 35% → -8 frame 15% → invisible.

---

## 2. ThermalField

2D heatmap on a grid. `Vec<Vec<f64>>` data mapped through a colormap. Character density encodes value intensity. Supports animated evolution.

### Visual Spec

```
LOW DENSITY:                HIGH DENSITY:
  · · · · · ·               ▓ █ █ ▓ ▒ ░
  · · + · · ·               █ █ █ █ █ ▓
  · + ✦ + · ·               ▓ █ ✦ █ █ ▓
  · · + · · ·               █ █ █ █ █ ▓
  · · · · · ·               ░ ▒ ▓ █ ▓ ░

COLORMAPS:
  RoseDust:  bg_void → rose_deep → rose → rose_bright → bone
  Viridis:   indigo → teal → green → yellow
  Inferno:   black → purple → orange → white
```

### Rust Struct

```rust
pub struct ThermalField {
    data: Vec<Vec<f64>>,        // row-major, values 0.0-1.0
    width: usize,
    height: usize,
    pub colormap: ThermalColormap,
    pub update_fn: Option<Box<dyn Fn(&mut Vec<Vec<f64>>, f64)>>,
}

pub enum ThermalColormap {
    RoseDust,
    Viridis,
    Inferno,
    Custom(Vec<Color>),
}

const THERMAL_BG: &[char] = &[' ', '░', '▒', '▓', '█'];
const THERMAL_FG: &[char] = &['.', '·', '+', '✦'];

impl DataPrimitive for ThermalField {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let scale_x = self.width as f64 / area.width as f64;
        let scale_y = self.height as f64 / area.height as f64;
        for row in 0..area.height {
            let dy = (row as f64 * scale_y) as usize;
            for col in 0..area.width {
                let dx = (col as f64 * scale_x) as usize;
                let v = self.data[dy.min(self.height-1)][dx.min(self.width-1)];
                let bg_idx = (v * 4.0) as usize;
                let ch = THERMAL_BG[bg_idx.min(4)];
                let color = self.colormap.sample(v);
                buf[(area.x + col, area.y + row)].set_char(ch).set_fg(color);
            }
        }
    }

    fn tick(&mut self, dt: f64) {
        if let Some(f) = &self.update_fn {
            // Clone and call — update_fn mutates the data grid
        }
    }
}
```

---

## 3. RadarDisplay

Polar/circular plot. Center point, concentric rings at intervals, radial value arms. Corner data boxes standard. Animated sweep arc option.

### Visual Spec

```
         ┌──────────┐
         │  VOL: H  │
         └────┬─────┘
   ╌ ╌ ╌ ╌ ╌│╌ ╌ ╌ ╌ ╌
  ·    ─ ─ ─╋─ ─ ─    ·
  ·  ─   ╌ ╌│╌ ╌   ─  ·   ← concentric rings (text_ghost → rose_dim → rose)
  · ─   ╌   ╋   ╌   ─ ·
  ·  ─   ╌ ╌│╌ ╌   ─  ·
  ·    ─ ─ ─╋─ ─ ─    ·
   ╌ ╌ ╌ ╌ ╌│╌ ╌ ╌ ╌ ╌
         ╔══╧══╗
         ║  ▸  ║    ← value arm pointing at current reading
         ╚═════╝

SWEEP ARC:
  A dim arc (`·`) sweeps clockwise at 1 rotation per 4 heartbeats.
  As it passes a probe: that probe's arm brightens (rose_bright flash, 200ms).
```

### Rust Struct

```rust
pub struct RadarDisplay {
    pub values: Vec<f64>,       // one per arm, 0.0-1.0
    pub labels: Vec<String>,    // per arm
    pub ring_count: usize,      // concentric rings
    pub sweep_enabled: bool,
    sweep_angle: f64,           // current sweep position (0..TAU)
    pub corner_data: [Option<CornerElement>; 4],
}

pub struct CornerElement {
    pub value: String,
    pub label: String,
    pub status: CornerStatus,
}

pub enum CornerStatus { Normal, Warning, Alert }

impl DataPrimitive for RadarDisplay {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let cx = area.x + area.width / 2;
        let cy = area.y + area.height / 2;
        let r = (area.width.min(area.height) / 2) as f64;
        // Render concentric rings (text_ghost for outer, rose_dim for inner)
        // Render value arms (length proportional to value)
        // Render sweep arc if enabled
        // Render corner elements
    }

    fn tick(&mut self, dt: f64) {
        if self.sweep_enabled {
            self.sweep_angle = (self.sweep_angle + dt * std::f64::consts::TAU / 8.0)
                % std::f64::consts::TAU;
        }
    }
}
```

### Character Mapping

| Element | Characters | Color |
|---------|-----------|-------|
| Center | `╋` | `bone` |
| Ring (outer) | `─ │ ╌` | `text_ghost` |
| Ring (inner) | `─ │` | `rose_dim` |
| Value arm | `▸ ─` | `rose` |
| Sweep arc | `·` | `text_dim` |
| Corner box | `┌ ┐ └ ┘` | `border` |

---

## 4. ForceGraph

Node-link graph with soft-body simulation. Nodes as `Box<dyn Renderable>`, edges as unicode lines. Verlet integration, repulsion + spring forces. Cursor acts as gravity well. Never fully settles.

### Visual Spec

```
    ◆ episode
   ╱                ◇ insight
  ╱                 │
 ● heuristic  ───── ∴ causal
        ╲
         ⚡ skill

MOTION: nodes drift gently. Edges recompute each frame. Cursor (◆ bright)
        attracts nearby nodes slightly — "parting the waters" effect.
```

### Rust Struct

```rust
pub struct ForceGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub cursor_pos: Option<(f64, f64)>,
    repulsion: f64,
    spring_k: f64,
    damping: f64,
}

pub struct GraphNode {
    pub id: usize,
    pub pos: (f64, f64),        // normalized 0..1
    vel: (f64, f64),
    pub label: String,
    pub glyph: char,
    pub color: Color,
    pub size: f64,              // 0.5-2.0, affects rendering weight
}

pub struct GraphEdge {
    pub a: usize,
    pub b: usize,
    pub weight: f64,            // 0.0-1.0 → line style
    pub edge_type: EdgeType,
}

pub enum EdgeType {
    Strong,     // ═══
    Normal,     // ───
    Weak,       // ┄┄┄
    Contradicts,// ╳╳╳
    DeadSource, // †┄┄┄†
}

impl DataPrimitive for ForceGraph {
    fn tick(&mut self, dt: f64) {
        // 1. Apply repulsion between all node pairs (O(n²), keep n < 100)
        // 2. Apply spring forces on edges
        // 3. Apply cursor gravity well if cursor_pos is Some
        // 4. Apply damping (vel *= self.damping)
        // 5. Integrate positions (Verlet)
        // Add small random perturbation to prevent full settling
        for node in &mut self.nodes {
            node.vel.0 += (fastrand::f64() - 0.5) * 0.001;
            node.vel.1 += (fastrand::f64() - 0.5) * 0.001;
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        // Scale node positions to area dimensions
        // Render edges first (using sdf_line approach for character selection)
        // Then render nodes (glyph + label if space allows)
    }
}
```

### Edge Character Selection

```rust
fn edge_char(dx: f64, dy: f64, edge_type: &EdgeType) -> char {
    let angle = dy.atan2(dx).to_degrees().abs();
    match edge_type {
        EdgeType::Strong => if angle < 22.5 || angle > 157.5 { '═' } else if angle < 67.5 { '╲' } else if angle < 112.5 { '║' } else { '╱' },
        EdgeType::Normal => if angle < 22.5 || angle > 157.5 { '─' } else if angle < 67.5 { '╲' } else if angle < 112.5 { '│' } else { '╱' },
        EdgeType::Weak   => if angle < 22.5 || angle > 157.5 { '╌' } else { '╎' },
        EdgeType::Contradicts => '╳',
        EdgeType::DeadSource => '†',
    }
}
```

---

## 5. TimelineRibbon

Horizontal time axis with colored event segments. Current time marker. Hover tooltip. Japanese timestamp style optional.

### Visual Spec

```
Jan 15               Jan 22               Jan 29
  │                    │                    │
  ├████████████████████████████████████░░░░░┤
  │■■■■■■■■■■■■■│▒▒▒▒▒▒▒▒▒▒▒▒▒▒│░░░░░░░░░░░│
  │ THRIVING    │  STABLE      │ DECLINING │
                                     ↑ now (2026-01-27)

HOVER TOOLTIP:
  ┌───────────────────────┐
  │ 2026-01-22 04:17 JST  │
  │ Phase: STABLE         │
  │ Vitality: 0.61        │
  └───────────────────────┘
```

### Rust Struct

```rust
pub struct TimelineRibbon {
    pub segments: Vec<TimelineSegment>,
    pub start_time: i64,    // unix timestamp
    pub end_time: i64,
    pub current_time: i64,
    pub japanese_timestamps: bool,
    pub hover_pos: Option<u16>,     // column under cursor
}

pub struct TimelineSegment {
    pub start: i64,
    pub end: i64,
    pub color: Color,
    pub label: String,
    pub fill_char: char,
}

impl DataPrimitive for TimelineRibbon {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let span = (self.end_time - self.start_time) as f64;
        // Map each segment to pixel range
        // Render fill chars with segment color
        // Render current time marker (bright vertical `┃`)
        // Render date labels at width-appropriate intervals
        // Render hover tooltip if hover_pos is Some
    }

    fn tick(&mut self, _dt: f64) {}
}
```

### Timestamp Format

```rust
fn format_timestamp(ts: i64, japanese: bool) -> String {
    if japanese {
        // "2026年01月22日 04:17"
        format!("{}年{:02}月{:02}日 {:02}:{:02}",
            year, month, day, hour, min)
    } else {
        format!("{:04}-{:02}-{:02} {:02}:{:02}",
            year, month, day, hour, min)
    }
}
```

---

## 6. IsometricGrid

2.5D grid for data mapped to spatial cells. Diamond-shaped cells rendered with half-block chars. Fixed camera angle. Per-cell color, height, and symbol.

### Visual Spec

```
       ▄▄▄▄▄▄▄                  ← top face (brighter)
      ▐███████▌
     ▐█▀▀▀▀▀▀█▌▐▌               ← cell with symbol
    ▐█▀ ◆ETH ▀█▌█▌
   ▐█▀▀▀▀▀▀▀▀▀▀█▌█▌             ← height encoded as vertical chars
  ▐██████████████▌█▌
   ▀██████████████▌
    ▀▀▀▀▀▀▀▀▀▀▀▀▀

ACTIVITY HEATMAP VIEW (territory visualization):
  Brighter cells = higher value. Color = category.
  Half-block precision for diagonal cell edges.
```

### Rust Struct

```rust
pub struct IsometricGrid {
    pub cells: Vec<Vec<IsoCell>>,
    pub grid_w: usize,
    pub grid_h: usize,
    camera_offset: (f64, f64),  // pan
}

pub struct IsoCell {
    pub value: f64,         // 0.0-1.0 → height
    pub color: Color,
    pub symbol: Option<char>,
    pub label: Option<String>,
}

impl DataPrimitive for IsometricGrid {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        // Project isometric grid to terminal coordinates
        // Cell (gx, gy) → screen (sx, sy) using isometric projection:
        //   sx = (gx - gy) * cell_w / 2
        //   sy = (gx + gy) * cell_h / 4
        // Render top face + optional side faces for height
        // Half-block chars for diagonal edges
    }

    fn tick(&mut self, _dt: f64) {}
}
```

### Projection Formula

```rust
fn iso_project(gx: f64, gy: f64, value: f64, cell_w: f64, cell_h: f64) -> (f64, f64) {
    let sx = (gx - gy) * cell_w * 0.5;
    let sy = (gx + gy) * cell_h * 0.25 - value * cell_h * 0.5;
    (sx, sy)
}
```

---

## 7. GlobeWireframe

Sphere via latitude/longitude lines, animated rotation. Half-block for filled arc segments. Surrounding data panels at configurable anchor positions. (Visual spec in `04-nerv-aesthetic.md` B10.)

### Rust Struct

```rust
pub struct GlobeWireframe {
    pub rotation: f64,              // current rotation angle (radians)
    pub rotation_speed: f64,        // rad/s
    pub ring_count: usize,          // latitude rings
    pub meridian_count: usize,      // longitude meridians
    pub panels: Vec<OrbitalPanel>,  // data panels at N/E/S/W
    pub grid_color: Color,
    pub fill_color: Color,
}

pub struct OrbitalPanel {
    pub anchor: CardinalDirection,
    pub label: String,
    pub value: String,
    pub color: Color,
}

pub enum CardinalDirection { North, East, South, West, NE, NW, SE, SW }

impl DataPrimitive for GlobeWireframe {
    fn tick(&mut self, dt: f64) {
        self.rotation = (self.rotation + self.rotation_speed * dt)
            % std::f64::consts::TAU;
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let cx = area.x + area.width / 2;
        let cy = area.y + area.height / 2;
        let rx = area.width as f64 * 0.4;
        let ry = area.height as f64 * 0.4;
        // Render latitude rings: ellipses at varying y offsets
        // Render meridians: sinusoidal curves, phase-shifted by rotation
        // Render panels at anchored positions with connection lines
    }
}
```

### Longitude Sinusoid

A meridian on a rotating globe projects as a sinusoidal curve:

```rust
fn meridian_x(phi: f64, theta: f64, rotation: f64) -> f64 {
    // phi = latitude (0..PI), theta = longitude offset
    let lon = theta + rotation;
    lon.cos() * phi.sin()  // x component (projected)
}
```

---

## 8. IrisVisualization

Concentric ring display. Configurable ring count, per-ring color/fill, arc segmentation. Inner ring most intense. (Visual spec in `04-nerv-aesthetic.md` B11.)

### Rust Struct

```rust
pub struct IrisVisualization {
    pub rings: Vec<IrisRing>,
    pub center_char: char,
    pub center_color: Color,
    pub rotation_speed: f64,
    rotation: f64,
}

pub struct IrisRing {
    pub radius: f64,        // 0.0-1.0 (fraction of min(w,h)/2)
    pub color: Color,
    pub segments: usize,    // 0 = full ring, 4/6/8 = segmented
    pub label: Option<String>,
    pub fill: RingFill,
}

pub enum RingFill {
    Solid,      // ─ │ ┌ ┐ └ ┘
    Dashed,     // ╌ ╎
    Dots,       // · · ·
}

impl DataPrimitive for IrisVisualization {
    fn tick(&mut self, dt: f64) {
        // Outermost ring rotates slowly
        self.rotation = (self.rotation + self.rotation_speed * dt)
            % std::f64::consts::TAU;
    }

    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let cx = area.x + area.width / 2;
        let cy = area.y + area.height / 2;
        // Render rings from outermost to innermost
        // For segmented rings: skip arc segments at evenly-spaced gaps
        // Place labels at gaps in segmented rings
        // Render center char at (cx, cy)
        let x = cx; let y = cy;
        buf[(x, y)].set_char(self.center_char).set_fg(self.center_color);
    }
}
```

### Default Ring Stack

```rust
impl IrisVisualization {
    pub fn at_field() -> Self {
        Self {
            rings: vec![
                IrisRing { radius: 1.0, color: Color::Rgb(48,40,52), segments: 0, label: Some("OPTICAL SCOPE UNIT".into()), fill: RingFill::Dots },
                IrisRing { radius: 0.75, color: Color::Rgb(122,80,96), segments: 8, label: None, fill: RingFill::Dashed },
                IrisRing { radius: 0.5, color: Color::Rgb(170,112,136), segments: 0, label: Some("AT FIELD ANALYSIS".into()), fill: RingFill::Solid },
                IrisRing { radius: 0.3, color: Color::Rgb(200,150,170), segments: 0, label: None, fill: RingFill::Solid },
            ],
            center_char: '●',
            center_color: Color::Rgb(200,190,170),  // bone
            rotation_speed: 0.1,
            rotation: 0.0,
        }
    }
}
```

---

## 9. DensityField

2D braille-resolution dot map. Value 0.0-1.0 per sub-cell. Each terminal cell contains a 2×4 braille dot matrix, giving 2× horizontal and 4× vertical resolution beyond standard character grid.

### Visual Spec

```
COGNITIVE LOAD VISUALIZATION:
  Low (0.0-0.2):              High (0.8-1.0):
  ⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀              ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
  ⠀⠁⠀⠀⠀⠂⠀⠁⠀⠀              ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
  ⠀⠀⠂⠁⠀⠀⠁⠀⠂⠀              ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿

  Braille Unicode:
  U+2800 (⠀) = no dots
  U+2801–U+28FF encode all dot combinations (8 dots per char, 256 values)
```

### Rust Struct

```rust
pub struct DensityField {
    pub data: Vec<f64>,     // flat array, width*height sub-cells
    pub sub_width: usize,   // = area.width * 2  (2 dots per cell wide)
    pub sub_height: usize,  // = area.height * 4 (4 dots per cell tall)
    pub color: Color,
    pub threshold: f64,     // sub-cells below this are empty
}

impl DensityField {
    pub fn from_fn(w: usize, h: usize, f: impl Fn(f64, f64) -> f64) -> Self {
        let mut data = vec![0.0; w * h];
        for y in 0..h {
            for x in 0..w {
                let nx = x as f64 / w as f64;
                let ny = y as f64 / h as f64;
                data[y * w + x] = f(nx, ny);
            }
        }
        Self { data, sub_width: w, sub_height: h, color: Color::Rgb(170,112,136), threshold: 0.15 }
    }
}

impl DataPrimitive for DensityField {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        for cy in 0..area.height {
            for cx in 0..area.width {
                // Sample 2×4 sub-cells for this terminal cell
                let mut bits: u8 = 0;
                for dot_x in 0..2_usize {
                    for dot_y in 0..4_usize {
                        let sx = cx as usize * 2 + dot_x;
                        let sy = cy as usize * 4 + dot_y;
                        if sx < self.sub_width && sy < self.sub_height {
                            let v = self.data[sy * self.sub_width + sx];
                            if v > self.threshold {
                                // Braille dot bit positions:
                                // dot_x=0: bits 0,1,2,6  dot_x=1: bits 3,4,5,7
                                let bit = match (dot_x, dot_y) {
                                    (0,0)=>0, (0,1)=>1, (0,2)=>2, (0,3)=>6,
                                    (1,0)=>3, (1,1)=>4, (1,2)=>5, (1,3)=>7,
                                    _ => unreachable!(),
                                };
                                bits |= 1 << bit;
                            }
                        }
                    }
                }
                if bits > 0 {
                    let ch = char::from_u32(0x2800 + bits as u32).unwrap_or('⠀');
                    buf[(area.x + cx, area.y + cy)].set_char(ch).set_fg(self.color);
                }
            }
        }
    }

    fn tick(&mut self, _dt: f64) {}
}
```

---

## 10. SequencerGrid

Multi-track horizontal event grid. Rows = tracks, cols = time steps. Each cell: active/inactive with color coding. From retro tracker aesthetic.

### Visual Spec

```
SEQUENCER (8 tracks × 16 steps):

  TRACK    ┊ 1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16
  ─────────┼──────────────────────────────────────────────────
  ECON ████┊ ██ ░░ ██ ░░ ██ ░░ ██ ░░ ██ ░░ ██ ░░ ██ ░░ ██ ░░
  EPIS ████┊ ██ ██ ░░ ░░ ██ ██ ░░ ░░ ██ ██ ░░ ░░ ██ ██ ░░ ░░
  STOC ████┊ ██ ░░ ░░ ██ ░░ ░░ ██ ░░ ░░ ██ ░░ ░░ ██ ░░ ░░ ██
  PAD-P ███┊ ██ ██ ██ ██ ░░ ░░ ░░ ░░ ██ ██ ██ ██ ░░ ░░ ░░ ░░
             ↑ current step (bright cursor)
```

### Rust Struct

```rust
pub struct SequencerGrid {
    pub tracks: Vec<SequencerTrack>,
    pub step_count: usize,
    pub current_step: usize,
    pub advance_on_tick: bool,
}

pub struct SequencerTrack {
    pub label: String,
    pub color: Color,
    pub steps: Vec<bool>,
    pub velocity: Vec<f64>,     // 0.0-1.0 per step (intensity)
}

impl SequencerGrid {
    pub fn advance(&mut self) {
        self.current_step = (self.current_step + 1) % self.step_count;
    }
}

impl DataPrimitive for SequencerGrid {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let step_w = (area.width as usize - 12) / self.step_count;
        for (ti, track) in self.tracks.iter().enumerate() {
            let y = area.y + ti as u16 + 2; // +2 for header
            // Render track label (left, 10 chars)
            buf.set_string(area.x, y, &format!("{:10}", &track.label),
                Style::default().fg(track.color));
            // Render steps
            for (si, &active) in track.steps.iter().enumerate() {
                let x = area.x + 11 + (si * step_w) as u16;
                let is_current = si == self.current_step;
                let ch = if active { '█' } else { '░' };
                let color = if is_current {
                    scale_color(track.color, 1.5)
                } else if active {
                    track.color
                } else {
                    Color::Rgb(40, 36, 44)
                };
                buf[(x, y)].set_char(ch).set_fg(color);
            }
        }
    }

    fn tick(&mut self, _dt: f64) {
        if self.advance_on_tick {
            self.advance();
        }
    }
}
```

---

## 11. PersistenceDiagram

Scatter plot of topological features from persistent homology. Each point is a (birth, death) pair from a Vietoris-Rips filtration over the market's observation space. Points near the diagonal are noise. Points far from the diagonal are significant structural features. The diagonal itself renders as a reference line.

### Visual Spec

```
STANDARD VIEW (40x16 braille canvas):

  death ↑
   1.0  │              ◇
        │         ●         ◇
        │    ●  ●     ◇
   0.5  │  ●  ●  ●      ◇
        │ ● ● ●   ◇
        │●●●·····················   ← diagonal (birth = death)
   0.0  └────────────────────────→ birth
        0.0           0.5       1.0

LEGEND:
  ●  H_0 (connected components / clusters)
  ◇  H_1 (loops / cycles)
  ■  H_2 (voids)

REFERENCE OVERLAY:
  Previous diagram renders as dim shadow points in rose_deep.
  Current diagram overlays in full brightness.
  The delta between them is the topological change since last gamma tick.
```

### Rust Struct

```rust
pub struct PersistenceDiagram {
    pub features: Vec<TopologicalFeature>,
    pub reference_features: Vec<TopologicalFeature>,  // previous diagram for comparison
    pub axis_range: (f64, f64),                       // birth/death axis bounds
    pub show_diagonal: bool,
    pub show_reference: bool,
}

pub struct TopologicalFeature {
    pub birth: f64,
    pub death: f64,
    pub dimension: HomologyDimension,
    pub persistence: f64,  // death - birth (precomputed)
}

pub enum HomologyDimension {
    H0,  // clusters — rendered as ●
    H1,  // loops — rendered as ◇
    H2,  // voids — rendered as ■
}

impl DataPrimitive for PersistenceDiagram {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let w = area.width as f64;
        let h = area.height as f64;
        let (lo, hi) = self.axis_range;
        let span = hi - lo;

        // Render diagonal line in text_ghost (braille dots along birth == death)
        if self.show_diagonal {
            for i in 0..area.width.min(area.height) {
                let x = area.x + i;
                let y = area.bottom() - 1 - i as u16;
                buf[(x, y)].set_char('·').set_fg(Color::Rgb(48, 40, 52)); // text_ghost
            }
        }

        // Render reference features as dim shadow
        if self.show_reference {
            for feat in &self.reference_features {
                let (sx, sy) = self.project(feat, w, h, lo, span);
                let x = area.x + sx as u16;
                let y = area.bottom() - 1 - sy as u16;
                if area.contains(x, y) {
                    let ch = feat.dimension.glyph();
                    buf[(x, y)].set_char(ch).set_fg(Color::Rgb(88, 56, 72)); // rose_deep
                }
            }
        }

        // Render current features at full brightness
        for feat in &self.features {
            let (sx, sy) = self.project(feat, w, h, lo, span);
            let x = area.x + sx as u16;
            let y = area.bottom() - 1 - sy as u16;
            if area.contains(x, y) {
                let ch = feat.dimension.glyph();
                let color = feat.dimension.color();
                // Pulse at heartbeat if high arousal
                let brightness = if ctx.pad.arousal > 0.7 {
                    0.7 + 0.3 * (ctx.heartbeat_phase * std::f64::consts::TAU).sin().abs()
                } else {
                    1.0
                };
                buf[(x, y)].set_char(ch).set_fg(scale_color(color, brightness));
            }
        }
    }

    fn tick(&mut self, _dt: f64) {}
}

impl HomologyDimension {
    fn glyph(&self) -> char {
        match self {
            Self::H0 => '●',
            Self::H1 => '◇',
            Self::H2 => '■',
        }
    }

    fn color(&self) -> Color {
        match self {
            Self::H0 => Color::Rgb(170, 112, 136),  // rose
            Self::H1 => Color::Rgb(204, 164, 80),    // warning
            Self::H2 => Color::Rgb(200, 150, 170),   // rose_bright
        }
    }
}
```

### Character Mapping

| Element | Character | Color |
|---------|-----------|-------|
| H_0 feature (current) | `●` | `rose` |
| H_0 feature (reference) | `●` | `rose_deep` |
| H_1 feature (current) | `◇` | `warning` |
| H_1 feature (reference) | `◇` | `bone_dim` |
| H_2 feature (current) | `■` | `rose_bright` |
| H_2 feature (reference) | `■` | `text_dim` |
| Diagonal | `·` | `text_ghost` |

### PAD Modulation

- **High arousal (>0.7):** Feature points pulse at heartbeat frequency. The scatter field breathes.
- **Low pleasure (<0.3):** Diagonal line thickens (rendering 3 braille rows instead of 1). More noise sensitivity visible, the boundary between signal and noise shifts.
- **Low dominance (<0.3):** Reference diagram shadow brightens from `rose_deep` to `rose_dim`. The Golem is comparing itself more anxiously to the baseline.

### Phase Degradation

| Phase | Effect |
|-------|--------|
| Thriving | Full rendering, all three homology dimensions |
| Stable | Standard |
| Conservation | H_2 features suppressed. Braille resolution halved |
| Declining | Only H_0 rendered. Feature points flicker randomly |
| Terminal | Frozen. Last diagram persists as phosphor ghost |

**Update frequency:** Every gamma tick (5-15s). Typical feature count: 20-80 points.

---

## 12. SimilarityLandscape

2D topographic heatmap of high-dimensional similarity projected to terminal space. Data sources include HDC cosine similarity between market states, sheaf consistency scores from Clade observations, and Betti curve fingerprints. Peaks represent clusters of similar states. Valleys represent dissimilar or transitional regions. Ridges connect related clusters.

### Visual Spec

```
TOPOGRAPHIC HEATMAP (48x20 braille canvas):

  ░░░░░░▒▒▓▓██▓▓▒▒░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
  ░░░░▒▒▓▓████████▓▓▒▒░░░░░░░░░░░░░░░▒▒▒▒░░░░░░░░
  ░░▒▒▓▓██████████████▓▓▒▒░░░░░░░░▒▒▓▓██▓▓▒▒░░░░░░
  ░░▓▓████████████████████▓▓░░░░░▒▓▓██████▓▓▒░░░░░░
  ░░▒▒▓▓██████████████▓▓▒▒░░░░░░▒▒▓▓████▓▓▒▒░░░░░░
  ░░░░▒▒▓▓████████▓▓▒▒░░░░░░░░░░░░▒▒▓▓▓▓▒▒░░░░░░░░
  ░░░░░░▒▒▓▓▓▓▓▓▒▒░░░░░░░░░░░░░░░░░░▒▒░░░░░░░░░░░░
         ↑ dominant peak           ↑ secondary peak
                    ◆ ← cursor (current market state position)

COLORMAP (ROSEDUST):
  valley: bg_void (darkest)
  slope:  rose_deep → rose_dim
  ridge:  rose
  peak:   rose_bright → bone (brightest)
```

### Rust Struct

```rust
pub struct SimilarityLandscape {
    pub heightmap: Vec<Vec<f64>>,       // 2D grid, values 0.0-1.0
    pub grid_width: usize,
    pub grid_height: usize,
    pub cursor_pos: Option<(f64, f64)>, // current state position in landscape coords
    pub pan_offset: (f64, f64),         // view pan for locked navigation
    pub data_source: LandscapeSource,
    pub colormap: ThermalColormap,      // reuses ThermalField's colormap enum
}

pub enum LandscapeSource {
    HdcSimilarity,       // HDC codebook projected to 2D
    SheafConsistency,    // Clade spatio-temporal consistency
    BettiFingerprint,    // Betti curve similarity
    SomaticMap,          // somatic marker affect clustering
}

impl DataPrimitive for SimilarityLandscape {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let scale_x = self.grid_width as f64 / area.width as f64;
        let scale_y = self.grid_height as f64 / area.height as f64;

        // Render heightmap as thermal field with ROSEDUST colormap
        for row in 0..area.height {
            let gy = ((row as f64 + self.pan_offset.1) * scale_y) as usize;
            for col in 0..area.width {
                let gx = ((col as f64 + self.pan_offset.0) * scale_x) as usize;
                let v = self.heightmap
                    .get(gy.min(self.grid_height - 1))
                    .and_then(|r| r.get(gx.min(self.grid_width - 1)))
                    .copied()
                    .unwrap_or(0.0);
                let bg_idx = (v * 4.0) as usize;
                let ch = THERMAL_BG[bg_idx.min(4)];
                let color = self.colormap.sample(v);
                buf[(area.x + col, area.y + row)].set_char(ch).set_fg(color);
            }
        }

        // Render cursor as bright marker
        if let Some((cx, cy)) = self.cursor_pos {
            let sx = ((cx - self.pan_offset.0) / scale_x) as u16;
            let sy = ((cy - self.pan_offset.1) / scale_y) as u16;
            let x = area.x + sx;
            let y = area.y + sy;
            if area.contains(x, y) {
                buf[(x, y)].set_char('◆').set_fg(Color::Rgb(200, 190, 170)); // bone
            }
        }
    }

    fn tick(&mut self, _dt: f64) {}
}
```

### Interaction

When locked (selected for navigation):
- Arrow keys pan the view (`pan_offset` shifts)
- Enter on a peak opens a detail modal listing which states/agents cluster there
- The cursor dot (`◆` in `bone`) tracks the current market state's position

### PAD Modulation

- **High arousal (>0.7):** Landscape updates more frequently (shorter projection window). The terrain shifts faster.
- **Low pleasure (<0.3):** Valleys deepen visually (wider color range in the cool end of the colormap).
- **High dominance (>0.7):** Cursor dot brightens. Landscape is more stable (longer projection window, less jitter between frames).

### Phase Degradation

| Phase | Effect |
|-------|--------|
| Thriving | Full resolution, smooth color gradients |
| Stable | Standard |
| Conservation | Resolution halved, updates at delta frequency only |
| Declining | Grayscale mode, rose desaturated |
| Terminal | Frozen at last computed state |

**Update frequency:** Theta frequency (30-120s). Projection via UMAP or PCA from high-dimensional source.

---

## 13. WassersteinRiver

Flowing time-series ribbon showing the Wasserstein distance between successive persistence diagrams. The ribbon width encodes distance: narrow means topological stability (successive diagrams are similar), wide means topological change (successive diagrams differ). The ribbon flows right to left, current moment at the right edge.

Dual-purpose: also renders somatic interference when multiple somatic markers fire simultaneously. The ribbon splits into colored sub-streams showing how competing gut feelings converge or diverge over time.

### Visual Spec

```
STABLE TOPOLOGY (narrow ribbon):
  ──────────────────────────────────────╌╌╌╌─────
  ────────────────────────────────────────────────

REGIME SHIFT (widening ribbon):
  ░░░░▒▒▒▒▓▓▓▓████████████████▓▓▓▓▒▒▒▒░░░░
  ░░▒▒▓▓████████████████████████████▓▓▒▒░░░░
  ░░░░▒▒▓▓████████████████████▓▓▒▒░░░░░░░░░░
  ────────────────────╌╌╌╌╌─────────────────
                    ↑ flood event

SOMATIC INTERFERENCE (split sub-streams):
  ▒▒▒▒▓▓▓▓██████▓▓▒▒░░░░░░░░░░░░░░░░   ← approach (bone_dim)
  ────────────────────────────────────
  ░░░░▒▒▓▓████████████▓▓▒▒░░░░░░░░░░   ← avoidance (rose_bright)

  Sub-streams merge when markers resolve. Diverge when they conflict.
```

### Rust Struct

```rust
pub struct WassersteinRiver {
    pub distances: VecDeque<f64>,     // W_p values over time, 0.0-1.0 normalized
    pub window: usize,                // how many ticks of history to show
    pub max_width: f64,               // maximum ribbon height as fraction of area height
    pub mode: RiverMode,
}

pub enum RiverMode {
    /// Single ribbon showing topological distance
    Topological,
    /// Split into sub-streams by somatic marker affect
    SomaticInterference {
        streams: Vec<SomaticStream>,
    },
}

pub struct SomaticStream {
    pub label: String,
    pub distances: VecDeque<f64>,
    pub color: Color,
    pub valence: SomaticValence,
}

pub enum SomaticValence {
    Approach,   // warm sub-stream (bone_dim)
    Avoidance,  // cool sub-stream (rose_bright)
    Neutral,    // standard sub-stream (rose)
}

impl DataPrimitive for WassersteinRiver {
    fn render(&self, area: Rect, buf: &mut Buffer, ctx: &AnimationContext) {
        let center_y = area.y + area.height / 2;
        let max_half = (area.height as f64 * self.max_width * 0.5) as u16;

        match &self.mode {
            RiverMode::Topological => {
                for (i, &dist) in self.distances.iter().rev()
                    .take(area.width as usize).enumerate()
                {
                    let x = area.right() - 1 - i as u16;
                    let half_width = (dist * max_half as f64).round() as u16;
                    let color = self.distance_color(dist);

                    // Render ribbon centered on center_y
                    for dy in 0..=half_width {
                        let chars = if dy == half_width { '░' }
                            else if dy > half_width / 2 { '▒' }
                            else { '█' };
                        if center_y + dy < area.bottom() {
                            buf[(x, center_y + dy)].set_char(chars).set_fg(color);
                        }
                        if center_y >= dy && center_y - dy >= area.y {
                            buf[(x, center_y - dy)].set_char(chars).set_fg(color);
                        }
                    }
                }
            }
            RiverMode::SomaticInterference { streams } => {
                // Render each sub-stream as its own thin ribbon
                // stacked vertically within the area
                let stream_height = area.height / streams.len().max(1) as u16;
                for (si, stream) in streams.iter().enumerate() {
                    let stream_center = area.y + stream_height * si as u16
                        + stream_height / 2;
                    let stream_max = (stream_height as f64 * 0.4) as u16;
                    for (i, &dist) in stream.distances.iter().rev()
                        .take(area.width as usize).enumerate()
                    {
                        let x = area.right() - 1 - i as u16;
                        let hw = (dist * stream_max as f64).round() as u16;
                        for dy in 0..=hw {
                            let ch = if dy == hw { '░' } else { '▓' };
                            if stream_center + dy < area.bottom() {
                                buf[(x, stream_center + dy)]
                                    .set_char(ch).set_fg(stream.color);
                            }
                            if stream_center >= dy + area.y {
                                buf[(x, stream_center - dy)]
                                    .set_char(ch).set_fg(stream.color);
                            }
                        }
                    }
                }
            }
        }
    }

    fn tick(&mut self, _dt: f64) {}
}

impl WassersteinRiver {
    pub fn push(&mut self, distance: f64) {
        if self.distances.len() >= self.window {
            self.distances.pop_front();
        }
        self.distances.push_back(distance.clamp(0.0, 1.0));
    }

    fn distance_color(&self, dist: f64) -> Color {
        if dist < 0.2 {
            Color::Rgb(122, 80, 96)    // rose_dim — quiet stream
        } else if dist < 0.6 {
            Color::Rgb(170, 112, 136)  // rose — visible flow
        } else {
            Color::Rgb(200, 190, 170)  // bone — flood
        }
    }
}
```

### Character Mapping

| Ribbon region | Character | Meaning |
|---------------|-----------|---------|
| Core (center) | `█` | Highest distance density |
| Mid | `▓` `▒` | Transition zone |
| Edge | `░` | Ribbon boundary |
| Narrow baseline | `─` `╌` | Minimal distance, stable topology |

### Color Mapping

| Distance range | Color | Meaning |
|----------------|-------|---------|
| 0.0 - 0.2 | `rose_dim` | Quiet stream, topological stability |
| 0.2 - 0.6 | `rose` | Visible flow, moderate change |
| 0.6 - 1.0 | `rose_bright` → `bone` | Flood, regime shift underway |

Somatic interference sub-streams:

| Valence | Color | Meaning |
|---------|-------|---------|
| Approach | `bone_dim` | Warm sub-stream, positive markers |
| Avoidance | `rose_bright` | Cool sub-stream, negative markers |
| Neutral | `rose` | Standard sub-stream |

### PAD Modulation

- **High arousal (>0.7):** Flow speed increases (more data points per second enter from the right edge). The river rushes.
- **Low pleasure (<0.3):** Color shifts cooler. The flood threshold drops (smaller distances produce wider ribbons). The Golem is more sensitive to change.
- **High dominance (>0.7):** River is more contained (max width caps at 70% of available height). The Golem constrains its reaction.

### Phase Degradation

| Phase | Effect |
|-------|--------|
| Thriving | Full width range, smooth color gradient |
| Stable | Standard |
| Conservation | Width range compressed, muted colors |
| Declining | River fragments into discrete blocks (flow breaks into staccato pulses) |
| Terminal | River dries up. Only occasional bright flashes where distance spikes |

**Update frequency:** Every gamma tick (5-15s). Computed as W_p(D_t, D_{t-1}) between successive persistence diagrams.

---

## Design Token Summary

```yaml
visualization_primitives:
  waveform:
    chars: "▁▂▃▄▅▆▇█"
    phosphor_decay: 0.2
    scroll_direction: "right-edge = most recent"

  thermal_field:
    bg_chars: " ░▒▓█"
    fg_chars: "·+✦"
    colormaps: ["RoseDust", "Viridis", "Inferno"]

  radar_display:
    center_char: "╋"
    arm_char: "▸"
    ring_chars: "─│╌╎"

  force_graph:
    edge_styles: ["═══", "───", "┄┄┄", "╳╳╳", "†┄†"]
    node_glyphs: "◆ ◇ ● ⚡ ⚠ ∴ †"
    damping: 0.92

  timeline_ribbon:
    fill_chars: "█▓▒░"
    current_marker: "┃"
    timestamp_format: "japanese optional"

  isometric_grid:
    projection: "standard 2:1 isometric"
    edge_chars: "▀▄▐▌"

  globe_wireframe:
    grid_chars: "╱╲─│"
    fill_chars: "▀▄"
    rotation_speed: 0.5

  iris_visualization:
    gradient: ["text_ghost", "text_dim", "rose_dim", "rose", "rose_bright"]
    center_char: "●"

  density_field:
    encoding: "braille U+2800 family"
    resolution_multiplier: "2x4 per terminal cell"

  sequencer_grid:
    active_char: "█"
    inactive_char: "░"
    current_highlight: "1.5x brightness"

  persistence_diagram:
    feature_glyphs: "● ◇ ■"
    diagonal_char: "·"
    dimensions: "H_0 H_1 H_2"
    update_frequency: "gamma tick (5-15s)"

  similarity_landscape:
    colormap: "ROSEDUST (bg_void → rose_deep → rose → bone)"
    cursor_char: "◆"
    resolution: "48x20 standard, 30x12 compact"
    update_frequency: "theta tick (30-120s)"

  wasserstein_river:
    core_chars: "█▓▒░"
    baseline_chars: "─╌"
    flow_direction: "right-to-left"
    update_frequency: "gamma tick (5-15s)"
```

---

## Signal-to-Primitive Mapping

Every data signal from the chain intelligence, technical analysis, HDC, and somatic subsystems maps to a specific visualization primitive. This table is the canonical reference for which primitive renders which signal, at what frequency, and at what rendering cost.

### Chain Intelligence Signals

| Signal | Type | Primitive | Screen Placement | Update Freq | Rendering Cost |
|--------|------|-----------|-----------------|-------------|----------------|
| `topology_signal` (Wasserstein distance) | f32, 0.0-1.0 | WaveformDisplay §1 | FATE > Chain Intelligence, time-series | Gamma (5-15s) | Low |
| `betti_0` (connected components) | u16 | FlashNumber (widget #1) | FATE > Chain Intelligence, inline | Gamma | Minimal |
| `betti_1` (loops) | u16 | FlashNumber (widget #1) | FATE > Chain Intelligence, inline | Gamma | Minimal |
| Betti curve (β_k vs ε) | fn(f32)->u16 | Sparkline (widget #8) | FATE > Chain Intelligence, per-dimension | Gamma | Low |
| Persistence diagram (birth/death pairs) | Vec<(f32, f32)> | PersistenceDiagram §11 | FATE > Chain Intelligence, braille scatter | Gamma | Medium |
| Regime transition distance | f32 time-series | WaveformDisplay §1 + FlashWidget (#18) | HEARTH > Signals | Gamma | Low |
| Topological regime signature | enum (calm/trending/volatile/crisis) | UnitArray (widget #6) | FATE > Chain Intelligence, regime indicator | Gamma | Minimal |
| Wasserstein distance ribbon | f32 time-series | WassersteinRiver §13 | FATE > Chain Intelligence, flowing ribbon | Gamma | Medium |

### Technical Analysis Signals

| Signal | Type | Primitive | Screen Placement | Update Freq | Rendering Cost |
|--------|------|-----------|-----------------|-------------|----------------|
| TA pattern detection state | per-pattern enum | UnitArray (widget #6) | FATE > Technical Analysis, signal battery | Gamma | Low |
| Gut response similarity | f32, 0.0-1.0 | ConfidenceBar (widget #4) | FATE > Technical Analysis, per-marker | Gamma | Low |
| Somatic map (PAD query) | (f32, f32, f32) | SimilarityLandscape §12 | FATE > Technical Analysis, 2D projection | Theta (30-120s) | High |
| Marker strength | f32, 0.0-1.0 | ConfidenceBar (widget #4) | FATE > Technical Analysis, per-marker bar | Gamma | Low |
| Somatic interference | Vec<(pattern, affect)> | WassersteinRiver §13 (somatic mode) | SOMA > Affect, competing markers | Gamma | Medium |
| Anti-somatic markers (traps) | flagged patterns | UnitArray (widget #6) with breach state | FATE > Technical Analysis, flagged cells | Gamma | Low |

### HDC Signals

| Signal | Type | Primitive | Screen Placement | Update Freq | Rendering Cost |
|--------|------|-----------|-----------------|-------------|----------------|
| HDC codebook similarity | projected 2D matrix | SimilarityLandscape §12 | FATE > Technical Analysis, somatic map | Theta | High |
| Pattern hypervector similarity | f32, 0.0-1.0 | ThermalField §2 | MIND > Grimoire, entry proximity | Delta (2-10min) | Medium |

### Sheaf Consistency / Integration Signals

| Signal | Type | Primitive | Screen Placement | Update Freq | Rendering Cost |
|--------|------|-----------|-----------------|-------------|----------------|
| Consistency score | f32, 0.0-1.0 | MortalityGauge (widget #2) variant | SOMA > Epistemic, FATE > Chain Intelligence | Theta | Low |
| Contradiction dimension | u8 | FlashNumber (widget #1) | SOMA > Epistemic, inline | Theta | Minimal |
| Per-edge disagreement | Vec<f32> | UnitArray (widget #6) overlay | MIND > Pipeline, module connections | Theta | Low |
| Spatio-temporal consistency | f32, 0.0-1.0 | SimilarityLandscape §12 | WORLD > Clade, consistency field | Theta | High |
| Phi(t) | f32 | WaveformDisplay §1 | MIND > Pipeline, slow trace | Delta | Low |
| MIB (bipartition) | (Set, Set) | UnitArray (widget #6) split coloring | MIND > Pipeline, module array | Delta | Low |

### Rendering Cost Key

| Cost | CPU per frame | Suitable for |
|------|--------------|--------------|
| Minimal | <0.1ms | Inline values, counters |
| Low | 0.1-0.5ms | Waveforms, sparklines, unit arrays |
| Medium | 0.5-2ms | Braille scatter plots, flowing ribbons |
| High | 2-5ms | 2D projections, thermal fields with UMAP |

All primitives scale rendering cost with terminal size. Compact mode (-) halves the resolution of Medium and High cost primitives. Conservation phase further reduces update frequency for High cost primitives to delta-only updates.

---

*⌈ every data shape has its own visual language ⌋*
*║▒░ ＢＡＲＤＯ ░▒║*
