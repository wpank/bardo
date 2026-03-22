use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect) {
    let popup = centered_rect(55, 70, area);
    f.render_widget(Clear, popup);

    let bindings = vec![
        ("", "── Navigation ──"),
        ("↑/↓ or j/k", "Navigate focused panel"),
        ("Tab", "Cycle focus (plans → tasks → output)"),
        ("Shift+Tab", "Reverse cycle focus"),
        ("Enter", "View plan detail / expand task"),
        ("Esc", "Close modal / go back"),
        ("←/→ or h/l", "Collapse/expand wave (plan tree)"),
        ("PgUp/PgDn", "Scroll focused panel"),
        ("End / Space", "Resume auto-scroll in output"),
        ("", ""),
        ("", "── Detail Sub-Tabs ──"),
        ("a", "Agents panel"),
        ("o", "Agent output"),
        ("d", "Diff panel"),
        ("e", "Error digest"),
        ("g", "Git view"),
        ("", ""),
        ("", "── Agent Tabs ──"),
        ("`", "Cycle agent tabs"),
        ("Alt+1..7", "Select agent tab directly"),
        ("v", "Toggle implementation/verification pane"),
        ("", ""),
        ("", "── Actions ──"),
        ("p", "Pause / resume pipeline"),
        ("i", "Inject message to active agent"),
        ("y/n", "Approve/reject command"),
        ("r", "Restart current phase"),
        ("c", "Re-verify plan (gates + reviews only)"),
        ("/", "Filter plans (fuzzy search)"),
        ("w", "Wave overview modal"),
        ("", ""),
        ("", "── Destructive (ctrl+, asks to confirm) ──"),
        ("ctrl+r", "Restart ALL plans from scratch"),
        ("ctrl+d", "Reset selected plan from scratch"),
        ("ctrl+x", "Force commit + advance to next plan"),
        ("ctrl+g", "Git reconcile (merge, tag, prune, staging)"),
        ("", ""),
        ("", "── Global ──"),
        ("F1-F4", "Dashboard (with different detail focus)"),
        ("F5", "Logs view"),
        ("F6", "Config view"),
        ("?", "Toggle this help"),
        ("q", "Quit"),
        ("Ctrl-C", "Force quit"),
    ];

    let lines: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            if key.is_empty() {
                Line::from(Span::styled(
                    format!(" {desc}"),
                    Style::default().fg(Theme::DREAM),
                ))
            } else {
                Line::from(vec![
                    Span::styled(format!(" {key:>14} "), Style::default().fg(Theme::BONE_DIM)),
                    Span::styled(format!(" {desc}"), Style::default().fg(Theme::FG)),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Keybindings")
            .border_style(Style::default().fg(Theme::DREAM))
            .style(Theme::default_style()),
    );
    f.render_widget(paragraph, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
