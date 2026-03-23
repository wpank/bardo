use super::plan::PlanInfo;
use super::tasks::TaskChecklist;

/// Complexity classification for a plan, determines which pipeline phases run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanComplexity {
    /// Add a derive, fix an import, tiny change. Skip strategist + reviews.
    Trivial,
    /// Small feature, few files. Low-effort strategist, skip reviews.
    Simple,
    /// Normal plan. Full pipeline.
    Standard,
    /// Large plan touching many crates, state machines, formulas. Full pipeline + critic.
    Complex,
}

/// Pipeline configuration derived from plan complexity.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub complexity: PlanComplexity,
    /// Whether to run the Strategist phase.
    pub run_strategist: bool,
    /// Effort level for the strategist ("low" or "high").
    pub strategist_effort: &'static str,
    /// Whether to run Architect + Auditor reviews.
    pub run_reviews: bool,
    /// Use a single QuickReviewer agent instead of the full Architect → Auditor+Scribe panel.
    /// True for Standard plans; false for Complex (which gets the full panel).
    pub use_quick_review: bool,
    /// Whether to run the Critic (doc review) phase.
    pub run_critic: bool,
    /// Maximum iterations before force-committing.
    pub max_iterations: u32,
}

impl PipelineConfig {
    pub fn for_complexity(complexity: PlanComplexity) -> Self {
        match complexity {
            PlanComplexity::Trivial => Self {
                complexity,
                run_strategist: false,
                strategist_effort: "low",
                run_reviews: false,
                use_quick_review: false,
                run_critic: false,
                max_iterations: 1,
            },
            PlanComplexity::Simple => Self {
                complexity,
                // Strategist removed — bardo-enrich.sh pre-generates briefs + task TOMLs.
                run_strategist: false,
                strategist_effort: "low",
                run_reviews: false,
                use_quick_review: false,
                run_critic: false,
                max_iterations: 2,
            },
            PlanComplexity::Standard => Self {
                complexity,
                run_strategist: false,
                strategist_effort: "medium",
                // Reviews skipped for Standard plans — self-validation + gates are sufficient.
                run_reviews: false,
                use_quick_review: false,
                run_critic: false,
                max_iterations: 2,
            },
            PlanComplexity::Complex => Self {
                complexity,
                run_strategist: false,
                strategist_effort: "high",
                run_reviews: true,
                // Single QuickReviewer for Complex plans, max 2 iterations.
                use_quick_review: true,
                run_critic: false,
                max_iterations: 2,
            },
        }
    }
}

/// Classify a plan's complexity based on its metadata.
pub fn classify_plan(plan: &PlanInfo) -> PlanComplexity {
    let fm = match &plan.frontmatter {
        Some(fm) => fm,
        None => return PlanComplexity::Standard, // no metadata = assume standard
    };

    let crates_count = fm.crates_touched.len();
    let task_count = fm.estimated_tasks.unwrap_or(0);
    let est_minutes = fm.estimated_minutes.unwrap_or(30);

    // Risk escalation: golem-core or 3+ dependencies -> at least Standard
    let touches_core = fm.crates_touched.iter().any(|c| c.contains("golem-core"));
    let many_deps = fm.depends_on.len() >= 3;
    let risk_escalation = touches_core || many_deps;

    // Classify based on size signals
    let base = if crates_count <= 1 && task_count <= 1 && est_minutes <= 10 {
        PlanComplexity::Trivial
    } else if crates_count <= 2 && task_count <= 3 && est_minutes <= 20 {
        PlanComplexity::Simple
    } else if crates_count >= 4 || task_count >= 8 || est_minutes >= 60 {
        PlanComplexity::Complex
    } else {
        PlanComplexity::Standard
    };

    // Apply risk escalation: bump Trivial/Simple to Standard
    if risk_escalation && matches!(base, PlanComplexity::Trivial | PlanComplexity::Simple) {
        PlanComplexity::Standard
    } else {
        base
    }
}

/// Classify a plan using task file metadata as a fallback when frontmatter is absent.
///
/// Most plans have no YAML frontmatter, so `classify_plan` always returns `Standard`.
/// This variant uses the actual task count, estimated time, and cross-plan dependency
/// count from the task file to produce a more accurate (and often lower) classification,
/// enabling Trivial/Simple plans to skip reviews they don't need.
pub fn classify_plan_with_tasks(
    plan: &PlanInfo,
    task_file: Option<&TaskChecklist>,
) -> PlanComplexity {
    // If the plan has real frontmatter with meaningful metadata, use the normal classifier.
    if let Some(ref fm) = plan.frontmatter {
        if fm.estimated_tasks.is_some() || !fm.crates_touched.is_empty() {
            return classify_plan(plan);
        }
    }

    // No useful frontmatter — derive from task file.
    let tf = match task_file {
        Some(tf) => tf,
        None => return classify_plan(plan), // no task file either, fall through
    };

    let task_count = tf.tasks.len();
    let est_minutes: u32 = tf.tasks.iter().filter_map(|t| t.estimated_minutes).sum();
    // Count distinct plans referenced in cross-plan deps (e.g. "09:T3" → plan "09")
    let cross_plan_deps: std::collections::HashSet<String> = tf
        .tasks
        .iter()
        .flat_map(|t| &t.depends_on)
        .filter_map(|d| d.split_once(':').map(|(p, _)| p.to_string()))
        .collect();
    let cross_plan_count = cross_plan_deps.len();

    // Risk escalation: many cross-plan deps or tasks touching core crates → bump up
    let risk_escalation = cross_plan_count >= 3;

    let base = if task_count <= 1 && est_minutes <= 10 {
        PlanComplexity::Trivial
    } else if task_count <= 3 && est_minutes <= 20 {
        PlanComplexity::Simple
    } else if task_count >= 8 || est_minutes >= 60 {
        PlanComplexity::Complex
    } else {
        PlanComplexity::Standard
    };

    if risk_escalation && matches!(base, PlanComplexity::Trivial | PlanComplexity::Simple) {
        PlanComplexity::Standard
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::plan::{PlanFrontmatter, PlanInfo};
    use std::path::PathBuf;

    fn plan_with(crates: Vec<&str>, tasks: usize, minutes: u32, deps: Vec<&str>) -> PlanInfo {
        PlanInfo {
            base: "test-plan".to_string(),
            num: "01".to_string(),
            path: PathBuf::from("plans/01-test.md"),
            frontmatter: Some(PlanFrontmatter {
                plan: Some("test-plan".to_string()),
                depends_on: deps.into_iter().map(String::from).collect(),
                parallel_with: vec![],
                crates_touched: crates.into_iter().map(String::from).collect(),
                estimated_tasks: Some(tasks),
                estimated_parallel_width: None,
                estimated_minutes: Some(minutes),
                refactor_after: false,
                parallel_safe: true,
                tasks: vec![],
            }),
        }
    }

    #[test]
    fn trivial_plan() {
        let plan = plan_with(vec!["golem-types"], 1, 5, vec![]);
        assert_eq!(classify_plan(&plan), PlanComplexity::Trivial);
    }

    #[test]
    fn simple_plan() {
        let plan = plan_with(vec!["golem-types", "golem-state"], 3, 15, vec![]);
        assert_eq!(classify_plan(&plan), PlanComplexity::Simple);
    }

    #[test]
    fn standard_plan() {
        let plan = plan_with(
            vec!["golem-types", "golem-state", "golem-core"],
            5,
            30,
            vec![],
        );
        // touches golem-core -> risk escalation to Standard
        assert_eq!(classify_plan(&plan), PlanComplexity::Standard);
    }

    #[test]
    fn complex_plan() {
        let plan = plan_with(
            vec!["golem-types", "golem-state", "golem-core", "golem-chain"],
            10,
            90,
            vec![],
        );
        assert_eq!(classify_plan(&plan), PlanComplexity::Complex);
    }

    #[test]
    fn risk_escalation_from_deps() {
        let plan = plan_with(vec!["foo"], 1, 5, vec!["01", "02", "03"]);
        // 3+ deps -> at least Standard
        assert_eq!(classify_plan(&plan), PlanComplexity::Standard);
    }

    #[test]
    fn pipeline_config_trivial() {
        let config = PipelineConfig::for_complexity(PlanComplexity::Trivial);
        assert!(!config.run_strategist);
        assert!(!config.run_reviews);
        assert_eq!(config.max_iterations, 1);
    }

    #[test]
    fn pipeline_config_complex() {
        let config = PipelineConfig::for_complexity(PlanComplexity::Complex);
        assert!(!config.run_strategist); // strategist replaced by bardo-enrich.sh pipeline
        assert!(config.run_reviews);
        assert!(!config.run_critic); // critic merged into QuickReviewer
        assert_eq!(config.max_iterations, 2);
    }
}
