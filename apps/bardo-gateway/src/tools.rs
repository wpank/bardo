//! Tool definition pruning based on per-session usage tracking.
//!
//! Over the course of a session, most tools go unused. After enough requests
//! have passed, we can strip tool definitions the model has never invoked,
//! reducing input tokens and cost.

use std::collections::HashSet;

use dashmap::DashMap;
use serde_json::Value;

/// Tracks which tools each session has used and prunes unused ones.
pub struct ToolTracker {
    /// session_id -> set of tool names used in that session.
    usage: DashMap<String, HashSet<String>>,
    /// session_id -> total request count.
    request_counts: DashMap<String, usize>,
}

impl ToolTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self {
            usage: DashMap::new(),
            request_counts: DashMap::new(),
        }
    }

    /// Record that a tool was used in this session.
    pub fn record_usage(&self, session_id: &str, tool_name: &str) {
        self.usage
            .entry(session_id.to_string())
            .or_default()
            .insert(tool_name.to_string());
    }

    /// Increment the request counter for this session.
    pub fn record_request(&self, session_id: &str) {
        *self
            .request_counts
            .entry(session_id.to_string())
            .or_insert(0) += 1;
    }

    /// Prune tools that have never been used in this session.
    ///
    /// Returns `None` if the session hasn't had enough requests to justify
    /// pruning (we need at least `min_requests` before we start cutting).
    /// Returns `Some(pruned_tools)` with only the tools the session has
    /// actually used, plus any tools without a recognizable `name` field.
    pub fn prune_tools(
        &self,
        session_id: &str,
        tools: &[Value],
        min_requests: usize,
    ) -> Option<Vec<Value>> {
        let count = self.request_counts.get(session_id).map(|c| *c).unwrap_or(0);

        if count < min_requests {
            return None;
        }

        let used = self.usage.get(session_id)?;

        let pruned: Vec<Value> = tools
            .iter()
            .filter(|tool| {
                // Keep tools whose name is in the used set, or that lack a name field
                tool.get("name")
                    .and_then(|n| n.as_str())
                    .map_or(true, |name| used.contains(name))
            })
            .cloned()
            .collect();

        // Only return pruned list if we actually removed something
        if pruned.len() < tools.len() {
            Some(pruned)
        } else {
            None
        }
    }

    /// Extract tool names from an Anthropic-format response.
    ///
    /// Looks for `tool_use` content blocks in the response's `content` array.
    pub fn extract_used_tools(response: &Value) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(content) = response.get("content").and_then(|c| c.as_array()) {
            for block in content {
                let is_tool_use = block
                    .get("type")
                    .and_then(|t| t.as_str())
                    .is_some_and(|t| t == "tool_use");
                if is_tool_use {
                    if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                        names.push(name.to_string());
                    }
                }
            }
        }
        names
    }

    /// Remove all tracking data for a session.
    pub fn remove_session(&self, session_id: &str) {
        self.usage.remove(session_id);
        self.request_counts.remove(session_id);
    }

    /// Retain only sessions that pass the predicate. Used by session eviction.
    pub fn retain_sessions(&self, mut keep: impl FnMut(&str) -> bool) {
        self.usage.retain(|k, _| keep(k));
        self.request_counts.retain(|k, _| keep(k));
    }

    /// Get all sessions with their tool usage and request counts (for persistence).
    pub fn all_sessions(&self) -> Vec<(String, HashSet<String>, usize)> {
        self.usage
            .iter()
            .map(|entry| {
                let sid = entry.key().clone();
                let tools = entry.value().clone();
                let count = self
                    .request_counts
                    .get(&sid)
                    .map(|c| *c.value())
                    .unwrap_or(0);
                (sid, tools, count)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_requests_returns_none() {
        let tracker = ToolTracker::new();
        tracker.record_request("s1");
        tracker.record_request("s1");

        let tools = vec![
            serde_json::json!({"name": "bash", "description": "run bash"}),
            serde_json::json!({"name": "read", "description": "read file"}),
        ];

        // min_requests = 5, only 2 recorded
        assert!(tracker.prune_tools("s1", &tools, 5).is_none());
    }

    #[test]
    fn prunes_unused_tools() {
        let tracker = ToolTracker::new();
        for _ in 0..10 {
            tracker.record_request("s1");
        }
        tracker.record_usage("s1", "bash");

        let tools = vec![
            serde_json::json!({"name": "bash", "description": "run bash"}),
            serde_json::json!({"name": "read", "description": "read file"}),
            serde_json::json!({"name": "write", "description": "write file"}),
        ];

        let pruned = tracker.prune_tools("s1", &tools, 5);
        assert!(pruned.is_some());
        let pruned = pruned.unwrap_or_default();
        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0].get("name").and_then(|n| n.as_str()), Some("bash"));
    }

    #[test]
    fn no_prune_if_all_tools_used() {
        let tracker = ToolTracker::new();
        for _ in 0..10 {
            tracker.record_request("s1");
        }
        tracker.record_usage("s1", "bash");
        tracker.record_usage("s1", "read");

        let tools = vec![
            serde_json::json!({"name": "bash", "description": "run bash"}),
            serde_json::json!({"name": "read", "description": "read file"}),
        ];

        // All tools used, nothing to prune
        assert!(tracker.prune_tools("s1", &tools, 5).is_none());
    }

    #[test]
    fn extract_tool_names_from_response() {
        let response = serde_json::json!({
            "content": [
                {"type": "text", "text": "Let me run that."},
                {"type": "tool_use", "id": "tu_1", "name": "bash", "input": {"command": "ls"}},
                {"type": "tool_use", "id": "tu_2", "name": "read", "input": {"path": "/tmp/x"}}
            ]
        });

        let names = ToolTracker::extract_used_tools(&response);
        assert_eq!(names, vec!["bash", "read"]);
    }

    #[test]
    fn extract_no_tools_from_text_only_response() {
        let response = serde_json::json!({
            "content": [
                {"type": "text", "text": "Hello!"}
            ]
        });
        assert!(ToolTracker::extract_used_tools(&response).is_empty());
    }

    #[test]
    fn remove_session_clears_data() {
        let tracker = ToolTracker::new();
        tracker.record_request("s1");
        tracker.record_usage("s1", "bash");
        tracker.remove_session("s1");

        assert!(tracker.usage.get("s1").is_none());
        assert!(tracker.request_counts.get("s1").is_none());
    }
}
