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

    /// INV-021: Cursor never exceeds filtered item count; down/up are bounded.
    #[test]
    fn test_scrollable_list_cursor_bounds() {
        // Empty list: cursor stays at 0
        let mut empty = ScrollableList::new(vec![]);
        empty.move_cursor_down();
        assert_eq!(empty.cursor, 0);
        empty.move_cursor_up();
        assert_eq!(empty.cursor, 0);

        // Single item: cursor stays at 0
        let mut single = ScrollableList::new(vec!["a".to_string()]);
        single.move_cursor_down();
        assert_eq!(single.cursor, 0);
        single.move_cursor_up();
        assert_eq!(single.cursor, 0);

        // Multiple items: cursor bounded by count
        let mut multi =
            ScrollableList::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        for _ in 0..10 {
            multi.move_cursor_down();
        }
        assert_eq!(multi.cursor, 2); // max index = len - 1
        for _ in 0..10 {
            multi.move_cursor_up();
        }
        assert_eq!(multi.cursor, 0);

        // With filter active, cursor bounded by filtered count
        multi.cursor = 0;
        multi.filter = Some("a".to_string());
        let filtered_count = multi.filtered_items().len();
        assert_eq!(filtered_count, 1);
        multi.move_cursor_down();
        assert_eq!(multi.cursor, 0); // only 1 item, can't go past 0

        // Out-of-bounds cursor from prior state
        multi.cursor = 999;
        multi.filter = None;
        multi.move_cursor_down();
        // After move_down with cursor=999, min(1000, 2) = 2
        assert!(multi.cursor <= multi.filtered_items().len().saturating_sub(1));
    }

    /// INV-022: Filtering applies case-insensitive substring match; order preserved.
    #[test]
    fn test_scrollable_list_filter() {
        let list = ScrollableList::new(vec![
            "apple".to_string(),
            "Apricot".to_string(),
            "banana".to_string(),
        ]);

        // No filter: all items in order
        let all = list.filtered_items();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], (0, "apple"));
        assert_eq!(all[1], (1, "Apricot"));
        assert_eq!(all[2], (2, "banana"));

        // Empty filter: all items
        let mut list_empty_filter = list.clone();
        list_empty_filter.filter = Some("".to_string());
        assert_eq!(list_empty_filter.filtered_items().len(), 3);

        // "ap" matches "apple" and "Apricot" (case-insensitive)
        let mut list_ap = list.clone();
        list_ap.filter = Some("ap".to_string());
        let filtered = list_ap.filtered_items();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0], (0, "apple"));
        assert_eq!(filtered[1], (1, "Apricot"));

        // "AP" also matches (case-insensitive)
        let mut list_ap_upper = list.clone();
        list_ap_upper.filter = Some("AP".to_string());
        assert_eq!(list_ap_upper.filtered_items().len(), 2);

        // "xyz" matches nothing
        let mut list_xyz = list.clone();
        list_xyz.filter = Some("xyz".to_string());
        assert_eq!(list_xyz.filtered_items().len(), 0);
    }
}
