use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::RunState;
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar;

/// Render structured error digest from gate output and agent errors.
/// Groups errors by file, colors by severity.
pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let mut lines: Vec<Line> = Vec::new();

    // Source 1: last gate output (compile/test errors)
    if !state.last_gate_output.is_empty() {
        let errors = parse_errors(&state.last_gate_output);
        if !errors.is_empty() {
            render_error_groups(&mut lines, &errors, "Gate");
        }
    }

    // Source 2: command output (streaming gate output)
    if !state.command_output.is_empty() && state.last_gate_output.is_empty() {
        let errors = parse_errors(&state.command_output);
        if !errors.is_empty() {
            render_error_groups(&mut lines, &errors, "Output");
        }
    }

    // Source 3: global error state
    if let Some(ref err) = state.error {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" ✗ ", Style::default().fg(Theme::EMBER)),
            Span::styled(
                "Pipeline Error",
                Style::default()
                    .fg(Theme::EMBER)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        for err_line in err.lines() {
            lines.push(Line::from(Span::styled(
                format!("   {err_line}"),
                Style::default().fg(Theme::EMBER),
            )));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " No errors",
            Style::default().fg(Theme::SAGE),
        )));
    }

    let error_count = lines
        .iter()
        .filter(|l| l.spans.iter().any(|s| s.style.fg == Some(Theme::EMBER)))
        .count();

    let title = if error_count > 0 {
        format!("Errors ({error_count})")
    } else {
        "Errors".to_string()
    };

    let visible_height = area.height.saturating_sub(2) as usize;
    let total = lines.len();
    let start = total.saturating_sub(visible_height);
    let visible: Vec<Line> = lines.into_iter().skip(start).take(visible_height).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(if error_count > 0 {
            Style::default().fg(Theme::EMBER)
        } else {
            Theme::unfocused_border_style()
        })
        .title_style(if error_count > 0 {
            Style::default()
                .fg(Theme::EMBER)
                .add_modifier(Modifier::BOLD)
        } else {
            Theme::title_style()
        });

    let paragraph = Paragraph::new(visible)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    if total > visible_height {
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        scrollbar::render_scrollbar(
            f.buffer_mut(),
            inner,
            total,
            visible_height,
            start,
            Theme::EMBER,
        );
    }
}

#[derive(Debug)]
struct ErrorEntry {
    file: String,
    line_num: Option<u32>,
    message: String,
    severity: Severity,
}

#[derive(Debug, PartialEq)]
enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
struct ErrorGroup {
    file: String,
    entries: Vec<ErrorEntry>,
}

fn parse_errors(output: &str) -> Vec<ErrorGroup> {
    let mut entries: Vec<ErrorEntry> = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Rust error format: "error[E0308]: mismatched types"
        // or "error: could not compile"
        if trimmed.starts_with("error") {
            entries.push(ErrorEntry {
                file: String::new(),
                line_num: None,
                message: trimmed.to_string(),
                severity: Severity::Error,
            });
            continue;
        }

        // Rust location: " --> src/file.rs:12:34"
        if trimmed.starts_with("-->") {
            let path_part = trimmed.trim_start_matches("-->").trim();
            let parts: Vec<&str> = path_part.split(':').collect();
            if let Some(last) = entries.last_mut() {
                last.file = parts.first().unwrap_or(&"").to_string();
                last.line_num = parts.get(1).and_then(|s| s.parse().ok());
            }
            continue;
        }

        // Warning format
        if trimmed.starts_with("warning") {
            entries.push(ErrorEntry {
                file: String::new(),
                line_num: None,
                message: trimmed.to_string(),
                severity: Severity::Warning,
            });
            continue;
        }
    }

    // Group by file
    let mut groups: Vec<ErrorGroup> = Vec::new();
    for entry in entries {
        let file = if entry.file.is_empty() {
            "<unknown>".to_string()
        } else {
            entry.file.clone()
        };

        if let Some(group) = groups.iter_mut().find(|g| g.file == file) {
            group.entries.push(entry);
        } else {
            groups.push(ErrorGroup {
                file,
                entries: vec![entry],
            });
        }
    }

    groups
}

fn render_error_groups(lines: &mut Vec<Line<'static>>, groups: &[ErrorGroup], source: &str) {
    if groups.is_empty() {
        return;
    }

    lines.push(Line::from(Span::styled(
        format!(" ─── {source} ───"),
        Style::default().fg(Theme::TEXT_GHOST),
    )));

    for group in groups {
        // File header
        lines.push(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                group.file.clone(),
                Style::default()
                    .fg(Theme::BONE_DIM)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        for (i, entry) in group.entries.iter().enumerate() {
            let connector = if i == group.entries.len() - 1 {
                "└─"
            } else {
                "├─"
            };

            let color = match entry.severity {
                Severity::Error => Theme::EMBER,
                Severity::Warning => Theme::WARNING,
            };

            let location = entry
                .line_num
                .map(|n| format!("L{n}: "))
                .unwrap_or_default();

            lines.push(Line::from(vec![
                Span::styled(
                    format!("   {connector} "),
                    Style::default().fg(Theme::FG_DIM),
                ),
                Span::styled(location, Style::default().fg(Theme::DREAM)),
                Span::styled(entry.message.clone(), Style::default().fg(color)),
            ]));
        }
    }
}
