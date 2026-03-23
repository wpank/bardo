//! Responsive layout breakpoints for the terminal scaffold.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

const COMPACT_MAX_COLS: u16 = 79;
const STANDARD_MAX_COLS: u16 = 119;
const WIDE_MAX_COLS: u16 = 179;
const TOP_CHROME_ROWS: u16 = 1;
const BOTTOM_CHROME_ROWS: u16 = 1;
const RESERVED_CHROME_ROWS: u16 = TOP_CHROME_ROWS + BOTTOM_CHROME_ROWS;

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
            0..=COMPACT_MAX_COLS => Self::Compact,
            80..=STANDARD_MAX_COLS => Self::Standard,
            120..=WIDE_MAX_COLS => Self::Wide,
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
        y: frame_size.y.saturating_add(TOP_CHROME_ROWS),
        width: frame_size.width,
        height: frame_size.height.saturating_sub(RESERVED_CHROME_ROWS),
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

        assert_eq!(LayoutBreakpoint::Compact.label(), "Compact");
        assert_eq!(LayoutBreakpoint::Standard.label(), "Standard");
        assert_eq!(LayoutBreakpoint::Wide.label(), "Wide");
        assert_eq!(LayoutBreakpoint::Ultra.label(), "Ultra");
    }

    #[test]
    fn compute_layout_reserves_chrome_rows_and_sidebar_width() {
        let frame = Rect::new(3, 5, 100, 40);
        let (sidebar, content) = compute_layout(frame, LayoutBreakpoint::Standard);

        assert_eq!(sidebar, Rect::new(3, 6, 6, 38));
        assert_eq!(content, Rect::new(9, 6, 94, 38));
    }

    #[test]
    fn compute_layout_suppresses_sidebar_for_compact() {
        let frame = Rect::new(2, 4, 79, 12);
        let (sidebar, content) = compute_layout(frame, LayoutBreakpoint::Compact);

        assert_eq!(sidebar, Rect::default());
        assert_eq!(content, Rect::new(2, 5, 79, 10));
    }

    // --- Named tests for verification chain (INV-002 through INV-013, INV-022) ---

    #[test]
    fn test_layout_breakpoint_compact() {
        assert_eq!(LayoutBreakpoint::from_cols(0), LayoutBreakpoint::Compact);
        assert_eq!(LayoutBreakpoint::from_cols(79), LayoutBreakpoint::Compact);
    }

    #[test]
    fn test_layout_breakpoint_standard() {
        assert_eq!(LayoutBreakpoint::from_cols(80), LayoutBreakpoint::Standard);
        assert_eq!(LayoutBreakpoint::from_cols(119), LayoutBreakpoint::Standard);
    }

    #[test]
    fn test_layout_breakpoint_wide() {
        assert_eq!(LayoutBreakpoint::from_cols(120), LayoutBreakpoint::Wide);
        assert_eq!(LayoutBreakpoint::from_cols(179), LayoutBreakpoint::Wide);
    }

    #[test]
    fn test_layout_breakpoint_ultra() {
        assert_eq!(LayoutBreakpoint::from_cols(180), LayoutBreakpoint::Ultra);
        assert_eq!(LayoutBreakpoint::from_cols(400), LayoutBreakpoint::Ultra);
    }

    #[test]
    fn test_sprite_sidebar_compact_zero() {
        assert_eq!(LayoutBreakpoint::Compact.sprite_sidebar_cols(), 0);
    }

    #[test]
    fn test_sprite_sidebar_standard_6col() {
        assert_eq!(LayoutBreakpoint::Standard.sprite_sidebar_cols(), 6);
    }

    #[test]
    fn test_sprite_sidebar_wide_10col() {
        assert_eq!(LayoutBreakpoint::Wide.sprite_sidebar_cols(), 10);
    }

    #[test]
    fn test_sprite_sidebar_ultra_14col() {
        assert_eq!(LayoutBreakpoint::Ultra.sprite_sidebar_cols(), 14);
    }

    #[test]
    fn test_panel_count_compact() {
        assert_eq!(LayoutBreakpoint::Compact.panel_count(), 1);
    }

    #[test]
    fn test_panel_count_standard() {
        assert_eq!(LayoutBreakpoint::Standard.panel_count(), 2);
    }

    #[test]
    fn test_panel_count_wide() {
        assert_eq!(LayoutBreakpoint::Wide.panel_count(), 3);
    }

    #[test]
    fn test_panel_count_ultra() {
        assert_eq!(LayoutBreakpoint::Ultra.panel_count(), 4);
    }

    #[test]
    fn test_chrome_rows_2() {
        let frame = Rect::new(0, 0, 120, 50);
        let (_sidebar, content) = compute_layout(frame, LayoutBreakpoint::Wide);
        // 2 chrome rows reserved: content height = frame height - 2
        assert_eq!(content.height, 48);
        assert_eq!(content.y, 1);
    }

    #[test]
    fn test_layout_inner_y_offset() {
        let frame = Rect::new(0, 0, 80, 24);
        let (_sidebar, content) = compute_layout(frame, LayoutBreakpoint::Standard);
        assert_eq!(content.y, 1);
    }
}
