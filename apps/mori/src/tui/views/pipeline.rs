use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::state::{FocusZone, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::widgets;

pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let has_waves = !state.execution_waves.is_empty();

    // Root: optional wave bar on top, then main content
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_waves {
            vec![Constraint::Length(4), Constraint::Min(0)]
        } else {
            vec![Constraint::Length(0), Constraint::Min(0)]
        })
        .split(area);

    // Wave bar (only when parallel execution)
    if has_waves {
        widgets::wave_bar::render(f, root[0], state);
    }

    // Main: left sidebar (25%) | right content (75%)
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(root[1]);

    // Left column: plan list | phase bar | task progress
    let extra = if state.doc_revision_count > 0 { 1 } else { 0 };
    let verdict_extra = if !state.code_verdict.is_empty() { 1 } else { 0 };
    let phase_height = if state.current_iteration > 1 { 14 } else { 12 } + extra + verdict_extra;
    let plan_height = (state.plans.len() as u16 + 3).min(12);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(plan_height),  // plan list
            Constraint::Length(phase_height), // phase bar
            Constraint::Min(5),               // task progress (grows)
        ])
        .split(main[0]);

    let plan_focused = state.focus == FocusZone::Plans;
    let task_focused = state.focus == FocusZone::Tasks;
    let output_focused = state.focus == FocusZone::AgentOutput;

    widgets::plan_list::render(f, left[0], state, atmosphere, plan_focused);
    widgets::phase_bar::render(f, left[1], state, atmosphere, false);
    widgets::task_progress::render(f, left[2], state, atmosphere, task_focused);

    // Right column: agent pool summary | agent output (selected agent)
    let active_agent_count = state
        .agents
        .values()
        .filter(|a| a.active || a.input_tokens > 0)
        .count();
    let agent_pool_height = (active_agent_count.max(1) as u16 + 2).min(8); // lines + borders, cap at 8

    let show_cmd_output = !state.gate_running.is_empty() || !state.command_output.is_empty();

    if show_cmd_output {
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(agent_pool_height),
                Constraint::Percentage(55),
                Constraint::Percentage(45),
            ])
            .split(main[1]);

        widgets::agent_pool::render(f, right[0], state, atmosphere);
        widgets::agent_output::render(f, right[1], state, atmosphere, output_focused);
        widgets::command_output::render(f, right[2], state, atmosphere, false);
    } else {
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(agent_pool_height), Constraint::Min(0)])
            .split(main[1]);

        widgets::agent_pool::render(f, right[0], state, atmosphere);
        widgets::agent_output::render(f, right[1], state, atmosphere, output_focused);
    }
}
