use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Active,
    Done,
    Blocked,
}

impl TaskStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Pending => "·",
            Self::Active => "►",
            Self::Done => "✓",
            Self::Blocked => "✗",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub acceptance: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Tasks sharing a parallel_group value can run simultaneously.
    #[serde(default)]
    pub parallel_group: Option<String>,
    /// When true (the default), no other task should touch this task's files.
    #[serde(default = "default_exclusive_files")]
    pub exclusive_files: bool,
    /// Estimated minutes for this task (Codex agent time).
    #[serde(default)]
    pub estimated_minutes: Option<u32>,

    // ── Enhanced context fields ──
    /// Type signatures this task must define (from plan Quick Reference).
    #[serde(default)]
    pub types_to_define: Option<Vec<String>>,
    /// Formulas to implement verbatim (from PRD2).
    #[serde(default)]
    pub formulas: Option<Vec<String>>,
    /// Invariant IDs this task must test (from ## Verification).
    #[serde(default)]
    pub test_invariants: Option<Vec<String>>,
    /// Imports needed from other crates/modules.
    #[serde(default)]
    pub imports: Option<Vec<String>>,
    /// Path to similar existing code to follow as pattern.
    #[serde(default)]
    pub example_pattern: Option<String>,
    /// Context files to read before implementing (injected into prompt).
    #[serde(default)]
    pub context_files: Option<Vec<String>>,
    /// Specific section of plan to focus on.
    #[serde(default)]
    pub plan_section: Option<String>,
    /// Skills to inject into prompts for this task (additive to role defaults).
    #[serde(default)]
    pub skills: Option<Vec<String>>,
}

fn default_exclusive_files() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMeta {
    pub plan: String,
    #[serde(default = "default_iteration")]
    pub iteration: u32,
    #[serde(default)]
    pub total: usize,
    #[serde(default)]
    pub done: usize,
    /// Cap on how many tasks can execute at once across the whole plan.
    #[serde(default)]
    pub max_parallel: Option<usize>,
    /// Total estimated minutes for all tasks in this plan.
    #[serde(default)]
    pub estimated_total_minutes: Option<u32>,
}

fn default_iteration() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFile {
    pub meta: TaskMeta,
    #[serde(rename = "task")]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub struct TaskChecklist {
    pub plan_num: String,
    pub iteration: u32,
    pub tasks: Vec<Task>,
}

impl TaskChecklist {
    pub fn total(&self) -> usize {
        self.tasks.len()
    }

    pub fn done_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count()
    }

    pub fn active_task(&self) -> Option<&Task> {
        self.tasks.iter().find(|t| t.status == TaskStatus::Active)
    }

    /// Estimated remaining minutes for non-done tasks, accounting for parallelism.
    ///
    /// Tasks in the same parallel group run concurrently, so we take the MAX
    /// estimate within each group. Groups are then summed sequentially (a
    /// simplification; the full DAG-based estimate lives on `TaskDag`).
    pub fn estimated_remaining_minutes(&self) -> u32 {
        let pending: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Done)
            .collect();
        estimate_parallel_minutes(&pending)
    }

    /// Estimated total minutes for all tasks, accounting for parallelism.
    pub fn estimated_total_minutes(&self) -> u32 {
        let all: Vec<&Task> = self.tasks.iter().collect();
        estimate_parallel_minutes(&all)
    }
}

/// How tasks within a plan should be executed.
#[derive(Debug, Clone)]
pub enum TaskExecutionMode {
    /// Execute tasks one at a time.
    Sequential,
    /// Execute independent tasks in parallel up to a limit.
    ParallelTasks { max_parallel: usize },
}

/// Load task checklist from plans/context/tasks/{num}-tasks.toml
pub fn load_checklist(repo_root: &Path, plan_num: &str) -> Result<Option<TaskChecklist>> {
    load_role_checklist(repo_root, plan_num, "tasks")
}

/// Load a role-specific task checklist.
///
/// Suffix determines which TOML is loaded:
///   "tasks"        → {num}-tasks.toml        (implementer)
///   "review-tasks" → {num}-review-tasks.toml  (architect + auditor)
///   "verify-tasks" → {num}-verify-tasks.toml  (verifier)
///   "scribe-tasks" → {num}-scribe-tasks.toml  (scribe)
pub fn load_role_checklist(
    repo_root: &Path,
    plan_num: &str,
    suffix: &str,
) -> Result<Option<TaskChecklist>> {
    let path = repo_root.join(format!("plans/context/tasks/{plan_num}-{suffix}.toml"));
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path)?;
    // Strip markdown code fences if present (generated by LLM pipelines)
    let content = if raw.starts_with("```") {
        let stripped = raw
            .strip_prefix("```toml\n")
            .or_else(|| raw.strip_prefix("```toml\r\n"))
            .or_else(|| raw.strip_prefix("```\n"))
            .unwrap_or(&raw);
        stripped
            .strip_suffix("\n```\n")
            .or_else(|| stripped.strip_suffix("\n```"))
            .or_else(|| stripped.strip_suffix("\r\n```"))
            .unwrap_or(stripped)
            .to_string()
    } else {
        raw
    };
    let task_file: TaskFile = toml::from_str(&content)?;

    Ok(Some(TaskChecklist {
        plan_num: plan_num.to_string(),
        iteration: task_file.meta.iteration,
        tasks: task_file.tasks,
    }))
}

// ---------------------------------------------------------------------------
// Parallel execution support
// ---------------------------------------------------------------------------

/// A group of tasks that can run simultaneously.
#[derive(Debug, Clone)]
pub struct TaskGroup {
    pub group_id: String,
    pub task_ids: Vec<String>,
    /// Max of constituent task estimates (since they run in parallel).
    pub estimated_minutes: u32,
}

/// Directed-acyclic-graph view of tasks, ordered by dependency and grouped
/// by `parallel_group`.
pub struct TaskDag {
    groups: Vec<TaskGroup>,
    tasks: Vec<Task>,
}

impl TaskDag {
    /// Build a `TaskDag` from a slice of tasks.
    ///
    /// Tasks that share a `parallel_group` value are collected into one
    /// `TaskGroup`. Tasks without a group become a singleton group keyed by
    /// their task id (prefixed with `__solo__` to avoid collisions with
    /// real group names).
    ///
    /// Groups are topologically sorted: a group appears after every group
    /// that contains a task it depends on.
    pub fn from_tasks(tasks: &[Task]) -> Self {
        // 1. Bucket tasks by group key.
        let mut group_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut task_to_group: HashMap<String, String> = HashMap::new();

        for task in tasks {
            let group_key = match &task.parallel_group {
                Some(g) => g.clone(),
                None => format!("__solo__{}", task.id),
            };
            group_map
                .entry(group_key.clone())
                .or_default()
                .push(task.id.clone());
            task_to_group.insert(task.id.clone(), group_key);
        }

        // 2. Build group-level dependency edges.
        //    group A depends on group B if any task in A depends on a task in B.
        let task_index: HashMap<String, &Task> = tasks.iter().map(|t| (t.id.clone(), t)).collect();

        let mut group_deps: HashMap<String, HashSet<String>> = HashMap::new();
        for task in tasks {
            let my_group = &task_to_group[&task.id];
            for dep_id in &task.depends_on {
                if let Some(dep_group) = task_to_group.get(dep_id) {
                    if dep_group != my_group {
                        group_deps
                            .entry(my_group.clone())
                            .or_default()
                            .insert(dep_group.clone());
                    }
                }
            }
        }

        // 3. Topological sort (Kahn's algorithm).
        let all_groups: Vec<String> = group_map.keys().cloned().collect();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for g in &all_groups {
            in_degree.entry(g.clone()).or_insert(0);
            if let Some(deps) = group_deps.get(g) {
                // in_degree is number of groups this group depends on
                *in_degree.get_mut(g).unwrap() = deps.len();
            }
        }

        // Build reverse adjacency: for each group, which groups depend on it?
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
        for (g, deps) in &group_deps {
            for dep in deps {
                dependents.entry(dep.clone()).or_default().push(g.clone());
            }
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(g, _)| g.clone())
            .collect();
        // Stable order: sort the initial queue so output is deterministic.
        queue.sort();

        let mut sorted_groups: Vec<TaskGroup> = Vec::new();
        while let Some(g) = queue.pop() {
            let task_ids = group_map.remove(&g).unwrap_or_default();
            let estimated_minutes = task_ids
                .iter()
                .filter_map(|id| {
                    task_index
                        .get(id.as_str())
                        .and_then(|t| t.estimated_minutes)
                })
                .max()
                .unwrap_or(0);
            sorted_groups.push(TaskGroup {
                group_id: g.clone(),
                task_ids,
                estimated_minutes,
            });
            if let Some(deps_of_g) = dependents.get(&g) {
                for dependent in deps_of_g {
                    if let Some(d) = in_degree.get_mut(dependent) {
                        *d = d.saturating_sub(1);
                        if *d == 0 {
                            queue.push(dependent.clone());
                            queue.sort();
                        }
                    }
                }
            }
        }

        // Any groups not yet emitted have a cycle; append them anyway so we
        // don't silently drop tasks.
        for (g, task_ids) in group_map {
            let estimated_minutes = task_ids
                .iter()
                .filter_map(|id| {
                    task_index
                        .get(id.as_str())
                        .and_then(|t| t.estimated_minutes)
                })
                .max()
                .unwrap_or(0);
            sorted_groups.push(TaskGroup {
                group_id: g,
                task_ids,
                estimated_minutes,
            });
        }

        TaskDag {
            groups: sorted_groups,
            tasks: tasks.to_vec(),
        }
    }

    /// Return every task whose dependencies are fully satisfied.
    ///
    /// A task is runnable when every id in its `depends_on` appears in
    /// `completed` and the task itself is not in `completed`.
    pub fn next_runnable(&self, completed: &HashSet<String>) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| {
                !completed.contains(&t.id) && t.depends_on.iter().all(|dep| completed.contains(dep))
            })
            .collect()
    }

    /// True when every task id appears in `completed`.
    pub fn all_complete(&self, completed: &HashSet<String>) -> bool {
        self.tasks.iter().all(|t| completed.contains(&t.id))
    }

    /// Theoretical maximum width: the size of the largest parallel group.
    pub fn max_parallel(&self) -> usize {
        self.groups
            .iter()
            .map(|g| g.task_ids.len())
            .max()
            .unwrap_or(1)
    }

    /// Borrow the ordered groups.
    pub fn groups(&self) -> &[TaskGroup] {
        &self.groups
    }

    /// Borrow all tasks.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Estimated remaining minutes given a set of completed task ids.
    ///
    /// For each group that still has incomplete tasks, we take the MAX
    /// estimate among its incomplete tasks (they run in parallel). Groups
    /// run sequentially relative to their dependency order, so we SUM those
    /// group-level maxima.
    pub fn estimated_remaining(&self, completed: &HashSet<String>) -> u32 {
        let task_index: HashMap<&str, &Task> =
            self.tasks.iter().map(|t| (t.id.as_str(), t)).collect();

        self.groups
            .iter()
            .map(|g| {
                g.task_ids
                    .iter()
                    .filter(|id| !completed.contains(id.as_str()))
                    .filter_map(|id| {
                        task_index
                            .get(id.as_str())
                            .and_then(|t| t.estimated_minutes)
                    })
                    .max()
                    .unwrap_or(0)
            })
            .sum()
    }
}

/// Estimate total minutes for a set of tasks, accounting for parallelism.
///
/// Tasks in the same `parallel_group` run concurrently (take the MAX).
/// Distinct groups run sequentially (SUM their maxima). Tasks without a
/// parallel_group each form their own singleton group.
fn estimate_parallel_minutes(tasks: &[&Task]) -> u32 {
    let mut groups: HashMap<String, u32> = HashMap::new();
    for task in tasks {
        let key = match &task.parallel_group {
            Some(g) => g.clone(),
            None => format!("__solo__{}", task.id),
        };
        let est = task.estimated_minutes.unwrap_or(0);
        let entry = groups.entry(key).or_insert(0);
        *entry = (*entry).max(est);
    }
    groups.values().sum()
}

/// Check whether two tasks have a file conflict.
///
/// Two tasks conflict when they share at least one file path **and** both
/// have `exclusive_files` set to true (the default). If either task opts out
/// of exclusivity, there is no conflict.
pub fn has_file_conflict(a: &Task, b: &Task) -> bool {
    if !a.exclusive_files || !b.exclusive_files {
        return false;
    }
    let b_files: HashSet<&str> = b.files.iter().map(|s| s.as_str()).collect();
    a.files.iter().any(|f| b_files.contains(f.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str, deps: &[&str], group: Option<&str>, files: &[&str]) -> Task {
        make_task_with_estimate(id, deps, group, files, None)
    }

    fn make_task_with_estimate(
        id: &str,
        deps: &[&str],
        group: Option<&str>,
        files: &[&str],
        estimated_minutes: Option<u32>,
    ) -> Task {
        Task {
            id: id.to_string(),
            title: id.to_string(),
            status: TaskStatus::Pending,
            files: files.iter().map(|s| s.to_string()).collect(),
            acceptance: vec![],
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            parallel_group: group.map(|s| s.to_string()),
            exclusive_files: true,
            estimated_minutes,
            types_to_define: None,
            formulas: None,
            test_invariants: None,
            imports: None,
            example_pattern: None,
            context_files: None,
            plan_section: None,
            skills: None,
        }
    }

    #[test]
    fn dag_next_runnable_respects_deps() {
        let tasks = vec![
            make_task("a", &[], None, &[]),
            make_task("b", &["a"], None, &[]),
            make_task("c", &["b"], None, &[]),
        ];
        let dag = TaskDag::from_tasks(&tasks);

        let completed = HashSet::new();
        let runnable = dag.next_runnable(&completed);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0].id, "a");

        let completed: HashSet<String> = ["a".into()].into();
        let runnable = dag.next_runnable(&completed);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0].id, "b");
    }

    #[test]
    fn dag_parallel_group_batching() {
        let tasks = vec![
            make_task("a1", &[], Some("A"), &["x.rs"]),
            make_task("a2", &[], Some("A"), &["y.rs"]),
            make_task("b1", &["a1", "a2"], Some("B"), &["z.rs"]),
        ];
        let dag = TaskDag::from_tasks(&tasks);

        // Both a1 and a2 are runnable at the start.
        let runnable = dag.next_runnable(&HashSet::new());
        let ids: HashSet<&str> = runnable.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains("a1"));
        assert!(ids.contains("a2"));
        assert!(!ids.contains("b1"));

        assert_eq!(dag.max_parallel(), 2);
    }

    #[test]
    fn dag_all_complete() {
        let tasks = vec![
            make_task("a", &[], None, &[]),
            make_task("b", &[], None, &[]),
        ];
        let dag = TaskDag::from_tasks(&tasks);
        assert!(!dag.all_complete(&HashSet::new()));
        assert!(dag.all_complete(&["a".into(), "b".into()].into()));
    }

    #[test]
    fn file_conflict_detection() {
        let a = make_task("a", &[], None, &["src/foo.rs", "src/bar.rs"]);
        let b = make_task("b", &[], None, &["src/bar.rs", "src/baz.rs"]);
        assert!(has_file_conflict(&a, &b));

        // No overlap
        let c = make_task("c", &[], None, &["src/other.rs"]);
        assert!(!has_file_conflict(&a, &c));
    }

    #[test]
    fn file_conflict_respects_exclusive_flag() {
        let mut a = make_task("a", &[], None, &["src/shared.rs"]);
        let b = make_task("b", &[], None, &["src/shared.rs"]);

        // Both exclusive: conflict
        assert!(has_file_conflict(&a, &b));

        // One opts out: no conflict
        a.exclusive_files = false;
        assert!(!has_file_conflict(&a, &b));
    }

    #[test]
    fn task_group_estimated_minutes_is_max() {
        // Two tasks in group "A": 5 min and 10 min. Group estimate = 10.
        let tasks = vec![
            make_task_with_estimate("a1", &[], Some("A"), &[], Some(5)),
            make_task_with_estimate("a2", &[], Some("A"), &[], Some(10)),
        ];
        let dag = TaskDag::from_tasks(&tasks);
        let group = &dag.groups()[0];
        assert_eq!(group.estimated_minutes, 10);
    }

    #[test]
    fn dag_estimated_remaining_sequential_groups() {
        // Group A (a1=5, a2=10) -> Group B (b1=7)
        // Total = max(5,10) + 7 = 17
        let tasks = vec![
            make_task_with_estimate("a1", &[], Some("A"), &[], Some(5)),
            make_task_with_estimate("a2", &[], Some("A"), &[], Some(10)),
            make_task_with_estimate("b1", &["a1", "a2"], Some("B"), &[], Some(7)),
        ];
        let dag = TaskDag::from_tasks(&tasks);
        assert_eq!(dag.estimated_remaining(&HashSet::new()), 17);

        // After completing group A, remaining = 7
        let completed: HashSet<String> = ["a1".into(), "a2".into()].into();
        assert_eq!(dag.estimated_remaining(&completed), 7);

        // All done = 0
        let completed: HashSet<String> = ["a1".into(), "a2".into(), "b1".into()].into();
        assert_eq!(dag.estimated_remaining(&completed), 0);
    }

    #[test]
    fn dag_estimated_remaining_solo_tasks() {
        // Three independent solo tasks: 3, 5, 8. They each get their own
        // group, so total = 3 + 5 + 8 = 16.
        let tasks = vec![
            make_task_with_estimate("x", &[], None, &[], Some(3)),
            make_task_with_estimate("y", &[], None, &[], Some(5)),
            make_task_with_estimate("z", &[], None, &[], Some(8)),
        ];
        let dag = TaskDag::from_tasks(&tasks);
        assert_eq!(dag.estimated_remaining(&HashSet::new()), 16);
    }

    #[test]
    fn checklist_estimated_minutes() {
        let mut tasks = vec![
            make_task_with_estimate("a1", &[], Some("A"), &[], Some(5)),
            make_task_with_estimate("a2", &[], Some("A"), &[], Some(10)),
            make_task_with_estimate("b1", &[], None, &[], Some(3)),
        ];
        let checklist = TaskChecklist {
            plan_num: "01".to_string(),
            iteration: 1,
            tasks: tasks.clone(),
        };
        // Total: max(5,10) + 3 = 13
        assert_eq!(checklist.estimated_total_minutes(), 13);
        // All pending, so remaining = total
        assert_eq!(checklist.estimated_remaining_minutes(), 13);

        // Mark a1 done
        tasks[0].status = TaskStatus::Done;
        let checklist = TaskChecklist {
            plan_num: "01".to_string(),
            iteration: 1,
            tasks,
        };
        // Remaining: a2 (10, still in group A) + b1 (3) = 13
        // (a2 is the max in its group, so 10 + 3 = 13)
        assert_eq!(checklist.estimated_remaining_minutes(), 13);
    }

    #[test]
    fn checklist_remaining_after_group_done() {
        let mut tasks = vec![
            make_task_with_estimate("a1", &[], Some("A"), &[], Some(5)),
            make_task_with_estimate("a2", &[], Some("A"), &[], Some(10)),
            make_task_with_estimate("b1", &[], None, &[], Some(3)),
        ];
        // Mark both A-group tasks done
        tasks[0].status = TaskStatus::Done;
        tasks[1].status = TaskStatus::Done;
        let checklist = TaskChecklist {
            plan_num: "01".to_string(),
            iteration: 1,
            tasks,
        };
        // Only b1 remains: 3
        assert_eq!(checklist.estimated_remaining_minutes(), 3);
    }

    #[test]
    fn estimated_minutes_missing_treated_as_zero() {
        let tasks = vec![
            make_task_with_estimate("a", &[], Some("A"), &[], Some(10)),
            make_task_with_estimate("b", &[], Some("A"), &[], None),
        ];
        let dag = TaskDag::from_tasks(&tasks);
        // max(10, 0) = 10
        assert_eq!(dag.estimated_remaining(&HashSet::new()), 10);
    }
}
