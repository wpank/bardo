//! Queue management: parses `.mori/queue.toml` for milestone-based plan ordering
//! and execution presets.
//!
//! The queue file replaces long CLI invocations with a versioned, diffable
//! configuration that groups plans into named milestones with optional
//! maintenance plan associations.

use std::path::Path;

use serde::Deserialize;

/// Top-level queue configuration parsed from `.mori/queue.toml`.
#[derive(Debug, Deserialize)]
pub struct QueueConfig {
    /// Run-level settings (mode, agent limits, preset).
    #[serde(default)]
    pub run: RunSettings,
    /// Ordered milestones. Each milestone groups plans with metadata.
    #[serde(default)]
    pub milestone: Vec<Milestone>,
}

/// Run-level settings from the `[run]` section.
#[derive(Debug, Default, Deserialize)]
pub struct RunSettings {
    /// Execution mode (e.g. "express").
    #[serde(default)]
    pub mode: Option<String>,
    /// Maximum concurrent agents.
    #[serde(default)]
    pub max_agents: Option<usize>,
    /// Maximum plans executing in parallel.
    #[serde(default)]
    pub max_parallel_plans: Option<usize>,
    /// Execution preset name (e.g. "quality", "balanced", "cost", "speed").
    #[serde(default)]
    pub preset: Option<String>,
}

/// A milestone groups plans with a name, description, tags, and optional
/// maintenance plan associations.
#[derive(Debug, Deserialize)]
pub struct Milestone {
    /// Human-readable milestone name (e.g. "Minimal MVP").
    pub name: String,
    /// Description of what this milestone achieves.
    #[serde(default)]
    pub description: String,
    /// Tags for filtering and grouping.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Ordered list of plan specs in this milestone.
    #[serde(default)]
    pub plans: Vec<String>,
    /// Maintenance plan associations (refactor, QA, docs, etc.).
    #[serde(default)]
    pub maintenance: Option<MaintenanceConfig>,
}

/// Maintenance plan configuration for a milestone.
#[derive(Debug, Deserialize)]
pub struct MaintenanceConfig {
    /// Batches of plans with associated maintenance plans that run after them.
    #[serde(default)]
    pub after_batch: Vec<MaintenanceBatch>,
}

/// A batch of implementation plans with their associated maintenance plans.
#[derive(Debug, Deserialize)]
pub struct MaintenanceBatch {
    /// Implementation plans this maintenance batch covers.
    pub plans: Vec<String>,
    /// Refactor plan to run after this batch.
    #[serde(default)]
    pub refactor: Option<String>,
    /// QA plan to run after this batch.
    #[serde(default)]
    pub qa: Option<String>,
    /// Documentation plan to run after this batch.
    #[serde(default)]
    pub docs: Option<String>,
    /// Integration test plan to run after this batch.
    #[serde(default)]
    pub integration: Option<String>,
    /// Audit plan to run after this batch.
    #[serde(default)]
    pub audit: Option<String>,
}

impl QueueConfig {
    /// Load from `.mori/queue.toml` relative to the given repo root.
    /// Returns `None` if the file doesn't exist or can't be parsed.
    pub fn load(repo_root: &Path) -> Option<Self> {
        let path = repo_root.join(".mori/queue.toml");
        let content = std::fs::read_to_string(&path).ok()?;
        match toml::from_str(&content) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                tracing::warn!("Failed to parse .mori/queue.toml: {e}");
                None
            }
        }
    }

    /// Extract a flat list of all plan specs across all milestones,
    /// including maintenance plans, in milestone order.
    ///
    /// Within each milestone, implementation plans come first, followed
    /// by maintenance plans from each batch.
    pub fn all_plan_specs(&self) -> Vec<String> {
        let mut specs = Vec::new();
        for milestone in &self.milestone {
            specs.extend(milestone.plans.iter().cloned());
            if let Some(ref maint) = milestone.maintenance {
                for batch in &maint.after_batch {
                    collect_maintenance_plans(batch, &mut specs);
                }
            }
        }
        specs
    }

    /// Get plans for a specific milestone by name (case-insensitive match).
    /// Returns the milestone's implementation plans followed by its
    /// maintenance plans.
    pub fn milestone_plans(&self, name: &str) -> Option<Vec<String>> {
        self.milestone
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
            .map(|m| {
                let mut specs = m.plans.clone();
                if let Some(ref maint) = m.maintenance {
                    for batch in &maint.after_batch {
                        collect_maintenance_plans(batch, &mut specs);
                    }
                }
                specs
            })
    }

    /// Find a milestone by name (case-insensitive).
    pub fn find_milestone(&self, name: &str) -> Option<&Milestone> {
        self.milestone
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
    }
}

/// Append all non-None maintenance plan specs from a batch into `out`.
fn collect_maintenance_plans(batch: &MaintenanceBatch, out: &mut Vec<String>) {
    if let Some(ref r) = batch.refactor {
        out.push(r.clone());
    }
    if let Some(ref q) = batch.qa {
        out.push(q.clone());
    }
    if let Some(ref w) = batch.docs {
        out.push(w.clone());
    }
    if let Some(ref x) = batch.integration {
        out.push(x.clone());
    }
    if let Some(ref a) = batch.audit {
        out.push(a.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_QUEUE: &str = r#"
[run]
mode = "express"
max_agents = 20
max_parallel_plans = 6
preset = "balanced"

[[milestone]]
name = "Minimal MVP"
description = "Core types, chain RPC, safety, inference, heartbeat."
tags = ["mvp", "core"]
plans = ["02", "04", "05", "06", "09"]

[milestone.maintenance]
after_batch = [
  { plans = ["02", "04", "05"], refactor = "R01", qa = "Q01", docs = "W01" },
  { plans = ["06", "09"], refactor = "R02", integration = "X01" },
]

[[milestone]]
name = "Demo Story"
description = "Terminal, trading, dreams."
tags = ["demo", "terminal"]
plans = ["07", "07a", "08"]
"#;

    #[test]
    fn parse_sample_queue() {
        let cfg: QueueConfig = toml::from_str(SAMPLE_QUEUE).expect("should parse");
        assert_eq!(cfg.milestone.len(), 2);
        assert_eq!(cfg.run.mode.as_deref(), Some("express"));
        assert_eq!(cfg.run.max_agents, Some(20));
        assert_eq!(cfg.run.max_parallel_plans, Some(6));
        assert_eq!(cfg.run.preset.as_deref(), Some("balanced"));
    }

    #[test]
    fn all_plan_specs_returns_milestone_order_with_maintenance() {
        let cfg: QueueConfig = toml::from_str(SAMPLE_QUEUE).expect("should parse");
        let specs = cfg.all_plan_specs();
        // Milestone 1 plans: 02, 04, 05, 06, 09
        // Milestone 1 maintenance: R01, Q01, W01, R02, X01
        // Milestone 2 plans: 07, 07a, 08
        let expected = vec![
            "02", "04", "05", "06", "09", "R01", "Q01", "W01", "R02", "X01", "07", "07a", "08",
        ];
        assert_eq!(specs, expected);
    }

    #[test]
    fn milestone_plans_by_name() {
        let cfg: QueueConfig = toml::from_str(SAMPLE_QUEUE).expect("should parse");
        let mvp = cfg
            .milestone_plans("Minimal MVP")
            .expect("should find milestone");
        assert_eq!(
            mvp,
            vec!["02", "04", "05", "06", "09", "R01", "Q01", "W01", "R02", "X01"]
        );
        let demo = cfg
            .milestone_plans("demo story")
            .expect("case-insensitive match");
        assert_eq!(demo, vec!["07", "07a", "08"]);
    }

    #[test]
    fn milestone_plans_missing_returns_none() {
        let cfg: QueueConfig = toml::from_str(SAMPLE_QUEUE).expect("should parse");
        assert!(cfg.milestone_plans("Nonexistent").is_none());
    }

    #[test]
    fn missing_file_returns_none() {
        let result = QueueConfig::load(Path::new("/nonexistent/path"));
        assert!(result.is_none());
    }

    #[test]
    fn minimal_queue_parses() {
        let minimal = r#"
[[milestone]]
name = "Only one"
plans = ["01"]
"#;
        let cfg: QueueConfig = toml::from_str(minimal).expect("should parse minimal");
        assert_eq!(cfg.milestone.len(), 1);
        assert!(cfg.run.mode.is_none());
        assert!(cfg.run.max_agents.is_none());
        assert_eq!(cfg.all_plan_specs(), vec!["01"]);
    }

    #[test]
    fn empty_queue_parses() {
        let empty = "";
        let cfg: QueueConfig = toml::from_str(empty).expect("should parse empty");
        assert!(cfg.milestone.is_empty());
        assert!(cfg.all_plan_specs().is_empty());
    }

    #[test]
    fn milestone_without_maintenance_parses() {
        let no_maint = r#"
[[milestone]]
name = "Simple"
plans = ["01", "02", "03"]
"#;
        let cfg: QueueConfig = toml::from_str(no_maint).expect("should parse");
        let plans = cfg.milestone_plans("Simple").expect("found");
        assert_eq!(plans, vec!["01", "02", "03"]);
    }

    #[test]
    fn partial_maintenance_batch_parses() {
        let partial = r#"
[[milestone]]
name = "Partial"
plans = ["01"]

[milestone.maintenance]
after_batch = [
  { plans = ["01"], refactor = "R01" },
]
"#;
        let cfg: QueueConfig = toml::from_str(partial).expect("should parse");
        let specs = cfg.all_plan_specs();
        assert_eq!(specs, vec!["01", "R01"]);
    }
}
