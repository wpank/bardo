# bardo-terminal

## What It Is

`bardo-terminal` is the workspace terminal binary and its crate-local widget library. The widgets are small `ratatui` renderers that screens compose directly into the frame buffer. They are deterministic for a given input struct and area, and the data-heavy widgets render through references so their backing collections do not have to move each frame.

## Features

- Dense braille sparklines for compact traces and PAD timelines
- Phase-colored vitality gauges plus scalar confidence and accuracy gauges
- Viridis-inspired pheromone heatmaps with layer tinting and pulse flashes
- Tick-based timeline ribbons with event glyphs and severity coloring
- Bounded event feeds with newest-first filtering
- Horizontal tab strips, a reusable status bar widget, cursor-driven lists, and a floating help overlay
- Crate-local widget surface accessed as `crate::widgets`
- Designed to fit the 60 fps render loop used by the terminal app

## Getting Started

Run the terminal:

```bash
cargo run -p bardo-terminal
```

Run the widget tests:

```bash
cargo test -p bardo-terminal -- widgets --nocapture
```

Use the widgets from a screen render implementation:

```rust
use crate::widgets::*;

let sparkline = BrailleSparkline {
    data: (0..40).map(|n| (n as f64).sin().abs()).collect(),
    max_value: 1.0,
    color: crate::palette::ROSE,
    label: Some("activity".to_string()),
};
frame.render_widget(sparkline, chart_area);
```

## Configuration

There is no widget-specific configuration file or environment variable set. The widgets inherit the terminal app's runtime settings and capabilities:

- `RUST_LOG` controls process-level tracing
- terminal size determines how much content each widget can render
- Unicode and box-drawing support affect braille, glyph, and border rendering
- color support affects the ROSEDUST palette and heatmap gradient fidelity

## API

The widget layer is internal to the binary crate, so the visible surface is `crate::widgets`. The module re-exports are crate-local and are intended for screen code inside `bardo-terminal`, not for a standalone library API.

### Widget Surface

| Widget | Ownership | What it renders |
| --- | --- | --- |
| `BrailleSparkline` | by value | Up to 80 samples as a compact braille trace. |
| `VitalityGauge` | by value | Phase-colored vitality with a placeholder `MockPhase`. |
| `ConfidenceGauge` | by value | Scalar 0..1 confidence with amber-to-green thresholds. |
| `AccuracyGauge` | by value | Scalar 0..1 accuracy with the same threshold scheme. |
| `PheromoneHeatmap` | by value | A Viridis-inspired grid with optional pulse flashes. |
| `TimelineRibbon` | by value | A tick-based event ribbon with glyphs and severity. |
| `EventFeed` | by reference | Bounded, newest-first log output with substring filtering. |
| `TabBar` | by value | A horizontal tab strip with an active highlight. |
| `StatusBar` | by value | A single-row footer-style status line for screens that want one. |
| `ScrollableList` | by reference | A cursor-driven list with filtered item views. |
| `KeyHelpOverlay` | by reference | A centered help box that disappears when hidden. |

### Data Shapes

| Type | Purpose |
| --- | --- |
| `MockPhase` | Placeholder vitality phase until live mortality state arrives. |
| `PheromoneLayer` | Heatmap layer tag for `Threat`, `Opportunity`, or `Wisdom`. |
| `RibbonEventType` | Timeline event category and glyph/color source. |
| `FeedLevel` | Feed severity label and color source. |
| `FeedEntry` | One log row with tick, level, and message text. |
| `TimelineEvent` | One ribbon event with tick and severity. |
| `KeyBinding` | One help row with a key chord and description. |

### Rendering Patterns

Use value-owned widgets when the screen can pass an owned data struct into `render_widget`:

```rust
let sparkline = BrailleSparkline {
    data: samples,
    max_value: 1.0,
    color: crate::palette::ROSE,
    label: Some("activity".to_string()),
};
frame.render_widget(sparkline, chart_area);
```

Use reference-owned widgets when the screen should keep the backing collection in place between frames:

```rust
frame.render_widget(&feed, feed_area);
frame.render_widget(&list, list_area);
frame.render_widget(&overlay, overlay_area);
```

`EventFeed`, `ScrollableList`, and `KeyHelpOverlay` implement `Widget` for references so the caller retains ownership of the underlying collections.

## Usage Examples

### Render a mixed dashboard row

```rust
let vitality = VitalityGauge {
    value: 0.76,
    label: "VITALITY".into(),
    phase: MockPhase::Stable,
};
let confidence = ConfidenceGauge {
    value: 0.64,
    label: "CONF".into(),
};
let accuracy = AccuracyGauge {
    value: 0.71,
    label: "ACC".into(),
};

frame.render_widget(vitality, Rect::new(0, 0, 12, 2));
frame.render_widget(confidence, Rect::new(12, 0, 12, 2));
frame.render_widget(accuracy, Rect::new(24, 0, 12, 2));

frame.render_widget(
    PheromoneHeatmap {
        grid: signal_grid,
        width: 12,
        height: 8,
        layer: PheromoneLayer::Threat,
        pulse_cells: vec![(1, 2)],
    },
    heatmap_area,
);
```

### Render reference-owned widgets

```rust
let mut feed = EventFeed::new(1000);
feed.push(FeedEntry {
    tick: tick_count,
    level: FeedLevel::Warn,
    message: "trade delayed".into(),
});
frame.render_widget(&feed, feed_area);

let mut list = ScrollableList::new(vec!["alpha".into(), "beta".into(), "gamma".into()]);
list.filter = Some("a".into());
frame.render_widget(&list, list_area);
```

### Show the help overlay

```rust
let overlay = KeyHelpOverlay {
    bindings: vec![
        KeyBinding {
            key: "?".into(),
            description: "toggle help".into(),
        },
        KeyBinding {
            key: "q".into(),
            description: "quit".into(),
        },
    ],
    visible: show_help,
};
frame.render_widget(&overlay, frame.area());
```

## Architecture

`bardo-terminal` keeps the widget layer private to the binary crate and routes screen rendering through `crate::widgets`. The implementation matches the 60 fps terminal model described in the product spec: screens assemble simple data structs, widgets render into the provided buffer, and the caller owns the app state.

- Pure render widgets consume `self` on render
- Data-bearing widgets that should remain in place render through `&EventFeed`, `&ScrollableList`, and `&KeyHelpOverlay`
- The sparkline, heatmap, and timeline widgets map directly to the custom widget categories in the Styx terminal spec
- The tab strip and status bar are reusable chrome widgets designed for consistent screen chrome across the terminal app

## Spec References

- Custom widget catalogue and usage notes: [`prd2/20-styx/05-tui-experience.md` §5](../../../prd2/20-styx/05-tui-experience.md)
- Braille sparkline, heatmap, and timeline widget requirements: [`prd2/20-styx/05-tui-experience.md`](../../../prd2/20-styx/05-tui-experience.md)
- Ambient pulse and heartbeat-driven motion vocabulary: [`prd2/13-runtime/19-cinematic-system.md` §3.2](../../../prd2/13-runtime/19-cinematic-system.md)
