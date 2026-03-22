use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;

/// Append a JSON event to `plans/context/events/{plan_num}-events.jsonl`.
pub fn log_event(
    repo_root: &Path,
    plan_num: &str,
    event_type: &str,
    kv: &[(&str, &str)],
) -> Result<()> {
    let events_dir = repo_root.join("plans/context/events");
    fs::create_dir_all(&events_dir)?;
    let path = events_dir.join(format!("{plan_num}-events.jsonl"));

    let mut obj = serde_json::Map::new();
    obj.insert(
        "ts".into(),
        serde_json::Value::String(Utc::now().to_rfc3339()),
    );
    obj.insert(
        "plan".into(),
        serde_json::Value::String(plan_num.to_string()),
    );
    obj.insert(
        "event".into(),
        serde_json::Value::String(event_type.to_string()),
    );
    for (k, v) in kv {
        obj.insert(k.to_string(), serde_json::Value::String(v.to_string()));
    }

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{}", serde_json::Value::Object(obj))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bardo-test-event-log-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn log_event_creates_file_and_appends() {
        let root = test_dir("append");

        log_event(
            &root,
            "05",
            "phase_transition",
            &[("from", "preflight"), ("to", "implementer")],
        )
        .unwrap();
        log_event(
            &root,
            "05",
            "gate_result",
            &[("gate", "compile"), ("passed", "true")],
        )
        .unwrap();

        let path = root.join("plans/context/events/05-events.jsonl");
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["event"], "phase_transition");
        assert_eq!(first["plan"], "05");
        assert_eq!(first["from"], "preflight");

        let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(second["event"], "gate_result");
        assert_eq!(second["gate"], "compile");

        let _ = std::fs::remove_dir_all(&root);
    }
}
