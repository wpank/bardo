//! Core entry types for the Grimoire knowledge system.
//!
//! Defines `GrimoireEntry` (the canonical knowledge record), `Episode` (raw episodic
//! observations for the vector store), and all supporting enums and metadata structs.

use golem_core::cortical::{PadVector, PlutchikEmotion};
use golem_core::id::GolemId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Entry Type Discriminators ───────────────────────────────────────────

/// Six knowledge entry categories in the Grimoire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    /// Declarative: "what is true."
    Insight,
    /// Prescriptive: "what to do."
    Heuristic,
    /// Immune system: "what to avoid."
    Warning,
    /// Structural: "what causes what."
    CausalLink,
    /// Speculative: "what might work."
    StrategyFragment,
    /// Permanent negative knowledge; confidence floor 0.3, decay 0.5x.
    AntiKnowledge,
}

impl EntryType {
    /// String representation for SQLite storage.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Insight => "insight",
            Self::Heuristic => "heuristic",
            Self::Warning => "warning",
            Self::CausalLink => "causal_link",
            Self::StrategyFragment => "strategy_fragment",
            Self::AntiKnowledge => "anti_knowledge",
        }
    }

    /// Parse from SQLite string.
    pub fn from_str_repr(s: &str) -> Option<Self> {
        match s {
            "insight" => Some(Self::Insight),
            "heuristic" => Some(Self::Heuristic),
            "warning" => Some(Self::Warning),
            "causal_link" => Some(Self::CausalLink),
            "strategy_fragment" => Some(Self::StrategyFragment),
            "anti_knowledge" => Some(Self::AntiKnowledge),
            _ => None,
        }
    }

    /// Confidence floor for this entry type.
    /// `AntiKnowledge` has a higher floor (0.30) to preserve negative knowledge.
    pub fn confidence_floor(self) -> f64 {
        match self {
            Self::AntiKnowledge => 0.30,
            _ => 0.05,
        }
    }
}

/// Governs temporal decay rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecayClass {
    /// ~48h half-life. Raw episodes, fast-moving observations.
    Ephemeral,
    /// ~7-day half-life. Operational conditions.
    Tactical,
    /// ~14–21 day half-life. Market regime patterns.
    RegimeConditional,
    /// No decay. Protocol mechanics, ABIs, mathematical constants.
    Structural,
    /// Very slow, Curator-managed. PLAYBOOK.md entries only.
    Procedural,
}

impl DecayClass {
    /// String representation for SQLite storage.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ephemeral => "ephemeral",
            Self::Tactical => "tactical",
            Self::RegimeConditional => "regime_conditional",
            Self::Structural => "structural",
            Self::Procedural => "procedural",
        }
    }

    /// Parse from SQLite string.
    pub fn from_str_repr(s: &str) -> Option<Self> {
        match s {
            "ephemeral" => Some(Self::Ephemeral),
            "tactical" => Some(Self::Tactical),
            "regime_conditional" => Some(Self::RegimeConditional),
            "structural" => Some(Self::Structural),
            "procedural" => Some(Self::Procedural),
            _ => None,
        }
    }
}

/// Polarity: K+ (positive, "do X") or K- (negative, "avoid X").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgePolarity {
    /// Positive knowledge: "do X."
    Positive,
    /// Negative knowledge: "avoid X."
    Negative,
}

impl Default for KnowledgePolarity {
    fn default() -> Self {
        Self::Positive
    }
}

impl KnowledgePolarity {
    /// String representation for SQLite.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }

    /// Parse from SQLite string.
    pub fn from_str_repr(s: &str) -> Option<Self> {
        match s {
            "positive" => Some(Self::Positive),
            "negative" => Some(Self::Negative),
            _ => None,
        }
    }
}

/// How this entry was created. Affects initial confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// Learned directly from experience.
    SelfLearned,
    /// Received from a Clade peer.
    Clade,
    /// Inherited from a predecessor Golem.
    Predecessor,
    /// Retrieved from the Styx archive.
    StyxQuery,
    /// Retrieved from the Lethe knowledge pool.
    Lethe,
    /// Acquired from the knowledge marketplace.
    Marketplace,
    /// Copied from a replicant.
    Replicant,
    /// Generated during the death protocol.
    DeathReflection,
    /// Generated during a dream cycle.
    Dream,
}

impl Provenance {
    /// String representation for SQLite.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfLearned => "self_learned",
            Self::Clade => "clade",
            Self::Predecessor => "predecessor",
            Self::StyxQuery => "styx_query",
            Self::Lethe => "lethe",
            Self::Marketplace => "marketplace",
            Self::Replicant => "replicant",
            Self::DeathReflection => "death_reflection",
            Self::Dream => "dream",
        }
    }

    /// Parse from SQLite string.
    pub fn from_str_repr(s: &str) -> Option<Self> {
        match s {
            "self_learned" => Some(Self::SelfLearned),
            "clade" => Some(Self::Clade),
            "predecessor" => Some(Self::Predecessor),
            "styx_query" => Some(Self::StyxQuery),
            "lethe" => Some(Self::Lethe),
            "marketplace" => Some(Self::Marketplace),
            "replicant" => Some(Self::Replicant),
            "death_reflection" => Some(Self::DeathReflection),
            "dream" => Some(Self::Dream),
            _ => None,
        }
    }
}

/// Lifecycle status of an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryStatus {
    /// In the active Grimoire, subject to retrieval.
    Active,
    /// Removed from active context, in cold storage.
    Archived,
    /// Held for review by the Curator.
    Quarantined,
}

impl Default for EntryStatus {
    fn default() -> Self {
        Self::Active
    }
}

// ── Metadata Structs ────────────────────────────────────────────────────

/// Source origin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySource {
    /// Golem that produced this entry.
    pub golem_id: String,
    /// Generation number (0 = directly learned).
    pub generation_number: Option<u32>,
    /// Owner wallet address.
    pub owner_address: Option<String>,
}

/// Emotional context at entry creation, for mood-congruent retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTag {
    /// Primary Plutchik emotion label.
    pub primary: PlutchikEmotion,
    /// Pleasure-Arousal-Dominance vector.
    pub pad: PadVector,
    /// Arousal intensity, 0.0–1.0.
    pub arousal: f64,
}

/// Memetic lifecycle tracking for a single Grimoire entry.
///
/// Tracks replication fitness using cultural-evolution primitives.
/// Stored as columns in `grimoire_entries` and mirrored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeticFields {
    /// How faithfully the entry has been transmitted (0.0–1.0).
    pub fidelity: f64,
    /// How often this entry is retrieved and acted upon (copies/tick).
    pub fecundity: f64,
    /// Effective fitness W(E) = fidelity * fecundity * longevity.
    pub fitness: f64,
    /// Normalized sum of squared prediction errors. High = possible parasite.
    pub parasite_score: f64,
    /// How many Curator cycles this entry has survived.
    pub generation: u32,
    /// Consecutive Curator cycles with effective_confidence < threshold.
    pub consecutive_low_confidence: u32,
}

impl Default for MemeticFields {
    fn default() -> Self {
        Self {
            fidelity: 1.0,
            fecundity: 0.0,
            fitness: 0.0,
            parasite_score: 0.0,
            generation: 0,
            consecutive_low_confidence: 0,
        }
    }
}

// ── Core Entry ──────────────────────────────────────────────────────────

/// The canonical Grimoire entry, covering all six knowledge categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrimoireEntry {
    /// UUIDv7 (time-ordered) for chronological sorting.
    pub id: Uuid,
    /// Golem that owns this entry.
    pub golem_id: GolemId,
    /// Discriminates entry category. Immutable after creation.
    pub category: EntryType,
    /// Human-readable content in natural language.
    pub content: String,
    /// 768-dim embedding (nomic-embed-text-v1.5, Matryoshka).
    pub embedding: Vec<f32>,
    /// Evidential quality, 0.0–1.0. Floor depends on category.
    pub confidence: f64,
    /// Composite quality (specificity, actionability, novelty, verifiability, consistency).
    pub quality_score: f64,
    /// Governs temporal decay rate. Immutable after creation.
    pub decay_class: DecayClass,
    /// Bi-temporal: when the observation was made (Unix ms).
    pub valid_from: i64,
    /// Bi-temporal: when the observation ceased being true (0 = still active).
    pub valid_until: i64,
    /// Episode IDs that generated or validated this entry (provenance chain).
    pub parent_episode_ids: Vec<Uuid>,
    /// Directed causal edges to parent entries (DAG).
    pub causal_parents: Vec<Uuid>,
    /// Semantic tags for retrieval filtering.
    pub tags: Vec<String>,
    /// How this entry was created.
    pub provenance: Provenance,
    /// Source origin metadata.
    pub source: EntrySource,
    /// Emotional context at creation time.
    pub emotional_tag: Option<EmotionalTag>,
    /// Last retrieval timestamp (Unix ms).
    pub last_accessed_at: i64,
    /// Successful retrieval count. Monotonically non-decreasing.
    pub strength: u32,
    /// How many times this entry was validated by subsequent evidence.
    pub validated_count: u32,
    /// How many times subsequent evidence contradicted this entry.
    pub contradicted_count: u32,
    /// Memetic fitness tracking.
    pub memetic: MemeticFields,
    /// True for death-sourced entries. Grants 1.2x retrieval boost + 3x decay slowdown.
    pub is_bloodstain: bool,
    /// Polarity: K+ (positive) or K- (negative).
    pub polarity: KnowledgePolarity,
}

impl GrimoireEntry {
    /// Returns the confidence floor for this entry's category.
    pub fn confidence_floor(&self) -> f64 {
        self.category.confidence_floor()
    }

    /// Clamps confidence to the valid range, respecting the category floor.
    pub fn clamp_confidence(&mut self) {
        let floor = self.confidence_floor();
        self.confidence = self.confidence.clamp(floor, 1.0);
    }

    /// Creates a test heuristic entry with sensible defaults.
    #[cfg(test)]
    pub fn test_heuristic(content: &str) -> Self {
        Self {
            id: Uuid::now_v7(),
            golem_id: GolemId::new(),
            category: EntryType::Heuristic,
            content: content.to_string(),
            embedding: vec![0.0; 768],
            confidence: 0.6,
            quality_score: 0.5,
            decay_class: DecayClass::Tactical,
            valid_from: 0,
            valid_until: 0,
            parent_episode_ids: vec![],
            causal_parents: vec![],
            tags: vec![],
            provenance: Provenance::SelfLearned,
            source: EntrySource {
                golem_id: String::new(),
                generation_number: None,
                owner_address: None,
            },
            emotional_tag: None,
            last_accessed_at: 0,
            strength: 1,
            validated_count: 0,
            contradicted_count: 0,
            memetic: MemeticFields::default(),
            is_bloodstain: false,
            polarity: KnowledgePolarity::Positive,
        }
    }

    /// Creates a test entry with a specific category and confidence.
    #[cfg(test)]
    pub fn test_with_category(category: EntryType, confidence: f64) -> Self {
        let mut entry = Self::test_heuristic("test entry");
        entry.id = Uuid::now_v7();
        entry.category = category;
        entry.confidence = confidence;
        entry.decay_class = match category {
            EntryType::Insight => DecayClass::Tactical,
            EntryType::Heuristic => DecayClass::RegimeConditional,
            EntryType::Warning => DecayClass::Tactical,
            EntryType::CausalLink => DecayClass::RegimeConditional,
            EntryType::StrategyFragment => DecayClass::Ephemeral,
            EntryType::AntiKnowledge => DecayClass::Tactical,
        };
        entry.clamp_confidence();
        entry
    }
}

// ── Episode (LanceDB) ──────────────────────────────────────────────────

/// Raw episodic record written to the vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// UUIDv7 (time-ordered).
    pub id: Uuid,
    /// Golem that recorded this episode.
    pub golem_id: GolemId,
    /// Natural language description of the episode.
    pub text: String,
    /// 768-dim embedding vector.
    pub vector: Vec<f32>,
    /// Tool that generated this episode (e.g. "swap", "observe").
    pub tool: String,
    /// Outcome: "positive", "negative", or "neutral".
    pub outcome: String,
    /// Tick at which this episode was recorded.
    pub tick_id: u64,
    /// Importance score (0.0–1.0).
    pub importance: f32,
    /// Emotional arousal at recording time.
    pub emotional_arousal: f32,
    /// PAD pleasure component.
    pub pad_pleasure: f32,
    /// PAD arousal component.
    pub pad_arousal: f32,
    /// PAD dominance component.
    pub pad_dominance: f32,
    /// Recording timestamp (Unix ms).
    pub timestamp_ms: i64,
    /// True for death-sourced episodes.
    pub is_bloodstain: bool,
    /// True after consolidation by the Curator.
    pub consolidated: bool,
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // INV-009: AntiKnowledge confidence floor
    #[test]
    fn test_anti_knowledge_confidence_floor() {
        let mut entry = GrimoireEntry::test_with_category(EntryType::AntiKnowledge, 0.1);
        // The floor for AntiKnowledge is 0.30, so clamping should raise it.
        entry.clamp_confidence();
        assert!(
            entry.confidence >= 0.30,
            "AntiKnowledge confidence {} should be >= 0.30",
            entry.confidence
        );

        // Standard entry with low confidence should clamp to 0.05.
        let mut standard = GrimoireEntry::test_with_category(EntryType::Insight, 0.02);
        standard.clamp_confidence();
        assert!(
            standard.confidence >= 0.05,
            "Standard confidence {} should be >= 0.05",
            standard.confidence
        );
    }

    // INV-020: Confidence range [0.0, 1.0]
    #[test]
    fn test_confidence_clamp_bounds() {
        for &raw in &[-0.5, 0.0, 0.05, 0.5, 1.0, 1.5] {
            let mut entry = GrimoireEntry::test_heuristic("bounds test");
            entry.confidence = raw;
            entry.clamp_confidence();
            let floor = entry.confidence_floor();
            assert!(
                entry.confidence >= floor && entry.confidence <= 1.0,
                "confidence {} out of [{}, 1.0] for raw {}",
                entry.confidence,
                floor,
                raw
            );
        }
    }

    // INV-021: Strength is monotonically non-decreasing
    #[test]
    fn test_strength_monotonic_increase() {
        let mut entry = GrimoireEntry::test_heuristic("strength test");
        let initial = entry.strength;
        // Simulate successful retrievals.
        entry.strength += 1;
        assert!(entry.strength > initial);
        entry.strength += 1;
        assert!(entry.strength > initial + 1);
        // Strength never decrements.
        assert!(entry.strength >= initial);
    }

    // INV-024: Entry type is immutable after creation (architectural invariant).
    #[test]
    fn test_entry_type_immutable() {
        let entry = GrimoireEntry::test_with_category(EntryType::Warning, 0.7);
        // Verify all six categories are valid and distinct.
        let categories = [
            EntryType::Insight,
            EntryType::Heuristic,
            EntryType::Warning,
            EntryType::CausalLink,
            EntryType::StrategyFragment,
            EntryType::AntiKnowledge,
        ];
        for (i, a) in categories.iter().enumerate() {
            for (j, b) in categories.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
        // The entry's category should remain what we set it to.
        assert_eq!(entry.category, EntryType::Warning);
    }

    // INV-023: Decay class is immutable after creation (architectural invariant).
    #[test]
    fn test_decay_class_immutable() {
        let entry = GrimoireEntry::test_heuristic("decay class test");
        let original_class = entry.decay_class;
        // Decay class should remain Tactical (the default for test_heuristic).
        assert_eq!(original_class, DecayClass::Tactical);
        // All five classes are valid and distinct.
        let classes = [
            DecayClass::Ephemeral,
            DecayClass::Tactical,
            DecayClass::RegimeConditional,
            DecayClass::Structural,
            DecayClass::Procedural,
        ];
        for (i, a) in classes.iter().enumerate() {
            for (j, b) in classes.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    // INV-026: UUIDv7 temporal ordering
    #[test]
    fn test_uuidv7_temporal_ordering() {
        let id1 = Uuid::now_v7();
        // Small delay to ensure different timestamps.
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = Uuid::now_v7();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id3 = Uuid::now_v7();

        // UUIDv7 IDs should be monotonically increasing.
        assert!(id1 < id2, "id1 ({id1}) should be < id2 ({id2})");
        assert!(id2 < id3, "id2 ({id2}) should be < id3 ({id3})");
    }

    // INV-022: EmotionalTag structure
    #[test]
    fn test_emotional_tag_structure() {
        let tag = EmotionalTag {
            primary: PlutchikEmotion::Joy,
            pad: PadVector {
                pleasure: 0.8,
                arousal: 0.6,
                dominance: 0.7,
            },
            arousal: 0.6,
        };
        assert_eq!(tag.primary, PlutchikEmotion::Joy);
        assert!((tag.pad.pleasure - 0.8).abs() < f64::EPSILON);
        assert!((tag.arousal - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_entry_type_roundtrip() {
        let types = [
            EntryType::Insight,
            EntryType::Heuristic,
            EntryType::Warning,
            EntryType::CausalLink,
            EntryType::StrategyFragment,
            EntryType::AntiKnowledge,
        ];
        for t in &types {
            let s = t.as_str();
            let parsed = EntryType::from_str_repr(s);
            assert_eq!(parsed, Some(*t), "roundtrip failed for {s}");
        }
    }

    #[test]
    fn test_decay_class_roundtrip() {
        let classes = [
            DecayClass::Ephemeral,
            DecayClass::Tactical,
            DecayClass::RegimeConditional,
            DecayClass::Structural,
            DecayClass::Procedural,
        ];
        for c in &classes {
            let s = c.as_str();
            let parsed = DecayClass::from_str_repr(s);
            assert_eq!(parsed, Some(*c), "roundtrip failed for {s}");
        }
    }

    #[test]
    fn test_grimoire_entry_serializes_roundtrip() {
        let entry = GrimoireEntry::test_heuristic("gas below 10 gwei at 2-4 AM UTC");
        let json = serde_json::to_string(&entry).expect("serialize");
        let deser: GrimoireEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(entry.id, deser.id);
        assert_eq!(entry.content, deser.content);
        assert_eq!(entry.category, deser.category);
    }
}
