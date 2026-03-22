use ratatui::layout::{Constraint, Direction, Layout, Margin};
use ratatui::Frame;

use super::atmosphere::Atmosphere;
use super::modals;
use super::tabs::Tab;
use super::theme::Theme;
use super::views;
use super::widgets;
use crate::state::{InputMode, RunState};

/// Render the full TUI
pub fn render(f: &mut Frame, state: &RunState, atmosphere: &Atmosphere) {
    // Clear with background color
    let area = f.area();
    let bg = ratatui::widgets::Block::default().style(Theme::default_style());
    f.render_widget(bg, area);

    // Phase 1a: Conditional 1-cell padding when terminal is large enough
    let padded = if area.height >= 30 && area.width >= 100 {
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        })
    } else {
        area
    };

    // Root layout: header bar (1 line) | content | status bar (1 line)
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header bar (shown on all tabs)
            Constraint::Min(0),    // content
            Constraint::Length(1), // status bar
        ])
        .split(padded);

    let active_tab = Tab::from_index(state.active_tab).unwrap_or(Tab::Dashboard);

    // Header bar renders on ALL tabs
    widgets::header_bar::render(f, root[0], state, atmosphere);

    // Per-tab view routing
    match active_tab {
        Tab::Dashboard => views::dashboard::render(f, root[1], state, atmosphere),
        Tab::Plans => views::plans::render(f, root[1], state, atmosphere),
        Tab::Agents => views::agents::render(f, root[1], state, atmosphere),
        Tab::Git => views::git_view::render(f, root[1], state, atmosphere),
        Tab::Logs => views::logs::render(f, root[1], state),
        Tab::Config => views::config::render(f, root[1], state),
    }

    // Render status bar
    widgets::status_bar::render(f, root[2], state, atmosphere);

    // Apply atmospheric post-processing (ambient fill + postfx pipeline + particles)
    // Skip expensive VFX when agents are running to reduce CPU usage.
    if !state.any_agent_active() {
        atmosphere.apply(f.buffer_mut(), area, &active_tab);
    }

    // Modals (render on top of everything, after atmosphere)
    // Determine if any modal is open for the dim overlay
    let any_modal = state.show_plan_detail
        || state.show_help
        || state.show_wave_overview
        || state.show_agent_pool_modal
        || state.show_task_detail
        || state.show_task_picker
        || state.pending_approval.is_some()
        || state.pending_confirm.is_some()
        || state.input_mode == InputMode::Inject;

    if any_modal {
        // Dim the background for depth perception
        super::postfx::dim_overlay(area, f.buffer_mut(), 0.45);
    }

    if state.input_mode == InputMode::Inject {
        modals::inject::render(f, area, &state.message_input, state.steer_target.as_deref());
    }
    if state.input_mode == InputMode::Filter {
        render_filter_overlay(f, area, &state.filter_text);
    }
    if let Some(ref approval) = state.pending_approval {
        modals::approval::render(f, area, approval);
    }
    if state.show_plan_detail {
        let summary = if state.plan_summary_content.is_empty() {
            None
        } else {
            Some(state.plan_summary_content.as_str())
        };
        modals::plan_detail::render(
            f,
            area,
            &state.plan_detail_content,
            state.plan_detail_scroll,
            summary,
            state.plan_summary_scroll,
            state.plan_detail_tab,
        );
    }
    if state.show_help {
        modals::help::render(f, area);
    }
    if state.show_wave_overview {
        modals::wave_overview::render(f, area, state);
    }
    if state.show_agent_pool_modal {
        modals::agent_pool_modal::render(f, area, state);
    }
    if state.show_task_detail {
        modals::task_detail::render(f, area, state);
    }
    if state.show_task_picker {
        modals::task_picker::render(f, area, state);
    }
    if let Some(ref action) = state.pending_confirm {
        modals::confirm::render(f, area, action);
    }

    // Toast notifications (up to 3, newest first)
    for (i, notif) in state.notifications.iter().rev().take(3).enumerate() {
        modals::notification::render_toast(f, area, &notif.message, &notif.level, i);
    }
}

/// Render the filter input as a small overlay at the bottom of the plan tree area
fn render_filter_overlay(f: &mut Frame, area: ratatui::layout::Rect, filter_text: &str) {
    use ratatui::style::Style;
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    // Position at bottom-left, overlaying the plan tree area
    let overlay = ratatui::layout::Rect::new(
        area.x + 1,
        area.y + area.height.saturating_sub(3),
        area.width.min(40),
        1,
    );

    let line = Line::from(vec![
        Span::styled(
            " / ",
            Style::default().fg(Theme::DREAM).bg(Theme::BG_HIGHLIGHT),
        ),
        Span::styled(
            filter_text,
            Style::default().fg(Theme::BONE).bg(Theme::BG_HIGHLIGHT),
        ),
        Span::styled(
            "█",
            Style::default().fg(Theme::ROSE).bg(Theme::BG_HIGHLIGHT),
        ),
        Span::styled(
            " ".repeat(overlay.width.saturating_sub(filter_text.len() as u16 + 4) as usize),
            Style::default().bg(Theme::BG_HIGHLIGHT),
        ),
    ]);

    f.render_widget(Paragraph::new(line), overlay);
}
