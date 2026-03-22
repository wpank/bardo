use super::dag::PlanDag;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// A task declared in a plan's TOML task file, loaded separately from
/// frontmatter. Stored on `PlanFrontmatter` for convenience so the DAG
/// builder has everything in one place.
#[derive(Debug, Clone, Default)]
pub struct FrontmatterTask {
    pub id: String,
    pub title: String,
    pub files: Vec<String>,
    /// Intra-plan: "T2", cross-plan: "09:T3"
    pub depends_on: Vec<String>,
    pub parallel_group: Option<String>,
    pub estimated_minutes: Option<u32>,
}

/// Parsed YAML frontmatter from a plan file. All fields optional so plans
/// without frontmatter still load fine.
#[derive(Debug, Clone)]
pub struct PlanFrontmatter {
    /// Plan identifier, e.g. "09-chain-layer"
    pub plan: Option<String>,
    /// Plan bases this plan depends on (must complete first)
    pub depends_on: Vec<String>,
    /// Plan bases that can run in parallel with this one
    pub parallel_with: Vec<String>,
    /// Crate directories this plan touches
    pub crates_touched: Vec<String>,
    /// Estimated number of Codex tasks
    pub estimated_tasks: Option<usize>,
    /// Estimated max parallel Codex sessions
    pub estimated_parallel_width: Option<usize>,
    /// Total estimated wall-clock minutes for this plan
    pub estimated_minutes: Option<u32>,
    /// Whether to run a refactor pass after this plan completes
    pub refactor_after: bool,
    /// Whether this plan's tasks are safe to run in parallel with other plans.
    /// Defaults to true.
    pub parallel_safe: bool,
    /// Task breakdown loaded from the companion TOML file (not from frontmatter).
    pub tasks: Vec<FrontmatterTask>,
}

impl Default for PlanFrontmatter {
    fn default() -> Self {
        Self {
            plan: None,
            depends_on: Vec::new(),
            parallel_with: Vec::new(),
            crates_touched: Vec::new(),
            estimated_tasks: None,
            estimated_parallel_width: None,
            estimated_minutes: None,
            refactor_after: false,
            parallel_safe: true,
            tasks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlanInfo {
    /// Full base name, e.g. "01-workspace-scaffold"
    pub base: String,
    /// Numeric/alpha prefix, e.g. "01" or "08a"
    pub num: String,
    /// Full path to the plan .md file
    pub path: PathBuf,
    /// Parsed frontmatter metadata (None if the file has no frontmatter)
    pub frontmatter: Option<PlanFrontmatter>,
}

impl PlanInfo {
    pub fn display_name(&self) -> &str {
        &self.base
    }
}

/// Parse a YAML-style array value like `["a", "b", "c"]` into a Vec<String>.
/// Handles missing brackets, quoted and unquoted items.
fn parse_yaml_array(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    // Strip outer brackets if present
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    if inner.trim().is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(|s| {
            s.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse frontmatter from plan file content. Expects `---` delimiters.
/// Returns None if no frontmatter block is found.
/// G3: Intermediate struct for serde_yaml parsing of frontmatter
#[derive(serde::Deserialize, Default)]
struct YamlFrontmatter {
    plan: Option<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    parallel_with: Vec<String>,
    #[serde(default)]
    crates_touched: Vec<String>,
    estimated_tasks: Option<usize>,
    estimated_parallel_width: Option<usize>,
    estimated_minutes: Option<u32>,
    #[serde(default)]
    refactor_after: bool,
    #[serde(default = "default_parallel_safe")]
    parallel_safe: bool,
}

fn default_parallel_safe() -> bool {
    true
}

pub fn parse_frontmatter(content: &str) -> Option<PlanFrontmatter> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    // Find the closing `---` (skip the opening one)
    let after_open = &trimmed[3..];
    let close_pos = after_open.find("\n---")?;
    let block = &after_open[..close_pos];

    // G3: Try serde_yaml first, fall back to hand-rolled parser
    if let Ok(yf) = serde_yaml::from_str::<YamlFrontmatter>(block) {
        return Some(PlanFrontmatter {
            plan: yf.plan,
            depends_on: yf.depends_on,
            parallel_with: yf.parallel_with,
            crates_touched: yf.crates_touched,
            estimated_tasks: yf.estimated_tasks,
            estimated_parallel_width: yf.estimated_parallel_width,
            estimated_minutes: yf.estimated_minutes,
            refactor_after: yf.refactor_after,
            parallel_safe: yf.parallel_safe,
            tasks: Vec::new(),
        });
    }

    // Fallback: hand-rolled parser
    let mut fm = PlanFrontmatter::default();

    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Split on first ':'
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "plan" => {
                    fm.plan = Some(value.trim_matches('"').trim_matches('\'').to_string());
                }
                "depends_on" => {
                    fm.depends_on = parse_yaml_array(value);
                }
                "parallel_with" => {
                    fm.parallel_with = parse_yaml_array(value);
                }
                "crates_touched" => {
                    fm.crates_touched = parse_yaml_array(value);
                }
                "estimated_tasks" => {
                    fm.estimated_tasks = value.parse().ok();
                }
                "estimated_parallel_width" => {
                    fm.estimated_parallel_width = value.parse().ok();
                }
                "estimated_minutes" => {
                    fm.estimated_minutes = value.parse().ok();
                }
                "refactor_after" => {
                    fm.refactor_after = matches!(value, "true" | "yes" | "1");
                }
                "parallel_safe" => {
                    fm.parallel_safe = !matches!(value, "false" | "no" | "0");
                }
                _ => {} // ignore unknown keys
            }
        }
    }

    Some(fm)
}

/// Return the plan body (everything after the frontmatter block).
/// If there is no frontmatter, returns the full content.
pub fn read_plan_body(plan: &PlanInfo) -> Result<String> {
    let content = std::fs::read_to_string(&plan.path)?;
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Ok(content);
    }
    let after_open = &trimmed[3..];
    if let Some(close_pos) = after_open.find("\n---") {
        // Skip past the closing `---` and the newline after it
        let rest = &after_open[close_pos + 4..]; // +4 = \n---
                                                 // Skip an optional newline right after the closing ---
        let rest = rest.strip_prefix('\n').unwrap_or(rest);
        Ok(rest.to_string())
    } else {
        // Malformed frontmatter — return everything
        Ok(content)
    }
}

/// Extract the numeric/alpha prefix from a plan base name
/// "01-workspace-scaffold" -> "01"
/// "08a-whatever" -> "08a"
fn plan_num(base: &str) -> String {
    // Take chars until we hit '-' (the separator between num and name)
    base.split('-').next().unwrap_or(base).to_string()
}

/// List all plan bases from the plans directory, sorted.
///
/// Detects both layouts:
/// - New: directories containing `plan.md` (e.g. `plans/01-workspace-scaffold/plan.md`)
/// - Legacy: flat `.md` files (e.g. `plans/01-workspace-scaffold.md`)
///
/// If both exist for the same base name, the directory version wins.
fn all_plan_bases(plans_dir: &Path) -> Result<Vec<String>> {
    let mut bases = Vec::new();
    if !plans_dir.exists() {
        bail!("Plans directory not found: {}", plans_dir.display());
    }
    for entry in std::fs::read_dir(plans_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // New layout: directory with plan.md inside
        if entry.path().is_dir()
            && name.starts_with(|c: char| c.is_ascii_alphanumeric())
            && entry.path().join("plan.md").exists()
        {
            bases.push(name);
            continue;
        }

        // Legacy: flat .md files
        if name.starts_with(|c: char| c.is_ascii_alphanumeric())
            && name.ends_with(".md")
            && name != "CONTEXT.md"
        {
            let base = name.trim_end_matches(".md").to_string();
            // Don't add if we already found a directory version
            if !bases.contains(&base) {
                bases.push(base);
            }
        }
    }
    bases.sort();
    // Deduplicate (directory version wins over flat file)
    bases.dedup();
    Ok(bases)
}

/// Check if a plan num is in range [lo, hi] using string comparison
fn in_range(num: &str, lo: &str, hi: &str) -> bool {
    num >= lo && num <= hi
}

/// Expand a plan spec into a list of plan base names
/// Specs can be: "01" (single), "01-09" (range), "08a-08d" (letter range)
fn expand_spec(plans_dir: &Path, spec: &str) -> Result<Vec<String>> {
    // Check if it's a range (contains '-' but not at position 0 and the part before '-' is numeric-ish)
    if let Some((lo, hi)) = parse_range(spec) {
        let all = all_plan_bases(plans_dir)?;
        let matched: Vec<String> = all
            .into_iter()
            .filter(|base| {
                let num = plan_num(base);
                in_range(&num, &lo, &hi)
            })
            .collect();
        Ok(matched)
    } else {
        // Single plan spec — find matching base
        let all = all_plan_bases(plans_dir)?;
        let matched: Vec<String> = all
            .into_iter()
            .filter(|base| {
                let num = plan_num(base);
                num == spec || base == spec
            })
            .collect();
        if matched.is_empty() {
            bail!("No plan found matching spec: {spec}");
        }
        Ok(matched)
    }
}

/// Try to parse a range spec like "01-09" or "08a-08d"
/// Returns None if it's not a range
fn parse_range(spec: &str) -> Option<(String, String)> {
    // A range has the form NUM-NUM where NUM is digits optionally followed by a letter
    // We need to distinguish "01-09" (range) from "01-workspace-scaffold" (plan name)
    // Heuristic: if both sides of the first '-' look like plan numbers (digits + optional letter), it's a range
    let parts: Vec<&str> = spec.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }
    let lo = parts[0];
    let hi = parts[1];

    // Both lo and hi must be short alphanumeric strings (plan numbers)
    let is_plan_num =
        |s: &str| !s.is_empty() && s.len() <= 4 && s.chars().all(|c| c.is_ascii_alphanumeric());

    if is_plan_num(lo) && is_plan_num(hi) {
        Some((lo.to_string(), hi.to_string()))
    } else {
        None
    }
}

/// Discover plans from a list of specs
pub fn discover_plans(plans_dir: &Path, specs: &[String]) -> Result<Vec<PlanInfo>> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for spec in specs {
        let bases = expand_spec(plans_dir, spec)?;
        for base in bases {
            if seen.contains(&base) {
                continue;
            }
            seen.insert(base.clone());
            let path = crate::orchestrator::paths::plan_doc(plans_dir, &base);
            if !path.exists() {
                bail!("Plan file not found: {}", path.display());
            }
            let frontmatter = std::fs::read_to_string(&path)
                .ok()
                .and_then(|content| parse_frontmatter(&content));
            result.push(PlanInfo {
                num: plan_num(&base),
                base,
                path,
                frontmatter,
            });
        }
    }

    // Sort by DAG execution order (wave index, then alphabetically within each wave).
    // Falls back to alphabetical if the DAG can't be built (e.g. a cycle).
    if let Ok(dag) = PlanDag::from_plans(&result) {
        let mut order: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();
        for wave in dag.compute_waves() {
            for (i, plan) in wave.plans.iter().enumerate() {
                order.insert(plan.clone(), (wave.index, i));
            }
        }
        result.sort_by_key(|p| {
            order
                .get(&p.base)
                .copied()
                .unwrap_or((usize::MAX, usize::MAX))
        });
    } else {
        result.sort_by(|a, b| a.base.cmp(&b.base));
    }
    Ok(result)
}

/// Read the content of a plan file
pub fn read_plan(plan: &PlanInfo) -> Result<String> {
    Ok(std::fs::read_to_string(&plan.path)?)
}
