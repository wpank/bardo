use std::path::Path;

use super::tasks::Task;
use crate::agent::AgentRole;

pub const HUMANIZER: &str = "humanizer";
pub const RATATUI_CINEMATIC: &str = "ratatui-cinematic";

/// Search paths: project `.claude/skills/{name}/SKILL.md` first,
/// then `~/.claude/skills/{name}/SKILL.md`.
pub fn load_skill(repo_root: &Path, name: &str) -> Option<String> {
    // Project-level first
    let project_path = repo_root.join(format!(".claude/skills/{name}/SKILL.md"));
    if let Ok(content) = std::fs::read_to_string(&project_path) {
        return Some(content);
    }

    // User-level fallback
    if let Some(home) = dirs::home_dir() {
        let user_path = home.join(format!(".claude/skills/{name}/SKILL.md"));
        if let Ok(content) = std::fs::read_to_string(&user_path) {
            return Some(content);
        }
    }

    None
}

/// Load multiple skills, skip missing ones.
pub fn load_skills(repo_root: &Path, names: &[String]) -> Vec<(String, String)> {
    names
        .iter()
        .filter_map(|name| load_skill(repo_root, name).map(|content| (name.clone(), content)))
        .collect()
}

/// Role-level defaults:
///   Scribe, Critic, DocVerifier -> [humanizer]
///   All others -> []
pub fn default_skills_for_role(role: AgentRole) -> Vec<String> {
    match role {
        AgentRole::Scribe | AgentRole::Critic | AgentRole::DocVerifier => {
            vec![HUMANIZER.to_string()]
        }
        _ => vec![],
    }
}

/// Merge role defaults + task.skills + auto-detection into deduplicated list.
/// Auto-detect: if role is Implementer or TerminalValidator and any task file
/// contains "bardo-terminal" or "terminal", add ratatui-cinematic.
pub fn skills_for_task(role: AgentRole, task: &Task) -> Vec<String> {
    let mut skills = default_skills_for_role(role.clone());

    // Add task-level skills
    if let Some(ref task_skills) = task.skills {
        for s in task_skills {
            if !skills.contains(s) {
                skills.push(s.clone());
            }
        }
    }

    // Auto-detect terminal work
    if matches!(role, AgentRole::Implementer | AgentRole::TerminalValidator)
        && is_terminal_task(&task.files)
    {
        let cinematic = RATATUI_CINEMATIC.to_string();
        if !skills.contains(&cinematic) {
            skills.push(cinematic);
        }
    }

    skills
}

/// Check if task files suggest terminal/visual work.
fn is_terminal_task(files: &[String]) -> bool {
    files
        .iter()
        .any(|f| f.contains("bardo-terminal") || f.contains("terminal"))
}

/// Build the `<skill>` XML section for prompt injection.
/// Splits budget evenly across loaded skills, truncates each.
/// Returns empty string when no skills resolve.
pub fn build_skill_section(repo_root: &Path, skills: &[String], budget_chars: usize) -> String {
    let loaded = load_skills(repo_root, skills);
    if loaded.is_empty() {
        return String::new();
    }

    let per_skill = budget_chars / loaded.len();
    let mut section = String::new();

    for (name, content) in &loaded {
        let trimmed = super::prompts::truncate(content, per_skill);
        section.push_str(&format!("\n<skill name=\"{name}\">\n{trimmed}\n</skill>\n"));
    }

    section
}
