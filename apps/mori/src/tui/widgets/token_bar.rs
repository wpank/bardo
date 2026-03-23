use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::agent::AgentRole;
use crate::state::RunState;
use crate::tui::theme::{self, Theme};

pub fn render(f: &mut Frame, area: Rect, state: &RunState) {
    let mut total_in: u64 = 0;
    let mut total_out: u64 = 0;

    let mut roles: Vec<AgentRole> = state.agents.keys().copied().collect();
    roles.sort_by_key(|r| r.index());

    let mut lines: Vec<Line> = Vec::new();

    for &role in &roles {
        let agent = match state.agents.get(&role) {
            Some(a) => a,
            None => continue,
        };
        total_in += agent.input_tokens;
        total_out += agent.output_tokens;

        let total = agent.input_tokens + agent.output_tokens;
        if total == 0 && !agent.active {
            continue; // Skip idle agents with no tokens
        }

        let accent = Theme::role_accent(role);
        let fill_pct = if state.context_limit > 0 {
            (agent.input_tokens as f64 / state.context_limit as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Model slug
        let model_slug = state.config.model_for(role).unwrap_or("?");
        let short_model = shorten_model(model_slug);

        // Token flash
        let flash = state.token_flash.get(&role).copied().unwrap_or(0);
        let name_style = if agent.active {
            Style::default().fg(accent)
        } else {
            Style::default().fg(Theme::FG_DIM)
        };

        // Line 1: role name + model + tokens + percentage
        let in_k = agent.input_tokens / 1000;
        let pct_display = (fill_pct * 100.0) as u32;
        let pct_str = format!("{pct_display}%");

        lines.push(Line::from(vec![
            Span::styled(format!(" {:5} ", role.short()), name_style),
            Span::styled(
                format!("{:6} ", short_model),
                Style::default().fg(Theme::DREAM),
            ),
            Span::styled(
                format!("{:>4}k ", in_k),
                if flash > 0 {
                    Style::default().fg(Theme::BONE)
                } else {
                    Style::default().fg(Theme::FG_DIM)
                },
            ),
        ]));

        // Line 2: gauge bar
        let gauge_width = (area.width as usize).saturating_sub(6); // borders + padding
        let filled = (fill_pct * gauge_width as f64) as usize;
        let empty = gauge_width.saturating_sub(filled);

        // Color from context gradient
        let bar_color = theme::gradient_context().sample(fill_pct);

        let gauge_spans = vec![
            Span::styled("  ", Style::default()),
            Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
            Span::styled(
                "\u{2591}".repeat(empty),
                Style::default().fg(Theme::TEXT_GHOST),
            ),
            Span::styled(
                format!(" {pct_str}"),
                Style::default().fg(if fill_pct > 0.8 {
                    Theme::EMBER
                } else {
                    Theme::FG_DIM
                }),
            ),
        ];
        lines.push(Line::from(gauge_spans));
    }

    let title = format!("Tokens ({}k in {}k out)", total_in / 1000, total_out / 1000,);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Theme::block_style())
        .title_style(Theme::title_style());

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn shorten_model(slug: &str) -> String {
    slug.replace("gpt-", "")
        .replace("-codex", "c")
        .replace("-mini", "m")
}
