//! Event feed widget with bounded scrollback and substring filtering.

use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::palette::{ROSE_BRIGHT, TEXT_DIM, TEXT_GHOST, TEXT_PRIMARY, WARNING};

/// Feed entry severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeedLevel {
    /// Informational entry.
    Info,
    /// Warning entry.
    Warn,
    /// Error entry.
    Error,
    /// Debug entry.
    Debug,
}

impl FeedLevel {
    /// Returns the display color for this feed level.
    pub(crate) const fn color(self) -> Color {
        match self {
            Self::Info => TEXT_PRIMARY,
            Self::Warn => WARNING,
            Self::Error => ROSE_BRIGHT,
            Self::Debug => TEXT_DIM,
        }
    }

    /// Returns the fixed-width display label for this feed level.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO ",
            Self::Warn => "WARN ",
            Self::Error => "ERROR",
            Self::Debug => "DBG  ",
        }
    }
}

/// One event feed row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeedEntry {
    /// Tick at which the entry was recorded.
    pub(crate) tick: u64,
    /// Entry severity.
    pub(crate) level: FeedLevel,
    /// Entry message.
    pub(crate) message: String,
}

/// Bounded scrollback buffer for terminal logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventFeed {
    /// Stored entries.
    pub(crate) entries: VecDeque<FeedEntry>,
    /// Maximum retained entries.
    pub(crate) max_entries: usize,
    /// Newest-relative scroll offset.
    pub(crate) scroll_offset: usize,
    /// Optional substring filter.
    pub(crate) filter: Option<String>,
}

impl EventFeed {
    /// Creates an empty feed with bounded storage.
    pub(crate) fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries.min(1000)),
            max_entries,
            scroll_offset: 0,
            filter: None,
        }
    }

    /// Pushes a new entry, evicting the oldest one when the buffer is full.
    pub(crate) fn push(&mut self, entry: FeedEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Returns the filtered entries in newest-first order.
    fn visible_entries(&self) -> Vec<&FeedEntry> {
        let filter = self.filter.as_deref().map(str::to_lowercase);

        self.entries
            .iter()
            .filter(|entry| match filter.as_deref() {
                Some(filter_text) => entry.message.to_lowercase().contains(filter_text),
                None => true,
            })
            .rev()
            .collect()
    }
}

impl Widget for &EventFeed {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let visible = self.visible_entries();
        let display_rows = area.height as usize;
        let start_index = self
            .scroll_offset
            .min(visible.len().saturating_sub(display_rows));

        for (row, entry) in visible
            .iter()
            .skip(start_index)
            .take(display_rows)
            .enumerate()
        {
            let y = area.y + row as u16;
            let tick_text = format!("{:>6} ", entry.tick);
            let level_text = entry.level.label();
            let reserved = tick_text.len() + level_text.len() + 1;
            let msg_max = area.width.saturating_sub(reserved as u16) as usize;
            let message_text: String = entry.message.chars().take(msg_max).collect();

            buf.set_stringn(
                area.x,
                y,
                &tick_text,
                area.width as usize,
                Style::default().fg(TEXT_GHOST),
            );

            let level_x = area.x + tick_text.len() as u16;
            buf.set_stringn(
                level_x,
                y,
                level_text,
                area.width.saturating_sub(tick_text.len() as u16) as usize,
                Style::default()
                    .fg(entry.level.color())
                    .add_modifier(Modifier::BOLD),
            );

            let message_x = level_x + level_text.len() as u16 + 1;
            if message_x < area.x + area.width {
                buf.set_stringn(
                    message_x,
                    y,
                    message_text,
                    area.width.saturating_sub(message_x - area.x) as usize,
                    Style::default().fg(entry.level.color()),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_feed_scroll() {
        let mut feed = EventFeed::new(5);
        for index in 0..10_u64 {
            feed.push(FeedEntry {
                tick: index,
                level: FeedLevel::Info,
                message: format!("msg {index}"),
            });
        }

        assert_eq!(feed.entries.len(), 5);
        assert_eq!(feed.entries.back().unwrap().tick, 9);

        feed.filter = Some("msg 6".to_string());
        let visible = feed.visible_entries();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].tick, 6);
    }

    /// INV-014: EventFeed maintains max_entries limit via FIFO eviction.
    #[test]
    fn test_event_feed_capacity() {
        for &(max_entries, inserts) in &[(1usize, 5usize), (10, 11), (100, 100), (1000, 1001)] {
            let mut feed = EventFeed::new(max_entries);
            for i in 0..inserts as u64 {
                feed.push(FeedEntry {
                    tick: i,
                    level: FeedLevel::Info,
                    message: format!("msg {i}"),
                });
            }
            assert!(
                feed.entries.len() <= max_entries,
                "len {} > max {max_entries} after {inserts} inserts",
                feed.entries.len()
            );
            if inserts > max_entries {
                assert_eq!(feed.entries.len(), max_entries);
                // Oldest entries dropped: front tick should be inserts - max_entries
                assert_eq!(
                    feed.entries.front().unwrap().tick,
                    (inserts - max_entries) as u64
                );
            }
        }
    }

    /// INV-015: Filter applies case-insensitive substring matching.
    #[test]
    fn test_event_feed_filter() {
        let mut feed = EventFeed::new(100);
        feed.push(FeedEntry {
            tick: 0,
            level: FeedLevel::Info,
            message: "msg 6".to_string(),
        });
        feed.push(FeedEntry {
            tick: 1,
            level: FeedLevel::Warn,
            message: "MSG 7".to_string(),
        });
        feed.push(FeedEntry {
            tick: 2,
            level: FeedLevel::Error,
            message: "other".to_string(),
        });

        // No filter: all entries
        feed.filter = None;
        assert_eq!(feed.visible_entries().len(), 3);

        // Empty filter: all entries
        feed.filter = Some("".to_string());
        assert_eq!(feed.visible_entries().len(), 3);

        // Case-insensitive "msg" matches "msg 6" and "MSG 7"
        feed.filter = Some("msg".to_string());
        let vis = feed.visible_entries();
        assert_eq!(vis.len(), 2);

        // Uppercase filter still matches
        feed.filter = Some("MSG".to_string());
        assert_eq!(feed.visible_entries().len(), 2);

        // No match
        feed.filter = Some("xyz".to_string());
        assert_eq!(feed.visible_entries().len(), 0);
    }

    /// INV-016: Scroll offset never exceeds visible entries.
    #[test]
    fn test_event_feed_scroll_bounds() {
        for &(total_entries, display_rows, scroll_offset) in &[
            (0usize, 1usize, 0usize),
            (0, 5, 100),
            (10, 5, 0),
            (10, 5, 5),
            (10, 5, 100),
            (10, 50, 0),
            (100, 5, 1000),
        ] {
            let mut feed = EventFeed::new(total_entries.max(1));
            for i in 0..total_entries as u64 {
                feed.push(FeedEntry {
                    tick: i,
                    level: FeedLevel::Info,
                    message: format!("msg {i}"),
                });
            }
            feed.scroll_offset = scroll_offset;

            let visible = feed.visible_entries();
            let total = visible.len();
            let start_idx = feed.scroll_offset.min(total.saturating_sub(display_rows));

            // start_idx + display_rows should not exceed total (unless total < display_rows)
            if total >= display_rows {
                assert!(
                    start_idx + display_rows <= total,
                    "start_idx({start_idx}) + rows({display_rows}) > total({total})"
                );
            }
            assert!(start_idx <= total.saturating_sub(1).max(0));
        }
    }

    /// INV-017: Each FeedLevel has a unique color and label.
    #[test]
    fn test_feed_level_colors_and_labels() {
        let levels = [
            FeedLevel::Info,
            FeedLevel::Warn,
            FeedLevel::Error,
            FeedLevel::Debug,
        ];

        // All labels are 5 chars
        for level in &levels {
            assert_eq!(
                level.label().len(),
                5,
                "{:?} label '{}' is not 5 chars",
                level,
                level.label()
            );
        }

        // Colors are assigned without panic
        let colors: Vec<Color> = levels.iter().map(|l| l.color()).collect();
        assert_eq!(colors.len(), 4);

        // Specific mappings
        assert_eq!(FeedLevel::Info.color(), TEXT_PRIMARY);
        assert_eq!(FeedLevel::Warn.color(), WARNING);
        assert_eq!(FeedLevel::Error.color(), ROSE_BRIGHT);
        assert_eq!(FeedLevel::Debug.color(), TEXT_DIM);

        assert_eq!(FeedLevel::Info.label(), "INFO ");
        assert_eq!(FeedLevel::Warn.label(), "WARN ");
        assert_eq!(FeedLevel::Error.label(), "ERROR");
        assert_eq!(FeedLevel::Debug.label(), "DBG  ");
    }
}
