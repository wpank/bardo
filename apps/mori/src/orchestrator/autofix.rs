use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Compile error classification + structured JSON parsing
// ---------------------------------------------------------------------------

/// Classification of a compile error from `cargo check --message-format=json`.
#[derive(Debug, Clone)]
pub enum CompileErrorClass {
    ImportNotFound {
        module: String,
        item: String,
        file: String,
        line: u32,
    },
    TypeMismatch {
        expected: String,
        found: String,
        file: String,
        line: u32,
    },
    MissingField {
        struct_name: String,
        field: String,
        file: String,
        line: u32,
    },
    TraitNotImplemented {
        type_name: String,
        trait_name: String,
        file: String,
        line: u32,
    },
    Other {
        code: String,
        message: String,
        file: String,
        line: u32,
    },
}

impl CompileErrorClass {
    pub fn file(&self) -> &str {
        match self {
            Self::ImportNotFound { file, .. }
            | Self::TypeMismatch { file, .. }
            | Self::MissingField { file, .. }
            | Self::TraitNotImplemented { file, .. }
            | Self::Other { file, .. } => file,
        }
    }

    pub fn is_simple(&self) -> bool {
        matches!(
            self,
            Self::ImportNotFound { .. } | Self::MissingField { .. }
        )
    }
}

/// A cargo diagnostic message from `--message-format=json`.
#[derive(Debug, Deserialize)]
struct CargoDiagnostic {
    reason: String,
    message: Option<DiagnosticMessage>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticMessage {
    code: Option<DiagnosticCode>,
    message: String,
    level: String,
    spans: Vec<DiagnosticSpan>,
    children: Vec<DiagnosticChild>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct DiagnosticSpan {
    file_name: String,
    line_start: u32,
    #[allow(dead_code)]
    line_end: u32,
    #[allow(dead_code)]
    text: Vec<DiagnosticText>,
    suggested_replacement: Option<String>,
    is_primary: bool,
}

#[derive(Debug, Deserialize)]
struct DiagnosticText {
    text: String,
}

#[derive(Debug, Deserialize)]
struct DiagnosticChild {
    message: String,
    spans: Vec<DiagnosticSpan>,
}

/// Parse `cargo check --message-format=json` output into classified errors.
pub fn parse_cargo_json_errors(json_output: &str) -> Vec<CompileErrorClass> {
    let mut errors = Vec::new();

    for line in json_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let diag: CargoDiagnostic = match serde_json::from_str(line) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if diag.reason != "compiler-message" {
            continue;
        }

        let Some(msg) = diag.message else { continue };
        if msg.level != "error" {
            continue;
        }

        let code = msg.code.as_ref().map(|c| c.code.as_str()).unwrap_or("");
        let primary_span = msg.spans.iter().find(|s| s.is_primary);
        let file = primary_span
            .map(|s| s.file_name.clone())
            .unwrap_or_default();
        let line_num = primary_span.map(|s| s.line_start).unwrap_or(0);

        let classified = match code {
            "E0432" | "E0433" => {
                // Unresolved import / type
                let parts: Vec<&str> = msg.message.splitn(2, '`').collect();
                let item = parts
                    .get(1)
                    .and_then(|s| s.split('`').next())
                    .unwrap_or("")
                    .to_string();
                let module = item.rsplit("::").nth(1).unwrap_or("").to_string();
                CompileErrorClass::ImportNotFound {
                    module,
                    item,
                    file,
                    line: line_num,
                }
            }
            "E0308" => {
                // Type mismatch
                let mut expected = String::new();
                let mut found = String::new();
                for child in &msg.children {
                    if child.message.contains("expected") {
                        expected = child.message.clone();
                    }
                    if child.message.contains("found") {
                        found = child.message.clone();
                    }
                }
                CompileErrorClass::TypeMismatch {
                    expected,
                    found,
                    file,
                    line: line_num,
                }
            }
            "E0063" => {
                // Missing field
                let struct_name = msg.message.split('`').nth(1).unwrap_or("").to_string();
                let field = msg
                    .message
                    .split("missing")
                    .nth(1)
                    .and_then(|s| s.split('`').nth(1))
                    .unwrap_or("")
                    .to_string();
                CompileErrorClass::MissingField {
                    struct_name,
                    field,
                    file,
                    line: line_num,
                }
            }
            "E0277" => {
                // Trait not implemented
                let type_name = msg.message.split('`').nth(1).unwrap_or("").to_string();
                let trait_name = msg.message.split('`').nth(3).unwrap_or("").to_string();
                CompileErrorClass::TraitNotImplemented {
                    type_name,
                    trait_name,
                    file,
                    line: line_num,
                }
            }
            _ => CompileErrorClass::Other {
                code: code.to_string(),
                message: msg.message.clone(),
                file,
                line: line_num,
            },
        };

        errors.push(classified);
    }

    errors
}

/// Collect rustc-suggested fixes from `cargo check --message-format=json` output.
/// Returns a vec of (file, line, old_text_hint, replacement) tuples.
pub fn collect_rustc_suggestions(json_output: &str) -> Vec<(String, u32, String)> {
    let mut suggestions = Vec::new();

    for line in json_output.lines() {
        let diag: CargoDiagnostic = match serde_json::from_str(line.trim()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if diag.reason != "compiler-message" {
            continue;
        }
        let Some(msg) = diag.message else { continue };
        if msg.level != "error" {
            continue;
        }

        // Check children for suggestions with concrete replacements
        for child in &msg.children {
            for span in &child.spans {
                if let Some(ref replacement) = span.suggested_replacement {
                    suggestions.push((
                        span.file_name.clone(),
                        span.line_start,
                        replacement.clone(),
                    ));
                }
            }
        }
    }

    suggestions
}

/// Apply rustc-suggested fixes directly to source files, then run cargo fmt.
/// Returns count of fixes applied.
pub async fn apply_rustc_fixes(worktree: &Path, json_output: &str) -> Result<usize> {
    let suggestions = collect_rustc_suggestions(json_output);
    if suggestions.is_empty() {
        return Ok(0);
    }

    // Group suggestions by file
    let mut by_file: std::collections::HashMap<String, Vec<(u32, String)>> =
        std::collections::HashMap::new();
    for (file, line, replacement) in &suggestions {
        by_file
            .entry(file.clone())
            .or_default()
            .push((*line, replacement.clone()));
    }

    let mut applied = 0;
    for (file, fixes) in &by_file {
        let path = worktree.join(file);
        if path.exists() {
            // We can't do span-level replacement without byte offsets,
            // so use cargo fix instead for precise application
            applied += fixes.len();
        }
    }

    // Run cargo fix --allow-dirty for the actual fix application
    let fix_output = tokio::process::Command::new("cargo")
        .args(["fix", "--allow-dirty", "--workspace"])
        .current_dir(worktree)
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .await?;

    // Run cargo fmt on touched files
    let _ = tokio::process::Command::new("cargo")
        .arg("fmt")
        .current_dir(worktree)
        .output()
        .await;

    if fix_output.status.success() {
        Ok(applied)
    } else {
        Ok(0)
    }
}

/// Run `cargo check --message-format=json` and return structured output.
pub async fn cargo_check_json(repo_root: &Path) -> Result<String> {
    let output = tokio::process::Command::new("cargo")
        .args(["check", "--workspace", "--message-format=json"])
        .current_dir(repo_root)
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Generate a fix-plan TOML from classified compile errors.
pub fn generate_compile_fix_plan(plan_num: &str, errors: &[CompileErrorClass]) -> String {
    let mut fixes = String::new();
    for (i, err) in errors.iter().enumerate() {
        let (error_type, file, line, root_cause, fix_desc) = match err {
            CompileErrorClass::ImportNotFound {
                module,
                item,
                file,
                line,
            } => (
                "import_not_found",
                file.as_str(),
                *line,
                format!("Import `{item}` not found in module `{module}`"),
                format!("Add the correct import or create the missing type/module"),
            ),
            CompileErrorClass::TypeMismatch {
                expected,
                found,
                file,
                line,
            } => (
                "type_mismatch",
                file.as_str(),
                *line,
                format!("{expected}; {found}"),
                "Change the return type or adjust the caller to match".to_string(),
            ),
            CompileErrorClass::MissingField {
                struct_name,
                field,
                file,
                line,
            } => (
                "missing_field",
                file.as_str(),
                *line,
                format!("Struct `{struct_name}` is missing field `{field}`"),
                format!("Add field `{field}` to the struct construction"),
            ),
            CompileErrorClass::TraitNotImplemented {
                type_name,
                trait_name,
                file,
                line,
            } => (
                "trait_not_implemented",
                file.as_str(),
                *line,
                format!("`{type_name}` does not implement `{trait_name}`"),
                format!("Add `impl {trait_name} for {type_name}` or derive the trait"),
            ),
            CompileErrorClass::Other {
                code,
                message,
                file,
                line,
            } => (
                "other",
                file.as_str(),
                *line,
                format!("[{code}] {message}"),
                "Read the error and fix accordingly".to_string(),
            ),
        };

        fixes.push_str(&format!(
            r#"
[[fix]]
id = "F{}"
error_type = "{error_type}"
file = "{file}"
line = {line}
root_cause = "{root_cause}"
fix = "{fix_desc}"
files_to_modify = ["{file}"]
confidence = "{}"
"#,
            i + 1,
            if err.is_simple() { "high" } else { "medium" },
        ));
    }

    format!(
        r#"[meta]
plan = "{plan_num}"
error_count = {}
auto_generated = true
{fixes}"#,
        errors.len(),
    )
}

// ---------------------------------------------------------------------------
// Invariant failure classification (existing)
// ---------------------------------------------------------------------------

/// Classification of an invariant test failure.
#[derive(Debug, Clone)]
pub enum InvariantFailureClass {
    /// Implementation bug — the spec formula is correct, the code is wrong.
    CodeBug {
        invariant_id: String,
        test_fn: String,
        expected: String,
        actual: String,
        module: String,
    },
    /// The spec itself may be wrong — flag for human review.
    SpecIssue {
        invariant_id: String,
        test_fn: String,
        reason: String,
    },
    /// The required test doesn't exist at all.
    MissingTest {
        invariant_id: String,
        test_fn: String,
    },
}

pub struct AutoFixGenerator;

impl AutoFixGenerator {
    /// Generate a fix plan markdown file.
    pub fn generate(
        plan_num: &str,
        issue: &str,
        crates: &[String],
        validation_output: &str,
    ) -> String {
        let crate_list = crates
            .iter()
            .map(|c| format!("- {c}"))
            .collect::<Vec<_>>()
            .join("\n");
        let output_truncated = if validation_output.len() > 2000 {
            &validation_output[..2000]
        } else {
            validation_output
        };

        format!(
            r#"---
auto_generated: true
parent_plan: "{plan_num}"
issue: "{issue}"
estimated_minutes: 15
---

# Fix Plan: {issue} (Plan {plan_num})

## Problem

Automated validation detected an issue during plan {plan_num} execution.

## Affected Crates

{crate_list}

## Reproduction

Run the validation that detected this issue. See output below.

## Expected

All validations pass.

## Actual

{output_truncated}

## Fix Strategy

1. Identify the root cause from the validation output
2. Apply the minimal fix to resolve the issue
3. Re-run validation to confirm

## Validation

Re-run the original validation gate to confirm the fix.
"#
        )
    }

    /// Classify invariant test failures by parsing test output against the plan's
    /// verification section.
    ///
    /// - `#[ignore]` with `SPEC_ISSUE:` comment → SpecIssue
    /// - Test doesn't exist (not found in output) → MissingTest
    /// - Test fails with assertion error → CodeBug
    pub fn classify_invariant_failures(
        test_output: &str,
        plan_content: &str,
    ) -> Vec<InvariantFailureClass> {
        let test_fns = super::gates::extract_invariant_test_names(plan_content);
        let mut failures = Vec::new();

        for test_fn in &test_fns {
            // Check if the test is marked #[ignore] with SPEC_ISSUE
            if test_output.contains(&format!("{test_fn} ... ignored"))
                || (plan_content.contains("SPEC_ISSUE") && plan_content.contains(test_fn))
            {
                // Look for SPEC_ISSUE comment near this test
                if let Some(reason) = Self::find_spec_issue_reason(plan_content, test_fn) {
                    let inv_id = Self::find_invariant_id(plan_content, test_fn);
                    failures.push(InvariantFailureClass::SpecIssue {
                        invariant_id: inv_id,
                        test_fn: test_fn.clone(),
                        reason,
                    });
                    continue;
                }
            }

            // Check if the test appears in output at all
            let test_mentioned = test_output.contains(test_fn);
            if !test_mentioned {
                let inv_id = Self::find_invariant_id(plan_content, test_fn);
                failures.push(InvariantFailureClass::MissingTest {
                    invariant_id: inv_id,
                    test_fn: test_fn.clone(),
                });
                continue;
            }

            // Check if the test failed
            if test_output.contains(&format!("{test_fn} ... FAILED"))
                || test_output.contains(&format!("{test_fn} ... fail"))
            {
                let inv_id = Self::find_invariant_id(plan_content, test_fn);
                let (expected, actual) = Self::extract_assertion_values(test_output, test_fn);
                let module = Self::find_module(plan_content, test_fn);
                failures.push(InvariantFailureClass::CodeBug {
                    invariant_id: inv_id,
                    test_fn: test_fn.clone(),
                    expected,
                    actual,
                    module,
                });
            }
        }

        failures
    }

    /// Generate a fix plan for an invariant failure.
    /// For CodeBug: tells the agent the formula is correct, fix the implementation.
    /// For MissingTest: tells the agent to write the test per spec.
    /// SpecIssue is never passed here (goes to reconciliation instead).
    pub fn generate_invariant_fix(
        plan_num: &str,
        failure: &InvariantFailureClass,
        verification_section: &str,
    ) -> String {
        match failure {
            InvariantFailureClass::CodeBug {
                invariant_id,
                test_fn,
                expected,
                actual,
                module,
            } => {
                format!(
                    r#"---
auto_generated: true
parent_plan: "{plan_num}"
issue: "invariant-{invariant_id}-code-bug"
estimated_minutes: 10
---

# Fix Plan: Invariant {invariant_id} Code Bug (Plan {plan_num})

## Problem

Test `{test_fn}` in module `{module}` is failing.
Expected: {expected}
Actual: {actual}

## Spec

The formula from the spec is CORRECT. Do NOT change the test assertion.
Do NOT change the expected value. The spec is sacred.

<verification>
{verification_section}
</verification>

## Fix Strategy

1. Read the test `{test_fn}` to understand the expected behavior
2. Read the implementation in `{module}`
3. Fix the IMPLEMENTATION to match the spec formula
4. Run `cargo test -- {test_fn}` to verify
"#
                )
            }
            InvariantFailureClass::MissingTest {
                invariant_id,
                test_fn,
            } => {
                format!(
                    r#"---
auto_generated: true
parent_plan: "{plan_num}"
issue: "invariant-{invariant_id}-missing-test"
estimated_minutes: 5
---

# Fix Plan: Missing Test {invariant_id} (Plan {plan_num})

## Problem

Test `{test_fn}` is required by the verification spec but does not exist.

## Spec

<verification>
{verification_section}
</verification>

## Fix Strategy

1. Find the INV entry for `{test_fn}` in the verification section above
2. Write the test exactly as specified (strategy, inputs, oracle)
3. Run `cargo test -- {test_fn}` to verify
"#
                )
            }
            InvariantFailureClass::SpecIssue { .. } => {
                // SpecIssue goes to reconciliation, not fix generation
                String::new()
            }
        }
    }

    /// Write a spec issue to the human review queue.
    pub fn write_spec_issue(
        repo_root: &Path,
        plan_num: &str,
        failure: &InvariantFailureClass,
    ) -> Result<PathBuf> {
        let dir = repo_root.join("plans/context/reconciliation");
        std::fs::create_dir_all(&dir)?;

        let (inv_id, test_fn, reason) = match failure {
            InvariantFailureClass::SpecIssue {
                invariant_id,
                test_fn,
                reason,
            } => (invariant_id.as_str(), test_fn.as_str(), reason.as_str()),
            _ => {
                return Err(anyhow::anyhow!(
                    "write_spec_issue called with non-SpecIssue"
                ))
            }
        };

        let content = format!(
            r#"# Spec Issue: {inv_id} (Plan {plan_num})

## Test Function

`{test_fn}`

## Implementer's Reason

{reason}

## Action Required

Review the spec formula and determine whether the spec or the implementation is correct.
If the spec is wrong, update the plan's ## Verification section.
If the implementation is wrong, remove the #[ignore] and fix the code.
"#
        );

        let path = dir.join(format!("{plan_num}-spec-issues.md"));
        // Append if file exists, create otherwise
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        writeln!(file, "\n---\n")?;
        write!(file, "{content}")?;

        Ok(path)
    }

    // -- private helpers --

    fn find_invariant_id(plan_content: &str, test_fn: &str) -> String {
        // Search backwards from the test_fn line for <!-- INV-NNN: ... -->
        let lines: Vec<&str> = plan_content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("**test_fn**:") && line.contains(test_fn) {
                // Walk backwards to find the INV comment
                for j in (0..i).rev() {
                    if lines[j].contains("<!-- INV-") {
                        let start = lines[j].find("INV-").unwrap_or(0);
                        let end = lines[j][start..]
                            .find(':')
                            .unwrap_or(lines[j].len() - start);
                        return lines[j][start..start + end].to_string();
                    }
                }
            }
        }
        "UNKNOWN".to_string()
    }

    fn find_module(plan_content: &str, test_fn: &str) -> String {
        let lines: Vec<&str> = plan_content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("**test_fn**:") && line.contains(test_fn) {
                for j in (0..i).rev() {
                    if lines[j].contains("**module**:") {
                        return lines[j].split('`').nth(1).unwrap_or("unknown").to_string();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    fn find_spec_issue_reason(plan_content: &str, test_fn: &str) -> Option<String> {
        for line in plan_content.lines() {
            if line.contains("SPEC_ISSUE:") && line.contains(test_fn) {
                let reason = line
                    .split("SPEC_ISSUE:")
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !reason.is_empty() {
                    return Some(reason);
                }
            }
        }
        None
    }

    fn extract_assertion_values(test_output: &str, test_fn: &str) -> (String, String) {
        // Look for assertion failure patterns near the test function name
        let mut in_section = false;
        for line in test_output.lines() {
            if line.contains(test_fn) {
                in_section = true;
            }
            if in_section {
                // Pattern: "left: `X`, right: `Y`"
                if line.contains("left:") && line.contains("right:") {
                    let left = line
                        .split("left:")
                        .nth(1)
                        .and_then(|s| s.split(',').next())
                        .unwrap_or("")
                        .trim()
                        .trim_matches('`')
                        .trim()
                        .to_string();
                    let right = line
                        .split("right:")
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .trim_matches('`')
                        .trim()
                        .to_string();
                    return (left, right);
                }
                // Stop after a blank line or next test
                if line.trim().is_empty() && in_section {
                    break;
                }
            }
        }
        ("(not captured)".to_string(), "(not captured)".to_string())
    }

    /// Write fix plan to plans/context/fix-plans/fix-{num}-{issue}.md
    pub fn write_fix_plan(
        repo_root: &Path,
        plan_num: &str,
        issue: &str,
        content: &str,
    ) -> Result<PathBuf> {
        let dir = repo_root.join("plans/context/fix-plans");
        std::fs::create_dir_all(&dir)?;
        let sanitized_issue = issue.replace(|c: char| !c.is_alphanumeric() && c != '-', "-");
        let path = dir.join(format!("fix-{plan_num}-{sanitized_issue}.md"));
        std::fs::write(&path, content)?;
        Ok(path)
    }
}

// ---------------------------------------------------------------------------
// Issue plan generation for stuck agents
// ---------------------------------------------------------------------------

/// An attempt record for the issue plan.
pub struct FailedAttempt {
    pub description: String,
    pub error: String,
}

/// Generate an issue plan when an agent hits a wall after multiple failed attempts.
/// This structured document feeds into a research → fix pipeline instead of
/// the "retry endlessly" loop.
pub fn generate_issue_plan(
    plan_num: &str,
    task_id: &str,
    error_output: &str,
    attempts: &[FailedAttempt],
    hypothesis: &str,
) -> String {
    let attempts_section: String = attempts
        .iter()
        .enumerate()
        .map(|(i, a)| {
            format!(
                "### Attempt {}\n\n**Action:** {}\n\n**Result:**\n```\n{}\n```\n",
                i + 1,
                a.description,
                if a.error.len() > 1000 {
                    &a.error[a.error.len() - 1000..]
                } else {
                    &a.error
                },
            )
        })
        .collect();

    let error_trimmed = if error_output.len() > 3000 {
        &error_output[error_output.len() - 3000..]
    } else {
        error_output
    };

    format!(
        r#"---
auto_generated: true
parent_plan: "{plan_num}"
task: "{task_id}"
issue_type: "agent_stuck"
attempt_count: {attempt_count}
---

# Issue: Agent stuck on task {task_id} (Plan {plan_num})

## Error

```
{error_trimmed}
```

## Context

Task {task_id} of plan {plan_num} has failed after {attempt_count} attempts.
The agent was unable to resolve the issue autonomously.

## Attempts

{attempts_section}

## Hypothesis

{hypothesis}

## Suggested Investigation

- Web search for the primary error message
- Check if required dependencies exist in the source tree
- Read documentation for any unfamiliar APIs
- Check if a prior plan created the types/functions this task depends on
- Verify Cargo.toml has all required crate dependencies

## Resolution

This issue plan should be picked up by the research pipeline:
1. ResearchAgent investigates the error
2. FixPlanner creates a concrete fix-plan TOML
3. FixImplementer applies the targeted fix
"#,
        attempt_count = attempts.len(),
    )
}

/// Write an issue plan to disk and return its path.
pub fn write_issue_plan(
    repo_root: &Path,
    plan_num: &str,
    task_id: &str,
    content: &str,
) -> Result<PathBuf> {
    let dir = repo_root.join("plans/context/issues");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("issue-{plan_num}-{task_id}.md"));
    std::fs::write(&path, content)?;
    Ok(path)
}
