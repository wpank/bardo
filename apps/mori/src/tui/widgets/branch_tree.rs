use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::{GitTreeNode, RunPlanStatus, RunState};
use crate::tui::atmosphere::Atmosphere;
use crate::tui::theme::Theme;
use crate::tui::widgets::scrollbar;

/// Render the git branch tree as a hierarchical view with connectors.
pub fn render(f: &mut Frame, area: Rect, state: &RunState, atmosphere: &Atmosphere) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Branches")
        .border_style(Theme::unfocused_border_style())
        .title_style(Theme::unfocused_title_style());

    if state.git_branch_tree.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "No branch data",
            Style::default().fg(Theme::TEXT_DIM),
        )))
        .block(block);
        f.render_widget(empty, area);
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let total = state.git_branch_tree.len();

    // Compute scroll offset to keep cursor visible
    let cursor = state.git_branch_cursor.min(total.saturating_sub(1));
    let offset = if inner_height == 0 {
        0
    } else if cursor < inner_height {
        0
    } else {
        cursor.saturating_sub(inner_height - 1)
    };

    let visible_nodes = state
        .git_branch_tree
        .iter()
        .enumerate()
        .skip(offset)
        .take(inner_height);

    let lines: Vec<Line> = visible_nodes
        .map(|(idx, node)| build_tree_line(node, idx, cursor, atmosphere))
        .collect();

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);

    // Scrollbar on the inner area (inside the block border)
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };
    scrollbar::render_scrollbar(
        f.buffer_mut(),
        inner,
        total,
        inner_height,
        offset,
        Theme::ROSE_DIM,
    );
}

/// Build a single line for a tree node with prefix connectors and status coloring.
fn build_tree_line<'a>(
    node: &'a GitTreeNode,
    idx: usize,
    cursor: usize,
    atmosphere: &Atmosphere,
) -> Line<'a> {
    let mut spans = Vec::new();

    // Tree connector prefix based on depth
    let prefix = build_prefix(node);
    spans.push(Span::styled(prefix, Style::default().fg(Theme::TEXT_GHOST)));

    // Branch name with status-based coloring
    let name_style = branch_style(node, atmosphere);
    spans.push(Span::styled(node.short_name.clone(), name_style));

    // Dimmed commit hash
    spans.push(Span::styled(
        format!(" {}", &node.commit),
        Style::default().fg(Theme::TEXT_GHOST),
    ));

    // Worktree path if present
    if let Some(ref wt) = node.worktree {
        spans.push(Span::styled(
            format!(" {wt}"),
            Style::default().fg(Theme::BONE_DIM),
        ));
    }

    if idx == cursor {
        // Apply selection highlight to all spans
        let highlighted: Vec<Span> = spans
            .into_iter()
            .map(|s| {
                let style = s.style.bg(Theme::BG_HIGHLIGHT);
                Span::styled(s.content.into_owned(), style)
            })
            .collect();
        Line::from(highlighted)
    } else {
        Line::from(spans)
    }
}

/// Build the tree-drawing prefix (connectors) for a node.
fn build_prefix(node: &GitTreeNode) -> String {
    if node.depth == 0 {
        return String::new();
    }

    let mut prefix = String::new();

    // Indentation for ancestors above depth-1
    for _ in 0..node.depth.saturating_sub(1) {
        prefix.push_str("│ ");
    }

    // Connector for this node's level
    if node.is_last_sibling {
        prefix.push_str("└─");
    } else {
        prefix.push_str("├─");
    }

    prefix
}

/// Determine the style for a branch name based on its status and current-ness.
fn branch_style(node: &GitTreeNode, atmosphere: &Atmosphere) -> Style {
    if node.is_current {
        return Style::default()
            .fg(Theme::ROSE_BRIGHT)
            .add_modifier(Modifier::BOLD);
    }

    match &node.status {
        Some(RunPlanStatus::MergedToMain) => Style::default()
            .fg(Theme::BONE)
            .add_modifier(Modifier::BOLD),
        Some(RunPlanStatus::Completed) | Some(RunPlanStatus::CompletedPrior) => {
            Style::default().fg(Theme::SAGE)
        }
        Some(RunPlanStatus::Active) => {
            // Breathing pulse: modulate brightness with atmosphere
            let brightness = atmosphere.breathing_brightness();
            let base = Theme::ROSE;
            let pulsed = crate::tui::theme::brighten(base, brightness);
            Style::default().fg(pulsed)
        }
        Some(RunPlanStatus::Failed) => Style::default().fg(Theme::EMBER),
        Some(RunPlanStatus::Pending) | Some(RunPlanStatus::Skipped) => {
            Style::default().fg(Theme::TEXT_GHOST)
        }
        None => Style::default().fg(Theme::BONE_DIM),
    }
}
