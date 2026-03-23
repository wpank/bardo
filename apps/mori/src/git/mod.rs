pub mod graph;
pub mod ops;
pub mod worktree;

use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{info, warn};

const AUTO_STASH_MESSAGE: &str =
    "bardo-ctl: auto-stash before checkout (local changes would be overwritten)";

fn checkout_would_overwrite_local_changes(git_error: &str) -> bool {
    let m = git_error.to_lowercase();
    m.contains("would be overwritten by checkout")
        || m.contains("would be overwritten by merge")
        || (m.contains("local changes") && m.contains("overwritten") && m.contains("checkout"))
}

/// Tie auto-stash restoration to the lifetime of a `bardo-ctl` run. On drop, reapplies stashes
/// created when checkout would have overwritten local edits (e.g. to `bardo-ctl.sh`).
pub struct AutoStashSession<'a> {
    gm: &'a GitManager,
}

impl<'a> AutoStashSession<'a> {
    pub fn new(gm: &'a GitManager) -> Self {
        Self { gm }
    }
}

impl Drop for AutoStashSession<'_> {
    fn drop(&mut self) {
        self.gm.restore_auto_stashed_changes();
    }
}

/// Events from git operations
#[derive(Debug, Clone)]
pub enum GitEvent {
    BranchCreated { name: String },
    Committed { hash: String, message: String },
    Merged { source: String, target: String },
    Tagged { name: String },
    Error { message: String },
}

pub struct GitManager {
    repo_root: PathBuf,
    event_tx: tokio::sync::mpsc::UnboundedSender<GitEvent>,
    /// Count of `git stash push` calls we performed for checkout; popped on session end.
    auto_stash_depth: AtomicU32,
}

impl GitManager {
    pub fn new(repo_root: PathBuf, event_tx: tokio::sync::mpsc::UnboundedSender<GitEvent>) -> Self {
        Self {
            repo_root,
            event_tx,
            auto_stash_depth: AtomicU32::new(0),
        }
    }

    fn note_auto_stash(&self) {
        self.auto_stash_depth.fetch_add(1, Ordering::SeqCst);
    }

    fn unnote_auto_stash(&self) {
        let _ = self
            .auto_stash_depth
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                Some(n.saturating_sub(1))
            });
    }

    /// Pop every auto-stash recorded during this run (LIFO order). Safe to call multiple times.
    pub fn restore_auto_stashed_changes(&self) {
        let n = self.auto_stash_depth.swap(0, Ordering::SeqCst);
        if n == 0 {
            return;
        }
        info!("Restoring {n} automatic git stash(es) from bardo-ctl checkout…");
        for i in 1..=n {
            match ops::run_git_with_plumbing_author(&self.repo_root, &["stash", "pop"]) {
                Ok(_) => info!("Applied auto-stash {i}/{n}"),
                Err(e) => {
                    warn!(
                        "Auto-stash pop failed at {i}/{n}: {e} — run `git stash list` and apply manually if needed"
                    );
                    break;
                }
            }
        }
    }

    fn emit(&self, event: GitEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Result<String> {
        ops::current_branch(&self.repo_root)
    }

    /// Create and checkout a new branch from the current HEAD
    pub fn create_branch(&self, name: &str) -> Result<()> {
        ops::run_git(&self.repo_root, &["checkout", "-b", name])?;
        self.emit(GitEvent::BranchCreated {
            name: name.to_string(),
        });
        info!("Created branch: {name}");
        Ok(())
    }

    /// Checkout an existing branch.
    /// If checkout fails due to dirty index (e.g. from a failed merge),
    /// cleans up and retries.
    pub fn checkout(&self, name: &str) -> Result<()> {
        match ops::run_git(&self.repo_root, &["checkout", name]) {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("resolve your current index")
                    || msg.contains("needs merge")
                    || msg.contains("not possible because you have unmerged")
                {
                    warn!("Checkout {name} failed with dirty index, cleaning up: {e}");
                    let _ = ops::run_git(&self.repo_root, &["merge", "--abort"]);
                    let _ = ops::run_git(&self.repo_root, &["reset", "--hard"]);
                    ops::run_git(&self.repo_root, &["checkout", name])?;
                    Ok(())
                } else if checkout_would_overwrite_local_changes(&msg) {
                    warn!(
                        "Checkout {name} blocked by local changes — stashing ({AUTO_STASH_MESSAGE}), then retrying"
                    );
                    ops::run_git_with_plumbing_author(
                        &self.repo_root,
                        &["stash", "push", "-m", AUTO_STASH_MESSAGE],
                    )?;
                    self.note_auto_stash();
                    match ops::run_git(&self.repo_root, &["checkout", name]) {
                        Ok(_) => Ok(()),
                        Err(e2) => {
                            self.unnote_auto_stash();
                            warn!("Checkout {name} failed after auto-stash: {e2} — restoring your working tree from stash");
                            if let Err(pop_e) = ops::run_git_with_plumbing_author(
                                &self.repo_root,
                                &["stash", "pop"],
                            ) {
                                warn!(
                                    "Could not pop auto-stash: {pop_e} — inspect `git stash list`"
                                );
                            }
                            Err(e2)
                        }
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Check if a branch exists
    pub fn branch_exists(&self, name: &str) -> bool {
        ops::run_git(&self.repo_root, &["rev-parse", "--verify", name]).is_ok()
    }

    /// Create or resume a batch branch. If the branch already exists, check it out.
    ///
    /// Used in sequential mode where agents commit directly in the main worktree.
    /// For parallel mode, use [`ensure_batch_branch`] instead.
    pub fn setup_batch_branch(&self, batch_branch: &str) -> Result<()> {
        if self.branch_exists(batch_branch) {
            if let Ok(cur) = self.current_branch() {
                if cur.trim() == batch_branch {
                    info!("Already on {batch_branch}, skipping checkout");
                    return Ok(());
                }
            }
            info!("Batch branch {batch_branch} exists, resuming");
            self.checkout(batch_branch)?;
        } else {
            self.create_branch(batch_branch)?;
        }
        Ok(())
    }

    /// Ensure the batch branch exists without checking it out.
    ///
    /// In parallel mode, all commits happen inside plan worktrees and merges use
    /// `git update-ref`, so the main worktree never needs to switch branches.
    /// Switching it is harmful: it changes the user's working directory and can
    /// make in-progress files disappear.
    pub fn ensure_batch_branch(&self, batch_branch: &str) -> Result<()> {
        if self.branch_exists(batch_branch) {
            info!("Batch branch {batch_branch} already exists");
            return Ok(());
        }
        // Create from current HEAD without switching to it.
        ops::run_git(&self.repo_root, &["branch", batch_branch])?;
        self.emit(GitEvent::BranchCreated {
            name: batch_branch.to_string(),
        });
        info!("Created batch branch: {batch_branch}");
        Ok(())
    }

    /// Create or resume a plan branch.
    ///
    /// If the plan branch already exists AND has commits ahead of the batch
    /// branch, it's a partially-completed plan from a prior run — check it out
    /// and resume. Otherwise, create fresh from the batch branch.
    pub fn setup_plan_branch(&self, plan_base: &str, batch_branch: &str) -> Result<()> {
        let plan_branch = format!("codex/plan/{plan_base}");

        if self.branch_exists(&plan_branch) {
            // Check if the plan branch has commits ahead of batch
            let ahead = ops::run_git(
                &self.repo_root,
                &[
                    "rev-list",
                    "--count",
                    &format!("{batch_branch}..{plan_branch}"),
                ],
            )
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

            if ahead > 0 {
                info!(
                    "Resuming plan branch {plan_branch} ({ahead} commits ahead of {batch_branch})"
                );
                self.checkout(&plan_branch)?;
                return Ok(());
            }

            // Branch exists but no work on it — delete and recreate from batch HEAD
            info!("Plan branch {plan_branch} exists but has no work, recreating");
            self.checkout(batch_branch)?;
            let _ = ops::run_git(&self.repo_root, &["branch", "-D", &plan_branch]);
        } else {
            self.checkout(batch_branch)?;
        }

        self.create_branch(&plan_branch)?;
        Ok(())
    }

    /// Like commit_all but operates in an arbitrary directory (e.g. a plan worktree).
    pub fn commit_all_in(&self, dir: &std::path::Path, message: &str) -> Result<String> {
        ops::run_git(dir, &["add", "-A"])?;
        let status = ops::run_git(dir, &["status", "--porcelain"])?;
        if status.trim().is_empty() {
            warn!("Nothing to commit in {}", dir.display());
            return ops::current_commit_hash(dir);
        }
        ops::run_git(dir, &["commit", "-m", message])?;
        let hash = ops::current_commit_hash(dir)?;
        self.emit(GitEvent::Committed {
            hash: hash.clone(),
            message: message.to_string(),
        });
        info!("Committed in worktree: {hash} -- {message}");
        Ok(hash)
    }

    /// Stage all changes, commit, and return the commit hash
    pub fn commit_all(&self, message: &str) -> Result<String> {
        ops::run_git(&self.repo_root, &["add", "-A"])?;

        // Check if there's anything to commit
        let status = ops::run_git(&self.repo_root, &["status", "--porcelain"])?;
        if status.trim().is_empty() {
            warn!("Nothing to commit");
            return ops::current_commit_hash(&self.repo_root);
        }

        ops::run_git(&self.repo_root, &["commit", "-m", message])?;
        let hash = ops::current_commit_hash(&self.repo_root)?;
        self.emit(GitEvent::Committed {
            hash: hash.clone(),
            message: message.to_string(),
        });
        info!("Committed: {hash} -- {message}");
        Ok(hash)
    }

    /// Merge plan branch into batch branch with --no-ff
    pub fn merge_plan_to_batch(&self, plan_base: &str, batch_branch: &str) -> Result<()> {
        let plan_branch = format!("codex/plan/{plan_base}");
        self.checkout(batch_branch)?;
        let message = format!("merge(batch): codex/plan/{plan_base}");
        ops::run_git(
            &self.repo_root,
            &["merge", "--no-ff", "-m", &message, &plan_branch],
        )?;
        self.emit(GitEvent::Merged {
            source: plan_branch.clone(),
            target: batch_branch.to_string(),
        });
        info!("Merged {plan_branch} -> {batch_branch}");

        // Delete the plan branch
        ops::run_git(&self.repo_root, &["branch", "-d", &plan_branch])?;
        Ok(())
    }

    /// Merge batch branch into a staging branch for final review
    pub fn merge_batch_to_staging(&self, batch_branch: &str, staging_branch: &str) -> Result<()> {
        if !self.branch_exists(staging_branch) {
            ops::run_git(&self.repo_root, &["checkout", "-b", staging_branch, "main"])?;
        } else {
            self.checkout(staging_branch)?;
        }
        let message = format!("merge(batch): {batch_branch}");
        ops::run_git(
            &self.repo_root,
            &["merge", "--no-ff", batch_branch, "-m", &message],
        )?;
        self.emit(GitEvent::Merged {
            source: batch_branch.to_string(),
            target: staging_branch.to_string(),
        });
        info!("Merged {batch_branch} -> {staging_branch}");
        Ok(())
    }

    /// Merge batch branch into main with --no-ff. Returns the merge commit hash.
    pub fn merge_batch_to_main(&self, batch_branch: &str) -> Result<String> {
        self.checkout("main")?;
        let message = format!("merge(main): {batch_branch}");
        ops::run_git(
            &self.repo_root,
            &["merge", "--no-ff", batch_branch, "-m", &message],
        )?;
        let hash = ops::current_commit_hash(&self.repo_root)?;
        self.emit(GitEvent::Merged {
            source: batch_branch.to_string(),
            target: "main".to_string(),
        });
        info!("Merged {batch_branch} -> main at {hash}");
        Ok(hash)
    }

    /// Tag the current commit
    pub fn tag(&self, name: &str) -> Result<()> {
        ops::run_git(&self.repo_root, &["tag", name])?;
        self.emit(GitEvent::Tagged {
            name: name.to_string(),
        });
        info!("Tagged: {name}");
        Ok(())
    }

    /// Delete a tag if it exists
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        if self.tag_exists(name) {
            ops::run_git(&self.repo_root, &["tag", "-d", name])?;
            info!("Deleted tag: {name}");
        }
        Ok(())
    }

    /// Check if a tag exists
    pub fn tag_exists(&self, name: &str) -> bool {
        ops::run_git(&self.repo_root, &["tag", "-l", name])
            .map(|out| !out.trim().is_empty())
            .unwrap_or(false)
    }

    /// Get recent log entries
    pub fn log_oneline(&self, count: usize) -> Result<String> {
        ops::run_git(&self.repo_root, &["log", "--oneline", &format!("-{count}")])
    }

    /// Get diff stat
    pub fn diff_stat(&self) -> Result<String> {
        ops::run_git(&self.repo_root, &["diff", "--stat"])
    }

    /// Get diff between a base branch and HEAD
    pub fn diff_branch(&self, base: &str) -> Result<String> {
        ops::run_git(&self.repo_root, &["diff", base, "HEAD"])
    }

    /// Get a WorktreeManager rooted at this repo.
    pub fn worktree_manager(&self) -> worktree::WorktreeManager {
        worktree::WorktreeManager::new(self.repo_root.clone())
    }
}
