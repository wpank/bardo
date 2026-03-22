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
}
