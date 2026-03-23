use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentBackend {
    Codex,
    Cursor,
    Claude,
}

impl AgentBackend {
    pub fn short(&self) -> &'static str {
        match self {
            Self::Codex => "cd",
            Self::Cursor => "cx",
            Self::Claude => "cl",
        }
    }

    /// Infer backend from model slug.
    /// Claude slugs: claude-* (direct Anthropic API via claude CLI)
    /// Cursor model slugs: composer-*, cursor-*, auto, sonnet-*, opus-*, haiku-*, gemini-*, kimi-*
    /// Everything else (gpt-*, o3, o4-mini, etc.) routes to Codex.
    pub fn from_model(slug: &str) -> Self {
        if slug.starts_with("claude-") {
            Self::Claude
        } else if slug.starts_with("composer-")
            || slug.starts_with("cursor-")
            || slug == "auto"
            || slug.starts_with("sonnet-")
            || slug.starts_with("opus-")
            || slug.starts_with("haiku-")
            || slug.starts_with("gemini-")
            || slug.starts_with("kimi-")
        {
            Self::Cursor
        } else {
            Self::Codex
        }
    }
}

/// A fully-resolved model specification: slug + inferred backend.
/// Created from any model string; the backend is always derived from the slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSpec {
    pub slug: String,
    pub backend: AgentBackend,
}

impl ModelSpec {
    pub fn from_slug(slug: impl Into<String>) -> Self {
        let slug = slug.into();
        let backend = AgentBackend::from_model(&slug);
        Self { slug, backend }
    }

    /// Short display label (strips common prefixes for TUI compactness).
    pub fn short(&self) -> String {
        self.slug
            .replace("composer-", "cx-")
            .replace("claude-", "cl-")
            .replace("gpt-", "")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRole {
    Conductor,
    Strategist,
    Implementer,
    Architect,
    Auditor,
    Scribe,
    Critic,
    Refactorer,
    PrePlanner,
    DocVerifier,
    IntegrationTester,
    MergeResolver,
    TerminalValidator,
    GolemLifecycleTester,
    SpecDriftDetector,
    RegressionDetector,
    PerformanceSentinel,
    CoverageTracker,
    PlanLifecycleManager,
    CrossSystemTester,
    // New specialized agent roles
    ErrorDiagnoser,
    Researcher,
    DependencyValidator,
    PatternExtractor,
    SnapshotComparator,
    /// Lightweight fix agent used in express mode after gate failure.
    AutoFixer,
    /// Single-pass reviewer for Standard plans: combines arch+audit concerns in one
    /// focused 5-minute pass. Replaces the Architect → Auditor+Scribe pipeline.
    QuickReviewer,
    /// Validates end-to-end pipeline (mirage + terminal + runtime) in full-loop test phase.
    FullLoopValidator,
}

impl AgentRole {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Conductor => "conductor",
            Self::Strategist => "strategist",
            Self::Implementer => "implementer",
            Self::Architect => "architect",
            Self::Auditor => "auditor",
            Self::Scribe => "scribe",
            Self::Critic => "critic",
            Self::Refactorer => "refactorer",
            Self::PrePlanner => "pre-planner",
            Self::DocVerifier => "doc-verifier",
            Self::IntegrationTester => "integration-tester",
            Self::MergeResolver => "merge-resolver",
            Self::TerminalValidator => "terminal-validator",
            Self::GolemLifecycleTester => "golem-lifecycle-tester",
            Self::SpecDriftDetector => "spec-drift-detector",
            Self::RegressionDetector => "regression-detector",
            Self::PerformanceSentinel => "performance-sentinel",
            Self::CoverageTracker => "coverage-tracker",
            Self::PlanLifecycleManager => "plan-lifecycle-mgr",
            Self::CrossSystemTester => "cross-system-tester",
            Self::ErrorDiagnoser => "error-diagnoser",
            Self::Researcher => "researcher",
            Self::DependencyValidator => "dep-validator",
            Self::PatternExtractor => "pattern-extractor",
            Self::SnapshotComparator => "snapshot-comparator",
            Self::AutoFixer => "auto-fixer",
            Self::QuickReviewer => "quick-reviewer",
            Self::FullLoopValidator => "full-loop-validator",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            Self::Conductor => "cond",
            Self::Strategist => "strat",
            Self::Implementer => "impl",
            Self::Architect => "arch",
            Self::Auditor => "audit",
            Self::Scribe => "scribe",
            Self::Critic => "critic",
            Self::Refactorer => "refac",
            Self::PrePlanner => "prepl",
            Self::DocVerifier => "docvf",
            Self::IntegrationTester => "itest",
            Self::MergeResolver => "merge",
            Self::TerminalValidator => "tval",
            Self::GolemLifecycleTester => "glct",
            Self::SpecDriftDetector => "sdrf",
            Self::RegressionDetector => "regd",
            Self::PerformanceSentinel => "perf",
            Self::CoverageTracker => "covr",
            Self::PlanLifecycleManager => "plcm",
            Self::CrossSystemTester => "xsys",
            Self::ErrorDiagnoser => "errdx",
            Self::Researcher => "rsrch",
            Self::DependencyValidator => "depv",
            Self::PatternExtractor => "patrn",
            Self::SnapshotComparator => "snapc",
            Self::AutoFixer => "afix",
            Self::QuickReviewer => "qrev",
            Self::FullLoopValidator => "FLV",
        }
    }

    pub fn backend(self) -> AgentBackend {
        match self {
            // Claude roles (all agent-mode roles default to Claude Code CLI)
            Self::Conductor
            | Self::Implementer
            | Self::Strategist
            | Self::Auditor
            | Self::Scribe
            | Self::Critic
            | Self::Researcher
            | Self::AutoFixer
            | Self::QuickReviewer
            | Self::FullLoopValidator => AgentBackend::Claude,

            // Codex roles (remaining)
            Self::Architect
            | Self::Refactorer
            | Self::PrePlanner
            | Self::DocVerifier
            | Self::IntegrationTester
            | Self::MergeResolver
            | Self::TerminalValidator
            | Self::GolemLifecycleTester
            | Self::SpecDriftDetector
            | Self::RegressionDetector
            | Self::PerformanceSentinel
            | Self::CoverageTracker
            | Self::PlanLifecycleManager
            | Self::CrossSystemTester
            | Self::ErrorDiagnoser
            | Self::DependencyValidator
            | Self::PatternExtractor
            | Self::SnapshotComparator => AgentBackend::Codex,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Conductor => 0,
            Self::Strategist => 1,
            Self::Implementer => 2,
            Self::Architect => 3,
            Self::Auditor => 4,
            Self::Scribe => 5,
            Self::Critic => 6,
            Self::Refactorer => 7,
            Self::PrePlanner => 8,
            Self::DocVerifier => 9,
            Self::IntegrationTester => 10,
            Self::MergeResolver => 11,
            Self::TerminalValidator => 12,
            Self::GolemLifecycleTester => 13,
            Self::SpecDriftDetector => 14,
            Self::RegressionDetector => 15,
            Self::PerformanceSentinel => 16,
            Self::CoverageTracker => 17,
            Self::PlanLifecycleManager => 18,
            Self::CrossSystemTester => 19,
            Self::ErrorDiagnoser => 20,
            Self::Researcher => 21,
            Self::DependencyValidator => 22,
            Self::PatternExtractor => 23,
            Self::SnapshotComparator => 24,
            Self::AutoFixer => 25,
            Self::QuickReviewer => 26,
            Self::FullLoopValidator => 27,
        }
    }

    /// All roles excluding Conductor (which is the meta-orchestrator)
    pub const ALL_AGENTS: [AgentRole; 27] = [
        AgentRole::Strategist,
        AgentRole::Implementer,
        AgentRole::Architect,
        AgentRole::Auditor,
        AgentRole::Scribe,
        AgentRole::Critic,
        AgentRole::Refactorer,
        AgentRole::PrePlanner,
        AgentRole::DocVerifier,
        AgentRole::IntegrationTester,
        AgentRole::MergeResolver,
        AgentRole::TerminalValidator,
        AgentRole::GolemLifecycleTester,
        AgentRole::SpecDriftDetector,
        AgentRole::RegressionDetector,
        AgentRole::PerformanceSentinel,
        AgentRole::CoverageTracker,
        AgentRole::PlanLifecycleManager,
        AgentRole::CrossSystemTester,
        AgentRole::ErrorDiagnoser,
        AgentRole::Researcher,
        AgentRole::DependencyValidator,
        AgentRole::PatternExtractor,
        AgentRole::SnapshotComparator,
        AgentRole::AutoFixer,
        AgentRole::QuickReviewer,
        AgentRole::FullLoopValidator,
    ];
}

impl fmt::Display for AgentRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_loop_validator_label_short_backend() {
        let r = AgentRole::FullLoopValidator;
        assert_eq!(r.label(), "full-loop-validator");
        assert_eq!(r.short(), "FLV");
        assert_eq!(r.backend(), AgentBackend::Claude);
    }
}
