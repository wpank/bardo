//! Semantic cache (L2) using SimHash locality-sensitive hashing.
//!
//! Sits between the hash cache (exact match) and the provider call.
//! Catches requests that are semantically similar but not byte-identical —
//! e.g., same system prompt with slightly different conversation history
//! asking essentially the same question.
//!
//! Uses SimHash: hash the system prompt + last user message into a 64-bit
//! fingerprint. Requests with Hamming distance ≤ threshold are cache hits.
//! No external model needed — pure CPU, ~50μs for 10K entries.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use bytes::Bytes;
use dashmap::DashMap;
use serde_json::Value;

/// Maximum Hamming distance for a semantic match (0 = exact, 5 = loose).
const DEFAULT_THRESHOLD: u32 = 4;

/// Maximum entries before evicting oldest.
const DEFAULT_MAX_ENTRIES: usize = 5000;

pub struct SemanticCache {
    entries: DashMap<u64, SemanticEntry>,
    threshold: u32,
    max_entries: usize,
}

struct SemanticEntry {
    response: Bytes,
    cost_usd: f64,
    model: String,
    created_at: Instant,
}

/// Result of a semantic cache lookup.
pub struct SemanticHit {
    pub response: Bytes,
    pub cost_usd: f64,
    pub model: String,
}

impl SemanticCache {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            threshold: DEFAULT_THRESHOLD,
            max_entries: DEFAULT_MAX_ENTRIES,
        }
    }

    /// Compute the semantic fingerprint of a request.
    pub fn fingerprint(body: &Value) -> u64 {
        let system = body
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

        // Last user message is the most semantically significant.
        let last_user = body
            .get("messages")
            .and_then(|m| m.as_array())
            .and_then(|arr| {
                arr.iter()
                    .rev()
                    .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
            })
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");

        let model = body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        simhash(&format!("{model}\n{system}\n{last_user}"))
    }

    /// Look up a semantically similar cached response.
    pub fn get(&self, fingerprint: u64) -> Option<SemanticHit> {
        // Scan all entries for Hamming distance within threshold.
        // For 5K entries this is ~50μs (u64 XOR + popcount per entry).
        for entry in self.entries.iter() {
            let distance = (fingerprint ^ *entry.key()).count_ones();
            if distance <= self.threshold {
                let e = entry.value();
                return Some(SemanticHit {
                    response: e.response.clone(),
                    cost_usd: e.cost_usd,
                    model: e.model.clone(),
                });
            }
        }
        None
    }

    /// Store a response in the semantic cache.
    pub fn put(&self, fingerprint: u64, response: Bytes, cost_usd: f64, model: String) {
        // Evict oldest if at capacity.
        if self.entries.len() >= self.max_entries {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|e| e.value().created_at)
                .map(|e| *e.key());
            if let Some(key) = oldest {
                self.entries.remove(&key);
            }
        }

        self.entries.insert(
            fingerprint,
            SemanticEntry {
                response,
                cost_usd,
                model,
                created_at: Instant::now(),
            },
        );
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similar_texts_have_low_hamming_distance() {
        let a = simhash("implement OAuth2 provider with Google authentication");
        let b = simhash("implement OAuth2 provider with GitHub authentication");
        let distance = (a ^ b).count_ones();
        assert!(distance <= 10, "distance was {distance}, expected ≤ 10");
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
}
