//! `mori-mcp init` -- scaffold a `.mori/` directory in a project.

use std::fs;
use std::path::Path;

/// Run the init command, creating `.mori/` scaffolding under `root`.
///
/// This is idempotent: existing files are never overwritten, and
/// `.gitignore` is only appended to when the mori rules are missing.
pub fn run(root: &Path) -> anyhow::Result<()> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    println!("Initializing mori in {}...", root.display());

    let mori = root.join(".mori");
    let mut created_anything = false;

    // .mori/ directory
    created_anything |= ensure_dir(&mori, ".mori/")?;

    // .mori/config.toml
    created_anything |= ensure_file(
        &mori.join("config.toml"),
        ".mori/config.toml",
        DEFAULT_CONFIG,
    )?;

    // .mori/plans/
    created_anything |= ensure_dir(&mori.join("plans"), ".mori/plans/")?;

    // .mori/runs/
    created_anything |= ensure_dir(&mori.join("runs"), ".mori/runs/")?;

    // .mori/mcp-config.json
    created_anything |= ensure_file(
        &mori.join("mcp-config.json"),
        ".mori/mcp-config.json",
        DEFAULT_MCP_CONFIG,
    )?;

    // .gitignore -- append only
    created_anything |= ensure_gitignore(&root)?;

    println!();
    if created_anything {
        println!("Done. Run `mori-mcp index init` to build the code index.");
    } else {
        println!("Already initialized.");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a directory if it doesn't exist. Returns `true` if it was created.
fn ensure_dir(path: &Path, label: &str) -> anyhow::Result<bool> {
    if path.is_dir() {
        println!("  skip: {label} (already exists)");
        Ok(false)
    } else {
        fs::create_dir_all(path)?;
        println!("  create: {label}");
        Ok(true)
    }
}

/// Write a file if it doesn't exist. Returns `true` if it was created.
fn ensure_file(path: &Path, label: &str, content: &str) -> anyhow::Result<bool> {
    if path.exists() {
        println!("  skip: {label} (already exists)");
        Ok(false)
    } else {
        fs::write(path, content)?;
        println!("  create: {label}");
        Ok(true)
    }
}

/// Append mori gitignore rules if they aren't already present.
/// Returns `true` if the file was modified.
fn ensure_gitignore(root: &Path) -> anyhow::Result<bool> {
    let gitignore = root.join(".gitignore");

    let existing = if gitignore.exists() {
        fs::read_to_string(&gitignore)?
    } else {
        String::new()
    };

    if existing.contains(".mori/*") {
        println!("  skip: .gitignore (mori rules already present)");
        return Ok(false);
    }

    // Build the block to append.
    let block = "\n# Mori — ignore everything except shared config\n\
                  .mori/*\n\
                  !.mori/config.toml\n\
                  !.mori/mcp-config.json\n";

    let mut out = existing;
    // Make sure we start on a fresh line.
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(block);

    fs::write(&gitignore, out)?;
    println!("  append: .gitignore (mori rules)");
    Ok(true)
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

/// Default `.mori/config.toml`.
const DEFAULT_CONFIG: &str = r#"# Mori configuration
# See: https://github.com/bardo-run/mori

# Agent models
claude_default_model = "claude-sonnet-4-6"
conductor_model = "claude-sonnet-4-6"

# Execution
max_iterations = 3
max_agents = 8
parallel_enabled = true
express_mode = false

# Gates
skip_tests = false
clippy_enabled = true

# Context
context_limit_k = 200
"#;

/// Default `.mori/mcp-config.json`.
const DEFAULT_MCP_CONFIG: &str = r#"{
  "mcpServers": {
    "mori": {
      "command": "mori-mcp",
      "args": ["context-server", "--root", "."]
    }
  }
}
"#;
