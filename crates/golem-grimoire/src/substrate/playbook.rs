//! PLAYBOOK.md filesystem store (procedural memory).
//!
//! Machine-evolved, written only by the Curator cycle.
//! Uses atomic writes via tempfile rename to prevent corruption.

use std::path::{Path, PathBuf};

use crate::error::GrimoireError;

/// Update directive for PLAYBOOK.md.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlaybookUpdate {
    /// Target section (None = global).
    pub section: Option<String>,
    /// Content to write.
    pub content: String,
    /// Current tick.
    pub tick: u64,
}

/// Filesystem store for PLAYBOOK.md (procedural memory).
pub struct PlaybookStore {
    path: PathBuf,
}

impl PlaybookStore {
    /// Open or create PLAYBOOK.md in `data_dir`.
    pub async fn open(data_dir: &Path) -> Result<Self, GrimoireError> {
        let path = data_dir.join("PLAYBOOK.md");

        if !path.exists() {
            let header = "# PLAYBOOK\n\n\
                          > Machine-evolved. Written only by the Curator cycle.\n\
                          > Last updated: tick 0\n\n\
                          ## Global\n\n";
            tokio::fs::write(&path, header)
                .await
                .map_err(|e| GrimoireError::Playbook(e.to_string()))?;
        }

        Ok(Self { path })
    }

    /// Read full PLAYBOOK.md content.
    pub async fn read(&self) -> Result<String, GrimoireError> {
        tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| GrimoireError::Playbook(e.to_string()))
    }

    /// Write full PLAYBOOK.md (atomic via tempfile rename).
    pub async fn write(&self, content: &str) -> Result<(), GrimoireError> {
        let temp_path = self.path.with_extension("md.tmp");
        tokio::fs::write(&temp_path, content)
            .await
            .map_err(|e| GrimoireError::Playbook(e.to_string()))?;
        tokio::fs::rename(&temp_path, &self.path)
            .await
            .map_err(|e| GrimoireError::Playbook(e.to_string()))?;
        Ok(())
    }

    /// Apply a `PlaybookUpdate` (append to section or global).
    pub async fn apply_update(&self, update: &PlaybookUpdate) -> Result<(), GrimoireError> {
        let mut content = self.read().await?;

        if let Some(section_name) = &update.section {
            let section_header = format!("\n## {section_name}\n\n");
            if content.contains(&section_header) {
                // Find end of section (next ## or end of file).
                if let Some(start) = content.find(&section_header) {
                    let after_header = start + section_header.len();
                    let next_section = content[after_header..].find("\n## ");
                    let end = match next_section {
                        Some(offset) => after_header + offset,
                        None => content.len(),
                    };
                    content.replace_range(after_header..end, &update.content);
                }
            } else {
                content.push_str(&section_header);
                content.push_str(&update.content);
                content.push('\n');
            }
        } else {
            // Global section: append after "## Global\n\n".
            let global_marker = "## Global\n\n";
            if let Some(pos) = content.find(global_marker) {
                let insert_pos = pos + global_marker.len();
                content.insert_str(insert_pos, &format!("{}\n\n", update.content));
            }
        }

        // Update tick timestamp.
        if let Some(pos) = content.find("> Last updated: tick") {
            let end_pos = content[pos..]
                .find('\n')
                .map_or(content.len(), |offset| pos + offset);
            content.replace_range(
                pos..end_pos,
                &format!("> Last updated: tick {}", update.tick),
            );
        }

        self.write(&content).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_playbook_create_and_read() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let store = PlaybookStore::open(dir.path()).await.expect("open");
        let content = store.read().await.expect("read");
        assert!(content.contains("# PLAYBOOK"));
        assert!(content.contains("## Global"));
    }

    #[tokio::test]
    async fn test_playbook_apply_update_global() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let store = PlaybookStore::open(dir.path()).await.expect("open");

        store
            .apply_update(&PlaybookUpdate {
                section: None,
                content: "Always check gas before swapping.".to_string(),
                tick: 50,
            })
            .await
            .expect("update");

        let content = store.read().await.expect("read");
        assert!(content.contains("Always check gas before swapping."));
        assert!(content.contains("tick 50"));
    }

    #[tokio::test]
    async fn test_playbook_apply_update_section() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let store = PlaybookStore::open(dir.path()).await.expect("open");

        store
            .apply_update(&PlaybookUpdate {
                section: Some("Strategy: uniswap-v3-lp".to_string()),
                content: "Rebalance when price moves >5%.".to_string(),
                tick: 100,
            })
            .await
            .expect("update");

        let content = store.read().await.expect("read");
        assert!(content.contains("## Strategy: uniswap-v3-lp"));
        assert!(content.contains("Rebalance when price moves >5%."));
    }
}
