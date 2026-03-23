use std::path::Path;

/// Run DX preflight checks and return warning strings.
pub fn preflight_dx_checks(repo_root: &Path) -> Vec<String> {
    let mut warnings = Vec::new();

    // 1. Check for fast linker (mold or lld)
    let cargo_config = repo_root.join(".cargo/config.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_config) {
        if !content.contains("mold") && !content.contains("lld") {
            warnings.push("No fast linker (mold/lld) configured in .cargo/config.toml".to_string());
        }
    } else {
        warnings
            .push("No .cargo/config.toml found — consider configuring mold/lld linker".to_string());
    }

    // 2. Check for sccache
    let has_sccache_env = std::env::var("RUSTC_WRAPPER")
        .map(|v| v.contains("sccache"))
        .unwrap_or(false);
    let has_sccache_config = cargo_config.exists()
        && std::fs::read_to_string(&cargo_config)
            .map(|c| c.contains("sccache"))
            .unwrap_or(false);
    if !has_sccache_env && !has_sccache_config {
        warnings.push(
            "sccache not configured — consider RUSTC_WRAPPER=sccache for faster rebuilds"
                .to_string(),
        );
    }

    // 3. Check for cargo nextest
    let nextest_ok = std::process::Command::new("cargo")
        .args(["nextest", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !nextest_ok {
        warnings.push(
            "cargo-nextest not installed — `cargo install cargo-nextest` for faster tests"
                .to_string(),
        );
    }

    // 4. Check cargo fmt
    let fmt_check = std::process::Command::new("cargo")
        .args(["fmt", "--check"])
        .current_dir(repo_root)
        .output();
    if let Ok(output) = fmt_check {
        if !output.status.success() {
            // Auto-fix
            let _ = std::process::Command::new("cargo")
                .arg("fmt")
                .current_dir(repo_root)
                .output();
            warnings.push("Auto-formatted code with cargo fmt".to_string());
        }
    }

    warnings
}
