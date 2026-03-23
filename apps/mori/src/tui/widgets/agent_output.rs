use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::{AgentPaneGroup, RunPlanStatus, RunState, VerifyStatus};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;

use super::scrollbar;

const AGENT_TABS: &[(AgentRole, &str)] = &[
    (AgentRole::Strategist, "1:strategist"),
    (AgentRole::Implementer, "2:implementer"),
    (AgentRole::Architect, "3:architect"),
    (AgentRole::Auditor, "4:auditor"),
    (AgentRole::Scribe, "5:scribe"),
    (AgentRole::Critic, "6:critic"),
    (AgentRole::Conductor, "7:conductor"),
];

pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere, focused: bool) {
    match state.agent_pane_group {
        AgentPaneGroup::Implementation => render_impl(f, area, state, atmosphere, focused),
        AgentPaneGroup::Verification => render_verify(f, area, state, focused),
    }
}

fn render_impl(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    atmosphere: &Atmosphere,
    focused: bool,
) {
    // In parallel mode, show per-plan dynamic agent tabs instead of role tabs.
    if !state.parallel_agents.is_empty() {
        render_impl_parallel(f, area, state, atmosphere, focused);
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_agent_tabs(f, layout[0], state, atmosphere);

    let (role, _) = AGENT_TABS
        .get(state.selected_agent_tab)
        .copied()
        .unwrap_or((AgentRole::Implementer, "impl"));

    let accent = Theme::role_accent(role);
    let output = state
        .agent_state(role)
        .map(|a| a.output.as_str())
        .unwrap_or("");
    let active = state.agent_state(role).map(|a| a.active).unwrap_or(false);

    let title = if active {
        format!(
            "{} {} [iter {}]",
            atmosphere.spinner_ethereal(),
            role.label(),
            state.current_iteration
        )
    } else {
        format!("{} [iter {}]", role.label(), state.current_iteration)
    };
    let title_style = if active {
        Style::default().fg(accent).add_modifier(Modifier::BOLD)
    } else {
        Theme::title_style()
    };

    // Build styled lines: grouped segments or empty-state message
    // Use cache to avoid re-parsing unchanged output every frame.
    let styled_lines = if output.is_empty() {
        render_empty_state(state, role, active, accent, atmosphere)
    } else if let Some(agent) = state.agent_state(role) {
        let mut cache = agent.render_cache.borrow_mut();
        if cache.last_len == output.len() {
            cache.lines.clone()
        } else {
            let segments = parse_segments(output);
            let groups = group_segments(&segments);
            let lines = render_groups(&groups, accent);
            cache.last_len = output.len();
            cache.lines = lines.clone();
            lines
        }
    } else {
        let segments = parse_segments(output);
        let groups = group_segments(&segments);
        render_groups(&groups, accent)
    };

    // Scroll logic: auto-scroll or pinned
    let visible_height = layout[1].height.saturating_sub(2) as usize;
    let total = styled_lines.len();

    let start = match state.agent_scroll {
        None => total.saturating_sub(visible_height),
        Some(offset) => offset.min(total.saturating_sub(visible_height.min(total))),
    };
    let end = (start + visible_height).min(total);

    let mut visible: Vec<Line> = styled_lines[start..end].to_vec();

    // Scroll indicators when pinned
    if state.agent_scroll.is_some() && start > 0 {
        if let Some(first) = visible.first_mut() {
            *first = Line::from(Span::styled(
                format!("▲ {} lines above", start),
                Style::default().fg(Theme::FG_DIM),
            ));
        }
    }
    if state.agent_scroll.is_some() && end < total {
        visible.push(Line::from(Span::styled(
            "[End] to resume auto-scroll",
            Style::default().fg(Theme::FG_DIM),
        )));
    }

    let border_s = if focused {
        Theme::focused_border_style()
    } else {
        Theme::unfocused_border_style()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(border_s)
        .title_style(title_style);

    let paragraph = Paragraph::new(visible)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, layout[1]);

    // Scrollbar
    if total > visible_height {
        let inner = Rect::new(
            layout[1].x + 1,
            layout[1].y + 1,
            layout[1].width.saturating_sub(2),
            layout[1].height.saturating_sub(2),
        );
        scrollbar::render_scrollbar(f.buffer_mut(), inner, total, visible_height, start, accent);
    }
}

/// Parallel mode: dynamic tabs — one per agent working on the selected plan.
fn render_impl_parallel(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    atmosphere: &Atmosphere,
    focused: bool,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let selected_base = state
        .plans
        .get(state.selected_plan_idx)
        .map(|p| p.base.as_str())
        .unwrap_or("");

    // Collect agents for this plan: implementers first (sorted by task), then others.
    let mut plan_agents: Vec<&crate::state::ParallelAgentState> = state
        .parallel_agents
        .iter()
        .filter(|p| p.plan.contains(selected_base) || selected_base.contains(p.plan.as_str()))
        .collect();
    plan_agents.sort_by_key(|p| (p.role != AgentRole::Implementer, p.task.clone()));

    // Tab bar
    {
        let selected = state.selected_agent_tab;
        let mut spans = Vec::new();
        spans.push(Span::styled(
            " [impl] ",
            Style::default().fg(Theme::ROSE).bg(Theme::BG_SECONDARY),
        ));
        for (i, pa) in plan_agents.iter().enumerate() {
            let model_short = if pa.model.is_empty() {
                shorten_model(state.config.model_for(pa.role).unwrap_or("?"))
            } else {
                shorten_model(&pa.model)
            };
            let label = if pa.role == AgentRole::Implementer {
                format!("{}:impl-{}({})", i + 1, pa.task, model_short)
            } else {
                format!("{}:{}({})", i + 1, pa.role.short(), model_short)
            };
            let icon = if pa.active {
                format!("{}", atmosphere.spinner())
            } else if pa.input_tokens > 0 {
                "✓".to_string()
            } else {
                "·".to_string()
            };
            let tab_label = format!(" {icon}{label} ");
            let accent = Theme::role_accent(pa.role);
            let style = if i == selected {
                Style::default().fg(Theme::BG).bg(accent)
            } else {
                Theme::tab_inactive_style()
            };
            spans.push(Span::styled(tab_label, style));
            spans.push(Span::styled(" ", Style::default().bg(Theme::BG_SECONDARY)));
        }
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (layout[0].width as usize).saturating_sub(used);
        spans.push(Span::styled(
            " ".repeat(remaining),
            Style::default().bg(Theme::BG_SECONDARY),
        ));
        let line = Line::from(spans);
        f.render_widget(Paragraph::new(line), layout[0]);
    }

    // Content area: show the selected parallel agent's output
    let selected_pa = plan_agents.get(state.selected_agent_tab);
    let (output_str, active, accent, instance_label) = if let Some(pa) = selected_pa {
        (
            pa.output.as_str(),
            pa.active,
            Theme::role_accent(pa.role),
            pa.instance_id.as_str(),
        )
    } else {
        ("", false, Theme::DREAM, "")
    };

    let title = if active {
        format!(
            "{} {} [{}]",
            atmosphere.spinner_ethereal(),
            if selected_pa.map(|p| p.role) == Some(AgentRole::Implementer) {
                "implementer"
            } else {
                selected_pa.map(|p| p.role.label()).unwrap_or("agent")
            },
            instance_label
        )
    } else {
        format!(
            "{} [{}]",
            selected_pa.map(|p| p.role.label()).unwrap_or("agent"),
            instance_label
        )
    };
    let title_style = if active {
        Style::default().fg(accent).add_modifier(Modifier::BOLD)
    } else {
        Theme::title_style()
    };

    let styled_lines: Vec<Line> = if output_str.is_empty() {
        let msg = if plan_agents.is_empty() {
            "  no agents for this plan"
        } else if selected_pa.map(|p| p.turn_started).unwrap_or(false) {
            "  processing prompt..."
        } else {
            "  waiting..."
        };
        vec![Line::from(Span::styled(
            msg,
            Style::default().fg(Theme::TEXT_DIM),
        ))]
    } else if let Some(pa) = selected_pa {
        let mut cache = pa.render_cache.borrow_mut();
        if cache.last_len == output_str.len() {
            cache.lines.clone()
        } else {
            let segments = parse_segments(output_str);
            let groups = group_segments(&segments);
            let lines = render_groups(&groups, accent);
            cache.last_len = output_str.len();
            cache.lines = lines.clone();
            lines
        }
    } else {
        let segments = parse_segments(output_str);
        let groups = group_segments(&segments);
        render_groups(&groups, accent)
    };

    let visible_height = layout[1].height.saturating_sub(2) as usize;
    let total = styled_lines.len();
    let start = match state.agent_scroll {
        None => total.saturating_sub(visible_height),
        Some(offset) => offset.min(total.saturating_sub(visible_height.min(total))),
    };
    let end = (start + visible_height).min(total);

    let mut visible: Vec<Line> = styled_lines[start..end].to_vec();
    if state.agent_scroll.is_some() && start > 0 {
        if let Some(first) = visible.first_mut() {
            *first = Line::from(Span::styled(
                format!("▲ {} lines above", start),
                Style::default().fg(Theme::FG_DIM),
            ));
        }
    }
    if state.agent_scroll.is_some() && end < total {
        visible.push(Line::from(Span::styled(
            "[End] to resume auto-scroll",
            Style::default().fg(Theme::FG_DIM),
        )));
    }

    let border_s = if focused {
        Theme::focused_border_style()
    } else {
        Theme::unfocused_border_style()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(border_s)
        .title_style(title_style);

    let paragraph = Paragraph::new(visible)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, layout[1]);

    if total > visible_height {
        let inner = Rect::new(
            layout[1].x + 1,
            layout[1].y + 1,
            layout[1].width.saturating_sub(2),
            layout[1].height.saturating_sub(2),
        );
        scrollbar::render_scrollbar(f.buffer_mut(), inner, total, visible_height, start, accent);
    }
}

/// Render empty-state message when agent has no output
fn render_empty_state(
    state: &RunState,
    role: AgentRole,
    active: bool,
    accent: ratatui::style::Color,
    atmosphere: &Atmosphere,
) -> Vec<Line<'static>> {
    let is_completed_prior = state
        .current_plan()
        .map(|p| p.status == RunPlanStatus::CompletedPrior)
        .unwrap_or(false);

    if is_completed_prior {
        vec![Line::from(Span::styled(
            "  - completed in prior run",
            Style::default().fg(Theme::SAGE),
        ))]
    } else if active {
        vec![Line::from(Span::styled(
            format!("  {} starting...", atmosphere.spinner()),
            Style::default().fg(accent),
        ))]
    } else if state.agent_state(role).is_some() {
        vec![Line::from(Span::styled(
            "  - idle",
            Style::default().fg(Theme::TEXT_DIM),
        ))]
    } else if role == AgentRole::Conductor {
        // Conductor shows a helpful message when waiting to be consulted
        vec![
            Line::from(Span::styled(
                "  Meta-agent — monitors pipeline and intervenes when needed",
                Style::default().fg(Theme::TEXT_DIM),
            )),
            Line::from(Span::styled(
                "  Consulted at: gate results, verdict evaluation, iteration loops",
                Style::default().fg(Theme::TEXT_GHOST),
            )),
        ]
    } else {
        vec![Line::from(Span::styled(
            "  - not spawned",
            Style::default().fg(Theme::TEXT_DIM),
        ))]
    }
}

/// Content segment types detected from the output stream
#[derive(Debug, PartialEq)]
enum SegmentKind {
    Thinking,
    Heading,
    ToolUse,
    Code,
    Success,
    Error,
    Blank,
    TurnMarker,
}

#[derive(Debug)]
struct Segment {
    kind: SegmentKind,
    text: String,
}

/// A group of consecutive same-kind segments
struct SegmentGroup {
    kind: SegmentKind,
    lines: Vec<String>,
}

/// Parse raw output into tagged segments
fn parse_segments(output: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut in_code_block = false;

    // Split on newlines first, then break long blobby lines at sentence boundaries
    let raw_lines: Vec<&str> = output.lines().collect();
    let lines: Vec<&str> = raw_lines
        .iter()
        .flat_map(|line| {
            if line.len() > 120 && !line.trim_start().starts_with("```") {
                split_on_sentences(line)
            } else {
                vec![*line]
            }
        })
        .collect();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("──── turn ") {
            segments.push(Segment {
                kind: SegmentKind::TurnMarker,
                text: line.to_string(),
            });
            continue;
        }

        if trimmed.is_empty() {
            segments.push(Segment {
                kind: SegmentKind::Blank,
                text: String::new(),
            });
            continue;
        }

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            segments.push(Segment {
                kind: SegmentKind::Code,
                text: line.to_string(),
            });
            continue;
        }

        if in_code_block {
            segments.push(Segment {
                kind: SegmentKind::Code,
                text: line.to_string(),
            });
            continue;
        }

        if trimmed.starts_with("# ") || trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            segments.push(Segment {
                kind: SegmentKind::Heading,
                text: line.to_string(),
            });
            continue;
        }

        if trimmed.starts_with("▸ ")
            || trimmed.starts_with("> ")
            || trimmed.starts_with("$ ")
            || trimmed.starts_with("Running ")
            || trimmed.starts_with("Reading ")
            || trimmed.starts_with("Writing ")
            || trimmed.starts_with("Editing ")
            || trimmed.starts_with("Created ")
        {
            segments.push(Segment {
                kind: SegmentKind::ToolUse,
                text: line.to_string(),
            });
            continue;
        }

        if trimmed.contains("✓")
            || trimmed.contains("PASS")
            || trimmed.contains("APPROVE")
            || trimmed.starts_with("ok ")
        {
            segments.push(Segment {
                kind: SegmentKind::Success,
                text: line.to_string(),
            });
            continue;
        }

        if trimmed.contains("ERROR")
            || trimmed.contains("FAILED")
            || trimmed.contains("REVISE")
            || trimmed.starts_with("error")
            || trimmed.starts_with("error[")
        {
            segments.push(Segment {
                kind: SegmentKind::Error,
                text: line.to_string(),
            });
            continue;
        }

        segments.push(Segment {
            kind: SegmentKind::Thinking,
            text: line.to_string(),
        });
    }

    segments
}

/// Group consecutive same-kind segments together
fn group_segments(segments: &[Segment]) -> Vec<SegmentGroup> {
    let mut groups: Vec<SegmentGroup> = Vec::new();

    for seg in segments {
        // Turn markers always stand alone
        if seg.kind == SegmentKind::TurnMarker {
            groups.push(SegmentGroup {
                kind: SegmentKind::TurnMarker,
                lines: vec![seg.text.clone()],
            });
            continue;
        }

        // Blanks don't extend groups -- they become their own group
        if seg.kind == SegmentKind::Blank {
            groups.push(SegmentGroup {
                kind: SegmentKind::Blank,
                lines: vec![String::new()],
            });
            continue;
        }

        // Check if we can extend the current group
        let extend = groups.last().map(|g| g.kind == seg.kind).unwrap_or(false);
        if extend {
            groups.last_mut().unwrap().lines.push(seg.text.clone());
        } else {
            groups.push(SegmentGroup {
                kind: match seg.kind {
                    SegmentKind::Thinking => SegmentKind::Thinking,
                    SegmentKind::Heading => SegmentKind::Heading,
                    SegmentKind::ToolUse => SegmentKind::ToolUse,
                    SegmentKind::Code => SegmentKind::Code,
                    SegmentKind::Success => SegmentKind::Success,
                    SegmentKind::Error => SegmentKind::Error,
                    SegmentKind::Blank => SegmentKind::Blank,
                    SegmentKind::TurnMarker => SegmentKind::TurnMarker,
                },
                lines: vec![seg.text.clone()],
            });
        }
    }

    groups
}

const BG_BUBBLE_ALT: ratatui::style::Color = ratatui::style::Color::Rgb(18, 16, 22);

const RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "pub", "use", "struct", "enum", "impl", "match", "if", "else", "for",
    "while", "return", "self", "Self", "mod", "crate", "super", "async", "await", "where", "trait",
    "type", "const", "static",
];

/// Simple per-line syntax coloring for code blocks.
fn syntax_color_line(text: &str, bg: ratatui::style::Color) -> Vec<Span<'static>> {
    let trimmed = text.trim_start();

    // ``` fence lines
    if trimmed.starts_with("```") {
        return vec![Span::styled(
            text.to_string(),
            Style::default().fg(Theme::TEXT_GHOST).bg(bg),
        )];
    }

    // Comment lines
    if trimmed.starts_with("//") {
        return vec![Span::styled(
            text.to_string(),
            Style::default().fg(Theme::TEXT_DIM).bg(bg),
        )];
    }

    // Tokenize: walk char by char and build spans
    let mut spans: Vec<Span<'static>> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut buf = String::new();
    let base_style = Style::default().fg(Theme::DREAM).bg(bg);

    while i < len {
        let ch = chars[i];

        // String literals
        if ch == '"' || ch == '\'' {
            if !buf.is_empty() {
                flush_word_buf(&mut buf, &mut spans, bg);
            }
            let quote = ch;
            let mut s = String::new();
            s.push(ch);
            i += 1;
            while i < len && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < len {
                    s.push(chars[i]);
                    s.push(chars[i + 1]);
                    i += 2;
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            if i < len {
                s.push(chars[i]);
                i += 1;
            }
            spans.push(Span::styled(s, Style::default().fg(Theme::SAGE).bg(bg)));
            continue;
        }

        // Word boundaries
        if ch.is_alphanumeric() || ch == '_' {
            buf.push(ch);
            i += 1;
        } else {
            if !buf.is_empty() {
                flush_word_buf(&mut buf, &mut spans, bg);
            }
            spans.push(Span::styled(ch.to_string(), base_style));
            i += 1;
        }
    }

    if !buf.is_empty() {
        flush_word_buf(&mut buf, &mut spans, bg);
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), base_style));
    }

    spans
}

fn flush_word_buf(buf: &mut String, spans: &mut Vec<Span<'static>>, bg: ratatui::style::Color) {
    let word = std::mem::take(buf);
    if RUST_KEYWORDS.contains(&word.as_str()) {
        spans.push(Span::styled(word, Style::default().fg(Theme::BONE).bg(bg)));
    } else {
        spans.push(Span::styled(word, Style::default().fg(Theme::DREAM).bg(bg)));
    }
}

/// Gradient horizontal rule: fades ROSE_DIM -> TEXT_GHOST across the width.
fn gradient_hr(width: usize) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for i in 0..width {
        let t = if width <= 1 {
            0.5
        } else {
            i as f64 / (width - 1) as f64
        };
        // Fade from ROSE_DIM (122,80,96) to TEXT_GHOST (80,64,80)
        let r = (122.0 + (80.0 - 122.0) * t) as u8;
        let g = (80.0 + (64.0 - 80.0) * t) as u8;
        let b = (96.0 + (80.0 - 96.0) * t) as u8;
        spans.push(Span::styled(
            "\u{2500}".to_string(),
            Style::default().fg(ratatui::style::Color::Rgb(r, g, b)),
        ));
    }
    spans
}

/// Render grouped segments into styled Lines with left-side icons and visual grouping
fn render_groups(groups: &[SegmentGroup], accent: ratatui::style::Color) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut turn_count: usize = 0; // track turn parity for alternating bg

    for group in groups {
        // Determine bubble background based on turn parity
        let bubble_bg = if turn_count % 2 == 0 {
            Theme::BG_SECONDARY
        } else {
            BG_BUBBLE_ALT
        };

        match group.kind {
            SegmentKind::Blank => {
                lines.push(Line::from(""));
            }
            SegmentKind::Heading => {
                if !lines.is_empty() {
                    lines.push(Line::from(""));
                }
                for text in &group.lines {
                    lines.push(Line::from(vec![
                        Span::styled(" \u{25C6} ", Style::default().fg(Theme::BONE)),
                        Span::styled(
                            text.trim_start_matches('#').trim().to_string(),
                            Style::default()
                                .fg(Theme::BONE)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
                lines.push(Line::from(""));
            }
            SegmentKind::Thinking => {
                for (i, text) in group.lines.iter().enumerate() {
                    let trimmed = text.trim();
                    let gutter = if i == 0 {
                        Span::styled(" \u{25E6} ", Style::default().fg(Theme::TEXT_DIM))
                    } else {
                        Span::styled(" \u{2502} ", Style::default().fg(Theme::TEXT_GHOST))
                    };

                    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                        let content = &trimmed[2..];
                        let mut spans = vec![
                            gutter,
                            Span::styled("\u{00B7} ", Style::default().fg(Theme::ROSE_DIM)),
                        ];
                        spans.extend(parse_inline_markdown(content, Theme::TEXT));
                        lines.push(Line::from(spans));
                    } else if trimmed.len() > 2
                        && trimmed
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                        && trimmed.contains(". ")
                    {
                        if let Some(dot) = trimmed.find(". ") {
                            if dot < 4 {
                                let num = &trimmed[..dot + 1];
                                let rest = &trimmed[dot + 2..];
                                let mut spans = vec![
                                    gutter,
                                    Span::styled(
                                        format!("{num} "),
                                        Style::default().fg(Theme::ROSE),
                                    ),
                                ];
                                spans.extend(parse_inline_markdown(rest, Theme::TEXT));
                                lines.push(Line::from(spans));
                                continue;
                            }
                        }
                        let mut spans = vec![gutter];
                        spans.extend(parse_inline_markdown(trimmed, Theme::TEXT));
                        lines.push(Line::from(spans));
                    } else {
                        let mut spans = vec![gutter];
                        spans.extend(parse_inline_markdown(trimmed, Theme::TEXT));
                        lines.push(Line::from(spans));
                    }
                }
            }
            SegmentKind::ToolUse => {
                for (i, text) in group.lines.iter().enumerate() {
                    // Use contextual icons based on tool type
                    let tool_icon = if i == 0 {
                        let trimmed = text.trim();
                        if trimmed.starts_with("Editing ") || trimmed.starts_with("Writing ") {
                            " ✎ "
                        } else if trimmed.starts_with("Reading ") {
                            " ◇ "
                        } else {
                            " ⚙ "
                        }
                    } else {
                        " │ "
                    };
                    let gutter = Span::styled(tool_icon, Style::default().fg(accent));
                    lines.push(Line::from(vec![
                        gutter,
                        Span::styled(text.clone(), Style::default().fg(accent).bg(bubble_bg)),
                    ]));
                }
            }
            SegmentKind::Code => {
                for text in &group.lines {
                    let gutter = Span::styled(" \u{2503} ", Style::default().fg(Theme::DREAM));
                    let mut spans = vec![gutter];
                    spans.extend(syntax_color_line(text, bubble_bg));
                    lines.push(Line::from(spans));
                }
            }
            SegmentKind::Success => {
                for text in &group.lines {
                    lines.push(Line::from(vec![
                        Span::styled(" \u{2713} ", Style::default().fg(Theme::SAGE)),
                        Span::styled(text.clone(), Style::default().fg(Theme::SAGE)),
                    ]));
                }
            }
            SegmentKind::Error => {
                for text in &group.lines {
                    lines.push(Line::from(vec![
                        Span::styled(" \u{2717} ", Style::default().fg(Theme::EMBER)),
                        Span::styled(text.clone(), Style::default().fg(Theme::EMBER)),
                    ]));
                }
            }
            SegmentKind::TurnMarker => {
                turn_count += 1;
                // Parse: "──── turn N · role · model · HH:MM:SS ────"
                lines.push(Line::from(""));
                for text in &group.lines {
                    let parts: Vec<&str> = text
                        .trim()
                        .trim_matches('\u{2500}')
                        .trim()
                        .split(" \u{00B7} ")
                        .collect();
                    let turn_part = parts.first().copied().unwrap_or("turn ?");
                    let role_part = parts.get(1).copied().unwrap_or("");
                    let model_part = parts.get(2).copied().unwrap_or("");
                    let time_part = parts.get(3).copied().unwrap_or("");

                    // Gradient horizontal rule
                    let content_len =
                        turn_part.len() + role_part.len() + model_part.len() + time_part.len() + 12;
                    let total_hr_width = 60usize.saturating_sub(content_len);
                    let left_width = total_hr_width / 2;
                    let right_width = total_hr_width - left_width;

                    let mut spans = gradient_hr(left_width);
                    spans.push(Span::styled(
                        format!(" {turn_part}"),
                        Style::default().fg(Theme::TEXT_DIM),
                    ));
                    spans.push(Span::styled(
                        " \u{00B7} ",
                        Style::default().fg(Theme::TEXT_GHOST),
                    ));
                    spans.push(Span::styled(
                        format!(" {model_part} "),
                        Style::default()
                            .fg(Theme::DREAM)
                            .bg(Theme::BG_SECONDARY)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        " \u{00B7} ",
                        Style::default().fg(Theme::TEXT_GHOST),
                    ));
                    spans.push(Span::styled(
                        role_part.to_string(),
                        Style::default().fg(accent),
                    ));
                    spans.push(Span::styled(
                        " \u{00B7} ",
                        Style::default().fg(Theme::TEXT_GHOST),
                    ));
                    spans.push(Span::styled(
                        time_part.to_string(),
                        Style::default().fg(Theme::DREAM),
                    ));
                    spans.push(Span::styled(" ".to_string(), Style::default()));
                    spans.extend(gradient_hr(right_width));

                    lines.push(Line::from(spans));
                }
                lines.push(Line::from(""));
            }
        }
    }

    lines
}

/// Detect if a string looks like a file path (contains '/' and ends with a known extension).
fn is_file_path(s: &str) -> bool {
    const EXTENSIONS: &[&str] = &[
        ".rs", ".toml", ".ts", ".tsx", ".md", ".json", ".yaml", ".yml", ".lock",
    ];
    s.contains('/') && EXTENSIONS.iter().any(|ext| s.ends_with(ext))
}

/// Parse inline markdown: `code`, **bold**, file paths, and regular text
fn parse_inline_markdown(text: &str, base_color: ratatui::style::Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut remaining = text.to_string();

    while !remaining.is_empty() {
        // Find first backtick or double asterisk
        let backtick_pos = remaining.find('`');
        let bold_pos = remaining.find("**");

        let next = match (backtick_pos, bold_pos) {
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
                // Text before backtick — check for file paths
                if pos > 0 {
                    spans.extend(style_with_paths(&remaining[..pos], base_color));
                }
                let after = &remaining[pos + 1..];
                if let Some(end) = after.find('`') {
                    spans.push(Span::styled(
                        after[..end].to_string(),
                        Style::default().fg(Theme::DREAM).bg(Theme::BG_SECONDARY),
                    ));
                    remaining = after[end + 1..].to_string();
                } else {
                    spans.extend(style_with_paths(&remaining[pos..], base_color));
                    remaining.clear();
                }
            }
            Some(('*', pos)) => {
                if pos > 0 {
                    spans.extend(style_with_paths(&remaining[..pos], base_color));
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
                    spans.extend(style_with_paths(&remaining[pos..], base_color));
                    remaining.clear();
                }
            }
            _ => {
                spans.extend(style_with_paths(&remaining, base_color));
                remaining.clear();
            }
        }
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            text.to_string(),
            Style::default().fg(base_color),
        ));
    }

    spans
}

/// Style text, detecting embedded file paths and highlighting them.
fn style_with_paths(text: &str, base_color: ratatui::style::Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    // Split on whitespace and check each word for file paths
    let mut last = 0;
    for (i, word) in text.split_whitespace().enumerate() {
        let word_start = text[last..].find(word).map(|p| last + p).unwrap_or(last);
        // Add any whitespace/text before the word
        if word_start > last {
            spans.push(Span::styled(
                text[last..word_start].to_string(),
                Style::default().fg(base_color),
            ));
        }
        if is_file_path(word) {
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(Theme::BONE_DIM).bg(Theme::BG_RAISED),
            ));
        } else {
            spans.push(Span::styled(
                word.to_string(),
                Style::default().fg(base_color),
            ));
        }
        last = word_start + word.len();
        let _ = i;
    }
    if last < text.len() {
        spans.push(Span::styled(
            text[last..].to_string(),
            Style::default().fg(base_color),
        ));
    }
    if spans.is_empty() {
        spans.push(Span::styled(
            text.to_string(),
            Style::default().fg(base_color),
        ));
    }
    spans
}

/// Split a blobby no-newline string into chunks at sentence boundaries.
/// Handles both `. I` (period-space-uppercase) and `.I` (period-uppercase, no space).
fn split_on_sentences(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return vec![];
    }

    let mut splits = Vec::new();
    let mut last = 0;
    let bytes = text.as_bytes();

    for i in 0..bytes.len().saturating_sub(1) {
        if bytes[i] == b'.' && i + 1 < bytes.len() {
            let (split_at, next_char) = if bytes[i + 1] == b' ' && i + 2 < bytes.len() {
                // ". X" — period, space, then check next char
                (i + 2, bytes[i + 2])
            } else if bytes[i + 1].is_ascii_uppercase() {
                // ".X" — period directly followed by uppercase (no space)
                (i + 1, bytes[i + 1])
            } else {
                continue;
            };
            if next_char.is_ascii_uppercase() {
                if split_at > last {
                    splits.push(&text[last..split_at]);
                    last = split_at;
                }
            }
        } else if bytes[i] == b';' && i + 1 < bytes.len() && bytes[i + 1] == b' ' {
            // "; " — semicolon as split point
            let end = i + 2;
            if end > last && end < bytes.len() {
                splits.push(&text[last..end]);
                last = end;
            }
        }
    }

    if last < text.len() {
        splits.push(&text[last..]);
    }

    splits
}

fn render_verify(f: &mut Frame, area: Rect, state: &RunState, focused: bool) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    // Verify tab bar
    let selected = state.selected_verify_idx;
    let mut spans = Vec::new();
    spans.push(Span::styled(
        " [verify] ",
        Style::default().fg(Theme::DREAM).bg(Theme::BG_SECONDARY),
    ));

    for (i, entry) in state.verify_entries.iter().enumerate() {
        let icon = match &entry.status {
            VerifyStatus::Pending => "○",
            VerifyStatus::Running => "◌",
            VerifyStatus::Passed => "✓",
            VerifyStatus::Failed(_) => "✗",
        };
        let icon_color = match &entry.status {
            VerifyStatus::Pending => Theme::TEXT_DIM,
            VerifyStatus::Running => Theme::DREAM,
            VerifyStatus::Passed => Theme::SAGE,
            VerifyStatus::Failed(_) => Theme::EMBER,
        };
        let label = format!(" {icon} {}:verify ", entry.plan_num);
        let style = if i == selected {
            Style::default().fg(Theme::BG).bg(Theme::DREAM)
        } else {
            Style::default().fg(icon_color).bg(Theme::BG_SECONDARY)
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::styled(" ", Style::default().bg(Theme::BG_SECONDARY)));
    }

    if state.verify_entries.is_empty() {
        spans.push(Span::styled(
            " no verification tasks ",
            Style::default().fg(Theme::TEXT_DIM).bg(Theme::BG_SECONDARY),
        ));
    }

    let used: usize = spans.iter().map(|s| s.content.len()).sum();
    let remaining = (area.width as usize).saturating_sub(used);
    spans.push(Span::styled(
        " ".repeat(remaining),
        Style::default().bg(Theme::BG_SECONDARY),
    ));
    let tab_line = Line::from(spans);
    f.render_widget(Paragraph::new(tab_line), layout[0]);

    // Content area
    let entry = state.verify_entries.get(selected);
    let output = entry
        .map(|e| e.output.as_str())
        .unwrap_or("No verification output.");
    let status_label = entry
        .map(|e| match &e.status {
            VerifyStatus::Pending => "pending".to_string(),
            VerifyStatus::Running => "running".to_string(),
            VerifyStatus::Passed => "passed".to_string(),
            VerifyStatus::Failed(msg) => format!("failed: {msg}"),
        })
        .unwrap_or_default();

    let plan_label = entry.map(|e| e.plan_base.as_str()).unwrap_or("verify");
    let title = format!("{plan_label} — {status_label} [h/l:switch v:impl]");

    let lines: Vec<Line> = output
        .lines()
        .map(|l| {
            Line::from(Span::styled(
                l.to_string(),
                Style::default().fg(Theme::TEXT),
            ))
        })
        .collect();

    let border_color = if focused {
        Theme::DREAM
    } else {
        Theme::TEXT_PHANTOM
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .border_style(Style::default().fg(border_color))
        .title_style(Style::default().fg(Theme::DREAM));

    let visible_height = layout[1].height.saturating_sub(2) as usize;
    let total = lines.len();
    let start = total.saturating_sub(visible_height);
    let visible: Vec<Line> = lines.into_iter().skip(start).take(visible_height).collect();

    let paragraph = Paragraph::new(visible)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, layout[1]);
}

fn render_agent_tabs(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let selected = state.selected_agent_tab;
    let mut spans = Vec::new();

    // Group indicator
    spans.push(Span::styled(
        " [impl] ",
        Style::default().fg(Theme::ROSE).bg(Theme::BG_SECONDARY),
    ));

    for (i, (role, label)) in AGENT_TABS.iter().enumerate() {
        let is_active = state.agent_state(*role).map(|a| a.active).unwrap_or(false);
        let has_run = state
            .agent_state(*role)
            .map(|a| a.input_tokens > 0)
            .unwrap_or(false);

        let (is_active, has_run) = if !state.parallel_agents.is_empty() {
            let base = state
                .plans
                .get(state.selected_plan_idx)
                .map(|p| p.base.as_str())
                .unwrap_or("");
            let active = state.parallel_agents.iter().any(|p| {
                p.role == *role && p.active && (p.plan.contains(base) || base.contains(&p.plan))
            });
            let ran = state.parallel_agents.iter().any(|p| {
                p.role == *role
                    && p.input_tokens > 0
                    && (p.plan.contains(base) || base.contains(&p.plan))
            });
            (active, ran)
        } else {
            (is_active, has_run)
        };

        let icon = if is_active {
            format!("{}", atmosphere.spinner())
        } else if has_run {
            "✓".to_string()
        } else {
            "·".to_string()
        };
        let tab_label = format!(" {icon}{label} ");
        let style = if i == selected {
            Style::default().fg(Theme::BG).bg(Theme::role_accent(*role))
        } else {
            Theme::tab_inactive_style()
        };
        spans.push(Span::styled(tab_label, style));
        spans.push(Span::styled(" ", Style::default().bg(Theme::BG_SECONDARY)));
    }

    let used: usize = spans.iter().map(|s| s.content.len()).sum();
    let remaining = (area.width as usize).saturating_sub(used);
    spans.push(Span::styled(
        " ".repeat(remaining),
        Style::default().bg(Theme::BG_SECONDARY),
    ));

    let line = Line::from(spans);
    let p = Paragraph::new(line);
    f.render_widget(p, area);
}

fn shorten_model(slug: &str) -> String {
    slug.replace("gpt-", "")
        .replace("-codex", "c")
        .replace("-mini", "m")
}
