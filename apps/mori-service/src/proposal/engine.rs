//! Proposal generation engine.
//!
//! Takes a `Draft` and produces a fully costed `Proposal` with milestones,
//! plan names, time estimates, and cost breakdowns. The engine works entirely
//! from heuristics: no LLM calls, no network I/O. It parses the accumulated
//! draft context, identifies feature boundaries, classifies complexity, and
//! runs the numbers through the cost module.

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    CostEstimate, Draft, Milestone, MilestoneStatus, ModifyProposalRequest, Proposal,
    ProposalStatus,
};

use super::cost::{self, TierDistribution};

// ---------------------------------------------------------------------------
// Complexity classification
// ---------------------------------------------------------------------------

/// Overall complexity of a feature area, derived from keyword analysis
/// and word count of the description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Complexity {
    /// Config change, dependency bump, one-liner. 2-3 tasks.
    Trivial,
    /// Small CLI tool, simple endpoint, single module. 3-5 tasks.
    Small,
    /// Standard feature: model + handler + tests + docs. 5-10 tasks.
    Medium,
    /// Multi-module feature: service layer, persistence, API, tests. 10-18 tasks.
    Large,
    /// Cross-system work: migrations, multi-crate refactors, infra. 18-30 tasks.
    Epic,
}

impl Complexity {
    /// Heuristic task count range for this complexity level.
    /// Returns (min, max); the engine picks the midpoint.
    fn task_range(&self) -> (u32, u32) {
        match self {
            Complexity::Trivial => (2, 3),
            Complexity::Small => (3, 5),
            Complexity::Medium => (5, 10),
            Complexity::Large => (10, 18),
            Complexity::Epic => (18, 30),
        }
    }

    /// Midpoint of the task range.
    fn typical_tasks(&self) -> u32 {
        let (lo, hi) = self.task_range();
        (lo + hi) / 2
    }

    /// Model tier distribution appropriate for this complexity.
    fn tier_distribution(&self) -> TierDistribution {
        match self {
            Complexity::Trivial => TierDistribution::simple(),
            Complexity::Small => TierDistribution::simple(),
            Complexity::Medium => TierDistribution::standard(),
            Complexity::Large => TierDistribution::standard(),
            Complexity::Epic => TierDistribution::complex(),
        }
    }
}

// ---------------------------------------------------------------------------
// Keyword sets for classification
// ---------------------------------------------------------------------------

/// Keywords that push complexity upward.
const COMPLEX_KEYWORDS: &[&str] = &[
    "migration",
    "refactor",
    "cross-crate",
    "distributed",
    "consensus",
    "blockchain",
    "cryptograph",
    "real-time",
    "streaming",
    "websocket",
    "oauth",
    "authentication",
    "authorization",
    "multi-tenant",
    "deployment",
    "infrastructure",
    "kubernetes",
    "docker",
    "database",
    "schema",
    "index",
    "performance",
    "optimization",
    "concurrency",
    "parallel",
    "async",
    "security",
    "audit",
    "encryption",
    "protocol",
    "state machine",
    "orchestrat",
    "pipeline",
    "queue",
    "event-driven",
    "microservice",
];

/// Keywords that indicate simpler work.
const SIMPLE_KEYWORDS: &[&str] = &[
    "cli",
    "script",
    "config",
    "flag",
    "rename",
    "update",
    "bump",
    "fix",
    "typo",
    "readme",
    "docs",
    "comment",
    "log",
    "print",
    "env",
    "variable",
    "constant",
    "enum variant",
    "helper",
    "utility",
];

/// Classify complexity from a text description.
fn classify_complexity(text: &str) -> Complexity {
    let lower = text.to_lowercase();
    let word_count = lower.split_whitespace().count();

    let complex_hits = COMPLEX_KEYWORDS
        .iter()
        .filter(|kw| lower.contains(**kw))
        .count();

    let simple_hits = SIMPLE_KEYWORDS
        .iter()
        .filter(|kw| lower.contains(**kw))
        .count();

    // Score: positive means complex, negative means simple.
    let signal = complex_hits as i32 - simple_hits as i32;

    // Word count contributes: longer descriptions usually describe bigger work.
    let length_signal = match word_count {
        0..=20 => -2,
        21..=60 => -1,
        61..=150 => 0,
        151..=400 => 1,
        _ => 2,
    };

    let total = signal + length_signal;

    match total {
        ..=-2 => Complexity::Trivial,
        -1 => Complexity::Small,
        0..=1 => Complexity::Medium,
        2..=3 => Complexity::Large,
        _ => Complexity::Epic,
    }
}

// ---------------------------------------------------------------------------
// Feature boundary detection
// ---------------------------------------------------------------------------

/// A detected feature area within the draft context.
#[derive(Debug, Clone)]
struct FeatureArea {
    /// Human-readable name for this feature area.
    name: String,
    /// The raw text describing this feature.
    description: String,
    /// Classified complexity.
    complexity: Complexity,
}

/// Boundary markers that indicate a new feature area in draft text.
const BOUNDARY_PATTERNS: &[&str] = &[
    "also add",
    "also implement",
    "also build",
    "also create",
    "and then",
    "additionally",
    "on top of that",
    "next,",
    "then,",
    "separately,",
    "another feature",
    "another thing",
    "we also need",
    "we'll also need",
    "plus ",
    "as well as",
    "in addition",
    "besides that",
    "beyond that",
    "the second",
    "the third",
    "the fourth",
    "phase 2",
    "phase 3",
    "milestone 2",
    "milestone 3",
    "feature 2",
    "feature 3",
];

/// Parse the accumulated draft context into distinct feature areas.
///
/// Uses three strategies in order:
/// 1. Numbered list detection (1. / 2. / 3. or - / * bullet lists with feature content)
/// 2. Boundary phrase splitting ("also add", "and then", etc.)
/// 3. Paragraph-based splitting for long text with natural breaks
/// 4. Falls back to treating the entire context as a single feature area
fn parse_feature_areas(context: &str) -> Vec<FeatureArea> {
    let trimmed = context.trim();
    if trimmed.is_empty() {
        return vec![FeatureArea {
            name: "Implementation".to_string(),
            description: String::new(),
            complexity: Complexity::Small,
        }];
    }

    // Strategy 1: numbered lists.
    let numbered = extract_numbered_items(trimmed);
    if numbered.len() >= 2 {
        return numbered
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                let name = derive_feature_name(&text, i);
                let complexity = classify_complexity(&text);
                FeatureArea {
                    name,
                    description: text,
                    complexity,
                }
            })
            .collect();
    }

    // Strategy 2: boundary phrase splitting.
    let boundary_split = split_by_boundaries(trimmed);
    if boundary_split.len() >= 2 {
        return boundary_split
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                let name = derive_feature_name(&text, i);
                let complexity = classify_complexity(&text);
                FeatureArea {
                    name,
                    description: text,
                    complexity,
                }
            })
            .collect();
    }

    // Strategy 3: paragraph splitting for long text.
    if trimmed.len() > 500 {
        let paragraphs = split_paragraphs(trimmed);
        if paragraphs.len() >= 2 {
            return paragraphs
                .into_iter()
                .enumerate()
                .map(|(i, text)| {
                    let name = derive_feature_name(&text, i);
                    let complexity = classify_complexity(&text);
                    FeatureArea {
                        name,
                        description: text,
                        complexity,
                    }
                })
                .collect();
        }
    }

    // Fallback: single feature area.
    let complexity = classify_complexity(trimmed);
    let name = derive_feature_name(trimmed, 0);
    vec![FeatureArea {
        name,
        description: trimmed.to_string(),
        complexity,
    }]
}

/// Extract items from a numbered list (1. / 2. / 3. or - / * bullet points).
fn extract_numbered_items(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_list = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Check for numbered item start: "1.", "2.", etc.
        let is_numbered =
            trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) && trimmed.contains('.');

        // Check for bullet item start: "- " or "* "
        let is_bullet = trimmed.starts_with("- ") || trimmed.starts_with("* ");

        if is_numbered || is_bullet {
            if in_list && !current.trim().is_empty() {
                items.push(current.trim().to_string());
            }
            // Strip the marker.
            let content = if is_numbered {
                trimmed
                    .splitn(2, '.')
                    .nth(1)
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string()
            } else {
                trimmed[2..].trim().to_string()
            };
            current = content;
            in_list = true;
        } else if in_list && !trimmed.is_empty() {
            // Continuation line.
            current.push(' ');
            current.push_str(trimmed);
        }
    }

    if in_list && !current.trim().is_empty() {
        items.push(current.trim().to_string());
    }

    // Only return if we found actual list content (items with >3 words each).
    let meaningful: Vec<String> = items
        .into_iter()
        .filter(|item| item.split_whitespace().count() >= 3)
        .collect();

    meaningful
}

/// Split text by boundary phrases.
fn split_by_boundaries(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut split_points: Vec<usize> = Vec::new();

    for pattern in BOUNDARY_PATTERNS {
        let mut start = 0;
        while let Some(pos) = lower[start..].find(pattern) {
            let abs_pos = start + pos;
            // Only split if the boundary is near a sentence start (preceded by
            // period, newline, or start of text within a few chars).
            if abs_pos == 0
                || lower[..abs_pos]
                    .rfind(|c: char| c == '.' || c == '\n')
                    .is_some_and(|p| abs_pos - p < 10)
            {
                split_points.push(abs_pos);
            }
            start = abs_pos + pattern.len();
        }
    }

    if split_points.is_empty() {
        return vec![text.to_string()];
    }

    split_points.sort_unstable();
    split_points.dedup();

    // Filter out points that are too close together (< 40 chars apart).
    let mut filtered = vec![split_points[0]];
    for &p in &split_points[1..] {
        if p - *filtered.last().unwrap() >= 40 {
            filtered.push(p);
        }
    }

    // Build segments.
    let mut segments = Vec::new();
    let mut prev = 0;
    for &point in &filtered {
        let seg = text[prev..point].trim().to_string();
        if !seg.is_empty() && seg.split_whitespace().count() >= 5 {
            segments.push(seg);
        }
        prev = point;
    }
    // Last segment.
    let tail = text[prev..].trim().to_string();
    if !tail.is_empty() && tail.split_whitespace().count() >= 5 {
        segments.push(tail);
    }

    segments
}

/// Split long text into paragraphs (double-newline separated).
fn split_paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|p| p.trim().to_string())
        .filter(|p| p.split_whitespace().count() >= 8)
        .collect()
}

/// Derive a human-readable milestone name from a feature description.
///
/// Attempts to extract a noun-phrase from the first sentence. Falls back to
/// generic naming with the index.
fn derive_feature_name(description: &str, index: usize) -> String {
    let lower = description.to_lowercase();

    // Try to match common patterns: "build a <thing>", "implement <thing>",
    // "add <thing>", "create <thing>", etc.
    let extraction_verbs = [
        "implement ",
        "build ",
        "create ",
        "add ",
        "set up ",
        "design ",
        "write ",
        "develop ",
    ];

    for verb in &extraction_verbs {
        if let Some(pos) = lower.find(verb) {
            let after_verb = &description[pos + verb.len()..];
            let name = extract_noun_phrase(after_verb);
            if !name.is_empty() {
                return titlecase(&name);
            }
        }
    }

    // Try to grab the first N meaningful words.
    let words: Vec<&str> = description
        .split_whitespace()
        .filter(|w| w.len() > 2 || w.chars().next().is_some_and(|c| c.is_uppercase()))
        .take(4)
        .collect();

    if words.len() >= 2 {
        return titlecase(&words.join(" "));
    }

    // Generic fallback.
    format!("Feature {}", index + 1)
}

/// Extract a noun phrase: take words until hitting a verb, conjunction, or punctuation.
fn extract_noun_phrase(text: &str) -> String {
    let stoppers = [
        "with",
        "that",
        "which",
        "using",
        "for",
        "from",
        "into",
        "and",
        "but",
        "or",
        "then",
        "also",
        "including",
        "supporting",
        "where",
        "when",
    ];
    let words: Vec<&str> = text
        .split_whitespace()
        .take(6)
        .take_while(|w| {
            let lower = w.to_lowercase();
            let clean = lower.trim_matches(|c: char| !c.is_alphanumeric());
            !stoppers.contains(&clean.as_ref()) && !w.ends_with(',') && !w.ends_with('.')
        })
        .collect();

    words.join(" ")
}

/// Simple titlecase: capitalize the first letter of each word.
fn titlecase(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let rest: String = chars.collect();
                    format!("{}{}", first.to_uppercase(), rest)
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Plan name generation
// ---------------------------------------------------------------------------

/// Generate NN-kebab-case plan names from a milestone name.
///
/// Given a milestone like "OAuth Providers" and a starting index of 3,
/// produces names like "03-oauth-providers".
fn generate_plan_names(milestone_name: &str, count: u32, start_index: &mut u32) -> Vec<String> {
    let base = milestone_name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>();

    let slug: String = base.split_whitespace().collect::<Vec<_>>().join("-");

    let mut names = Vec::with_capacity(count as usize);

    if count == 1 {
        names.push(format!("{:02}-{slug}", *start_index));
        *start_index += 1;
    } else {
        // Multiple plans: append a qualifier.
        let qualifiers = [
            "core",
            "models",
            "handlers",
            "tests",
            "integration",
            "api",
            "config",
            "migration",
            "validation",
            "hooks",
        ];
        for i in 0..count {
            let qualifier = qualifiers.get(i as usize).unwrap_or(&"ext");
            names.push(format!("{:02}-{slug}-{qualifier}", *start_index));
            *start_index += 1;
        }
    }

    names
}

// ---------------------------------------------------------------------------
// ProposalEngine
// ---------------------------------------------------------------------------

/// The proposal generation engine. Stateless except for a retry buffer configuration.
///
/// Takes a `Draft` with accumulated context and produces a structured `Proposal`
/// with milestones, cost estimates, and time estimates. All estimation is
/// heuristic-based: no LLM calls during proposal generation.
pub struct ProposalEngine {
    /// Fraction of estimated cost reserved for retries (e.g. 0.15 = 15%).
    retry_buffer_pct: f64,
}

impl ProposalEngine {
    /// Create a new engine with the given retry buffer percentage.
    pub fn new(retry_buffer_pct: f64) -> Self {
        Self { retry_buffer_pct }
    }

    /// Analyze draft context and generate a structured proposal with milestones.
    ///
    /// `spread_pct` is the gateway margin applied on top of raw costs. Pass `None`
    /// to use the default 20%.
    pub fn generate_proposal(&self, draft: &Draft, spread_pct: f64) -> anyhow::Result<Proposal> {
        let features = parse_feature_areas(&draft.accumulated_context);

        if features.is_empty() {
            anyhow::bail!("draft context is empty; cannot generate proposal");
        }

        let mut milestones = Vec::with_capacity(features.len());
        let mut plan_index: u32 = 1;
        let mut total_inference = 0.0_f64;
        let mut total_compute = 0.0_f64;

        for feature in &features {
            let task_count = feature.complexity.typical_tasks();
            let distribution = feature.complexity.tier_distribution();
            let plan_cost = cost::estimate_milestone_cost(task_count, &distribution, spread_pct);

            // Determine plan count for this milestone. Larger milestones split
            // into multiple plans for parallelism.
            let plan_count = self.plan_count_for(task_count);
            let plan_names = generate_plan_names(&feature.name, plan_count, &mut plan_index);

            let estimated_time = self.estimate_time(task_count, feature.complexity);

            milestones.push(Milestone {
                name: feature.name.clone(),
                plans: plan_names,
                task_count,
                estimated_cost: CostEstimate {
                    inference: round2(plan_cost.inference),
                    compute: round2(plan_cost.compute),
                    total: round2(plan_cost.total),
                },
                estimated_time,
                actual_cost: None,
                status: MilestoneStatus::Pending,
            });

            total_inference += plan_cost.inference;
            total_compute += plan_cost.compute;
        }

        let subtotal_raw = total_inference + total_compute;
        let subtotal_with_spread = subtotal_raw * (1.0 + spread_pct);
        let retry_buffer = subtotal_with_spread * self.retry_buffer_pct;
        let total_cost = round2(subtotal_with_spread + retry_buffer);
        let net_cost = round2(total_cost - draft.session_cost_so_far);

        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(24);
        let proposal_id = format!("p-{}", &Uuid::new_v4().to_string()[..4]);

        Ok(Proposal {
            proposal_id,
            draft_id: draft.draft_id.clone(),
            version: 1,
            milestones,
            total_cost,
            draft_cost_spent: draft.session_cost_so_far,
            net_cost: net_cost.max(0.0),
            valid_for: "24h".to_string(),
            status: ProposalStatus::Proposed,
            created_at: now,
            expires_at,
        })
    }

    /// Re-estimate a modified proposal in place.
    ///
    /// Recalculates all milestone costs and updates totals. Called after adding
    /// or removing milestones during the iteration phase.
    pub fn re_estimate(&self, proposal: &mut Proposal, spread_pct: f64) -> anyhow::Result<()> {
        let mut total_inference = 0.0_f64;
        let mut total_compute = 0.0_f64;

        for ms in &mut proposal.milestones {
            // Re-derive complexity from task count (since we may not have
            // the original description after modification).
            let complexity = complexity_from_task_count(ms.task_count);
            let distribution = complexity.tier_distribution();
            let plan_cost = cost::estimate_milestone_cost(ms.task_count, &distribution, spread_pct);

            ms.estimated_cost = CostEstimate {
                inference: round2(plan_cost.inference),
                compute: round2(plan_cost.compute),
                total: round2(plan_cost.total),
            };
            ms.estimated_time = self.estimate_time(ms.task_count, complexity);

            total_inference += plan_cost.inference;
            total_compute += plan_cost.compute;
        }

        let subtotal_raw = total_inference + total_compute;
        let subtotal_with_spread = subtotal_raw * (1.0 + spread_pct);
        let retry_buffer = subtotal_with_spread * self.retry_buffer_pct;
        proposal.total_cost = round2(subtotal_with_spread + retry_buffer);
        proposal.net_cost = round2((proposal.total_cost - proposal.draft_cost_spent).max(0.0));
        proposal.version += 1;

        Ok(())
    }

    /// Produce an initial assistant response for a new draft.
    ///
    /// Takes the user's initial content and an optional repo URL, and returns
    /// a string describing a rough structure for the work. Synchronous (no LLM
    /// calls; heuristic only).
    pub fn draft_respond(&self, content: &str, repo_url: Option<&str>) -> anyhow::Result<String> {
        let features = parse_feature_areas(content);
        let mut lines = Vec::new();

        lines.push("Here's what I'm seeing in your request:".to_string());
        for (i, f) in features.iter().enumerate() {
            lines.push(format!(
                "{}. **{}** ({:?} complexity, ~{} tasks)",
                i + 1,
                f.name,
                f.complexity,
                f.complexity.typical_tasks(),
            ));
        }

        if let Some(url) = repo_url {
            lines.push(format!(
                "\nI'll use the repo at {url} for codebase context."
            ));
        }

        lines.push(
            "\nWant to refine any of these areas, or should I generate a proposal?".to_string(),
        );
        Ok(lines.join("\n"))
    }

    /// Continue a draft conversation given accumulated context and a new message.
    ///
    /// Returns a response string. Synchronous.
    pub fn draft_iterate(
        &self,
        accumulated_context: &str,
        new_message: &str,
    ) -> anyhow::Result<String> {
        let combined = format!("{accumulated_context}\n{new_message}");
        let features = parse_feature_areas(&combined);

        let mut lines = Vec::new();
        lines.push("Updated understanding:".to_string());
        for (i, f) in features.iter().enumerate() {
            lines.push(format!(
                "{}. **{}** ({:?}, ~{} tasks)",
                i + 1,
                f.name,
                f.complexity,
                f.complexity.typical_tasks(),
            ));
        }
        lines.push("\nAnything else to adjust, or ready for a proposal?".to_string());
        Ok(lines.join("\n"))
    }

    /// Apply modifications to a proposal (add/remove milestones) and re-estimate.
    ///
    /// Returns the updated proposal with incremented version. Synchronous.
    pub fn modify_proposal(
        &self,
        mut proposal: Proposal,
        request: &ModifyProposalRequest,
        spread_pct: f64,
    ) -> anyhow::Result<Proposal> {
        // Remove milestones by name.
        if let Some(ref names) = request.remove_milestones {
            proposal.milestones.retain(|m| !names.contains(&m.name));
        }

        // Add new milestones from descriptions.
        if let Some(ref additions) = request.add_milestones {
            let mut plan_index: u32 = proposal
                .milestones
                .iter()
                .flat_map(|m| &m.plans)
                .filter_map(|p| p.split('-').next().and_then(|n| n.parse::<u32>().ok()))
                .max()
                .unwrap_or(0)
                + 1;

            for desc in additions {
                let complexity = classify_complexity(&desc.description);
                let task_count = complexity.typical_tasks();
                let name = derive_feature_name(&desc.description, proposal.milestones.len());
                let plan_count = self.plan_count_for(task_count);
                let plan_names = generate_plan_names(&name, plan_count, &mut plan_index);
                let distribution = complexity.tier_distribution();
                let plan_cost =
                    cost::estimate_milestone_cost(task_count, &distribution, spread_pct);
                let estimated_time = self.estimate_time(task_count, complexity);

                proposal.milestones.push(Milestone {
                    name,
                    plans: plan_names,
                    task_count,
                    estimated_cost: CostEstimate {
                        inference: round2(plan_cost.inference),
                        compute: round2(plan_cost.compute),
                        total: round2(plan_cost.total),
                    },
                    estimated_time,
                    actual_cost: None,
                    status: MilestoneStatus::Pending,
                });
            }
        }

        self.re_estimate(&mut proposal, spread_pct)?;
        Ok(proposal)
    }

    /// Generate a new milestone for a feature being added to a running build.
    ///
    /// Takes a description and a max budget. Returns the new `Milestone`.
    /// Synchronous.
    pub fn plan_feature(
        &self,
        description: &str,
        _max_additional_cost: f64,
    ) -> anyhow::Result<Milestone> {
        let complexity = classify_complexity(description);
        let task_count = complexity.typical_tasks();
        let name = derive_feature_name(description, 0);
        let plan_count = self.plan_count_for(task_count);
        let mut plan_index: u32 = 90; // high index to avoid collisions with existing plans
        let plan_names = generate_plan_names(&name, plan_count, &mut plan_index);
        let distribution = complexity.tier_distribution();
        let spread_pct = cost::DEFAULT_SPREAD_PCT;
        let plan_cost = cost::estimate_milestone_cost(task_count, &distribution, spread_pct);
        let estimated_time = self.estimate_time(task_count, complexity);

        Ok(Milestone {
            name,
            plans: plan_names,
            task_count,
            estimated_cost: CostEstimate {
                inference: round2(plan_cost.inference),
                compute: round2(plan_cost.compute),
                total: round2(plan_cost.total),
            },
            estimated_time,
            actual_cost: None,
            status: MilestoneStatus::Pending,
        })
    }

    /// Quick cost estimate from a description string. Returns a full `Proposal`
    /// built from a synthetic draft so Twitter and other lightweight integrations
    /// can show cost breakdowns without going through the full draft flow.
    pub fn quick_estimate(&self, description: &str) -> anyhow::Result<Proposal> {
        let now = chrono::Utc::now();
        let draft = Draft {
            draft_id: "quick".to_string(),
            messages: vec![],
            accumulated_context: description.to_string(),
            session_cost_so_far: 0.0,
            repo_url: None,
            created_at: now,
            updated_at: now,
        };
        self.generate_proposal(&draft, cost::DEFAULT_SPREAD_PCT)
    }

    /// Estimate wall-clock time for a milestone based on task count,
    /// parallelism potential, and complexity.
    ///
    /// Returns a human-readable string like "~45 min" or "~2.5 hr".
    fn estimate_time(&self, task_count: u32, complexity: Complexity) -> String {
        // Base minutes per task varies by complexity:
        // - Trivial/Small tasks: fast model, quick turnaround (~3 min/task).
        // - Medium tasks: standard implementation cycle (~5 min/task).
        // - Large/Epic: complex reasoning, more iterations (~7 min/task).
        let minutes_per_task: f64 = match complexity {
            Complexity::Trivial => 2.5,
            Complexity::Small => 3.0,
            Complexity::Medium => 5.0,
            Complexity::Large => 6.5,
            Complexity::Epic => 8.0,
        };

        // Parallelism factor: we assume wave scheduling achieves ~60% efficiency.
        // With default parallelism width of 8, effective concurrency is ~5 tasks.
        let effective_parallelism = match task_count {
            0..=3 => 1.0,
            4..=6 => 2.0,
            7..=12 => 3.5,
            13..=20 => 5.0,
            _ => 6.0,
        };

        // Add overhead for enrichment pipeline (~2 min) and merge/integration (~1 min per plan).
        let plan_count = self.plan_count_for(task_count) as f64;
        let enrichment_overhead = 2.0;
        let merge_overhead = plan_count * 1.0;

        let task_time = (task_count as f64 * minutes_per_task) / effective_parallelism;
        let total_minutes = task_time + enrichment_overhead + merge_overhead;

        format_duration(total_minutes)
    }

    /// Decide how many plans a milestone should split into, based on task count.
    fn plan_count_for(&self, task_count: u32) -> u32 {
        match task_count {
            0..=4 => 1,
            5..=8 => 2,
            9..=14 => 3,
            15..=22 => 4,
            _ => 5,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Reverse-map a task count back to an approximate complexity level.
/// Used by `re_estimate` when the original description is unavailable.
fn complexity_from_task_count(task_count: u32) -> Complexity {
    match task_count {
        0..=3 => Complexity::Trivial,
        4..=5 => Complexity::Small,
        6..=10 => Complexity::Medium,
        11..=18 => Complexity::Large,
        _ => Complexity::Epic,
    }
}

/// Round to 2 decimal places.
fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Format a duration in minutes to a human-readable string.
fn format_duration(minutes: f64) -> String {
    if minutes < 1.0 {
        "~1 min".to_string()
    } else if minutes < 60.0 {
        format!("~{} min", minutes.round() as u32)
    } else {
        let hours = minutes / 60.0;
        if (hours - hours.round()).abs() < 0.15 {
            format!("~{} hr", hours.round() as u32)
        } else {
            format!("~{:.1} hr", (hours * 2.0).round() / 2.0)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::proposal::cost::DEFAULT_SPREAD_PCT;
    use crate::types::Draft;

    fn make_draft(context: &str) -> Draft {
        Draft {
            draft_id: "d-test".to_string(),
            messages: vec![],
            accumulated_context: context.to_string(),
            session_cost_so_far: 0.30,
            repo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn classify_simple_context() {
        let c = classify_complexity("add a CLI flag for verbose output");
        assert!(c <= Complexity::Small);
    }

    #[test]
    fn classify_complex_context() {
        let c = classify_complexity(
            "implement a distributed consensus protocol with database migration \
             and real-time streaming over websockets with full security audit",
        );
        assert!(c >= Complexity::Large);
    }

    #[test]
    fn classify_medium_context() {
        let c = classify_complexity(
            "build a REST API endpoint that accepts JSON, validates input, \
             persists to the database, and returns a structured response",
        );
        assert!(c >= Complexity::Small);
        assert!(c <= Complexity::Large);
    }

    #[test]
    fn parse_numbered_list() {
        let text = "Here's what I want:\n\
            1. Build an OAuth2 service with Google and GitHub providers\n\
            2. Add rate limiting middleware with token bucket algorithm\n\
            3. Create a deployment pipeline for Fly.io";
        let features = parse_feature_areas(text);
        assert_eq!(features.len(), 3);
    }

    #[test]
    fn parse_bullet_list() {
        let text = "Features needed:\n\
            - User authentication with JWT tokens and refresh flow\n\
            - Admin dashboard for managing users and permissions\n\
            - Webhook system for event notifications";
        let features = parse_feature_areas(text);
        assert_eq!(features.len(), 3);
    }

    #[test]
    fn parse_boundary_phrases() {
        let text = "I want an OAuth service with Google login. \
            Also add rate limiting to protect the API from abuse. \
            Then, create a webhook system for notifying downstream services.";
        let features = parse_feature_areas(text);
        assert!(features.len() >= 2, "expected >=2, got {}", features.len());
    }

    #[test]
    fn single_feature_fallback() {
        let text = "Build a simple key-value store";
        let features = parse_feature_areas(text);
        assert_eq!(features.len(), 1);
    }

    #[test]
    fn generate_proposal_smoke() {
        let engine = ProposalEngine::new(0.15);
        let draft = make_draft(
            "1. Build an OAuth2 service with Google and GitHub providers\n\
             2. Add rate limiting middleware\n\
             3. Create admin dashboard",
        );

        let proposal = engine.generate_proposal(&draft, 0.20).unwrap();

        assert_eq!(proposal.milestones.len(), 3);
        assert!(proposal.total_cost > 0.0);
        assert!(proposal.net_cost < proposal.total_cost);
        assert_eq!(proposal.draft_cost_spent, 0.30);
        assert_eq!(proposal.status, ProposalStatus::Proposed);
        assert_eq!(proposal.valid_for, "24h");

        // Each milestone should have at least one plan.
        for ms in &proposal.milestones {
            assert!(!ms.plans.is_empty());
            assert!(ms.task_count >= 2);
            assert!(ms.estimated_cost.total > 0.0);
            assert!(!ms.estimated_time.is_empty());
        }

        // Plan names should be NN-kebab-case.
        let first_plan = &proposal.milestones[0].plans[0];
        assert!(
            first_plan.starts_with("01-"),
            "first plan should start with 01-, got: {first_plan}"
        );
    }

    #[test]
    fn re_estimate_updates_version_and_totals() {
        let engine = ProposalEngine::new(0.15);
        let draft = make_draft("Build a REST API with CRUD endpoints and tests");

        let mut proposal = engine.generate_proposal(&draft, 0.20).unwrap();
        let original_total = proposal.total_cost;
        let original_version = proposal.version;

        // Add a milestone.
        proposal.milestones.push(Milestone {
            name: "Extra Feature".to_string(),
            plans: vec!["99-extra".to_string()],
            task_count: 8,
            estimated_cost: CostEstimate {
                inference: 0.0,
                compute: 0.0,
                total: 0.0,
            },
            estimated_time: String::new(),
            actual_cost: None,
            status: MilestoneStatus::Pending,
        });

        engine.re_estimate(&mut proposal, 0.20).unwrap();

        assert_eq!(proposal.version, original_version + 1);
        assert!(
            proposal.total_cost > original_total,
            "total should increase after adding milestone"
        );
    }

    #[test]
    fn retry_buffer_affects_total() {
        let engine_low = ProposalEngine::new(0.10);
        let engine_high = ProposalEngine::new(0.25);
        let draft = make_draft("Build a web service with authentication and database");

        let prop_low = engine_low
            .generate_proposal(&draft, DEFAULT_SPREAD_PCT)
            .unwrap();
        let prop_high = engine_high
            .generate_proposal(&draft, DEFAULT_SPREAD_PCT)
            .unwrap();

        assert!(
            prop_high.total_cost > prop_low.total_cost,
            "higher retry buffer should produce higher total"
        );
    }

    #[test]
    fn plan_names_are_sequential() {
        let engine = ProposalEngine::new(0.15);
        let draft = make_draft(
            "1. Build user authentication\n\
             2. Build payment processing\n\
             3. Build notification system",
        );

        let proposal = engine.generate_proposal(&draft, 0.20).unwrap();
        let all_plans: Vec<&str> = proposal
            .milestones
            .iter()
            .flat_map(|ms| ms.plans.iter().map(String::as_str))
            .collect();

        // All plan names should have unique NN- prefixes.
        let mut seen_prefixes = std::collections::HashSet::new();
        for plan in &all_plans {
            let prefix = &plan[..3]; // "01-", "02-", etc.
            assert!(
                seen_prefixes.insert(prefix.to_string()),
                "duplicate plan prefix: {prefix}"
            );
        }
    }

    #[test]
    fn format_duration_ranges() {
        assert_eq!(format_duration(0.5), "~1 min");
        assert_eq!(format_duration(15.0), "~15 min");
        assert_eq!(format_duration(45.0), "~45 min");
        assert_eq!(format_duration(58.0), "~58 min");
        assert_eq!(format_duration(60.0), "~1 hr");
        assert_eq!(format_duration(90.0), "~1.5 hr");
        assert_eq!(format_duration(120.0), "~2 hr");
    }

    #[test]
    fn empty_context_returns_error_or_minimal() {
        let engine = ProposalEngine::new(0.15);
        let draft = make_draft("");
        // An empty context should still produce a proposal (with a generic milestone).
        let proposal = engine.generate_proposal(&draft, 0.20).unwrap();
        assert!(!proposal.milestones.is_empty());
    }

    #[test]
    fn epic_complexity_produces_many_plans() {
        let engine = ProposalEngine::new(0.15);
        let draft = make_draft(
            "Build a distributed microservice architecture with real-time streaming, \
             database migration, Kubernetes deployment, security audit, authentication \
             with OAuth and SAML, event-driven message queue, consensus protocol, \
             multi-tenant data isolation, and a performance optimization pipeline \
             with caching and parallel processing across all services",
        );

        let proposal = engine.generate_proposal(&draft, 0.20).unwrap();
        let total_plans: usize = proposal.milestones.iter().map(|ms| ms.plans.len()).sum();
        assert!(
            total_plans >= 3,
            "epic work should produce multiple plans, got {total_plans}"
        );
        assert!(
            proposal.total_cost > 5.0,
            "epic proposal should cost >$5, got {}",
            proposal.total_cost
        );
    }
}
