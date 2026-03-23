use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IterationMemory {
    pub plan: String,
    pub iterations: Vec<IterationEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IterationEntry {
    pub iteration: u32,
    pub gate_results: HashMap<String, String>,
    pub diagnosis: Option<String>,
    pub files_changed: Vec<String>,
}

impl IterationMemory {
    pub fn load(repo_root: &Path, plan_num: &str) -> Result<Self> {
        let path = repo_root.join(format!(
            "plans/context/iteration-memory/{plan_num}-memory.json"
        ));
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self {
                plan: plan_num.to_string(),
                iterations: Vec::new(),
            })
        }
    }

    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let dir = repo_root.join("plans/context/iteration-memory");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}-memory.json", self.plan));
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn record_iteration(&mut self, entry: IterationEntry) {
        self.iterations.push(entry);
    }

    /// Format all reflections as markdown for injection into agent context.
    pub fn format_reflections_md(&self) -> Option<String> {
        let diagnosed: Vec<&IterationEntry> = self
            .iterations
            .iter()
            .filter(|e| e.diagnosis.is_some())
            .collect();

        if diagnosed.is_empty() {
            return None;
        }

        let mut md = String::from("# Prior Iteration Reflections\n\n");
        md.push_str(
            "These are analyses of previous failed attempts at this plan. \
             Learn from these failures and avoid repeating the same mistakes.\n\n",
        );

        for entry in &diagnosed {
            md.push_str(&format!("## Iteration {} Reflection\n\n", entry.iteration));
            if let Some(ref diag) = entry.diagnosis {
                md.push_str(diag);
                md.push_str("\n\n");
            }
            if !entry.files_changed.is_empty() {
                md.push_str(&format!(
                    "Files changed in that attempt: {}\n\n",
                    entry.files_changed.join(", ")
                ));
            }
            md.push_str("---\n\n");
        }

        Some(md)
    }

    /// Check if an error pattern already has a reflection (for dedup).
    pub fn has_error_pattern(&self, first_error_line: &str) -> bool {
        self.iterations.iter().any(|entry| {
            entry
                .gate_results
                .values()
                .any(|v| v.contains(first_error_line))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("bardo-test-iter-mem-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn load_missing_returns_empty() {
        let root = test_dir("missing");
        let mem = IterationMemory::load(&root, "03").unwrap();
        assert_eq!(mem.plan, "03");
        assert!(mem.iterations.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn round_trip_save_load() {
        let root = test_dir("roundtrip");

        let mut mem = IterationMemory::load(&root, "07").unwrap();
        let mut gate_results = HashMap::new();
        gate_results.insert("compile".to_string(), "PASS".to_string());
        gate_results.insert("test".to_string(), "FAIL".to_string());
        mem.record_iteration(IterationEntry {
            iteration: 1,
            gate_results,
            diagnosis: Some("missing import in lib.rs".to_string()),
            files_changed: vec!["src/lib.rs".to_string()],
        });
        mem.save(&root).unwrap();

        let loaded = IterationMemory::load(&root, "07").unwrap();
        assert_eq!(loaded.iterations.len(), 1);
        assert_eq!(loaded.iterations[0].iteration, 1);
        assert_eq!(loaded.iterations[0].gate_results["compile"], "PASS");
        assert_eq!(
            loaded.iterations[0].diagnosis.as_deref(),
            Some("missing import in lib.rs")
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn format_reflections_md_skips_empty() {
        let mem = IterationMemory {
            plan: "01".to_string(),
            iterations: vec![IterationEntry {
                iteration: 1,
                gate_results: HashMap::new(),
                diagnosis: None,
                files_changed: vec![],
            }],
        };
        assert!(mem.format_reflections_md().is_none());
    }

    #[test]
    fn format_reflections_md_includes_diagnosis() {
        let mem = IterationMemory {
            plan: "01".to_string(),
            iterations: vec![
                IterationEntry {
                    iteration: 1,
                    gate_results: HashMap::new(),
                    diagnosis: Some("Missing import".to_string()),
                    files_changed: vec!["src/lib.rs".to_string()],
                },
                IterationEntry {
                    iteration: 2,
                    gate_results: HashMap::new(),
                    diagnosis: None,
                    files_changed: vec![],
                },
            ],
        };
        let md = mem.format_reflections_md().unwrap();
        assert!(md.contains("# Prior Iteration Reflections"));
        assert!(md.contains("## Iteration 1 Reflection"));
        assert!(md.contains("Missing import"));
        assert!(md.contains("src/lib.rs"));
        assert!(!md.contains("Iteration 2"));
    }

    #[test]
    fn has_error_pattern_matches() {
        let mem = IterationMemory {
            plan: "01".to_string(),
            iterations: vec![IterationEntry {
                iteration: 1,
                gate_results: {
                    let mut m = HashMap::new();
                    m.insert(
                        "error".to_string(),
                        "error[E0433]: failed to resolve".to_string(),
                    );
                    m
                },
                diagnosis: Some("fix it".to_string()),
                files_changed: vec![],
            }],
        };
        assert!(mem.has_error_pattern("error[E0433]: failed to resolve"));
        assert!(!mem.has_error_pattern("error[E0599]: no method"));
    }
}
