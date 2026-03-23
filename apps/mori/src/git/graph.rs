use super::ops;
use anyhow::Result;
use std::path::Path;

use crate::state::{GitTreeNode, RunPlanEntry, RunPlanStatus};

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub commit: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// List all branches with their HEAD commits
pub fn list_branches(repo_root: &Path) -> Result<Vec<BranchInfo>> {
    let current = ops::current_branch(repo_root).unwrap_or_default();
    let output = ops::run_git(
        repo_root,
        &[
            "branch",
            "-a",
            "--format=%(refname:short) %(objectname:short) %(upstream:short)",
        ],
    )?;

    let branches: Vec<BranchInfo> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            BranchInfo {
                name: parts.first().unwrap_or(&"").to_string(),
                commit: parts.get(1).unwrap_or(&"").to_string(),
                is_current: parts.first().map(|n| *n == current).unwrap_or(false),
                upstream: parts
                    .get(2)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            }
        })
        .collect();
    Ok(branches)
}

/// Get git log with graph decorations
pub fn log_graph(repo_root: &Path, count: usize) -> Result<Vec<String>> {
    let output = ops::run_git(
        repo_root,
        &[
            "log",
            "--oneline",
            "--graph",
            "--decorate",
            "--all",
            &format!("-{count}"),
        ],
    )?;
    Ok(output.lines().map(String::from).collect())
}

/// Get git log for a specific branch (filtered view)
pub fn log_graph_for_branch(repo_root: &Path, branch: &str, count: usize) -> Result<Vec<String>> {
    let output = ops::run_git(
        repo_root,
        &[
            "log",
            "--oneline",
            "--graph",
            "--decorate",
            &format!("-{count}"),
            branch,
        ],
    )?;
    Ok(output.lines().map(String::from).collect())
}

/// Build a hierarchical tree from the branch naming convention.
/// Groups branches under main -> batch -> plan hierarchy.
pub fn build_branch_tree(repo_root: &Path, plans: &[RunPlanEntry]) -> Vec<GitTreeNode> {
    let branches = list_branches(repo_root).unwrap_or_default();
    let worktrees = super::worktree::list_worktrees(repo_root).unwrap_or_default();
    let mut nodes = Vec::new();

    // Find main branch
    let main_branch = branches
        .iter()
        .find(|b| b.name == "main" || b.name == "master");
    if let Some(main) = main_branch {
        nodes.push(GitTreeNode {
            name: main.name.clone(),
            short_name: main.name.clone(),
            depth: 0,
            plan_base: None,
            task_id: None,
            status: None,
            is_current: main.is_current,
            commit: main.commit.clone(),
            worktree: None,
            is_last_sibling: false,
        });
    }

    // Group codex branches
    let batch_branches: Vec<_> = branches
        .iter()
        .filter(|b| b.name.starts_with("codex/batch/"))
        .collect();
    let plan_branches: Vec<_> = branches
        .iter()
        .filter(|b| b.name.starts_with("codex/plan/"))
        .collect();

    for (i, batch) in batch_branches.iter().enumerate() {
        let is_last_batch = i == batch_branches.len() - 1 && plan_branches.is_empty();
        let wt = worktrees
            .iter()
            .find(|w| w.branch == batch.name)
            .map(|w| w.path.clone());
        nodes.push(GitTreeNode {
            name: batch.name.clone(),
            short_name: batch.name.trim_start_matches("codex/batch/").to_string(),
            depth: 1,
            plan_base: None,
            task_id: None,
            status: None,
            is_current: batch.is_current,
            commit: batch.commit.clone(),
            worktree: wt,
            is_last_sibling: is_last_batch,
        });

        // Find plan branches under this batch
        for (j, plan_b) in plan_branches.iter().enumerate() {
            let plan_name = plan_b.name.trim_start_matches("codex/plan/");
            let plan_status = plans
                .iter()
                .find(|p| p.base == plan_name)
                .map(|p| p.status.clone());
            let wt = worktrees
                .iter()
                .find(|w| w.branch == plan_b.name)
                .map(|w| w.path.clone());

            nodes.push(GitTreeNode {
                name: plan_b.name.clone(),
                short_name: plan_name.to_string(),
                depth: 2,
                plan_base: Some(plan_name.to_string()),
                task_id: None,
                status: plan_status,
                is_current: plan_b.is_current,
                commit: plan_b.commit.clone(),
                worktree: wt,
                is_last_sibling: j == plan_branches.len() - 1,
            });
        }
    }

    // If no batch branches, add plan branches directly under main
    if batch_branches.is_empty() {
        for (j, plan_b) in plan_branches.iter().enumerate() {
            let plan_name = plan_b.name.trim_start_matches("codex/plan/");
            let plan_status = plans
                .iter()
                .find(|p| p.base == plan_name)
                .map(|p| p.status.clone());
            let wt = worktrees
                .iter()
                .find(|w| w.branch == plan_b.name)
                .map(|w| w.path.clone());

            nodes.push(GitTreeNode {
                name: plan_b.name.clone(),
                short_name: plan_name.to_string(),
                depth: 1,
                plan_base: Some(plan_name.to_string()),
                task_id: None,
                status: plan_status,
                is_current: plan_b.is_current,
                commit: plan_b.commit.clone(),
                worktree: wt,
                is_last_sibling: j == plan_branches.len() - 1,
            });
        }
    }

    nodes
}
