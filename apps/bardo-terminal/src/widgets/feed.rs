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
}
