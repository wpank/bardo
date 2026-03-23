use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::RunState;
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;

const HEARTBEAT_FRAMES: [&str; 4] = ["\u{00b7}", "\u{00b0}", ".", "\u{25cf}"];

pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let bg = Style::default().bg(Theme::BG_SECONDARY);

    let merged_count = state
        .plans
        .iter()
        .filter(|p| matches!(p.status, crate::state::RunPlanStatus::MergedToMain))
        .count();
    let completed = state
        .plans
        .iter()
        .filter(|p| {
            matches!(
                p.status,
                crate::state::RunPlanStatus::Completed
                    | crate::state::RunPlanStatus::CompletedPrior
                    | crate::state::RunPlanStatus::MergedToMain
            )
        })
        .count();
    let total = state.plans.len();

    let mut spans = vec![Span::styled(" ", bg)];

    // Main repo git info: branch + commit hash + last commit time
    if !state.git_branch.is_empty() {
        spans.push(Span::styled(
            state.git_branch.clone(),
            Style::default().fg(Theme::BONE).bg(Theme::BG_SECONDARY),
        ));
        // Commit hash from the main (non-worktree) entry in the worktree list
        if let Some(main_wt) = state.git_worktree_list.first() {
            if !main_wt.commit.is_empty() {
                spans.push(Span::styled(
                    format!(" {}", &main_wt.commit[..main_wt.commit.len().min(7)]),
                    Style::default()
                        .fg(Theme::TEXT_GHOST)
                        .bg(Theme::BG_SECONDARY),
                ));
            }
        }
        if let Some(secs) = state.git_last_commit_secs {
            if secs > 0 {
                spans.push(Span::styled(
                    format!(" {}", crate::state::format_ago(secs)),
                    Style::default()
                        .fg(Theme::TEXT_GHOST)
                        .bg(Theme::BG_SECONDARY),
                ));
            }
        }
        spans.push(Span::styled(
            " │ ",
            Style::default().fg(Theme::ROSE_DIM).bg(Theme::BG_SECONDARY),
        ));
    }

    // Heartbeat dot
    let hb_idx = (atmosphere.frame() / 8) as usize % HEARTBEAT_FRAMES.len();
    spans.push(Span::styled(
        HEARTBEAT_FRAMES[hb_idx],
        Style::default().fg(Theme::ROSE_DIM).bg(Theme::BG_SECONDARY),
    ));

    // Paused indicator
    if state.pipeline_run_state == crate::state::PipelineRunState::Paused {
        spans.push(Span::styled(
            " PAUSED ",
            Style::default().fg(Theme::BG).bg(Theme::STATUS_WARN),
        ));
    }

    // Plan progress
    let progress_text = if state.complete {
        "COMPLETE".to_string()
    } else if let Some(ref err) = state.error {
        format!("ERR: {}", err.chars().take(30).collect::<String>())
    } else {
        format!(" {}/{}", completed, total)
    };
    spans.push(Span::styled(
        format!(" {progress_text} "),
        if state.error.is_some() {
            Theme::error_style()
        } else if state.complete {
            Theme::success_style()
        } else {
            Style::default().fg(Theme::ROSE)
        }
        .bg(Theme::BG_SECONDARY),
    ));

    // Main merge indicator — shown when any plans are merged to main
    if merged_count > 0 {
        let last_merge = state.main_merges.last();
        let hash_str = last_merge
            .map(|m| format!(" @{}", &m.merge_commit[..m.merge_commit.len().min(7)]))
            .unwrap_or_default();
        spans.push(Span::styled(
            format!(" ⬆ main:{merged_count}{hash_str} "),
            Style::default().fg(Theme::BONE).bg(Theme::BG_SECONDARY),
        ));
        spans.push(Span::styled(
            " │ ",
            Style::default().fg(Theme::ROSE_DIM).bg(Theme::BG_SECONDARY),
        ));
    }

    // Context-sensitive keybind hints based on active tab
    let keys = match state.active_tab {
        0 => {
            // Dashboard
            if state.error.is_some() {
                "↑↓:nav  r:restart-phase  c:reverify  ?:help  q:quit"
            } else if state.pending_approval.is_some() {
                "y:approve  n:reject  i:inject  q:quit"
            } else if state.filter_active {
                "type:filter  Enter:accept  Esc:cancel"
            } else {
                match state.focus {
                    crate::state::FocusZone::Plans => "↑↓:nav  Enter:expand  p:pause  Tab:panel  /:filter  a/o/d/e/g:sub-tab  ?:help",
                    crate::state::FocusZone::Tasks => "↑↓:nav  Enter:task detail  Tab:panel  ?:help  q:quit",
                    crate::state::FocusZone::AgentOutput => "↑↓:scroll  End:auto  i:inject  Tab:panel  `:agent  ?:help  q:quit",
                    crate::state::FocusZone::CommandOutput => "↑↓:scroll  Tab:panel  ?:help  q:quit",
                }
            }
        }
        1 => "↑↓:nav  Enter:drill  ←:back  m:merge→main  /:filter  Tab:panel  ?:help  q:quit",
        2 => "↑↓:nav  Tab:panel  `:cycle  End:auto  ?:help  q:quit",
        3 => "↑↓:nav  Enter:select  ?:help  q:quit",
        4 => "PgUp/PgDn:scroll  F1:dashboard  q:quit",
        5 => "j/k:nav  h/l:cycle  Enter:toggle  F1:dashboard  q:quit",
        _ => "F1:dashboard  ?:help  q:quit",
    };

    spans.push(Span::styled(
        format!(" {keys}"),
        Style::default().fg(Theme::FG_DIM).bg(Theme::BG_SECONDARY),
    ));

    let line = Line::from(spans);
    let p = Paragraph::new(line).style(bg);
    f.render_widget(p, area);
}
