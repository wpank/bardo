use super::ops;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{info, instrument, warn};

/// Write .cursor/cli.json into a worktree so Cursor ACP agents have file/run permissions.
///
/// NOTE: do NOT include a "version" key — newer cursor agent versions reject it with a schema
/// validation error that kills the process before it can do any work.
fn write_cursor_cli_config(worktree_path: &Path) -> Result<()> {
    let dir = worktree_path.join(".cursor");
    std::fs::create_dir_all(&dir)?;
    let config = serde_json::json!({
        "permissions": {
            "allow": ["read_file", "write_file", "run_command"],
            "deny": []
        }
    });
    std::fs::write(dir.join("cli.json"), serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        // Skip symlinks to avoid infinite loops or broken links
        if ty.is_symlink() {
            continue;
        }
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Root files copied from the main checkout into a worktree after `git worktree add`.
///
/// Worktrees already contain **tracked** files from the branch; we still copy these from
/// `repo_root` so agents see the operator's current workspace root (same idea as `plans/` /
/// `prd2/`, which are copied because they are often gitignored and absent from the index).
const WORKTREE_REPO_ROOT_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "bardo-ctl.sh",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "clippy.toml",
    "deny.toml",
    "nextest.toml",
];

const WORKTREE_REPO_ROOT_DIRS: &[&str] = &[".cargo", "plans", "prd2", "agents", "tests"];

fn copy_repo_root_files_into_worktree(repo_root: &Path, worktree_root: &Path) {
    for name in WORKTREE_REPO_ROOT_FILES {
        let src = repo_root.join(name);
        let dst = worktree_root.join(name);
        if src.is_file() {
            if let Err(e) = std::fs::copy(&src, &dst) {
                warn!("Failed to copy {name} to {}: {e}", worktree_root.display());
            } else {
                info!("Copied {name} into worktree {}", worktree_root.display());
            }
        }
    }
    for name in WORKTREE_REPO_ROOT_DIRS {
        let src = repo_root.join(name);
        let dst = worktree_root.join(name);
        if !src.is_dir() {
            continue;
        }
        // Always re-sync plans/ and prd2/ from the main checkout so worktrees see fresh
        // context (briefs, extracts, tasks) and ignored gitignored deltas. Other dirs are
        // one-time overlays (large or rarely changed).
        let always_sync = matches!(*name, "plans" | "prd2");
        if always_sync || !dst.exists() {
            if let Err(e) = copy_dir_all(&src, &dst) {
                warn!("Failed to sync {name}/ to {}: {e}", worktree_root.display());
            } else if always_sync {
                info!("Synced {name}/ into worktree {}", worktree_root.display());
            } else {
                info!("Copied {name}/ into worktree {}", worktree_root.display());
            }
        }
    }
}

/// Create context/in/ and context/out/ directories in a worktree.
/// Called after worktree creation so agents have their I/O dirs ready.
pub fn create_context_dirs(worktree: &Path) -> Result<()> {
    std::fs::create_dir_all(worktree.join("context/in"))?;
    std::fs::create_dir_all(worktree.join("context/out"))?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct WorktreeEntry {
    pub path: String,
    pub branch: String,
    pub commit: String,
}

pub fn list_worktrees(repo_root: &Path) -> Result<Vec<WorktreeEntry>> {
    let output = ops::run_git(repo_root, &["worktree", "list", "--porcelain"])?;
    let mut entries = Vec::new();
    let mut current_path = String::new();
    let mut current_branch = String::new();
    let mut current_commit = String::new();

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if !current_path.is_empty() {
                entries.push(WorktreeEntry {
                    path: current_path.clone(),
                    branch: current_branch.clone(),
                    commit: current_commit.clone(),
                });
            }
            current_path = path.to_string();
            current_branch.clear();
            current_commit.clear();
        } else if let Some(hash) = line.strip_prefix("HEAD ") {
            current_commit = hash.chars().take(8).collect();
        } else if let Some(branch) = line.strip_prefix("branch ") {
            current_branch = branch
                .strip_prefix("refs/heads/")
                .unwrap_or(branch)
                .to_string();
        }
    }
    if !current_path.is_empty() {
        entries.push(WorktreeEntry {
            path: current_path,
            branch: current_branch,
            commit: current_commit,
        });
    }
    Ok(entries)
}

/// Worktree for an entire plan (parallel wave execution).
#[derive(Debug, Clone)]
pub struct PlanWorktree {
    pub path: PathBuf,
    pub branch: String,
    pub plan_base: String,
}

/// Represents an active worktree for a task.
#[derive(Debug, Clone)]
pub struct TaskWorktree {
    pub path: PathBuf,
    pub branch: String,
    pub plan_base: String,
    pub task_id: String,
}

/// Manages git worktrees for parallel task execution.
///
/// Each plan task gets its own worktree so multiple Codex agents can
/// operate on the same repo concurrently without stepping on each other.
#[derive(Clone)]
pub struct WorktreeManager {
    repo_root: PathBuf,
    worktree_base: PathBuf,
}

impl WorktreeManager {
    pub fn new(repo_root: PathBuf) -> Self {
        let worktree_base = repo_root.join(".worktrees");
        Self {
            repo_root,
            worktree_base,
        }
    }

    /// Get the base directory for worktrees.
    pub fn worktree_base(&self) -> &Path {
        &self.worktree_base
    }

    /// Ensure the `.worktrees/` directory exists.
    pub fn init(&self) -> Result<()> {
        if !self.worktree_base.exists() {
            std::fs::create_dir_all(&self.worktree_base)
                .context("failed to create .worktrees directory")?;
            info!("Created worktree base: {}", self.worktree_base.display());
        }
        Ok(())
    }

    /// Create a worktree for a plan task.
    ///
    /// Branch naming: `codex/plan/{plan_base}/{task_id}`
    /// Directory: `.worktrees/plan-{plan_base}-{task_id}/`
    /// The worktree branches from `codex/plan/{plan_base}` (the plan branch).
    pub fn create_task_worktree(&self, plan_base: &str, task_id: &str) -> Result<TaskWorktree> {
        let branch = format!("codex/plan/{plan_base}/{task_id}");
        let dir_name = format!("plan-{plan_base}-{task_id}");
        let path = self.worktree_base.join(&dir_name);
        let plan_branch = format!("codex/plan/{plan_base}");

        // Clean up leftover from a prior run
        if path.exists() {
            self.remove_worktree(&path)?;
        }

        let path_str = path.to_str().context("worktree path is not valid UTF-8")?;
        ops::run_git(
            &self.repo_root,
            &["worktree", "add", "-b", &branch, path_str, &plan_branch],
        )
        .with_context(|| format!("failed to create task worktree at {path_str}"))?;

        if let Err(e) = write_cursor_cli_config(&path) {
            warn!("Failed to write .cursor/cli.json for task worktree: {e}");
        }

        copy_repo_root_files_into_worktree(&self.repo_root, &path);
        if let Err(e) = crate::orchestrator::context::regenerate_workspace_map(&path) {
            warn!("Failed to regenerate workspace-map.md in task worktree: {e}");
        }

        info!("Created task worktree: {branch} at {path_str}");

        Ok(TaskWorktree {
            path,
            branch,
            plan_base: plan_base.to_string(),
            task_id: task_id.to_string(),
        })
    }

    /// Create a worktree for a utility operation (pre-planner, refactorer, etc.).
    ///
    /// Branch naming: `codex/util/{name}`
    /// Directory: `.worktrees/{name}/`
    pub fn create_utility_worktree(&self, name: &str, base_branch: &str) -> Result<PathBuf> {
        let path = self.worktree_base.join(name);
        if path.exists() {
            self.remove_worktree(&path)?;
        }

        let branch = format!("codex/util/{name}");
        let path_str = path.to_str().context("worktree path is not valid UTF-8")?;
        ops::run_git(
            &self.repo_root,
            &["worktree", "add", "-b", &branch, path_str, base_branch],
        )
        .with_context(|| format!("failed to create utility worktree at {path_str}"))?;

        if let Err(e) = write_cursor_cli_config(&path) {
            warn!("Failed to write .cursor/cli.json for utility worktree: {e}");
        }

        copy_repo_root_files_into_worktree(&self.repo_root, &path);
        if let Err(e) = crate::orchestrator::context::regenerate_workspace_map(&path) {
            warn!("Failed to regenerate workspace-map.md in utility worktree: {e}");
        }

        info!("Created utility worktree: {branch} at {path_str}");
        Ok(path)
    }

    /// Create or reuse a worktree for a plan in parallel wave execution.
    ///
    /// If a worktree already exists at the expected path AND is on the right
    /// branch, reuse it (preserving any uncommitted work from a prior crash).
    /// Otherwise, clean up and create fresh.
    pub fn create_plan_worktree(&self, plan_base: &str, base_branch: &str) -> Result<PlanWorktree> {
        let branch = format!("codex/plan/{plan_base}");
        let dir_name = format!("plan-{plan_base}");
        let path = self.worktree_base.join(&dir_name);

        if path.exists() {
            // Check if the existing worktree is on the right branch
            let existing_branch = ops::run_git(&path, &["rev-parse", "--abbrev-ref", "HEAD"]).ok();

            if existing_branch.as_deref() == Some(&branch) {
                // Worktree exists and is on the correct branch — reuse it.
                // Check for uncommitted changes and log them.
                let status = ops::run_git(&path, &["status", "--porcelain"]).unwrap_or_default();
                if !status.trim().is_empty() {
                    info!("Reusing plan worktree for {plan_base} with uncommitted changes");
                } else {
                    info!("Reusing clean plan worktree for {plan_base}");
                }
                // Refresh .cursor/cli.json (removes stale "version" key if present)
                if let Err(e) = write_cursor_cli_config(&path) {
                    warn!("Failed to refresh .cursor/cli.json for reused worktree: {e}");
                }
                // Ensure root files are present even on reuse
                copy_repo_root_files_into_worktree(&self.repo_root, &path);
                if let Err(e) = crate::orchestrator::context::regenerate_workspace_map(&path) {
                    warn!("Failed to regenerate workspace-map.md in reused plan worktree: {e}");
                }
                if let Err(e) = create_context_dirs(&path) {
                    warn!("Failed to create context dirs for plan worktree: {e}");
                }
                return Ok(PlanWorktree {
                    path,
                    branch,
                    plan_base: plan_base.to_string(),
                });
            }

            // Wrong branch or corrupt — clean up and recreate
            warn!(
                "Plan worktree for {plan_base} on wrong branch ({:?}), recreating",
                existing_branch
            );
            if let Err(e) = self.remove_worktree(&path) {
                warn!("Failed to remove stale worktree: {e} — force removing directory");
                let _ = std::fs::remove_dir_all(&path);
                let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);
            }
        }

        let path_str = path.to_str().context("worktree path not valid UTF-8")?;

        // If the directory still exists (e.g. left over from a crash), force remove it
        if path.exists() {
            warn!("Worktree directory still exists at {path_str}, force removing");
            let _ = std::fs::remove_dir_all(&path);
            let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);
        }

        // Check if branch exists, create from base if not
        let branch_exists =
            ops::run_git(&self.repo_root, &["rev-parse", "--verify", &branch]).is_ok();

        if branch_exists {
            // Try adding the worktree — if it fails because of a locked worktree
            // reference or because the branch is already checked out, prune and
            // retry with --force (allows the same branch in multiple worktrees).
            if let Err(e) = ops::run_git(&self.repo_root, &["worktree", "add", path_str, &branch]) {
                warn!("Worktree add failed: {e} — pruning and retrying with --force");
                let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);
                ops::run_git(
                    &self.repo_root,
                    &["worktree", "add", "--force", path_str, &branch],
                )?;
            }
        } else {
            if let Err(e) = ops::run_git(
                &self.repo_root,
                &["worktree", "add", "-b", &branch, path_str, base_branch],
            ) {
                warn!("Worktree add -b failed: {e} — pruning and retrying");
                let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);
                ops::run_git(
                    &self.repo_root,
                    &["worktree", "add", "-b", &branch, path_str, base_branch],
                )?;
            }
        }

        if let Err(e) = write_cursor_cli_config(&path) {
            warn!("Failed to write .cursor/cli.json for plan worktree: {e}");
        }

        info!("Created plan worktree: {branch} at {path_str}");

        copy_repo_root_files_into_worktree(&self.repo_root, &path);
        if let Err(e) = crate::orchestrator::context::regenerate_workspace_map(&path) {
            warn!("Failed to regenerate workspace-map.md in plan worktree: {e}");
        }
        if let Err(e) = create_context_dirs(&path) {
            warn!("Failed to create context dirs for plan worktree: {e}");
        }

        Ok(PlanWorktree {
            path,
            branch,
            plan_base: plan_base.to_string(),
        })
    }

    /// Merge a plan worktree into the batch branch (sequential, not octopus).
    ///
    /// Strategy: merge batch into the plan worktree (so conflicts resolve there),
    /// then fast-forward the batch branch ref to the plan's HEAD.
    /// This avoids `git checkout` in the main repo which can fail when the
    /// batch branch is already checked out or the working tree is dirty.
    #[instrument(skip_all, fields(plan = %worktree.plan_base))]
    pub fn merge_plan_worktree(&self, worktree: &PlanWorktree, batch_branch: &str) -> Result<()> {
        // Step 1: Inside the plan worktree, merge the batch branch in.
        // This brings any changes from other plans into this worktree.
        // Use run_git_with_plumbing_author so merge commits can be created even without global git config.
        let merge_result = ops::run_git_with_plumbing_author(
            &worktree.path,
            &["merge", "--no-edit", batch_branch],
        );
        if let Err(e) = &merge_result {
            // F4: Capture conflicted files before aborting
            let status =
                ops::run_git(&worktree.path, &["status", "--porcelain"]).unwrap_or_default();
            let conflicted: Vec<&str> = status
                .lines()
                .filter(|l| l.starts_with("UU") || l.starts_with("AA") || l.starts_with("DD"))
                .collect();
            let _ = ops::run_git(&worktree.path, &["merge", "--abort"]);
            anyhow::bail!(
                "Merge conflict merging {batch_branch} into {}: {e}\nConflicted files: {}",
                worktree.branch,
                if conflicted.is_empty() {
                    "(unknown)".to_string()
                } else {
                    conflicted.join(", ")
                }
            );
        }

        // Step 2: Get the current HEAD of the plan worktree (which now
        // includes both plan changes and batch changes).
        let plan_head = ops::run_git(&worktree.path, &["rev-parse", "HEAD"])?;
        let plan_head = plan_head.trim();

        // Step 3: Fast-forward the batch branch to the plan's HEAD.
        // Using update-ref is safe regardless of what's checked out in main repo.
        ops::run_git(
            &self.repo_root,
            &[
                "update-ref",
                &format!("refs/heads/{batch_branch}"),
                plan_head,
            ],
        )?;

        // Step 4: If the main repo has batch branch checked out, sync the working tree.
        let current_branch = ops::run_git(&self.repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_default();
        if current_branch.trim() == batch_branch {
            info!("Batch branch {batch_branch} is checked out — syncing working tree");
            let _ = ops::run_git(&self.repo_root, &["reset", "--hard", "HEAD"]);
        }

        info!(
            "Merged {} into {} (plan head: {})",
            worktree.branch,
            batch_branch,
            &plan_head[..8.min(plan_head.len())]
        );

        // F1: Auto-tag on merge
        let tag_name = format!("plan/{}", worktree.plan_base);
        let tag_msg = format!(
            "Plan {} merged at {}",
            worktree.plan_base,
            chrono::Utc::now().to_rfc3339()
        );
        if let Err(e) = ops::run_git(
            &self.repo_root,
            &["tag", "-a", &tag_name, "-m", &tag_msg, plan_head],
        ) {
            warn!("Failed to create tag {tag_name}: {e}");
        } else {
            info!("Created tag {tag_name}");
        }

        Ok(())
    }

    /// Clean up a plan worktree after merge (or during reset).
    /// Resilient: handles in-progress merges, dirty state, and git worktree
    /// registry inconsistencies.
    #[instrument(skip_all, fields(plan = %worktree.plan_base))]
    pub fn cleanup_plan_worktree(&self, worktree: &PlanWorktree) -> Result<()> {
        // Log what we're about to destroy
        if worktree.path.exists() {
            let uncommitted = crate::git::ops::run_git(&worktree.path, &["status", "--porcelain"])
                .map(|s| s.lines().count())
                .unwrap_or(0);
            info!(
                "Cleaning up plan worktree for {} (uncommitted_files={})",
                worktree.plan_base, uncommitted
            );
        }
        if worktree.path.exists() {
            // Abort any in-progress merge in the worktree
            let _ = ops::run_git(&worktree.path, &["merge", "--abort"]);
            // Reset any dirty state so `git worktree remove` doesn't complain
            let _ = ops::run_git(&worktree.path, &["reset", "--hard"]);
            let _ = ops::run_git(&worktree.path, &["clean", "-fd"]);
        }

        // Try the normal remove first
        if let Err(e) = self.remove_worktree(&worktree.path) {
            warn!(
                "Normal worktree remove failed for {}: {e}",
                worktree.plan_base
            );
            // Fallback: force-remove the directory and prune the worktree registry
            if worktree.path.exists() {
                let _ = std::fs::remove_dir_all(&worktree.path);
            }
            let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);
        }

        // Clean up the branch (force delete, may already be gone)
        let _ = ops::run_git(&self.repo_root, &["branch", "-D", &worktree.branch]);
        info!("Cleaned up plan worktree for {}", worktree.plan_base);
        Ok(())
    }

    /// Merge a task worktree's branch back into the plan branch.
    ///
    /// Tries fast-forward first. Falls back to a merge commit if the
    /// plan branch has diverged (e.g. another task already merged in).
    ///
    /// Merges inside the plan worktree directory (`.worktrees/plan-{base}`)
    /// to avoid `git checkout` in the main repo, which would switch the
    /// working directory and remove files belonging to the batch branch.
    #[instrument(skip_all, fields(plan = %worktree.plan_base, task = %worktree.task_id))]
    pub fn merge_task(&self, worktree: &TaskWorktree) -> Result<()> {
        let plan_branch = format!("codex/plan/{}", worktree.plan_base);
        let plan_wt_path = self
            .worktree_base
            .join(format!("plan-{}", worktree.plan_base));

        // Determine where to run the merge: plan worktree if it exists,
        // otherwise fall back to update-ref (no checkout in main repo).
        if plan_wt_path.exists() {
            // Merge inside the plan worktree — safe, doesn't touch main repo
            let ff_result = ops::run_git(&plan_wt_path, &["merge", "--ff-only", &worktree.branch]);

            match ff_result {
                Ok(_) => {
                    info!(
                        "Fast-forward merged {} into {} (in plan worktree)",
                        worktree.branch, plan_branch
                    );
                }
                Err(_) => {
                    let msg = format!("merge(task): {} from {}", worktree.task_id, worktree.branch);
                    ops::run_git(
                        &plan_wt_path,
                        &["merge", "--no-ff", "-m", &msg, &worktree.branch],
                    )
                    .with_context(|| {
                        format!("failed to merge {} into {}", worktree.branch, plan_branch)
                    })?;
                    info!(
                        "Merge committed {} into {} (in plan worktree)",
                        worktree.branch, plan_branch
                    );
                }
            }
        } else {
            // Plan worktree doesn't exist — use update-ref for fast-forward,
            // or bail if branches have diverged (can't merge without a worktree).
            let task_head = ops::run_git(&self.repo_root, &["rev-parse", &worktree.branch])?
                .trim()
                .to_string();

            let is_ff = ops::run_git(
                &self.repo_root,
                &["merge-base", "--is-ancestor", &plan_branch, &task_head],
            )
            .is_ok();

            if is_ff {
                ops::run_git(
                    &self.repo_root,
                    &[
                        "update-ref",
                        &format!("refs/heads/{plan_branch}"),
                        &task_head,
                    ],
                )?;
                info!(
                    "Fast-forward merged {} into {} via update-ref",
                    worktree.branch, plan_branch
                );
            } else {
                anyhow::bail!(
                    "Cannot merge {} into {}: plan worktree not found and branches have diverged",
                    worktree.branch,
                    plan_branch
                );
            }
        }

        Ok(())
    }

    /// Remove a worktree directory and delete its associated branch.
    /// F3: Retries up to 3 times with 100ms delay before force-remove fallback.
    pub fn remove_worktree(&self, path: &Path) -> Result<()> {
        let branch = self.branch_for_worktree(path);

        let path_str = path.to_str().context("worktree path is not valid UTF-8")?;
        let mut last_err = None;
        for attempt in 0..3 {
            match ops::run_git(
                &self.repo_root,
                &["worktree", "remove", "--force", path_str],
            ) {
                Ok(_) => {
                    last_err = None;
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt < 2 {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }
        if let Some(e) = last_err {
            // F3: Force-remove fallback
            warn!("Worktree remove failed after 3 attempts for {path_str}: {e}, force removing");
            if path.exists() {
                let _ = std::fs::remove_dir_all(path);
            }
            let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);
        }

        if let Some(branch) = branch {
            if let Err(e) = ops::run_git(&self.repo_root, &["branch", "-D", &branch]) {
                warn!("Could not delete branch {branch}: {e}");
            }
        }

        Ok(())
    }

    /// Clean up a task worktree after its branch has been merged.
    #[instrument(skip_all, fields(plan = %worktree.plan_base, task = %worktree.task_id))]
    pub fn cleanup_task(&self, worktree: &TaskWorktree) -> Result<()> {
        self.remove_worktree(&worktree.path)?;
        // The branch might already be deleted by remove_worktree, try -d (safe delete)
        let _ = ops::run_git(&self.repo_root, &["branch", "-d", &worktree.branch]);
        info!(
            "Cleaned up worktree for {}/{}",
            worktree.plan_base, worktree.task_id
        );
        Ok(())
    }

    /// Remove all worktrees managed by this instance (everything under `.worktrees/`).
    ///
    /// Intended for cleanup on exit or abort.
    #[instrument(skip_all)]
    pub fn cleanup_all(&self) -> Result<()> {
        let entries = list_worktrees(&self.repo_root)?;
        for entry in entries {
            let path = PathBuf::from(&entry.path);
            if path.starts_with(&self.worktree_base) {
                if let Err(e) = self.remove_worktree(&path) {
                    warn!("Failed to remove worktree {}: {e}", path.display());
                }
            }
        }
        // Remove the directory itself if it's now empty
        if self.worktree_base.exists() {
            let _ = std::fs::remove_dir(&self.worktree_base);
        }
        Ok(())
    }

    /// Look up the branch associated with a worktree path.
    fn branch_for_worktree(&self, path: &Path) -> Option<String> {
        let entries = list_worktrees(&self.repo_root).ok()?;
        entries
            .into_iter()
            .find(|e| PathBuf::from(&e.path) == path)
            .map(|e| e.branch)
    }

    /// Validate and repair git state on startup after a crash.
    ///
    /// - Cleans up orphaned worktrees (directory gone but git still tracks them)
    /// - Detects worktrees in a conflicted state and resets them
    /// - Returns a list of plan bases that have valid, reusable worktrees
    pub fn validate_and_repair(&self) -> Result<Vec<String>> {
        // Prune worktrees whose directories no longer exist
        let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);

        let entries = list_worktrees(&self.repo_root)?;
        let mut valid_plans = Vec::new();

        for entry in &entries {
            let path = PathBuf::from(&entry.path);
            if !path.starts_with(&self.worktree_base) {
                continue;
            }

            // Check if the worktree directory actually exists
            if !path.exists() {
                warn!(
                    "Orphaned worktree ref for {} — pruning ref, preserving branch",
                    entry.branch
                );
                // Do NOT call remove_worktree here: that force-deletes the branch
                // with `git branch -D`, which can destroy unmerged plan work.
                // A plain `worktree prune` is enough to clean up the dead reference.
                let _ = ops::run_git(&self.repo_root, &["worktree", "prune"]);
                continue;
            }

            // Check for merge conflict state (MERGE_HEAD present)
            let merge_head = path.join(".git").join("MERGE_HEAD");
            // For linked worktrees, .git is a file pointing to the gitdir
            let has_conflict = merge_head.exists() || {
                let dot_git = path.join(".git");
                if dot_git.is_file() {
                    // Read the gitdir path and check for MERGE_HEAD there
                    std::fs::read_to_string(&dot_git)
                        .ok()
                        .and_then(|content| {
                            let gitdir = content.trim().strip_prefix("gitdir: ")?;
                            Some(PathBuf::from(gitdir).join("MERGE_HEAD").exists())
                        })
                        .unwrap_or(false)
                } else {
                    false
                }
            };

            if has_conflict {
                warn!(
                    "Worktree {} has unresolved merge conflict, resetting",
                    path.display()
                );
                let _ = ops::run_git(&path, &["merge", "--abort"]);
            }

            // Check for rebase in progress (handle linked worktrees where .git is a file)
            let gitdir_path = if let Ok(content) = std::fs::read_to_string(path.join(".git")) {
                // Linked worktree: .git is a file with "gitdir: <path>"
                content.trim().strip_prefix("gitdir: ").map(PathBuf::from)
            } else {
                // Normal worktree: .git is a directory
                Some(path.join(".git"))
            };

            if let Some(gitdir) = gitdir_path {
                if gitdir.join("rebase-merge").exists() || gitdir.join("rebase-apply").exists() {
                    warn!(
                        "Worktree {} has rebase in progress, aborting",
                        path.display()
                    );
                    let _ = ops::run_git(&path, &["rebase", "--abort"]);
                }
            }

            // Extract plan base from directory name
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(plan_base) = name.strip_prefix("plan-") {
                    // Don't include task worktrees: they match "...-T{digits}" pattern
                    let is_task_worktree = plan_base
                        .rfind("-T")
                        .map(|idx| plan_base[idx + 2..].chars().all(|c| c.is_ascii_digit()))
                        .unwrap_or(false);
                    if !is_task_worktree {
                        valid_plans.push(plan_base.to_string());
                    }
                }
            }
        }

        info!(
            "Worktree validation: {} valid plan worktrees found",
            valid_plans.len()
        );
        Ok(valid_plans)
    }

    /// List all active task worktrees (those matching `plan-{base}-T{n}` naming).
    pub fn active_task_worktrees(&self) -> Result<Vec<TaskWorktree>> {
        let entries = list_worktrees(&self.repo_root)?;
        let mut result = Vec::new();

        for entry in entries {
            let path = PathBuf::from(&entry.path);
            if !path.starts_with(&self.worktree_base) {
                continue;
            }

            // Parse directory name: plan-{plan_base}-{task_id}
            // Task IDs start with 'T', so we split on the last "-T" occurrence.
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let rest = match name.strip_prefix("plan-") {
                Some(r) => r,
                None => continue,
            };
            // Find the last "-T" to split plan_base from task_id
            if let Some(idx) = rest.rfind("-T") {
                let task_part = &rest[idx + 2..]; // skip "-T"
                                                  // Only match if what follows "-T" is all digits (task number)
                if task_part.chars().all(|c| c.is_ascii_digit()) {
                    let plan_base = rest[..idx].to_string();
                    let task_id = rest[idx + 1..].to_string(); // skip the '-', keep the 'T...'
                    result.push(TaskWorktree {
                        path,
                        branch: entry.branch,
                        plan_base,
                        task_id,
                    });
                }
            }
        }

        Ok(result)
    }
}
