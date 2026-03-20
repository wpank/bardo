//! Timeline ribbon widget for event markers over a tick window.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::palette::{BONE, BONE_DIM, BORDER, DREAM, ROSE_BRIGHT, SUCCESS, WARNING};

fn in_bounds(buf: &Buffer, x: u16, y: u16) -> bool {
    let area = buf.area();
    x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom()
}

/// Event glyph class on the ribbon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RibbonEventType {
    /// Trade event.
    TradeExecuted,
    /// Dream event.
    DreamStarted,
    /// Phase change marker.
    PhaseChange,
    /// Anomaly marker.
    Anomaly,
    /// Death marker.
    Death,
}

impl RibbonEventType {
    /// Returns the glyph used to render this event kind.
    pub(crate) const fn glyph(self) -> char {
        match self {
            Self::TradeExecuted => '▲',
            Self::DreamStarted => '◌',
            Self::PhaseChange => '◆',
            Self::Anomaly => '!',
            Self::Death => '✕',
        }
    }

    /// Returns the display color for this event kind.
    pub(crate) const fn color(self) -> Color {
        match self {
            Self::TradeExecuted => SUCCESS,
            Self::DreamStarted => DREAM,
            Self::PhaseChange => BONE,
            Self::Anomaly => WARNING,
            Self::Death => ROSE_BRIGHT,
        }
    }
}

/// One event on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineEvent {
    /// Event tick.
    pub(crate) tick: u64,
    /// Event type.
    pub(crate) event_type: RibbonEventType,
    /// Brightness/severity modulation.
    pub(crate) severity: u8,
}

/// Horizontal event ribbon over a tick window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimelineRibbon {
    /// TODO Plan 70a: connect to the `golem_core::GolemEvent` stream.
    pub(crate) events: Vec<TimelineEvent>,
    /// Tick span covered by the ribbon.
    pub(crate) window_ticks: u64,
    /// Rightmost tick displayed.
    pub(crate) current_tick: u64,
}

impl Widget for TimelineRibbon {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let width = area.width as u64;
        let ticks_per_cell = (self.window_ticks.max(1) + width.saturating_sub(1)) / width.max(1);
        let start_tick = self.current_tick.saturating_sub(self.window_ticks);
        let y = area.y;

        for col in 0..area.width {
            let x = area.x + col;
            if !in_bounds(buf, x, y) {
                continue;
            }

            let cell_start_tick = start_tick + col as u64 * ticks_per_cell;
            let cell_end_tick = cell_start_tick + ticks_per_cell.max(1);
            let is_current = col == area.width.saturating_sub(1);
            let event_in_cell = self
                .events
                .iter()
                .filter(|event| event.tick >= cell_start_tick && event.tick < cell_end_tick)
                .max_by_key(|event| event.severity);

            let cell = buf.get_mut(x, y);
            if let Some(event) = event_in_cell {
                cell.set_char(event.event_type.glyph());
                cell.set_style(
                    Style::default()
                        .fg(event.event_type.color())
                        .add_modifier(Modifier::BOLD),
                );
            } else {
                cell.set_char(if is_current { '┤' } else { '─' });
                cell.set_style(Style::default().fg(if is_current { BONE_DIM } else { BORDER }));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ribbon_event_type_uses_expected_glyphs() {
        assert_eq!(RibbonEventType::TradeExecuted.glyph(), '▲');
        assert_eq!(RibbonEventType::DreamStarted.glyph(), '◌');
        assert_eq!(RibbonEventType::PhaseChange.glyph(), '◆');
        assert_eq!(RibbonEventType::Anomaly.glyph(), '!');
        assert_eq!(RibbonEventType::Death.glyph(), '✕');
    }

    #[test]
    fn ribbon_event_type_uses_expected_colors() {
        assert_eq!(RibbonEventType::TradeExecuted.color(), SUCCESS);
        assert_eq!(RibbonEventType::DreamStarted.color(), DREAM);
        assert_eq!(RibbonEventType::PhaseChange.color(), BONE);
        assert_eq!(RibbonEventType::Anomaly.color(), WARNING);
        assert_eq!(RibbonEventType::Death.color(), ROSE_BRIGHT);
    }

    #[test]
    fn timeline_ribbon_renders_events_with_severity_break_ties() {
        use ratatui::buffer::Buffer;

        let mut buf = Buffer::empty(ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 1,
        });

        let ribbon = TimelineRibbon {
            events: vec![
                TimelineEvent {
                    tick: 5,
                    event_type: RibbonEventType::TradeExecuted,
                    severity: 10,
                },
                TimelineEvent {
                    tick: 5,
                    event_type: RibbonEventType::Death,
                    severity: 255, // Higher severity should win
                },
            ],
            window_ticks: 100,
            current_tick: 100,
        };

        let area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 1,
        };

        ribbon.render(area, &mut buf);

        // The cell containing tick 5 should show Death glyph (higher severity)
        let cell = buf.get(0, 0);
        assert_eq!(cell.symbol(), "✕"); // Death glyph
    }

    #[test]
    fn timeline_ribbon_renders_current_tick_marker() {
        use ratatui::buffer::Buffer;

        let mut buf = Buffer::empty(ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 5,
            height: 1,
        });

        let ribbon = TimelineRibbon {
            events: vec![],
            window_ticks: 100,
            current_tick: 100,
        };

        let area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 5,
            height: 1,
        };

        ribbon.render(area, &mut buf);

        // Rightmost cell should show '┤' for current tick
        let rightmost_cell = buf.get(4, 0);
        assert_eq!(rightmost_cell.symbol(), "┤");
    }
}
