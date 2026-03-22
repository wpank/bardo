//! Request-level tier routing via HTTP headers.
//!
//! Callers set `X-Tier` (0/1/2) and optionally `X-Vitality` (0.0-1.0) to
//! control model selection at the gateway level. If no `X-Tier` header is
//! present, the request passes through unmodified.

use bardo_primitives::{InferenceTier as CognitiveTier, TierRouter};

/// Tier routing info extracted from request headers.
pub struct TierInfo {
    /// The cognitive tier (T0, T1, T2).
    pub tier: CognitiveTier,
    /// Vitality score used for T2 model degradation.
    pub vitality: f32,
    /// The model selected by the tier router, or `None` for T0 (suppressed).
    pub routed_model: Option<String>,
}

impl TierInfo {
    /// Parse tier routing from request headers.
    ///
    /// Reads `X-Tier` (required, u8 0/1/2) and `X-Vitality` (optional, f32,
    /// defaults to 1.0). Returns `None` if no `X-Tier` header is present,
    /// meaning the request should pass through without tier-based routing.
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Option<Self> {
        let tier_val: u8 = headers.get("x-tier")?.to_str().ok()?.parse().ok()?;
        let tier = CognitiveTier::try_from(tier_val).ok()?;
        let vitality: f32 = headers
            .get("x-vitality")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);
        let routed_model = TierRouter::select_model(tier, vitality).map(String::from);
        Some(Self {
            tier,
            vitality,
            routed_model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn t0_suppresses_inference() {
        let mut headers = HeaderMap::new();
        headers.insert("x-tier", HeaderValue::from_static("0"));
        let info = TierInfo::from_headers(&headers);
        assert!(info.is_some());
        let info = info.unwrap_or_else(|| panic!("expected Some"));
        assert_eq!(info.tier, CognitiveTier::T0);
        assert!(info.routed_model.is_none());
    }

    #[test]
    fn t1_routes_to_haiku() {
        let mut headers = HeaderMap::new();
        headers.insert("x-tier", HeaderValue::from_static("1"));
        let info = TierInfo::from_headers(&headers);
        assert!(info.is_some());
        let info = info.unwrap_or_else(|| panic!("expected Some"));
        assert_eq!(info.routed_model.as_deref(), Some("claude-haiku-4-5"));
    }

    #[test]
    fn t2_opus_with_high_vitality() {
        let mut headers = HeaderMap::new();
        headers.insert("x-tier", HeaderValue::from_static("2"));
        headers.insert("x-vitality", HeaderValue::from_static("0.8"));
        let info = TierInfo::from_headers(&headers);
        assert!(info.is_some());
        let info = info.unwrap_or_else(|| panic!("expected Some"));
        assert_eq!(info.routed_model.as_deref(), Some("claude-opus-4-6"));
    }

    #[test]
    fn t2_sonnet_with_low_vitality() {
        let mut headers = HeaderMap::new();
        headers.insert("x-tier", HeaderValue::from_static("2"));
        headers.insert("x-vitality", HeaderValue::from_static("0.1"));
        let info = TierInfo::from_headers(&headers);
        assert!(info.is_some());
        let info = info.unwrap_or_else(|| panic!("expected Some"));
        assert_eq!(info.routed_model.as_deref(), Some("claude-sonnet-4"));
    }

    #[test]
    fn missing_tier_header_returns_none() {
        let headers = HeaderMap::new();
        assert!(TierInfo::from_headers(&headers).is_none());
    }

    #[test]
    fn invalid_tier_value_returns_none() {
        let mut headers = HeaderMap::new();
        headers.insert("x-tier", HeaderValue::from_static("5"));
        assert!(TierInfo::from_headers(&headers).is_none());
    }

    #[test]
    fn default_vitality_is_1_0() {
        let mut headers = HeaderMap::new();
        headers.insert("x-tier", HeaderValue::from_static("2"));
        // No x-vitality header -> defaults to 1.0 -> should get opus
        let info = TierInfo::from_headers(&headers);
        assert!(info.is_some());
        let info = info.unwrap_or_else(|| panic!("expected Some"));
        assert!((info.vitality - 1.0).abs() < f32::EPSILON);
        assert_eq!(info.routed_model.as_deref(), Some("claude-opus-4-6"));
    }
}
