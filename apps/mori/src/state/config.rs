use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::agent::{AgentBackend, AgentRole, ModelSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigState {
    pub role_models: HashMap<String, String>,
    // was `default_model` — alias kept for backward compat with saved config.toml
    #[serde(alias = "default_model")]
    pub codex_default_model: String,
    #[serde(skip)]
    pub available_models: Vec<ModelInfo>,
    pub context_limit_k: u32,
    pub default_effort: ReasoningEffort,
    pub role_effort: HashMap<String, ReasoningEffort>,
    pub auto_advance_batch: bool,
    pub architect_enabled: bool,
    pub auditor_enabled: bool,
    pub scribe_enabled: bool,
    pub critic_enabled: bool,
    pub skip_tests: bool,
    pub max_iterations: u32,
    pub clippy_enabled: bool,
    /// Context pressure threshold (percent, default 80)
    pub context_pressure_pct: u32,
    /// Max concurrent agents (parallel mode)
    pub max_agents: usize,
    /// Enable parallel execution mode
    pub parallel_enabled: bool,
    /// Enable pre-planning phase
    pub pre_plan: bool,
    /// Default model for Cursor ACP agents
    #[serde(default = "default_cursor_model")]
    pub cursor_default_model: String,
    /// Default model for Claude Code CLI agents
    #[serde(default = "default_claude_model")]
    pub claude_default_model: String,
    /// Model used specifically for the Conductor meta-orchestrator.
    #[serde(default = "default_conductor_model")]
    pub conductor_model: String,
    /// Per-role context limit overrides (in thousands)
    #[serde(default)]
    pub role_context_k: HashMap<String, u32>,
    /// Auto-advance to next plan on completion (within a batch)
    pub auto_advance_plan: bool,
    /// Enable Codex fast mode (service_tier="fast"; 1.5× speed, 2× credits; GPT-5.4 only)
    pub fast_mode: bool,
    /// Express mode: single implementer pass, no strategist/reviews, auto-fix on gate failure
    #[serde(default)]
    pub express_mode: bool,
    /// Max auto-fix attempts before failing a plan (express mode only)
    #[serde(default = "default_max_auto_fix_attempts")]
    pub max_auto_fix_attempts: u32,
    /// Model for auto-fixer agent (express mode only)
    #[serde(default = "default_auto_fix_model")]
    pub auto_fix_model: String,
    /// Global fallback model: if any agent spawn fails, retry once with this model.
    /// The fallback may use a different backend (e.g., primary=claude-sonnet-4-6, fallback=composer-2-fast).
    #[serde(default)]
    pub fallback_model: Option<String>,
    /// UI navigation state (not persisted)
    #[serde(skip)]
    pub selected_row: usize,
    /// Which config section is active (0=models, 1=agents, 2=execution, 3=gates)
    #[serde(skip)]
    pub active_section: usize,
    /// Whether a cell is currently being edited
    #[serde(skip)]
    pub editing: bool,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub slug: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Max,
}

impl ReasoningEffort {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Max => "max",
        }
    }

    pub fn cycle_next(&self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Max,
            Self::Max => Self::Low,
        }
    }

    pub fn cycle_prev(&self) -> Self {
        match self {
            Self::Low => Self::Max,
            Self::Medium => Self::Low,
            Self::High => Self::Medium,
            Self::Max => Self::High,
        }
    }
}

fn default_cursor_model() -> String {
    "composer-2-fast".to_string()
}

fn default_max_auto_fix_attempts() -> u32 {
    3
}

fn default_auto_fix_model() -> String {
    "claude-haiku-4-5".to_string()
}

fn default_claude_model() -> String {
    "claude-haiku-4-5".to_string()
}

fn default_conductor_model() -> String {
    "claude-sonnet-4-6".to_string()
}

impl Default for ConfigState {
    fn default() -> Self {
        // Try to read the default model from ~/.codex/config.toml
        let default_model = dirs_home()
            .and_then(|h| std::fs::read_to_string(h.join(".codex/config.toml")).ok())
            .and_then(|content| {
                content
                    .lines()
                    .find(|l| l.starts_with("model"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|v| v.trim().trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "gpt-5.4".to_string());
        let mut available = load_available_models();
        available.extend(load_cursor_models());
        available.extend(load_claude_models());
        let default_model = normalize_model_slug(&default_model, &available);

        let role_models = {
            let mut m = HashMap::new();
            m.insert("implementer".to_string(), "claude-opus-4-6".to_string());
            m.insert("scribe".to_string(), "claude-sonnet-4-6".to_string());
            m
        };

        Self {
            role_models,
            codex_default_model: default_model,
            cursor_default_model: "composer-2-fast".to_string(),
            claude_default_model: "claude-haiku-4-5".to_string(),
            conductor_model: "claude-sonnet-4-6".to_string(),
            available_models: available,
            context_limit_k: 200,
            default_effort: ReasoningEffort::Max,
            role_effort: HashMap::new(),
            auto_advance_batch: true,
            architect_enabled: true,
            auditor_enabled: true,
            scribe_enabled: true,
            critic_enabled: true,
            skip_tests: false,
            max_iterations: 3,
            clippy_enabled: false,
            context_pressure_pct: 80,
            max_agents: 8,
            parallel_enabled: false,
            pre_plan: false,
            role_context_k: HashMap::new(),
            auto_advance_plan: true,
            fast_mode: false,
            express_mode: false,
            max_auto_fix_attempts: 3,
            auto_fix_model: "claude-haiku-4-5".to_string(),
            fallback_model: None,
            selected_row: 0,
            active_section: 0,
            editing: false,
        }
    }
}

impl ConfigState {
    /// Build from AppConfig CLI flags
    pub fn from_app_config(
        model: Option<&str>,
        skip_tests: bool,
        max_iterations: u32,
        no_docs: bool,
        no_review: bool,
        fast_mode: bool,
    ) -> Self {
        let mut cfg = Self::default();
        if let Some(m) = model {
            cfg.codex_default_model = normalize_model_slug(m, &cfg.available_models);
        }
        cfg.skip_tests = skip_tests;
        cfg.max_iterations = max_iterations;
        cfg.fast_mode = fast_mode;
        if no_docs {
            cfg.scribe_enabled = false;
            cfg.critic_enabled = false;
        }
        if no_review {
            cfg.architect_enabled = false;
            cfg.auditor_enabled = false;
        }
        cfg
    }

    /// Build from full AppConfig (captures parallelism and conductor flags)
    pub fn from_full_app_config(config: &crate::app::AppConfig) -> Self {
        let mut cfg = Self::from_app_config(
            config.model.as_deref(),
            config.skip_tests,
            config.max_iterations,
            config.no_docs,
            config.no_review,
            config.fast,
        );
        cfg.max_agents = config.max_agents;
        cfg.parallel_enabled = config.parallel;
        cfg.pre_plan = config.pre_plan;
        cfg.express_mode = config.express;
        if config.fallback_model.is_some() {
            cfg.fallback_model = config.fallback_model.clone();
        }
        cfg
    }

    /// Load persisted config from plan-runs/config.toml
    pub fn load(repo_root: &Path) -> Option<Self> {
        let path = repo_root.join("tmp/plan-runs/config.toml");
        let content = std::fs::read_to_string(path).ok()?;
        let mut cfg: Self = toml::from_str(&content).ok()?;
        let mut models = load_available_models();
        models.extend(load_cursor_models());
        models.extend(load_claude_models());
        cfg.available_models = models;
        Some(cfg)
    }

    /// Save config to plan-runs/config.toml
    pub fn save(&self, repo_root: &Path) -> anyhow::Result<()> {
        let dir = repo_root.join("tmp/plan-runs");
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(dir.join("config.toml"), content)?;
        Ok(())
    }

    /// Get the model slug for a specific role.
    /// Conductor always uses its dedicated model. Per-role overrides win over backend defaults.
    pub fn model_for(&self, role: AgentRole) -> Option<&str> {
        if role == AgentRole::Conductor {
            return Some(&self.conductor_model);
        }
        if let Some(m) = self.role_models.get(role.label()) {
            return Some(m.as_str());
        }
        match role.backend() {
            AgentBackend::Cursor => Some(&self.cursor_default_model),
            AgentBackend::Claude => Some(&self.claude_default_model),
            AgentBackend::Codex => Some(&self.codex_default_model),
        }
    }

    /// Convenience: resolved model spec (slug + backend) for a role.
    pub fn model_spec_for(&self, role: AgentRole) -> ModelSpec {
        ModelSpec::from_slug(self.model_for(role).unwrap_or("gpt-5.4"))
    }

    /// Snapshot the current model slug for every role (used for hot-reload diff).
    pub fn snapshot_models(&self) -> HashMap<String, String> {
        AgentRole::ALL_AGENTS
            .iter()
            .chain(std::iter::once(&AgentRole::Conductor))
            .map(|&r| {
                (
                    r.label().to_string(),
                    self.model_for(r).unwrap_or("").to_string(),
                )
            })
            .collect()
    }

    /// Which roles have a different model compared to a prior snapshot.
    pub fn changed_roles(&self, prev: &HashMap<String, String>) -> Vec<AgentRole> {
        AgentRole::ALL_AGENTS
            .iter()
            .chain(std::iter::once(&AgentRole::Conductor))
            .filter(|&&r| {
                let current = self.model_for(r).unwrap_or("");
                prev.get(r.label()).map(|p| p.as_str()) != Some(current)
            })
            .copied()
            .collect()
    }

    /// Returns true if this role's backend supports reasoning effort.
    /// Codex uses model_reasoning_effort config; Claude uses --effort flag.
    /// Cursor roles ignore effort entirely.
    pub fn effort_configurable(&self, role: AgentRole) -> bool {
        matches!(role.backend(), AgentBackend::Codex | AgentBackend::Claude)
    }

    /// Get reasoning effort for a specific role.
    /// Per-role overrides win; otherwise falls back to role-specific defaults
    /// tuned for one-shot completion.
    pub fn effort_for(&self, role: AgentRole) -> ReasoningEffort {
        if let Some(&effort) = self.role_effort.get(role.label()) {
            return effort;
        }
        // Role-specific defaults: max for anything that needs to get it right
        // first try (implementation, debugging, review, documentation).
        match role {
            // Implementation — must one-shot code + tests
            AgentRole::Implementer => ReasoningEffort::Max,
            // Strategy — deep analysis for execution plan
            AgentRole::Strategist => ReasoningEffort::Max,
            // Architecture review — thorough, find all issues in one pass
            AgentRole::Architect => ReasoningEffort::Max,
            // Audit — catch everything the architect missed
            AgentRole::Auditor => ReasoningEffort::Max,
            // Error diagnosis — precision matters for targeted fixes
            AgentRole::ErrorDiagnoser => ReasoningEffort::Max,
            // Refactoring — needs full understanding of existing code
            AgentRole::Refactorer => ReasoningEffort::Max,
            // Merge resolution — wrong resolution = broken code
            AgentRole::MergeResolver => ReasoningEffort::Max,
            // Integration testing — needs to reason about cross-crate interactions
            AgentRole::IntegrationTester => ReasoningEffort::Max,
            // Documentation — high but not max, less reasoning-heavy
            AgentRole::Scribe | AgentRole::DocVerifier => ReasoningEffort::High,
            // Critic — reviewing docs, high is sufficient
            AgentRole::Critic => ReasoningEffort::High,
            // Pre-planning — high, setting up context
            AgentRole::PrePlanner => ReasoningEffort::High,
            // Research — max, needs to find and synthesize information
            AgentRole::Researcher => ReasoningEffort::Max,
            // Dependency validation — systematic checking, high is fine
            AgentRole::DependencyValidator => ReasoningEffort::High,
            // Pattern extraction — reading + summarizing, high
            AgentRole::PatternExtractor => ReasoningEffort::High,
            // Auto-fixer — medium is enough for targeted compile fixes
            AgentRole::AutoFixer => ReasoningEffort::Medium,
            // Everything else — default to high
            _ => self.default_effort,
        }
    }

    /// Context limit for a specific role (in tokens). Falls back to global.
    pub fn context_limit_for(&self, role: AgentRole) -> u64 {
        self.role_context_k
            .get(role.label())
            .copied()
            .unwrap_or(self.context_limit_k) as u64
            * 1000
    }

    /// Compute section boundary rows dynamically from ALL_AGENTS.len().
    /// Returns (s1, s2, s3, s4, apply, total) where each value is the first
    /// row index of that section (apply = the Apply button row).
    pub fn layout() -> (usize, usize, usize, usize, usize, usize) {
        let n = AgentRole::ALL_AGENTS.len();
        let s1 = 5; // Per-Role Overrides (after 5 backend defaults)
        let s2 = s1 + 1 + n; // Context & Effort (after conductor + n agents)
        let s3 = s2 + 1 + n + 1; // Agent Toggles (after global ctx + n per-role ctx + effort)
        let s4 = s3 + 4; // Execution (after 4 toggles)
        let apply = s4 + 8; // Apply button (after 8 execution rows)
        let total = apply + 1;
        (s1, s2, s3, s4, apply, total)
    }

    /// Total number of config rows for navigation
    pub fn row_count(&self) -> usize {
        Self::layout().5
    }

    /// Get (section_index, row_in_section) from a flat row index
    pub fn section_of_row(&self, row: usize) -> (usize, usize) {
        let (s1, s2, s3, s4, ..) = Self::layout();
        if row < s1 {
            (0, row)
        } else if row < s2 {
            (1, row - s1)
        } else if row < s3 {
            (2, row - s2)
        } else if row < s4 {
            (3, row - s3)
        } else {
            (4, row - s4)
        }
    }
}

/// Load available models from ~/.codex/models_cache.json or use hardcoded fallback
fn load_available_models() -> Vec<ModelInfo> {
    if let Some(home) = dirs_home() {
        let path = home.join(".codex/models_cache.json");
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                // models_cache.json has { "models": [...] } structure
                if let Some(arr) = val
                    .get("models")
                    .and_then(|m| m.as_array())
                    .or_else(|| val.as_array())
                {
                    let models: Vec<ModelInfo> = arr
                        .iter()
                        .filter_map(|v| {
                            let slug = v.get("id").or(v.get("slug"))?.as_str()?;
                            let name = v
                                .get("display_name")
                                .or(v.get("name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or(slug);
                            Some(ModelInfo {
                                slug: slug.to_string(),
                                display_name: name.to_string(),
                            })
                        })
                        .collect();
                    if !models.is_empty() {
                        return models;
                    }
                }
            }
        }
    }
    // Hardcoded fallback (OpenAI Codex-supported models)
    vec![
        ModelInfo {
            slug: "gpt-5.4".into(),
            display_name: "GPT-5.4".into(),
        },
        ModelInfo {
            slug: "gpt-5.4-mini".into(),
            display_name: "GPT-5.4-Mini".into(),
        },
        ModelInfo {
            slug: "gpt-5.3-codex".into(),
            display_name: "GPT-5.3-Codex".into(),
        },
        ModelInfo {
            slug: "gpt-5.2-codex".into(),
            display_name: "GPT-5.2-Codex".into(),
        },
        ModelInfo {
            slug: "gpt-5.1-codex".into(),
            display_name: "GPT-5.1-Codex".into(),
        },
        ModelInfo {
            slug: "gpt-5-codex".into(),
            display_name: "GPT-5-Codex".into(),
        },
    ]
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}

fn load_cursor_models() -> Vec<ModelInfo> {
    // Slugs from `agent --list-models` (2026-03-20).
    vec![
        ModelInfo {
            slug: "auto".into(),
            display_name: "Auto".into(),
        },
        ModelInfo {
            slug: "composer-2-fast".into(),
            display_name: "Composer 2 Fast".into(),
        },
        ModelInfo {
            slug: "composer-2".into(),
            display_name: "Composer 2".into(),
        },
        ModelInfo {
            slug: "composer-1.5".into(),
            display_name: "Composer 1.5".into(),
        },
        ModelInfo {
            slug: "opus-4.6-thinking".into(),
            display_name: "Claude Opus 4.6 (Thinking)".into(),
        },
        ModelInfo {
            slug: "opus-4.6".into(),
            display_name: "Claude Opus 4.6".into(),
        },
        ModelInfo {
            slug: "opus-4.5-thinking".into(),
            display_name: "Claude Opus 4.5 (Thinking)".into(),
        },
        ModelInfo {
            slug: "opus-4.5".into(),
            display_name: "Claude Opus 4.5".into(),
        },
        ModelInfo {
            slug: "sonnet-4.5-thinking".into(),
            display_name: "Claude Sonnet 4.5 (Thinking)".into(),
        },
        ModelInfo {
            slug: "sonnet-4.5".into(),
            display_name: "Claude Sonnet 4.5".into(),
        },
        ModelInfo {
            slug: "gpt-5.4-high".into(),
            display_name: "GPT-5.4 High".into(),
        },
        ModelInfo {
            slug: "gpt-5.4-xhigh-fast".into(),
            display_name: "GPT-5.4 Extra High Fast".into(),
        },
        ModelInfo {
            slug: "gpt-5.3-codex-high".into(),
            display_name: "GPT-5.3 Codex High".into(),
        },
        ModelInfo {
            slug: "gpt-5.2".into(),
            display_name: "GPT-5.2".into(),
        },
    ]
}

fn load_claude_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            slug: "claude-opus-4-6".into(),
            display_name: "Claude Opus 4.6".into(),
        },
        ModelInfo {
            slug: "claude-sonnet-4-6".into(),
            display_name: "Claude Sonnet 4.6".into(),
        },
        ModelInfo {
            slug: "claude-haiku-4-5".into(),
            display_name: "Claude Haiku 4.5".into(),
        },
    ]
}

/// Normalize a model input to match an available model slug.
/// Cursor slugs ("cursor-*", "claude-*", "auto") pass through as-is.
/// Otherwise: exact match, then "gpt-{input}" exact, then prefix fallback.
pub fn normalize_model_slug(input: &str, models: &[ModelInfo]) -> String {
    // Non-Codex slugs pass through without gpt- prefixing
    if input == "auto"
        || input.starts_with("composer-")
        || input.starts_with("cursor-")
        || input.starts_with("claude-")
        || input.starts_with("sonnet-")
        || input.starts_with("opus-")
        || input.starts_with("gemini-")
        || input.starts_with("kimi-")
    {
        return input.to_string();
    }
    // Exact match on input
    if models.iter().any(|m| m.slug == input) {
        return input.to_string();
    }
    // Try "gpt-{input}" exact match
    let prefixed = format!("gpt-{input}");
    if models.iter().any(|m| m.slug == prefixed) {
        return prefixed;
    }
    // Prefix match as fallback
    if let Some(m) = models.iter().find(|m| m.slug.starts_with(&prefixed)) {
        return m.slug.clone();
    }
    input.to_string()
}
