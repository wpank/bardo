//! Query types for context requests.

/// How to search the index.
#[derive(Debug, Clone, Default)]
pub enum SearchStrategy {
    /// Keyword search only (fast, name-based).
    #[default]
    Keyword,
    /// HDC fingerprint similarity (structural).
    Similar {
        /// Minimum similarity threshold (0.0–1.0).
        threshold: f32,
    },
    /// Keyword + HDC fused via Reciprocal Rank Fusion.
    Hybrid {
        /// Minimum similarity threshold for the HDC leg.
        threshold: f32,
    },
}

/// A request for context from the index.
#[derive(Debug, Clone)]
pub struct ContextQuery {
    /// The search text (symbol name, natural language, etc.).
    pub query: String,
    /// Which search strategy to use.
    pub strategy: SearchStrategy,
    /// Max symbols to include in the response.
    pub limit: usize,
    /// How many lines of source context to include around each symbol.
    pub context_lines: usize,
    /// Whether to include related symbols (types from signatures).
    pub include_related: bool,
}

impl Default for ContextQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            strategy: SearchStrategy::default(),
            limit: 10,
            context_lines: 10,
            include_related: true,
        }
    }
}

impl ContextQuery {
    /// Create a keyword-only query.
    pub fn keyword(query: impl Into<String>, limit: usize) -> Self {
        Self {
            query: query.into(),
            limit,
            ..Default::default()
        }
    }

    /// Create a hybrid (keyword + HDC) query.
    pub fn hybrid(query: impl Into<String>, limit: usize, threshold: f32) -> Self {
        Self {
            query: query.into(),
            strategy: SearchStrategy::Hybrid { threshold },
            limit,
            ..Default::default()
        }
    }
}
