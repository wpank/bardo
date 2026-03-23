use std::path::{Path, PathBuf};

use tracing::{info, warn};

use super::iteration_memory::{IterationEntry, IterationMemory};

/// Maximum characters of gate output to send for reflection.
const MAX_GATE_OUTPUT_CHARS: usize = 8000;

/// Generate a verbal reflection from a gate failure using haiku-4-5.
///
/// Returns the reflection text, or None if the API call fails (non-fatal).
/// Cost: ~$0.01 per call.
async fn generate_reflection(
    gate_output: &str,
    plan_base: &str,
    iteration: u32,
    files_changed: &[String],
) -> Option<String> {
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            warn!("reflection: ANTHROPIC_API_KEY not set, skipping");
            return None;
        }
    };

    let truncated = if gate_output.len() > MAX_GATE_OUTPUT_CHARS {
        let mut start = gate_output.len() - MAX_GATE_OUTPUT_CHARS;
        while !gate_output.is_char_boundary(start) {
            start += 1;
        }
        &gate_output[start..]
    } else {
        gate_output
    };

    let files_list = if files_changed.is_empty() {
        "(unknown)".to_string()
    } else {
        files_changed.join(", ")
    };

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 512,
        "system": concat!(
            "You are a build failure analyst. Given a gate failure output, produce a concise reflection.\n\n",
            "Format:\n\n",
            "## What failed\n<one sentence>\n\n",
            "## Why it failed\n<root cause analysis, 2-3 sentences>\n\n",
            "## What to try differently\n<concrete action items, 2-3 bullets>\n\n",
            "## Files/functions to focus on\n<specific file:function references from the error output>\n\n",
            "Be specific and actionable. Reference exact error codes, file paths, and line numbers from the output."
        ),
        "messages": [{
            "role": "user",
            "content": format!(
                "Plan: {plan_base} (iteration {iteration})\nFiles changed: {files_list}\n\nGate failure output:\n```\n{truncated}\n```"
            )
        }]
    });

    let client = reqwest::Client::new();
    let resp = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("reflection: API call failed: {e}");
            return None;
        }
    };

    if !resp.status().is_success() {
        warn!("reflection: API returned {}", resp.status());
        return None;
    }

    let parsed: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            warn!("reflection: failed to parse response: {e}");
            return None;
        }
    };

    let text = parsed
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string());

    if let Some(ref t) = text {
        info!(
            plan = plan_base,
            iteration,
            reflection_len = t.len(),
            "Generated reflection for gate failure"
        );
    }

    text
}

/// Extract file paths from compiler error output (lines containing ` --> file:line`).
fn extract_affected_files(gate_output: &str) -> Vec<String> {
    gate_output
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim();
            trimmed
                .strip_prefix("--> ")
                .and_then(|rest| rest.split(':').next())
                .map(|s| s.to_string())
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

/// Extract the first error line for dedup purposes.
fn first_error_line(gate_output: &str) -> &str {
    gate_output
        .lines()
        .find(|l| l.starts_with("error[E") || l.starts_with("error: "))
        .unwrap_or("")
}

/// Spawn a background task to generate and store a gate failure reflection.
///
/// Non-blocking: the retry proceeds immediately while the reflection is generated.
/// The reflection (~1-2s via haiku) typically completes before the next injection
/// reads iteration memory (~5-10s for retry setup).
pub fn spawn_reflection(repo_root: &Path, plan: &str, gate_output: &str, iteration: u32) {
    let repo_root = repo_root.to_path_buf();
    let plan = plan.to_string();
    let gate_output = gate_output.to_string();

    // Extract plan number (e.g. "03" from "03-some-plan")
    let plan_num = plan.split('-').next().unwrap_or(&plan).to_string();

    // Dedup: skip if same error pattern already reflected
    let mem = IterationMemory::load(&repo_root, &plan_num).unwrap_or_else(|_| IterationMemory {
        plan: plan_num.clone(),
        iterations: Vec::new(),
    });

    let err_line = first_error_line(&gate_output).to_string();
    if !err_line.is_empty() && mem.has_error_pattern(&err_line) {
        info!(plan = %plan, "Skipping duplicate reflection for: {err_line}");
        return;
    }

    tokio::spawn(generate_and_store(
        repo_root,
        plan,
        plan_num,
        gate_output,
        err_line,
        iteration,
    ));
}

async fn generate_and_store(
    repo_root: PathBuf,
    plan: String,
    plan_num: String,
    gate_output: String,
    err_line: String,
    iteration: u32,
) {
    let affected_files = extract_affected_files(&gate_output);

    let reflection =
        match generate_reflection(&gate_output, &plan, iteration, &affected_files).await {
            Some(r) => r,
            None => return,
        };

    let mut mem =
        IterationMemory::load(&repo_root, &plan_num).unwrap_or_else(|_| IterationMemory {
            plan: plan_num.clone(),
            iterations: Vec::new(),
        });

    let mut gate_results = std::collections::HashMap::new();
    if !err_line.is_empty() {
        gate_results.insert("error".to_string(), err_line);
    }

    mem.record_iteration(IterationEntry {
        iteration,
        gate_results,
        diagnosis: Some(reflection),
        files_changed: affected_files,
    });

    if let Err(e) = mem.save(&repo_root) {
        warn!("Failed to save iteration memory for {plan}: {e}");
    }
}
