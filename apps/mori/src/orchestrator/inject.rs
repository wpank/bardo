use super::artifacts::ArtifactStore;
use super::memory::PlaybookConfig;
use super::registry::Registry;
use super::schema::{CompletionReport, ReviewReport};
use crate::agent::AgentRole;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::warn;

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

        let plans_dir = crate::orchestrator::paths::plans_root(&self.repo_root);

        // Plan spec -- try per-plan directory first, then flat file, then prefix scan
        if let Some(plan_dir) = crate::orchestrator::paths::find_plan_dir(&plans_dir, plan_num) {
            // New layout: plans/{base}/plan.md
            self.copy_in_file(worktree, "plan.md", &plan_dir.join("plan.md"))?;

            // Per-plan artifacts from the directory
            self.copy_in_file(worktree, "brief.md", &plan_dir.join("brief.md"))?;
            self.copy_in_file(worktree, "tasks.toml", &plan_dir.join("tasks.toml"))?;
            self.copy_in_file(
                worktree,
                "prd2-extract.md",
                &plan_dir.join("prd-extract.md"),
            )?;
            self.copy_in_file(
                worktree,
                "verify-tasks.toml",
                &plan_dir.join("verify-tasks.toml"),
            )?;
        } else {
            // Legacy flat layout
            let plan_file = plans_dir.join(format!("{plan_num}.md"));
            if plan_file.exists() {
                std::fs::copy(&plan_file, ctx_dir.join("plan.md"))?;
            } else if let Ok(entries) = std::fs::read_dir(&plans_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with(plan_num) && name.ends_with(".md") {
                        std::fs::copy(entry.path(), ctx_dir.join("plan.md"))?;
                        break;
                    }
                }
            }

            // Legacy per-plan artifacts
            let brief_path = plans_dir.join(format!("context/briefs/{plan_num}-brief.md"));
            self.copy_in_file(worktree, "brief.md", &brief_path)?;

            let tasks_path = plans_dir.join(format!("context/tasks/{plan_num}-tasks.toml"));
            self.copy_in_file(worktree, "tasks.toml", &tasks_path)?;

            let prd2 = plans_dir.join(format!("context/prd2-extracts/{plan_num}-prd2.md"));
            self.copy_in_file(worktree, "prd2-extract.md", &prd2)?;

            let verify_tasks =
                plans_dir.join(format!("context/tasks/{plan_num}-verify-tasks.toml"));
            self.copy_in_file(worktree, "verify-tasks.toml", &verify_tasks)?;
        }

        // Global artifacts -- use paths::global_artifact for new/legacy resolution
        let wmap_path = crate::orchestrator::paths::global_artifact(&plans_dir, "workspace-map.md");
        self.copy_in_file(worktree, "workspace-map.md", &wmap_path)?;

        let pre_path =
            crate::orchestrator::paths::global_artifact(&plans_dir, "preflight-snapshot.md");
        self.copy_in_file(worktree, "preflight.md", &pre_path)?;

        let ignored = crate::orchestrator::paths::global_artifact(&plans_dir, "ignored-tests.md");
        self.copy_in_file(worktree, "ignored-tests.md", &ignored)?;

        // Cross-plan registry (authoritative path is plans/CONTEXT.md, not plans/context/)
        let cross = self.repo_root.join("plans/CONTEXT.md");
        self.copy_in_file(worktree, "cross-plan-context.md", &cross)?;

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

        // Playbook injection: match rules from .mori/memory/playbook.toml
        // against the plan's task files and inject matched advice.
        let playbook_path = self.repo_root.join(".mori/memory/playbook.toml");
        if playbook_path.exists() {
            match std::fs::read_to_string(&playbook_path) {
                Ok(content) => match toml::from_str::<PlaybookConfig>(&content) {
                    Ok(playbook) => {
                        // Collect all file paths from this plan's task list
                        let plan_files: Vec<String> =
                            crate::orchestrator::tasks::load_checklist(self.repo_root, plan_num)
                                .ok()
                                .flatten()
                                .map(|cl| cl.tasks.iter().flat_map(|t| t.files.clone()).collect())
                                .unwrap_or_default();

                        let matched = playbook.match_rules(&plan_files, &[]);
                        if !matched.is_empty() {
                            let mut md = String::from("# Playbook Notes (from prior builds)\n\n");
                            for rule in &matched {
                                md.push_str(&format!("- {}\n", rule.context));
                            }
                            self.write_in_file(worktree, "playbook.md", &md)?;
                        }
                    }
                    Err(e) => warn!("playbook: failed to parse playbook.toml: {e}"),
                },
                Err(e) => warn!("playbook: failed to read playbook.toml: {e}"),
            }
        }

        // Inject prior iteration reflections (reflexion loop)
        match super::iteration_memory::IterationMemory::load(self.repo_root, plan_num) {
            Ok(mem) => {
                if let Some(reflections_md) = mem.format_reflections_md() {
                    self.write_in_file(worktree, "reflections.md", &reflections_md)?;
                }
            }
            Err(e) => warn!("Failed to load iteration memory for {plan_num}: {e}"),
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
