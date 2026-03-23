//! Venice provider configuration types.

use serde::{Deserialize, Serialize};

use super::diem::DiemConfig;

/// Venice API web search mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchMode {
    Off,
    Auto,
    Always,
}

impl Default for WebSearchMode {
    fn default() -> Self {
        Self::Off
    }
}

/// Venice-specific inference parameters injected into the request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceParameters {
    /// When true, strips <think>...</think> tags from response content.
    /// Set false for dream cycles, death reflection, and complex daimon appraisal.
    pub strip_thinking_response: bool,
    /// Venice web search mode for grounding.
    pub enable_web_search: WebSearchMode,
}

impl Default for VeniceParameters {
    fn default() -> Self {
        Self {
            strip_thinking_response: false,
            enable_web_search: WebSearchMode::Off,
        }
    }
}

/// Model tier mapping for Venice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceModelMapping {
    /// Fast, cheap private analysis.
    pub t1: String,
    /// Deep reasoning with visible <think> tags.
    pub t2_reasoning: String,
    /// Frontier general-purpose.
    pub t2_general: String,
    /// Vision model.
    pub vision: String,
}

impl Default for VeniceModelMapping {
    fn default() -> Self {
        Self {
            t1: "llama-3.3-70b".to_string(),
            t2_reasoning: "deepseek-ai-DeepSeek-R1".to_string(),
            t2_general: "zai-org-glm-4.7".to_string(),
            vision: "qwen-2.5-vl-72b".to_string(),
        }
    }
}

/// Top-level Venice provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeniceProviderConfig {
    /// Venice API base URL.
    pub base_url: String,
    /// Venice API key (env: VENICE_API_KEY).
    pub api_key: String,
    /// Model tier mapping.
    pub models: VeniceModelMapping,
    /// Venice-specific inference parameters.
    pub venice_parameters: VeniceParameters,
    /// Daily spending cap in USD.
    pub daily_cap_usd: f64,
    /// Optional DIEM staking config.
    pub diem: Option<DiemConfig>,
}

impl Default for VeniceProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.venice.ai/api/v1".to_string(),
            api_key: String::new(),
            models: VeniceModelMapping::default(),
            venice_parameters: VeniceParameters::default(),
            daily_cap_usd: 50.0,
            diem: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model_mapping() {
        let m = VeniceModelMapping::default();
        assert_eq!(m.t1, "llama-3.3-70b");
        assert_eq!(m.t2_reasoning, "deepseek-ai-DeepSeek-R1");
        assert_eq!(m.t2_general, "zai-org-glm-4.7");
        assert_eq!(m.vision, "qwen-2.5-vl-72b");
    }
}
