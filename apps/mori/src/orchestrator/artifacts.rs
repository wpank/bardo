use super::schema::{CompletionReport, IssueSeverity, ReviewReport};
use crate::agent::AgentRole;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ArtifactStore {
    root: PathBuf,
}

impl ArtifactStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn artifact_dir(&self, plan: &str, iter: u32) -> PathBuf {
        self.root.join(plan).join(format!("iter-{iter}"))
    }

    pub fn write_completion(&self, plan: &str, iter: u32, report: &CompletionReport) -> Result<()> {
        let dir = self.artifact_dir(plan, iter);
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(report)?;
        std::fs::write(dir.join("completion.json"), json)?;
        Ok(())
    }

    pub fn read_completion(&self, plan: &str, iter: u32) -> Result<Option<CompletionReport>> {
        let path = self.artifact_dir(plan, iter).join("completion.json");
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&json)?))
    }

    pub fn write_review(
        &self,
        plan: &str,
        iter: u32,
        role: AgentRole,
        review: &ReviewReport,
    ) -> Result<()> {
        let dir = self.artifact_dir(plan, iter);
        std::fs::create_dir_all(&dir)?;
        let filename = format!("{}-review.json", role.label());
        let json = serde_json::to_string_pretty(review)?;
        std::fs::write(dir.join(filename), json)?;
        Ok(())
    }

    pub fn write_review_narrative(
        &self,
        plan: &str,
        iter: u32,
        role: AgentRole,
        md: &str,
    ) -> Result<()> {
        let dir = self.artifact_dir(plan, iter);
        std::fs::create_dir_all(&dir)?;
        let filename = format!("{}-review.md", role.label());
        std::fs::write(dir.join(filename), md)?;
        Ok(())
    }

    pub fn read_reviews(&self, plan: &str, iter: u32) -> Result<Vec<ReviewReport>> {
        let dir = self.artifact_dir(plan, iter);
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut reviews = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("-review.json") {
                let json = std::fs::read_to_string(entry.path())?;
                if let Ok(r) = serde_json::from_str::<ReviewReport>(&json) {
                    reviews.push(r);
                }
            }
        }
        Ok(reviews)
    }

    /// Compile previous iteration reviews into a prompt-ready string.
    pub fn prev_iter_summary(&self, plan: &str, iter: u32) -> Result<String> {
        if iter == 0 {
            return Ok(String::new());
        }
        let reviews = self.read_reviews(plan, iter)?;
        if reviews.is_empty() {
            return Ok(String::new());
        }
        let mut lines = vec![format!("# Previous Iteration ({iter}) Reviews\n")];
        for r in &reviews {
            lines.push(format!("## {} — {:?}\n", r.reviewer, r.verdict));
            lines.push(format!("{}\n", r.summary));
            let blocking: Vec<&_> = r
                .issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Blocking))
                .collect();
            if !blocking.is_empty() {
                lines.push("### Blocking Issues\n".to_string());
                for issue in blocking {
                    let loc = issue
                        .file
                        .as_deref()
                        .map(|f| {
                            if let Some(ln) = issue.line {
                                format!(" ({f}:{ln})")
                            } else {
                                format!(" ({f})")
                            }
                        })
                        .unwrap_or_default();
                    lines.push(format!(
                        "- [{}]{loc}: {}\n  Fix: {}\n",
                        issue.id, issue.description, issue.fix_hint
                    ));
                }
            }
        }
        Ok(lines.join("\n"))
    }
}
