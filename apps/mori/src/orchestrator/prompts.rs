use anyhow::Result;
use std::path::Path;

use super::context;
use super::plan::PlanInfo;
use super::skills;
use crate::agent::AgentRole;
use crate::git::ops;

// ---------------------------------------------------------------------------
// Dynamic prompt budgeting
// ---------------------------------------------------------------------------

/// Per-section character budgets for prompt construction.
/// Computed dynamically based on model context window size.
pub struct PromptBudget {
    pub plan: usize,
    pub workspace_map: usize,
    pub prd2: usize,
    pub context: usize,
    pub brief: usize,
    pub reviews: usize,
    pub instructions: usize,
    pub file_context: usize,
    pub skills: usize,
}

/// Compute prompt budgets based on the role and model's context window.
/// Assumes ~4 chars per token as a rough estimate.
pub fn budget_for(role: AgentRole, model: &str) -> PromptBudget {
    let total_ctx_tokens: usize = if model.contains("opus") || model.contains("sonnet") {
        200_000
    } else if model.contains("gpt-5") || model.contains("o3") || model.contains("o4") {
        128_000
    } else {
        100_000
    };

    // Reserve 40% for output, 60% for input
    let input_tokens = (total_ctx_tokens as f64 * 0.6) as usize;
    // Convert tokens to approximate chars (4 chars/token)
    let input_chars = input_tokens * 4;

    match role {
        // Implementer gets the most context — needs plan, prd2, brief, file content
        AgentRole::Implementer => PromptBudget {
            plan: input_chars * 25 / 100,
            workspace_map: input_chars * 10 / 100,
            prd2: input_chars * 20 / 100,
            context: input_chars * 5 / 100,
            brief: input_chars * 10 / 100,
            reviews: input_chars * 10 / 100,
            instructions: input_chars * 5 / 100,
            file_context: input_chars * 10 / 100,
            skills: input_chars * 5 / 100,
        },
        // Strategist needs more workspace map + plan for analysis
        AgentRole::Strategist => PromptBudget {
            plan: input_chars * 30 / 100,
            workspace_map: input_chars * 20 / 100,
            prd2: input_chars * 20 / 100,
            context: input_chars * 7 / 100,
            brief: input_chars * 5 / 100,
            reviews: input_chars * 10 / 100,
            instructions: input_chars * 5 / 100,
            file_context: input_chars * 0 / 100,
            skills: input_chars * 5 / 100,
        },
        // Reviewers (Architect, Auditor) need plan + workspace map
        AgentRole::Architect | AgentRole::Auditor => PromptBudget {
            plan: input_chars * 25 / 100,
            workspace_map: input_chars * 15 / 100,
            prd2: input_chars * 15 / 100,
            context: input_chars * 2 / 100,
            brief: input_chars * 10 / 100,
            reviews: input_chars * 15 / 100,
            instructions: input_chars * 5 / 100,
            file_context: input_chars * 8 / 100,
            skills: input_chars * 5 / 100,
        },
        // Scribe — heavier PRD2 context for academic citation preservation
        AgentRole::Scribe => PromptBudget {
            plan: input_chars * 25 / 100,
            workspace_map: input_chars * 10 / 100,
            prd2: input_chars * 20 / 100,
            context: input_chars * 7 / 100,
            brief: input_chars * 10 / 100,
            reviews: input_chars * 10 / 100,
            instructions: input_chars * 5 / 100,
            file_context: input_chars * 8 / 100,
            skills: input_chars * 5 / 100,
        },
        // Critic — reviews scribe output, lighter needs
        AgentRole::Critic => PromptBudget {
            plan: input_chars * 25 / 100,
            workspace_map: input_chars * 10 / 100,
            prd2: input_chars * 10 / 100,
            context: input_chars * 7 / 100,
            brief: input_chars * 15 / 100,
            reviews: input_chars * 15 / 100,
            instructions: input_chars * 5 / 100,
            file_context: input_chars * 8 / 100,
            skills: input_chars * 5 / 100,
        },
        // Default for all other roles
        _ => PromptBudget {
            plan: input_chars * 25 / 100,
            workspace_map: input_chars * 15 / 100,
            prd2: input_chars * 15 / 100,
            context: input_chars * 7 / 100,
            brief: input_chars * 10 / 100,
            reviews: input_chars * 10 / 100,
            instructions: input_chars * 5 / 100,
            file_context: input_chars * 8 / 100,
            skills: input_chars * 5 / 100,
        },
    }
}

/// A prompt section with priority for budget-aware assembly.
pub struct PromptSection {
    pub name: &'static str,
    pub content: String,
    /// 5 = always include, 1 = drop first when budget tight.
    pub priority: u8,
    /// Never include more than this many chars, regardless of budget.
    pub hard_cap: Option<usize>,
    /// Cache layer for prefix alignment (1=role, 2=workspace, 3=plan, 0=unique).
    /// The gateway places `cache_control` breakpoints at layer transitions.
    pub cache_layer: u8,
}

/// Assemble a prompt from priority-ranked sections.
/// char_budget ≈ token_budget × 4
pub fn assemble_prompt(mut sections: Vec<PromptSection>, token_budget: usize) -> String {
    let char_budget = token_budget * 4;

    // Apply hard caps
    for s in &mut sections {
        if let Some(cap) = s.hard_cap {
            if s.content.len() > cap {
                s.content.truncate(cap);
                s.content.push_str("\n...(truncated)");
            }
        }
    }

    // Build index sorted by priority desc
    let mut order: Vec<usize> = (0..sections.len()).collect();
    order.sort_by(|&a, &b| sections[b].priority.cmp(&sections[a].priority));

    let mut included: Vec<bool> = vec![false; sections.len()];
    let mut used = 0usize;

    for i in order {
        let len = sections[i].content.len();
        if used + len <= char_budget {
            used += len;
            included[i] = true;
        } else if sections[i].priority >= 5 {
            let remaining = char_budget.saturating_sub(used);
            if remaining > 100 {
                sections[i].content.truncate(remaining);
                sections[i].content.push_str("\n...(truncated)");
                used += sections[i].content.len();
                included[i] = true;
            }
        }
    }

    // Emit in original order, inserting layer markers at transitions.
    // The gateway uses these markers to place cache_control breakpoints.
    let mut result = Vec::new();
    let mut last_layer: u8 = 0;
    for (i, s) in sections.into_iter().enumerate() {
        if !included[i] {
            continue;
        }
        if s.cache_layer > 0 && s.cache_layer != last_layer {
            result.push(format!("<!-- mori:layer:{} -->", s.cache_layer));
            last_layer = s.cache_layer;
        }
        result.push(s.content);
    }
    result.join("\n\n")
}

/// Build implementer prompt sections from injected context files.
pub fn implementer_sections(
    agents_md: &str,
    plan_md: &str,
    brief_md: &str,
    tasks_toml: &str,
    workspace_map: &str,
    preflight: &str,
    registry_snapshot: &str,
    prev_reviews: Option<&str>,
) -> Vec<PromptSection> {
    let mut sections = vec![
        PromptSection {
            name: "agents_instructions",
            content: agents_md.to_string(),
            priority: 5,
            hard_cap: None,
            cache_layer: 1, // Stable across all agents of same role
        },
        PromptSection {
            name: "plan_spec",
            content: plan_md.to_string(),
            priority: 5,
            hard_cap: Some(50_000),
            cache_layer: 3, // Stable within a plan
        },
        PromptSection {
            name: "brief",
            content: brief_md.to_string(),
            priority: 4,
            hard_cap: None,
            cache_layer: 3,
        },
        PromptSection {
            name: "tasks",
            content: tasks_toml.to_string(),
            priority: 3,
            hard_cap: None,
            cache_layer: 0, // Unique per task
        },
        PromptSection {
            name: "workspace_map",
            content: workspace_map.to_string(),
            priority: 3,
            hard_cap: Some(20_000),
            cache_layer: 2, // Stable within a build
        },
        PromptSection {
            name: "preflight",
            content: preflight.to_string(),
            priority: 3,
            hard_cap: Some(5_000),
            cache_layer: 2, // Stable within a build
        },
        PromptSection {
            name: "registry",
            content: registry_snapshot.to_string(),
            priority: 2,
            hard_cap: Some(8_000),
            cache_layer: 0, // Changes as plans complete
        },
    ];
    if let Some(reviews) = prev_reviews {
        sections.push(PromptSection {
            name: "prev_reviews",
            content: reviews.to_string(),
            priority: 4,
            hard_cap: Some(15_000),
            cache_layer: 0, // Unique per iteration
        });
    }
    sections
}

/// Truncate content to max chars, cutting at line boundaries to avoid breaking
/// markdown/code syntax. Keeps the beginning which has the most important context.
pub(crate) fn truncate(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.len() <= max_chars {
        return s.to_string();
    }
    // Find a char boundary at or before max_chars
    let limit = (0..=max_chars.min(s.len()))
        .rev()
        .find(|&i| s.is_char_boundary(i))
        .unwrap_or(0);
    // Find the last newline before the limit
    let cut_point = s[..limit].rfind('\n').unwrap_or(limit);
    let total_lines = s.lines().count();
    let kept_lines = s[..cut_point].lines().count();
    format!(
        "{}\n\n[... truncated {}/{} lines, {} total chars]",
        &s[..cut_point],
        total_lines - kept_lines,
        total_lines,
        s.len()
    )
}

/// Truncate from the end, keeping the LAST `max_chars` (for error output where
/// the most recent errors are at the bottom).
fn truncate_tail(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        return s.to_string();
    }
    let raw_start = s.len() - max_chars;
    // Find a char boundary at or after raw_start
    let start = (raw_start..=s.len())
        .find(|&i| s.is_char_boundary(i))
        .unwrap_or(s.len());
    // Find the first newline after start to avoid breaking mid-line
    let cut_point = s[start..]
        .find('\n')
        .map(|i| start + i + 1)
        .unwrap_or(start);
    format!("[... earlier output truncated]\n{}", &s[cut_point..])
}

/// Extract cross-plan API changes from recent commits
/// Reads git log for recent commits and formats API changes into a brief section
fn cross_plan_diff_section(repo_root: &Path, crates_touched: &[String]) -> String {
    if crates_touched.is_empty() {
        return String::new();
    }

    // Try to get recent commits that touched the target crates
    match ops::run_git(
        repo_root,
        &["log", "--oneline", "-20", "--follow", "--", "crates/"],
    ) {
        Ok(log_output) => {
            let mut changes = Vec::new();
            let lines: Vec<&str> = log_output.lines().collect();

            for (idx, line) in lines.iter().take(10).enumerate() {
                // Look for commits mentioning API changes
                if line.contains("add")
                    || line.contains("remove")
                    || line.contains("change")
                    || line.contains("refactor")
                    || line.contains("trait")
                    || line.contains("pub fn")
                {
                    // Try to get the commit message
                    let commit_hash = line.split_whitespace().next().unwrap_or("");
                    if !commit_hash.is_empty() {
                        if let Ok(commit_msg) =
                            ops::run_git(repo_root, &["log", "--format=%B", "-1", commit_hash])
                        {
                            let msg = commit_msg.trim();
                            if !msg.is_empty() && msg.len() < 200 {
                                changes.push(format!("- {}", msg.lines().next().unwrap_or(msg)));
                            }
                        }
                    }
                }
            }

            if !changes.is_empty() {
                format!(
                    "## Cross-Plan API Changes (recent commits)\n\n\
                     These changes may affect your implementation:\n\n{}\n",
                    changes.join("\n")
                )
            } else {
                String::new()
            }
        }
        Err(_) => String::new(), // Silently skip if git fails
    }
}

/// Where plan-scoped artifacts live under `plans/context/` (for secondary roles).
pub const CONTEXT_LAYOUT_STANZA: &str = r#"## Plans context layout

- `plans/context/workspace-map.md` — crate file tree; use this instead of `find`/`ls` on `crates/`.
- `plans/context/preflight-snapshot.md` — ambient compile/test baseline when present.
- `plans/CONTEXT.md` — cross-plan registry (types, boundaries, decisions).
- `plans/context/ignored-tests.md` — ledger of `#[ignore]` tests.
- `plans/context/prd2-extracts/{NN}-prd2.md` — PRD2 extracts per plan (optional).
- `plans/context/decompositions/{NN}-decomposition.md` — step breakdown (optional).
- `plans/context/tasks/{NN}-tasks.toml`, `{NN}-verify-tasks.toml`, `{NN}-review-tasks.toml`, `{NN}-scribe-tasks.toml` — checklists.
- `plans/context/verify-chains/{NN}-verify.sh` — invariant runner when generated (optional).
- `plans/context/briefs/{NN}-brief.md` — implementation brief when present (legacy Strategist output or hand-written).
- `plans/context/bundles/{NN}-bundle.md` — distiller output: single-file digest of key artifacts (optional).
- `plans/context/review-context/{NN}-review-context.md` — aggregated review inputs when generated (optional).
- `plans/context/golden-path-index.json` — crate → `CLAUDE.md` pointers (optional).
- `plans/context/completion/` — per-plan completion reports; `last-completed.md` is the latest pointer when synced.
- `tmp/agent-messages.md` — if non-empty, read first: conductor/supervisor steering for the active pipeline (do not delete).
- `context/in/` — when the orchestrator injects files into a worktree, mirrored copies of the above may appear here (`brief.md`, `prd2-extract.md`, etc.).
"#;

fn read_agents_md(repo_root: &Path) -> String {
    let root_agents = repo_root.join("AGENTS.md");
    if root_agents.is_file() {
        return std::fs::read_to_string(&root_agents).unwrap_or_default();
    }
    let legacy = repo_root.join("agents/AGENTS.md");
    std::fs::read_to_string(&legacy).unwrap_or_default()
}

/// Injects a titled XML block when `root/relative_path` exists and is non-empty.
///
/// Resolves paths through the new per-plan directory layout first, falling
/// back to the legacy `plans/context/` layout.
fn optional_context_file_section(
    root: &Path,
    relative_path: &str,
    section_title: &str,
    xml_tag: &str,
    max_chars: usize,
) -> String {
    let path = super::paths::resolve_context_path(root, relative_path);
    if !path.is_file() {
        return String::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if content.is_empty() {
        return String::new();
    }
    format!(
        "\n## {section_title}\n\n<{xml_tag}>\n{body}\n</{xml_tag}>\n",
        section_title = section_title,
        xml_tag = xml_tag,
        body = truncate(&content, max_chars),
    )
}

fn optional_verify_chain_section(root: &Path, plan_num: &str) -> String {
    let plans_dir = super::paths::plans_root(root);
    let path = super::paths::plan_artifact_by_num(
        &plans_dir,
        plan_num,
        &format!("{plan_num}-verify.sh"),
        "verify-chains",
        &format!("{plan_num}-verify.sh"),
    );
    if !path.is_file() {
        return String::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let snippet = truncate(&content, 4000);
    // Show the path relative to repo root for the agent prompt
    let rel = path
        .strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());
    format!(
        "\n## Verify chain script\n\nPath: `{rel}`. After implementation, run `bash {rel}` when validating invariants.\n\n```bash\n{snippet}\n```\n"
    )
}

// ---------------------------------------------------------------------------
// Shared plan context — identical prefix for all agents within a plan run
// ---------------------------------------------------------------------------

/// Shared context that is identical across all agents within a single plan execution.
/// Constructed once per plan, passed to all prompt builders.
/// Ordering is fixed to maximize prompt cache hits — API providers cache
/// byte-identical prefixes at 90% token discount.
#[derive(Debug, Clone)]
pub struct SharedPlanContext {
    /// AGENTS.md conventions (stable across all plans)
    pub system_prefix: String,
    /// Same for all agents in this plan
    pub prd2_extract: String,
    /// Same for all agents in this plan
    pub plan_content: String,
    /// Same within an iteration (may change after implementation)
    pub workspace_map: String,
    /// CONTEXT.md cross-plan context
    pub cross_plan_ctx: String,
    /// Strategist brief (same for reviewers + implementer)
    pub brief: String,
}

/// Build the shared context once per plan entry.
/// All agents receive the same prefix in the same byte order for cache hits.
pub fn build_shared_context(
    repo_root: &Path,
    plan: &PlanInfo,
    _iteration: u32,
) -> Result<SharedPlanContext> {
    let plan_content = super::plan::read_plan(plan)?;
    let prd2_extract = context::read_prd2_extract(repo_root, &plan.num)?;
    let workspace_map = context::read_workspace_map(repo_root)?;
    let cross_plan_ctx = context::read_context(repo_root)?;

    // Read the strategist brief if it exists
    let plans_dir = super::paths::plans_root(repo_root);
    let brief_path = super::paths::plan_artifact_by_num(
        &plans_dir,
        &plan.num,
        "brief.md",
        "briefs",
        &format!("{}-brief.md", plan.num),
    );
    let brief = std::fs::read_to_string(&brief_path).unwrap_or_default();

    // Repo-root AGENTS.md (fallback: agents/AGENTS.md for older layouts)
    let system_prefix = read_agents_md(repo_root);

    Ok(SharedPlanContext {
        system_prefix,
        prd2_extract,
        plan_content,
        workspace_map,
        cross_plan_ctx,
        brief,
    })
}

/// Format the shared prefix sections in a fixed order for cache hits.
/// Returns the prefix string that should appear at the start of every prompt.
pub fn format_shared_prefix(ctx: &SharedPlanContext, budget: &PromptBudget) -> String {
    let plan = truncate(&ctx.plan_content, budget.plan);
    let prd2 = truncate(&ctx.prd2_extract, budget.prd2);
    let workspace = truncate(&ctx.workspace_map, budget.workspace_map);
    let cross_plan = truncate(&ctx.cross_plan_ctx, budget.context);

    // Fixed ordering: system -> plan -> prd2 -> workspace -> cross-plan
    // This ordering is load-bearing for prompt caching — do not rearrange.
    let mut prefix =
        String::with_capacity(plan.len() + prd2.len() + workspace.len() + cross_plan.len() + 512);

    if !ctx.system_prefix.is_empty() {
        prefix.push_str(&format!(
            "## Agent Conventions\n\n<conventions>\n{}\n</conventions>\n\n",
            truncate(&ctx.system_prefix, budget.instructions)
        ));
    }

    prefix.push_str(&format!(
        "## Plan\n\n<plan>\n{plan}\n</plan>\n\n\
         ## PRD2 Specification Context\n\n<prd2-context>\n{prd2}\n</prd2-context>\n\n\
         ## Workspace Map\n\n<workspace-map>\n{workspace}\n</workspace-map>\n\n\
         ## Cross-Plan Context\n\n<context>\n{cross_plan}\n</context>\n\n"
    ));

    prefix
}

/// Read all completion summaries from plans/context/completion/*-summary.md.
/// Also checks per-plan directories for summary.md files (new layout).
/// Returns them joined with separators, suitable for prompt injection.
fn read_completion_summaries(repo_root: &Path) -> String {
    let mut summaries = Vec::new();

    // Legacy: plans/context/completion/*-summary.md
    let completion_dir = repo_root.join("plans/context/completion");
    if completion_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&completion_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "md").unwrap_or(false)
                    && entry.file_name().to_string_lossy().ends_with("-summary.md")
                {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        summaries.push(content);
                    }
                }
            }
        }
    }

    // New: plans/{base}/summary.md
    let plans_dir = super::paths::plans_root(repo_root);
    if let Ok(entries) = std::fs::read_dir(&plans_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let summary_path = entry.path().join("summary.md");
                if summary_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&summary_path) {
                        summaries.push(content);
                    }
                }
            }
        }
    }

    summaries.join("\n---\n")
}

/// Compress review feedback for iteration 2+.
///
/// Reads the archived architect + auditor reviews from the prior iteration,
/// tries to parse their structured TOML blocks, and produces a tight "fix
/// directive" containing only unresolved blocking issues. If structured parsing
/// fails for both reviews, returns None to fall back to raw review excerpts.
///
/// Result: ~500-1000 chars instead of ~6000 chars of raw markdown.
fn compress_feedback(repo_root: &Path, plan_num: &str, prior_iter: u32) -> Option<String> {
    let mut all_issues: Vec<String> = Vec::new();
    let mut parsed_any = false;

    for suffix in &["arch"] {
        let archive_path = repo_root.join(format!(
            "plans/context/archive/{plan_num}/iter-{prior_iter}/{plan_num}-{suffix}.md"
        ));
        let content = std::fs::read_to_string(&archive_path).ok()?;
        if let Some(review) = super::review::parse_structured_review(&content) {
            parsed_any = true;
            for issue in review.unresolved_blocking() {
                let line_ref = issue.line.map(|l| format!(":{l}")).unwrap_or_default();
                all_issues.push(format!(
                    "- [{}] `{}{}` — {} (hint: {})",
                    issue.id, issue.file, line_ref, issue.description, issue.fix_hint
                ));
            }
        }
    }

    if !parsed_any {
        return None;
    }

    if all_issues.is_empty() {
        return Some("### Compressed Feedback\n\nNo unresolved blocking issues from prior reviews. Proceed with implementation.\n".to_string());
    }

    Some(format!(
        "### Compressed Feedback (iter {prior_iter})\n\n\
         Fix ONLY these {} unresolved blocking issues:\n\n{}\n",
        all_issues.len(),
        all_issues.join("\n")
    ))
}

/// Extract the ## Verification section from plan content.
/// Returns the section text if found, or a fallback message.
fn extract_verification_section(plan_content: &str) -> String {
    if let Some(start) = plan_content.find("## Verification") {
        let rest = &plan_content[start..];
        let end = rest[1..].find("\n## ").map(|i| i + 1).unwrap_or(rest.len());
        rest[..end].to_string()
    } else {
        "No verification section in this plan. Write tests based on formulas \
         and constraints in Quick Reference and Implementation Details."
            .to_string()
    }
}

/// Build the implementer prompt for no-review mode
pub fn implementer_prompt(repo_root: &Path, plan: &PlanInfo) -> Result<String> {
    let workspace_map = truncate(&context::read_workspace_map(repo_root)?, 8000);
    let plan_content = super::plan::read_plan(plan)?;
    let ctx = truncate(&context::read_context(repo_root)?, 4000);
    let ignored_tests = truncate(&context::read_ignored_tests(repo_root)?, 2000);
    let prd2_extract = truncate(&context::read_prd2_extract(repo_root, &plan.num)?, 8000);

    let noreview_plans_dir = super::paths::plans_root(repo_root);
    let last_completed_path = super::paths::global_artifact(&noreview_plans_dir, "last-completed.md");
    let last_completed = if last_completed_path.exists() {
        truncate(
            &std::fs::read_to_string(&last_completed_path).unwrap_or_default(),
            3000,
        )
    } else {
        String::new()
    };

    let decomp = optional_context_file_section(
        repo_root,
        &format!("plans/context/decompositions/{}-decomposition.md", plan.num),
        "Decomposition (optional)",
        "decomposition",
        12000,
    );
    let preflight = optional_context_file_section(
        repo_root,
        "plans/context/preflight-snapshot.md",
        "Preflight snapshot (optional)",
        "preflight",
        5000,
    );
    let verify_tasks = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{}-verify-tasks.toml", plan.num),
        "Verify task checklist (optional)",
        "verify-tasks",
        8000,
    );
    let verify_chain = optional_verify_chain_section(repo_root, &plan.num);

    let plan_num = &plan.num;
    let prompt = format!(
        r#"Implement the plan at `plans/{base}.md`.

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>
{decomp}{preflight}{verify_tasks}{verify_chain}
## Plan

<plan>
{plan_content}
</plan>

## Cross-Plan Context

<context>
{ctx}
</context>

## Previous Plan Summaries

<last-completed>
{last_completed}
</last-completed>

## Ignored Tests Ledger

<ignored-tests>
{ignored_tests}
</ignored-tests>

## PRD2 Specification Context

The following is an extract of the PRD2 spec sections relevant to this plan.
PRD2 is authoritative for business logic, formulas, thresholds, and domain semantics. For Rust type signatures, field names, and struct shapes, the plan's Quick Reference takes precedence over PRD2. See the Authority Chain in AGENTS.md.
For full documents, read the files in `prd2/` directly.

<prd2-context>
{prd2_extract}
</prd2-context>

## What Reviewers Will Check

Your code will be reviewed against these criteria. Satisfy them all before finishing.

### Architect (code quality)
- `cargo check --workspace` passes with zero errors
- No `unwrap()` calls in library crates (`crates/` directory) — use `?`, `ok_or()`, `map_err()`
- Every new `pub` type, function, and field in a library crate has a doc comment
- No hardcoded absolute paths (`/Users/`, `/home/`) in any committed file
- No upward dependencies — `golem-core` must have zero workspace-internal deps
- All tests from the Verification section of the plan pass

### Auditor (spec fidelity)
- Every symbol listed in the plan's **Exports** section exists with the exact visibility stated
- All formula constants match prd2 values exactly — no rounding, no approximation
- All INV-NNN invariant tests listed in the plan exist and pass
- All behavioral rules (states, transitions, lifecycle events) are implemented

## Instructions

1. Read the plan carefully. Implement each unit of work in sequence.
2. For each unit: implement the code, write tests, create/update mdbook documentation.
3. Verify exports, doc comments, and unwrap() usage.

## End-to-End Test Harness

The `bardo-test-harness` crate (`tests/harness/`) lets you spawn real subsystems and test against them:

- **TerminalProbe**: `cargo run -p bardo-terminal -- --headless --rpc-port PORT` exposes JSON-RPC methods `terminal.health`, `terminal.snapshot`, `terminal.action`, `terminal.shutdown`
- **TestGolemInstance**: wraps `GolemConfig` with tick-based lifecycle simulation
- **MirageInstance** (feature-gated): wraps `mirage_rs::spawn_mirage_test_instance()` for EVM fork testing

When your plan touches golem lifecycle, terminal rendering, or chain/trading logic, write integration tests in `tests/harness/tests/` that use `BardoTestHarness::spawn()` to start the relevant components, assert behavior, and tear down cleanly. Example:

```rust
use bardo_test_harness::{{BardoTestHarness, HarnessConfig}};

#[tokio::test]
async fn test_my_feature() {{
    let harness = BardoTestHarness::spawn(HarnessConfig {{
        enable_terminal: true,
        enable_golem: true,
        ..Default::default()
    }}).await.unwrap();
    let health = harness.health_check().await;
    assert!(health.terminal_responding);
    harness.teardown().await.unwrap();
}}
```

## Self-Validation (REQUIRED before signaling done)

1. Run `cargo check -p <your-crate>` to verify compilation
2. If errors: fix them and re-check (max 3 attempts, then document and move on)
3. Run `cargo test -p <your-crate>` for crates you modified
4. If test failures in YOUR code: fix them
5. Only then signal completion

External gates will verify workspace-wide compilation after you finish.
Your self-check catches most issues while you still have full context.

## Before You Finish

1. Grep `crates/` for `\.unwrap()` — fix any you added
2. Check every symbol in the plan's Exports section exists in the code
3. Verify every new `pub` type and function has a doc comment
4. Re-read the **What Reviewers Will Check** section above and confirm each item
(Note: gates verify compilation, testing, and verify-chain scripts after you finish — do not run them yourself)

Only then write your completion report.
5. Write a completion report to plans/context/completion/{plan_num}-completion.md with:
   - Types Defined (with crate::module::path)
   - Deviations from plan (if any)
   - Test Results (pass/fail counts)
6. Write a summary to plans/context/completion/{plan_num}-summary.md.
7. **Self-Check** — Before finishing, verify and write results to `plans/context/completion/{plan_num}-selfcheck.toml`:
   ```toml
   [selfcheck]
   exports_verified = true     # false if any planned export is missing
   docs_verified = true        # false if any planned doc comment is missing
   unwrap_free = true          # false if you added any unwrap() in library crates
   errors = []                 # list of error strings if any check failed
   ```
   Run these checks:
   a. For each Export in the plan: `rg "pub (fn|struct|enum|trait) {{name}}"` confirms existence
   b. Grep for `unwrap()` in `crates/` — fix any you added
   c. Verify every new `pub` type has a doc comment
   d. Compilation, testing, and verify-chain scripts will be verified by gates after you finish

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask the user any questions. Do NOT say "if you want" or "shall I proceed". Complete all work and end your turn when done. If unsure about something, make the best decision and document your reasoning.
"#,
        base = plan.base
    );

    Ok(prompt)
}

/// Build the implementer prompt for review mode (with strategist brief + task checklist)
pub fn implementer_prompt_with_brief(
    repo_root: &Path,
    plan: &PlanInfo,
    iteration: u32,
) -> Result<String> {
    let workspace_map = {
        let crates = plan
            .frontmatter
            .as_ref()
            .map(|f| f.crates_touched.clone())
            .unwrap_or_default();
        if crates.is_empty() {
            truncate(&context::read_workspace_map(repo_root)?, 6000)
        } else {
            truncate(
                &context::generate_filtered_workspace_map(repo_root, &crates)?,
                6000,
            )
        }
    };
    let plan_content = super::plan::read_plan(plan)?;
    let verification_section = extract_verification_section(&plan_content);
    let ctx = truncate(&context::read_context(repo_root)?, 4000);
    let ignored_tests = truncate(&context::read_ignored_tests(repo_root)?, 2000);
    let prd2_extract = truncate(&context::read_prd2_extract(repo_root, &plan.num)?, 6000);

    let impl_plans_dir = super::paths::plans_root(repo_root);
    let last_completed_path = super::paths::global_artifact(&impl_plans_dir, "last-completed.md");
    let last_completed = if last_completed_path.exists() {
        truncate(
            &std::fs::read_to_string(&last_completed_path).unwrap_or_default(),
            3000,
        )
    } else {
        String::new()
    };

    // Read the strategist brief
    let brief_path = super::paths::plan_artifact_by_num(
        &impl_plans_dir,
        &plan.num,
        "brief.md",
        "briefs",
        &format!("{}-brief.md", plan.num),
    );
    let brief = std::fs::read_to_string(&brief_path).unwrap_or_default();

    // Read task checklist if it exists
    let tasks_path = super::paths::plan_artifact_by_num(
        &impl_plans_dir,
        &plan.num,
        "tasks.toml",
        "tasks",
        &format!("{}-tasks.toml", plan.num),
    );
    let tasks_section = if tasks_path.exists() {
        let tasks_content = std::fs::read_to_string(&tasks_path).unwrap_or_default();
        // Show the path relative to repo root for the agent prompt
        let tasks_rel = tasks_path
            .strip_prefix(repo_root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| format!("plans/context/tasks/{}-tasks.toml", plan.num));
        format!(
            "\n## Task Checklist\n\n<tasks>\n{tasks_content}\n</tasks>\n\nUpdate task status in `{tasks_rel}` as you work:\n- Set status to \"active\" when starting a task\n- Set status to \"done\" when complete\n- Update meta.done count\n",
        )
    } else {
        String::new()
    };

    let iteration_note = if iteration > 1 {
        let prior_iter = iteration - 1;

        // Try compressed feedback: parse structured TOML reviews, filter to
        // unresolved blocking issues, produce a tight fix directive (~500-1k
        // chars instead of ~6k of raw markdown).
        let compressed = compress_feedback(repo_root, &plan.num, prior_iter);

        let (review_context, gate_section) = if let Some(ref compressed_text) = compressed {
            (compressed_text.clone(), String::new())
        } else {
            // Fallback: raw review excerpts (reviews without TOML blocks)
            let mut raw_ctx = String::new();
            for suffix in &["arch", "audit"] {
                let archive_path = repo_root.join(format!(
                    "plans/context/archive/{}/iter-{}/{}-{}.md",
                    plan.num, prior_iter, plan.num, suffix
                ));
                if let Ok(content) = std::fs::read_to_string(&archive_path) {
                    let trimmed = truncate(&content, 3000);
                    raw_ctx.push_str(&format!(
                        "\n### {} Review (iter {})\n{}\n",
                        suffix, prior_iter, trimmed
                    ));
                }
            }
            let gate_output_path = super::paths::global_artifact(&impl_plans_dir, "last-gate-output.txt");
            let gate_output = std::fs::read_to_string(&gate_output_path).unwrap_or_default();
            let gate_sec = if !gate_output.is_empty() {
                format!("```\n{}\n```\n", truncate_tail(&gate_output, 2000))
            } else {
                String::new()
            };
            (raw_ctx, gate_sec)
        };

        if !gate_section.is_empty() {
            format!(
                r#"
## STOP — COMPILATION FAILED (Iteration {iteration})

### Compile Errors (fix these first — no acceptance criteria count until this passes)

{gate_section}

### Reviewer Issues to Fix After Compilation

{review_context}

After fixing each item: re-read the relevant code section and confirm the fix is complete. Then move to the next item.
"#
            )
        } else {
            format!(
                r#"
## Iteration {iteration} — REVISION REQUIRED

You MUST resolve every item below before finishing. Fixing some but not all will result in another rejection cycle — the reviewer will see the unresolved items and REVISE again.

**DO NOT re-implement from scratch.** Read the existing code, identify what's wrong, and fix only those issues.

Treat this as a mandatory checklist. Do not write your completion report until every item is checked off:

{review_context}

After fixing each item: re-read the relevant code section and confirm the fix is complete. Then move to the next item.
"#
            )
        }
    } else {
        String::new()
    };

    // Skill injection: auto-detect terminal work from plan content
    let mut impl_skills: Vec<String> = Vec::new();
    if plan_content.contains("bardo-terminal") || plan_content.contains("terminal") {
        impl_skills.push(skills::RATATUI_CINEMATIC.to_string());
    }
    let skill_section = skills::build_skill_section(repo_root, &impl_skills, 24000);

    let decomp = optional_context_file_section(
        repo_root,
        &format!("plans/context/decompositions/{}-decomposition.md", plan.num),
        "Decomposition (optional)",
        "decomposition",
        12000,
    );
    let preflight = optional_context_file_section(
        repo_root,
        "plans/context/preflight-snapshot.md",
        "Preflight snapshot (optional)",
        "preflight",
        5000,
    );
    let verify_tasks = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{}-verify-tasks.toml", plan.num),
        "Verify task checklist (optional)",
        "verify-tasks",
        8000,
    );
    let verify_chain = optional_verify_chain_section(repo_root, &plan.num);
    let agent_messages = optional_context_file_section(
        repo_root,
        "tmp/agent-messages.md",
        "Conductor Steering (READ FIRST if present)",
        "agent-messages",
        5000,
    );

    let shared_rubric = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{}-shared-rubric.md", plan.num),
        "Shared Review Rubric (canonical checklist for all agents)",
        "shared-rubric",
        3000,
    );

    // Cross-plan context: recent API changes from sibling plans
    let crates_touched = plan
        .frontmatter
        .as_ref()
        .map(|f| f.crates_touched.clone())
        .unwrap_or_default();
    let cross_plan_section = cross_plan_diff_section(repo_root, &crates_touched);

    let prompt = format!(
        r#"Implement the plan at `plans/{base}.md`.
{agent_messages}{shared_rubric}{cross_plan_section}
## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>
{decomp}{preflight}{verify_tasks}{verify_chain}
## Plan

<plan>
{plan_content}
</plan>

## Verification Requirements

<verification>
{verification_section}
</verification>

YOU MUST write tests for every INV-NNN invariant listed above:
- strategy=proptest: proptest! block with listed input ranges
- strategy=unit: #[test] fn with listed test_fn name
- strategy=integration: #[test] fn wiring relevant crates

Every test_fn in Regression Anchors MUST exist and pass.
Do NOT weaken invariant assertions. Fix the implementation, not the test.
If a spec invariant seems wrong, mark with #[ignore] and comment:
// SPEC_ISSUE: {{description of why the spec seems wrong}}

## Strategist Brief

<brief>
{brief}
</brief>
{tasks_section}
## Cross-Plan Context

<context>
{ctx}
</context>

## Previous Plan Summary

<last-completed>
{last_completed}
</last-completed>

## Ignored Tests Ledger

<ignored-tests>
{ignored_tests}
</ignored-tests>
{iteration_note}
## PRD2 Specification Context

The following is an extract of the PRD2 spec sections relevant to this plan.
PRD2 is authoritative for business logic, formulas, thresholds, and domain semantics. For Rust type signatures, field names, and struct shapes, the plan's Quick Reference takes precedence over PRD2. See the Authority Chain in AGENTS.md.
For full documents, read the files in `prd2/` directly.

<prd2-context>
{prd2_extract}
</prd2-context>
{skill_section}
## What Reviewers Will Check

Your code will be reviewed against these criteria. Satisfy them all before finishing.

### Architect (code quality)
- `cargo check --workspace` passes with zero errors
- No `unwrap()` calls in library crates (`crates/` directory) — use `?`, `ok_or()`, `map_err()`
- Every new `pub` type, function, and field in a library crate has a doc comment
- No hardcoded absolute paths (`/Users/`, `/home/`) in any committed file
- No upward dependencies — `golem-core` must have zero workspace-internal deps
- All tests from the Verification section of the plan pass

### Auditor (spec fidelity)
- Every symbol listed in the plan's **Exports** section exists with the exact visibility stated
- All formula constants match prd2 values exactly — no rounding, no approximation
- All INV-NNN invariant tests listed in the plan exist and pass
- All behavioral rules (states, transitions, lifecycle events) are implemented
- Verify-chain scripts are run by gates after you finish (do not run them yourself)

## End-to-End Test Harness

The `bardo-test-harness` crate (`tests/harness/`) lets you spawn real subsystems and test against them:

- **TerminalProbe**: `cargo run -p bardo-terminal -- --headless --rpc-port PORT` exposes JSON-RPC methods `terminal.health`, `terminal.snapshot`, `terminal.action`, `terminal.shutdown`
- **TestGolemInstance**: wraps `GolemConfig` with tick-based lifecycle simulation
- **MirageInstance** (feature-gated): wraps `mirage_rs::spawn_mirage_test_instance()` for EVM fork testing

When your plan touches golem lifecycle, terminal rendering, or chain/trading logic, write integration tests in `tests/harness/tests/` that use `BardoTestHarness::spawn()` to start the relevant components, assert behavior, and tear down cleanly. Example:

```rust
use bardo_test_harness::{{BardoTestHarness, HarnessConfig}};

#[tokio::test]
async fn test_my_feature() {{
    let harness = BardoTestHarness::spawn(HarnessConfig {{
        enable_terminal: true,
        enable_golem: true,
        ..Default::default()
    }}).await.unwrap();
    let health = harness.health_check().await;
    assert!(health.terminal_responding);
    harness.teardown().await.unwrap();
}}
```

## Instructions

**Check first:** If `tmp/agent-messages.md` exists and is non-empty, read it before everything else. It contains conductor/supervisor steering that supersedes other instructions.

**Before you start:** The reviewer checks these things. Pass all of them before finishing — any failure triggers a full review iteration:
(a) Every plan Export exists in source (rg confirms)
(b) Every Cargo.toml entry in "Cargo Dependencies" matches exactly — optional flags, features, workspace = true
(c) Every config file (.cargo/config.toml, justfile, rustfmt.toml) matches plan spec verbatim
(d) Every Quick Reference struct/fn/enum field matches plan QR exactly — field names, types, signatures
(e) Every INV-NNN has a test_fn that exists
(f) Every Gitbook doc page in the plan exists in docs/src/
## Self-Validation (REQUIRED before signaling done)

1. Run `cargo check -p <your-crate>` to verify compilation
2. If errors: fix them and re-check (max 3 attempts, then document and move on)
3. Run `cargo test -p <your-crate>` for crates you modified
4. If test failures in YOUR code: fix them
5. Only then signal completion

External gates will verify workspace-wide compilation after you finish.
Your self-check catches most issues while you still have full context.

**Instructions:**

1. Read the plan and the strategist brief carefully. Follow the execution order from the brief.
2. For each unit: implement the code, write tests, AND write the corresponding mdbook docs immediately.
3. Use `rg` to verify type signatures and exports exist.

## Documentation (write during implementation, not after)

For each unit you implement, write the corresponding mdbook page immediately.
- Read the FULL prd2 source files (not the truncated extract above) for complete context
- Create/update docs under `docs/src/` per the plan's Documentation section
- Document what you ACTUALLY built, not what the plan assumed
- Preserve FULL implementation details, context, and rationale from prd2 — do NOT truncate or simplify
- Preserve ALL academic and research context: citations, paper references, formulas with their origins
- Maintain all citations verbatim (author, year, what they say, how code implements it)
- Docs should be nearly as long as the original prd2 documents
- Include code examples for public APIs
- This will use significant context — that is expected and desired

## Self-Review (check before signaling done)

Before ending your turn, verify:
- No `unwrap()` in library crates (use `?`, `ok_or()`, `map_err()`)
- Every new `pub` type, function, and field has a doc comment
- Every symbol in the plan's Exports section exists with correct visibility
- All formula constants match PRD2 values exactly
- mdbook docs exist for every page listed in the plan's Documentation section
5. Write a completion report to plans/context/completion/{num}-completion.md with:
   - Types Defined (with crate::module::path)
   - Deviations from plan (if any)
   - Test Results (pass/fail counts)
6. Write a summary to plans/context/completion/{num}-summary.md.

6a. **Append to `plans/CONTEXT.md`** (mandatory — the cross-plan registry goes stale without this):
    ```
    ## Plan {{num}}: [{{name}}] — Completed [DATE]
    ### Types Defined
    - `TypeName` — `crate::module::path` — one-line description
    ### Deviations
    - [What changed vs. plan, and why]
    ### Status
    - cargo check: pass/fail | cargo test: N pass, N fail, N ignored
    ```

6b. **Overwrite `plans/context/last-completed.md`** with a typed summary using the AGENTS.md template (see AGENTS.md Phase 2a).

6c. **Update `plans/context/ignored-tests.md`** — add new `#[ignore]` tests you added; remove any entries for tests you un-ignored.

7. If a dependency from a prior plan is missing, add a stub with `todo!()` and document it — don't block on it.
8. **Self-Check** — Before finishing, verify and write results to `plans/context/completion/{num}-selfcheck.toml`:
   ```toml
   [selfcheck]
   exports_verified = true  # false if any planned export is missing
   doc_pages_exist = true   # false if any planned doc page is missing
   errors = []          # list of error strings if any check failed
   ```
   Run these checks (code review only, no cargo):
   a. For each Export in the plan: `rg "pub (fn|struct|enum|trait) {{name}}"` confirms existence
   b. For each doc page in plan: verify file exists in docs/src/
   c. For each entry in plan's "Cargo Dependencies" section, read the actual Cargo.toml and verify it matches
   d. For each struct/enum/fn/const in the plan's Quick Reference, `rg` to verify field names and signatures match
If `<skill>` tags are present above, follow the guidance they contain.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask the user any questions. Do NOT say "if you want" or "shall I proceed". Complete all work and end your turn when done. If unsure about something, make the best decision and document your reasoning. If you need implementation details, use web search to look them up.
"#,
        agent_messages = agent_messages,
        shared_rubric = shared_rubric,
        base = plan.base,
        num = plan.num,
    );

    Ok(prompt)
}

/// Build a lightweight "fix-only" prompt for iteration 2+
/// Strips most context to keep the agent focused on specific errors/issues
pub fn implementer_fix_prompt(repo_root: &Path, plan: &PlanInfo, iteration: u32) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;
    let plan_num = &plan.num;

    // Build the iteration note (errors + review issues)
    let prior_iter = iteration - 1;
    let compressed = compress_feedback(repo_root, &plan_num, prior_iter);

    let (review_context, gate_section) = if let Some(ref compressed_text) = compressed {
        (compressed_text.clone(), String::new())
    } else {
        let mut raw_ctx = String::new();
        for suffix in &["arch", "audit"] {
            let archive_path = repo_root.join(format!(
                "plans/context/archive/{}/iter-{}/{}-{}.md",
                plan_num, prior_iter, plan_num, suffix
            ));
            if let Ok(content) = std::fs::read_to_string(&archive_path) {
                let trimmed = truncate(&content, 2000);
                raw_ctx.push_str(&format!(
                    "\n### {} Review (iter {})\n{}\n",
                    suffix, prior_iter, trimmed
                ));
            }
        }
        let fix_plans_dir = super::paths::plans_root(repo_root);
        let gate_output_path = super::paths::global_artifact(&fix_plans_dir, "last-gate-output.txt");
        let gate_output = std::fs::read_to_string(&gate_output_path).unwrap_or_default();
        let gate_sec = if !gate_output.is_empty() {
            format!("```\n{}\n```\n", truncate_tail(&gate_output, 3000))
        } else {
            String::new()
        };
        (raw_ctx, gate_sec)
    };

    let issues_section = if !gate_section.is_empty() {
        format!(
            r#"## STOP — COMPILATION FAILED (Iteration {iteration})

### Compile Errors (fix these first — no acceptance criteria count until this passes)

{gate_section}

### Reviewer Issues to Fix After Compilation

{review_context}

After fixing each item: re-read the relevant code section and confirm the fix is complete. Then move to the next item."#
        )
    } else {
        format!(
            r#"## Iteration {iteration} — REVISION REQUIRED (Fix Mode)

You MUST resolve every item below before finishing. Fixing some but not all will result in another rejection cycle.

**DO NOT re-implement from scratch.** Read the existing code, identify what's wrong, and fix only those issues.

{review_context}

After fixing each item: re-read the relevant code section and confirm the fix is complete. Then move to the next item."#
        )
    };

    // Optional: read failing acceptance criteria
    let verify_tasks = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{}-verify-tasks.toml", plan_num),
        "Failing Acceptance Criteria (if any)",
        "verify-tasks",
        4000,
    );

    let shared_rubric = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{}-shared-rubric.md", plan_num),
        "Shared Review Rubric (reference only)",
        "shared-rubric",
        2000,
    );

    // Cross-plan context: recent API changes from sibling plans
    let crates_touched = plan
        .frontmatter
        .as_ref()
        .map(|f| f.crates_touched.clone())
        .unwrap_or_default();
    let cross_plan_section = cross_plan_diff_section(repo_root, &crates_touched);

    let prompt = format!(
        r#"You are the Implementer. This is iteration {iteration} — you are fixing specific issues from the prior compilation or review cycle.

{cross_plan_section}{shared_rubric}

## Your Fix Task

{issues_section}

{verify_tasks}

## Instructions

1. Read the issues above carefully. These are the only things you need to fix.
2. Open the relevant files in your editor. Make minimal, surgical changes.
3. After each fix, re-read the relevant code to confirm it's correct.
4. Run `cargo check -p <affected-crate>` to verify your fix compiles (max 3 attempts).
5. Move to the next issue.

Complete all work and end your turn when done. There is no human waiting — fix it yourself.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask questions. Do NOT say "if you want". Complete all work and end your turn when done."#,
        iteration = iteration,
        issues_section = issues_section,
        shared_rubric = shared_rubric,
        verify_tasks = verify_tasks,
    );

    Ok(prompt)
}

/// Build the strategist prompt
pub fn strategist_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    iteration: u32,
    prior_reviews: &[String],
) -> Result<String> {
    let workspace_map = truncate(&context::read_workspace_map(repo_root)?, 10000);
    let plan_content = super::plan::read_plan(plan)?;
    let ctx = truncate(&context::read_context(repo_root)?, 4000);
    let prd2_extract = truncate(&context::read_prd2_extract(repo_root, &plan.num)?, 8000);

    let review_section = if iteration > 1 {
        let prior_iter = iteration - 1;
        if let Some(compressed) = compress_feedback(repo_root, &plan.num, prior_iter) {
            format!("\n## Prior Review — Blocking Issues to Address\n\n{compressed}\n")
        } else if !prior_reviews.is_empty() {
            // Fallback: raw review excerpts (reviews without TOML blocks)
            let reviews: Vec<String> = prior_reviews.iter().map(|r| truncate(r, 3000)).collect();
            let joined = reviews.join("\n\n---\n\n");
            format!(
                "\n## Prior Reviews (Iteration {iteration})\n\n<reviews>\n{joined}\n</reviews>\n"
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let remediation = if iteration > 1 {
        "\n6. **Remediation Plan**: For each `[B-N]` issue in the Prior Review section above, provide specific fix instructions. Reference the exact ID (e.g., `[B-1]`) so the implementer can cross-reference directly.\n"
    } else {
        ""
    };

    let strategist_extra = optional_context_file_section(
        repo_root,
        &format!("plans/context/decompositions/{}-decomposition.md", plan.num),
        "Existing decomposition (optional)",
        "decomposition",
        12000,
    ) + &optional_context_file_section(
        repo_root,
        "plans/context/preflight-snapshot.md",
        "Preflight snapshot (optional)",
        "preflight",
        5000,
    ) + &optional_context_file_section(
        repo_root,
        "plans/context/ignored-tests.md",
        "Ignored tests ledger",
        "ignored-tests",
        3000,
    );

    let prompt = format!(
        r#"You are the Strategist. Your job is to analyze the plan and produce a brief + structured task checklist.

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## Plan

<plan>
{plan_content}
</plan>

## Cross-Plan Context

<context>
{ctx}
</context>
{review_section}
## PRD2 Specification Context

The following is an extract of the PRD2 spec sections relevant to this plan.
PRD2 is authoritative for business logic, formulas, thresholds, and domain semantics. For Rust type signatures, field names, and struct shapes, the plan's Quick Reference takes precedence over PRD2. See the Authority Chain in AGENTS.md.
For full documents, read the files in `prd2/` directly.

<prd2-context>
{prd2_extract}
</prd2-context>
{strategist_extra}
## Instructions

Before writing the brief, read the PRD2 files listed in <prd2-context> above.
Use them to verify that your task breakdown covers all spec requirements.
Flag any plan requirements that don't match the PRD2 spec.
If a `<decomposition>` section is present above, align your execution order and task breakdown with those steps (do not contradict them without calling out why).

Write a brief to `plans/context/briefs/{num}-brief.md` with these sections:

1. **Dependency Verification**: Check that all imported types/traits from prior plans exist in the codebase.
2. **Conflict Scan**: Identify potential conflicts with existing code.
3. **Execution Order**: Optimal sequence for implementing the plan's units.
4. **Pattern Alignment**: Ensure the plan follows patterns established in prior plans.
5. **Risk Flags**: Anything that might cause compilation or test failures.
6. **Verification Completeness**: Check that ## Verification exists and covers every formula in Quick Reference, every state machine enum, and every boundary condition. If missing or incomplete, flag: "VERIFICATION_INCOMPLETE: {{what's missing}}".
{remediation}
The brief must have ALL sections. Be concrete — reference specific files, types, and line numbers.

## Task Checklist

After writing the brief, write a TOML task checklist to `plans/context/tasks/{num}-tasks.toml`:

**IMPORTANT — Preserve enriched TOML:** If `plans/context/tasks/{num}-tasks.toml` already exists, **read it first**. Preserve any `acceptance` arrays, `test_invariants`, and `parallel_group` assignments that already exist — they may have been enriched by `bardo-enrich.sh`. Only overwrite tasks that are incorrect or missing for this iteration's needs. Do not replace an enriched TOML with a skeletal one.

```toml
[meta]
plan = "{base}"
iteration = {iteration}
total = <number of tasks>
done = 0

[[task]]
id = "T1"
title = "First task title"
status = "pending"
files = ["path/to/file.rs"]
acceptance = ["Acceptance criterion 1"]
depends_on = []
parallel_group = "A"
exclusive_files = true
```

Each task should be a discrete unit of work from the plan. Include file paths and acceptance criteria.

### Parallel Group Rules

- `depends_on`: list of task IDs (e.g. ["T1"]) that must complete before this task starts.
- `parallel_group`: tasks sharing a group value (e.g. "A") can run simultaneously.
- `exclusive_files`: when true (the default), no other concurrent task should touch this task's files. Set to false only for shared config or generated files.
- Group "A" has no dependencies. Group "B" depends on "A". Group "C" depends on "B". Etc.
- Tasks in the same parallel group MUST NOT touch the same files when `exclusive_files` is true.
"#,
        num = plan.num,
        base = plan.base,
        iteration = iteration,
    );

    Ok(prompt)
}

/// Build the architect reviewer prompt
pub fn architect_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    iteration: u32,
    worktree: Option<&Path>,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;

    let map_root = worktree.unwrap_or(repo_root);
    let workspace_map = {
        let crates = plan
            .frontmatter
            .as_ref()
            .map(|f| f.crates_touched.clone())
            .unwrap_or_default();
        if crates.is_empty() {
            truncate(&context::read_workspace_map(map_root)?, 6000)
        } else {
            truncate(
                &context::generate_filtered_workspace_map(map_root, &crates)?,
                6000,
            )
        }
    };

    // Read brief and PRD2 from worktree (map_root) so agents see modifications
    let arch_plans_dir = super::paths::plans_root(map_root);
    let brief_path = super::paths::plan_artifact_by_num(
        &arch_plans_dir,
        &plan.num,
        "brief.md",
        "briefs",
        &format!("{}-brief.md", plan.num),
    );
    let brief = truncate(
        &std::fs::read_to_string(&brief_path).unwrap_or_default(),
        4000,
    );
    let prd2_extract = truncate(&context::read_prd2_extract(map_root, &plan.num)?, 4000);

    // Prior reviews live in repo_root archive (copied there after each review)
    let prior_review = if iteration > 1 {
        let prior_path = repo_root.join(format!(
            "plans/context/archive/{}/iter-{}/{}",
            plan.num,
            iteration - 1,
            format!("{}-arch.md", plan.num)
        ));
        truncate(
            &std::fs::read_to_string(&prior_path).unwrap_or_default(),
            3000,
        )
    } else {
        String::new()
    };

    let prior_section = if !prior_review.is_empty() {
        format!("\n## Prior Review\n\n<prior-review>\n{prior_review}\n</prior-review>\n\nDo NOT re-raise issues that have been fixed.\n")
    } else {
        String::new()
    };

    // Load review task TOML if it exists
    let review_tasks_path = super::paths::plan_artifact_by_num(
        &arch_plans_dir,
        &plan.num,
        "review-tasks.toml",
        "tasks",
        &format!("{}-review-tasks.toml", plan.num),
    );
    let review_tasks_section = if review_tasks_path.exists() {
        let content = std::fs::read_to_string(&review_tasks_path).unwrap_or_default();
        format!(
            "\n## Review Task Checklist\n\n\
             Work through these review tasks systematically. Mark each with your verdict.\n\n\
             <review-tasks>\n{content}\n</review-tasks>\n"
        )
    } else {
        String::new()
    };

    let shared_rubric_architect = optional_context_file_section(
        map_root,
        &format!("plans/context/tasks/{}-shared-rubric.md", plan.num),
        "Shared Review Rubric (only raise blocking issues on this list)",
        "shared-rubric",
        3000,
    );

    let reviewer_context_extra = shared_rubric_architect
        + &optional_context_file_section(
            map_root,
            &format!("plans/context/decompositions/{}-decomposition.md", plan.num),
            "Decomposition (optional)",
            "decomposition",
            8000,
        )
        + &optional_context_file_section(
            map_root,
            "plans/context/preflight-snapshot.md",
            "Preflight snapshot (optional)",
            "preflight",
            4000,
        )
        + &optional_context_file_section(
            map_root,
            &format!("plans/context/tasks/{}-verify-tasks.toml", plan.num),
            "Verify task checklist (optional)",
            "verify-tasks",
            6000,
        )
        + &optional_verify_chain_section(map_root, &plan.num)
        + &optional_context_file_section(
            repo_root,
            &format!("plans/context/completion/{}-completion.md", plan.num),
            "Implementer's completion report",
            "completion-summary",
            2000,
        )
        + &optional_context_file_section(
            repo_root,
            "plans/context/last-gate-output.txt",
            "Last gate output (compile + test results)",
            "gate-output",
            3000,
        );

    let prompt = format!(
        r#"You are the Architect. Review the implementation for code quality.

## Plan

<plan>
{plan_content}
</plan>

## Brief

<brief>
{brief}
</brief>
{prior_section}{review_tasks_section}
## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## PRD2 Specification Context

<prd2-context>
{prd2_extract}
</prd2-context>
{reviewer_context_extra}
## Instructions

1. Run `cargo clippy --workspace` and review warnings.
2. **Test results** — the compile and test gates have already run. Results are in `<gate-output>` above. **Do not re-run `cargo test`** — use those results. You may run `cargo check` to navigate code.
3. Review the diff of modified files for correctness.
4. Check: Rust patterns, module structure, error handling, test coverage.
5. Check that all files, functions, types, and exports described in the plan actually exist in the source tree.
6. If `<completion-summary>` is present above and the implementer acknowledged a deviation and explained why it is acceptable, weigh that context before issuing REVISE on that item.
7. **Verify chain** — If `plans/context/verify-chains/{num}-verify.sh` exists, you may run `bash` on it. If it is **missing**, note that the verify-chain script was not generated; that is **not** by itself a reason to REVISE. If the script exists but fails, distinguish missing binaries/tests not yet written from genuine implementation bugs.
8. Before reviewing, run `git diff HEAD` (or `git diff main...HEAD` if not on main) to see exactly what changed. Focus your review on the diff — do not re-review pre-existing code.

**BE EXHAUSTIVE.** Do not stop after finding the first problem. Scan the entire implementation and compile a COMPLETE list of every issue. The implementer will fix all of them in one pass — if you only report one issue, you cause an unnecessary re-review cycle. Go through the plan section by section, file by file, and verify each requirement.

**You have exactly one pass to find all issues.** The implementer will address everything you report here in a single fix cycle. Issues you fail to mention now will not be raised in the next review — that review starts fresh and must also be exhaustive. If you are aware of 3 problems, report all 3 now, not 1 now and 2 next time. An incomplete review that forces a second cycle to surface known issues is a failure of your role.

Write your review to `plans/context/reviews/{num}-arch.md`.

## Nits

If you notice something minor — style, naming, cosmetic, missed doc comments, trivial clippy suggestions
that don't indicate bugs — write it to `plans/context/nits/{num}-nits.toml` rather than listing it
in this review. Minor observations are NOT grounds for REVISE.

TOML format:

```toml
[[nit]]
reviewer = "architect"          # or auditor / critic / quick-reviewer
file = "crates/foo/src/lib.rs"  # relative to repo root; omit if not file-specific
line = 42                       # optional
description = "variable name `x` could be more descriptive"
category = "style"              # style | naming | docs | spec_deviation | other
```

Write as many `[[nit]]` entries as needed. If the file doesn't exist yet, create it.
These will be swept in a future cleanup pass.

## DO NOT REVISE for any of these — write them as nits instead

- clippy warnings (they don't block compilation; ignore them)
- missing doc comments on private, internal, or test functions
- code style differences from your preference (the implementer's style is valid)
- you would have structured the module differently
- naming preferences (snake_case variants, abbreviations, variable names)
- `unwrap()` usage where `?` would be "cleaner" — only flag if the call can actually panic on real input
- minor logging/tracing additions you'd have included
- the implementer didn't address a nit from a prior review

If you found an issue but aren't certain it causes a runtime failure on valid input: write it to `plans/context/nits/{{NUM}}-nits.toml`, not the review. REVISE is only for genuine functional bugs that would break the code.

End with exactly one of:
- `## Verdict: APPROVE` if implementation is correct
- `## Verdict: REVISE` if there are BLOCKING issues that prevent the code from working

**APPROVE if compile and tests pass.** Only REVISE for genuine blocking bugs — things that would cause runtime failures or data corruption. Stylistic issues, missing docs, clippy warnings, and minor code quality concerns are NOT blocking. If `cargo check` and `cargo test` both pass, the default should be APPROVE unless you find a real functional bug.

If REVISE, list ALL blocking issues as `- [B-N] description`. Each issue MUST be a concrete functional bug with file path and line reference. Every `[[issues]]` entry in the structured TOML block **must** include `fix_hint`. State the exact change in one sentence: file path, what to add/change/remove. Example: `fix_hint = "Cargo.toml line 14: add optional = true to the rodio dependency"`. An issue without a fix_hint forces the implementer to re-read the full review to understand the fix. Do NOT stop at one or two — list every issue you can find so they can all be fixed in a single iteration.
{review_toml_template}
IMPORTANT: This is a fully autonomous pipeline. Do NOT ask the user any questions. Do NOT end with a follow-up offer or question.
"#,
        num = plan.num,
        review_toml_template = super::review::REVIEW_TOML_TEMPLATE
    );

    Ok(prompt)
}

/// Build the auditor prompt (spec compliance)
pub fn auditor_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    iteration: u32,
    worktree: Option<&Path>,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;

    let map_root = worktree.unwrap_or(repo_root);
    let workspace_map = {
        let crates = plan
            .frontmatter
            .as_ref()
            .map(|f| f.crates_touched.clone())
            .unwrap_or_default();
        if crates.is_empty() {
            truncate(&context::read_workspace_map(map_root)?, 6000)
        } else {
            truncate(
                &context::generate_filtered_workspace_map(map_root, &crates)?,
                6000,
            )
        }
    };

    // Read brief and PRD2 from worktree (map_root) so agents see modifications
    let audit_plans_dir = super::paths::plans_root(map_root);
    let brief_path = super::paths::plan_artifact_by_num(
        &audit_plans_dir,
        &plan.num,
        "brief.md",
        "briefs",
        &format!("{}-brief.md", plan.num),
    );
    let brief = truncate(
        &std::fs::read_to_string(&brief_path).unwrap_or_default(),
        4000,
    );
    let prd2_extract = truncate(&context::read_prd2_extract(map_root, &plan.num)?, 6000);

    // Prior reviews live in repo_root archive (copied there after each review)
    let prior_review = if iteration > 1 {
        let prior_path = repo_root.join(format!(
            "plans/context/archive/{}/iter-{}/{}-audit.md",
            plan.num,
            iteration - 1,
            plan.num
        ));
        truncate(
            &std::fs::read_to_string(&prior_path).unwrap_or_default(),
            3000,
        )
    } else {
        String::new()
    };

    let prior_section = if !prior_review.is_empty() {
        format!("\n## Prior Review\n\n<prior-review>\n{prior_review}\n</prior-review>\n\nDo NOT re-raise issues that have been fixed.\n")
    } else {
        String::new()
    };

    // Load review task TOML if it exists (auditor shares the same review TOML)
    let review_tasks_path = super::paths::plan_artifact_by_num(
        &audit_plans_dir,
        &plan.num,
        "review-tasks.toml",
        "tasks",
        &format!("{}-review-tasks.toml", plan.num),
    );
    let review_tasks_section = if review_tasks_path.exists() {
        let content = std::fs::read_to_string(&review_tasks_path).unwrap_or_default();
        format!(
            "\n## Review Task Checklist\n\n\
             Work through these review tasks. Focus on blocking and major severity items.\n\n\
             <review-tasks>\n{content}\n</review-tasks>\n"
        )
    } else {
        String::new()
    };

    let shared_rubric_auditor = optional_context_file_section(
        map_root,
        &format!("plans/context/tasks/{}-shared-rubric.md", plan.num),
        "Shared Review Rubric (only raise blocking issues on this list)",
        "shared-rubric",
        3000,
    );

    let auditor_context_extra = shared_rubric_auditor
        + &optional_context_file_section(
            map_root,
            &format!("plans/context/decompositions/{}-decomposition.md", plan.num),
            "Decomposition (optional)",
            "decomposition",
            8000,
        )
        + &optional_context_file_section(
            map_root,
            "plans/context/preflight-snapshot.md",
            "Preflight snapshot (optional)",
            "preflight",
            4000,
        )
        + &optional_context_file_section(
            map_root,
            &format!("plans/context/tasks/{}-verify-tasks.toml", plan.num),
            "Verify task checklist (optional)",
            "verify-tasks",
            6000,
        )
        + &optional_verify_chain_section(map_root, &plan.num)
        + &optional_context_file_section(
            repo_root,
            &format!("plans/context/completion/{}-completion.md", plan.num),
            "Implementer's completion report",
            "completion-summary",
            2000,
        )
        + &optional_context_file_section(
            repo_root,
            "plans/context/last-gate-output.txt",
            "Last gate output (compile + test results)",
            "gate-output",
            3000,
        );

    let first_iteration_verify_note = if iteration <= 1 {
        format!(
            "\n**Review iteration note:** On the first pass after implementation, `cargo test` failures or a failing `plans/context/verify-chains/{}-verify.sh` often mean tests or crates are not present yet. Treat those as **implementation gaps**, not specification violations: use `[B-N]` for missing code/tests, reserve `[S-N]` for cases where the implementation clearly exists but violates the written spec or INV contract. If the verify-chain script is missing, say so — do not REVISE solely for a missing script.\n",
            plan.num
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are the Auditor. Verify the implementation matches the specification.

## Plan

<plan>
{plan_content}
</plan>
{review_tasks_section}
## Brief

<brief>
{brief}
</brief>
{prior_section}
## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## PRD2 Specification Context

<prd2-context>
{prd2_extract}
</prd2-context>
{auditor_context_extra}
{first_iteration_verify_note}
## Instructions

1. Check that all exports listed in the plan are implemented.
2. Verify type signatures match the plan's specification.
3. Check that all units of work are addressed.
4. Verify tests cover the specified verification criteria.
5. Cross-reference every file path mentioned in the plan against the actual source tree.
6. Invariant Coverage Check: For each INV-NNN in ## Verification:
   a. Verify a test matching test_fn exists in source
   b. Verify the test asserts the listed constraint (not just compiles)
   c. If test is #[ignore] with SPEC_ISSUE, log but do NOT block
   d. Missing test for any INV -> [B-N]: Missing test for INV-{{NNN}}
7. For each type and formula in the plan's Quick Reference, verify it matches
   the PRD2 source. Check field names, value ranges, enum variants, and formula
   constants. PRD2 is authoritative for business logic, formulas, thresholds, and domain semantics. For Rust type signatures and struct shapes, the plan's Quick Reference takes precedence over PRD2. If the implementer followed the Quick Reference over a conflicting PRD2, that is CORRECT behavior — do not flag it as a violation. Only flag deviations from the Quick Reference itself.
8. **Test results** — the compile and test gates have already run. Results are in `<gate-output>` above. **Do not re-run `cargo test`** — use those results.
9. If `<completion-summary>` is present above and the implementer acknowledged a deviation and explained why it is acceptable, weigh that context before issuing REVISE on that item.
10. **Verify chain script** — If `plans/context/verify-chains/{num}-verify.sh` is **missing**, state that it was not generated; do **not** file `[S-N]` solely for a missing script. If the script **exists** and fails, distinguish (a) tests/crates not built yet or missing from the workspace vs (b) real invariant violations. Only use `[S-N]` for genuine spec/test gaps once implementation should be complete. When failure output is clearly \"binary/crate not found\" or \"no test targets\", prefer `[B-N]` (implementation not landed) over `[S-N]` (spec wrong).
11. **Cargo and config file audit** — Read the plan's Cargo Dependencies section and verify the actual Cargo.toml matches. Check: optional flags, features lists, workspace = true. Read any config files specified in the plan (.cargo/config.toml, rustfmt.toml, justfile) and verify content matches the plan spec exactly. These are spec violations if the plan is explicit about their content.

**BE EXHAUSTIVE.** Do not stop after finding the first problem. Walk through the entire plan specification and verify every requirement. Compile a COMPLETE list of all issues so they can all be fixed in one pass.

**You have exactly one pass to find all issues.** The implementer will address everything you report here in a single fix cycle. Issues you fail to mention now will not be raised in the next review — that review starts fresh and must also be exhaustive. If you are aware of 3 problems, report all 3 now, not 1 now and 2 next time. An incomplete review that forces a second cycle to surface known issues is a failure of your role.

Write your review to `plans/context/reviews/{num}-audit.md`.

## Nits

If you notice something minor — style, naming, cosmetic, missed doc comments, trivial clippy suggestions
that don't indicate bugs — write it to `plans/context/nits/{num}-nits.toml` rather than listing it
in this review. Minor observations are NOT grounds for REVISE.

TOML format:

```toml
[[nit]]
reviewer = "auditor"            # or architect / critic / quick-reviewer
file = "crates/foo/src/lib.rs"  # relative to repo root; omit if not file-specific
line = 42                       # optional
description = "variable name `x` could be more descriptive"
category = "style"              # style | naming | docs | spec_deviation | other
```

Write as many `[[nit]]` entries as needed. If the file doesn't exist yet, create it.
These will be swept in a future cleanup pass.

## DO NOT REVISE for any of these — write them as nits instead

- clippy warnings (they don't block compilation)
- missing doc comments on private, internal, or test functions
- code structure differences from your preference (the implementer's approach is valid)
- naming preferences or variable names (snake_case variants, abbreviations)
- minor logging or instrumentation additions you'd have included
- the implementer didn't address a nit from a prior review
- test naming conventions or organization you'd prefer differently

If you found an issue but aren't certain it violates the specification or breaks functionality on valid input: write it to `plans/context/nits/{{NUM}}-nits.toml`, not the review. REVISE is only for genuine spec violations that cause functional problems.

End with exactly one of:
- `## Verdict: APPROVE` if specification is substantially met
- `## Verdict: REVISE` if there are BLOCKING issues that prevent the code from working

**APPROVE if compile and tests pass.** Only REVISE for genuine spec violations that cause functional problems. Minor deviations from the plan (naming differences, slightly different module structure, missing optional features) are NOT blocking. If the core functionality works and tests pass, the default should be APPROVE.

If REVISE, list ALL blocking issues as `- [B-N] description`. Each issue MUST reference a specific missing or broken requirement. Every `[[issues]]` entry in the structured TOML block **must** include `fix_hint`. State the exact change needed. Example: `fix_hint = "crates/foo/Cargo.toml line 12: add optional = true to rodio dependency"`. An issue without a fix_hint forces the implementer to re-read the full review. Do NOT stop at one or two — list every issue you can find.
{review_toml_template}
IMPORTANT: This is a fully autonomous pipeline. Do NOT ask the user any questions. Do NOT end with a follow-up offer or question.
"#,
        num = plan.num,
        review_toml_template = super::review::REVIEW_TOML_TEMPLATE
    );

    Ok(prompt)
}

/// Build the combined reviewer prompt (code quality + spec fidelity).
///
/// Replaces the separate Architect + Auditor passes for Complex plans.
/// Output goes to `plans/context/reviews/{num}-arch.md` — same path the
/// Architect used — so all downstream readers work unchanged.
pub fn combined_reviewer_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    iteration: u32,
    worktree: Option<&Path>,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;

    let map_root = worktree.unwrap_or(repo_root);
    let workspace_map = {
        let crates = plan
            .frontmatter
            .as_ref()
            .map(|f| f.crates_touched.clone())
            .unwrap_or_default();
        if crates.is_empty() {
            truncate(&context::read_workspace_map(map_root)?, 6000)
        } else {
            truncate(
                &context::generate_filtered_workspace_map(map_root, &crates)?,
                6000,
            )
        }
    };

    let combined_plans_dir = super::paths::plans_root(map_root);
    let brief_path = super::paths::plan_artifact_by_num(
        &combined_plans_dir,
        &plan.num,
        "brief.md",
        "briefs",
        &format!("{}-brief.md", plan.num),
    );
    let brief = truncate(
        &std::fs::read_to_string(&brief_path).unwrap_or_default(),
        4000,
    );
    let prd2_extract = truncate(&context::read_prd2_extract(map_root, &plan.num)?, 6000);

    let prior_review = if iteration > 1 {
        let prior_path = repo_root.join(format!(
            "plans/context/archive/{}/iter-{}/{}-arch.md",
            plan.num,
            iteration - 1,
            plan.num
        ));
        truncate(
            &std::fs::read_to_string(&prior_path).unwrap_or_default(),
            3000,
        )
    } else {
        String::new()
    };

    let prior_section = if !prior_review.is_empty() {
        format!("\n## Prior Review\n\n<prior-review>\n{prior_review}\n</prior-review>\n\nDo NOT re-raise issues that have been fixed.\n")
    } else {
        String::new()
    };

    let review_tasks_path = super::paths::plan_artifact_by_num(
        &combined_plans_dir,
        &plan.num,
        "review-tasks.toml",
        "tasks",
        &format!("{}-review-tasks.toml", plan.num),
    );
    let review_tasks_section = if review_tasks_path.exists() {
        let content = std::fs::read_to_string(&review_tasks_path).unwrap_or_default();
        format!(
            "\n## Review Task Checklist\n\n\
             Work through these review tasks systematically. Mark each with your verdict.\n\n\
             <review-tasks>\n{content}\n</review-tasks>\n"
        )
    } else {
        String::new()
    };

    let shared_rubric = optional_context_file_section(
        map_root,
        &format!("plans/context/tasks/{}-shared-rubric.md", plan.num),
        "Shared Review Rubric (only raise blocking issues on this list)",
        "shared-rubric",
        3000,
    );

    let reviewer_context_extra = shared_rubric
        + &optional_context_file_section(
            map_root,
            &format!("plans/context/decompositions/{}-decomposition.md", plan.num),
            "Decomposition (optional)",
            "decomposition",
            8000,
        )
        + &optional_context_file_section(
            map_root,
            "plans/context/preflight-snapshot.md",
            "Preflight snapshot (optional)",
            "preflight",
            4000,
        )
        + &optional_context_file_section(
            map_root,
            &format!("plans/context/tasks/{}-verify-tasks.toml", plan.num),
            "Verify task checklist (optional)",
            "verify-tasks",
            6000,
        )
        + &optional_verify_chain_section(map_root, &plan.num)
        + &optional_context_file_section(
            repo_root,
            &format!("plans/context/completion/{}-completion.md", plan.num),
            "Implementer's completion report",
            "completion-summary",
            2000,
        )
        + &optional_context_file_section(
            repo_root,
            "plans/context/last-gate-output.txt",
            "Last gate output (compile + test results)",
            "gate-output",
            3000,
        );

    let first_iteration_verify_note = if iteration <= 1 {
        format!(
            "\n**Review iteration note:** On the first pass after implementation, `cargo test` failures or a failing `plans/context/verify-chains/{}-verify.sh` often mean tests or crates are not present yet. Treat those as **implementation gaps**, not specification violations: use `[B-N]` for missing code/tests, reserve `[S-N]` for cases where the implementation clearly exists but violates the written spec or INV contract. If the verify-chain script is missing, say so — do not REVISE solely for a missing script.\n",
            plan.num
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are the Reviewer. Review the implementation for both code quality and specification fidelity.

## Plan

<plan>
{plan_content}
</plan>

## Brief

<brief>
{brief}
</brief>
{prior_section}{review_tasks_section}
## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## PRD2 Specification Context

<prd2-context>
{prd2_extract}
</prd2-context>
{reviewer_context_extra}{first_iteration_verify_note}
## Instructions

### Code Quality

1. Run `cargo clippy --workspace` and review warnings.
2. **Test results** — the compile and test gates have already run. Results are in `<gate-output>` above. **Do not re-run `cargo test`** — use those results. You may run `cargo check` to navigate code.
3. Review the diff of modified files for correctness.
4. Check: Rust patterns, module structure, error handling, test coverage.
5. Check that all files, functions, types, and exports described in the plan actually exist in the source tree.
6. Check `pub` visibility boundaries, no upward deps, doc comments on public items.
7. If `<completion-summary>` is present above and the implementer acknowledged a deviation and explained why it is acceptable, weigh that context before issuing REVISE on that item.
8. **Verify chain** — If `plans/context/verify-chains/{num}-verify.sh` exists, you may run `bash` on it. If it is **missing**, note that the verify-chain script was not generated; that is **not** by itself a reason to REVISE. If the script exists but fails, distinguish missing binaries/tests not yet written from genuine implementation bugs.
9. Before reviewing, run `git diff HEAD` (or `git diff main...HEAD` if not on main) to see exactly what changed. Focus your review on the diff — do not re-review pre-existing code.

### Spec Fidelity

10. Check that all exports listed in the plan are implemented with correct visibility and signatures.
11. Verify type signatures match the plan's specification.
12. Check that all units of work are addressed.
13. Verify tests cover the specified verification criteria.
14. Invariant Coverage Check: For each INV-NNN in ## Verification:
    a. Verify a test matching test_fn exists in source
    b. Verify the test asserts the listed constraint (not just compiles)
    c. If test is #[ignore] with SPEC_ISSUE, log but do NOT block
    d. Missing test for any INV -> [B-N]: Missing test for INV-{{NNN}}
15. For each type and formula in the plan's Quick Reference, verify it matches the PRD2 source. Check field names, value ranges, enum variants, and formula constants. PRD2 is authoritative for business logic, formulas, thresholds, and domain semantics. For Rust type signatures and struct shapes, the plan's Quick Reference takes precedence over PRD2. If the implementer followed the Quick Reference over a conflicting PRD2, that is CORRECT behavior — do not flag it as a violation. Only flag deviations from the Quick Reference itself.
16. **Cargo and config file audit** — Read the plan's Cargo Dependencies section and verify the actual Cargo.toml matches. Check: optional flags, features lists, workspace = true. Read any config files specified in the plan (.cargo/config.toml, rustfmt.toml, justfile) and verify content matches the plan spec exactly. These are spec violations if the plan is explicit about their content.

**BE EXHAUSTIVE.** Do not stop after finding the first problem. Scan the entire implementation and compile a COMPLETE list of every issue. The implementer will fix all of them in one pass — if you only report one issue, you cause an unnecessary re-review cycle. Go through the plan section by section, file by file, and verify each requirement.

**You have exactly one pass to find all issues.** The implementer will address everything you report here in a single fix cycle. Issues you fail to mention now will not be raised in the next review — that review starts fresh and must also be exhaustive. If you are aware of 3 problems, report all 3 now, not 1 now and 2 next time. An incomplete review that forces a second cycle to surface known issues is a failure of your role.

Write your review to `plans/context/reviews/{num}-arch.md`.

## Nits

If you notice something minor — style, naming, cosmetic, missed doc comments, trivial clippy suggestions
that don't indicate bugs — write it to `plans/context/nits/{num}-nits.toml` rather than listing it
in this review. Minor observations are NOT grounds for REVISE.

TOML format:

```toml
[[nit]]
reviewer = "reviewer"           # or critic / quick-reviewer
file = "crates/foo/src/lib.rs"  # relative to repo root; omit if not file-specific
line = 42                       # optional
description = "variable name `x` could be more descriptive"
category = "style"              # style | naming | docs | spec_deviation | other
```

Write as many `[[nit]]` entries as needed. If the file doesn't exist yet, create it.
These will be swept in a future cleanup pass.

## DO NOT REVISE for any of these — write them as nits instead

- clippy warnings (they don't block compilation; ignore them)
- missing doc comments on private, internal, or test functions
- code style differences from your preference (the implementer's style is valid)
- you would have structured the module differently
- naming preferences (snake_case variants, abbreviations, variable names)
- `unwrap()` usage where `?` would be "cleaner" — only flag if the call can actually panic on real input
- minor logging/tracing additions you'd have included
- the implementer didn't address a nit from a prior review
- test naming conventions or organization you'd prefer differently

If you found an issue but aren't certain it causes a runtime failure on valid input: write it to `plans/context/nits/{{NUM}}-nits.toml`, not the review. REVISE is only for genuine functional bugs or missing required implementations.

End with exactly one of:
- `## Verdict: APPROVE` if implementation is correct and specification is substantially met
- `## Verdict: REVISE` if there are BLOCKING issues that prevent the code from working

**APPROVE if compile and tests pass.** Only REVISE for genuine blocking bugs — things that would cause runtime failures, data corruption, or missing required implementations. Stylistic issues, missing docs, clippy warnings, and minor code quality concerns are NOT blocking. If `cargo check` and `cargo test` both pass, the default should be APPROVE unless you find a real functional bug or spec violation.

If REVISE, list ALL blocking issues as `- [B-N] description` (implementation bugs) or `- [S-N] description` (spec violations). Each issue MUST be a concrete bug with file path and line reference. Every `[[issues]]` entry in the structured TOML block **must** include `fix_hint`. State the exact change in one sentence: file path, what to add/change/remove. Example: `fix_hint = "Cargo.toml line 14: add optional = true to the rodio dependency"`. An issue without a fix_hint forces the implementer to re-read the full review to understand the fix. Do NOT stop at one or two — list every issue you can find so they can all be fixed in a single iteration.
{review_toml_template}
IMPORTANT: This is a fully autonomous pipeline. Do NOT ask the user any questions. Do NOT end with a follow-up offer or question.
"#,
        num = plan.num,
        review_toml_template = super::review::REVIEW_TOML_TEMPLATE
    );

    Ok(prompt)
}

/// Build the scribe prompt (documentation)
pub fn scribe_prompt(repo_root: &Path, plan: &PlanInfo, worktree: Option<&Path>) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;

    let map_root = worktree.unwrap_or(repo_root);
    let workspace_map = truncate(&context::read_workspace_map(map_root)?, 6000);
    // Read PRD2 from worktree (map_root) so agents see modifications
    let prd2_extract = truncate(&context::read_prd2_extract(map_root, &plan.num)?, 16000);

    // Load scribe-specific task TOML if it exists
    let scribe_plans_dir = super::paths::plans_root(map_root);
    let scribe_tasks_path = super::paths::plan_artifact_by_num(
        &scribe_plans_dir,
        &plan.num,
        "scribe-tasks.toml",
        "tasks",
        &format!("{}-scribe-tasks.toml", plan.num),
    );
    let scribe_tasks_section = if scribe_tasks_path.exists() {
        let content = std::fs::read_to_string(&scribe_tasks_path).unwrap_or_default();
        format!(
            "\n## Scribe Task Checklist\n\n\
             Work through these tasks in order. Citation tasks (C-type) are mandatory --\n\
             every academic reference must appear in the documentation with full attribution.\n\n\
             <scribe-tasks>\n{content}\n</scribe-tasks>\n\n\
             Update task status as you work. Mark citation tasks done only when the\n\
             reference appears in the docs with: full citation, what it says, how the\n\
             code implements it, and where implementation diverges from the original.\n"
        )
    } else {
        String::new()
    };

    // Skill injection: humanizer always for Scribe
    let scribe_skills = skills::default_skills_for_role(AgentRole::Scribe);
    let skill_section = skills::build_skill_section(repo_root, &scribe_skills, 24000);

    let prompt = format!(
        r#"You are the Scribe. Write reference documentation for the implementation.

Your reader has never seen this codebase. They don't know what the system does, why
it exists, or what decisions shaped it. Everything must be self-contained.

## Holistic Plan Narrative

Before documenting individual modules, frame the plan as a single coherent story.
Your documentation must open with a plan-level overview that answers:

- What problem does this ENTIRE plan solve end-to-end?
- How do the modules in this plan compose into a working system?
- What is the data lifecycle -- from input to storage to retrieval to output?
- What are the failure modes that span multiple modules?

This narrative goes at the TOP of your output file, before any per-module sections.
Write it as a systems engineer explaining the whole thing to a colleague. Think
research paper introduction: establish the problem, the prior art, the approach,
and the key insight -- not a table of contents listing what each section covers.

Include a `graph TD` Mermaid diagram showing how the plan's modules relate to each
other and to the broader system. This is your reader's first orientation point.

## Plan

<plan>
{plan_content}
</plan>

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>
{scribe_tasks_section}

## PRD2 Specification Context

The following is an extract of the PRD2 spec sections that define this module's
requirements. This is your PRIMARY source for context, rationale, formulas, and
citations. The extract is truncated -- read the full files in `prd2/` directly for
complete formulas, edge cases, and academic references.

<prd2-context>
{prd2_extract}
</prd2-context>

{skill_section}
{context_layout}
## Pre-Submission Checklist (Critic will verify all of these)

Before finishing, run through this list yourself. The Critic will REVISE if any are missing:

- [ ] Plan-level holistic narrative section present at the top of the document
- [ ] All 7 required sections present and non-empty
- [ ] Every public type, trait, and function in the module documented with a usage example
- [ ] Every formula has a PRD2 citation: `prd2/path/to/file.md §Section — "quote"`
- [ ] Every state machine has a `stateDiagram-v2` Mermaid diagram
- [ ] Every inter-module flow has a `sequenceDiagram` diagram
- [ ] Minimum 4 numbered, captioned Mermaid diagrams (Fig. 1, Fig. 2, etc.)
- [ ] No diagram exceeds 15 nodes — complex ones split into sub-diagrams
- [ ] Cross-module section includes at least one sequenceDiagram and one graph TD
- [ ] Academic references (Gompertz, Ebbinghaus, PAD, etc.) include: author, year, what they say, and where implementation diverges from the original
- [ ] Humanizer skill applied — check for: delve, tapestry, robust, seamless, leverage, ensures, groundbreaking, vital, pivotal, utilize, streamline, nuanced
- [ ] No "This module provides..." opening — open with the problem, not the solution
- [ ] API Reference examples compile (or are explicitly marked as pseudocode)

Check these yourself before submitting. A Critic REVISE for any of these costs the same as an Architect REVISE.

## Instructions

### Read the FULL PRD2 files — do NOT rely on the truncated extract

Before writing anything, read the FULL prd2 source files listed in the plan's Source Files
sections. The extract above is truncated. Go to the actual files in `prd2/` for complete
formulas, behavioral specs, academic references, and implementation rationale.

**CRITICAL: Docs must preserve the full depth of the PRD2 source material.**
- Do NOT truncate, summarize, or simplify the prd2 content
- Docs should be nearly as long as the original prd2 documents
- Preserve ALL academic and research context: every citation, paper reference, formula with its origin
- Maintain all citations verbatim: author, year, what they say, how the code implements it, where implementation diverges
- Preserve full implementation details, context, and design rationale from prd2
- This will use significant context — that is expected and desired

### Structure (all seven sections required)

Write to `plans/context/docs/{num}-docs.md` and update mdbook pages in `docs/src/`.

1. **Context and Motivation** -- Why does this module exist? What breaks without it?
   Pull from prd2 rationale sections. Translate spec language into engineering
   context. Do NOT open with "This module provides..." -- open with the problem.
   Connect this module's motivation to the plan-level narrative: how does solving
   this module's problem advance the plan's overall goal?

2. **Architecture and Design Decisions** -- How does this module fit into the larger
   system? What tradeoffs were made? What alternatives were rejected? Reference the
   prd2 design rationale with its reasoning intact.

3. **Core Concepts** -- Define every domain term. Include all formulas with their
   prd2 source AND any academic origin. Example:
   > Gompertz mortality: λ(t) = ae^(bt)
   > Source: prd2/mortality/model.md §2.1, originally Gompertz (1825)

4. **Public API Reference** -- Every public type, function, and trait with:
   - Full type signature
   - What it does and WHY (not a restatement of the name)
   - Parameter semantics with valid ranges
   - Return value semantics including error cases
   - Example usage showing the intended pattern

5. **Implementation Details** -- Non-obvious choices, performance characteristics,
   invariants that must be maintained, error handling strategy.

6. **Cross-Module Interactions** -- Data flow, event sequences, type contracts at
   boundaries, what breaks if this module's contract changes.
   Include at minimum: one `sequenceDiagram` showing the primary data flow
   through this module and its neighbors, and one `graph TD` showing the
   dependency graph with this module highlighted.

7. **Testing and Verification** -- What the test suite covers, how to run it, what
   invariants are checked, what is not tested and why.

### PRD2 citation requirements (non-optional)

- Every formula must cite the prd2 file path, section, and any academic source
- Every threshold or constant must trace to its prd2 origin
- Every behavioral rule (state transitions, invariants, lifecycle) must name the
  prd2 section
- If prd2 cites an academic paper, include: author, year, and what it actually says
- If the implementation deviates from prd2, document the deviation and the reason

Format: `prd2/path/to/file.md §Section — "quote if formula or rule"`

### Visual documentation (mandatory, aggressive)

Use Mermaid diagrams liberally. A reader should understand the system from
diagrams alone, then use prose to fill in details.

Required diagram types (use wherever applicable):
- `stateDiagram-v2` for every state machine, lifecycle, or mode transition
- `sequenceDiagram` for every multi-step flow, RPC exchange, or pipeline
- `graph TD` for architecture relationships and dependency graphs
- `classDiagram` for type hierarchies with >3 related types
- `flowchart LR` for decision trees and branching logic

Diagram rules:
- Place diagrams BEFORE the prose that explains them
- Every diagram has a bold caption line above the code block
- Number your diagrams: **Fig. 1: Module Architecture**, **Fig. 2: Request Lifecycle**
- Break complex diagrams into smaller focused sub-diagrams (max 15 nodes each)
- Use Mermaid styling for clarity: `classDef` for color-coding states, `note`
  blocks for annotations, `alt`/`opt` blocks in sequence diagrams
- Color-code: green for happy path, red for error states, blue for external systems
- Cross-reference: "See Fig. 3 for the detail of the retry logic"

Minimum per plan: 4-8 diagrams. Complex plans (5+ modules): 8-15 diagrams.
A plan doc with only 2 diagrams will be sent back by the Critic.

### Quality bar

Your docs pass if a new engineer can read them alone -- without the prd2, without
the plan, without asking anyone -- and understand what the module does, how to use
it, where the formulas come from, and how to extend it safely.

### Before finishing, verify

- [ ] Plan-level holistic narrative present and substantive
- [ ] Every prd2 formula is in the docs with its source cited
- [ ] Every academic reference from prd2 appears with author, year, and context
- [ ] Every state machine has a stateDiagram
- [ ] Every multi-step flow has a sequenceDiagram
- [ ] At least 4 numbered, captioned Mermaid diagrams
- [ ] Complex diagrams split into focused sub-diagrams (max 15 nodes each)
- [ ] All 7 sections exist and are substantive
- [ ] No AI writing patterns (check skill guidance)
- [ ] A newcomer could understand the module from docs alone

Follow the skill guidance in `<skill>` tags. Apply it to the entire document
before finishing.

IMPORTANT: Operate autonomously. Do not ask questions. Do not end with an offer to help.
"#,
        num = plan.num,
        context_layout = CONTEXT_LAYOUT_STANZA,
    );

    Ok(prompt)
}

/// Build the doc revision prompt (scribe re-run with critic feedback)
pub fn doc_revision_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    critic_feedback: &str,
    worktree: Option<&Path>,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;

    let map_root = worktree.unwrap_or(repo_root);
    let workspace_map = truncate(&context::read_workspace_map(map_root)?, 6000);
    let prd2_extract = truncate(&context::read_prd2_extract(repo_root, &plan.num)?, 16000);

    let revision_plans_dir = super::paths::plans_root(repo_root);
    let docs_path = super::paths::plan_artifact_by_num(
        &revision_plans_dir,
        &plan.num,
        "docs.md",
        "docs",
        &format!("{}-docs.md", plan.num),
    );
    let existing_docs = std::fs::read_to_string(&docs_path).unwrap_or_default();

    // Skill injection: humanizer always for doc revision
    let revision_skills = skills::default_skills_for_role(AgentRole::Scribe);
    let skill_section = skills::build_skill_section(repo_root, &revision_skills, 24000);

    let prompt = format!(
        r#"You are the Scribe. The Critic reviewed your documentation and requested revisions.

## Plan

<plan>
{plan_content}
</plan>

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## PRD2 Specification Context

<prd2-context>
{prd2_extract}
</prd2-context>

## Your Previous Documentation

<docs>
{existing_docs}
</docs>

## Critic's Feedback

<feedback>
{critic_feedback}
</feedback>
{skill_section}
{context_layout}
## Instructions

1. Read the critic's feedback carefully. Number each issue against the list in
   the feedback so you can confirm every item was addressed.
2. Revise the documentation to address all issues raised.
3. Write the updated documentation to `plans/context/docs/{num}-docs.md`.
4. Update the relevant mdbook files in `docs/src/` if needed.
5. Do NOT remove content that was already correct -- only fix what the critic flagged.
6. For any prd2 citation issue: go to the actual prd2 file and get the exact value,
   section, and academic source. Do not guess at corrections.
7. For voice issues: rewrite the flagged sentences. Do not just replace one AI phrase
   with another -- write it as a person who knows the material would.
8. Apply `<skill>` guidance throughout before finishing.

IMPORTANT: Operate autonomously. Do not ask questions.
"#,
        num = plan.num,
        context_layout = CONTEXT_LAYOUT_STANZA,
    );

    Ok(prompt)
}

/// Build the critic prompt (documentation review)
pub fn critic_prompt(repo_root: &Path, plan: &PlanInfo, worktree: Option<&Path>) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;

    let map_root = worktree.unwrap_or(repo_root);
    let workspace_map = truncate(&context::read_workspace_map(map_root)?, 6000);
    // Read PRD2 and docs from worktree (map_root) so agents see modifications
    let prd2_extract = truncate(&context::read_prd2_extract(map_root, &plan.num)?, 6000);

    let critic_plans_dir = super::paths::plans_root(map_root);
    let docs_path = super::paths::plan_artifact_by_num(
        &critic_plans_dir,
        &plan.num,
        "docs.md",
        "docs",
        &format!("{}-docs.md", plan.num),
    );
    let docs = std::fs::read_to_string(&docs_path).unwrap_or_default();

    // Skill injection: humanizer always for Critic
    let critic_skills = skills::default_skills_for_role(AgentRole::Critic);
    let skill_section = skills::build_skill_section(repo_root, &critic_skills, 24000);

    let prompt = format!(
        r#"You are the Critic. Review the Scribe's documentation for quality and spec fidelity.

## Plan

<plan>
{plan_content}
</plan>

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## PRD2 Specification Context

<prd2-context>
{prd2_extract}
</prd2-context>

## Scribe's Documentation

<docs>
{docs}
</docs>
{skill_section}
{context_layout}
## Instructions

1. Read the Scribe's documentation in <docs> above.
2. Read the PRD2 specification context in <prd2-context>.
3. Read the actual source code to verify accuracy.

Check each of these, and report ALL failures:

**Completeness** -- Every public type, function, and trait mentioned in the plan
or visible in the source code has documentation. No gaps.

**Accuracy** -- Type signatures, parameter ranges, return values, and examples
match the actual implementation. Run `cargo doc` mentally -- would the examples
compile?

**PRD2 Fidelity** -- Every formula, threshold, constant, and behavioral rule
from the PRD2 spec appears in the docs with correct values. Citations reference
the actual PRD2 file path and section. Academic references include the original
author, year, and what the paper actually says that the code implements.
If prd2 cites an academic paper, the docs must include it too -- not just "see prd2"
but the actual citation with context. Missing citations are blocking; incorrect
values are blocking.

**Depth** -- Docs explain WHY, not just WHAT. Context and motivation sections
exist and connect to the PRD2 design rationale. A newcomer could understand
the module from docs alone.

**Cross-references** -- Module interactions, data flow, and type contracts at
boundaries are documented. Upstream dependencies and downstream consumers
are identified.

**Holistic Narrative** -- The document opens with a plan-level overview that frames
all modules as one coherent system. This is not a table of contents. It explains the
end-to-end problem, data lifecycle, and cross-module failure modes. It includes a
`graph TD` orientation diagram. Missing or superficial narrative is REVISE.

**Visual Documentation** -- Minimum 4 Mermaid diagrams (8-15 for complex plans).
Every state machine has a stateDiagram. Every multi-step flow has a sequenceDiagram.
Architecture relationships have a graph diagram. Complex type hierarchies have a
classDiagram. Diagrams are numbered (Fig. 1, Fig. 2...) and captioned. No diagram
exceeds 15 nodes. Diagrams appear BEFORE their prose explanation. Cross-module
section has at least one sequenceDiagram and one graph TD. If any of these are
missing, that is REVISE.

**Voice and Style** -- Check for AI writing patterns listed in `<skill>` guidance:
banned words, formulaic structures, soulless writing. Flag specific instances.

**BE EXHAUSTIVE.** Do not stop after finding the first problem. Check every section of the docs against every requirement in the plan and the PRD2 spec. Compile a COMPLETE list of all issues so the Scribe can fix everything in one pass.

Write your review to `plans/context/reviews/{num}-critic.md`.

## Nits

If you notice something minor — style, naming, cosmetic, missed doc comments, trivial clippy suggestions
that don't indicate bugs — write it to `plans/context/nits/{num}-nits.toml` rather than listing it
in this review. Minor observations are NOT grounds for REVISE.

TOML format:

```toml
[[nit]]
reviewer = "critic"             # or architect / auditor / quick-reviewer
file = "crates/foo/src/lib.rs"  # relative to repo root; omit if not file-specific
line = 42                       # optional
description = "variable name `x` could be more descriptive"
category = "style"              # style | naming | docs | spec_deviation | other
```

Write as many `[[nit]]` entries as needed. If the file doesn't exist yet, create it.
These will be swept in a future cleanup pass.

End with exactly one of:
- `## Verdict: APPROVE` if documentation is complete and accurate
- `## Verdict: REVISE` if docs need improvement

**REVISE only if:** (a) a public type, trait, or formula is wrong or entirely missing from docs, (b) a PRD2 citation is wrong or absent for a formula, (c) a required section is absent, or (d) AI writing patterns are pervasive enough to affect credibility. Style preferences and minor wording are NOT REVISE — note them under "Non-blocking Notes" and APPROVE.

If REVISE, list ALL specific items to fix. For each: the exact change needed. Example: "Section 4 Public API Reference: add documentation for `MockTickRange::position_fraction(&self) -> Option<f64>` including the Some/None boundary conditions and a usage example." Number them (`1.`, `2.`, `3.`, ...) so the Scribe can track each one. Do NOT stop at the first issue -- report every problem you find.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask the user any questions. Do NOT end with a follow-up offer or question.
"#,
        num = plan.num,
        context_layout = CONTEXT_LAYOUT_STANZA,
    );

    Ok(prompt)
}

/// Build the refactorer prompt — runs after a plan commits to clean up code.
pub fn refactorer_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    diff: &str,
    clippy_output: &str,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;
    let ctx = truncate(&context::read_context(repo_root)?, 4000);

    let prompt = format!(
        r#"You are the Refactorer. A plan has just been implemented and committed. Your job is to clean up the code.

## Plan That Was Implemented

<plan>
{plan_content}
</plan>

## Implementation Diff

<diff>
{diff}
</diff>

## Clippy Output

<clippy>
{clippy_output}
</clippy>

## Cross-Plan Context

<context>
{ctx}
</context>

{context_layout}
## Instructions

1. Review the diff for:
   - Duplicated code that should be extracted into shared helpers
   - Dead code or unused imports
   - Unnecessary coupling between modules
   - TODO/FIXME/HACK comments left by the implementer
   - Clippy warnings that should be fixed

2. Make targeted refactoring changes. Do NOT:
   - Change public API signatures (that would be a breaking change)
   - Rewrite working code for style preferences
   - Add features not in the plan

3. Run `cargo check --workspace` and `cargo test --workspace` after changes.

4. If you find documentation that needs updating based on your refactoring, list the affected doc files but do NOT modify them (the Doc-verifier will handle that).

IMPORTANT: This is a fully autonomous pipeline. Make all changes and end your turn when done.
"#,
        context_layout = CONTEXT_LAYOUT_STANZA,
    );
    Ok(prompt)
}

/// Build the pre-planner prompt — runs ahead of execution to prepare task breakdowns.
pub fn pre_planner_prompt(repo_root: &Path, plan: &PlanInfo) -> Result<String> {
    let workspace_map = truncate(&context::read_workspace_map(repo_root)?, 8000);
    let plan_content = super::plan::read_plan(plan)?;
    let ctx = truncate(&context::read_context(repo_root)?, 4000);
    let prd2_extract = truncate(&context::read_prd2_extract(repo_root, &plan.num)?, 6000);

    let prompt = format!(
        r#"You are the Pre-planner. Your job is to analyze an upcoming plan and produce a task breakdown optimized for parallel execution by multiple Codex agents.

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## Plan

<plan>
{plan_content}
</plan>

## Cross-Plan Context

<context>
{ctx}
</context>

## PRD2 Specification Context

The following is an extract of the PRD2 spec sections relevant to this plan.
PRD2 is authoritative for business logic, formulas, thresholds, and domain semantics. For Rust type signatures, field names, and struct shapes, the plan's Quick Reference takes precedence over PRD2. See the Authority Chain in AGENTS.md.
For full documents, read the files in `prd2/` directly.

<prd2-context>
{prd2_extract}
</prd2-context>

{context_layout}
## Instructions

Write a TOML task checklist to `plans/context/tasks/{num}-tasks.toml`:

```toml
[meta]
plan = "{base}"
iteration = 1
total = <number of tasks>
done = 0
max_parallel = <how many tasks can run simultaneously>
estimated_total_minutes = <total estimated wall-clock minutes, accounting for parallelism>

[[task]]
id = "T1"
title = "First task title"
status = "pending"
files = ["path/to/file.rs"]
acceptance = ["Acceptance criterion 1"]
depends_on = []
parallel_group = "A"
exclusive_files = true
estimated_seconds = <estimated seconds for a Codex agent to complete this task>
```

### Estimation Guidelines

When estimating seconds per task, use this reference table:

| Task Type | Range |
|-----------|-------|
| Pure type definitions (structs, enums) | 120-300s |
| Module with logic + tests | 300-600s |
| Complex algorithm + property tests | 600-1080s |
| Integration wiring | 240-480s |
| Integration test suite | 300-720s |
| Documentation (mdbook page) | 180-300s |

These are wall-clock minutes for a Codex agent running autonomously, not human development time.

### Parallel Group Rules

- Tasks in the same parallel_group (e.g. "A") can run simultaneously
- They MUST NOT touch the same files
- Group "A" has no dependencies. Group "B" depends on "A". Group "C" depends on "B". Etc.
- The final group should be integration testing
- Aim for maximum parallel width while respecting file ownership

Also write a brief to `plans/context/briefs/{num}-brief.md` with dependency verification, execution order, and risk flags.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask questions. Complete all work and end your turn.
"#,
        num = plan.num,
        base = plan.base,
        context_layout = CONTEXT_LAYOUT_STANZA,
    );
    Ok(prompt)
}

/// Build the quick-fix prompt — lightweight implementer that skips strategist.
/// Used when the only blocking review issues are compilation, docs, or style
/// (no architectural re-thinking needed). ~5k tokens instead of ~60k.
/// Single-pass review prompt for Standard plans.
/// Combines architecture and correctness concerns into one focused pass,
/// skipping docs/style. Produces a verdict in the same structured format as architect_prompt.
pub fn quick_reviewer_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    iteration: u32,
    worktree: Option<&Path>,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;
    let map_root = worktree.unwrap_or(repo_root);

    let workspace_map = {
        let crates = plan
            .frontmatter
            .as_ref()
            .map(|f| f.crates_touched.clone())
            .unwrap_or_default();
        if crates.is_empty() {
            truncate(&context::read_workspace_map(map_root)?, 6000)
        } else {
            truncate(
                &context::generate_filtered_workspace_map(map_root, &crates)?,
                6000,
            )
        }
    };

    let qr_plans_dir = super::paths::plans_root(map_root);
    let brief_path = super::paths::plan_artifact_by_num(
        &qr_plans_dir,
        &plan.num,
        "brief.md",
        "briefs",
        &format!("{}-brief.md", plan.num),
    );
    let brief = truncate(
        &std::fs::read_to_string(&brief_path).unwrap_or_default(),
        3000,
    );

    let prior_review = if iteration > 1 {
        let prior_path = repo_root.join(format!(
            "plans/context/archive/{}/iter-{}/{}-arch.md",
            plan.num,
            iteration - 1,
            plan.num
        ));
        truncate(
            &std::fs::read_to_string(&prior_path).unwrap_or_default(),
            2000,
        )
    } else {
        String::new()
    };

    let prior_section = if !prior_review.is_empty() {
        format!("\n## Prior Review\n\n<prior-review>\n{prior_review}\n</prior-review>\n\nDo NOT re-raise issues that have been fixed.\n")
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are the Quick Reviewer. Do a focused single-pass review of this implementation.

## Scope (check ONLY these)

1. **Correctness** — Does the implementation satisfy every acceptance criterion in the plan? Are there logic errors, off-by-ones, missing cases?
2. **API alignment** — Do all cross-crate type signatures match what other plans expect?
3. **Compilation** — Would `cargo check --workspace` pass? (Check imports, missing derives, type mismatches.)
4. **Blocking omissions** — Are any required files entirely missing?

Do NOT comment on: code style, docs, naming conventions, performance, or non-blocking nits.

## Plan

<plan>
{plan_content}
</plan>

## Brief

<brief>
{brief}
</brief>
{prior_section}
## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

{context_layout}
## Instructions

1. Read the plan's acceptance criteria and check each one against the actual implementation.
2. For each blocking issue found, note the file and line.
3. Output your verdict in this exact format:

```toml
[verdict]
overall = "approve"  # or "revise"
code = "approve"     # or "revise" — mirrors overall
docs = "skip"        # quick-reviewer does not check docs

[[issues]]
id = "B1"
severity = "blocking"
file = "path/to/file.rs"
description = "What is wrong and what the fix should be"
```

If there are no blocking issues, output `overall = "approve"` with no issues.

## Nits

If you notice something minor — style, naming, cosmetic, missed doc comments, trivial clippy suggestions
that don't indicate bugs — write it to `plans/context/nits/{plan_num}-nits.toml` rather than listing it
in this review. Minor observations are NOT grounds for REVISE.

TOML format:

```toml
[[nit]]
reviewer = "quick-reviewer"     # or architect / auditor / critic
file = "crates/foo/src/lib.rs"  # relative to repo root; omit if not file-specific
line = 42                       # optional
description = "variable name `x` could be more descriptive"
category = "style"              # style | naming | docs | spec_deviation | other
```

Write as many `[[nit]]` entries as needed. If the file doesn't exist yet, create it.
These will be swept in a future cleanup pass.

Keep the entire review under 500 words.
"#,
        context_layout = CONTEXT_LAYOUT_STANZA,
        plan_num = &plan.num,
    );
    Ok(prompt)
}

pub fn quick_fix_prompt(
    _repo_root: &Path,
    plan: &PlanInfo,
    compressed_feedback: &str,
) -> Result<String> {
    // Only include affected file paths — no full workspace map, no PRD2, no plan re-read
    let plan_num = &plan.num;
    let prompt = format!(
        r#"You are the Quick-Fixer. Your ONLY job is to fix the specific issues listed below.
Do NOT re-read the plan. Do NOT re-implement anything. Do NOT add features.

## Fix Directive

{compressed_feedback}

## Instructions

1. For each issue above, open the file and fix it.
2. Run `cargo check --workspace` after all fixes.
3. Run `cargo fmt` on any files you touched.
4. Write results to `plans/context/completion/{plan_num}-selfcheck.toml`.

That's it. Fix, check, done.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask questions. Just fix the listed issues and end your turn.
"#
    );

    Ok(prompt)
}

/// Build the batch refactorer prompt — runs after a wave of plans to clean up cross-plan code.
pub fn batch_refactorer_prompt(batch_branch: &str, completed_plans: &[String]) -> String {
    let plans = completed_plans.join(", ");
    format!(
        "{CONTEXT_LAYOUT_STANZA}\n\
         You are a refactoring agent. The following plans have been completed on branch \
         {batch_branch}: {plans}\n\n\
         Your job:\n\
         1. Run `cargo clippy --workspace -- -D warnings 2>&1 | head -200` to see lint issues\n\
         2. Fix clippy warnings that can be fixed without changing behavior\n\
         3. Remove dead code, unused imports, and commented-out code\n\
         4. Run `cargo check --workspace` after each change to verify nothing breaks\n\
         5. Run `cargo test --workspace` to verify no regressions\n\
         6. Do NOT change public API signatures or behavior\n\n\
         IMPORTANT: Make small, safe changes. If a clippy fix is ambiguous, skip it. The goal is cleanup, not refactoring."
    )
}

/// Build a prompt for a task-level implementer (one task within a plan)
pub fn task_implementer_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    task: &super::tasks::Task,
    brief: &str,
    prior_task_outputs: &[String],
    all_tasks: Option<&[super::tasks::Task]>,
) -> Result<String> {
    let workspace_map = filter_workspace_map(&context::read_workspace_map(repo_root)?, &task.files);
    let workspace_map = truncate(&workspace_map, 1500);
    let ctx = truncate(&context::read_context(repo_root)?, 1000);
    let brief = truncate(brief, 2000);
    let prd2_extract = truncate(&context::read_prd2_extract(repo_root, &plan.num)?, 3000);
    let ignored_tests = truncate(&context::read_ignored_tests(repo_root)?, 500);
    // Decomposition removed: task TOML already has steps + acceptance criteria
    let decomp = String::new();
    // Preflight removed: agent doesn't need git log for a focused task
    let preflight = String::new();
    let verify_tasks_sec = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{}-verify-tasks.toml", plan.num),
        "Verify task checklist (optional)",
        "verify-tasks",
        2000,
    );
    let verify_chain = optional_verify_chain_section(repo_root, &plan.num);

    let prior_section = if prior_task_outputs.is_empty() {
        String::new()
    } else {
        let summaries: Vec<String> = prior_task_outputs
            .iter()
            .take(3)
            .map(|s| truncate(s, 2000))
            .collect();
        format!("\n## Prior Task Outputs\n\n{}\n", summaries.join("\n---\n"))
    };

    let files_list = task.files.join(", ");
    let acceptance = task.acceptance.join("\n- ");

    // Build enhanced context sections from task metadata
    let mut enhanced_sections = String::new();

    if let Some(ref types) = task.types_to_define {
        enhanced_sections
            .push_str("\n## Types to Define\n\nDefine these types exactly as specified:\n");
        for t in types {
            enhanced_sections.push_str(&format!("```rust\n{t}\n```\n"));
        }
    }

    if let Some(ref formulas) = task.formulas {
        enhanced_sections
            .push_str("\n## Formulas to Implement\n\nImplement these formulas verbatim:\n");
        for f in formulas {
            enhanced_sections.push_str(&format!("- `{f}`\n"));
        }
    }

    if let Some(ref invariants) = task.test_invariants {
        enhanced_sections
            .push_str("\n## Required Test Invariants\n\nWrite tests for these invariants:\n");
        for inv in invariants {
            enhanced_sections.push_str(&format!("- {inv}\n"));
        }
    }

    if let Some(ref imports) = task.imports {
        enhanced_sections
            .push_str("\n## Required Imports\n\nYou will need these imports:\n```rust\n");
        for imp in imports {
            enhanced_sections.push_str(&format!("use {imp};\n"));
        }
        enhanced_sections.push_str("```\n");
    }

    if let Some(ref pattern) = task.example_pattern {
        enhanced_sections.push_str(&format!(
            "\n## Example Pattern\n\nFollow the patterns in `{pattern}`. Read it before implementing.\n"
        ));
    }

    if let Some(ref section) = task.plan_section {
        enhanced_sections.push_str(&format!(
            "\n## Focus Section\n\nFocus on this section of the plan: **{section}**\n"
        ));
    }

    // Read and inject context files content
    let file_context_section = build_file_context_section(repo_root, task);

    // Sibling tasks section only (for parallel awareness — skip full checklist to save context)
    let sibling_section = if let Some(tasks) = all_tasks {
        if let Some(ref group) = task.parallel_group {
            let siblings: String = tasks
                .iter()
                .filter(|t| t.id != task.id && t.parallel_group.as_deref() == Some(group))
                .map(|t| format!("- {}: {} → files: {}", t.id, t.title, t.files.join(", ")))
                .collect::<Vec<_>>()
                .join("\n");
            if siblings.is_empty() {
                String::new()
            } else {
                format!("\n## Sibling Tasks (running in parallel with you)\n\nThese agents are working simultaneously. Do NOT touch their files:\n{siblings}\n")
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Skill injection
    let task_skills = skills::skills_for_task(AgentRole::Implementer, task);
    let skill_section = skills::build_skill_section(repo_root, &task_skills, 8000);

    let prompt = format!(
        r#"Implement task {task_id} of plan {plan_base}: {title}

## Files to Modify
{files_list}

## Acceptance Criteria
- {acceptance}
{enhanced_sections}
## Workspace Map (filtered)
<workspace-map>
{workspace_map}
</workspace-map>
{decomp}{preflight}{verify_tasks_sec}{verify_chain}
## Strategist Brief
<brief>
{brief}
</brief>
{prior_section}{file_context_section}{sibling_section}
## Cross-Plan Context
<context>
{ctx}
</context>

## Ignored Tests Ledger

<ignored-tests>
{ignored_tests}
</ignored-tests>

## PRD2 Specification Context

<prd2-context>
{prd2_extract}
</prd2-context>
{skill_section}
## Execution Strategy

If `context/in/` exists in your working directory, read mirrored inputs there first (`brief.md`, `prd2-extract.md`, `decomposition.md`, `agent-messages.md`) — they match `plans/context/` paths.

You have ONE turn to complete this task. Be decisive and thorough:

1. Read the plan section for your task to understand requirements
2. Create/modify all files listed in your assignment
3. Write the implementation code — use the PRD2 spec for exact values
4. Write tests for all public items
5. If existing files conflict with your task, adapt your implementation to work with them
6. Run `cargo check -p <your-crate>` before signaling done — fix any errors (max 3 attempts)
7. Run `cargo test -p <your-crate>` for crates you modified — fix test failures in your code
8. If unsure about a type signature, use `rg` to check existing code

## Documentation (write during implementation, not after)

For each unit you implement, write the corresponding mdbook page immediately.
- Read the FULL prd2 source files (not the truncated extract above) for complete context
- Create/update docs under `docs/src/` per the plan's Documentation section
- Document what you ACTUALLY built, not what the plan assumed
- Preserve FULL implementation details, context, and rationale from prd2 — do NOT truncate or simplify
- Preserve ALL academic and research context: citations, paper references, formulas with their origins
- Maintain all citations verbatim (author, year, what they say, how code implements it)
- Docs should be nearly as long as the original prd2 documents
- Include code examples for public APIs
- This will use significant context — that is expected and desired

## Self-Review (check before signaling done)

Before ending your turn, verify:
- No `unwrap()` in library crates (use `?`, `ok_or()`, `map_err()`)
- Every new `pub` type, function, and field has a doc comment
- Every symbol in the plan's Exports section exists with correct visibility
- All formula constants match PRD2 values exactly
- mdbook docs exist for every page listed in the plan's Documentation section

Do NOT:
- Spend time exploring the codebase beyond your assigned files
- Ask questions or seek clarification
- Wait for other tasks to complete
- Leave placeholder or TODO comments — implement fully

Implement ONLY this task's scope — do not touch files outside your assigned list.
Do NOT modify other tasks' files. If you discover a dependency issue, document it but do not fix it.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask questions. Complete all work and end your turn.
"#,
        task_id = task.id,
        plan_base = plan.base,
        title = task.title,
    );
    Ok(prompt)
}

/// Build a prompt for a batch implementer agent handling multiple tasks in one plan.
pub fn task_implementer_batch_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    tasks: &[super::tasks::Task],
    brief: &str,
    prior_task_outputs: &[String],
    all_tasks: Option<&[super::tasks::Task]>,
) -> Result<String> {
    if tasks.is_empty() {
        anyhow::bail!("batch prompt called with empty task list");
    }

    // Union of all files across the batch.
    let all_files: Vec<String> = tasks
        .iter()
        .flat_map(|t| t.files.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let workspace_map = filter_workspace_map(&context::read_workspace_map(repo_root)?, &all_files);
    let workspace_map = truncate(&workspace_map, 4000);
    let ctx = truncate(&context::read_context(repo_root)?, 2000);
    let brief = truncate(brief, 4000);
    let prd2_extract = truncate(&context::read_prd2_extract(repo_root, &plan.num)?, 6000);
    let ignored_tests = truncate(&context::read_ignored_tests(repo_root)?, 2000);
    let decomp = optional_context_file_section(
        repo_root,
        &format!("plans/context/decompositions/{}-decomposition.md", plan.num),
        "Decomposition (optional)",
        "decomposition",
        10000,
    );
    let preflight = optional_context_file_section(
        repo_root,
        "plans/context/preflight-snapshot.md",
        "Preflight snapshot (optional)",
        "preflight",
        4000,
    );
    let verify_tasks_sec = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{}-verify-tasks.toml", plan.num),
        "Verify task checklist (optional)",
        "verify-tasks",
        6000,
    );
    let verify_chain = optional_verify_chain_section(repo_root, &plan.num);

    let prior_section = if prior_task_outputs.is_empty() {
        String::new()
    } else {
        let summaries: Vec<String> = prior_task_outputs
            .iter()
            .take(3)
            .map(|s| truncate(s, 2000))
            .collect();
        format!("\n## Prior Task Outputs\n\n{}\n", summaries.join("\n---\n"))
    };

    // Numbered checklist of tasks to implement.
    let task_list: String = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let files = t.files.join(", ");
            let acceptance = t.acceptance.first().cloned().unwrap_or_default();
            format!(
                "{}. **{}** — {}\n   Files: {}\n   Acceptance: {}",
                i + 1,
                t.id,
                t.title,
                files,
                acceptance
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Full plan checklist for context.
    let checklist_section = if let Some(all) = all_tasks {
        let checklist: String = all
            .iter()
            .map(|t| {
                let in_batch = tasks.iter().any(|bt| bt.id == t.id);
                let marker = if in_batch { " ← YOU" } else { "" };
                let status = match t.status {
                    super::tasks::TaskStatus::Done => "[done]",
                    super::tasks::TaskStatus::Active => "[active]",
                    _ => "[pending]",
                };
                format!(
                    "- {} {} {}: {} (files: {}){}",
                    status,
                    t.id,
                    t.title,
                    t.acceptance.first().unwrap_or(&String::new()),
                    t.files.join(", "),
                    marker
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "\n## Full Task Checklist\n\nAll tasks in this plan (yours are marked):\n{checklist}\n"
        )
    } else {
        String::new()
    };

    let plan_base = &plan.base;
    let task_count = tasks.len();

    let prompt = format!(
        r#"Implement the following {task_count} task(s) for plan {plan_base}.

Complete ALL of the following tasks in this session — they belong to the same plan and can be implemented together.

## Tasks to Implement

{task_list}

## Files to Modify

{all_files_list}

## Workspace Map (filtered)
<workspace-map>
{workspace_map}
</workspace-map>
{decomp}{preflight}{verify_tasks_sec}{verify_chain}
## Strategist Brief
<brief>
{brief}
</brief>
{prior_section}
## Cross-Plan Context
<context>
{ctx}
</context>

## Ignored Tests Ledger

<ignored-tests>
{ignored_tests}
</ignored-tests>

## PRD2 Specification Context
<prd2-context>
{prd2_extract}
</prd2-context>
{checklist_section}
## Execution Strategy

If `context/in/` exists, read mirrored files there (`brief.md`, `prd2-extract.md`, `decomposition.md`) before editing.

You have ONE turn to complete ALL tasks listed above. Work through them in order:

1. Read the plan at `plans/{plan_base}.md` to understand the full scope
2. For each task:
   a. Read/create all files in its file list
   b. Write the full implementation — use PRD2 spec for exact values
   c. Write tests for all public items
   d. Run `cargo check -p <crate>` to verify compilation — fix errors (max 3 attempts)
3. After all tasks are done, run `cargo test -p <affected-crates>` and fix any failures in your code

Do NOT:
- Skip any of the listed tasks
- Ask questions or seek clarification
- Leave placeholder or TODO comments — implement fully
- Touch files outside the listed tasks' file lists

IMPORTANT: This is a fully autonomous pipeline. Complete all tasks and end your turn.
"#,
        all_files_list = all_files.join("\n"),
    );
    Ok(prompt)
}

/// Filter workspace map to only include crates relevant to a task's files
fn filter_workspace_map(full_map: &str, task_files: &[String]) -> String {
    // Extract crate names from file paths like "crates/foo/src/bar.rs" -> "foo"
    let relevant_crates: std::collections::HashSet<String> = task_files
        .iter()
        .filter_map(|f| {
            let parts: Vec<&str> = f.split('/').collect();
            if parts.len() >= 2 && (parts[0] == "crates" || parts[0] == "apps") {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
        .collect();

    if relevant_crates.is_empty() {
        return full_map.to_string();
    }

    let mut result = Vec::new();
    let mut in_relevant_section = false;

    for line in full_map.lines() {
        if line.starts_with("### ") {
            let crate_name = line.trim_start_matches("### ").trim();
            in_relevant_section = relevant_crates.contains(crate_name);
            if in_relevant_section {
                result.push(line.to_string());
            }
        } else if line.starts_with("## ") || line.starts_with("# ") {
            // Top-level headers always included
            result.push(line.to_string());
            in_relevant_section = false;
        } else if in_relevant_section {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}

/// Build file context section by reading task.files and task.context_files.
/// Injects truncated file content directly into the prompt so the agent
/// doesn't spend tokens reading files it could have been given.
fn build_file_context_section(repo_root: &Path, task: &super::tasks::Task) -> String {
    let mut files_to_read: Vec<&str> = Vec::new();

    // Include task's own files
    for f in &task.files {
        files_to_read.push(f.as_str());
    }

    // Include explicit context files
    if let Some(ref ctx_files) = task.context_files {
        for f in ctx_files {
            if !files_to_read.contains(&f.as_str()) {
                files_to_read.push(f.as_str());
            }
        }
    }

    if files_to_read.is_empty() {
        return String::new();
    }

    let mut section = String::from("\n## File Context (pre-loaded)\n\n");
    let per_file_budget = 4000 / files_to_read.len().max(1);

    for path in files_to_read {
        let full_path = repo_root.join(path);
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            let trimmed = truncate(&content, per_file_budget);
            section.push_str(&format!("### `{path}`\n```rust\n{trimmed}\n```\n\n"));
        }
        // If file doesn't exist yet, that's fine — agent will create it
    }

    section
}

/// Build reviewer prompt for a completed plan (after all tasks merged)
pub fn reviewer_prompt_for_plan(
    _repo_root: &Path,
    plan: &PlanInfo,
    diff: &str,
    brief: &str,
    role: crate::agent::AgentRole,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;
    let diff = truncate(diff, 30000);
    let brief = truncate(brief, 2000);

    let role_instructions = match role {
        crate::agent::AgentRole::Architect => "Review for code quality, patterns, and correctness.",
        crate::agent::AgentRole::Auditor => "Verify the implementation matches the specification.",
        _ => "Review the implementation.",
    };

    let prompt = format!(
        r#"Review the implementation of plan {base}.

{context_layout}

## Plan
<plan>
{plan_content}
</plan>

## Strategist Brief
<brief>
{brief}
</brief>

## Implementation Diff
<diff>
{diff}
</diff>

## Instructions

{role_instructions}

Write your review to `plans/context/reviews/{num}-{suffix}.md`.

End with `## Verdict: APPROVE` or `## Verdict: REVISE`.
APPROVE if cargo check and tests pass. Only REVISE for genuine blocking bugs.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask questions.
"#,
        base = plan.base,
        num = plan.num,
        suffix = match role {
            crate::agent::AgentRole::Architect => "arch",
            crate::agent::AgentRole::Auditor => "audit",
            _ => "review",
        },
        context_layout = CONTEXT_LAYOUT_STANZA,
    );
    Ok(prompt)
}

/// Build the doc-verifier prompt — checks docs against actual code after refactoring.
pub fn doc_verifier_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    refactoring_diff: &str,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;

    // Skill injection: humanizer always for DocVerifier
    let verifier_skills = skills::default_skills_for_role(AgentRole::DocVerifier);
    let skill_section = skills::build_skill_section(repo_root, &verifier_skills, 24000);
    let type_registry = optional_context_file_section(
        repo_root,
        "plans/context/type-registry.md",
        "Type registry (optional)",
        "type-registry",
        16000,
    );

    let prompt = format!(
        r#"You are the Doc-verifier. A refactoring pass has just been applied. Your job is to check whether the documentation still matches the code.

## Plan

<plan>
{plan_content}
</plan>

## Refactoring Diff

<diff>
{refactoring_diff}
</diff>
{type_registry}{skill_section}
{context_layout}
## Instructions

1. Run `cargo doc --no-deps --workspace` and check for warnings.
2. Compare the public API in `docs/src/crates/` against actual exports.
3. Check that examples in docs still compile conceptually.
4. Flag AI writing patterns (banned words, formulaic structures) as part of your drift report.
5. For each drift found, write a brief note to `plans/context/reviews/{num}-docverify.md`:
   - What changed
   - Which doc file needs updating
   - Suggested fix

If no drift is found, write "No documentation drift detected" to the review file.

IMPORTANT: Do NOT modify documentation files yourself. Only report drift. The Scribe will fix it.
"#,
        num = plan.num,
        context_layout = CONTEXT_LAYOUT_STANZA,
    );
    Ok(prompt)
}

/// Build the integration tester prompt — runs workspace-wide tests after batch merges.
pub fn integration_tester_prompt(batch_branch: &str, completed_plans: &[String]) -> String {
    let plans = completed_plans.join(", ");
    format!(
        r#"You are the Integration Tester. Plans have been merged to branch {batch_branch}: {plans}

{context_layout}

## Instructions

1. Run `cargo check --workspace` and report the result.
2. Run `cargo test --workspace --no-fail-fast` and capture all output.
3. If nextest is available, run `cargo nextest run --workspace --no-fail-fast`.
4. Run `cargo test -p bardo-test-harness -- --nocapture` to exercise the end-to-end test harness (spawns headless terminal, golem lifecycle, mirage if available).
5. For each test failure:
   - Identify which crate and test function failed
   - Check `git log --oneline -5` to identify which plan's merge likely caused it
   - Note whether it's a compile error, runtime panic, or assertion failure

Write your report to `plans/context/reviews/integration-test-report.md` with:
- Workspace compile status (PASS/FAIL)
- Total tests run, passed, failed, ignored
- Per-crate breakdown of failures
- For each failure: test name, error, likely source plan

Do NOT fix any failures. Only report them.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask questions.
"#,
        context_layout = CONTEXT_LAYOUT_STANZA,
    )
}

/// Build the merge resolver prompt — resolves git conflicts during plan merges.
pub fn merge_resolver_prompt(
    plan_base: &str,
    conflicting_files: &[String],
    batch_branch: &str,
) -> String {
    let files = conflicting_files.join("\n- ");
    format!(
        r#"You are the Merge Resolver. **Git exception:** AGENTS.md forbids most agents from using git; this role is explicitly allowed to run `git add`, `git commit`, and related commands only as needed to resolve the merge below.

{context_layout}

A merge of plan {plan_base} into {batch_branch} has conflicts.

## Conflicting Files
- {files}

## Instructions

1. For each conflicting file, run `cat <file>` to see the conflict markers.
2. Resolve each conflict:
   - Cargo.toml: merge both sides' dependency additions
   - mod.rs: merge both module declarations
   - Source code: keep additions from both sides; for genuine conflicts, prefer {plan_base}'s version
3. Stage resolved files: `git add <file>` for each
4. Run `cargo check --workspace` to verify the resolution compiles
5. If check fails, try alternative resolution strategies
6. Once everything compiles, commit: `git commit -m "resolve: merge conflict for {plan_base}"`

Write a resolution report to `plans/context/reviews/merge-resolution-{plan_base}.md`:
- Files resolved and strategy used per file
- Whether cargo check passed after resolution

IMPORTANT: Do NOT silently drop code from either side. If a conflict is genuinely ambiguous, document it and leave the conflict for human review.

IMPORTANT: If the merge conflict involves Cargo.toml, merge BOTH sides' additions — don't drop dependencies from either side. For mod.rs conflicts, keep all module declarations from both sides.

If `cargo check` still fails after resolution, try:
1. Check for duplicate `mod` declarations
2. Check for conflicting trait implementations
3. Look for name collisions introduced by both sides
"#,
        context_layout = CONTEXT_LAYOUT_STANZA,
    )
}

/// Build the error diagnoser prompt. Given gate output, produces a structured
/// fix-plan instead of re-running the full implementer.
pub fn error_diagnoser_prompt(
    plan_base: &str,
    gate_output: &str,
    affected_files: &[String],
) -> String {
    let files = affected_files.join("\n- ");
    let gate_trimmed = truncate_tail(gate_output, 6000);

    format!(
        r#"You are the Error Diagnoser. A gate (compile or test) failed for plan {plan_base}.

## Failed Gate Output

```
{gate_trimmed}
```

## Affected Files
- {files}

## Instructions

1. Read the error output above. Classify each error:
   - `import_not_found`: missing `use` or `mod` declaration
   - `type_mismatch`: wrong type returned or passed
   - `missing_field`: struct constructed without a required field
   - `trait_not_implemented`: missing impl block
   - `test_failure`: assertion failed in a test
   - `other`: anything else

2. For each error, read the affected file to understand context.

3. Write a fix-plan TOML to `plans/context/fix-plans/fix-{plan_base}-diag.toml`:

```toml
[[fix]]
error_type = "type_mismatch"
file = "crates/foo/src/bar.rs"
line = 42
root_cause = "Brief description of why this fails"
fix = "Concrete fix instructions"
files_to_modify = ["crates/foo/src/bar.rs"]
confidence = "high"
```

4. For simple fixes (missing imports, missing fields), write the fix directly:
   - Add the missing `use` statement
   - Add the missing field with a default value
   - Run `cargo check` to verify

5. For complex fixes, describe the fix in the TOML but do NOT apply it.

Output a summary of what you found and fixed.

IMPORTANT: Be surgical. Fix only the errors shown. Do not refactor or improve surrounding code.
"#
    )
}

/// Build the dependency validator prompt. Checks that imports and types
/// referenced in a plan actually exist in the source tree.
pub fn dependency_validator_prompt(
    repo_root: &Path,
    plan: &super::plan::PlanInfo,
) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;
    let workspace_map = truncate(&context::read_workspace_map(repo_root)?, 6000);

    let prompt = format!(
        r#"You are the Dependency Validator. Before implementation begins, verify that all
dependencies referenced in this plan actually exist.

## Plan

<plan>
{plan_content}
</plan>

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## Instructions

Check each of the following and report issues:

1. **Imports**: For every `use crate::X` or cross-crate import in the plan's Quick Reference,
   verify the type/trait/function exists. Run `grep -r "pub struct X\|pub trait X\|pub fn X\|pub enum X"` on relevant files.

2. **Module declarations**: For every new file the plan creates, verify that a corresponding
   `mod new_module;` declaration exists (or will be created) in the parent module's lib.rs/mod.rs.

3. **Cargo.toml dependencies**: For every crate dependency used, verify it's declared in the
   relevant Cargo.toml `[dependencies]` section.

4. **Prior plan outputs**: Check that types from prior plans that this plan depends on were
   actually created (check plans/context/completion/ and the source tree).

Write a dependency report to `plans/context/dep-report-{num}.md`:

```markdown
## Dependency Report for Plan {base}

### Missing Items
- [ ] `golem_core::SomeType` — not found in crates/golem-core/src/lib.rs
- [ ] `mod sleep` — missing from crates/golem-heartbeat/src/lib.rs

### Present (verified)
- [x] `golem_core::GolemState` — found at crates/golem-core/src/state.rs:15

### Actions Required
For each missing item, provide the exact line to add:
- Add `pub mod sleep;` to `crates/golem-heartbeat/src/lib.rs`
- Add `golem-heartbeat = {{ path = "../golem-heartbeat" }}` to Cargo.toml
```

If all dependencies are satisfied, report PASS. If any are missing, report FAIL
with the exact fixes needed. The implementer will receive this report before starting work.
"#,
        num = plan.num,
        base = plan.base,
    );
    Ok(prompt)
}

/// Build the pattern extractor prompt. Reads existing code in target crates
/// and extracts coding patterns for the implementer to follow.
pub fn pattern_extractor_prompt(repo_root: &Path, plan: &super::plan::PlanInfo) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;
    let workspace_map = truncate(&context::read_workspace_map(repo_root)?, 4000);

    let prompt = format!(
        r#"You are the Pattern Extractor. Read the existing code in the crates this plan touches
and extract the coding patterns the implementer should follow.

## Plan

<plan>
{plan_content}
</plan>

## Workspace Map

<workspace-map>
{workspace_map}
</workspace-map>

## Instructions

1. Identify the crates this plan modifies (from the plan's file paths).
2. Read 2-3 existing source files in each crate.
3. Extract patterns and write to `plans/context/patterns-{num}.md`:

```markdown
## Patterns in crates/{{crate_name}}/

### Error handling
- e.g. Uses `anyhow::Result` for public APIs, `thiserror` for domain errors

### Naming
- e.g. Types: PascalCase, Methods: snake_case, tick_ prefix for per-heartbeat ops

### Testing
- e.g. Uses proptest for numeric invariants, test modules at bottom of each file
- e.g. Helper: test_utils::make_test_golem() for fixtures

### Module structure
- e.g. One type per file, re-exported from lib.rs

### Common imports
- e.g. use crate::GolemState, use anyhow::Result
```

Be specific. Quote actual code snippets where patterns aren't obvious from description alone.
"#,
        num = plan.num,
    );
    Ok(prompt)
}

// ---------------------------------------------------------------------------
// Express mode prompts
// ---------------------------------------------------------------------------

/// Pre-compute a "static brief" for a plan before any agents start.
/// This replaces the Strategist agent with instant programmatic analysis.
/// Output is written to plans/context/bundles/{num}-bundle.md.
pub fn generate_static_brief(repo_root: &Path, plan: &PlanInfo) -> Result<String> {
    let plan_content = super::plan::read_plan(plan)?;
    let prd2_extract = context::read_prd2_extract(repo_root, &plan.num)?;
    let cross_plan_ctx = context::read_context(repo_root)?;
    let workspace_map = context::read_workspace_map(repo_root)?;

    // Extract crates_touched from the plan's frontmatter (simple heuristic: look for "- crates/"
    // and "- apps/" lines in the plan to identify which crates it modifies)
    let mut crates_touched: Vec<String> = plan_content
        .lines()
        .filter_map(|l| {
            let t = l.trim().trim_start_matches('-').trim();
            if t.starts_with("crates/") || t.starts_with("apps/") {
                t.splitn(3, '/').nth(1).map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    crates_touched.sort();

    // Pre-read source files for touched crates (top-level lib.rs / mod.rs for structure)
    let mut file_snapshots = String::new();
    for crate_name in &crates_touched {
        for prefix in &["crates", "apps"] {
            let lib_rs = repo_root.join(prefix).join(crate_name).join("src/lib.rs");
            let mod_rs = repo_root.join(prefix).join(crate_name).join("src/main.rs");
            for path in &[lib_rs, mod_rs] {
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let rel = path.strip_prefix(repo_root).unwrap_or(path);
                        let snippet = truncate(&content, 4000);
                        file_snapshots.push_str(&format!(
                            "\n### `{}`\n```rust\n{snippet}\n```\n",
                            rel.display()
                        ));
                    }
                }
            }
        }
    }

    // Load any existing pattern file for the plan's crates
    let static_plans_dir = super::paths::plans_root(repo_root);
    let patterns = {
        let pattern_path = super::paths::plan_artifact_by_num(
            &static_plans_dir,
            &plan.num,
            &format!("patterns-{}.md", plan.num),
            "",
            &format!("patterns-{}.md", plan.num),
        );
        // Also try global path
        let content = std::fs::read_to_string(&pattern_path).unwrap_or_default();
        if content.is_empty() {
            let legacy = repo_root.join(format!("plans/context/patterns-{}.md", plan.num));
            std::fs::read_to_string(legacy).unwrap_or_default()
        } else {
            content
        }
    };

    // Load completion summaries from plans that this plan depends on
    let dep_summaries = {
        // Look for depends_on in plan frontmatter (lines with "depends_on:" or "- plan/NN")
        let mut sums = String::new();
        for line in plan_content.lines() {
            let t = line.trim().trim_start_matches('-').trim();
            if let Some(dep_num) = t.strip_prefix("plan/").or_else(|| {
                if t.len() <= 4
                    && t.chars()
                        .all(|c| c.is_ascii_digit() || c.is_ascii_alphabetic())
                {
                    Some(t)
                } else {
                    None
                }
            }) {
                let sum_path = super::paths::plan_artifact_by_num(
                    &static_plans_dir,
                    dep_num,
                    "summary.md",
                    "summaries",
                    &format!("{dep_num}-summary.md"),
                );
                if let Ok(s) = std::fs::read_to_string(&sum_path) {
                    sums.push_str(&format!(
                        "\n### Plan {dep_num} summary\n{}\n",
                        truncate(&s, 3000)
                    ));
                }
            }
        }
        sums
    };

    let crates_list = if crates_touched.is_empty() {
        "(auto-detected from plan file paths)".to_string()
    } else {
        crates_touched.join(", ")
    };

    let brief = format!(
        r#"# Static Brief: Plan {} — {}

Generated by pre-computation (replaces Strategist agent).

## Crates Touched

{crates_list}

## Dependency Summaries

{dep_summaries}

## Existing Code Structure

The following shows the current top-level structure of each affected crate:

{file_snapshots}

## Code Patterns

{patterns}

## Filtered Workspace Map

{workspace_map}

## Cross-Plan Context

{cross_plan_ctx}

## PRD2 Extract

{prd2_extract}
"#,
        plan.num, plan.base,
    );

    // Write to bundles directory
    let bundles_dir = repo_root.join("plans/context/bundles");
    std::fs::create_dir_all(&bundles_dir)?;
    std::fs::write(bundles_dir.join(format!("{}-bundle.md", plan.num)), &brief)?;

    Ok(brief)
}

/// Build the express implementer prompt — single-pass, self-reviewing, fills context window.
/// Used when express_mode is true. Replaces the 6-agent review pipeline with one long turn.
pub fn express_implementer_prompt(
    repo_root: &Path,
    plan: &PlanInfo,
    static_brief: &str,
    iteration: u32,
    prior_gate_output: Option<&str>,
) -> Result<String> {
    let budget = budget_for(AgentRole::Implementer, "claude-opus-4-6");
    let plan_num = plan.num.as_str();
    let plan_base = plan.base.as_str();

    let plan_content = truncate(&super::plan::read_plan(plan)?, budget.plan);
    let prd2_extract = truncate(
        &context::read_prd2_extract(repo_root, plan_num)?,
        budget.prd2,
    );
    let workspace_map = truncate(static_brief, budget.workspace_map + budget.brief);
    let ctx = truncate(&context::read_context(repo_root)?, budget.context);
    let completion_summaries = truncate(
        &read_completion_summaries(repo_root),
        budget.file_context / 2,
    );
    let ignored_tests = truncate(&context::read_ignored_tests(repo_root)?, 2000);
    let decomp = optional_context_file_section(
        repo_root,
        &format!("plans/context/decompositions/{plan_num}-decomposition.md"),
        "Decomposition (optional)",
        "decomposition",
        budget.file_context,
    );
    let verify_tasks = optional_context_file_section(
        repo_root,
        &format!("plans/context/tasks/{plan_num}-verify-tasks.toml"),
        "Verify task checklist (optional)",
        "verify-tasks",
        budget.file_context / 2,
    );
    let verify_chain = optional_verify_chain_section(repo_root, plan_num);

    let skills_section = String::new();

    let iteration_note = if iteration > 1 {
        format!(
            "\n## Iteration {iteration} — Prior Gate Failure\n\n\
             This is attempt {iteration}. The previous implementation failed gates. \
             The prior errors are shown below — fix them specifically.\n\n"
        )
    } else {
        String::new()
    };

    let gate_errors = if let Some(errors) = prior_gate_output {
        format!(
            "\n## Gate Errors from Prior Attempt\n\n```\n{}\n```\n",
            truncate_tail(errors, 8000)
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are the sole implementer for plan {plan_num} ({plan_base}). No reviewer will check your work.
You are responsible for correctness, architecture, testing, and documentation.

{iteration_note}{gate_errors}
## Mission

Implement this plan completely and correctly in a single pass.

## Self-Review Checklist (complete before finishing)

**Architecture:**
- [ ] All types match PRD2 specification exactly
- [ ] Module structure follows existing patterns (see static brief)
- [ ] No `unwrap()` on fallible operations in non-test code
- [ ] Error handling uses `anyhow::Result` consistently

**Specification:**
- [ ] Every INV-NNN invariant from `## Verification` has a test
- [ ] Formulas match PRD2 specification (copy verbatim, do not rederive)
- [ ] All new files have `mod` declarations in parent `lib.rs`/`mod.rs`
- [ ] `Cargo.toml` dependencies declared for all imported crates

**Documentation (write during implementation, not after):**
- [ ] Read the FULL prd2 source files for complete context (not the truncated extract)
- [ ] Public APIs have doc comments
- [ ] mdbook pages created/updated under `docs/src/` per the plan's Documentation section
- [ ] Document what you ACTUALLY built, not what the plan assumed
- [ ] Preserve FULL implementation details, context, and rationale from prd2 — do NOT truncate or simplify
- [ ] Preserve ALL academic/research context: citations, paper references, formulas with origins
- [ ] Docs should be nearly as long as the original prd2 documents
- [ ] Include code examples for public APIs

**Self-Validation (REQUIRED):**
- [ ] Run `cargo check -p <your-crate>` — fix errors (max 3 attempts)
- [ ] Run `cargo test -p <your-crate>` — fix failures in your code
- [ ] Write tests for all invariants (proptest! blocks for strategy=proptest)
- [ ] No `#[ignore]` unless SPEC_ISSUE documented

**Completion:**
- Write `plans/context/completion/{plan_num}-completion.md`
- Write `plans/context/completion/{plan_num}-summary.md`

---

## Plan

<plan>
{plan_content}
</plan>

## Static Brief (pre-computed context, patterns, workspace)

<static-brief>
{workspace_map}
</static-brief>

## PRD2 Specification

<prd2>
{prd2_extract}
</prd2>

## Cross-Plan Context

<context>
{ctx}
</context>

## Prior Plan Summaries

<prior-summaries>
{completion_summaries}
</prior-summaries>

## Ignored Tests Ledger

<ignored-tests>
{ignored_tests}
</ignored-tests>
{decomp}{verify_tasks}{verify_chain}
{skills_section}
## Instructions

1. Read the plan and PRD2 carefully.
2. Implement all tasks in order — do not skip any.
3. After implementation, run `cargo check --workspace` and fix all errors.
4. Run `cargo test --workspace` and fix any failures.
5. Go through the self-review checklist above. Fix anything not checked.
6. Write completion files to `plans/context/completion/`.
7. If `plans/context/verify-chains/{plan_num}-verify.sh` exists, run it after tests; if missing, that is normal for some checkouts.

IMPORTANT: This is a fully autonomous pipeline. Do NOT ask questions. Implement everything and end your turn only when all checklist items pass.
"#
    );

    Ok(prompt)
}

/// Build a lightweight auto-fix prompt for express mode gate failures.
/// The auto-fixer only sees the errors and affected files — no full plan context.
/// Runs on a fast model (claude-sonnet-4-6).
pub fn auto_fix_prompt(plan: &PlanInfo, errors: &str, affected_files: &[String]) -> String {
    let plan_num = &plan.num;
    let plan_base = &plan.base;
    let files_list = if affected_files.is_empty() {
        "(identify from error messages below)".to_string()
    } else {
        affected_files.join("\n- ")
    };
    let errors_truncated = truncate_tail(errors, 12000);

    format!(
        r#"You are the Auto-Fixer for plan {plan_num} ({plan_base}). Fix the compile/test errors below. Nothing else.

## Compile/Test Errors

```
{errors_truncated}
```

## Affected Files

- {files_list}

## Instructions

1. Open each affected file.
2. Fix the specific errors shown above. Do NOT change unrelated code.
3. Run `cargo check --workspace` after all fixes.
4. If tests fail, fix those too: `cargo test --workspace 2>&1 | tail -50`.
5. Run `cargo fmt` on files you touched.

IMPORTANT: Fix only what the errors require. Do NOT re-read the plan or re-implement. End your turn when `cargo check --workspace` is clean.
"#
    )
}
