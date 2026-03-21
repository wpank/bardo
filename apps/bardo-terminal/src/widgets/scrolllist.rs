//! Scrollable list widget with cursor highlighting and substring filtering.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::palette::{BG_RAISED, BG_VOID, BONE, TEXT_PRIMARY};

/// Cursor-driven scrollable list with optional substring filtering.
///
/// # Parent Responsibility
///
/// When the `filter` field changes, the parent screen must reset `cursor` to 0
/// (or clamp it to the new filtered item count) to prevent the cursor from
/// pointing to an out-of-bounds index. The `filtered_items()` method does not
/// automatically adjust `cursor` when the filter changes.
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

    /// INV-021: cursor cannot exceed filtered item count.
    #[test]
    fn test_scrollable_list_cursor_bounds() {
        let mut list = ScrollableList::new(vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
        ]);

        // Down past the end stays clamped at last index.
        for _ in 0..10 {
            list.move_cursor_down();
        }
        assert_eq!(list.cursor, 2);

        // Up past zero stays at zero.
        for _ in 0..10 {
            list.move_cursor_up();
        }
        assert_eq!(list.cursor, 0);

        // With a filter that narrows to 1 item, cursor stays at 0.
        list.filter = Some("two".to_string());
        list.cursor = 0;
        list.move_cursor_down();
        assert_eq!(list.cursor, 0); // only 1 filtered item

        // Empty list: cursor doesn't move.
        let mut empty = ScrollableList::new(vec![]);
        empty.move_cursor_down();
        assert_eq!(empty.cursor, 0);
        empty.move_cursor_up();
        assert_eq!(empty.cursor, 0);
    }

    /// INV-022: case-insensitive substring filtering preserves original indices.
    #[test]
    fn test_scrollable_list_filter() {
        let list_items = vec![
            "Alpha".to_string(),
            "BETA".to_string(),
            "gamma".to_string(),
            "AlphaBeta".to_string(),
        ];

        // Case-insensitive match on "alpha".
        let mut list = ScrollableList::new(list_items.clone());
        list.filter = Some("alpha".to_string());
        let filtered = list.filtered_items();
        assert_eq!(filtered, vec![(0, "Alpha"), (3, "AlphaBeta")]);

        // Case-insensitive match on "BETA".
        list.filter = Some("BETA".to_string());
        let filtered = list.filtered_items();
        assert_eq!(filtered, vec![(1, "BETA"), (3, "AlphaBeta")]);

        // No filter returns all items.
        list.filter = None;
        let filtered = list.filtered_items();
        assert_eq!(filtered.len(), 4);

        // Filter with no matches returns empty.
        list.filter = Some("zzz".to_string());
        let filtered = list.filtered_items();
        assert!(filtered.is_empty());
    }
}
