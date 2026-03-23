//! Episode logging and playbook types for the mori memory system.
//!
//! Episodes are appended to `.mori/memory/episodes.jsonl` after each task completes.
//! Playbook rules live in `.mori/memory/playbook.toml` and are injected into agent
//! context when their file-glob triggers match the current plan's files.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::warn;

// ---------------------------------------------------------------------------
// Episode types
// ---------------------------------------------------------------------------

/// A single task execution record, written as one JSON line per episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique episode id (plan_id:task_id:timestamp).
    pub id: String,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Plan base name (e.g. "03a").
    pub plan_id: String,
    /// Task id within the plan.
    pub task_id: String,
    /// Agent role that executed this task (e.g. "implementer").
    pub role: String,
    /// Model slug used (e.g. "gpt-5.4").
    pub model: String,
    /// Files the task was expected to touch.
    pub files_changed: Vec<String>,
    /// Cumulative input tokens for this agent instance.
    pub input_tokens: u64,
    /// Cumulative output tokens for this agent instance.
    pub output_tokens: u64,
    /// Estimated cost in USD.
    pub cost_usd: f64,
    /// Whether the task passed its gate (true when logging at completion).
    pub gate_passed: bool,
    /// Number of iterations/retries before success.
    pub iterations: u32,
    /// Wall-clock seconds from task start to completion.
    pub duration_secs: u32,
    /// Optional error signature if the task had issues.
    pub error_signature: Option<String>,
    /// Optional self-reflection from the agent.
    pub reflection: Option<String>,
}

/// Append an episode to `.mori/memory/episodes.jsonl`.
///
/// This is intentionally non-blocking from the caller's perspective: if the
/// write fails we log a warning and return Ok. The orchestrator must never
/// crash because of memory logging.
pub fn log_episode(repo_root: &Path, episode: &Episode) {
    let dir = repo_root.join(".mori/memory");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!("memory: failed to create .mori/memory dir: {e}");
        return;
    }
    let path = dir.join("episodes.jsonl");
    let json = match serde_json::to_string(episode) {
        Ok(j) => j,
        Err(e) => {
            warn!("memory: failed to serialize episode: {e}");
            return;
        }
    };
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(f) => f,
        Err(e) => {
            warn!("memory: failed to open episodes.jsonl: {e}");
            return;
        }
    };
    use std::io::Write;
    if let Err(e) = writeln!(file, "{json}") {
        warn!("memory: failed to write episode: {e}");
    }
}

// ---------------------------------------------------------------------------
// Playbook types (used by inject.rs for context injection)
// ---------------------------------------------------------------------------

/// Top-level playbook config parsed from `.mori/memory/playbook.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaybookConfig {
    /// List of rules to match against task files.
    #[serde(default)]
    pub rule: Vec<PlaybookRule>,
}

/// A single playbook rule that fires when file globs or tags match.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaybookRule {
    /// Unique rule identifier.
    pub id: String,
    /// File glob patterns that trigger this rule.
    #[serde(default)]
    pub trigger_files: Vec<String>,
    /// Tag-based triggers.
    #[serde(default)]
    pub trigger_tags: Vec<String>,
    /// The advice text injected into agent context.
    pub context: String,
    /// Confidence score (0.0-1.0). Rules below 0.5 are ignored.
    #[serde(default)]
    pub confidence: f64,
    /// How many times this rule has been validated by outcomes.
    #[serde(default)]
    pub validated_count: u32,
}

impl PlaybookConfig {
    /// Return all rules whose triggers match the given files or tags.
    /// Only rules with confidence >= 0.5 are considered.
    pub fn match_rules(&self, files: &[String], tags: &[String]) -> Vec<&PlaybookRule> {
        self.rule
            .iter()
            .filter(|r| {
                if r.confidence < 0.5 {
                    return false;
                }
                let file_match = r.trigger_files.is_empty()
                    || r.trigger_files
                        .iter()
                        .any(|pattern| files.iter().any(|f| glob_match(pattern, f)));
                let tag_match =
                    r.trigger_tags.is_empty() || r.trigger_tags.iter().any(|t| tags.contains(t));
                file_match && tag_match
            })
            .collect()
    }
}

/// Simple glob matching: handles `*` as a wildcard segment.
///
/// Supports patterns like `src/auth/*`, `crates/golem-*/src/*`, and literal
/// equality when no `*` is present.
fn glob_match(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return text == pattern;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 2 {
        text.starts_with(parts[0]) && text.ends_with(parts[1])
    } else {
        // Multi-star: check that all non-empty parts appear in order
        let mut remaining = text;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if i == 0 {
                if !remaining.starts_with(part) {
                    return false;
                }
                remaining = &remaining[part.len()..];
            } else if let Some(pos) = remaining.find(part) {
                remaining = &remaining[pos + part.len()..];
            } else {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_match_literal() {
        assert!(glob_match("src/main.rs", "src/main.rs"));
        assert!(!glob_match("src/main.rs", "src/lib.rs"));
    }

    #[test]
    fn glob_match_single_star() {
        assert!(glob_match("src/auth/*", "src/auth/middleware.rs"));
        assert!(!glob_match("src/auth/*", "src/handler.rs"));
    }

    #[test]
    fn glob_match_middle_star() {
        assert!(glob_match(
            "crates/golem-*/src/*",
            "crates/golem-core/src/lib.rs"
        ));
        assert!(!glob_match(
            "crates/golem-*/src/*",
            "crates/other/src/lib.rs"
        ));
    }

    #[test]
    fn playbook_match_by_file() {
        let config = PlaybookConfig {
            rule: vec![PlaybookRule {
                id: "r1".to_string(),
                trigger_files: vec!["src/auth/*".to_string()],
                trigger_tags: vec![],
                context: "Check lifetimes".to_string(),
                confidence: 0.9,
                validated_count: 3,
            }],
        };
        let files = vec!["src/auth/middleware.rs".to_string()];
        let matched = config.match_rules(&files, &[]);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "r1");
    }

    #[test]
    fn playbook_skip_low_confidence() {
        let config = PlaybookConfig {
            rule: vec![PlaybookRule {
                id: "r1".to_string(),
                trigger_files: vec![],
                trigger_tags: vec![],
                context: "Low confidence".to_string(),
                confidence: 0.3,
                validated_count: 0,
            }],
        };
        let matched = config.match_rules(&[], &[]);
        assert!(matched.is_empty());
    }

    #[test]
    fn playbook_match_by_tag() {
        let config = PlaybookConfig {
            rule: vec![PlaybookRule {
                id: "r2".to_string(),
                trigger_files: vec![],
                trigger_tags: vec!["core".to_string()],
                context: "Use GolemError".to_string(),
                confidence: 0.85,
                validated_count: 5,
            }],
        };
        let tags = vec!["core".to_string()];
        let matched = config.match_rules(&[], &tags);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn episode_roundtrip() {
        let ep = Episode {
            id: "03a:t1:2026-03-22T00:00:00Z".to_string(),
            timestamp: "2026-03-22T00:00:00Z".to_string(),
            plan_id: "03a".to_string(),
            task_id: "t1".to_string(),
            role: "implementer".to_string(),
            model: "gpt-5.4".to_string(),
            files_changed: vec!["src/main.rs".to_string()],
            input_tokens: 1000,
            output_tokens: 500,
            cost_usd: 0.05,
            gate_passed: true,
            iterations: 1,
            duration_secs: 120,
            error_signature: None,
            reflection: None,
        };
        let json = serde_json::to_string(&ep).expect("serialize");
        let parsed: Episode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.plan_id, "03a");
        assert_eq!(parsed.task_id, "t1");
    }
}
