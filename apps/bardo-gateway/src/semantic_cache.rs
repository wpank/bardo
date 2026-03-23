//! Semantic cache (L2) with configurable backend.
//!
//! Two backends:
//! - **SimHash** (default): 64-bit fingerprint, Hamming distance matching.
//!   Pure CPU, ~50us for 10K entries, no model download. Good enough for
//!   catching rephrased identical questions.
//! - **Embedding** (opt-in, requires `embedding` feature): fastembed ONNX
//!   embeddings with brute-force cosine similarity. ~3-5ms per embedding,
//!   ~100MB model download. Much more accurate for semantic similarity.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bytes::Bytes;
use dashmap::DashMap;
use serde_json::Value;

/// Maximum Hamming distance for a semantic match (0 = exact, 5 = loose).
/// With system prompts hashed to a fixed token, the user message dominates
/// the SimHash shingles. Threshold 3 catches minor wording differences while
/// keeping false positives low for agent workloads.
const DEFAULT_SIMHASH_THRESHOLD: u32 = 3;

/// Default cosine similarity threshold for the embedding backend.
const DEFAULT_EMBEDDING_THRESHOLD: f32 = 0.92;

/// Maximum entries before evicting oldest.
const DEFAULT_MAX_ENTRIES: usize = 5000;

/// Default TTL for cache entries (30 minutes).
/// Agent sessions can span hours; a hit from 10 minutes ago is still useful.
const DEFAULT_TTL: Duration = Duration::from_secs(1800);

/// Which similarity backend powers the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheBackend {
    /// 64-bit SimHash with Hamming distance. Fast, approximate.
    SimHash,
    /// ONNX embedding with cosine similarity. Accurate, heavier.
    Embedding,
}

/// A cached response stored under the SimHash backend.
struct SimHashEntry {
    response: Bytes,
    cost_usd: f64,
    model: String,
    created_at: Instant,
    ttl: Duration,
}

/// A cached response stored under the embedding backend.
#[cfg(feature = "embedding")]
struct EmbeddingEntry {
    response: Bytes,
    cost_usd: f64,
    model: String,
    created_at: Instant,
    ttl: Duration,
}

/// Result of a semantic cache lookup or iteration.
pub struct SemanticHit {
    /// The SimHash fingerprint (0 for embedding-only hits).
    pub fingerprint: u64,
    /// The cached response body.
    pub response: Bytes,
    /// Cost attributed to the original request.
    pub cost_usd: f64,
    /// Model that produced the response.
    pub model: String,
}

/// Dual-backend semantic cache.
///
/// Always has the SimHash path available. When compiled with the `embedding`
/// feature and initialization succeeds, the embedding path takes priority.
pub struct SemanticCache {
    // ── SimHash state (always available) ────────────────────────────
    simhash_entries: DashMap<u64, SimHashEntry>,
    simhash_threshold: u32,

    // ── Embedding state (feature-gated) ────────────────────────────
    #[cfg(feature = "embedding")]
    embedding_model: Option<fastembed::TextEmbedding>,
    #[cfg(feature = "embedding")]
    embedding_entries: DashMap<u64, EmbeddingEntry>,
    #[cfg(feature = "embedding")]
    embeddings: parking_lot::RwLock<Vec<(u64, Vec<f32>)>>,

    /// Active backend.
    backend: CacheBackend,
    /// Cosine similarity threshold (embedding backend).
    #[allow(dead_code)]
    threshold: f32,
    /// Maximum entries across both backends.
    max_entries: usize,
    /// Monotonic id generator for embedding entries.
    next_id: AtomicU64,
    /// TTL applied to new entries.
    ttl: Duration,
}

impl SemanticCache {
    /// Create a new cache using the SimHash backend.
    pub fn new() -> Self {
        Self {
            simhash_entries: DashMap::new(),
            simhash_threshold: DEFAULT_SIMHASH_THRESHOLD,
            #[cfg(feature = "embedding")]
            embedding_model: None,
            #[cfg(feature = "embedding")]
            embedding_entries: DashMap::new(),
            #[cfg(feature = "embedding")]
            embeddings: parking_lot::RwLock::new(Vec::new()),
            backend: CacheBackend::SimHash,
            threshold: DEFAULT_EMBEDDING_THRESHOLD,
            max_entries: DEFAULT_MAX_ENTRIES,
            next_id: AtomicU64::new(0),
            ttl: DEFAULT_TTL,
        }
    }

    /// Create a new cache that tries to use the embedding backend.
    ///
    /// Falls back to SimHash if fastembed initialization fails or if the
    /// `embedding` feature is not compiled in.
    #[cfg(feature = "embedding")]
    pub fn new_with_embeddings(threshold: f32, max_entries: usize) -> Self {
        let model = fastembed::TextEmbedding::try_new(
            fastembed::InitOptions::new(fastembed::EmbeddingModel::NomicEmbedTextV15)
                .with_show_download_progress(true),
        )
        .ok();

        let backend = if model.is_some() {
            tracing::info!("Semantic cache: embedding backend ready (nomic-embed-text-v1.5)");
            CacheBackend::Embedding
        } else {
            tracing::warn!("Semantic cache: fastembed init failed, falling back to SimHash");
            CacheBackend::SimHash
        };

        Self {
            simhash_entries: DashMap::new(),
            simhash_threshold: DEFAULT_SIMHASH_THRESHOLD,
            embedding_model: model,
            embedding_entries: DashMap::new(),
            embeddings: parking_lot::RwLock::new(Vec::new()),
            backend,
            threshold,
            max_entries,
            next_id: AtomicU64::new(0),
            ttl: DEFAULT_TTL,
        }
    }

    /// Which backend is active.
    pub fn backend(&self) -> CacheBackend {
        self.backend
    }

    // ── Text extraction ────────────────────────────────────────────

    /// Returns true if the last user message contains tool_result blocks.
    ///
    /// Tool-result follow-ups are context-dependent (the response depends on
    /// the specific tool output, not just the user's text). Caching these
    /// produces false matches because `extract_cache_text` can't distinguish
    /// different tool results — they all have empty text content.
    pub fn last_message_has_tool_results(body: &Value) -> bool {
        body.get("messages")
            .and_then(|m| m.as_array())
            .and_then(|arr| {
                arr.iter()
                    .rev()
                    .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
            })
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
            .is_some_and(|blocks| {
                blocks
                    .iter()
                    .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"))
            })
    }

    /// Extract the semantically significant text from a request body.
    ///
    /// Used as input to both SimHash and embedding backends. Concatenates
    /// the model name, system prompt, and last user message.
    pub fn extract_cache_text(body: &Value) -> String {
        // Hash the system prompt to a fixed-length token instead of including
        // the full text. Long system prompts (thousands of chars) otherwise
        // dominate the SimHash shingles, drowning out the user message and
        // causing unrelated questions to match.
        let system_hash = {
            let sys_text = body
                .get("system")
                .map(|s| match s {
                    Value::String(s) => s.as_str(),
                    Value::Array(arr) => arr
                        .last()
                        .and_then(|b| b.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or(""),
                    _ => "",
                })
                .unwrap_or("");
            let mut h = DefaultHasher::new();
            sys_text.hash(&mut h);
            format!("{:016x}", h.finish())
        };

        let last_user = body
            .get("messages")
            .and_then(|m| m.as_array())
            .and_then(|arr| {
                arr.iter()
                    .rev()
                    .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
            })
            .and_then(|m| m.get("content"))
            .map(|c| match c {
                Value::String(s) => s.clone(),
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<&str>>()
                    .join(" "),
                _ => String::new(),
            })
            .unwrap_or_default();

        let model = body.get("model").and_then(|m| m.as_str()).unwrap_or("");

        format!("{model}\n{system_hash}\n{last_user}")
    }

    // ── Legacy SimHash API (kept for persistence compatibility) ────

    /// Compute the SimHash fingerprint of a request.
    pub fn fingerprint(body: &Value) -> u64 {
        simhash(&Self::extract_cache_text(body))
    }

    /// Look up a semantically similar cached response by SimHash fingerprint.
    ///
    /// Scans all SimHash entries for Hamming distance within threshold.
    /// For 5K entries this is ~50us (u64 XOR + popcount per entry).
    pub fn get(&self, fingerprint: u64) -> Option<SemanticHit> {
        let now = Instant::now();
        for entry in self.simhash_entries.iter() {
            let e = entry.value();
            // Skip expired entries.
            if now.duration_since(e.created_at) > e.ttl {
                continue;
            }
            let distance = (fingerprint ^ *entry.key()).count_ones();
            if distance <= self.simhash_threshold {
                return Some(SemanticHit {
                    fingerprint,
                    response: e.response.clone(),
                    cost_usd: e.cost_usd,
                    model: e.model.clone(),
                });
            }
        }
        None
    }

    /// Store a response in the SimHash cache.
    pub fn put(&self, fingerprint: u64, response: Bytes, cost_usd: f64, model: String) {
        self.evict_simhash_if_full();
        self.simhash_entries.insert(
            fingerprint,
            SimHashEntry {
                response,
                cost_usd,
                model,
                created_at: Instant::now(),
                ttl: self.ttl,
            },
        );
    }

    // ── Backend-transparent API ────────────────────────────────────

    /// Look up a cached response using whichever backend is active.
    ///
    /// On the SimHash path this computes a fingerprint and scans by Hamming
    /// distance. On the embedding path it generates an embedding vector and
    /// does brute-force cosine similarity.
    pub fn lookup(&self, text: &str) -> Option<SemanticHit> {
        match self.backend {
            CacheBackend::SimHash => {
                let fp = simhash(text);
                self.get(fp)
            }
            #[cfg(feature = "embedding")]
            CacheBackend::Embedding => self.lookup_embedding(text),
            #[cfg(not(feature = "embedding"))]
            CacheBackend::Embedding => {
                // Should never happen, but fall back to SimHash.
                let fp = simhash(text);
                self.get(fp)
            }
        }
    }

    /// Store a response using whichever backend is active.
    ///
    /// Always stores in SimHash as well (for persistence and fallback).
    /// Skips caching if the response contains tool_use blocks — replaying
    /// those produces invalid tool_use IDs on subsequent turns.
    pub fn store(&self, text: &str, response: Bytes, cost_usd: f64, model: String) {
        // Never cache tool_use responses. The IDs are unique per request;
        // replaying them causes 400s from the API on the next turn.
        if let Ok(body) = serde_json::from_slice::<Value>(&response) {
            if body.get("stop_reason").and_then(|v| v.as_str()) == Some("tool_use") {
                return;
            }
            // Also check content blocks directly in case stop_reason is missing.
            if let Some(content) = body.get("content").and_then(|c| c.as_array()) {
                if content
                    .iter()
                    .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
                {
                    return;
                }
            }
        }

        // Always populate SimHash for persistence compatibility.
        let fp = simhash(text);
        self.put(fp, response.clone(), cost_usd, model.clone());

        #[cfg(feature = "embedding")]
        if self.backend == CacheBackend::Embedding {
            self.store_embedding(text, response, cost_usd, model);
        }
    }

    // ── Embedding backend internals ────────────────────────────────

    /// Embed a text string, returning the vector. Returns `None` if the
    /// embedding model is unavailable or embedding fails.
    #[cfg(feature = "embedding")]
    fn embed(&self, text: &str) -> Option<Vec<f32>> {
        let model = self.embedding_model.as_ref()?;
        let results = model.embed(vec![text.to_string()], None).ok()?;
        results.into_iter().next()
    }

    #[cfg(feature = "embedding")]
    fn lookup_embedding(&self, text: &str) -> Option<SemanticHit> {
        let query_vec = self.embed(text)?;
        let now = Instant::now();

        let store = self.embeddings.read();
        let mut best_sim: f32 = -1.0;
        let mut best_id: Option<u64> = None;

        for (id, vec) in store.iter() {
            // Check expiry before computing similarity.
            if let Some(entry) = self.embedding_entries.get(id) {
                if now.duration_since(entry.created_at) > entry.ttl {
                    continue;
                }
            } else {
                continue;
            }

            let sim = cosine_similarity(&query_vec, vec);
            if sim > best_sim {
                best_sim = sim;
                best_id = Some(*id);
            }
        }

        if best_sim >= self.threshold {
            let id = best_id?;
            let entry = self.embedding_entries.get(&id)?;
            Some(SemanticHit {
                fingerprint: 0,
                response: entry.response.clone(),
                cost_usd: entry.cost_usd,
                model: entry.model.clone(),
            })
        } else {
            None
        }
    }

    #[cfg(feature = "embedding")]
    fn store_embedding(&self, text: &str, response: Bytes, cost_usd: f64, model: String) {
        let vec = match self.embed(text) {
            Some(v) => v,
            None => return,
        };

        self.evict_embedding_if_full();

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.embedding_entries.insert(
            id,
            EmbeddingEntry {
                response,
                cost_usd,
                model,
                created_at: Instant::now(),
                ttl: self.ttl,
            },
        );
        self.embeddings.write().push((id, vec));
    }

    // ── Eviction ───────────────────────────────────────────────────

    fn evict_simhash_if_full(&self) {
        if self.simhash_entries.len() >= self.max_entries {
            let oldest = self
                .simhash_entries
                .iter()
                .min_by_key(|e| e.value().created_at)
                .map(|e| *e.key());
            if let Some(key) = oldest {
                self.simhash_entries.remove(&key);
            }
        }
    }

    #[cfg(feature = "embedding")]
    fn evict_embedding_if_full(&self) {
        if self.embedding_entries.len() >= self.max_entries {
            let oldest = self
                .embedding_entries
                .iter()
                .min_by_key(|e| e.value().created_at)
                .map(|e| *e.key());
            if let Some(key) = oldest {
                self.embedding_entries.remove(&key);
                let mut store = self.embeddings.write();
                store.retain(|(id, _)| *id != key);
            }
        }
    }

    // ── Metrics / iteration ────────────────────────────────────────

    /// Number of SimHash entries in the cache.
    pub fn len(&self) -> usize {
        self.simhash_entries.len()
    }

    /// Total entries across all backends.
    pub fn total_entries(&self) -> usize {
        let mut n = self.simhash_entries.len();
        #[cfg(feature = "embedding")]
        {
            n += self.embedding_entries.len();
        }
        n
    }

    /// Iterate over all SimHash cache entries (for persistence).
    pub fn iter(&self) -> impl Iterator<Item = SemanticHit> + '_ {
        self.simhash_entries.iter().map(|e| SemanticHit {
            fingerprint: *e.key(),
            response: e.value().response.clone(),
            cost_usd: e.value().cost_usd,
            model: e.value().model.clone(),
        })
    }
}

// ── SimHash implementation ─────────────────────────────────────────

/// SimHash: locality-sensitive hash that preserves similarity.
///
/// Splits text into 3-gram shingles, hashes each shingle, and combines
/// using a weighted bit-voting scheme. Similar texts produce fingerprints
/// with low Hamming distance.
fn simhash(text: &str) -> u64 {
    let mut counts = [0i32; 64];
    let chars: Vec<char> = text.chars().collect();

    if chars.len() < 3 {
        // Too short for shingles — fall back to regular hash.
        let mut h = DefaultHasher::new();
        text.hash(&mut h);
        return h.finish();
    }

    // Generate 3-character shingles and hash each.
    for window in chars.windows(3) {
        let shingle: String = window.iter().collect();
        let mut h = DefaultHasher::new();
        shingle.hash(&mut h);
        let hash = h.finish();

        // Vote on each bit position.
        for bit in 0..64 {
            if (hash >> bit) & 1 == 1 {
                counts[bit] += 1;
            } else {
                counts[bit] -= 1;
            }
        }
    }

    // Build fingerprint from majority votes.
    let mut fingerprint: u64 = 0;
    for bit in 0..64 {
        if counts[bit] > 0 {
            fingerprint |= 1 << bit;
        }
    }

    fingerprint
}

// ── Cosine similarity ──────────────────────────────────────────────

/// Brute-force cosine similarity between two vectors.
///
/// Returns a value in \[-1, 1\]. For normalized embeddings this is just the
/// dot product, but we normalize defensively.
#[cfg(feature = "embedding")]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── SimHash unit tests (existing) ──────────────────────────────

    #[test]
    fn similar_texts_have_low_hamming_distance() {
        let a = simhash("implement OAuth2 provider with Google authentication");
        let b = simhash("implement OAuth2 provider with GitHub authentication");
        let distance = (a ^ b).count_ones();
        assert!(distance <= 10, "distance was {distance}, expected <= 10");
    }

    #[test]
    fn different_texts_have_high_hamming_distance() {
        let a = simhash("implement OAuth2 provider with Google authentication");
        let b = simhash("fix the database migration script for PostgreSQL");
        let distance = (a ^ b).count_ones();
        assert!(distance > 10, "distance was {distance}, expected > 10");
    }

    #[test]
    fn identical_texts_have_zero_distance() {
        let a = simhash("the quick brown fox jumps over the lazy dog");
        let b = simhash("the quick brown fox jumps over the lazy dog");
        assert_eq!(a, b);
    }

    // ── TTL tests ──────────────────────────────────────────────────

    #[test]
    fn ttl_expiration() {
        let cache = SemanticCache {
            simhash_entries: DashMap::new(),
            simhash_threshold: DEFAULT_SIMHASH_THRESHOLD,
            #[cfg(feature = "embedding")]
            embedding_model: None,
            #[cfg(feature = "embedding")]
            embedding_entries: DashMap::new(),
            #[cfg(feature = "embedding")]
            embeddings: parking_lot::RwLock::new(Vec::new()),
            backend: CacheBackend::SimHash,
            threshold: DEFAULT_EMBEDDING_THRESHOLD,
            max_entries: DEFAULT_MAX_ENTRIES,
            next_id: AtomicU64::new(0),
            ttl: DEFAULT_TTL,
        };

        let text = "what is the meaning of life";
        let fp = simhash(text);
        let response = Bytes::from_static(b"42");

        // Insert an entry with 0-second TTL (already expired).
        cache.simhash_entries.insert(
            fp,
            SimHashEntry {
                response,
                cost_usd: 0.001,
                model: "test".into(),
                created_at: Instant::now() - Duration::from_secs(1),
                ttl: Duration::from_secs(0),
            },
        );

        assert!(
            cache.get(fp).is_none(),
            "expired entry should not be returned"
        );
        assert_eq!(cache.len(), 1, "expired entry still in map until GC");
    }

    // ── Text extraction tests ──────────────────────────────────────

    #[test]
    fn cache_text_extraction_basic() {
        let body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "system": "You are a helpful assistant.",
            "messages": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there!"},
                {"role": "user", "content": "What is Rust?"}
            ]
        });

        let text = SemanticCache::extract_cache_text(&body);
        assert!(text.contains("claude-sonnet-4-20250514"));
        // System prompt is hashed to a fixed token, not included verbatim.
        assert!(!text.contains("You are a helpful assistant."));
        assert!(text.contains("What is Rust?"));
        // Should pick last user message, not first.
        assert!(!text.contains("Hello"));
    }

    #[test]
    fn cache_text_extraction_different_system_prompts() {
        let body_a = serde_json::json!({
            "model": "test-model",
            "system": "You are a helpful assistant.",
            "messages": [{"role": "user", "content": "hello"}]
        });
        let body_b = serde_json::json!({
            "model": "test-model",
            "system": "You are a coding assistant.",
            "messages": [{"role": "user", "content": "hello"}]
        });

        let text_a = SemanticCache::extract_cache_text(&body_a);
        let text_b = SemanticCache::extract_cache_text(&body_b);
        // Same user message but different system prompts should produce
        // different cache texts (system hash differs).
        assert_ne!(text_a, text_b);
    }

    #[test]
    fn cache_text_extraction_array_system() {
        let body = serde_json::json!({
            "model": "test-model",
            "system": [
                {"type": "text", "text": "first block"},
                {"type": "text", "text": "second block"}
            ],
            "messages": [
                {"role": "user", "content": "question"}
            ]
        });

        let text = SemanticCache::extract_cache_text(&body);
        // System prompt is hashed, not included verbatim.
        assert!(!text.contains("second block"));
        assert!(text.contains("question"));
    }

    #[test]
    fn cache_text_extraction_missing_fields() {
        let body = serde_json::json!({});
        let text = SemanticCache::extract_cache_text(&body);
        // Should not panic. Format: "{model}\n{system_hash}\n{last_user}"
        // With empty fields: "\n{hash_of_empty_string}\n"
        assert!(text.starts_with('\n'));
        assert!(text.ends_with('\n'));
        // The middle segment is a 16-char hex hash of "".
        let parts: Vec<&str> = text.split('\n').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[1].len(), 16);
    }

    // ── Backend selection tests ────────────────────────────────────

    #[test]
    fn default_backend_is_simhash() {
        let cache = SemanticCache::new();
        assert_eq!(cache.backend(), CacheBackend::SimHash);
    }

    // ── lookup/store round-trip via SimHash ────────────────────────

    #[test]
    fn lookup_store_roundtrip_simhash() {
        let cache = SemanticCache::new();
        let text = "claude-sonnet-4-20250514\nYou are helpful.\nExplain monads";
        let response = Bytes::from_static(b"A monad is a monoid in the category of endofunctors.");

        cache.store(
            text,
            response.clone(),
            0.005,
            "claude-sonnet-4-20250514".into(),
        );

        let hit = cache.lookup(text);
        assert!(hit.is_some(), "exact text should match via SimHash");
        let hit = hit.expect("just asserted Some");
        assert_eq!(hit.response, response);
        assert_eq!(hit.model, "claude-sonnet-4-20250514");
    }

    // ── Cosine similarity (embedding feature only) ─────────────────

    #[cfg(feature = "embedding")]
    mod embedding_tests {
        use super::*;

        #[test]
        fn cosine_similarity_identical() {
            let a = vec![1.0, 0.0, 0.0];
            let sim = cosine_similarity(&a, &a);
            assert!(
                (sim - 1.0).abs() < 1e-6,
                "identical vectors should have similarity 1.0, got {sim}"
            );
        }

        #[test]
        fn cosine_similarity_orthogonal() {
            let a = vec![1.0, 0.0];
            let b = vec![0.0, 1.0];
            let sim = cosine_similarity(&a, &b);
            assert!(
                sim.abs() < 1e-6,
                "orthogonal vectors should have similarity ~0, got {sim}"
            );
        }

        #[test]
        fn embedding_backend_falls_back_on_no_model() {
            // Build with embedding feature but don't initialize a model.
            let cache = SemanticCache {
                simhash_entries: DashMap::new(),
                simhash_threshold: DEFAULT_SIMHASH_THRESHOLD,
                embedding_model: None,
                embedding_entries: DashMap::new(),
                embeddings: parking_lot::RwLock::new(Vec::new()),
                backend: CacheBackend::SimHash, // fallback
                threshold: DEFAULT_EMBEDDING_THRESHOLD,
                max_entries: DEFAULT_MAX_ENTRIES,
                next_id: AtomicU64::new(0),
                ttl: DEFAULT_TTL,
            };

            assert_eq!(cache.backend(), CacheBackend::SimHash);
        }
    }
}
