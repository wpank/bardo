//! In-memory episodic vector store.
//!
//! Stores episodes with 768-dim embeddings and provides brute-force cosine similarity search.
//! A production implementation would back this with LanceDB + IVF-PQ ANN indexing;
//! this in-memory version provides the same interface for development and testing.

use std::sync::Mutex;

use uuid::Uuid;

use crate::entry::Episode;
use crate::error::GrimoireError;
use crate::retrieval::cosine_similarity;

/// Required embedding dimensionality (nomic-embed-text-v1.5).
pub const EMBEDDING_DIM: usize = 768;

/// Episode count threshold for ANN index build.
pub const INDEX_BUILD_THRESHOLD: usize = 1000;

/// Number of IVF partitions for ANN index.
pub const IVF_PARTITIONS: usize = 256;

/// Number of PQ sub-vectors for ANN index.
pub const PQ_SUB_VECTORS: usize = 16;

/// In-memory episodic vector store.
///
/// Stores episodes and provides brute-force cosine similarity search.
/// Thread-safe via internal `Mutex`.
pub struct EpisodicStore {
    episodes: Mutex<Vec<Episode>>,
    index_built: Mutex<bool>,
}

impl EpisodicStore {
    /// Create a new empty episodic store.
    pub fn new() -> Self {
        Self {
            episodes: Mutex::new(Vec::new()),
            index_built: Mutex::new(false),
        }
    }

    /// Write an episode to the store. Validates 768-dim embedding.
    pub fn write_episode(&self, episode: &Episode) -> Result<(), GrimoireError> {
        if episode.vector.len() != EMBEDDING_DIM {
            return Err(GrimoireError::InvalidState(format!(
                "embedding dimension {} != expected {EMBEDDING_DIM}",
                episode.vector.len()
            )));
        }

        let mut episodes = self
            .episodes
            .lock()
            .map_err(|e| GrimoireError::StorageUnavailable(e.to_string()))?;
        episodes.push(episode.clone());

        // Check if we should build the ANN index.
        if episodes.len() > INDEX_BUILD_THRESHOLD {
            let mut index_built = self
                .index_built
                .lock()
                .map_err(|e| GrimoireError::StorageUnavailable(e.to_string()))?;
            if !*index_built {
                tracing::info!(
                    "Episode count {} exceeds threshold {INDEX_BUILD_THRESHOLD}, ANN index would be built",
                    episodes.len()
                );
                *index_built = true;
            }
        }

        Ok(())
    }

    /// Search for the top-k most similar episodes by cosine similarity.
    ///
    /// Returns `(episode_id, distance)` pairs sorted by descending similarity.
    pub fn search(
        &self,
        query_vector: &[f32],
        top_k: usize,
    ) -> Result<Vec<(Uuid, f32)>, GrimoireError> {
        let episodes = self
            .episodes
            .lock()
            .map_err(|e| GrimoireError::StorageUnavailable(e.to_string()))?;

        let mut scored: Vec<(Uuid, f32)> = episodes
            .iter()
            .map(|ep| {
                let sim = cosine_similarity(query_vector, &ep.vector);
                (ep.id, sim)
            })
            .collect();

        // Sort by similarity descending.
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored)
    }

    /// Returns the total episode count.
    pub fn episode_count(&self) -> Result<usize, GrimoireError> {
        let episodes = self
            .episodes
            .lock()
            .map_err(|e| GrimoireError::StorageUnavailable(e.to_string()))?;
        Ok(episodes.len())
    }

    /// Returns whether the ANN index has been (logically) built.
    pub fn is_index_built(&self) -> Result<bool, GrimoireError> {
        let built = self
            .index_built
            .lock()
            .map_err(|e| GrimoireError::StorageUnavailable(e.to_string()))?;
        Ok(*built)
    }

    /// Get recent episodes by tick, returning up to `limit` episodes.
    pub fn recent_episodes(
        &self,
        _current_tick: u64,
        limit: usize,
    ) -> Result<Vec<Episode>, GrimoireError> {
        let episodes = self
            .episodes
            .lock()
            .map_err(|e| GrimoireError::StorageUnavailable(e.to_string()))?;

        // Return the most recent episodes (last N by insertion order).
        let start = episodes.len().saturating_sub(limit);
        Ok(episodes[start..].to_vec())
    }
}

impl Default for EpisodicStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::Episode;
    use golem_core::id::GolemId;
    use uuid::Uuid;

    fn make_episode(vector: Vec<f32>) -> Episode {
        Episode {
            id: Uuid::now_v7(),
            golem_id: GolemId::new(),
            text: "test episode".to_string(),
            vector,
            tool: "test".to_string(),
            outcome: "neutral".to_string(),
            tick_id: 0,
            importance: 0.5,
            emotional_arousal: 0.3,
            pad_pleasure: 0.0,
            pad_arousal: 0.0,
            pad_dominance: 0.0,
            timestamp_ms: 0,
            is_bloodstain: false,
            consolidated: false,
        }
    }

    // INV-017: All embeddings are 768-dimensional.
    #[test]
    fn test_embedding_dimensionality() {
        let store = EpisodicStore::new();

        // Valid 768-dim embedding should succeed.
        let ep = make_episode(vec![0.1; EMBEDDING_DIM]);
        assert!(store.write_episode(&ep).is_ok());

        // Wrong dimension should fail.
        let bad_ep = make_episode(vec![0.1; 512]);
        assert!(store.write_episode(&bad_ep).is_err());
    }

    #[test]
    fn test_embedding_norm_stability() {
        let store = EpisodicStore::new();

        // Write an episode with a known-norm vector.
        let mut vec = vec![0.0; EMBEDDING_DIM];
        vec[0] = 1.0;
        let ep = make_episode(vec.clone());
        store.write_episode(&ep).is_ok();

        // Search with the same vector should find it with similarity ~1.0.
        let results = store.search(&vec, 1).expect("search");
        assert_eq!(results.len(), 1);
        assert!((results[0].1 - 1.0).abs() < 0.01);
    }

    // INV-018: ANN index build threshold.
    #[test]
    fn test_lance_index_deferred_build() {
        let store = EpisodicStore::new();

        // Before threshold: no index.
        for _ in 0..INDEX_BUILD_THRESHOLD {
            let ep = make_episode(vec![0.1; EMBEDDING_DIM]);
            store.write_episode(&ep).expect("write");
        }
        assert!(!store.is_index_built().expect("check"));

        // One more pushes past the threshold.
        let ep = make_episode(vec![0.1; EMBEDDING_DIM]);
        store.write_episode(&ep).expect("write");
        assert!(store.is_index_built().expect("check"));
    }

    #[test]
    fn test_index_parameters() {
        // Verify the constants match the plan specification.
        assert_eq!(INDEX_BUILD_THRESHOLD, 1000);
        assert_eq!(IVF_PARTITIONS, 256);
        assert_eq!(PQ_SUB_VECTORS, 16);
    }

    #[test]
    fn test_search_returns_sorted() {
        let store = EpisodicStore::new();

        // Write two episodes with different vectors.
        let mut v1 = vec![0.0; EMBEDDING_DIM];
        v1[0] = 1.0; // Aligned with query.
        let ep1 = make_episode(v1);
        store.write_episode(&ep1).expect("write");

        let mut v2 = vec![0.0; EMBEDDING_DIM];
        v2[1] = 1.0; // Orthogonal to query.
        let ep2 = make_episode(v2);
        store.write_episode(&ep2).expect("write");

        // Query aligned with ep1.
        let mut query = vec![0.0; EMBEDDING_DIM];
        query[0] = 1.0;
        let results = store.search(&query, 2).expect("search");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, ep1.id, "ep1 should be first (most similar)");
        assert!(
            results[0].1 > results[1].1,
            "results should be sorted by similarity"
        );
    }
}
