use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{DetailSubTab, FocusZone, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;
use crate::tui::views;
use crate::tui::widgets;

/// Primary execution dashboard: master-detail layout.
/// Left panel: plan tree + phase bar + tasks (30%)
/// Right panel: detail sub-tabs with contextual content (70%)
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let has_waves = !state.execution_waves.is_empty();

    // Optional wave progress ribbon on top, then master-detail
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_waves {
            vec![Constraint::Length(1), Constraint::Min(0)]
        } else {
            vec![Constraint::Length(0), Constraint::Min(0)]
        })
        .split(area);

    if has_waves {
        widgets::wave_progress::render(f, root[0], state, atmosphere);
    }

    // Master-detail split: 30% left, 1-cell gap, remaining right
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(root[1]);

    render_left_panel(f, main[0], state, atmosphere);
    // main[1] is VOID spacer — stays empty
    render_right_panel(f, main[2], state, atmosphere);
}

fn render_left_panel(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let plan_focused = state.focus == FocusZone::Plans;
    let task_focused = state.focus == FocusZone::Tasks;

    // Dynamic heights
    let extra = if state.doc_revision_count > 0 { 1 } else { 0 };
    let verdict_extra = if !state.code_verdict.is_empty() { 1 } else { 0 };
    let phase_height = if state.current_iteration > 1 { 14 } else { 12 } + extra + verdict_extra;

    // Task height: if we have a checklist for the selected plan, show it; otherwise minimal
    let task_count = state
        .selected_checklist()
        .map(|cl| cl.tasks.len())
        .unwrap_or(0);
    let task_height = if task_count > 0 {
        (task_count as u16 + 3).min(10)
    } else {
        3
    };

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),               // plan tree (grows)
            Constraint::Length(1),            // phase timeline (compact)
            Constraint::Length(phase_height), // phase bar (detailed)
            Constraint::Length(task_height),  // tasks
        ])
        .split(area);

    widgets::plan_tree::render(f, left[0], state, atmosphere, plan_focused);
    widgets::phase_timeline::render(f, left[1], state, atmosphere);
    widgets::phase_bar::render(f, left[2], state, atmosphere, false);
    widgets::task_progress::render(f, left[3], state, atmosphere, task_focused);
}

fn render_right_panel(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let output_focused = state.focus == FocusZone::AgentOutput;

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // sub-tab bar
            Constraint::Min(0),    // content
        ])
        .split(area);

    render_sub_tab_bar(f, right[0], state);

    match state.detail_sub_tab {
        DetailSubTab::Agents => {
            render_agents_content(f, right[1], state, atmosphere, output_focused)
        }
        DetailSubTab::Output => {
            widgets::agent_output::render(f, right[1], state, atmosphere, output_focused);
        }
        DetailSubTab::Diff => {
            widgets::diff_panel::render(f, right[1], state);
        }
        DetailSubTab::Errors => {
            widgets::error_digest::render(f, right[1], state);
        }
        DetailSubTab::Git => {
            let plan_name = state
                .plans
                .get(state.selected_plan_idx)
                .map(|p| p.base.as_str());
            if let Some(plan) = plan_name {
                views::git_view::render_for_plan(f, right[1], state, atmosphere, plan);
            } else {
                views::git_view::render(f, right[1], state, atmosphere);
            }
        }
    }
}

fn render_sub_tab_bar(f: &mut Frame, area: Rect, state: &RunState) {
    let tabs: [(DetailSubTab, &str, ratatui::style::Color); 5] = [
        (DetailSubTab::Agents, "a:Agents", Theme::ROSE),
        (DetailSubTab::Output, "o:Output", Theme::BONE_DIM),
        (DetailSubTab::Diff, "d:Diff", Theme::SAGE),
        (DetailSubTab::Errors, "e:Errors", Theme::EMBER),
        (DetailSubTab::Git, "g:Git", Theme::DREAM),
    ];

    let mut spans = vec![Span::styled(" ", Style::default().bg(Theme::BG_SECONDARY))];

    for (tab, label, accent) in &tabs {
        let style = if *tab == state.detail_sub_tab {
            Style::default()
                .fg(Theme::BG)
                .bg(*accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::FG_DIM).bg(Theme::BG_SECONDARY)
        };
        spans.push(Span::styled(format!(" {label} "), style));
        spans.push(Span::styled(" ", Style::default().bg(Theme::BG_SECONDARY)));
    }

    // Error count badge
    let error_count = count_errors(state);
    if error_count > 0 {
        spans.push(Span::styled(
            format!(" {error_count}✗"),
            Style::default()
                .fg(Theme::EMBER)
                .bg(Theme::BG_SECONDARY)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Fill remaining
    let used: usize = spans.iter().map(|s| s.content.len()).sum();
    let remaining = (area.width as usize).saturating_sub(used);
    spans.push(Span::styled(
        " ".repeat(remaining),
        Style::default().bg(Theme::BG_SECONDARY),
    ));

    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}

/// Agents detail tab: agent grid + selected agent output + token burn | sys metrics
fn render_agents_content(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    atmosphere: &Atmosphere,
    output_focused: bool,
) {
    let active_agent_count = if !state.parallel_agents.is_empty() {
        state
            .parallel_agents
            .iter()
            .filter(|p| p.active || p.input_tokens > 0)
            .count()
    } else {
        state
            .agents
            .values()
            .filter(|a| a.active || a.input_tokens > 0)
            .count()
    };
    let agent_pool_height = (active_agent_count.max(1) as u16 + 2).min(8);

    let has_burn_data = !state.token_burn_history.is_empty();
    let active_burn_agents = state.token_burn_history.len() as u16;
    let sparkline_height = if has_burn_data {
        (4 + active_burn_agents).min(10)
    } else {
        6
    };
    // Always show bottom strip (token burn left, sys metrics right)
    let bottom_height = sparkline_height.max(6);

    let show_cmd_output = !state.gate_running.is_empty() || !state.command_output.is_empty();

    if show_cmd_output {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(agent_pool_height),
                Constraint::Percentage(55),
                Constraint::Min(0),
                Constraint::Length(bottom_height),
            ])
            .split(area);

        if !state.parallel_agents.is_empty() {
            widgets::parallel_pool::render(f, layout[0], state, atmosphere);
        } else {
            widgets::agent_pool::render(f, layout[0], state, atmosphere);
        }
        widgets::agent_output::render(f, layout[1], state, atmosphere, output_focused);
        let cmd_focused = state.focus == FocusZone::CommandOutput;
        widgets::command_output::render(f, layout[2], state, atmosphere, cmd_focused);
        render_bottom_strip(f, layout[3], state, atmosphere, has_burn_data);
    } else {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(agent_pool_height),
                Constraint::Min(0),
                Constraint::Length(bottom_height),
            ])
            .split(area);

        if !state.parallel_agents.is_empty() {
            widgets::parallel_pool::render(f, layout[0], state, atmosphere);
        } else {
            widgets::agent_pool::render(f, layout[0], state, atmosphere);
        }
        widgets::agent_output::render(f, layout[1], state, atmosphere, output_focused);
        render_bottom_strip(f, layout[2], state, atmosphere, has_burn_data);
    }
}

/// Token Burn (left) | System Metrics (right) side by side.
fn render_bottom_strip(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    atmosphere: &Atmosphere,
    has_burn_data: bool,
) {
    if has_burn_data {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);
        widgets::token_sparkline::render(f, cols[0], state, atmosphere);
        widgets::sys_metrics::render(f, cols[1], state);
    } else {
        // No token data yet — sys metrics takes the full width
        widgets::sys_metrics::render(f, area, state);
    }
}

fn count_errors(state: &RunState) -> usize {
    let mut count = 0;
    if state.error.is_some() {
        count += 1;
    }
    if !state.last_gate_output.is_empty() {
        count += state
            .last_gate_output
            .lines()
            .filter(|l| l.trim().starts_with("error"))
            .count();
    }
    count
}
