//! Responsive layout breakpoints for the terminal scaffold.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Responsive breakpoint for the scaffold layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutBreakpoint {
    /// Fewer than 80 columns.
    Compact,
    /// 80 to 119 columns.
    Standard,
    /// 120 to 179 columns.
    Wide,
    /// 180 columns or more.
    Ultra,
}

impl LayoutBreakpoint {
    /// Returns the breakpoint for a given terminal width.
    pub(crate) const fn from_cols(cols: u16) -> Self {
        match cols {
            0..=79 => Self::Compact,
            80..=119 => Self::Standard,
            120..=179 => Self::Wide,
            _ => Self::Ultra,
        }
    }

    /// Returns the number of columns reserved for the sprite sidebar.
    pub(crate) const fn sprite_sidebar_cols(self) -> u16 {
        match self {
            Self::Compact => 0,
            Self::Standard => 6,
            Self::Wide => 10,
            Self::Ultra => 14,
        }
    }

    /// Returns the number of content panels for this breakpoint.
    pub(crate) const fn panel_count(self) -> u8 {
        match self {
            Self::Compact => 1,
            Self::Standard => 2,
            Self::Wide => 3,
            Self::Ultra => 4,
        }
    }

    /// Returns the display label used in the chrome.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Standard => "Standard",
            Self::Wide => "Wide",
            Self::Ultra => "Ultra",
        }
    }
}

/// Computes the sidebar and content rectangles for the current frame.
pub(crate) fn compute_layout(frame_size: Rect, bp: LayoutBreakpoint) -> (Rect, Rect) {
    let sidebar_cols = bp.sprite_sidebar_cols();
    let inner = Rect {
        x: frame_size.x,
        y: frame_size.y.saturating_add(1),
        width: frame_size.width,
        height: frame_size.height.saturating_sub(2),
    };

    if sidebar_cols == 0 {
        return (Rect::default(), inner);
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_cols), Constraint::Min(0)])
        .split(inner);

    (chunks[0], chunks[1])
}

#[cfg(test)]
mod tests {
    use super::{LayoutBreakpoint, compute_layout};
    use ratatui::layout::Rect;

    #[test]
    fn breakpoint_thresholds_match_spec() {
        assert_eq!(LayoutBreakpoint::from_cols(0), LayoutBreakpoint::Compact);
        assert_eq!(LayoutBreakpoint::from_cols(79), LayoutBreakpoint::Compact);
        assert_eq!(LayoutBreakpoint::from_cols(80), LayoutBreakpoint::Standard);
        assert_eq!(LayoutBreakpoint::from_cols(119), LayoutBreakpoint::Standard);
        assert_eq!(LayoutBreakpoint::from_cols(120), LayoutBreakpoint::Wide);
        assert_eq!(LayoutBreakpoint::from_cols(179), LayoutBreakpoint::Wide);
        assert_eq!(LayoutBreakpoint::from_cols(180), LayoutBreakpoint::Ultra);
        assert_eq!(LayoutBreakpoint::from_cols(400), LayoutBreakpoint::Ultra);
    }

    #[test]
    fn breakpoint_helpers_match_expected_values() {
        assert_eq!(LayoutBreakpoint::Compact.sprite_sidebar_cols(), 0);
        assert_eq!(LayoutBreakpoint::Standard.sprite_sidebar_cols(), 6);
        assert_eq!(LayoutBreakpoint::Wide.sprite_sidebar_cols(), 10);
        assert_eq!(LayoutBreakpoint::Ultra.sprite_sidebar_cols(), 14);

        assert_eq!(LayoutBreakpoint::Compact.panel_count(), 1);
        assert_eq!(LayoutBreakpoint::Standard.panel_count(), 2);
        assert_eq!(LayoutBreakpoint::Wide.panel_count(), 3);
        assert_eq!(LayoutBreakpoint::Ultra.panel_count(), 4);
    }

    #[test]
    fn compute_layout_reserves_chrome_rows() {
        let frame = Rect::new(0, 0, 100, 40);
        let (sidebar, content) = compute_layout(frame, LayoutBreakpoint::Standard);

        assert_eq!(sidebar.width, 6);
        assert_eq!(content.x, 6);
        assert_eq!(content.y, 1);
        assert_eq!(content.height, 38);
    }
}
