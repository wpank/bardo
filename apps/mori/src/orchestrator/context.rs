use anyhow::Result;
use std::path::Path;
use std::time::Duration;
use tracing::info;

/// Generate workspace-map.md content by listing all crate source files
pub fn generate_workspace_map(repo_root: &Path) -> Result<String> {
    info!("Generating workspace map");
    let mut lines = vec!["# Workspace Map\n".to_string()];

    let crates_dir = repo_root.join("crates");
    if crates_dir.exists() {
        lines.push("## Library Crates\n".to_string());
        let mut entries: Vec<_> = std::fs::read_dir(&crates_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let src_dir = entry.path().join("src");
            if src_dir.exists() {
                lines.push(format!("### {name}\n"));
                collect_rs_files(&src_dir, &src_dir, &mut lines)?;
                lines.push(String::new());
            }
        }
    }

    let apps_dir = repo_root.join("apps");
    if apps_dir.exists() {
        lines.push("## Application Crates\n".to_string());
        let mut entries: Vec<_> = std::fs::read_dir(&apps_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let src_dir = entry.path().join("src");
            if src_dir.exists() {
                lines.push(format!("### {name}\n"));
                collect_rs_files(&src_dir, &src_dir, &mut lines)?;
                lines.push(String::new());
            }
        }
    }

    Ok(lines.join("\n"))
}

/// Generate a workspace map filtered to only include specific crates.
/// Used for agents that only need to see the crates they're working on.
pub fn generate_filtered_workspace_map(repo_root: &Path, crates: &[String]) -> Result<String> {
    if crates.is_empty() {
        return generate_workspace_map(repo_root);
    }

    let crate_set: std::collections::HashSet<&str> = crates.iter().map(|s| s.as_str()).collect();
    let mut lines = vec!["# Workspace Map (filtered)\n".to_string()];

    for prefix in &["crates", "apps"] {
        let dir = repo_root.join(prefix);
        if !dir.exists() {
            continue;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut section_lines = Vec::new();
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if !crate_set.contains(name.as_str()) {
                continue;
            }
            let src_dir = entry.path().join("src");
            if src_dir.exists() {
                section_lines.push(format!("### {name}\n"));
                collect_rs_files(&src_dir, &src_dir, &mut section_lines)?;
                section_lines.push(String::new());
            }
        }

        if !section_lines.is_empty() {
            let heading = if *prefix == "crates" {
                "## Library Crates\n"
            } else {
                "## Application Crates\n"
            };
            lines.push(heading.to_string());
            lines.extend(section_lines);
        }
    }

    Ok(lines.join("\n"))
}

fn collect_rs_files(dir: &Path, base: &Path, lines: &mut Vec<String>) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, base, lines)?;
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let rel = path.strip_prefix(base).unwrap_or(&path);
            lines.push(format!("- `{}`", rel.display()));
        }
    }
    Ok(())
}

/// Generate preflight-snapshot.md content
pub fn generate_preflight_snapshot(repo_root: &Path) -> Result<String> {
    let mut lines = vec!["# Preflight Snapshot\n".to_string()];

    // Git log (last 10 commits)
    lines.push("## Recent Commits\n".to_string());
    let git_log = std::process::Command::new("git")
        .args(["log", "--oneline", "-10"])
        .current_dir(repo_root)
        .output()?;
    lines.push(format!(
        "```\n{}\n```\n",
        String::from_utf8_lossy(&git_log.stdout)
    ));

    // Git status
    lines.push("## Git Status\n".to_string());
    let git_status = std::process::Command::new("git")
        .args(["status", "--short"])
        .current_dir(repo_root)
        .output()?;
    lines.push(format!(
        "```\n{}\n```\n",
        String::from_utf8_lossy(&git_status.stdout)
    ));

    // Cargo check status
    lines.push("## Cargo Check\n".to_string());
    let cargo_check = std::process::Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(repo_root)
        .output()?;
    let check_status = if cargo_check.status.success() {
        "OK"
    } else {
        "FAILED"
    };
    lines.push(format!("Status: {check_status}\n"));

    Ok(lines.join("\n"))
}

/// Generate and write the pre-computed context bundle for a plan.
/// Written to plans/context/bundles/{num}-bundle.md.
/// Called during express mode startup before any agents run.
pub fn generate_context_bundle(repo_root: &Path, plan_num: &str) -> Result<()> {
    let bundles_dir = repo_root.join("plans/context/bundles");
    std::fs::create_dir_all(&bundles_dir)?;

    let bundle_path = bundles_dir.join(format!("{plan_num}-bundle.md"));
    if bundle_path.exists() {
        // Idempotent: skip if already generated
        return Ok(());
    }

    // Pre-extract PRD2 context if not already done
    extract_prd2_context(repo_root, plan_num)?;

    // Write a placeholder bundle; the full bundle is built by generate_static_brief()
    // in prompts.rs once the plan is loaded. This ensures the directory exists.
    let prd2 = read_prd2_extract(repo_root, plan_num)?;
    let ctx = read_context(repo_root)?;
    let workspace_map = read_workspace_map(repo_root)?;

    let content = format!(
        "# Context Bundle: Plan {plan_num}\n\n\
         ## PRD2 Extract\n\n{prd2}\n\n\
         ## Cross-Plan Context\n\n{ctx}\n\n\
         ## Workspace Map\n\n{workspace_map}\n"
    );
    std::fs::write(&bundle_path, content)?;
    Ok(())
}

/// Write workspace-map.md and preflight-snapshot.md to plans/context/
pub fn write_preflight_files(repo_root: &Path) -> Result<()> {
    let context_dir = repo_root.join("plans/context");
    std::fs::create_dir_all(&context_dir)?;

    let workspace_map = generate_workspace_map(repo_root)?;
    std::fs::write(context_dir.join("workspace-map.md"), workspace_map)?;

    let preflight = generate_preflight_snapshot(repo_root)?;
    std::fs::write(context_dir.join("preflight-snapshot.md"), preflight)?;

    // Ensure CONTEXT.md exists
    let context_file = repo_root.join("plans/CONTEXT.md");
    if !context_file.exists() {
        std::fs::write(
            &context_file,
            "# Cross-Plan Context\n\n## Types Defined\n\n## Crate Boundaries\n\n## Public Traits & Signatures\n\n## Decisions & Deviations\n\n## Open Questions\n",
        )?;
    }

    // Ensure all context subdirectories exist
    std::fs::create_dir_all(context_dir.join("briefs"))?;
    std::fs::create_dir_all(context_dir.join("reviews"))?;
    std::fs::create_dir_all(context_dir.join("tasks"))?;
    std::fs::create_dir_all(context_dir.join("docs"))?;
    std::fs::create_dir_all(context_dir.join("archive"))?;
    std::fs::create_dir_all(context_dir.join("summaries"))?;
    std::fs::create_dir_all(context_dir.join("prd2-extracts"))?;
    std::fs::create_dir_all(context_dir.join("checklists"))?;
    std::fs::create_dir_all(context_dir.join("review-context"))?;
    std::fs::create_dir_all(context_dir.join("bundles"))?;
    std::fs::create_dir_all(context_dir.join("completion"))?;
    std::fs::create_dir_all(context_dir.join("events"))?;
    std::fs::create_dir_all(context_dir.join("gate-results"))?;
    std::fs::create_dir_all(context_dir.join("iteration-memory"))?;

    // Ensure ignored-tests.md exists
    let test_ledger = context_dir.join("ignored-tests.md");
    if !test_ledger.exists() {
        std::fs::write(
            &test_ledger,
            "# Ignored Tests Ledger\n\n| Test | Crate | Reason | Unblock Plan |\n|------|-------|--------|--------------|\n",
        )?;
    }

    Ok(())
}

/// Archive current iteration files before starting a new iteration.
/// Moves briefs, reviews, and docs for the given plan to archive/{num}/iter-{N}/
pub fn archive_iteration(repo_root: &Path, plan_num: &str, iteration: u32) -> Result<()> {
    let context_dir = repo_root.join("plans/context");
    let archive_dir = context_dir.join(format!("archive/{plan_num}/iter-{iteration}"));
    std::fs::create_dir_all(&archive_dir)?;

    // Move brief
    let brief = context_dir.join(format!("briefs/{plan_num}-brief.md"));
    if brief.exists() {
        std::fs::rename(&brief, archive_dir.join(format!("{plan_num}-brief.md")))?;
    }

    // Move reviews
    for suffix in &["arch", "audit", "critic"] {
        let review = context_dir.join(format!("reviews/{plan_num}-{suffix}.md"));
        if review.exists() {
            std::fs::rename(&review, archive_dir.join(format!("{plan_num}-{suffix}.md")))?;
        }
    }

    // Move docs
    let docs = context_dir.join(format!("docs/{plan_num}-docs.md"));
    if docs.exists() {
        std::fs::rename(&docs, archive_dir.join(format!("{plan_num}-docs.md")))?;
    }

    // Move tasks
    let tasks = context_dir.join(format!("tasks/{plan_num}-tasks.toml"));
    if tasks.exists() {
        std::fs::rename(&tasks, archive_dir.join(format!("{plan_num}-tasks.toml")))?;
    }

    info!(
        "Archived iteration {iteration} for plan {plan_num} to {}",
        archive_dir.display()
    );
    Ok(())
}

/// Read CONTEXT.md
pub fn read_context(repo_root: &Path) -> Result<String> {
    let path = repo_root.join("plans/CONTEXT.md");
    Ok(std::fs::read_to_string(path).unwrap_or_default())
}

/// Read workspace-map.md
pub fn read_workspace_map(repo_root: &Path) -> Result<String> {
    let path = repo_root.join("plans/context/workspace-map.md");
    Ok(std::fs::read_to_string(path).unwrap_or_default())
}

/// Regenerate workspace-map.md by scanning the given directory (typically a plan worktree).
/// Returns the generated content.
pub fn regenerate_workspace_map(scan_root: &Path) -> Result<String> {
    info!("Regenerating workspace map from {}", scan_root.display());
    let content = generate_workspace_map(scan_root)?;
    let context_dir = scan_root.join("plans/context");
    std::fs::create_dir_all(&context_dir)?;
    std::fs::write(context_dir.join("workspace-map.md"), &content)?;
    Ok(content)
}

/// Read ignored-tests.md
pub fn read_ignored_tests(repo_root: &Path) -> Result<String> {
    let path = repo_root.join("plans/context/ignored-tests.md");
    Ok(std::fs::read_to_string(path).unwrap_or_default())
}

/// Read pre-extracted prd2 context for a plan.
pub fn read_prd2_extract(repo_root: &Path, plan_num: &str) -> Result<String> {
    let path = repo_root.join(format!("plans/context/prd2-extracts/{plan_num}-prd2.md"));
    Ok(std::fs::read_to_string(path).unwrap_or_default())
}

/// Run the prd2 extraction script for a plan. Idempotent.
///
/// Captures script stdout/stderr so progress lines do not corrupt the ratatui alternate screen
/// (the TUI owns `stdout`).
pub fn extract_prd2_context(repo_root: &Path, plan_num: &str) -> Result<()> {
    let script = repo_root.join("plans/context/prompts/extract-prd2-context.sh");
    if !script.exists() {
        return Ok(());
    }
    let output = std::process::Command::new("bash")
        .arg(&script)
        .arg(plan_num)
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        tracing::warn!(
            "prd2 extraction failed for plan {plan_num}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !stderr.is_empty() {
            tracing::info!("{stderr}");
        }
    }
    Ok(())
}

/// Read pre-generated checklist for a plan.
pub fn read_checklist(repo_root: &Path, plan_num: &str) -> Result<String> {
    let path = repo_root.join(format!("plans/context/checklists/{plan_num}-checklist.md"));
    Ok(std::fs::read_to_string(path).unwrap_or_default())
}

/// Read pre-generated verify-tasks for a plan.
pub fn read_verify_tasks(repo_root: &Path, plan_num: &str) -> Result<String> {
    let path = repo_root.join(format!("plans/context/tasks/{plan_num}-verify-tasks.toml"));
    Ok(std::fs::read_to_string(path).unwrap_or_default())
}

/// Read an existing summary for a plan
pub fn read_summary(repo_root: &Path, plan_num: &str) -> Result<Option<String>> {
    let path = repo_root.join(format!("plans/context/summaries/{plan_num}-summary.md"));
    if path.exists() {
        Ok(Some(std::fs::read_to_string(path)?))
    } else {
        Ok(None)
    }
}

/// Write a summary for a plan
pub fn write_summary(repo_root: &Path, plan_num: &str, content: &str) -> Result<()> {
    let dir = repo_root.join("plans/context/summaries");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{plan_num}-summary.md")), content)?;
    Ok(())
}

/// Generate an executive summary for a completed plan.
/// Parses last-completed.md, runs git diff --stat, and composes structured markdown.
pub fn generate_summary(
    repo_root: &Path,
    plan: &super::plan::PlanInfo,
    iteration: u32,
    elapsed: Duration,
) -> Result<String> {
    let last_completed_path = repo_root.join("plans/context/last-completed.md");
    let last_completed = std::fs::read_to_string(&last_completed_path).unwrap_or_default();

    // Extract sections from last-completed.md
    let what_changed = extract_section(&last_completed, "What Changed")
        .unwrap_or_else(|| "No details available.".to_string());
    let test_status =
        extract_section(&last_completed, "Test Status").unwrap_or_else(|| "Unknown".to_string());
    let deviations = extract_section(&last_completed, "Deviations")
        .or_else(|| extract_section(&last_completed, "Deviation"))
        .unwrap_or_else(|| "None".to_string());

    // Get git diff --stat for the plan's tag
    let diff_stat = std::process::Command::new("git")
        .args([
            "diff",
            "--stat",
            &format!("plan/{}~1..plan/{}", plan.base, plan.base),
        ])
        .current_dir(repo_root)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            // Fallback: just show recent diff stat
            std::process::Command::new("git")
                .args(["diff", "--stat", "HEAD~1"])
                .current_dir(repo_root)
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_else(|| "Unable to determine.".to_string())
        });

    // Detect affected crates from diff stat
    let affected_crates = detect_affected_crates(&diff_stat);
    let verify_commands = if affected_crates.is_empty() {
        "cargo check --workspace\ncargo test --workspace".to_string()
    } else {
        let mut cmds: Vec<String> = affected_crates
            .iter()
            .flat_map(|c| vec![format!("cargo check -p {c}"), format!("cargo test -p {c}")])
            .collect();
        cmds.push("cargo check --workspace".to_string());
        cmds.join("\n")
    };

    let mins = elapsed.as_secs() / 60;
    let secs = elapsed.as_secs() % 60;

    let summary = format!(
        "# Executive Summary: Plan {} - {}\n\n\
         ## What Was Built\n{}\n\n\
         ## Files Modified\n```\n{}\n```\n\n\
         ## Test Results\n{}\n\n\
         ## Verification Commands\n```\n{}\n```\n\n\
         ## Deviations From Plan\n{}\n\n\
         ## Time & Iterations\n\
         - Iterations: {}\n\
         - Total time: {}m {:02}s\n",
        plan.num,
        plan.base,
        what_changed,
        diff_stat.trim(),
        test_status,
        verify_commands,
        deviations,
        iteration,
        mins,
        secs,
    );

    Ok(summary)
}

/// Info about a completed plan for batch summary generation
#[derive(Debug, Clone)]
pub struct CompletedPlanInfo {
    pub base: String,
    pub num: String,
    pub elapsed: Duration,
    pub iterations: u32,
    pub test_count: Option<usize>,
    pub file_count: Option<usize>,
    pub issues: Vec<String>,
}

/// Generate a batch summary for all completed plans
pub fn generate_batch_summary(
    batch_id: &str,
    completed_plans: &[CompletedPlanInfo],
    total_plans: usize,
    total_tasks: usize,
    wall_clock_elapsed: Duration,
    critical_path_remaining_min: u32,
    total_tokens: (u64, u64),
) -> String {
    let completed = completed_plans.len();
    let elapsed_h = wall_clock_elapsed.as_secs() / 3600;
    let elapsed_m = (wall_clock_elapsed.as_secs() % 3600) / 60;
    let remaining_h = critical_path_remaining_min / 60;
    let remaining_m = critical_path_remaining_min % 60;
    let input_m = total_tokens.0 as f64 / 1_000_000.0;
    let output_m = total_tokens.1 as f64 / 1_000_000.0;

    let mut lines = vec![
        format!("# Batch: codex/batch/{batch_id}"),
        String::new(),
        format!("## Progress: {completed}/{total_plans} plans | {total_tasks} tasks"),
        format!("## Wall clock: {elapsed_h}h {elapsed_m:02}m | Critical path: {remaining_h}h {remaining_m:02}m remaining"),
        format!("## Tokens: {input_m:.1}M input, {output_m:.1}M output"),
        String::new(),
    ];

    for plan in completed_plans {
        let mins = plan.elapsed.as_secs() / 60;
        let tests = plan
            .test_count
            .map(|n| format!(", {} tests", n))
            .unwrap_or_default();
        let files = plan
            .file_count
            .map(|n| format!(", {} files", n))
            .unwrap_or_default();
        lines.push(format!(
            "### {} ({}m, {} iter{}{})",
            plan.base, mins, plan.iterations, tests, files
        ));
        for issue in &plan.issues {
            lines.push(format!("- {issue}"));
        }
        lines.push(String::new());
    }

    // Verification section
    lines.push("### Verification".to_string());
    lines.push("```".to_string());
    lines.push("cargo check --workspace".to_string());
    lines.push("cargo test --workspace --no-fail-fast".to_string());
    lines.push("```".to_string());
    lines.push(String::new());

    // Manual verification
    lines.push("### Manual Verification Commands".to_string());
    lines.push("```".to_string());
    lines.push(format!("git checkout codex/batch/{batch_id}"));
    lines.push("cargo test --workspace --no-fail-fast".to_string());
    lines.push("```".to_string());

    lines.join("\n")
}

/// Write batch summary to plans/context/summaries/
pub fn write_batch_summary(repo_root: &Path, batch_id: &str, content: &str) -> Result<()> {
    let dir = repo_root.join("plans/context/summaries");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("batch-{batch_id}.md")), content)?;
    Ok(())
}

/// Extract a markdown section by heading name from content
fn extract_section(content: &str, heading: &str) -> Option<String> {
    let patterns = [format!("## {heading}"), format!("### {heading}")];

    for pattern in &patterns {
        if let Some(start_idx) = content.find(pattern.as_str()) {
            let after_heading = &content[start_idx + pattern.len()..];
            // Find next heading
            let end = after_heading
                .find("\n## ")
                .or_else(|| after_heading.find("\n### "))
                .or_else(|| after_heading.find("\n# "))
                .unwrap_or(after_heading.len());
            let section = after_heading[..end].trim().to_string();
            if !section.is_empty() {
                return Some(section);
            }
        }
    }
    None
}

/// Detect affected crate names from git diff --stat output
fn detect_affected_crates(diff_stat: &str) -> Vec<String> {
    let mut crates = std::collections::HashSet::new();
    for line in diff_stat.lines() {
        let trimmed = line.trim();
        // Match paths like crates/foo/... or apps/foo/...
        if let Some(rest) = trimmed
            .strip_prefix("crates/")
            .or_else(|| trimmed.strip_prefix("apps/"))
        {
            if let Some(slash) = rest.find('/') {
                crates.insert(rest[..slash].to_string());
            }
        }
    }
    let mut sorted: Vec<String> = crates.into_iter().collect();
    sorted.sort();
    sorted
}
