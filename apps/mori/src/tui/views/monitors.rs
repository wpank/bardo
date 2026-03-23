use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::state::RunState;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(4)])
        .split(area);

    render_watchers(f, sections[0]);
    render_history(f, sections[1], state);
}

fn render_watchers(f: &mut Frame, area: Rect) {
    let watchers = vec![
        ("AgentSilence", "5m timeout"),
        ("CompileFailRepeat", "3x threshold"),
        ("TaskStall", "10m timeout"),
        ("ContextPressure", ">80% tokens"),
        ("PhaseTimeout", "30m timeout"),
    ];

    let items: Vec<ListItem> = watchers
        .iter()
        .map(|(name, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(" ● ", Style::default().fg(Theme::STATUS_OK)),
                Span::styled(*name, Style::default().fg(Theme::FG)),
                Span::styled(format!("  {desc}"), Style::default().fg(Theme::FG_DIM)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Conductor Watchers")
            .style(Theme::block_style())
            .title_style(Theme::title_style()),
    );
    f.render_widget(list, area);
}

fn render_history(f: &mut Frame, area: Rect, state: &RunState) {
    let items: Vec<ListItem> = state
        .conductor_history
        .iter()
        .rev()
        .take(20)
        .map(|entry| {
            ListItem::new(Line::from(vec![
                Span::styled(&entry.timestamp, Style::default().fg(Theme::FG_DIM)),
                Span::styled(
                    format!(" [{}] ", entry.watcher),
                    Style::default().fg(Theme::DREAM),
                ),
                Span::styled(&entry.target, Style::default().fg(Theme::ROSE)),
                Span::styled(
                    format!(": {}", entry.message),
                    Style::default().fg(Theme::FG),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Intervention History")
            .style(Theme::block_style())
            .title_style(Theme::title_style()),
    );
    f.render_widget(list, area);
}
