use anyhow::Result;
use chrono::Utc;
use std::path::Path;
use tracing::{info, instrument, warn};

#[derive(Debug, Clone)]
pub struct GateResult {
    pub passed: bool,
    pub output: String,
    pub test_count: Option<TestCount>,
    /// Structured error digest for feeding back to agents.
    /// Contains only the unique errors with file/line info, not the full output.
    pub error_digest: Option<String>,
}

/// Extract a structured error digest from cargo output.
/// Pulls out unique `error[E...]` blocks with file:line references.
/// This gives agents targeted signal instead of pages of raw compiler output.
pub fn extract_error_digest(output: &str) -> Option<String> {
    let mut errors: Vec<String> = Vec::new();
    let mut current_error: Option<String> = None;
    let mut seen_messages: std::collections::HashSet<String> = std::collections::HashSet::new();

    for line in output.lines() {
        if line.starts_with("error[E") || line.starts_with("error: ") {
            // Save previous error block
            if let Some(err) = current_error.take() {
                if seen_messages.insert(err.lines().next().unwrap_or("").to_string()) {
                    errors.push(err);
                }
            }
            current_error = Some(line.to_string());
        } else if line.starts_with("  --> ") || line.starts_with("   |") {
            // Append context lines to current error
            if let Some(ref mut err) = current_error {
                err.push('\n');
                err.push_str(line);
            }
        } else if line.starts_with("error: could not compile") {
            // Final summary line — skip
        } else if current_error.is_some() && line.trim().is_empty() {
            // End of error block
            if let Some(err) = current_error.take() {
                if seen_messages.insert(err.lines().next().unwrap_or("").to_string()) {
                    errors.push(err);
                }
            }
        }
    }
    // Flush last error
    if let Some(err) = current_error.take() {
        if seen_messages.insert(err.lines().next().unwrap_or("").to_string()) {
            errors.push(err);
        }
    }

    if errors.is_empty() {
        return None;
    }

    let count = errors.len();
    // Cap at 10 unique errors to avoid overwhelming the agent
    let digest = errors.into_iter().take(10).collect::<Vec<_>>().join("\n\n");
    Some(format!("{count} unique error(s):\n\n{digest}"))
}

#[derive(Debug, Clone)]
pub struct TestCount {
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
}

/// Detect which workspace crates have changes (staged + unstaged) by checking
/// git diff for modified .rs files and mapping them to crate names.
/// Returns `-p crate1 -p crate2 ...` args, or `--workspace` if detection fails.
async fn affected_crate_args(repo_root: &Path) -> Vec<String> {
    // Get changed .rs files from git
    let diff_output = tokio::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD", "--", "*.rs"])
        .current_dir(repo_root)
        .output()
        .await;

    let staged_output = tokio::process::Command::new("git")
        .args(["diff", "--name-only", "--cached", "--", "*.rs"])
        .current_dir(repo_root)
        .output()
        .await;

    // Also catch untracked .rs files
    let untracked_output = tokio::process::Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard", "--", "*.rs"])
        .current_dir(repo_root)
        .output()
        .await;

    let mut changed_files: Vec<String> = Vec::new();
    for output in [diff_output, staged_output, untracked_output] {
        if let Ok(o) = output {
            if o.status.success() {
                let text = String::from_utf8_lossy(&o.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !changed_files.contains(&trimmed.to_string()) {
                        changed_files.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    if changed_files.is_empty() {
        // No changes detected — still run workspace check (maybe non-.rs changes)
        info!("No changed .rs files detected, using --workspace");
        return vec!["--workspace".to_string()];
    }

    // Map file paths to crate directories
    let mut crate_dirs: Vec<String> = Vec::new();
    for file in &changed_files {
        // Files are like crates/golem-core/src/lib.rs or apps/mirage-rs/src/main.rs
        let parts: Vec<&str> = file.split('/').collect();
        if parts.len() >= 2 && (parts[0] == "crates" || parts[0] == "apps") {
            let crate_dir = format!("{}/{}", parts[0], parts[1]);
            if !crate_dirs.contains(&crate_dir) {
                crate_dirs.push(crate_dir);
            }
        }
    }

    if crate_dirs.is_empty() {
        info!("Changed files not in crates/ or apps/, using --workspace");
        return vec!["--workspace".to_string()];
    }

    // Read crate names from their Cargo.toml [package] name fields
    let mut crate_names: Vec<String> = Vec::new();
    for dir in &crate_dirs {
        let cargo_toml = repo_root.join(dir).join("Cargo.toml");
        if let Ok(content) = tokio::fs::read_to_string(&cargo_toml).await {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("name") {
                    if let Some(name) = trimmed
                        .split('=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"').to_string())
                    {
                        if !name.is_empty() {
                            crate_names.push(name);
                        }
                    }
                    break;
                }
            }
        }
    }

    if crate_names.is_empty() {
        return vec!["--workspace".to_string()];
    }

    info!(
        "Scoped gates to {} affected crates: {}",
        crate_names.len(),
        crate_names.join(", ")
    );

    let mut args = Vec::new();
    for name in &crate_names {
        args.push("-p".to_string());
        args.push(name.clone());
    }
    args
}

/// Run `cargo fmt --check` and auto-fix if needed
#[instrument(skip_all)]
pub async fn format_gate(repo_root: &Path) -> Result<GateResult> {
    info!("Running format gate: cargo fmt --check");
    let fmt_timeout = std::time::Duration::from_secs(5 * 60);
    let check = tokio::time::timeout(
        fmt_timeout,
        tokio::process::Command::new("cargo")
            .args(["fmt", "--check"])
            .current_dir(repo_root)
            .output(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("format gate timed out after 5min"))?
    .map_err(|e| anyhow::anyhow!("format gate failed: {e}"))?;

    if check.status.success() {
        Ok(GateResult {
            passed: true,
            output: "Already formatted".to_string(),
            test_count: None,
            error_digest: None,
        })
    } else {
        info!("Auto-formatting with cargo fmt");
        let fmt = tokio::time::timeout(
            fmt_timeout,
            tokio::process::Command::new("cargo")
                .arg("fmt")
                .current_dir(repo_root)
                .output(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("format gate (auto-fix) timed out after 5min"))?
        .map_err(|e| anyhow::anyhow!("format gate (auto-fix) failed: {e}"))?;

        let output = if fmt.status.success() {
            "Auto-formatted".to_string()
        } else {
            let stderr = String::from_utf8_lossy(&fmt.stderr);
            format!("Format failed: {stderr}")
        };

        Ok(GateResult {
            passed: true, // always pass -- formatting is auto-fixed
            output,
            test_count: None,
            error_digest: None,
        })
    }
}

/// Run `cargo clippy` as a combined compile+lint gate. Clippy is a superset of
/// `cargo check` — it compiles the code AND runs lints. Running check then clippy
/// separately causes a full rebuild because they use different compiler wrappers,
/// invalidating each other's cache. This single pass eliminates that overhead.
#[instrument(skip_all, fields(plan = %plan))]
pub async fn clippy_compile_gate(repo_root: &Path, plan: &str) -> Result<GateResult> {
    let scope = affected_crate_args(repo_root).await;
    info!(
        "Running combined clippy+compile gate: cargo clippy {}",
        scope.join(" ")
    );

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("clippy");
    cmd.args(&scope);
    cmd.current_dir(repo_root);
    cmd.env("CARGO_INCREMENTAL", "0");

    let output = tokio::time::timeout(std::time::Duration::from_secs(15 * 60), cmd.output())
        .await
        .map_err(|_| anyhow::anyhow!("clippy+compile gate timed out after 15min"))?
        .map_err(|e| anyhow::anyhow!("clippy+compile gate failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    // For the compile gate, we only fail on actual compilation errors, not clippy warnings.
    // Check if there are real errors (not just warnings) by looking at the exit code
    // and whether stderr has "error[E" patterns (compiler errors vs clippy errors).
    let has_compile_errors = !output.status.success() && stderr.contains("error[E");
    let passed = !has_compile_errors;

    if !passed {
        warn!(plan = %plan, "Clippy+compile gate FAILED (compile errors)");
        let error_digest = extract_error_digest(&combined);
        return Ok(GateResult {
            passed: false,
            output: combined,
            test_count: None,
            error_digest,
        });
    } else {
        let has_warnings = stderr.contains("warning:");
        if has_warnings {
            info!("Clippy+compile gate passed with warnings (affected crates)");
        } else {
            info!("Clippy+compile gate clean (affected crates)");
        }
    }

    // Second pass: check that the full workspace compiles with this plan's changes.
    // This catches cross-crate API breaks that the affected-crate check missed.
    info!("Running clippy+compile gate: cargo check --workspace (full workspace validation)");
    let mut ws_cmd = tokio::process::Command::new("cargo");
    ws_cmd.arg("check");
    ws_cmd.arg("--workspace");
    ws_cmd.current_dir(repo_root);
    ws_cmd.env("CARGO_INCREMENTAL", "0");

    let ws_output = tokio::time::timeout(std::time::Duration::from_secs(10 * 60), ws_cmd.output())
        .await
        .map_err(|_| anyhow::anyhow!("workspace clippy+compile gate timed out after 10min"))?
        .map_err(|e| anyhow::anyhow!("workspace clippy+compile gate failed: {e}"))?;

    let ws_stdout = String::from_utf8_lossy(&ws_output.stdout).to_string();
    let ws_stderr = String::from_utf8_lossy(&ws_output.stderr).to_string();
    let ws_combined =
        format!("{stdout}\n{stderr}\n--- Workspace check ---\n{ws_stdout}\n{ws_stderr}");

    let ws_passed = ws_output.status.success();
    if !ws_passed {
        warn!(plan = %plan, "Clippy+compile gate FAILED (workspace check)");
    } else {
        info!(plan = %plan, "Clippy+compile gate passed (workspace check)");
    }

    let error_digest = if !ws_passed {
        extract_error_digest(&ws_combined)
    } else {
        None
    };
    Ok(GateResult {
        passed: ws_passed,
        output: ws_combined,
        test_count: None,
        error_digest,
    })
}

/// Run `cargo check` scoped to affected crates (falls back to --workspace).
/// Used as fallback when clippy is disabled.
#[instrument(skip_all, fields(plan = %plan))]
pub async fn compile_gate(repo_root: &Path, plan: &str) -> Result<GateResult> {
    let scope = affected_crate_args(repo_root).await;
    info!("Running compile gate: cargo check {}", scope.join(" "));

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("check");
    cmd.args(&scope);
    cmd.current_dir(repo_root);
    // Disable incremental so sccache can cache compilations across branches
    cmd.env("CARGO_INCREMENTAL", "0");

    let output = tokio::time::timeout(std::time::Duration::from_secs(15 * 60), cmd.output())
        .await
        .map_err(|_| anyhow::anyhow!("compile gate timed out after 15min"))?
        .map_err(|e| anyhow::anyhow!("compile gate failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    let passed = output.status.success();
    if !passed {
        warn!(plan = %plan, "Compile gate FAILED");
        let error_digest = extract_error_digest(&combined);
        return Ok(GateResult {
            passed: false,
            output: combined,
            test_count: None,
            error_digest,
        });
    } else {
        info!(plan = %plan, "Compile gate passed (affected crates)");
    }

    // Second pass: check that the full workspace compiles with this plan's changes.
    // This catches cross-crate API breaks that the affected-crate check missed.
    info!("Running compile gate: cargo check --workspace (full workspace validation)");
    let mut ws_cmd = tokio::process::Command::new("cargo");
    ws_cmd.arg("check");
    ws_cmd.arg("--workspace");
    ws_cmd.current_dir(repo_root);
    ws_cmd.env("CARGO_INCREMENTAL", "0");

    let ws_output = tokio::time::timeout(std::time::Duration::from_secs(10 * 60), ws_cmd.output())
        .await
        .map_err(|_| anyhow::anyhow!("workspace compile gate timed out after 10min"))?
        .map_err(|e| anyhow::anyhow!("workspace compile gate failed: {e}"))?;

    let ws_stdout = String::from_utf8_lossy(&ws_output.stdout).to_string();
    let ws_stderr = String::from_utf8_lossy(&ws_output.stderr).to_string();
    let ws_combined =
        format!("{stdout}\n{stderr}\n--- Workspace check ---\n{ws_stdout}\n{ws_stderr}");

    let ws_passed = ws_output.status.success();
    if !ws_passed {
        warn!(plan = %plan, "Compile gate FAILED (workspace check)");
    } else {
        info!(plan = %plan, "Compile gate passed (workspace check)");
    }

    let error_digest = if !ws_passed {
        extract_error_digest(&ws_combined)
    } else {
        None
    };
    Ok(GateResult {
        passed: ws_passed,
        output: ws_combined,
        test_count: None,
        error_digest,
    })
}

/// Run `cargo clippy` scoped to affected crates (non-blocking, always passes)
#[instrument(skip_all)]
pub async fn clippy_gate(repo_root: &Path) -> Result<GateResult> {
    let scope = affected_crate_args(repo_root).await;
    info!("Running clippy gate: cargo clippy {}", scope.join(" "));

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("clippy");
    cmd.args(&scope);
    cmd.current_dir(repo_root);
    cmd.env("CARGO_INCREMENTAL", "0");

    let output = cmd.output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    // Clippy is non-blocking: always passes, but reports warnings
    let has_warnings = stderr.contains("warning:");
    if has_warnings {
        info!("Clippy found warnings (non-blocking)");
    } else {
        info!("Clippy clean");
    }

    Ok(GateResult {
        passed: true,
        output: combined,
        test_count: None,
        error_digest: None,
    })
}

/// Query sccache stats and return formatted hit rate string
pub async fn sccache_stats() -> Option<String> {
    let output = tokio::process::Command::new("sccache")
        .arg("--show-stats")
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut hits: Option<u64> = None;
    let mut misses: Option<u64> = None;

    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("cache hit") {
            hits = extract_stat_number(line);
        } else if lower.contains("cache miss") {
            misses = extract_stat_number(line);
        }
    }

    match (hits, misses) {
        (Some(h), Some(m)) if h + m > 0 => {
            let total = h + m;
            let pct = (h * 100) / total;
            Some(format!("sccache: {}% ({}/{})", pct, h, total))
        }
        _ => None,
    }
}

fn extract_stat_number(line: &str) -> Option<u64> {
    // Find rightmost number in the line
    line.split_whitespace().rev().find_map(|word| {
        word.trim_matches(|c: char| !c.is_ascii_digit())
            .parse()
            .ok()
    })
}

/// Run cargo tests scoped to affected crates
#[instrument(skip_all, fields(timeout_secs))]
pub async fn test_gate(repo_root: &Path, timeout_secs: u64) -> Result<GateResult> {
    let scope = affected_crate_args(repo_root).await;
    info!("Running test gate with scope: {}", scope.join(" "));

    // Prefer nextest if available
    let use_nextest = tokio::process::Command::new("cargo")
        .args(["nextest", "--version"])
        .current_dir(repo_root)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    let output = if use_nextest {
        info!("Using cargo nextest");
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("nextest").arg("run");
        cmd.args(&scope);
        cmd.current_dir(repo_root);
        cmd.env("CARGO_INCREMENTAL", "0");
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output()).await
    } else {
        info!("Using cargo test");
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.arg("test");
        cmd.args(&scope);
        cmd.current_dir(repo_root);
        cmd.env("CARGO_INCREMENTAL", "0");
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output()).await
    };

    match output {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = format!("{stdout}\n{stderr}");

            let test_count = parse_test_counts(&combined);
            let passed = output.status.success();

            if !passed {
                warn!("Test gate FAILED");
            } else {
                info!("Test gate passed");
            }

            let error_digest = if !passed {
                extract_error_digest(&combined)
            } else {
                None
            };
            Ok(GateResult {
                passed,
                output: combined,
                test_count: Some(test_count),
                error_digest,
            })
        }
        Ok(Err(e)) => Ok(GateResult {
            passed: false,
            output: format!("Failed to run tests: {e}"),
            test_count: None,
            error_digest: None,
        }),
        Err(_) => Ok(GateResult {
            passed: false,
            output: format!("Test gate timed out after {timeout_secs}s"),
            test_count: None,
            error_digest: None,
        }),
    }
}

/// Parse test counts from cargo test output
/// Public accessor for test counts parsing. Returns (passed, failed).
pub fn parse_test_counts_pub(output: &str) -> (u32, u32) {
    let tc = parse_test_counts(output);
    (tc.passed, tc.failed)
}

fn parse_test_counts(output: &str) -> TestCount {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut ignored = 0u32;

    for line in output.lines() {
        if line.contains("test result:") {
            if let Some(p) = extract_count(line, "passed") {
                passed += p;
            }
            if let Some(f) = extract_count(line, "failed") {
                failed += f;
            }
            if let Some(i) = extract_count(line, "ignored") {
                ignored += i;
            }
        }
    }

    TestCount {
        passed,
        failed,
        ignored,
    }
}

fn extract_count(line: &str, label: &str) -> Option<u32> {
    let idx = line.find(label)?;
    let before = &line[..idx].trim_end();
    let num_str: String = before
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let num_str: String = num_str.chars().rev().collect();
    num_str.parse().ok()
}

/// Extract failing test names from cargo test output.
/// Parses lines like `test foo::bar::test_name ... FAILED` and returns the full paths.
pub fn extract_failing_test_names(output: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        // cargo test format: "test path::to::test_name ... FAILED"
        if trimmed.starts_with("test ") && trimmed.ends_with("FAILED") {
            if let Some(name) = trimmed
                .strip_prefix("test ")
                .and_then(|s| s.split(" ... ").next())
            {
                failures.push(name.trim().to_string());
            }
        }
        // nextest format: "FAIL [   0.123s] crate-name path::to::test_name"
        if trimmed.starts_with("FAIL ") {
            let parts: Vec<&str> = trimmed.splitn(4, ' ').collect();
            if parts.len() >= 4 {
                failures.push(parts[3].trim().to_string());
            }
        }
    }
    failures
}

/// Extract a short error snippet (first ~50 lines of failures section) from cargo test output.
pub fn extract_test_failure_snippet(output: &str, max_lines: usize) -> String {
    let mut snippet_lines = Vec::new();
    let mut in_failures = false;

    for line in output.lines() {
        if line.starts_with("failures:") || line.starts_with("---- ") && line.contains("FAILED") {
            in_failures = true;
        }
        if in_failures {
            snippet_lines.push(line);
            if snippet_lines.len() >= max_lines {
                snippet_lines.push("... (truncated)");
                break;
            }
        }
        // Stop at the test result summary if we've captured enough
        if in_failures && line.starts_with("test result:") {
            break;
        }
    }

    if snippet_lines.is_empty() {
        // Fallback: grab the last 30 lines of output
        let all_lines: Vec<&str> = output.lines().collect();
        let start = all_lines.len().saturating_sub(max_lines);
        snippet_lines = all_lines[start..].to_vec();
    }

    snippet_lines.join("\n")
}

/// Extract invariant test function names from a plan's ## Verification section.
/// Looks for `**test_fn**: \`name\`` patterns.
pub fn extract_invariant_test_names(plan_content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in plan_content.lines() {
        if line.contains("**test_fn**:") {
            if let Some(name) = line.split('`').nth(1) {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// Run only the invariant test functions specified in a plan's ## Verification section.
/// Returns pass if no invariant tests are specified (plan has no verification section).
pub async fn invariant_gate(repo_root: &Path, plan_content: &str) -> Result<GateResult> {
    let test_fns = extract_invariant_test_names(plan_content);
    if test_fns.is_empty() {
        return Ok(GateResult {
            passed: true,
            output: "No invariant tests specified".into(),
            test_count: None,
            error_digest: None,
        });
    }

    let scope = affected_crate_args(repo_root).await;
    // Build a regex filter matching any of the test function names
    let filter = test_fns.join("|");

    info!("Running invariant gate: {} test functions", test_fns.len());

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("test").args(&scope).args(["--", "-E"]).arg(&filter);
    cmd.current_dir(repo_root);
    cmd.env("CARGO_INCREMENTAL", "0");

    let output = cmd.output().await?;
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let passed = output.status.success();
    let test_count = parse_test_counts(&combined);
    let error_digest = if !passed {
        extract_error_digest(&combined)
    } else {
        None
    };

    if !passed {
        warn!("Invariant gate FAILED ({} test fns)", test_fns.len());
    } else {
        info!("Invariant gate passed ({} test fns)", test_fns.len());
    }

    Ok(GateResult {
        passed,
        output: combined,
        test_count: Some(test_count),
        error_digest,
    })
}

/// Spawn the headless terminal, hit the RPC health endpoint, capture a snapshot,
/// verify basic sanity (non-empty buffer, valid active screen), then shut it down.
#[instrument(skip_all)]
pub async fn terminal_render_gate(repo_root: &Path) -> Result<GateResult> {
    info!("Running terminal render gate: headless terminal + RPC health");

    // Build first so startup is fast
    let build = tokio::process::Command::new("cargo")
        .args(["build", "-p", "bardo-terminal"])
        .current_dir(repo_root)
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .await?;

    if !build.status.success() {
        let stderr = String::from_utf8_lossy(&build.stderr).to_string();
        return Ok(GateResult {
            passed: false,
            output: format!("Terminal build failed:\n{stderr}"),
            test_count: None,
            error_digest: extract_error_digest(&stderr),
        });
    }

    // Spawn headless terminal
    let mut child = tokio::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "bardo-terminal",
            "--",
            "--headless",
            "--width",
            "80",
            "--height",
            "24",
            "--rpc-port",
            "9100",
        ])
        .current_dir(repo_root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Poll RPC health endpoint with retries
    let client = reqwest::Client::new();
    let mut responsive = false;
    let mut active_screen = String::new();
    let mut snapshot_text = String::new();
    let mut failures: Vec<String> = Vec::new();

    for attempt in 0..10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let health_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "terminal.health",
            "id": attempt + 1,
        });
        match client
            .post("http://127.0.0.1:9100")
            .json(&health_req)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(result) = body.get("result") {
                        responsive = true;
                        active_screen = result
                            .get("active_screen")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        break;
                    }
                }
            }
            _ => continue,
        }
    }

    if !responsive {
        failures.push("Terminal did not respond to health check after 5s".to_string());
    }

    // Capture snapshot
    if responsive {
        let snap_req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "terminal.getSnapshot",
            "id": 100,
        });
        if let Ok(resp) = client
            .post("http://127.0.0.1:9100")
            .json(&snap_req)
            .send()
            .await
        {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(result) = body.get("result") {
                    snapshot_text = result
                        .get("buffer_text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if snapshot_text.trim().is_empty() {
                        failures.push("Snapshot buffer is empty".to_string());
                    }
                    if snapshot_text.contains("panic") || snapshot_text.contains("PANIC") {
                        failures.push("Snapshot contains panic text".to_string());
                    }
                }
            }
        }
    }

    // Shutdown
    let shutdown_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "terminal.shutdown",
        "id": 200,
    });
    let _ = client
        .post("http://127.0.0.1:9100")
        .json(&shutdown_req)
        .send()
        .await;
    // Give it a moment to exit, then force-kill
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let _ = child.kill().await;

    let passed = failures.is_empty();
    let assertions_total = 3u32; // responsive, non-empty buffer, no panics
    let assertions_passed = assertions_total.saturating_sub(failures.len() as u32);

    let output = if passed {
        format!("Terminal render gate passed: screen={active_screen}, buffer={}B, {assertions_passed}/{assertions_total} assertions", snapshot_text.len())
    } else {
        format!(
            "Terminal render gate FAILED ({assertions_passed}/{assertions_total}): {}",
            failures.join("; ")
        )
    };

    if !passed {
        warn!("Terminal render gate FAILED: {}", failures.join("; "));
    } else {
        info!("Terminal render gate passed: screen={}", active_screen);
    }

    Ok(GateResult {
        passed,
        output,
        test_count: None,
        error_digest: if passed {
            None
        } else {
            Some(failures.join("\n"))
        },
    })
}

/// Run golem lifecycle tests via `cargo test -p golem-core -- test_golem`.
#[instrument(skip_all)]
pub async fn golem_lifecycle_gate(repo_root: &Path) -> Result<GateResult> {
    info!("Running golem lifecycle gate: cargo test -p golem-core -- test_golem");

    let output = tokio::process::Command::new("cargo")
        .args(["test", "-p", "golem-core", "--", "test_golem"])
        .current_dir(repo_root)
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    let test_count = parse_test_counts(&combined);
    let passed = output.status.success();

    if !passed {
        warn!("Golem lifecycle gate FAILED");
    } else {
        info!("Golem lifecycle gate passed ({} tests)", test_count.passed);
    }

    let error_digest = if !passed {
        extract_error_digest(&combined)
    } else {
        None
    };

    Ok(GateResult {
        passed,
        output: combined,
        test_count: Some(test_count),
        error_digest,
    })
}

/// Detect which crates a plan touches by reading its frontmatter `crates_touched` field.
pub fn plan_touches_crate(plan_content: &str, crate_prefix: &str) -> bool {
    for line in plan_content.lines() {
        if line.starts_with("crates_touched:") || line.starts_with("crates_touched :") {
            let value = line.splitn(2, ':').nth(1).unwrap_or("");
            return value.contains(crate_prefix);
        }
    }
    false
}

/// Quick `cargo check --workspace` on the batch branch after a merge.
/// Catches integration issues (API mismatches across plans) before the next plan starts.
/// Runs with CARGO_INCREMENTAL=0 so sccache handles caching across branches.
#[instrument(skip_all)]
pub async fn post_merge_compile_check(repo_root: &Path) -> Result<GateResult> {
    info!("Running post-merge compile check: cargo check --workspace");

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(["check", "--workspace"]);
    cmd.current_dir(repo_root);
    cmd.env("CARGO_INCREMENTAL", "0");

    let output = cmd.output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{stdout}\n{stderr}");

    let passed = output.status.success();
    if passed {
        info!("Post-merge compile check passed");
    } else {
        warn!("Post-merge compile check FAILED");
    }

    let error_digest = if !passed {
        extract_error_digest(&combined)
    } else {
        None
    };
    Ok(GateResult {
        passed,
        output: combined,
        test_count: None,
        error_digest,
    })
}

// ---------------------------------------------------------------------------
// Ignored test enforcement
// ---------------------------------------------------------------------------

/// An entry from the ignored-tests.md ledger.
#[derive(Debug, Clone)]
pub struct IgnoredTestEntry {
    pub test_name: String,
    pub crate_name: String,
    pub reason: String,
    pub unblocked_by_plan: String,
}

/// Parse the ignored-tests.md markdown table into structured entries.
pub fn parse_ignored_tests_ledger(content: &str) -> Vec<IgnoredTestEntry> {
    let mut entries = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("| Test")
            || trimmed.starts_with("|---")
            || trimmed.starts_with("|-")
        {
            continue;
        }
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed.split('|').collect();
        // "|col1|col2|col3|col4|" splits to ["", "col1", "col2", "col3", "col4", ""]
        if cols.len() >= 5 {
            let test_name = cols[1].trim().to_string();
            let crate_name = cols[2].trim().to_string();
            let reason = cols[3].trim().to_string();
            let unblocked_by = cols[4].trim().to_string();
            if !test_name.is_empty() {
                entries.push(IgnoredTestEntry {
                    test_name,
                    crate_name,
                    reason,
                    unblocked_by_plan: unblocked_by,
                });
            }
        }
    }
    entries
}

/// Check if an ignored test entry matches a plan number.
/// Plan number "06" matches "06-terminal-navigation" or just "06".
pub fn entry_matches_plan(entry: &IgnoredTestEntry, plan_number: &str) -> bool {
    let plan_field = entry.unblocked_by_plan.trim();
    plan_field == plan_number || plan_field.starts_with(&format!("{plan_number}-"))
}

/// After a plan passes tests, check ignored-tests.md for entries that
/// this plan should unblock. Run each matching test with --include-ignored
/// and fail if any still fail.
pub async fn ignored_test_gate(
    repo_root: &Path,
    plan_number: &str,
    ignored_tests_path: &Path,
) -> Result<GateResult> {
    info!(
        "Running ignored test enforcement gate for plan {}",
        plan_number
    );

    let content = match tokio::fs::read_to_string(ignored_tests_path).await {
        Ok(c) => c,
        Err(_) => {
            info!("No ignored-tests.md found, passing gate");
            return Ok(GateResult {
                passed: true,
                output: "No ignored-tests.md ledger found".to_string(),
                test_count: None,
                error_digest: None,
            });
        }
    };

    let entries = parse_ignored_tests_ledger(&content);
    let matching: Vec<&IgnoredTestEntry> = entries
        .iter()
        .filter(|e| entry_matches_plan(e, plan_number))
        .collect();

    if matching.is_empty() {
        return Ok(GateResult {
            passed: true,
            output: format!("No ignored tests unblocked by plan {plan_number}"),
            test_count: None,
            error_digest: None,
        });
    }

    info!(
        "Found {} ignored tests that plan {} should unblock",
        matching.len(),
        plan_number
    );

    let mut failures: Vec<String> = Vec::new();

    for entry in &matching {
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            tokio::process::Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    &entry.crate_name,
                    "--",
                    &entry.test_name,
                    "--include-ignored",
                ])
                .current_dir(repo_root)
                .env("CARGO_INCREMENTAL", "0")
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) if !output.status.success() => {
                failures.push(format!(
                    "{}::{} (crate {})",
                    entry.crate_name, entry.test_name, entry.crate_name
                ));
            }
            Ok(Err(e)) => {
                failures.push(format!(
                    "{}::{} (failed to run: {})",
                    entry.crate_name, entry.test_name, e
                ));
            }
            Err(_) => {
                failures.push(format!(
                    "{}::{} (timed out after 120s)",
                    entry.crate_name, entry.test_name
                ));
            }
            _ => {
                info!("Ignored test {} now passes", entry.test_name);
            }
        }
    }

    if failures.is_empty() {
        Ok(GateResult {
            passed: true,
            output: format!(
                "All {} ignored tests now pass for plan {plan_number}",
                matching.len()
            ),
            test_count: None,
            error_digest: None,
        })
    } else {
        let digest = format!(
            "Plan {plan_number} should unblock these tests per ignored-tests.md but they still fail:\n{}",
            failures.join("\n")
        );
        warn!("Ignored test gate FAILED: {}", digest);
        Ok(GateResult {
            passed: false,
            output: digest.clone(),
            test_count: None,
            error_digest: Some(digest),
        })
    }
}

// ---------------------------------------------------------------------------
// Cargo deny gate
// ---------------------------------------------------------------------------

/// Run `cargo deny check` to catch GPL dependencies and known vulnerabilities.
/// Hard gate -- blocks merge.
pub async fn deny_gate(repo_root: &Path) -> Result<GateResult> {
    info!("Running dependency deny gate: cargo deny check");

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        tokio::process::Command::new("cargo")
            .args(["deny", "check"])
            .current_dir(repo_root)
            .output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = format!("{stdout}\n{stderr}");
            let passed = output.status.success();

            if passed {
                info!("Deny gate passed");
            } else {
                warn!("Deny gate FAILED");
            }

            let error_digest = if !passed {
                let truncated: String = combined.chars().take(2000).collect();
                Some(truncated)
            } else {
                None
            };

            Ok(GateResult {
                passed,
                output: combined,
                test_count: None,
                error_digest,
            })
        }
        Ok(Err(e)) => Ok(GateResult {
            passed: false,
            output: format!("Failed to run cargo deny: {e}"),
            test_count: None,
            error_digest: Some(format!("Failed to run cargo deny: {e}")),
        }),
        Err(_) => Ok(GateResult {
            passed: false,
            output: "cargo deny timed out after 120s".to_string(),
            test_count: None,
            error_digest: Some("cargo deny timed out after 120s".to_string()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Post-merge regression gate
// ---------------------------------------------------------------------------

/// Run full workspace tests after a plan merges to catch cross-plan regressions.
/// Unlike test_gate (which scopes to affected crates), this runs --workspace.
pub async fn post_merge_regression_gate(repo_root: &Path, timeout_secs: u64) -> Result<GateResult> {
    info!("Running post-merge regression gate: workspace-wide tests");

    // Prefer nextest if available
    let use_nextest = tokio::process::Command::new("cargo")
        .args(["nextest", "--version"])
        .current_dir(repo_root)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    let output = if use_nextest {
        info!("Using cargo nextest for regression gate");
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.args(["nextest", "run", "--workspace"]);
        cmd.current_dir(repo_root);
        cmd.env("CARGO_INCREMENTAL", "0");
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output()).await
    } else {
        info!("Using cargo test for regression gate");
        let mut cmd = tokio::process::Command::new("cargo");
        cmd.args(["test", "--workspace"]);
        cmd.current_dir(repo_root);
        cmd.env("CARGO_INCREMENTAL", "0");
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output()).await
    };

    match output {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = format!("{stdout}\n{stderr}");

            let test_count = parse_test_counts(&combined);
            let passed = output.status.success();

            if passed {
                info!(
                    "Post-merge regression gate passed ({} tests)",
                    test_count.passed
                );
            } else {
                warn!(
                    "Post-merge regression gate FAILED ({} failures)",
                    test_count.failed
                );
            }

            let error_digest = if !passed {
                extract_error_digest(&combined)
            } else {
                None
            };
            Ok(GateResult {
                passed,
                output: combined,
                test_count: Some(test_count),
                error_digest,
            })
        }
        Ok(Err(e)) => Ok(GateResult {
            passed: false,
            output: format!("Failed to run regression tests: {e}"),
            test_count: None,
            error_digest: None,
        }),
        Err(_) => Ok(GateResult {
            passed: false,
            output: format!("Regression tests timed out after {timeout_secs}s"),
            test_count: None,
            error_digest: None,
        }),
    }
}

/// Read the implementer's self-check TOML and fast-fail if compilation failed.
/// Returns pass if the selfcheck file is missing (backward compat).
pub async fn selfcheck_gate(repo_root: &Path, plan_num: &str) -> Result<GateResult> {
    let path = repo_root.join(format!(
        "plans/context/completion/{plan_num}-selfcheck.toml"
    ));
    if !path.exists() {
        return Ok(GateResult {
            passed: true,
            output: "No selfcheck file (backward compat)".into(),
            test_count: None,
            error_digest: None,
        });
    }

    let content = tokio::fs::read_to_string(&path).await?;

    // Parse the TOML
    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            return Ok(GateResult {
                passed: true,
                output: "Selfcheck TOML parse error (skipping)".into(),
                test_count: None,
                error_digest: None,
            })
        }
    };
    let selfcheck = parsed.get("selfcheck");

    let compilation = selfcheck
        .and_then(|s| s.get("compilation"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let tests = selfcheck
        .and_then(|s| s.get("tests"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let exports = selfcheck
        .and_then(|s| s.get("exports_verified"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let errors: Vec<String> = selfcheck
        .and_then(|s| s.get("errors"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let passed = compilation && tests;
    let mut output_parts = Vec::new();
    if !compilation {
        output_parts.push("Self-check: compilation FAILED".to_string());
    }
    if !tests {
        output_parts.push("Self-check: tests FAILED".to_string());
    }
    if !exports {
        output_parts.push("Self-check: missing exports detected".to_string());
    }
    for err in &errors {
        output_parts.push(format!("Self-check error: {err}"));
    }

    let output = if output_parts.is_empty() {
        "Self-check: all checks passed".to_string()
    } else {
        output_parts.join("\n")
    };

    Ok(GateResult {
        passed,
        output,
        test_count: None,
        error_digest: if !passed {
            Some(errors.join("\n"))
        } else {
            None
        },
    })
}

/// Results from running all gates in parallel.
#[derive(Debug, Clone)]
pub struct AllGateResults {
    pub compile: GateResult,
    pub test: GateResult,
    pub format: GateResult,
    pub dep_deny: Option<GateResult>,
    /// True if all gates passed.
    pub all_passed: bool,
    /// Combined output from all gates.
    pub combined_output: String,
    /// If compile failed, the other results may be unreliable.
    pub compile_failed_early: bool,
}

/// Run compile, test, and format gates concurrently.
/// If compile fails, test results are marked unreliable (compile_failed_early).
/// This replaces the sequential 5-phase gate chain, cutting gate time from
/// 3-5 min to 1-2 min.
pub async fn run_all_gates(repo_root: &Path, test_timeout_secs: u64) -> Result<AllGateResults> {
    info!("Running all gates in parallel");

    let (compile_result, test_result, format_result) = tokio::join!(
        clippy_compile_gate(repo_root, ""),
        test_gate(repo_root, test_timeout_secs),
        format_gate(repo_root),
    );

    let compile = compile_result?;
    let test = test_result?;
    let format = format_result?;

    let compile_failed_early = !compile.passed;
    let all_passed = compile.passed && test.passed && format.passed;

    let mut combined_parts = Vec::new();
    if !compile.passed {
        combined_parts.push(format!("=== COMPILE GATE FAILED ===\n{}", compile.output));
    }
    if !test.passed {
        combined_parts.push(format!("=== TEST GATE FAILED ===\n{}", test.output));
    }
    if !format.passed {
        combined_parts.push(format!("=== FORMAT GATE FAILED ===\n{}", format.output));
    }
    if all_passed {
        combined_parts.push("All gates passed.".to_string());
    }

    let combined_output = combined_parts.join("\n\n");

    Ok(AllGateResults {
        compile,
        test,
        format,
        dep_deny: None,
        all_passed,
        combined_output,
        compile_failed_early,
    })
}

/// Run all gates including dependency deny check.
pub async fn run_all_gates_with_deny(
    repo_root: &Path,
    test_timeout_secs: u64,
) -> Result<AllGateResults> {
    info!("Running all gates in parallel (with dep-deny)");

    let (compile_result, test_result, format_result, deny_result) = tokio::join!(
        clippy_compile_gate(repo_root, ""),
        test_gate(repo_root, test_timeout_secs),
        format_gate(repo_root),
        deny_gate(repo_root),
    );

    let compile = compile_result?;
    let test = test_result?;
    let format = format_result?;
    let deny = deny_result?;

    let compile_failed_early = !compile.passed;
    let all_passed = compile.passed && test.passed && format.passed && deny.passed;

    let mut combined_parts = Vec::new();
    if !compile.passed {
        combined_parts.push(format!("=== COMPILE GATE FAILED ===\n{}", compile.output));
    }
    if !test.passed {
        combined_parts.push(format!("=== TEST GATE FAILED ===\n{}", test.output));
    }
    if !format.passed {
        combined_parts.push(format!("=== FORMAT GATE FAILED ===\n{}", format.output));
    }
    if !deny.passed {
        combined_parts.push(format!("=== DEP DENY FAILED ===\n{}", deny.output));
    }
    if all_passed {
        combined_parts.push("All gates passed.".to_string());
    }

    Ok(AllGateResults {
        compile,
        test,
        format,
        dep_deny: Some(deny),
        all_passed,
        combined_output: combined_parts.join("\n\n"),
        compile_failed_early,
    })
}

/// Write a markdown summary of gate results to `plans/context/gate-results/{plan_num}-gate.md`.
pub fn persist_gate_result(
    repo_root: &Path,
    plan_num: &str,
    compile_passed: bool,
    test_passed: bool,
) -> Result<()> {
    let dir = repo_root.join("plans/context/gate-results");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{plan_num}-gate.md"));

    let status = |passed: bool| if passed { "PASS" } else { "FAIL" };
    let passed_count = [compile_passed, test_passed].iter().filter(|&&p| p).count();
    let failed_count = 2 - passed_count;

    let content = format!(
        "## Gate Summary -- Plan {plan_num}\nTimestamp: {}\n\n\
         CompileGate: {}\n\
         TestGate: {}\n\n\
         Passed: {passed_count}  Failed: {failed_count}\n",
        Utc::now().to_rfc3339(),
        status(compile_passed),
        status(test_passed),
    );

    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persist_gate_result_creates_file() {
        let root =
            std::env::temp_dir().join(format!("bardo-test-gate-persist-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        persist_gate_result(&root, "03", true, false).unwrap();

        let path = root.join("plans/context/gate-results/03-gate.md");
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("CompileGate: PASS"));
        assert!(content.contains("TestGate: FAIL"));
        assert!(content.contains("Passed: 1  Failed: 1"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn parse_empty_ledger() {
        let content = "# Ignored Test Ledger\n\n| Test Name | Crate | Reason | Unblocked By Plan |\n|-----------|-------|--------|-------------------|\n";
        let entries = parse_ignored_tests_ledger(content);
        assert!(entries.is_empty());
    }

    #[test]
    fn parse_ledger_with_entries() {
        let content = "\
# Ignored Test Ledger

| Test Name | Crate | Reason | Unblocked By Plan |
|-----------|-------|--------|-------------------|
| test_render_screen | bardo-terminal | needs RPC server | 06-terminal-navigation |
| test_golem_snapshot | golem-core | snapshot format | 07-golem-snapshots |
";
        let entries = parse_ignored_tests_ledger(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].test_name, "test_render_screen");
        assert_eq!(entries[0].crate_name, "bardo-terminal");
        assert_eq!(entries[0].unblocked_by_plan, "06-terminal-navigation");
        assert_eq!(entries[1].test_name, "test_golem_snapshot");
    }

    #[test]
    fn entry_matches_plan_number() {
        let entry = IgnoredTestEntry {
            test_name: "test_foo".into(),
            crate_name: "my-crate".into(),
            reason: "needs feature".into(),
            unblocked_by_plan: "06-terminal-navigation".into(),
        };
        assert!(entry_matches_plan(&entry, "06"));
        assert!(!entry_matches_plan(&entry, "07"));
        assert!(!entry_matches_plan(&entry, "0"));
        assert!(!entry_matches_plan(&entry, "6"));
    }

    #[test]
    fn entry_matches_bare_number() {
        let entry = IgnoredTestEntry {
            test_name: "test_foo".into(),
            crate_name: "my-crate".into(),
            reason: "needs feature".into(),
            unblocked_by_plan: "06".into(),
        };
        assert!(entry_matches_plan(&entry, "06"));
        assert!(!entry_matches_plan(&entry, "07"));
    }

    #[test]
    fn extract_error_digest_returns_none_for_clean() {
        assert!(extract_error_digest("Compiling foo v0.1.0\nFinished").is_none());
    }

    #[test]
    fn extract_error_digest_captures_errors() {
        let output =
            "error[E0432]: unresolved import\n  --> src/lib.rs:1:5\n\nerror: could not compile";
        let digest = extract_error_digest(output);
        assert!(digest.is_some());
        assert!(digest.unwrap().contains("E0432"));
    }
}
