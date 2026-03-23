//! Enrichment orchestration.
//!
//! Runs enrichment steps against plan directories, generating briefs, task
//! TOMLs, verification tasks, review checklists, and more. Steps can use
//! direct LLM calls (claude CLI), the bardo-gateway real-time API, or the
//! gateway batch API for 50% cost savings.

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::batch_client::{self, BatchClient};
use crate::direct_client;
use crate::prompts;

/// Configuration for an enrichment run.
pub struct EnrichContext {
    /// Project root directory.
    pub root: PathBuf,
    /// Gateway URL (if using gateway or batch mode).
    pub gateway_url: Option<String>,
    /// Gateway API key.
    pub gateway_key: Option<String>,
    /// Use batch API instead of real-time.
    pub batch_mode: bool,
    /// Override the default model for all steps.
    pub model_override: Option<String>,
    /// Regenerate even if output file already exists.
    pub force: bool,
    /// Print what would be done without doing it.
    pub dry_run: bool,
}

/// An individual enrichment step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnrichStep {
    /// Generate PRD context extract.
    Prd,
    /// Generate implementation brief.
    Briefs,
    /// Generate tasks.toml.
    Tasks,
    /// Generate step-by-step decomposition.
    Decompose,
    /// Generate verify-tasks.toml.
    Verify,
    /// Generate review-tasks.toml.
    Review,
    /// Generate testing backlog.
    Tests,
    /// Generate review rubric / invariants.
    Invariants,
    /// Generate scribe-tasks.toml.
    Scribe,
}

impl EnrichStep {
    /// Default model for this step. Heavier steps use Sonnet, lighter ones Haiku.
    pub fn default_model(self) -> &'static str {
        match self {
            Self::Decompose | Self::Verify | Self::Review | Self::Tests | Self::Scribe => {
                "claude-sonnet-4-6"
            }
            Self::Briefs | Self::Tasks | Self::Prd | Self::Invariants => {
                "claude-haiku-4-5-20251001"
            }
        }
    }

    /// Whether this step requires an LLM call.
    /// Some steps can do pure extraction without calling a model.
    pub fn needs_llm(self) -> bool {
        match self {
            Self::Briefs | Self::Tasks | Self::Prd => false,
            Self::Verify
            | Self::Review
            | Self::Decompose
            | Self::Tests
            | Self::Invariants
            | Self::Scribe => true,
        }
    }

    /// Output filename within the plan directory.
    pub fn output_filename(self) -> &'static str {
        match self {
            Self::Prd => "prd-extract.md",
            Self::Briefs => "brief.md",
            Self::Tasks => "tasks.toml",
            Self::Decompose => "decomposition.md",
            Self::Verify => "verify-tasks.toml",
            Self::Review => "review-tasks.toml",
            Self::Tests => "testing-backlog.md",
            Self::Invariants => "rubric.md",
            Self::Scribe => "scribe-tasks.toml",
        }
    }

    /// All steps in dependency order.
    pub fn all_ordered() -> &'static [Self] {
        &[
            Self::Prd,
            Self::Briefs,
            Self::Tasks,
            Self::Decompose,
            Self::Verify,
            Self::Review,
            Self::Tests,
            Self::Invariants,
            Self::Scribe,
        ]
    }
}

impl fmt::Display for EnrichStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Prd => "prd",
            Self::Briefs => "briefs",
            Self::Tasks => "tasks",
            Self::Decompose => "decompose",
            Self::Verify => "verify",
            Self::Review => "review",
            Self::Tests => "tests",
            Self::Invariants => "invariants",
            Self::Scribe => "scribe",
        };
        write!(f, "{name}")
    }
}

/// Collected input text for a step.
struct StepInputs {
    plan_content: String,
    tasks_content: Option<String>,
    brief_content: Option<String>,
    decomposition_content: Option<String>,
}

/// Run a single enrichment step for a plan.
pub async fn run_step(ctx: &EnrichContext, step: EnrichStep, plan_base: &str) -> Result<()> {
    let plan_dir = find_plan_dir(&ctx.root, plan_base)?;
    let output_file = plan_dir.join(step.output_filename());

    // Skip if output exists and --force not set.
    if output_file.exists() && !ctx.force {
        if ctx.dry_run {
            println!("  skip: {} (exists)", output_file.display());
        } else {
            println!(
                "  skip: {} (exists, use --force to regenerate)",
                output_file.display()
            );
        }
        return Ok(());
    }

    if ctx.dry_run {
        println!("  would generate: {}", output_file.display());
        return Ok(());
    }

    // Read inputs.
    let inputs = read_step_inputs(&plan_dir, step)?;

    // For non-LLM steps, generate via extraction.
    if !step.needs_llm() {
        let result = generate_without_llm(step, &inputs)?;
        std::fs::write(&output_file, &result)
            .with_context(|| format!("failed to write {}", output_file.display()))?;
        println!("  created: {} (extracted)", output_file.display());
        return Ok(());
    }

    // Build prompt.
    let (system, user_msg) = build_prompt(step, &inputs);

    // Call LLM.
    let model = ctx
        .model_override
        .as_deref()
        .unwrap_or(step.default_model());
    let result = call_llm(ctx, model, &system, &user_msg).await?;

    // Strip markdown fences if the LLM wrapped the output.
    let cleaned = strip_fences(&result, step);

    std::fs::write(&output_file, &cleaned)
        .with_context(|| format!("failed to write {}", output_file.display()))?;
    println!("  created: {}", output_file.display());
    Ok(())
}

/// Run all enrichment steps for a plan in dependency order.
pub async fn run_all(ctx: &EnrichContext, plan_base: &str) -> Result<()> {
    println!("Enriching plan: {plan_base}");
    for step in EnrichStep::all_ordered() {
        if let Err(e) = run_step(ctx, *step, plan_base).await {
            eprintln!("  error in {step}: {e}");
        }
    }
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────

/// Find the plan directory. Checks `.mori/plans/{base}*/` then `plans/{base}*/`.
pub fn find_plan_dir(root: &Path, plan_base: &str) -> Result<PathBuf> {
    let search_dirs = [root.join(".mori/plans"), root.join("plans")];

    for search in &search_dirs {
        if !search.is_dir() {
            continue;
        }
        let entries = std::fs::read_dir(search)
            .with_context(|| format!("failed to read {}", search.display()))?;

        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(plan_base) && entry.path().join("plan.md").exists() {
                return Ok(entry.path());
            }
        }
    }

    bail!(
        "plan directory for '{plan_base}' not found under .mori/plans/ or plans/ in {}",
        root.display()
    );
}

/// Read the input files needed for a given step.
fn read_step_inputs(plan_dir: &Path, step: EnrichStep) -> Result<StepInputs> {
    let plan_path = plan_dir.join("plan.md");
    let plan_content = read_optional_file(&plan_path)?
        .ok_or_else(|| anyhow::anyhow!("plan.md not found in {}", plan_dir.display()))?;

    let tasks_content = match step {
        EnrichStep::Verify | EnrichStep::Tests => read_optional_file(&plan_dir.join("tasks.toml"))?,
        _ => None,
    };

    let brief_content = match step {
        EnrichStep::Decompose => read_optional_file(&plan_dir.join("brief.md"))?,
        _ => None,
    };

    let decomposition_content = match step {
        EnrichStep::Briefs => read_optional_file(&plan_dir.join("decomposition.md"))?,
        _ => None,
    };

    Ok(StepInputs {
        plan_content,
        tasks_content,
        brief_content,
        decomposition_content,
    })
}

/// Read a file, returning `None` if it does not exist. Logs a warning on other errors.
fn read_optional_file(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => {
            eprintln!("  warning: could not read {}: {e}", path.display());
            Ok(None)
        }
    }
}

/// Build the (system, user) prompt pair for a step.
fn build_prompt(step: EnrichStep, inputs: &StepInputs) -> (String, String) {
    match step {
        EnrichStep::Briefs => (
            prompts::BRIEF_SYSTEM.to_string(),
            prompts::brief_user(
                &inputs.plan_content,
                inputs.decomposition_content.as_deref(),
            ),
        ),
        EnrichStep::Tasks => (
            prompts::TASKS_SYSTEM.to_string(),
            prompts::tasks_user(&inputs.plan_content),
        ),
        EnrichStep::Verify => (
            prompts::VERIFY_SYSTEM.to_string(),
            prompts::verify_user(&inputs.plan_content, inputs.tasks_content.as_deref()),
        ),
        EnrichStep::Review => (
            prompts::REVIEW_SYSTEM.to_string(),
            prompts::review_user(&inputs.plan_content),
        ),
        EnrichStep::Prd => (
            prompts::PRD_SYSTEM.to_string(),
            prompts::prd_user(&inputs.plan_content, &[]),
        ),
        EnrichStep::Decompose => (
            prompts::DECOMPOSE_SYSTEM.to_string(),
            prompts::decompose_user(&inputs.plan_content, inputs.brief_content.as_deref()),
        ),
        EnrichStep::Tests => (
            prompts::TESTS_SYSTEM.to_string(),
            prompts::tests_user(&inputs.plan_content, inputs.tasks_content.as_deref()),
        ),
        EnrichStep::Invariants => (
            prompts::INVARIANTS_SYSTEM.to_string(),
            prompts::invariants_user(&inputs.plan_content),
        ),
        EnrichStep::Scribe => (
            prompts::SCRIBE_SYSTEM.to_string(),
            prompts::scribe_user(&inputs.plan_content),
        ),
    }
}

/// Dispatch an LLM call to the configured backend.
async fn call_llm(
    ctx: &EnrichContext,
    model: &str,
    system: &str,
    user_msg: &str,
) -> Result<String> {
    if let Some(ref url) = ctx.gateway_url {
        let key = ctx.gateway_key.as_deref().unwrap_or("");

        if ctx.batch_mode {
            let client = BatchClient::new(url, key);
            let item_id = client.submit(model, system, user_msg, 8192).await?;
            println!("    batch submitted: {item_id}, waiting...");
            client.wait_for_result(&item_id, None).await
        } else {
            batch_client::call_gateway(url, key, model, system, user_msg, 8192).await
        }
    } else {
        direct_client::call_claude(model, system, user_msg, 8192).await
    }
}

/// Generate content for non-LLM steps via pure extraction.
fn generate_without_llm(step: EnrichStep, inputs: &StepInputs) -> Result<String> {
    match step {
        EnrichStep::Briefs => extract_brief(&inputs.plan_content),
        EnrichStep::Tasks => extract_tasks(&inputs.plan_content),
        EnrichStep::Prd => extract_prd_refs(&inputs.plan_content),
        _ => bail!("step {step} requires an LLM call"),
    }
}

/// Extract a brief from the plan via section parsing (no LLM).
fn extract_brief(plan: &str) -> Result<String> {
    let mut brief = String::new();
    let plan_name = extract_plan_name(plan);

    brief.push_str(&format!(
        "# Implementation brief: {plan_name}\n\
         > Machine-generated by `mori-mcp enrich briefs` (extraction mode).\n\n"
    ));

    // Extract sections by heading.
    let sections = [
        ("Prerequisites", "## Dependencies"),
        ("Imports", "## Imports"),
        ("Exports", "## Exports"),
    ];

    for (src_heading, dst_heading) in &sections {
        if let Some(content) = extract_section(plan, src_heading) {
            brief.push_str(&format!("{dst_heading}\n\n{content}\n\n"));
        }
    }

    // Add a placeholder for execution order.
    brief.push_str(
        "## Execution Order\n\n\
         See decomposition.md for step-by-step instructions.\n\n\
         ## Verification Checklist\n\n\
         - [ ] Build passes\n\
         - [ ] Tests pass\n\
         - [ ] Exports contract satisfied\n",
    );

    Ok(brief)
}

/// Extract tasks from plan ## Unit headings (no LLM).
fn extract_tasks(plan: &str) -> Result<String> {
    let plan_name = extract_plan_name(plan);
    let mut toml = String::new();
    let mut tasks: Vec<(String, Vec<String>)> = Vec::new();

    // Look for ## Unit headings or numbered ## headings.
    let mut current_title: Option<String> = None;
    let mut current_files: Vec<String> = Vec::new();

    for line in plan.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // Save previous task.
            if let Some(title) = current_title.take() {
                tasks.push((title, std::mem::take(&mut current_files)));
            }
            // Skip non-implementation headings.
            let skip = [
                "Context",
                "Previous",
                "Prerequisites",
                "Imports",
                "Exports",
                "Quick Reference",
                "Invariants",
            ];
            if !skip.iter().any(|s| heading.starts_with(s)) {
                current_title = Some(heading.to_string());
            }
        } else if current_title.is_some() {
            // Look for file paths in backticks.
            for segment in line.split('`') {
                let trimmed = segment.trim();
                if (trimmed.contains('/') || trimmed.contains('.'))
                    && !trimmed.contains(' ')
                    && trimmed.len() < 120
                    && !trimmed.starts_with("http")
                {
                    current_files.push(trimmed.to_string());
                }
            }
        }
    }
    if let Some(title) = current_title {
        tasks.push((title, current_files));
    }

    // Build TOML.
    let total = tasks.len();
    toml.push_str(&format!(
        "[meta]\nplan = \"{plan_name}\"\niteration = 1\ntotal = {total}\ndone = 0\nmax_parallel = 3\nestimated_total_minutes = {}\n",
        total * 10
    ));

    for (i, (title, files)) in tasks.iter().enumerate() {
        let id = format!("T{}", i + 1);
        let files_str: String = files
            .iter()
            .map(|f| format!("  \"{f}\""))
            .collect::<Vec<_>>()
            .join(",\n");
        toml.push_str(&format!(
            "\n[[task]]\nid = \"{id}\"\ntitle = \"{title}\"\nstatus = \"pending\"\nfiles = [\n{files_str}\n]\nacceptance = [\"implementation matches plan\"]\ndepends_on = []\nestimated_seconds = 600\n"
        ));
    }

    Ok(toml)
}

/// Extract PRD references from plan content (no LLM).
fn extract_prd_refs(plan: &str) -> Result<String> {
    let plan_name = extract_plan_name(plan);
    let mut output = format!(
        "# PRD Context for {plan_name}\n\
         # Auto-generated by mori-mcp enrich prd (extraction mode)\n\n"
    );

    // Find references to prd2/ paths.
    let mut refs: Vec<String> = Vec::new();
    for line in plan.lines() {
        // Match patterns like `prd2/...` or prd2/...
        for segment in line.split(|c: char| c == '`' || c == '(' || c == ')' || c == ' ') {
            let trimmed = segment.trim();
            if trimmed.starts_with("prd2/") && trimmed.len() > 6 {
                let clean = trimmed.trim_end_matches(|c: char| {
                    !c.is_alphanumeric() && c != '.' && c != '/' && c != '-'
                });
                if !refs.contains(&clean.to_string()) {
                    refs.push(clean.to_string());
                }
            }
        }
    }

    if refs.is_empty() {
        output.push_str("No PRD references found in plan.\n");
    } else {
        output.push_str(&format!("# Sources: {} inline refs\n\n", refs.len()));
        for r in &refs {
            output.push_str(&format!("- `{r}`\n"));
        }
        output.push_str(
            "\nNote: Full PRD content extraction requires access to the PRD files.\n\
             Run with --force and an LLM to generate full extracts.\n",
        );
    }

    Ok(output)
}

/// Extract the plan name from the first heading.
fn extract_plan_name(plan: &str) -> String {
    for line in plan.lines() {
        if let Some(heading) = line.strip_prefix("# ") {
            return heading.trim().to_string();
        }
    }
    "unknown-plan".to_string()
}

/// Extract content under a ## heading.
fn extract_section(plan: &str, heading: &str) -> Option<String> {
    let mut found = false;
    let mut content = String::new();

    for line in plan.lines() {
        if found {
            // Stop at next ## heading or --- separator.
            if line.starts_with("## ") || line.starts_with("---") {
                break;
            }
            content.push_str(line);
            content.push('\n');
        } else if let Some(h) = line.strip_prefix("## ") {
            if h.trim().starts_with(heading) {
                found = true;
            }
        }
    }

    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Strip markdown code fences that the LLM might wrap around TOML or markdown output.
fn strip_fences(content: &str, step: EnrichStep) -> String {
    let trimmed = content.trim();

    // For TOML outputs, strip ```toml ... ``` fences.
    let is_toml = matches!(
        step,
        EnrichStep::Tasks | EnrichStep::Verify | EnrichStep::Review | EnrichStep::Scribe
    );

    if is_toml {
        if let Some(rest) = trimmed.strip_prefix("```toml") {
            if let Some(inner) = rest.strip_suffix("```") {
                return inner.trim().to_string();
            }
        }
    }

    // For markdown outputs, strip ```markdown ... ``` fences.
    if let Some(rest) = trimmed.strip_prefix("```markdown") {
        if let Some(inner) = rest.strip_suffix("```") {
            return inner.trim().to_string();
        }
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_plan_dir_in_mori_plans() {
        // This test depends on the actual filesystem. We test the logic path
        // by checking it returns an error for a nonexistent plan.
        let result = find_plan_dir(Path::new("/nonexistent"), "test-plan");
        assert!(result.is_err());
    }

    #[test]
    fn extract_plan_name_from_heading() {
        let plan = "# Plan 01: Workspace Scaffold\n\nSome content.";
        assert_eq!(extract_plan_name(plan), "Plan 01: Workspace Scaffold");
    }

    #[test]
    fn extract_plan_name_default() {
        assert_eq!(extract_plan_name("no heading here"), "unknown-plan");
    }

    #[test]
    fn extract_section_finds_content() {
        let plan = "## Context\nSome context.\n\n## Prerequisites\nNone.\n\n---\n\n## Exports\n";
        let result = extract_section(plan, "Prerequisites");
        assert_eq!(result, Some("None.".to_string()));
    }

    #[test]
    fn extract_section_returns_none_for_missing() {
        let plan = "## Context\nSome context.\n";
        assert!(extract_section(plan, "Nonexistent").is_none());
    }

    #[test]
    fn strip_toml_fences() {
        let input = "```toml\n[meta]\nplan = \"test\"\n```";
        let result = strip_fences(input, EnrichStep::Tasks);
        assert_eq!(result, "[meta]\nplan = \"test\"");
    }

    #[test]
    fn strip_fences_leaves_clean_content() {
        let input = "[meta]\nplan = \"test\"";
        let result = strip_fences(input, EnrichStep::Tasks);
        assert_eq!(result, input);
    }

    #[test]
    fn enrich_step_display() {
        assert_eq!(format!("{}", EnrichStep::Briefs), "briefs");
        assert_eq!(format!("{}", EnrichStep::Verify), "verify");
    }

    #[test]
    fn all_ordered_has_nine_steps() {
        assert_eq!(EnrichStep::all_ordered().len(), 9);
    }

    #[test]
    fn extract_brief_produces_output() {
        let plan = "# Test Plan\n\n## Context\nTest.\n\n## Prerequisites\nNone.\n\n---\n";
        let result = extract_brief(plan);
        assert!(result.is_ok());
        let brief = result.expect("extract_brief should succeed");
        assert!(brief.contains("Test Plan"));
    }

    #[test]
    fn extract_prd_refs_finds_refs() {
        let plan = "# Test\n\nSee `prd2/01-golem/01-cognition.md` for details.\n";
        let result = extract_prd_refs(plan);
        assert!(result.is_ok());
        let output = result.expect("extract_prd_refs should succeed");
        assert!(output.contains("prd2/01-golem/01-cognition.md"));
    }
}
