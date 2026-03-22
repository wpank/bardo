use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::PlanDetailTab;
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar;

pub fn render(
    f: &mut Frame,
    area: Rect,
    plan_detail_content: &str,
    plan_detail_scroll: usize,
    summary_content: Option<&str>,
    summary_scroll: usize,
    active_tab: PlanDetailTab,
) {
    let popup = centered_rect(85, 85, area);
    f.render_widget(Clear, popup);

    let has_summary = summary_content.is_some();

    // Build content area: if tabs, reserve 1 line for tab bar
    let (tab_area, content_area) = if has_summary {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(Rect::new(
                popup.x + 1,
                popup.y + 1,
                popup.width.saturating_sub(2),
                popup.height.saturating_sub(2),
            ));
        (Some(split[0]), split[1])
    } else {
        (
            None,
            Rect::new(
                popup.x + 1,
                popup.y + 1,
                popup.width.saturating_sub(2),
                popup.height.saturating_sub(2),
            ),
        )
    };

    // Render tab bar if summary exists
    if let Some(tab_rect) = tab_area {
        let summary_style = if active_tab == PlanDetailTab::Summary {
            Style::default().fg(Theme::BG).bg(Theme::ROSE)
        } else {
            Theme::tab_inactive_style()
        };
        let details_style = if active_tab == PlanDetailTab::PlanDetails {
            Style::default().fg(Theme::BG).bg(Theme::ROSE)
        } else {
            Theme::tab_inactive_style()
        };

        let tab_line = Line::from(vec![
            Span::styled(" ", Style::default().bg(Theme::BG_SECONDARY)),
            Span::styled(" Summary ", summary_style),
            Span::styled("  ", Style::default().bg(Theme::BG_SECONDARY)),
            Span::styled(" Plan Details ", details_style),
            Span::styled(
                " ".repeat(tab_rect.width.saturating_sub(30) as usize),
                Style::default().bg(Theme::BG_SECONDARY),
            ),
        ]);
        f.render_widget(Paragraph::new(tab_line), tab_rect);
    }

    // Select content and scroll based on active tab
    let (content, scroll) = match (has_summary, active_tab) {
        (true, PlanDetailTab::Summary) => (summary_content.unwrap_or(""), summary_scroll),
        _ => (plan_detail_content, plan_detail_scroll),
    };

    let all_lines = render_markdown(content);
    let total = all_lines.len();
    let visible = content_area.height as usize;

    let display: Vec<Line> = all_lines.into_iter().skip(scroll).take(visible).collect();

    let title = if has_summary {
        " Plan [Tab:switch Esc:close j/k:scroll PgUp/PgDn]"
    } else {
        " Plan Detail [Esc:close j/k:scroll PgUp/PgDn]"
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Theme::DREAM))
        .style(Theme::default_style());
    f.render_widget(block, popup);

    let paragraph = Paragraph::new(display);
    f.render_widget(paragraph, content_area);

    if total > visible {
        scrollbar::render_scrollbar(
            f.buffer_mut(),
            content_area,
            total,
            visible,
            scroll,
            Theme::BONE_DIM,
        );
    }
}

/// Render markdown content into styled Lines with proper formatting
fn render_markdown(content: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut in_table = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();

        // Code block fences
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                code_lang = trimmed[3..].trim().to_string();
                lines.push(Line::from(""));
                let label = if code_lang.is_empty() {
                    "  code".to_string()
                } else {
                    format!("  {code_lang}")
                };
                lines.push(Line::from(Span::styled(
                    label,
                    Style::default()
                        .fg(Theme::DREAM)
                        .add_modifier(Modifier::DIM),
                )));
            } else {
                code_lang.clear();
                lines.push(Line::from(""));
            }
            continue;
        }

        if in_code_block {
            // Code lines with syntax-aware coloring
            let styled = render_code_line(raw_line, &code_lang);
            lines.push(styled);
            continue;
        }

        // Horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            lines.push(Line::from(Span::styled(
                "  ────────────────────────────────────────",
                Style::default().fg(Theme::TEXT_GHOST),
            )));
            continue;
        }

        // Table rows (contain |)
        if trimmed.contains('|') && (trimmed.starts_with('|') || in_table) {
            in_table = true;
            // Table separator rows
            if trimmed
                .chars()
                .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
            {
                lines.push(Line::from(Span::styled(
                    format!("  {trimmed}"),
                    Style::default().fg(Theme::TEXT_GHOST),
                )));
            } else {
                // Table data rows
                let cells: Vec<&str> = trimmed.split('|').filter(|s| !s.is_empty()).collect();
                let mut spans = vec![Span::styled("  ", Style::default())];
                for (i, cell) in cells.iter().enumerate() {
                    let cell_text = cell.trim();
                    let style = if i == 0 {
                        Style::default().fg(Theme::BONE_DIM) // first column dimmer
                    } else {
                        Style::default().fg(Theme::TEXT)
                    };
                    spans.push(Span::styled(format!(" {cell_text} "), style));
                    if i < cells.len() - 1 {
                        spans.push(Span::styled("│", Style::default().fg(Theme::TEXT_GHOST)));
                    }
                }
                lines.push(Line::from(spans));
            }
            continue;
        } else {
            in_table = false;
        }

        // Empty lines
        if trimmed.is_empty() {
            lines.push(Line::from(""));
            continue;
        }

        // H1
        if trimmed.starts_with("# ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {}", &trimmed[2..]),
                Style::default()
                    .fg(Theme::BONE)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {}", "═".repeat(trimmed.len().min(50))),
                Style::default().fg(Theme::BONE_DIM),
            )));
            continue;
        }

        // H2
        if trimmed.starts_with("## ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {}", &trimmed[3..]),
                Style::default()
                    .fg(Theme::BONE)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {}", "─".repeat(trimmed.len().min(40))),
                Style::default().fg(Theme::TEXT_GHOST),
            )));
            continue;
        }

        // H3
        if trimmed.starts_with("### ") {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {}", &trimmed[4..]),
                Style::default()
                    .fg(Theme::ROSE)
                    .add_modifier(Modifier::BOLD),
            )));
            continue;
        }

        // Bullet points
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let indent = raw_line.len() - raw_line.trim_start().len();
            let pad = " ".repeat(indent + 2);
            let bullet_text = &trimmed[2..];
            let mut spans = vec![Span::styled(format!("{pad}  "), Style::default())];
            spans.extend(parse_inline_md(bullet_text));
            lines.push(Line::from(spans));
            continue;
        }

        // Numbered lists
        if trimmed.len() > 2
            && trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            if let Some(dot_pos) = trimmed.find(". ") {
                if dot_pos < 4 {
                    let num = &trimmed[..dot_pos + 1];
                    let rest = &trimmed[dot_pos + 2..];
                    let mut spans = vec![Span::styled(
                        format!("  {num} "),
                        Style::default().fg(Theme::ROSE),
                    )];
                    spans.extend(parse_inline_md(rest));
                    lines.push(Line::from(spans));
                    continue;
                }
            }
        }

        // Regular text with inline formatting
        let mut spans = vec![Span::styled("  ", Style::default())];
        spans.extend(parse_inline_md(trimmed));
        lines.push(Line::from(spans));
    }

    lines
}

/// Render a code line with basic syntax coloring
fn render_code_line(line: &str, lang: &str) -> Line<'static> {
    let bg = Theme::BG_SECONDARY;
    let base_style = Style::default().fg(Theme::DREAM).bg(bg);

    // Rust-specific syntax highlighting
    if lang == "rust" || lang == "rs" {
        let trimmed = line.trim();
        let style = if trimmed.starts_with("//") || trimmed.starts_with("///") {
            Style::default().fg(Theme::TEXT_DIM).bg(bg)
        } else if trimmed.starts_with("pub ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("use ")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("trait ")
            || trimmed.starts_with("type ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("async ")
        {
            Style::default().fg(Theme::ROSE).bg(bg)
        } else if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            Style::default().fg(Theme::WARNING).bg(bg)
        } else {
            base_style
        };

        return Line::from(vec![
            Span::styled("  ", Style::default().bg(bg)),
            Span::styled("│ ", Style::default().fg(Theme::TEXT_GHOST).bg(bg)),
            Span::styled(line.to_string(), style),
        ]);
    }

    // TOML highlighting
    if lang == "toml" {
        let trimmed = line.trim();
        let style = if trimmed.starts_with('[') {
            Style::default().fg(Theme::ROSE).bg(bg)
        } else if trimmed.starts_with('#') {
            Style::default().fg(Theme::TEXT_DIM).bg(bg)
        } else if trimmed.contains(" = ") {
            Style::default().fg(Theme::SAGE).bg(bg)
        } else {
            base_style
        };

        return Line::from(vec![
            Span::styled("  ", Style::default().bg(bg)),
            Span::styled("│ ", Style::default().fg(Theme::TEXT_GHOST).bg(bg)),
            Span::styled(line.to_string(), style),
        ]);
    }

    // Generic code
    Line::from(vec![
        Span::styled("  ", Style::default().bg(bg)),
        Span::styled("│ ", Style::default().fg(Theme::TEXT_GHOST).bg(bg)),
        Span::styled(line.to_string(), base_style),
    ])
}

/// Parse inline markdown: `code`, **bold**, *italic*
fn parse_inline_md(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut remaining = text.to_string();

    while !remaining.is_empty() {
        let backtick = remaining.find('`');
        let bold = remaining.find("**");

        let next = match (backtick, bold) {
            (Some(b), Some(a)) => {
                if b < a {
                    Some(('`', b))
                } else {
                    Some(('*', a))
                }
            }
            (Some(b), None) => Some(('`', b)),
            (None, Some(a)) => Some(('*', a)),
            (None, None) => None,
        };

        match next {
            Some(('`', pos)) => {
                if pos > 0 {
                    spans.push(Span::styled(
                        remaining[..pos].to_string(),
                        Style::default().fg(Theme::FG),
                    ));
                }
                let after = &remaining[pos + 1..];
                if let Some(end) = after.find('`') {
                    spans.push(Span::styled(
                        after[..end].to_string(),
                        Style::default().fg(Theme::DREAM).bg(Theme::BG_SECONDARY),
                    ));
                    remaining = after[end + 1..].to_string();
                } else {
                    spans.push(Span::styled(
                        remaining[pos..].to_string(),
                        Style::default().fg(Theme::FG),
                    ));
                    remaining.clear();
                }
            }
            Some(('*', pos)) => {
                if pos > 0 {
                    spans.push(Span::styled(
                        remaining[..pos].to_string(),
                        Style::default().fg(Theme::FG),
                    ));
                }
                let after = &remaining[pos + 2..];
                if let Some(end) = after.find("**") {
                    spans.push(Span::styled(
                        after[..end].to_string(),
                        Style::default()
                            .fg(Theme::BONE)
                            .add_modifier(Modifier::BOLD),
                    ));
                    remaining = after[end + 2..].to_string();
                } else {
                    spans.push(Span::styled(
                        remaining[pos..].to_string(),
                        Style::default().fg(Theme::FG),
                    ));
                    remaining.clear();
                }
            }
            _ => {
                spans.push(Span::styled(
                    remaining.clone(),
                    Style::default().fg(Theme::FG),
                ));
                remaining.clear();
            }
        }
    }

    spans
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
