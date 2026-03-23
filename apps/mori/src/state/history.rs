use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::agent::AgentRole;

/// Persisted thread IDs for crash recovery
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreadHistory {
    pub threads: HashMap<String, String>, // role_label -> thread_id
    pub last_plan: String,
    pub last_phase: String,
    pub last_iteration: u32,
}

impl ThreadHistory {
    pub fn path(repo_root: &Path) -> PathBuf {
        crate::orchestrator::paths::runs_dir(repo_root).join("thread-history.json")
    }

    pub fn load(repo_root: &Path) -> Result<Self> {
        let path = Self::path(repo_root);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let path = Self::path(repo_root);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn set_thread(&mut self, role: AgentRole, thread_id: &str) {
        self.threads
            .insert(role.label().to_string(), thread_id.to_string());
    }

    pub fn get_thread(&self, role: AgentRole) -> Option<&str> {
        self.threads.get(role.label()).map(|s| s.as_str())
    }

    pub fn update_position(&mut self, plan: &str, phase: &str, iteration: u32) {
        self.last_plan = plan.to_string();
        self.last_phase = phase.to_string();
        self.last_iteration = iteration;
    }
}
