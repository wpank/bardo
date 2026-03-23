//! Security class router for privacy-aware provider selection.
//!
//! Classifies requests deterministically and enforces routing constraints:
//! Private requests must go through Venice (zero-retention); if Venice is
//! not configured, the request is rejected with 503.

use serde::{Deserialize, Serialize};

use super::error::VeniceError;
use super::security_class::{self, ClassificationResult, SecurityClass};

/// Configuration for the security class router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Dollar threshold for PortfolioComposition trigger.
    pub portfolio_usd_threshold: f64,
    /// Dollar threshold for MevSensitive trigger.
    pub mev_usd_threshold: f64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            portfolio_usd_threshold: 1_000.0,
            mev_usd_threshold: 500.0,
        }
    }
}

/// Routes requests based on security classification.
pub struct SecurityClassRouter {
    config: RouterConfig,
    venice_available: bool,
}

impl SecurityClassRouter {
    pub fn new(config: RouterConfig, venice_available: bool) -> Self {
        Self {
            config,
            venice_available,
        }
    }

    /// Classify a request body.
    pub fn classify(
        &self,
        body: &serde_json::Value,
        subsystem: Option<&str>,
    ) -> ClassificationResult {
        let text = security_class::extract_request_text(body);
        security_class::classify_request(
            &text,
            subsystem,
            self.config.portfolio_usd_threshold,
            self.config.mev_usd_threshold,
        )
    }

    /// Enforce routing rules. Returns Err if Private but no Venice.
    pub fn enforce_routing(&self, result: &ClassificationResult) -> Result<(), VeniceError> {
        if result.class == SecurityClass::Private && !self.venice_available {
            return Err(VeniceError::PrivateInferenceUnavailable);
        }
        Ok(())
    }

    /// Suggest a provider override based on classification.
    /// Returns `Some("venice")` if the request should be routed to Venice.
    pub fn suggest_provider(&self, result: &ClassificationResult) -> Option<&'static str> {
        match result.class {
            SecurityClass::Private if self.venice_available => Some("venice"),
            SecurityClass::Confidential if self.venice_available => Some("venice"),
            _ => None,
        }
    }

    pub fn venice_available(&self) -> bool {
        self.venice_available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforce_private_without_venice() {
        let router = SecurityClassRouter::new(RouterConfig::default(), false);
        let result = ClassificationResult {
            class: SecurityClass::Private,
            reason: "test".into(),
            triggers: vec![],
        };
        assert!(router.enforce_routing(&result).is_err());
    }

    #[test]
    fn test_enforce_private_with_venice() {
        let router = SecurityClassRouter::new(RouterConfig::default(), true);
        let result = ClassificationResult {
            class: SecurityClass::Private,
            reason: "test".into(),
            triggers: vec![],
        };
        assert!(router.enforce_routing(&result).is_ok());
    }

    #[test]
    fn test_suggest_provider_private() {
        let router = SecurityClassRouter::new(RouterConfig::default(), true);
        let result = ClassificationResult {
            class: SecurityClass::Private,
            reason: "test".into(),
            triggers: vec![],
        };
        assert_eq!(router.suggest_provider(&result), Some("venice"));
    }

    #[test]
    fn test_suggest_provider_standard() {
        let router = SecurityClassRouter::new(RouterConfig::default(), true);
        let result = ClassificationResult::standard();
        assert_eq!(router.suggest_provider(&result), None);
    }

    #[test]
    fn test_classify_body() {
        let router = SecurityClassRouter::new(RouterConfig::default(), true);
        let body = serde_json::json!({
            "messages": [
                {"role": "user", "content": "set the reserve price at $5000"}
            ]
        });
        let result = router.classify(&body, None);
        assert_eq!(result.class, SecurityClass::Private);
    }
}
