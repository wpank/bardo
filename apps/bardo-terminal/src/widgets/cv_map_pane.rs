//! CV Map pane widget — tabular display of all CV outputs from CvMapper.
//!
//! Columns: Name | Source | Range (min..max) | Curve | Alpha
//! Selected row is highlighted. Inverted outputs show `< inverted` annotation.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::{
    palette::{BG_RAISED, BG_VOID, BONE, ROSE, ROSE_DIM, TEXT_DIM, TEXT_PRIMARY},
    sonification::CvMapState,
};

/// Column widths (fixed layout):
/// Name: 22, Source: 12, Range: 14, Curve: 12, Alpha: 6
const COL_NAME: u16 = 22;
const COL_SOURCE: u16 = 12;
const COL_RANGE: u16 = 14;
const COL_CURVE: u16 = 12;
const COL_ALPHA: u16 = 6;

/// Renders the CV mapping table with cursor highlighting.
pub(crate) struct CvMapPane<'a> {
    pub(crate) state: &'a CvMapState,
}

impl<'a> Widget for CvMapPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 2 {
            return;
        }

        let header_y = area.y;
        let header_style = Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD);

        // Render header row
        let mut x = area.x;
        render_cell(buf, x, header_y, "CV Output", COL_NAME, header_style);
        x += COL_NAME;
        render_cell(buf, x, header_y, "Source", COL_SOURCE, header_style);
        x += COL_SOURCE;
        render_cell(buf, x, header_y, "Range", COL_RANGE, header_style);
        x += COL_RANGE;
        render_cell(buf, x, header_y, "Curve", COL_CURVE, header_style);
        x += COL_CURVE;
        render_cell(buf, x, header_y, "Alpha", COL_ALPHA, header_style);

        // Render data rows
        let rows_available = (area.height - 1) as usize;
        let scroll_offset = if self.state.cursor >= rows_available {
            self.state.cursor - rows_available + 1
        } else {
            0
        };

        for (row_idx, output) in self
            .state
            .outputs
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(rows_available)
        {
            let y = area.y + 1 + (row_idx - scroll_offset) as u16;
            if y >= area.y + area.height {
                break;
            }

            let is_selected = row_idx == self.state.cursor;
            let is_editing = is_selected && self.state.editing;
            let base_fg = if is_selected { BONE } else { TEXT_PRIMARY };
            let base_bg = if is_selected { BG_RAISED } else { BG_VOID };
            let base_style = Style::default().fg(base_fg).bg(base_bg);

            let mut x = area.x;

            // Name column — show inverted annotation
            let name_text = if output.is_inverted() {
                format!("{} < inv", output.name)
            } else {
                output.name.clone()
            };
            let name_style = if output.is_inverted() {
                base_style.fg(ROSE)
            } else {
                base_style
            };
            render_cell(buf, x, y, &name_text, COL_NAME, name_style);
            x += COL_NAME;

            // Source column
            let source_style = edit_highlight(base_style, is_editing, self.state.edit_col, 0);
            render_cell(buf, x, y, &output.source, COL_SOURCE, source_style);
            x += COL_SOURCE;

            // Range column
            let range_style = edit_highlight(base_style, is_editing, self.state.edit_col, 1);
            render_cell(buf, x, y, &output.range_label(), COL_RANGE, range_style);
            x += COL_RANGE;

            // Curve column
            let curve_style = edit_highlight(base_style, is_editing, self.state.edit_col, 2);
            render_cell(buf, x, y, &output.curve.label(), COL_CURVE, curve_style);
            x += COL_CURVE;

            // Alpha column
            let alpha_style = edit_highlight(base_style, is_editing, self.state.edit_col, 3);
            render_cell(
                buf,
                x,
                y,
                &format!("{:.2}", output.alpha),
                COL_ALPHA,
                alpha_style,
            );
        }
    }
}

fn edit_highlight(base: Style, is_editing: bool, edit_col: usize, col: usize) -> Style {
    if is_editing && edit_col == col {
        base.fg(ROSE_DIM).add_modifier(Modifier::UNDERLINED)
    } else {
        base
    }
}

fn render_cell(buf: &mut Buffer, x: u16, y: u16, text: &str, width: u16, style: Style) {
    let area = buf.area();
    if y < area.top() || y >= area.bottom() {
        return;
    }
    let max_x = area.right().min(x + width);
    if x >= max_x {
        return;
    }
    buf.set_stringn(x, y, text, (max_x - x) as usize, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sonification::CvMapState;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn cv_map_pane_renders_without_panic() {
        let state = CvMapState::default();
        let pane = CvMapPane { state: &state };
        let area = Rect::new(0, 0, 80, 16);
        let mut buf = Buffer::empty(area);
        pane.render(area, &mut buf);

        // Header row should contain column labels
        let header: String = (0..80)
            .map(|x| buf.get(x, 0).symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(header.contains("CV Output"));
        assert!(header.contains("Source"));
        assert!(header.contains("Range"));
    }

    #[test]
    fn cv_map_pane_shows_all_13_default_rows() {
        let state = CvMapState::default();
        let pane = CvMapPane { state: &state };
        // Need header + 13 rows = 14 lines
        let area = Rect::new(0, 0, 80, 14);
        let mut buf = Buffer::empty(area);
        pane.render(area, &mut buf);

        // First data row should show "clock_bpm"
        let row1: String = (0..80)
            .map(|x| buf.get(x, 1).symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(row1.contains("clock_bpm"));
    }

    #[test]
    fn cv_map_pane_shows_inverted_annotation() {
        let mut state = CvMapState::default();
        // Make first output inverted
        state.outputs[0].output_min = 1.0;
        state.outputs[0].output_max = 0.0;
        let pane = CvMapPane { state: &state };
        let area = Rect::new(0, 0, 80, 14);
        let mut buf = Buffer::empty(area);
        pane.render(area, &mut buf);

        let row1: String = (0..80)
            .map(|x| buf.get(x, 1).symbol().to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(row1.contains("inv"));
    }

    #[test]
    fn cv_map_pane_no_panic_on_tiny_area() {
        let state = CvMapState::default();
        let pane = CvMapPane { state: &state };
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        pane.render(area, &mut buf); // should not panic
    }
}
