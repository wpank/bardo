//! Source code snippet extraction.

use std::path::Path;

use crate::error::ContextError;

/// Configuration for snippet extraction.
#[derive(Debug, Clone)]
pub struct SnippetConfig {
    /// Lines of context before the symbol.
    pub before: usize,
    /// Lines of context after the symbol.
    pub after: usize,
}

impl Default for SnippetConfig {
    fn default() -> Self {
        Self {
            before: 5,
            after: 10,
        }
    }
}

impl SnippetConfig {
    /// Create from a total context_lines count (split roughly evenly, biased after).
    pub fn from_context_lines(n: usize) -> Self {
        let before = n / 3;
        let after = n - before;
        Self { before, after }
    }
}

/// A source code snippet around a specific line.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Snippet {
    /// The extracted source lines.
    pub code: String,
    /// First line number in the snippet (1-indexed).
    pub start_line: u32,
    /// Last line number in the snippet (1-indexed).
    pub end_line: u32,
}

/// Extract a snippet from a file at a given line.
pub fn extract_snippet(
    root: &Path,
    rel_path: &str,
    target_line: u32,
    config: &SnippetConfig,
) -> Result<Snippet, ContextError> {
    let full_path = root.join(rel_path);
    let source = std::fs::read_to_string(&full_path)?;
    let lines: Vec<&str> = source.lines().collect();

    if lines.is_empty() {
        return Ok(Snippet {
            code: String::new(),
            start_line: 1,
            end_line: 1,
        });
    }

    let target_idx = (target_line as usize).saturating_sub(1);
    let start = target_idx.saturating_sub(config.before);
    let end = (target_idx + config.after + 1).min(lines.len());

    let code = lines[start..end].join("\n");

    Ok(Snippet {
        code,
        start_line: (start + 1) as u32,
        end_line: end as u32,
    })
}
