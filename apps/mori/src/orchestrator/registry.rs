use super::schema::{CompletionReport, TypeRecord};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone)]
pub struct Registry {
    root: PathBuf,
    write_lock: std::sync::Arc<Mutex<()>>,
}

impl Registry {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            write_lock: std::sync::Arc::new(Mutex::new(())),
        }
    }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        for name in &[
            "types.json",
            "traits.json",
            "deviations.json",
            "issues.json",
            "completed.json",
        ] {
            let path = self.root.join(name);
            if !path.exists() {
                std::fs::write(&path, "[]")?;
            }
        }
        Ok(())
    }

    /// Append completion data. Only called by orchestrator after a plan completes.
    pub fn append_completion(&self, report: &CompletionReport) -> Result<()> {
        let _lock = self.write_lock.lock().unwrap();
        if !report.types_defined.is_empty() {
            self.append_json_records("types.json", &report.types_defined)?;
        }
        if !report.deviations.is_empty() {
            let records: Vec<serde_json::Value> = report
                .deviations
                .iter()
                .map(|d| serde_json::json!({"plan": report.plan, "description": d}))
                .collect();
            self.append_json_values("deviations.json", &records)?;
        }
        let completed = serde_json::json!({
            "plan": report.plan,
            "compile_status": report.compile_status,
            "test_counts": report.test_counts,
        });
        self.append_json_values("completed.json", &[completed])?;
        Ok(())
    }

    /// Generate a bounded (~5000 char) registry snapshot for the given plan dependencies.
    pub fn snapshot_for(&self, plan_deps: &[String]) -> Result<String> {
        let types: Vec<TypeRecord> = self.read_json("types.json")?;
        let mut lines = vec!["# Registry Snapshot\n".to_string()];
        let relevant: Vec<&TypeRecord> = if plan_deps.is_empty() {
            types.iter().collect()
        } else {
            types
                .iter()
                .filter(|t| plan_deps.contains(&t.plan))
                .collect()
        };
        if !relevant.is_empty() {
            lines.push("## Types\n".to_string());
            for t in &relevant {
                lines.push(format!("- `{}` ({}): {}\n", t.name, t.path, t.description));
            }
        }
        let snapshot = lines.join("\n");
        if snapshot.len() > 5000 {
            Ok(snapshot[..5000].to_string() + "\n...(truncated)")
        } else {
            Ok(snapshot)
        }
    }

    pub fn all_types(&self) -> Result<Vec<TypeRecord>> {
        self.read_json("types.json")
    }

    fn read_json<T: serde::de::DeserializeOwned>(&self, filename: &str) -> Result<Vec<T>> {
        let path = self.root.join(filename);
        if !path.exists() {
            return Ok(vec![]);
        }
        let json = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&json).unwrap_or_default())
    }

    fn append_json_records<T: serde::Serialize>(
        &self,
        filename: &str,
        new_items: &[T],
    ) -> Result<()> {
        let path = self.root.join(filename);
        let mut existing: Vec<serde_json::Value> = if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            vec![]
        };
        for item in new_items {
            existing.push(serde_json::to_value(item)?);
        }
        std::fs::write(&path, serde_json::to_string_pretty(&existing)?)?;
        Ok(())
    }

    fn append_json_values(&self, filename: &str, new_items: &[serde_json::Value]) -> Result<()> {
        let path = self.root.join(filename);
        let mut existing: Vec<serde_json::Value> = if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            vec![]
        };
        existing.extend_from_slice(new_items);
        std::fs::write(&path, serde_json::to_string_pretty(&existing)?)?;
        Ok(())
    }
}
