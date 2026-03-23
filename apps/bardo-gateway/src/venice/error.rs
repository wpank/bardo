//! Venice-specific error types.

use super::diem::DiemCategory;

#[derive(Debug, thiserror::Error)]
pub enum VeniceError {
    #[error("venice api error {status}: {body}")]
    Api { status: u16, body: String },

    #[error("venice request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("daily cap {cap_usd} usd exceeded; consumed {consumed_usd} usd today")]
    DailyCapExceeded { cap_usd: f64, consumed_usd: f64 },

    #[error("diem budget exhausted for category {category:?}")]
    DiemExhausted { category: DiemCategory },

    #[error("private inference required but venice not configured")]
    PrivateInferenceUnavailable,
}
