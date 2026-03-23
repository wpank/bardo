use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::state::{RunPlanStatus, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;
use crate::tui::widgets::branch_tree;

/// Render with no plan filter (used by standalone Git tab F4).
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    render_filtered(f, area, state, atmosphere, None);
}

/// Render filtered to a specific plan (used by dashboard git sub-tab).
pub fn render_for_plan(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    atmosphere: &Atmosphere,
    plan_base: &str,
) {
    render_filtered(f, area, state, atmosphere, Some(plan_base));
}

fn render_filtered(
    f: &mut Frame,
    area: Rect,
    state: &RunState,
    atmosphere: &Atmosphere,
    plan_filter: Option<&str>,
) {
    // Left 35% | 1-cell spacer | Right 65%
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(1),
            Constraint::Percentage(65),
        ])
        .split(area);

    // Left column: branch tree (60%) + worktrees (40%)
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(columns[0]);

    // Right column: commit graph (60%) + branch info (40%)
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(columns[2]);

    branch_tree::render(f, left[0], state, atmosphere);
    render_worktrees(f, left[1], &state.git_worktree_list, plan_filter);
    render_commit_graph(f, right[0], &state.git_commit_graph, state, plan_filter);
    render_branch_info(f, right[1], state, plan_filter);
}

// ── Worktree list ──────────────────────────────────────────────────────

fn render_worktrees(
    f: &mut Frame,
    area: Rect,
    worktrees: &[crate::git::worktree::WorktreeEntry],
    plan_filter: Option<&str>,
) {
    let title = match plan_filter {
        Some(plan) => format!("Worktrees ({plan})"),
        None => "Worktrees".to_string(),
    };

    let items: Vec<ListItem> = worktrees
        .iter()
        .filter(|w| {
            match plan_filter {
                None => true,
                Some(plan) => {
                    // Show batch branch, main, and the selected plan's worktree
                    w.branch.starts_with("codex/batch/")
                        || w.branch == "main"
                        || w.branch == format!("codex/plan/{plan}")
                        || w.branch.starts_with(&format!("codex/task/{plan}/"))
                }
            }
        })
        .map(|w| {
            let is_selected_plan = plan_filter
                .map(|p| w.branch == format!("codex/plan/{p}"))
                .unwrap_or(false);
            let marker = if is_selected_plan {
                Span::styled(
                    "► ",
                    Style::default()
                        .fg(Theme::ROSE)
                        .add_modifier(Modifier::BOLD),
                )
            } else if w.branch.starts_with("codex/plan/") {
                Span::styled("► ", Style::default().fg(Theme::ROSE))
            } else {
                Span::styled("· ", Style::default().fg(Theme::TEXT_GHOST))
            };

            let branch_style = if is_selected_plan {
                Style::default()
                    .fg(Theme::ROSE)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::DREAM)
            };

            ListItem::new(Line::from(vec![
                marker,
                Span::styled(&w.branch, branch_style),
                Span::styled(format!(" {}", w.commit), Style::default().fg(Theme::FG_DIM)),
                Span::styled(format!(" {}", w.path), Style::default().fg(Theme::BONE_DIM)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Theme::unfocused_border_style())
            .title_style(Theme::unfocused_title_style()),
    );

    f.render_widget(list, area);
}

// ── Commit graph ───────────────────────────────────────────────────────

fn render_commit_graph(
    f: &mut Frame,
    area: Rect,
    graph_lines: &[String],
    state: &RunState,
    plan_filter: Option<&str>,
) {
    let title = match plan_filter {
        Some(plan) => format!("Commits ({plan})"),
        None => "Commit Graph".to_string(),
    };

    // Use cached graph lines, optionally filtered to plan-related commits
    let log_lines: Vec<&String> = if let Some(plan) = plan_filter {
        let plan_branch = format!("codex/plan/{plan}");
        graph_lines
            .iter()
            .filter(|l| l.contains(&plan_branch) || l.contains("codex/batch/") || l.contains("*"))
            .collect()
    } else {
        graph_lines.iter().collect()
    };

    let lines: Vec<Line> = log_lines
        .iter()
        .map(|raw| colorize_graph_line(raw, state))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Theme::unfocused_border_style())
                .title_style(Theme::unfocused_title_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Parse a single git log --graph line and apply status-aware coloring.
fn colorize_graph_line<'a>(raw: &'a str, state: &RunState) -> Line<'a> {
    // Split into graph-prefix characters and the rest of the line.
    // Graph characters are at the start: |, /, \, *, space, _
    let mut graph_end = 0;
    for (i, ch) in raw.char_indices() {
        if matches!(ch, '|' | '/' | '\\' | '*' | ' ' | '_') {
            graph_end = i + ch.len_utf8();
        } else {
            break;
        }
    }

    let (graph_part, text_part) = raw.split_at(graph_end);
    let mut spans: Vec<Span<'a>> = Vec::new();

    // Render graph prefix, replacing * with ● in ROSE
    if !graph_part.is_empty() {
        let graph_colored = graph_part.replace('*', "●");
        if graph_part.contains('*') {
            // Mixed line: split around the bullet
            for segment in graph_colored.split_inclusive('●') {
                if segment.ends_with('●') {
                    let prefix = &segment[..segment.len() - '●'.len_utf8()];
                    if !prefix.is_empty() {
                        spans.push(Span::styled(
                            prefix.to_string(),
                            Style::default().fg(Theme::TEXT_GHOST),
                        ));
                    }
                    spans.push(Span::styled(
                        "●".to_string(),
                        Style::default().fg(Theme::ROSE),
                    ));
                } else {
                    spans.push(Span::styled(
                        segment.to_string(),
                        Style::default().fg(Theme::TEXT_GHOST),
                    ));
                }
            }
        } else {
            spans.push(Span::styled(
                graph_colored,
                Style::default().fg(Theme::TEXT_GHOST),
            ));
        }
    }

    // Color the text portion based on content
    let text_style = if raw.contains("HEAD") {
        Style::default().fg(Theme::ROSE)
    } else if raw.contains("codex/plan/") {
        plan_line_style(raw, state)
    } else if raw.contains("codex/batch/") {
        Style::default().fg(Theme::DREAM)
    } else if raw.contains("tag:") {
        Style::default().fg(Theme::BONE_DIM)
    } else {
        Style::default().fg(Theme::FG)
    };

    spans.push(Span::styled(text_part, text_style));

    Line::from(spans)
}

/// Determine color for a line referencing a codex/plan/ branch by cross-referencing
/// plan status from RunState.
fn plan_line_style(line: &str, state: &RunState) -> Style {
    // Try to extract the plan name from the line: "codex/plan/XX-name"
    if let Some(start) = line.find("codex/plan/") {
        let after = &line[start + "codex/plan/".len()..];
        // Plan name ends at comma, paren, space, or end of string
        let plan_name: String = after
            .chars()
            .take_while(|c| !matches!(c, ',' | ')' | ' '))
            .collect();

        if let Some(entry) = state.plans.iter().find(|p| p.base == plan_name) {
            return match entry.status {
                RunPlanStatus::MergedToMain => Style::default()
                    .fg(Theme::BONE)
                    .add_modifier(ratatui::style::Modifier::BOLD),
                RunPlanStatus::Completed | RunPlanStatus::CompletedPrior => {
                    Style::default().fg(Theme::SAGE)
                }
                RunPlanStatus::Active => Style::default().fg(Theme::ROSE),
                RunPlanStatus::Failed => Style::default().fg(Theme::EMBER),
                _ => Style::default().fg(Theme::DREAM),
            };
        }
    }

    Style::default().fg(Theme::DREAM)
}

// ── Branch info panel ──────────────────────────────────────────────────

fn render_branch_info(f: &mut Frame, area: Rect, state: &RunState, plan_filter: Option<&str>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Branch Info")
        .border_style(Theme::unfocused_border_style())
        .title_style(Theme::unfocused_title_style());

    // When filtered to a plan, show that plan's branch info directly
    // instead of using the git_branch_cursor
    let target_node: Option<&crate::state::GitTreeNode> = if let Some(plan) = plan_filter {
        let plan_branch = format!("codex/plan/{plan}");
        state.git_branch_tree.iter().find(|n| n.name == plan_branch)
    } else if !state.git_branch_tree.is_empty() {
        let cursor = state
            .git_branch_cursor
            .min(state.git_branch_tree.len().saturating_sub(1));
        Some(&state.git_branch_tree[cursor])
    } else {
        None
    };

    let Some(node) = target_node else {
        let msg = match plan_filter {
            Some(plan) => format!("No branch for plan {plan}"),
            None => "No branch selected".to_string(),
        };
        let empty = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Theme::TEXT_DIM),
        )))
        .block(block);
        f.render_widget(empty, area);
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    // Branch name (full)
    lines.push(Line::from(Span::styled(
        &node.name,
        Style::default()
            .fg(Theme::BONE)
            .add_modifier(Modifier::BOLD),
    )));

    // Commit
    lines.push(Line::from(vec![
        Span::styled("commit ", Style::default().fg(Theme::TEXT_GHOST)),
        Span::styled(&node.commit, Style::default().fg(Theme::FG)),
    ]));

    // Status if it's a plan branch
    if let Some(ref status) = node.status {
        let (label, color) = match status {
            RunPlanStatus::MergedToMain => ("⬆ merged-to-main", Theme::BONE),
            RunPlanStatus::Completed | RunPlanStatus::CompletedPrior => ("completed", Theme::SAGE),
            RunPlanStatus::Active => ("active", Theme::ROSE),
            RunPlanStatus::Failed => ("failed", Theme::EMBER),
            RunPlanStatus::Pending => ("pending", Theme::TEXT_GHOST),
            RunPlanStatus::Skipped => ("skipped", Theme::TEXT_DIM),
        };
        lines.push(Line::from(vec![
            Span::styled("status ", Style::default().fg(Theme::TEXT_GHOST)),
            Span::styled(label, Style::default().fg(color)),
        ]));
    }

    // Worktree if present
    if let Some(ref wt) = node.worktree {
        lines.push(Line::from(vec![
            Span::styled("worktree ", Style::default().fg(Theme::TEXT_GHOST)),
            Span::styled(wt.as_str(), Style::default().fg(Theme::BONE_DIM)),
        ]));
    }

    // Diff stat hint: if the selected branch matches the current git_branch, show branch_diff
    if node.name == state.git_branch && !state.branch_diff.is_empty() {
        lines.push(Line::from(Span::raw("")));
        for diff_line in state.branch_diff.lines().take(
            area.height.saturating_sub(2) as usize
                - lines.len().min(area.height.saturating_sub(2) as usize),
        ) {
            let style = if diff_line.starts_with('+') {
                Style::default().fg(Theme::SAGE)
            } else if diff_line.starts_with('-') {
                Style::default().fg(Theme::EMBER)
            } else {
                Style::default().fg(Theme::FG_DIM)
            };
            lines.push(Line::from(Span::styled(diff_line, style)));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
