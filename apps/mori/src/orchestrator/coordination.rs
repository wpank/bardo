use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

const COORDINATION_DIRS: &[&str] = &[
    "plans/context/validations",
    "plans/context/spec-drift",
    "plans/context/coverage",
    "plans/context/benchmarks",
    "plans/context/regressions",
    "plans/context/integration",
    "plans/context/fix-plans",
    "plans/context/reconciliation",
    "plans/context/nits",
];

pub fn ensure_coordination_dirs(repo_root: &Path) -> Result<()> {
    for dir in COORDINATION_DIRS {
        std::fs::create_dir_all(repo_root.join(dir))?;
    }
    Ok(())
}

pub fn write_validation_result(
    repo_root: &Path,
    category: &str,
    name: &str,
    data: &impl Serialize,
) -> Result<PathBuf> {
    let dir = repo_root.join("plans/context").join(category);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{name}.json"));
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, json)?;
    Ok(path)
}

pub fn read_validation_results(repo_root: &Path, category: &str) -> Result<Vec<serde_json::Value>> {
    let dir = repo_root.join("plans/context").join(category);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut results = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(value) = serde_json::from_str(&content) {
                results.push(value);
            }
        }
    }
    Ok(results)
}
