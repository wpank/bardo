//! Three-substrate storage: episodic (vector), semantic (SQLite), playbook (filesystem).

pub mod episodic;
pub mod playbook;
pub mod semantic;

use std::sync::Arc;

use crate::error::GrimoireError;

/// Holds references to all three Grimoire substrates.
pub struct SubstrateSet {
    /// In-memory vector store for episodes.
    pub episodic: Arc<episodic::EpisodicStore>,
    /// SQLite semantic store.
    pub semantic: Arc<semantic::SemanticStore>,
    /// PLAYBOOK.md filesystem store.
    pub playbook: Arc<playbook::PlaybookStore>,
}

impl SubstrateSet {
    /// Open all three substrates from a data directory.
    pub async fn open(data_dir: &std::path::Path) -> Result<Self, GrimoireError> {
        std::fs::create_dir_all(data_dir)?;

        let episodic = Arc::new(episodic::EpisodicStore::new());
        let semantic = Arc::new(semantic::SemanticStore::open(data_dir)?);
        let playbook = Arc::new(playbook::PlaybookStore::open(data_dir).await?);

        Ok(Self {
            episodic,
            semantic,
            playbook,
        })
    }

    /// Open all substrates in memory (for testing).
    pub async fn open_memory(tmp_dir: &std::path::Path) -> Result<Self, GrimoireError> {
        let episodic = Arc::new(episodic::EpisodicStore::new());
        let semantic = Arc::new(semantic::SemanticStore::open_memory()?);
        let playbook = Arc::new(playbook::PlaybookStore::open(tmp_dir).await?);

        Ok(Self {
            episodic,
            semantic,
            playbook,
        })
    }
}
