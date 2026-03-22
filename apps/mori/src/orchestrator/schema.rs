use crate::agent::AgentRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRecord {
    pub plan: String,
    pub name: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRecord {
    pub plan: String,
    pub name: String,
    pub path: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompileStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCounts {
    pub pass: u32,
    pub fail: u32,
    pub ignored: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionReport {
    pub plan: String,
    pub iteration: u32,
    pub completed_tasks: Vec<String>,
    pub failed_tasks: Vec<String>,
    pub types_defined: Vec<TypeRecord>,
    pub compile_status: CompileStatus,
    pub test_counts: TestCounts,
    pub deviations: Vec<String>,
    pub issues_for_next: Vec<String>,
    pub notes_for_reviewers: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReviewVerdict {
    Approve,
    Revise,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub description: String,
    pub fix_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub reviewer: AgentRole,
    pub plan: String,
    pub iteration: u32,
    pub verdict: ReviewVerdict,
    pub compile_verified: bool,
    pub test_counts: Option<TestCounts>,
    pub issues: Vec<ReviewIssue>,
    pub summary: String,
}
