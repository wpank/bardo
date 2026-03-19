//! Scrollable list widget with cursor highlighting and substring filtering.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::palette::{BG_RAISED, BG_VOID, BONE, TEXT_PRIMARY};

/// Cursor-driven scrollable list with optional substring filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScrollableList {
    /// Available items.
    pub(crate) items: Vec<String>,
    /// Cursor in the filtered view.
    pub(crate) cursor: usize,
    /// Top visible row.
    pub(crate) scroll_offset: usize,
    /// Optional substring filter.
    pub(crate) filter: Option<String>,
}

impl ScrollableList {
    /// Creates a new list from item strings.
    pub(crate) fn new(items: Vec<String>) -> Self {
        Self {
            items,
            cursor: 0,
            scroll_offset: 0,
            filter: None,
        }
    }

    /// Returns `(original_index, item)` pairs that match the current filter.
    pub(crate) fn filtered_items(&self) -> Vec<(usize, &str)> {
        let filter = self.filter.as_deref().map(str::to_lowercase);

        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| match filter.as_deref() {
                Some(filter_text) => item.to_lowercase().contains(filter_text),
                None => true,
            })
            .map(|(index, item)| (index, item.as_str()))
            .collect()
    }

    /// Moves the cursor down within the filtered view.
    pub(crate) fn move_cursor_down(&mut self) {
        let item_count = self.filtered_items().len();
        if item_count == 0 {
            return;
        }
        self.cursor = (self.cursor + 1).min(item_count - 1);
    }

    /// Moves the cursor up within the filtered view.
    pub(crate) fn move_cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
}

impl Widget for &ScrollableList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let filtered = self.filtered_items();
        let display_rows = area.height as usize;

        for (row, (_, item)) in filtered
            .iter()
            .skip(self.scroll_offset)
            .take(display_rows)
            .enumerate()
        {
            let y = area.y + row as u16;
            let is_cursor = self.scroll_offset + row == self.cursor;
            let prefix = if is_cursor { "▶ " } else { "  " };
            let line = format!("{prefix}{item}");

            buf.set_stringn(
                area.x,
                y,
                &line,
                area.width as usize,
                Style::default()
                    .fg(if is_cursor { BONE } else { TEXT_PRIMARY })
                    .bg(if is_cursor { BG_RAISED } else { BG_VOID })
                    .add_modifier(if is_cursor {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtered_items_preserve_original_indices() {
        let mut list = ScrollableList::new(vec![
            "alpha".to_string(),
            "beta".to_string(),
            "alphabet".to_string(),
        ]);
        list.filter = Some("alp".to_string());

        assert_eq!(list.filtered_items(), vec![(0, "alpha"), (2, "alphabet")]);
    }

    #[test]
    fn cursor_navigation_stays_in_bounds() {
        let mut list = ScrollableList::new(vec!["a".to_string(), "b".to_string()]);
        list.move_cursor_down();
        list.move_cursor_down();
        assert_eq!(list.cursor, 1);
        list.move_cursor_up();
        assert_eq!(list.cursor, 0);
    }
}
