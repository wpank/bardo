use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Preflight,
    Implementer,
    CompileGate,
    TestGate,
    Reviewing,    // Architect + Auditor + Scribe parallel
    CriticReview, // Critic reviews docs
    Verdict,
    Committing,
}

impl Phase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::Implementer => "implementer",
            Self::CompileGate => "compile-gate",
            Self::TestGate => "test-gate",
            Self::Reviewing => "reviewing",
            Self::CriticReview => "critic-review",
            Self::Verdict => "verdict",
            Self::Committing => "committing",
        }
    }
}

/// Verdict from a review
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Approve,
    Revise { issues: Vec<String> },
}

/// Extract verdict from a review markdown file.
/// Tries structured TOML parsing first, falls back to regex-based extraction.
/// C3: Scans only the first 50 lines for fallback. Returns Revise (not Missing) when no verdict found.
pub fn extract_verdict(review_content: &str) -> Verdict {
    // Try structured TOML parsing first (from ```toml block)
    if let Some(structured) = super::review::parse_structured_review(review_content) {
        return match structured.verdict.overall {
            super::review::VerdictStatus::Approve => Verdict::Approve,
            super::review::VerdictStatus::Revise => {
                let issues = structured
                    .issues
                    .iter()
                    .filter(|i| i.severity == super::review::IssueSeverity::Blocking)
                    .map(|i| format!("- [{}] {}: {}", i.id, i.file, i.description))
                    .collect();
                Verdict::Revise { issues }
            }
            super::review::VerdictStatus::Skip => Verdict::Approve,
        };
    }

    // Fallback: regex-based extraction from markdown
    let scan_lines: Vec<&str> = review_content.lines().take(50).collect();

    // Primary pass: look for `## Verdict:` heading format (properly formatted files)
    for line in &scan_lines {
        let trimmed = line.trim().to_lowercase();
        if trimmed.contains("## verdict:") || trimmed.contains("##verdict:") {
            if trimmed.contains("approve") {
                return Verdict::Approve;
            }
            if trimmed.contains("revise") {
                return Verdict::Revise {
                    issues: collect_blocking_issues(review_content),
                };
            }
        }
    }

    // Fallback pass: scan for `verdict:` or `outcome:` anywhere in a line.
    // Catches `**Outcome:** **REVISE**`, `Verdict: APPROVE`, etc.
    for line in &scan_lines {
        let lower = line.trim().to_lowercase();
        // Strip markdown bold markers for matching
        let stripped = lower.replace("**", "").replace('*', "");
        if stripped.contains("verdict:") || stripped.contains("outcome:") {
            if stripped.contains("approve") || stripped.contains("complete") {
                return Verdict::Approve;
            }
            if stripped.contains("revise") || stripped.contains("revision") {
                return Verdict::Revise {
                    issues: collect_blocking_issues(review_content),
                };
            }
        }
    }

    // C3: No verdict found — return Revise instead of blocking forever
    Verdict::Revise {
        issues: vec!["No verdict found in first 50 lines of review".to_string()],
    }
}

/// Read verdict from context/out/review.json. Hard error if absent/invalid.
/// No regex fallback -- if the file is missing, that's an error (not a silent approve).
pub fn extract_verdict_from_file(out_dir: &std::path::Path) -> anyhow::Result<Verdict> {
    use super::schema::{IssueSeverity, ReviewVerdict};
    let path = out_dir.join("review.json");
    if !path.exists() {
        anyhow::bail!(
            "review.json not written by agent at {} -- treating as retry-needed",
            out_dir.display()
        );
    }
    let json = std::fs::read_to_string(&path)?;
    let review: super::schema::ReviewReport = serde_json::from_str(&json)
        .with_context(|| format!("review.json invalid JSON at {}", out_dir.display()))?;
    Ok(match review.verdict {
        ReviewVerdict::Approve => Verdict::Approve,
        ReviewVerdict::Revise => {
            let issues = review
                .issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Blocking))
                .map(|i| format!("[{}] {:?}: {}", i.id, i.category, i.description))
                .collect();
            Verdict::Revise { issues }
        }
    })
}

/// C4: Collect blocking issues from review content.
/// Matches: `- [B-`, `- **[B-`, `* [B-`, `* **[B-`, indented variants, numbered items.
fn collect_blocking_issues(review_content: &str) -> Vec<String> {
    review_content
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("- [B-")
                || t.starts_with("- **[B-")
                || t.starts_with("* [B-")
                || t.starts_with("* **[B-")
                || (t.starts_with(|c: char| c.is_ascii_digit()) && t.contains("[B-"))
        })
        .map(|l| l.trim().to_string())
        .collect()
}
