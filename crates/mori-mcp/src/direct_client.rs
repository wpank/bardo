//! Direct LLM client that shells out to the `claude` CLI.
//!
//! Works without a gateway -- just needs `claude` in PATH.

use anyhow::{Context, Result, bail};
use tokio::process::Command;

/// Call Claude CLI with a system prompt and user message.
/// Returns the model's text response.
///
/// # Errors
///
/// Returns an error if `claude` is not found in PATH, exits non-zero,
/// or produces non-UTF-8 output.
pub async fn call_claude(
    model: &str,
    system: &str,
    user_message: &str,
    _max_tokens: u32,
) -> Result<String> {
    // Check that claude is available before attempting to call it.
    let which = Command::new("which")
        .arg("claude")
        .output()
        .await
        .context("failed to run `which claude`")?;

    if !which.status.success() {
        bail!(
            "claude CLI not found in PATH. Install it from https://docs.anthropic.com/en/docs/claude-cli \
             or use --gateway-url to route through bardo-gateway instead."
        );
    }

    let mut cmd = Command::new("claude");
    cmd.arg("--print")
        .arg("--model")
        .arg(model)
        .arg("--max-turns")
        .arg("1")
        .arg("--output-format")
        .arg("text")
        .arg("--system-prompt")
        .arg(system)
        .arg(user_message);

    let output = cmd
        .output()
        .await
        .context("failed to spawn claude CLI process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("claude CLI exited with {}: {stderr}", output.status);
    }

    let stdout = String::from_utf8(output.stdout)
        .context("claude CLI produced non-UTF-8 output")?;

    Ok(stdout)
}

#[cfg(test)]
mod tests {
    // Integration tests would require the claude CLI to be installed,
    // so we only test the module compiles. Real testing happens via
    // `mori-mcp enrich ... --dry-run`.
}
