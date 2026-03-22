use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::agent::{AgentBackend, AgentRole};
use crate::state::RunState;
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar;

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_settings(f, sections[0], state);
    render_agent_status(f, sections[1], state);
}

fn render_settings(f: &mut Frame, area: Rect, state: &RunState) {
    let cfg = &state.config;
    let sel = cfg.selected_row;
    let visible_height = area.height.saturating_sub(2) as usize;

    let mut lines: Vec<Line> = Vec::new();
    let mut row_idx: usize = 0;

    // ─── Section 0: Backend Defaults ───
    lines.push(section_header("Backend Defaults", cfg.active_section == 0));
    lines.push(config_row(
        row_idx,
        sel,
        "Codex default",
        &format!("< {} >", display_model(cfg, &cfg.codex_default_model)),
    ));
    row_idx += 1;
    lines.push(config_row(
        row_idx,
        sel,
        "Cursor default",
        &format!("< {} >", display_model(cfg, &cfg.cursor_default_model)),
    ));
    row_idx += 1;
    lines.push(config_row(
        row_idx,
        sel,
        "Claude default",
        &format!("< {} >", display_model(cfg, &cfg.claude_default_model)),
    ));
    row_idx += 1;
    {
        let m = &cfg.conductor_model;
        let badge = AgentBackend::from_model(m).short();
        lines.push(config_row(
            row_idx,
            sel,
            "Conductor model",
            &format!("< {} > [{}]", display_model(cfg, m), badge),
        ));
    }
    row_idx += 1;
    {
        let fb_text = match &cfg.fallback_model {
            Some(m) => {
                let badge = AgentBackend::from_model(m).short();
                format!("< {} > [{}]", display_model(cfg, m), badge)
            }
            None => "< (none) >".to_string(),
        };
        lines.push(config_row(row_idx, sel, "Fallback model", &fb_text));
    }
    row_idx += 1;

    lines.push(Line::from(""));

    // ─── Section 1: Per-Role Overrides ───
    lines.push(section_header(
        "Per-Role Overrides",
        cfg.active_section == 1,
    ));
    // Conductor role override
    {
        let role = AgentRole::Conductor;
        let key = role.label();
        let (val_text, is_override) = if let Some(m) = cfg.role_models.get(key) {
            let badge = AgentBackend::from_model(m).short();
            (format!("< {} > [{}]", display_model(cfg, m), badge), true)
        } else {
            let m = &cfg.conductor_model;
            let badge = AgentBackend::from_model(m).short();
            (format!("  ({}: {})", badge, shorten_model(m)), false)
        };
        lines.push(if is_override {
            config_row(row_idx, sel, "conductor", &val_text)
        } else {
            config_row_dim(row_idx, sel, "conductor", &val_text)
        });
        row_idx += 1;
    }
    // ALL_AGENTS role overrides
    for &role in &AgentRole::ALL_AGENTS {
        let key = role.label();
        let default_model = match role.backend() {
            AgentBackend::Cursor => cfg.cursor_default_model.as_str(),
            AgentBackend::Claude => cfg.claude_default_model.as_str(),
            AgentBackend::Codex => cfg.codex_default_model.as_str(),
        };
        let (val_text, is_override) = if let Some(m) = cfg.role_models.get(key) {
            let badge = AgentBackend::from_model(m).short();
            (format!("< {} > [{}]", display_model(cfg, m), badge), true)
        } else {
            let badge = role.backend().short();
            (
                format!("  ({}: {})", badge, shorten_model(default_model)),
                false,
            )
        };
        lines.push(if is_override {
            config_row(row_idx, sel, role.short(), &val_text)
        } else {
            config_row_dim(row_idx, sel, role.short(), &val_text)
        });
        row_idx += 1;
    }

    lines.push(Line::from(""));

    // ─── Section 2: Context & Effort ───
    lines.push(section_header("Context & Effort", cfg.active_section == 2));
    // Global context limit
    let ctx_label = format!("{}k", cfg.context_limit_k);
    lines.push(config_row(
        row_idx,
        sel,
        "Global context limit",
        &format!("< {} >", ctx_label),
    ));
    row_idx += 1;
    // Per-role context overrides (ALL_AGENTS)
    for &role in &AgentRole::ALL_AGENTS {
        let key = role.label();
        let role_ctx = cfg.role_context_k.get(key);
        let label = format!("{} ctx", role.short());
        let value = match role_ctx {
            Some(k) => format!("< {}k >", k),
            None => format!("  (default: {}k)", cfg.context_limit_k),
        };
        lines.push(config_row(row_idx, sel, &label, &value));
        row_idx += 1;
    }
    // Global reasoning effort
    lines.push(config_row(
        row_idx,
        sel,
        "Reasoning effort",
        &format!("< {} >", cfg.default_effort.label()),
    ));
    row_idx += 1;

    lines.push(Line::from(""));

    // ─── Section 3: Agent Toggles ───
    lines.push(section_header("Agent Toggles", cfg.active_section == 3));
    lines.push(toggle_row(row_idx, sel, "Architect", cfg.architect_enabled));
    row_idx += 1;
    lines.push(toggle_row(row_idx, sel, "Auditor", cfg.auditor_enabled));
    row_idx += 1;
    lines.push(toggle_row(row_idx, sel, "Scribe", cfg.scribe_enabled));
    row_idx += 1;
    lines.push(toggle_row(row_idx, sel, "Critic", cfg.critic_enabled));
    row_idx += 1;

    lines.push(Line::from(""));

    // ─── Section 4: Execution ───
    lines.push(section_header("Execution", cfg.active_section == 4));
    lines.push(toggle_row(
        row_idx,
        sel,
        "Parallel mode",
        cfg.parallel_enabled,
    ));
    row_idx += 1;
    lines.push(config_row(
        row_idx,
        sel,
        "Max agents",
        &format!("< {} >", cfg.max_agents),
    ));
    row_idx += 1;
    lines.push(toggle_row(
        row_idx,
        sel,
        "Auto-advance batch",
        cfg.auto_advance_batch,
    ));
    row_idx += 1;
    lines.push(toggle_row(
        row_idx,
        sel,
        "Auto-advance plan",
        cfg.auto_advance_plan,
    ));
    row_idx += 1;
    lines.push(toggle_row(row_idx, sel, "Pre-planning", cfg.pre_plan));
    row_idx += 1;
    lines.push(config_row(
        row_idx,
        sel,
        "Max iterations",
        &format!("< {} >", cfg.max_iterations),
    ));
    row_idx += 1;
    lines.push(toggle_row(row_idx, sel, "Skip tests", cfg.skip_tests));
    row_idx += 1;
    lines.push(toggle_row(row_idx, sel, "Clippy gate", cfg.clippy_enabled));
    row_idx += 1;

    lines.push(Line::from(""));

    // Apply button
    let apply_sel = row_idx == sel;
    let apply_style = if apply_sel {
        Style::default()
            .fg(Theme::BONE)
            .bg(Theme::BG_HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::ROSE)
    };
    let apply_bg = if apply_sel {
        Theme::BG_HIGHLIGHT
    } else {
        Theme::BG
    };
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default().bg(apply_bg)),
        Span::styled("[ Apply & Save ]", apply_style),
    ]));

    let total = lines.len();

    // Scroll to keep selected row visible (maps row → approximate line index)
    let (section, _) = cfg.section_of_row(sel);
    let approx_line = sel + 1 + 2 * section;
    let scroll = approx_line.saturating_sub(visible_height / 2);

    let visible: Vec<Line> = lines
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Configuration [j/k:nav h/l:cycle Enter:toggle]")
        .style(Theme::block_style())
        .border_style(Style::default().fg(Theme::ROSE))
        .title_style(Theme::title_style());

    let paragraph = Paragraph::new(visible).block(block);
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
            scroll,
            Theme::ROSE,
        );
    }
}

fn section_header<'a>(title: &str, active: bool) -> Line<'a> {
    let style = if active {
        Style::default()
            .fg(Theme::BONE)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::BONE_DIM)
    };
    Line::from(vec![
        Span::styled(format!(" ─── {title} "), style),
        Span::styled(
            "───────────────────────────",
            Style::default().fg(Theme::TEXT_GHOST),
        ),
    ])
}

fn config_row<'a>(idx: usize, selected: usize, label: &str, value: &str) -> Line<'a> {
    let is_sel = idx == selected;
    let bg = if is_sel {
        Theme::BG_HIGHLIGHT
    } else {
        Theme::BG
    };
    let label_style = Style::default().fg(Theme::BONE_DIM).bg(bg);
    let value_style = if is_sel {
        Style::default().fg(Theme::BONE).bg(bg)
    } else {
        Style::default().fg(Theme::FG).bg(bg)
    };
    let indicator = if is_sel { "▸" } else { " " };
    Line::from(vec![
        Span::styled(
            format!(" {indicator} "),
            Style::default().fg(Theme::ROSE).bg(bg),
        ),
        Span::styled(format!("{label}: "), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

/// Like config_row but shows the value in dim style (inherited defaults).
fn config_row_dim<'a>(idx: usize, selected: usize, label: &str, value: &str) -> Line<'a> {
    let is_sel = idx == selected;
    let bg = if is_sel {
        Theme::BG_HIGHLIGHT
    } else {
        Theme::BG
    };
    let label_style = Style::default().fg(Theme::BONE_DIM).bg(bg);
    let value_style = if is_sel {
        Style::default().fg(Theme::FG_DIM).bg(bg)
    } else {
        Style::default().fg(Theme::TEXT_GHOST).bg(bg)
    };
    let indicator = if is_sel { "▸" } else { " " };
    Line::from(vec![
        Span::styled(
            format!(" {indicator} "),
            Style::default().fg(Theme::ROSE).bg(bg),
        ),
        Span::styled(format!("{label}: "), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

fn toggle_row<'a>(idx: usize, selected: usize, label: &str, value: bool) -> Line<'a> {
    let is_sel = idx == selected;
    let bg = if is_sel {
        Theme::BG_HIGHLIGHT
    } else {
        Theme::BG
    };
    let label_style = Style::default().fg(Theme::BONE_DIM).bg(bg);
    let (icon, icon_color) = if value {
        ("[✓]", Theme::SAGE)
    } else {
        ("[ ]", Theme::TEXT_GHOST)
    };
    let indicator = if is_sel { "▸" } else { " " };
    Line::from(vec![
        Span::styled(
            format!(" {indicator} "),
            Style::default().fg(Theme::ROSE).bg(bg),
        ),
        Span::styled(format!("{label}: "), label_style),
        Span::styled(icon.to_string(), Style::default().fg(icon_color).bg(bg)),
    ])
}

fn display_model(cfg: &crate::state::config::ConfigState, slug: &str) -> String {
    cfg.available_models
        .iter()
        .find(|m| m.slug == slug)
        .map(|m| m.display_name.clone())
        .unwrap_or_else(|| slug.to_string())
}

fn shorten_model(slug: &str) -> String {
    slug.replace("gpt-", "")
        .replace("-codex", "c")
        .replace("-mini", "m")
        .replace("composer-", "cx-")
        .replace("claude-", "cl-")
}

fn render_agent_status(f: &mut Frame, area: Rect, state: &RunState) {
    let roles = AgentRole::ALL_AGENTS;

    let mut lines: Vec<Line> = Vec::new();

    for role in roles {
        let agent = state.agents.get(&role);
        let cfg_model = state.config.model_for(role).unwrap_or("?");
        let cfg_ctx = state.config.context_limit_for(role) / 1000;
        let cfg_effort = state.config.effort_for(role);

        let (status_icon, status_color) = match agent {
            Some(a) if a.active => ("●", Theme::STATUS_ACTIVE),
            Some(a) if a.input_tokens > 0 => ("○", Theme::STATUS_OK),
            Some(_) => ("○", Theme::FG_DIM),
            None => ("·", Theme::TEXT_GHOST),
        };

        let tokens = agent
            .map(|a| format!("{}k/{}k", a.input_tokens / 1000, cfg_ctx))
            .unwrap_or_else(|| format!("0/{}k", cfg_ctx));

        let ctx_pct = agent
            .map(|a| {
                let limit = state.config.context_limit_for(role);
                if limit > 0 {
                    (a.input_tokens as f64 / limit as f64 * 100.0) as u32
                } else {
                    0
                }
            })
            .unwrap_or(0);

        let pct_color = if ctx_pct > 90 {
            Theme::EMBER
        } else if ctx_pct > 80 {
            Theme::WARNING
        } else {
            Theme::FG_DIM
        };

        let turns = agent.map(|a| a.turn_count).unwrap_or(0);

        let short_model = shorten_model(cfg_model);

        let effort_span = if state.config.effort_configurable(role) {
            Span::styled(
                format!(" {}", cfg_effort.label()),
                Style::default().fg(Theme::DREAM),
            )
        } else {
            Span::styled(" N/A".to_string(), Style::default().fg(Theme::TEXT_GHOST))
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {status_icon} "),
                Style::default().fg(status_color),
            ),
            Span::styled(
                format!("{:12}", role.label()),
                Style::default().fg(Theme::role_accent(role)),
            ),
            Span::styled(
                format!(" {short_model:6}"),
                Style::default().fg(Theme::DREAM),
            ),
            Span::styled(format!(" {tokens:>9}"), Style::default().fg(Theme::FG_DIM)),
            Span::styled(format!(" {ctx_pct:>3}%"), Style::default().fg(pct_color)),
            Span::styled(format!(" t{turns}"), Style::default().fg(Theme::TEXT_DIM)),
            effort_span,
        ]));

        // Context gauge bar (compact, one line per agent)
        if agent.map(|a| a.input_tokens > 0).unwrap_or(false) {
            let limit = state.config.context_limit_for(role);
            let fill = if limit > 0 {
                agent.unwrap().input_tokens as f64 / limit as f64
            } else {
                0.0
            };
            let gauge_w = (area.width as usize).saturating_sub(6);
            let filled = (fill * gauge_w as f64) as usize;
            let empty = gauge_w.saturating_sub(filled);
            let bar_color = crate::tui::theme::gradient_context().sample(fill);

            lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
                Span::styled("░".repeat(empty), Style::default().fg(Theme::TEXT_GHOST)),
            ]));
        }
    }

    // Summary
    lines.push(Line::from(""));
    let active_count = state.agents.values().filter(|a| a.active).count();
    let total_in: u64 = state.agents.values().map(|a| a.input_tokens).sum();
    let total_out: u64 = state.agents.values().map(|a| a.output_tokens).sum();
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} active", active_count),
            Style::default().fg(Theme::ROSE),
        ),
        Span::styled(
            format!("  {}k in / {}k out", total_in / 1000, total_out / 1000),
            Style::default().fg(Theme::FG_DIM),
        ),
    ]));

    if state.config.parallel_enabled {
        lines.push(Line::from(Span::styled(
            format!(" parallel: {} max agents", state.config.max_agents),
            Style::default().fg(Theme::DREAM),
        )));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Agent Status")
            .style(Theme::block_style())
            .border_style(Style::default().fg(Theme::FG_DIM))
            .title_style(Theme::title_style()),
    );
    f.render_widget(paragraph, area);
}
