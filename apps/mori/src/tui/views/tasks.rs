use crate::state::RunState;
use crate::tui::atmosphere::Atmosphere;
use crate::tui::widgets;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Task DAG visualization with progress
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    // Delegate to the existing task_progress widget
    widgets::task_progress::render(f, area, state, atmosphere, false);
}
