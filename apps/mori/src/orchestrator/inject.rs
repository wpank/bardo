use super::artifacts::ArtifactStore;
use super::registry::Registry;
use super::schema::{CompletionReport, ReviewReport};
use crate::agent::AgentRole;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ContextInjector<'a> {
    pub artifact_store: &'a ArtifactStore,
    pub registry: &'a Registry,
    pub repo_root: &'a Path,
}

/// Owned version of ContextInjector for async spawning.
pub struct OwnedContextInjector {
    artifact_store: Arc<ArtifactStore>,
    registry: Arc<Registry>,
    repo_root: PathBuf,
}

impl ContextInjector<'_> {
    fn write_in_file(&self, worktree: &Path, name: &str, content: &str) -> Result<()> {
        let dir = worktree.join("context/in");
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join(name), content)?;
        Ok(())
    }

    fn copy_in_file(&self, worktree: &Path, name: &str, src: &Path) -> Result<()> {
        if src.exists() {
            let dir = worktree.join("context/in");
            std::fs::create_dir_all(&dir)?;
            std::fs::copy(src, dir.join(name))?;
        }
        Ok(())
    }

    /// Inject context/in/ before implementer agent starts.
    pub fn inject_for_implementer(
        &self,
        worktree: &Path,
        plan_num: &str,
        iter: u32,
        plan_deps: &[String],
    ) -> Result<()> {
        let ctx_dir = worktree.join("context/in");
        std::fs::create_dir_all(&ctx_dir)?;

        // Plan spec -- try exact match, then prefix match
        let plan_file = self.repo_root.join("plans").join(format!("{plan_num}.md"));
        if plan_file.exists() {
            std::fs::copy(&plan_file, ctx_dir.join("plan.md"))?;
        } else if let Ok(entries) = std::fs::read_dir(self.repo_root.join("plans")) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(plan_num) && name.ends_with(".md") {
                    std::fs::copy(entry.path(), ctx_dir.join("plan.md"))?;
                    break;
                }
            }
        }

        // Brief
        let brief_path = self
            .repo_root
            .join(format!("plans/context/briefs/{plan_num}-brief.md"));
        self.copy_in_file(worktree, "brief.md", &brief_path)?;

        // Tasks
        let tasks_path = self
            .repo_root
            .join(format!("plans/context/tasks/{plan_num}-tasks.toml"));
        self.copy_in_file(worktree, "tasks.toml", &tasks_path)?;

        // Workspace map
        let wmap_path = self.repo_root.join("plans/context/workspace-map.md");
        self.copy_in_file(worktree, "workspace-map.md", &wmap_path)?;

        // Preflight snapshot
        let pre_path = self.repo_root.join("plans/context/preflight-snapshot.md");
        self.copy_in_file(worktree, "preflight.md", &pre_path)?;

        // Cross-plan registry (authoritative path is plans/CONTEXT.md, not plans/context/)
        let cross = self.repo_root.join("plans/CONTEXT.md");
        self.copy_in_file(worktree, "cross-plan-context.md", &cross)?;

        // PRD2 extract + verify checklist (same paths agents use on disk)
        let prd2 = self
            .repo_root
            .join(format!("plans/context/prd2-extracts/{plan_num}-prd2.md"));
        self.copy_in_file(worktree, "prd2-extract.md", &prd2)?;
        let verify_tasks = self
            .repo_root
            .join(format!("plans/context/tasks/{plan_num}-verify-tasks.toml"));
        self.copy_in_file(worktree, "verify-tasks.toml", &verify_tasks)?;

        let ignored = self.repo_root.join("plans/context/ignored-tests.md");
        self.copy_in_file(worktree, "ignored-tests.md", &ignored)?;

        // Conductor / operator steering (if any)
        let nudge = self.repo_root.join("tmp/agent-messages.md");
        self.copy_in_file(worktree, "agent-messages.md", &nudge)?;

        // AGENTS.md (copy, not symlink)
        let agents_path = self.repo_root.join("AGENTS.md");
        self.copy_in_file(worktree, "agents.md", &agents_path)?;

        // Previous iteration reviews (iter > 1)
        if iter > 1 {
            let prev = self.artifact_store.prev_iter_summary(plan_num, iter - 1)?;
            if !prev.is_empty() {
                self.write_in_file(worktree, "prev-reviews.md", &prev)?;
            }
        }

        Ok(())
    }

    /// Inject context/in/ before reviewer agent starts.
    pub fn inject_for_reviewer(
        &self,
        worktree: &Path,
        plan_num: &str,
        iter: u32,
        plan_deps: &[String],
    ) -> Result<()> {
        self.inject_for_implementer(worktree, plan_num, iter, plan_deps)?;

        // Remove tasks.toml -- reviewers don't need it
        let _ = std::fs::remove_file(worktree.join("context/in/tasks.toml"));

        // Add completion summary from artifacts
        if let Ok(Some(completion)) = self.artifact_store.read_completion(plan_num, iter) {
            let summary = format!(
                "# Implementation Summary\n\n\
                 Compile: {:?}\n\
                 Tests: pass={} fail={} ignored={}\n\n\
                 ## Notes from Implementer\n{}\n\n\
                 ## Deviations\n{}\n",
                completion.compile_status,
                completion.test_counts.pass,
                completion.test_counts.fail,
                completion.test_counts.ignored,
                completion.notes_for_reviewers,
                if completion.deviations.is_empty() {
                    "None".to_string()
                } else {
                    completion.deviations.join("\n")
                },
            );
            self.write_in_file(worktree, "completion-summary.md", &summary)?;
        }

        Ok(())
    }

    /// Read context/out/review.json after reviewer finishes. Hard error if absent/invalid.
    pub fn collect_review(
        &self,
        worktree: &Path,
        plan: &str,
        iter: u32,
        role: AgentRole,
    ) -> Result<ReviewReport> {
        let path = worktree.join("context/out/review.json");
        let json = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "review.json missing for {plan}/{role} iter {iter} -- agent did not write output"
            )
        })?;
        serde_json::from_str::<ReviewReport>(&json)
            .with_context(|| format!("review.json invalid JSON for {plan}/{role} iter {iter}"))
    }

    /// Read context/out/completion.json after implementer finishes.
    pub fn collect_completion(
        &self,
        worktree: &Path,
        plan: &str,
        iter: u32,
    ) -> Result<CompletionReport> {
        let path = worktree.join("context/out/completion.json");
        let json = std::fs::read_to_string(&path)
            .with_context(|| format!("completion.json missing for plan {plan} iter {iter} -- implementer did not write output"))?;
        serde_json::from_str::<CompletionReport>(&json)
            .with_context(|| format!("completion.json invalid JSON for plan {plan} iter {iter}"))
    }
}

impl<'a> ContextInjector<'a> {
    /// Convert to owned version for async spawning.
    pub fn to_owned(&self) -> OwnedContextInjector {
        OwnedContextInjector {
            artifact_store: Arc::new(self.artifact_store.clone()),
            registry: Arc::new(self.registry.clone()),
            repo_root: self.repo_root.to_path_buf(),
        }
    }
}

impl OwnedContextInjector {
    /// Async wrapper: inject context for implementer in background.
    /// Returns a JoinHandle that completes when injection is done.
    pub fn pre_inject_implementer_async(
        self,
        worktree: PathBuf,
        plan_num: String,
        iter: u32,
        plan_deps: Vec<String>,
    ) -> tokio::task::JoinHandle<Result<()>> {
        tokio::task::spawn_blocking(move || {
            let artifact_store = self.artifact_store.as_ref();
            let registry = self.registry.as_ref();
            let repo_root = self.repo_root.as_path();

            let injector = ContextInjector {
                artifact_store,
                registry,
                repo_root,
            };

            injector.inject_for_implementer(&worktree, &plan_num, iter, &plan_deps)
        })
    }

    /// Async wrapper: inject context for reviewer in background.
    /// Returns a JoinHandle that completes when injection is done.
    pub fn pre_inject_reviewer_async(
        self,
        worktree: PathBuf,
        plan_num: String,
        iter: u32,
        plan_deps: Vec<String>,
    ) -> tokio::task::JoinHandle<Result<()>> {
        tokio::task::spawn_blocking(move || {
            let artifact_store = self.artifact_store.as_ref();
            let registry = self.registry.as_ref();
            let repo_root = self.repo_root.as_path();

            let injector = ContextInjector {
                artifact_store,
                registry,
                repo_root,
            };

            injector.inject_for_reviewer(&worktree, &plan_num, iter, &plan_deps)
        })
    }
}
