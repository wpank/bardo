//! Three-tier inference security classification.
//!
//! Deterministic keyword/pattern matching over request message text.
//! No LLM calls. No randomness.

use serde::{Deserialize, Serialize};

/// Three-tier security classification for inference requests.
/// Total order: Standard < Confidential < Private.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityClass {
    /// Normal inference. Any provider. Provider may retain for training.
    Standard,
    /// Venice preferred. Billing/audit retention only. No training data.
    Confidential,
    /// Venice mandatory. Zero retention. Gateway returns 503 rather than
    /// falling back to a retaining provider.
    Private,
}

impl Default for SecurityClass {
    fn default() -> Self {
        Self::Standard
    }
}

impl SecurityClass {
    pub fn requires_venice(&self) -> bool {
        matches!(self, SecurityClass::Private)
    }

    pub fn prefers_venice(&self) -> bool {
        matches!(self, SecurityClass::Confidential)
    }
}

/// What triggered a non-Standard classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityTrigger {
    /// Request contains specific asset amounts above $1,000.
    PortfolioComposition,
    /// Request discusses execution timing for pending swaps/rebalances.
    RebalanceTiming,
    /// Inter-agent commercial discussion (reserve prices, walk-away conditions).
    DealNegotiation,
    /// Governance proposal analysis with position exposure data.
    GovernanceDeliberation,
    /// Execution timing that could be front-run (pending actions > $500).
    MevSensitive,
    /// Evaluating another agent's behavioral patterns or weaknesses.
    CounterpartyAnalysis,
    /// Terminal phase reasoning.
    DeathReflection,
    /// Owner-identifying information present in context.
    OwnerPii,
}

/// Maps a trigger to its security class.
pub fn trigger_class(t: SecurityTrigger) -> SecurityClass {
    match t {
        SecurityTrigger::PortfolioComposition
        | SecurityTrigger::RebalanceTiming
        | SecurityTrigger::OwnerPii => SecurityClass::Confidential,

        SecurityTrigger::DealNegotiation
        | SecurityTrigger::GovernanceDeliberation
        | SecurityTrigger::MevSensitive
        | SecurityTrigger::CounterpartyAnalysis
        | SecurityTrigger::DeathReflection => SecurityClass::Private,
    }
}

/// Result of classifying a request.
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// Highest security class detected (max over all triggered classes).
    pub class: SecurityClass,
    /// Human-readable explanation of the highest-severity trigger.
    pub reason: String,
    /// All triggers that fired during classification.
    pub triggers: Vec<SecurityTrigger>,
}

impl ClassificationResult {
    pub fn standard() -> Self {
        Self {
            class: SecurityClass::Standard,
            reason: String::new(),
            triggers: vec![],
        }
    }
}

/// Classify request text by scanning for sensitive patterns.
///
/// Deterministic: same input always produces same output.
/// Collects all matching triggers, then takes the max class.
pub fn classify_request(
    text: &str,
    subsystem: Option<&str>,
    portfolio_threshold: f64,
    mev_threshold: f64,
) -> ClassificationResult {
    let mut triggers = Vec::new();

    // Death reflection: subsystem tag or keywords in text.
    if subsystem == Some("death") {
        triggers.push(SecurityTrigger::DeathReflection);
    }
    let death_keywords = [
        "death reflection",
        "terminal reasoning",
        "end of life",
        "dying",
    ];
    if death_keywords.iter().any(|kw| text_contains_ci(text, kw)) {
        triggers.push(SecurityTrigger::DeathReflection);
    }

    // Deal negotiation keywords.
    let deal_keywords = [
        "reserve price",
        "walk away",
        "walk-away",
        "counter offer",
        "counteroffer",
        "negotiat",
        "bilateral allocation",
    ];
    if deal_keywords.iter().any(|kw| text_contains_ci(text, kw)) {
        triggers.push(SecurityTrigger::DealNegotiation);
    }

    // Governance deliberation: governance keywords + position signals.
    let gov_keywords = ["governance proposal", "on-chain governance", "quorum"];
    let position_signals = ["position", "exposure", "holding", "stake"];
    let has_gov = gov_keywords.iter().any(|kw| text_contains_ci(text, kw));
    let has_vote = text_contains_ci(text, "vote") || text_contains_ci(text, "voting");
    let has_position = position_signals.iter().any(|kw| text_contains_ci(text, kw));
    if has_gov || (has_vote && has_position) {
        triggers.push(SecurityTrigger::GovernanceDeliberation);
    }

    // Counterparty analysis.
    let counter_keywords = [
        "counterparty behavior",
        "agent weakness",
        "behavioral pattern",
    ];
    if counter_keywords.iter().any(|kw| text_contains_ci(text, kw)) {
        triggers.push(SecurityTrigger::CounterpartyAnalysis);
    }

    // MEV sensitive: execution amount > threshold + timing keywords.
    let timing_keywords = ["block", "timestamp", "deadline", "execute at", "at price"];
    let has_timing = timing_keywords.iter().any(|kw| text_contains_ci(text, kw));
    if has_timing {
        if let Some(amount) = extract_max_dollar_amount(text) {
            if amount >= mev_threshold {
                triggers.push(SecurityTrigger::MevSensitive);
            }
        }
    }

    // Portfolio composition: dollar amounts > threshold.
    if let Some(amount) = extract_max_dollar_amount(text) {
        if amount >= portfolio_threshold {
            // Only add if not already triggered by MEV (avoid double-counting).
            if !triggers.contains(&SecurityTrigger::MevSensitive) {
                triggers.push(SecurityTrigger::PortfolioComposition);
            }
        }
    }

    // Rebalance timing.
    let rebalance_keywords = [
        "rebalance",
        "swap timing",
        "execution timing",
        "pending swap",
        "when to execute",
    ];
    if rebalance_keywords
        .iter()
        .any(|kw| text_contains_ci(text, kw))
    {
        triggers.push(SecurityTrigger::RebalanceTiming);
    }

    // Owner PII: basic patterns.
    let pii_keywords = ["social security", "passport number", "date of birth"];
    if pii_keywords.iter().any(|kw| text_contains_ci(text, kw)) {
        triggers.push(SecurityTrigger::OwnerPii);
    }

    // Deduplicate triggers by checking membership before push was done above,
    // but some paths may push the same trigger twice (e.g., DeathReflection from
    // both subsystem tag and keyword match). Remove duplicates.
    {
        let mut seen = std::collections::HashSet::new();
        triggers.retain(|t| seen.insert(*t));
    }

    // Compute class as max over all triggered classes.
    let class = triggers
        .iter()
        .map(|t| trigger_class(*t))
        .max()
        .unwrap_or(SecurityClass::Standard);

    let reason = if let Some(t) = triggers.iter().max_by_key(|t| trigger_class(**t)) {
        format!("{t:?}")
    } else {
        String::new()
    };

    ClassificationResult {
        class,
        reason,
        triggers,
    }
}

/// Extract all message text from a raw JSON request body.
pub fn extract_request_text(body: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(sys) = body.get("system").and_then(|v| v.as_str()) {
        text.push_str(sys);
        text.push('\n');
    }
    if let Some(messages) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            extract_message_text(msg, &mut text);
        }
    }
    text
}

fn extract_message_text(msg: &serde_json::Value, out: &mut String) {
    match msg.get("content") {
        Some(serde_json::Value::String(s)) => {
            out.push_str(s);
            out.push('\n');
        }
        Some(serde_json::Value::Array(blocks)) => {
            for block in blocks {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                        out.push_str(t);
                        out.push('\n');
                    }
                }
            }
        }
        _ => {}
    }
}

/// Case-insensitive substring check.
fn text_contains_ci(text: &str, pattern: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(&pattern.to_ascii_lowercase())
}

/// Extract the maximum dollar amount from text.
/// Matches patterns like $1,000, $500.00, $2500.
fn extract_max_dollar_amount(text: &str) -> Option<f64> {
    let mut max_amount: Option<f64> = None;
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'$' {
            i += 1;
            let start = i;
            // Consume digits, commas, and dots.
            while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b',' || bytes[i] == b'.') {
                i += 1;
            }
            if i > start {
                let num_str: String = text[start..i].chars().filter(|c| *c != ',').collect();
                if let Ok(val) = num_str.parse::<f64>() {
                    max_amount = Some(max_amount.map_or(val, |prev: f64| prev.max(val)));
                }
            }
        } else {
            i += 1;
        }
    }

    max_amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_class_total_order() {
        assert!(SecurityClass::Standard < SecurityClass::Confidential);
        assert!(SecurityClass::Confidential < SecurityClass::Private);
        assert!(SecurityClass::Standard < SecurityClass::Private);
    }

    #[test]
    fn test_trigger_class_mapping() {
        assert_eq!(
            trigger_class(SecurityTrigger::PortfolioComposition),
            SecurityClass::Confidential
        );
        assert_eq!(
            trigger_class(SecurityTrigger::RebalanceTiming),
            SecurityClass::Confidential
        );
        assert_eq!(
            trigger_class(SecurityTrigger::OwnerPii),
            SecurityClass::Confidential
        );
        assert_eq!(
            trigger_class(SecurityTrigger::DealNegotiation),
            SecurityClass::Private
        );
        assert_eq!(
            trigger_class(SecurityTrigger::GovernanceDeliberation),
            SecurityClass::Private
        );
        assert_eq!(
            trigger_class(SecurityTrigger::MevSensitive),
            SecurityClass::Private
        );
        assert_eq!(
            trigger_class(SecurityTrigger::CounterpartyAnalysis),
            SecurityClass::Private
        );
        assert_eq!(
            trigger_class(SecurityTrigger::DeathReflection),
            SecurityClass::Private
        );
    }

    #[test]
    fn test_classification_standard_for_benign() {
        let result = classify_request("Hello, how are you?", None, 1000.0, 500.0);
        assert_eq!(result.class, SecurityClass::Standard);
        assert!(result.triggers.is_empty());
    }

    #[test]
    fn test_classification_private_for_deal_negotiation() {
        let result = classify_request(
            "Set the reserve price at $5000 and walk away if below $3000",
            None,
            1000.0,
            500.0,
        );
        assert_eq!(result.class, SecurityClass::Private);
        assert!(result.triggers.contains(&SecurityTrigger::DealNegotiation));
    }

    #[test]
    fn test_classification_private_for_death_subsystem() {
        let result = classify_request("reflecting on existence", Some("death"), 1000.0, 500.0);
        assert_eq!(result.class, SecurityClass::Private);
        assert!(result.triggers.contains(&SecurityTrigger::DeathReflection));
    }

    #[test]
    fn test_classification_confidential_for_portfolio() {
        let result = classify_request(
            "My portfolio has $2,500 in ETH and $1,200 in USDC",
            None,
            1000.0,
            500.0,
        );
        assert_eq!(result.class, SecurityClass::Confidential);
        assert!(
            result
                .triggers
                .contains(&SecurityTrigger::PortfolioComposition)
        );
    }

    #[test]
    fn test_portfolio_threshold_boundary() {
        let below = classify_request("I have $999.99 in tokens", None, 1000.0, 500.0);
        assert_eq!(below.class, SecurityClass::Standard);

        let at = classify_request("I have $1000.00 in tokens", None, 1000.0, 500.0);
        assert_eq!(at.class, SecurityClass::Confidential);

        let above = classify_request("I have $1000.01 in tokens", None, 1000.0, 500.0);
        assert_eq!(above.class, SecurityClass::Confidential);
    }

    #[test]
    fn test_mev_threshold_boundary() {
        // MEV needs both amount >= 500 AND timing keyword.
        let below = classify_request("Execute at block 100 with $499.99", None, 1000.0, 500.0);
        assert!(!below.triggers.contains(&SecurityTrigger::MevSensitive));

        let at = classify_request("Execute at block 100 with $500.00", None, 1000.0, 500.0);
        assert!(at.triggers.contains(&SecurityTrigger::MevSensitive));
        assert_eq!(at.class, SecurityClass::Private);
    }

    #[test]
    fn test_classification_max_rule() {
        // Both confidential (portfolio) and private (deal negotiation) triggers.
        let result = classify_request(
            "My $2,500 portfolio should negotiate a reserve price",
            None,
            1000.0,
            500.0,
        );
        assert_eq!(result.class, SecurityClass::Private);
    }

    #[test]
    fn test_extract_dollar_amounts() {
        assert_eq!(extract_max_dollar_amount("costs $500"), Some(500.0));
        assert_eq!(extract_max_dollar_amount("$1,000 and $2,500"), Some(2500.0));
        assert_eq!(extract_max_dollar_amount("no money here"), None);
        assert_eq!(extract_max_dollar_amount("$0"), Some(0.0));
    }

    #[test]
    fn test_extract_request_text() {
        let body = serde_json::json!({
            "system": "You are helpful",
            "messages": [
                {"role": "user", "content": "Hello world"},
                {"role": "assistant", "content": [
                    {"type": "text", "text": "Hi there"}
                ]}
            ]
        });
        let text = extract_request_text(&body);
        assert!(text.contains("You are helpful"));
        assert!(text.contains("Hello world"));
        assert!(text.contains("Hi there"));
    }
}
