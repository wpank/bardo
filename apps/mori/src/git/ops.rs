use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// Run a git command and return stdout
pub fn run_git(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Like [`run_git`], but sets author/committer env vars so commands that create commits
/// (e.g. `git stash push`) succeed even when `user.name` / `user.email` are unset.
pub fn run_git_with_plumbing_author(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .env("GIT_AUTHOR_NAME", "bardo-ctl")
        .env("GIT_AUTHOR_EMAIL", "bardo-ctl@local")
        .env("GIT_COMMITTER_NAME", "bardo-ctl")
        .env("GIT_COMMITTER_EMAIL", "bardo-ctl@local")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get current branch name
pub fn current_branch(repo_root: &Path) -> Result<String> {
    run_git(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Get current commit hash (short)
pub fn current_commit_hash(repo_root: &Path) -> Result<String> {
    run_git(repo_root, &["rev-parse", "--short", "HEAD"])
}
