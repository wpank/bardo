use super::schema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Structured review output — parsed from TOML fenced blocks in review files
// ---------------------------------------------------------------------------

/// A structured review parsed from a ```toml block in a reviewer's markdown output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredReview {
    pub verdict: ReviewVerdict,
    #[serde(default)]
    pub issues: Vec<ReviewIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewVerdict {
    #[serde(default = "default_verdict_skip")]
    pub code: VerdictStatus,
    #[serde(default = "default_verdict_skip")]
    pub docs: VerdictStatus,
    pub overall: VerdictStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerdictStatus {
    Approve,
    Revise,
    Skip,
}

fn default_verdict_skip() -> VerdictStatus {
    VerdictStatus::Skip
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub file: String,
    #[serde(default)]
    pub line: Option<u32>,
    pub description: String,
    #[serde(default)]
    pub fix_hint: String,
    #[serde(default)]
    pub addressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Blocking,
    Major,
    Minor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    Compilation,
    Test,
    TypeMismatch,
    MissingImpl,
    Docs,
    Style,
    SpecDeviation,
}

impl IssueCategory {
    /// Returns true if this issue can be handled by the quick-fix path
    /// (no strategist re-run needed).
    pub fn is_quick_fixable(&self) -> bool {
        matches!(
            self,
            IssueCategory::Compilation | IssueCategory::Docs | IssueCategory::Style
        )
    }
}

impl StructuredReview {
    /// True when all blocking issues are quick-fixable (compilation, docs, style).
    pub fn all_issues_quick_fixable(&self) -> bool {
        let blocking: Vec<&ReviewIssue> = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Blocking && !i.addressed)
            .collect();
        !blocking.is_empty() && blocking.iter().all(|i| i.category.is_quick_fixable())
    }

    /// Collect unresolved blocking issues.
    pub fn unresolved_blocking(&self) -> Vec<&ReviewIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Blocking && !i.addressed)
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Parsing: extract StructuredReview from review markdown
// ---------------------------------------------------------------------------

/// Try to extract a StructuredReview from a review markdown file.
/// Looks for a ```toml fenced block and parses it. Returns None if
/// no valid TOML block is found.
pub fn parse_structured_review(review_content: &str) -> Option<StructuredReview> {
    // Find ```toml block
    let toml_start = review_content.find("```toml")?;
    let after_fence = &review_content[toml_start + 7..];
    let toml_end = after_fence.find("```")?;
    let toml_block = after_fence[..toml_end].trim();

    toml::from_str::<StructuredReview>(toml_block).ok()
}

/// Parse a ReviewReport from `{out_dir}/review.json`.
/// Returns `None` if the file doesn't exist; `Err` if present but invalid JSON.
pub fn parse_review_json(out_dir: &std::path::Path) -> Option<schema::ReviewReport> {
    let path = out_dir.join("review.json");
    if !path.exists() {
        return None;
    }
    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

/// TOML template that gets injected into reviewer prompts so they know
/// the expected output format.
pub const REVIEW_TOML_TEMPLATE: &str = r#"
After your written review, include a structured summary in a ```toml block:

```toml
[verdict]
code = "approve"      # "approve" or "revise"
docs = "skip"         # "approve", "revise", or "skip"
overall = "approve"   # "approve" or "revise"

[[issues]]
id = "B-1"
severity = "blocking"          # "blocking", "major", or "minor"
category = "compilation"       # "compilation", "test", "type_mismatch", "missing_impl", "docs", "style", "spec_deviation"
file = "crates/foo/src/lib.rs"
line = 42
description = "Missing import for BarTrait"
fix_hint = "Add `use crate::bar::BarTrait;`"
addressed = false
```

Rules:
- Always include the ```toml block even for APPROVE (with an empty issues list)
- Every blocking issue from your review MUST appear as an [[issues]] entry
- Use severity="blocking" only for functional bugs that prevent the code from working
- The `id` field should match your review text (B-1, B-2, etc.)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_approve_review() {
        let content = r#"## Review

Everything looks good.

```toml
[verdict]
code = "approve"
docs = "skip"
overall = "approve"
```

## Verdict: APPROVE
"#;
        let review = parse_structured_review(content).unwrap();
        assert_eq!(review.verdict.code, VerdictStatus::Approve);
        assert_eq!(review.verdict.overall, VerdictStatus::Approve);
        assert!(review.issues.is_empty());
    }

    #[test]
    fn parse_revise_review_with_issues() {
        let content = r#"## Review

Found problems.

```toml
[verdict]
code = "revise"
docs = "skip"
overall = "revise"

[[issues]]
id = "B-1"
severity = "blocking"
category = "compilation"
file = "crates/foo/src/lib.rs"
line = 42
description = "Missing import"
fix_hint = "Add use statement"
addressed = false

[[issues]]
id = "B-2"
severity = "blocking"
category = "docs"
file = "docs/src/foo.md"
description = "Missing doc page"
fix_hint = "Create the page"
addressed = false
```

## Verdict: REVISE
"#;
        let review = parse_structured_review(content).unwrap();
        assert_eq!(review.verdict.code, VerdictStatus::Revise);
        assert_eq!(review.issues.len(), 2);
        assert!(review.all_issues_quick_fixable());
    }

    #[test]
    fn non_quick_fixable_issues() {
        let content = r#"```toml
[verdict]
code = "revise"
overall = "revise"

[[issues]]
id = "B-1"
severity = "blocking"
category = "missing_impl"
file = "crates/foo/src/lib.rs"
description = "State machine not implemented"
fix_hint = "Implement the FSM"
addressed = false
```"#;
        let review = parse_structured_review(content).unwrap();
        assert!(!review.all_issues_quick_fixable());
    }

    #[test]
    fn parse_quick_reviewer_toml_overall_only() {
        // Quick reviewer only emits `overall`; `code` and `docs` should default to Skip.
        let content = r#"## Review

No blocking issues found.

```toml
[verdict]
overall = "approve"
code = "approve"
docs = "skip"
```
"#;
        let review = parse_structured_review(content).unwrap();
        assert_eq!(review.verdict.overall, VerdictStatus::Approve);
        assert!(review.issues.is_empty());
    }

    #[test]
    fn parse_quick_reviewer_toml_overall_only_no_code_field() {
        // Older quick-reviewer output with only `overall` — must not fail deserialization.
        let content = r#"```toml
[verdict]
overall = "approve"
```"#;
        let review = parse_structured_review(content).unwrap();
        assert_eq!(review.verdict.overall, VerdictStatus::Approve);
        assert_eq!(review.verdict.code, VerdictStatus::Skip);
        assert_eq!(review.verdict.docs, VerdictStatus::Skip);
    }

    #[test]
    fn no_toml_block_returns_none() {
        let content = "## Review\n\nLooks fine.\n\n## Verdict: APPROVE\n";
        assert!(parse_structured_review(content).is_none());
    }

    #[test]
    fn docs_only_revise() {
        let content = r#"```toml
[verdict]
code = "approve"
docs = "revise"
overall = "revise"

[[issues]]
id = "B-1"
severity = "blocking"
category = "docs"
file = "docs/src/foo.md"
description = "Missing doc page"
fix_hint = "Create the page"
addressed = false
```"#;
        let review = parse_structured_review(content).unwrap();
        assert_eq!(review.verdict.code, VerdictStatus::Approve);
        assert_eq!(review.verdict.docs, VerdictStatus::Revise);
    }
}
