//! Plan artifact path resolution.
//!
//! Supports three layouts (checked in order):
//! - `.mori/plans/{base}/plan.md` (current default)
//! - `plans/{base}/plan.md` (alternate)
//! - `plans/{base}.md` + `plans/context/briefs/{num}-brief.md` (legacy flat)
//!
//! First match wins. This allows gradual migration without breaking anything.

use std::path::{Path, PathBuf};

/// Resolve the runs directory for runtime state (status, events, logs).
///
/// Checks `.mori/runs/` first, falls back to `tmp/plan-runs/`.
pub fn runs_dir(repo_root: &Path) -> PathBuf {
    let mori_runs = repo_root.join(".mori/runs");
    if mori_runs.is_dir() {
        return mori_runs;
    }
    repo_root.join("tmp/plan-runs")
}

/// Resolve the plans root directory.
///
/// Checks `.mori/plans/` first, then `plans/`. Returns whichever exists and
/// contains plan directories (or .md files). Falls back to `plans/` if neither
/// has content.
pub fn plans_root(repo_root: &Path) -> PathBuf {
    let mori_plans = repo_root.join(".mori/plans");
    if mori_plans.is_dir() {
        // Check it actually has content (not just an empty dir)
        if let Ok(mut entries) = std::fs::read_dir(&mori_plans) {
            if entries.next().is_some() {
                return mori_plans;
            }
        }
    }
    // Fallback to legacy plans/
    repo_root.join("plans")
}

/// Find the plan directory matching a plan number prefix.
///
/// Scans `plans_dir` for a directory whose name starts with `plan_num`
/// (followed by `-` or end-of-string) and contains `plan.md`.
/// Handles all plan number formats: "01", "08a", "R01", "Q01", etc.
pub fn find_plan_dir(plans_dir: &Path, plan_num: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(plans_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir()
                && name.starts_with(plan_num)
                && (name.len() == plan_num.len()
                    || name.as_bytes().get(plan_num.len()) == Some(&b'-'))
                && entry.path().join("plan.md").exists()
            {
                return Some(entry.path());
            }
        }
    }
    None
}

/// Resolve the base name for a plan number.
///
/// If the new-layout directory exists, returns its name (e.g., "01-workspace-scaffold").
/// Otherwise returns `None` and callers fall back to flat-file naming.
pub fn find_plan_base(plans_dir: &Path, plan_num: &str) -> Option<String> {
    find_plan_dir(plans_dir, plan_num)
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
}

/// Resolve the path to a plan's main document.
pub fn plan_doc(plans_dir: &Path, base: &str) -> PathBuf {
    // New: plans/{base}/plan.md
    let new_path = plans_dir.join(base).join("plan.md");
    if new_path.exists() {
        return new_path;
    }
    // Legacy: plans/{base}.md
    plans_dir.join(format!("{base}.md"))
}

/// Resolve a per-plan artifact (brief, tasks, etc).
///
/// `base` is the full plan base name (e.g. "01-workspace-scaffold").
/// `artifact` is the filename in the new layout (e.g. "brief.md").
/// `legacy_subdir` is the context subdirectory (e.g. "briefs").
/// `legacy_name` is the filename in the legacy layout (e.g. "01-brief.md").
pub fn plan_artifact(
    plans_dir: &Path,
    base: &str,
    artifact: &str,
    legacy_subdir: &str,
    legacy_name: &str,
) -> PathBuf {
    // New: plans/{base}/{artifact}
    let new_path = plans_dir.join(base).join(artifact);
    if new_path.exists() {
        return new_path;
    }
    // Legacy: plans/context/{legacy_subdir}/{legacy_name}
    plans_dir
        .join("context")
        .join(legacy_subdir)
        .join(legacy_name)
}

/// Resolve a per-plan artifact using the plan number to find the directory.
///
/// Combines `find_plan_dir` + `plan_artifact`. If no per-plan directory exists,
/// falls back directly to the legacy path.
pub fn plan_artifact_by_num(
    plans_dir: &Path,
    plan_num: &str,
    artifact: &str,
    legacy_subdir: &str,
    legacy_name: &str,
) -> PathBuf {
    if let Some(plan_dir) = find_plan_dir(plans_dir, plan_num) {
        let new_path = plan_dir.join(artifact);
        if new_path.exists() {
            return new_path;
        }
    }
    // Legacy: plans/context/{legacy_subdir}/{legacy_name}
    plans_dir
        .join("context")
        .join(legacy_subdir)
        .join(legacy_name)
}

/// Resolve a global context file.
///
/// Checks `plans/{name}` first (new layout), then `plans/context/{name}` (legacy).
pub fn global_artifact(plans_dir: &Path, name: &str) -> PathBuf {
    // New: plans/{name}
    let new_path = plans_dir.join(name);
    if new_path.exists() {
        return new_path;
    }
    // Legacy: plans/context/{name}
    plans_dir.join("context").join(name)
}

/// Resolve the registry directory.
pub fn registry_dir(plans_dir: &Path) -> PathBuf {
    // New: plans/registry/
    let new_path = plans_dir.join("registry");
    if new_path.exists() {
        return new_path;
    }
    // Legacy: plans/context/registry/
    plans_dir.join("context").join("registry")
}

/// Resolve the reviews directory for a plan.
pub fn reviews_dir(plans_dir: &Path, base: &str) -> PathBuf {
    // New: plans/{base}/reviews/
    let new_path = plans_dir.join(base).join("reviews");
    if new_path.exists() {
        return new_path;
    }
    // Legacy: plans/context/reviews/
    plans_dir.join("context").join("reviews")
}

/// Resolve a relative path string that might reference new or legacy layout.
///
/// Given a `relative_path` like `plans/context/briefs/{num}-brief.md`, checks
/// if there's a per-plan directory equivalent. Used by `optional_context_file_section`
/// in prompts.rs to resolve paths that are already formatted as legacy strings.
pub fn resolve_context_path(repo_root: &Path, relative_path: &str) -> PathBuf {
    let full = repo_root.join(relative_path);
    if full.exists() {
        return full;
    }
    // If the path looks like plans/context/{subdir}/{num}-{artifact}, try the new layout
    if let Some(rest) = relative_path.strip_prefix("plans/context/") {
        if let Some((subdir, filename)) = rest.split_once('/') {
            // Extract plan number from filename (e.g., "01-brief.md" -> "01")
            if let Some(num_end) = filename.find('-') {
                let plan_num = &filename[..num_end];
                let plans_dir = plans_root(repo_root);
                if let Some(plan_dir) = find_plan_dir(&plans_dir, plan_num) {
                    // Map legacy subdir/filename to new artifact name
                    let artifact = legacy_to_new_artifact(subdir, filename);
                    let new_path = plan_dir.join(&artifact);
                    if new_path.exists() {
                        return new_path;
                    }
                }
            }
        }
    }
    full
}

/// Map a legacy subdir + filename to the new per-plan artifact name.
///
/// Examples:
/// - ("briefs", "01-brief.md") -> "brief.md"
/// - ("tasks", "01-tasks.toml") -> "tasks.toml"
/// - ("tasks", "01-verify-tasks.toml") -> "verify-tasks.toml"
/// - ("prd2-extracts", "01-prd2.md") -> "prd-extract.md"
/// - ("decompositions", "01-decomposition.md") -> "decomposition.md"
/// - ("tasks", "01-review-tasks.toml") -> "review-tasks.toml"
/// - ("tasks", "01-scribe-tasks.toml") -> "scribe-tasks.toml"
/// - ("tasks", "01-shared-rubric.md") -> "shared-rubric.md"
fn legacy_to_new_artifact(subdir: &str, filename: &str) -> String {
    // Strip the plan number prefix (everything up to and including the first '-')
    let without_num = if let Some(pos) = filename.find('-') {
        &filename[pos + 1..]
    } else {
        filename
    };

    match subdir {
        "briefs" => "brief.md".to_string(),
        "prd2-extracts" => "prd-extract.md".to_string(),
        "decompositions" => "decomposition.md".to_string(),
        "verify-chains" => without_num.to_string(),
        _ => without_num.to_string(),
    }
}
