use super::plan::PlanInfo;
use super::tasks::TaskFile;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet, VecDeque};

/// Globally unique task identifier: plan base name + task ID within that plan.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct GlobalTaskId {
    pub plan: String,
    pub task: String,
}

impl GlobalTaskId {
    /// Parse a "plan:task" string back into a GlobalTaskId.
    pub fn parse(s: &str) -> Option<Self> {
        let (plan, task) = s.split_once(':')?;
        if plan.is_empty() || task.is_empty() {
            return None;
        }
        Some(Self {
            plan: plan.to_string(),
            task: task.to_string(),
        })
    }
}

impl std::fmt::Display for GlobalTaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.plan, self.task)
    }
}

/// Cross-plan task-level DAG that enables maximum parallelism by tracking
/// dependencies between individual tasks across plans rather than treating
/// each plan as an atomic unit.
pub struct UnifiedTaskDag {
    nodes: Vec<GlobalTaskId>,
    deps: HashMap<GlobalTaskId, HashSet<GlobalTaskId>>,
    reverse_deps: HashMap<GlobalTaskId, HashSet<GlobalTaskId>>,
    file_sets: HashMap<GlobalTaskId, HashSet<String>>,
    estimates: HashMap<GlobalTaskId, u32>,
    plan_for_task: HashMap<GlobalTaskId, String>,
}

/// The synthetic task ID used for plans that have no task breakdown.
const WHOLE_PLAN_TASK: &str = "__whole__";

impl UnifiedTaskDag {
    /// Build a unified task DAG from a set of plans and their loaded task files.
    ///
    /// `task_files` maps plan base name -> parsed TaskFile. Plans without an
    /// entry in `task_files` are treated as atomic (one synthetic node).
    pub fn from_plans(plans: &[PlanInfo], task_files: &HashMap<String, TaskFile>) -> Result<Self> {
        let plan_bases: HashSet<String> = plans.iter().map(|p| p.base.clone()).collect();

        let mut nodes = Vec::new();
        let mut deps: HashMap<GlobalTaskId, HashSet<GlobalTaskId>> = HashMap::new();
        let mut reverse_deps: HashMap<GlobalTaskId, HashSet<GlobalTaskId>> = HashMap::new();
        let mut file_sets: HashMap<GlobalTaskId, HashSet<String>> = HashMap::new();
        let mut estimates: HashMap<GlobalTaskId, u32> = HashMap::new();
        let mut plan_for_task: HashMap<GlobalTaskId, String> = HashMap::new();

        // Phase 1: create nodes
        for plan in plans {
            if let Some(tf) = task_files.get(&plan.base) {
                for task in &tf.tasks {
                    let gid = GlobalTaskId {
                        plan: plan.base.clone(),
                        task: task.id.clone(),
                    };
                    nodes.push(gid.clone());
                    deps.entry(gid.clone()).or_default();
                    reverse_deps.entry(gid.clone()).or_default();
                    file_sets.insert(gid.clone(), task.files.iter().cloned().collect());
                    if let Some(est) = task.estimated_minutes {
                        estimates.insert(gid.clone(), est);
                    }
                    plan_for_task.insert(gid, plan.base.clone());
                }
            } else {
                // Synthetic whole-plan node
                let gid = GlobalTaskId {
                    plan: plan.base.clone(),
                    task: WHOLE_PLAN_TASK.to_string(),
                };
                nodes.push(gid.clone());
                deps.entry(gid.clone()).or_default();
                reverse_deps.entry(gid.clone()).or_default();
                // Whole-plan nodes get the plan's crates_touched as file set
                if let Some(ref fm) = plan.frontmatter {
                    file_sets.insert(gid.clone(), fm.crates_touched.iter().cloned().collect());
                    if let Some(est) = fm.estimated_minutes {
                        estimates.insert(gid.clone(), est);
                    }
                }
                plan_for_task.insert(gid, plan.base.clone());
            }
        }

        // Build a lookup: plan base num prefix -> plan base name
        let num_to_base: HashMap<String, String> = plans
            .iter()
            .map(|p| (p.num.clone(), p.base.clone()))
            .collect();

        // Phase 2: wire up edges

        // 2a. Intra-plan task deps + cross-plan explicit deps
        for plan in plans {
            if let Some(tf) = task_files.get(&plan.base) {
                for task in &tf.tasks {
                    let gid = GlobalTaskId {
                        plan: plan.base.clone(),
                        task: task.id.clone(),
                    };
                    for dep_ref in &task.depends_on {
                        if let Some((plan_prefix, task_id)) = dep_ref.split_once(':') {
                            // Cross-plan: "09:T3"
                            let resolved_plan = num_to_base
                                .get(plan_prefix)
                                .or_else(|| plan_bases.get(plan_prefix).map(|s| s));
                            if let Some(dep_plan) = resolved_plan {
                                let dep_gid = GlobalTaskId {
                                    plan: dep_plan.clone(),
                                    task: task_id.to_string(),
                                };
                                if deps.contains_key(&dep_gid) {
                                    deps.entry(gid.clone()).or_default().insert(dep_gid.clone());
                                    reverse_deps.entry(dep_gid).or_default().insert(gid.clone());
                                }
                            }
                        } else {
                            // Intra-plan dep
                            let dep_gid = GlobalTaskId {
                                plan: plan.base.clone(),
                                task: dep_ref.clone(),
                            };
                            if deps.contains_key(&dep_gid) {
                                deps.entry(gid.clone()).or_default().insert(dep_gid.clone());
                                reverse_deps.entry(dep_gid).or_default().insert(gid.clone());
                            }
                        }
                    }
                }
            }
        }

        // 2b. Implicit plan-level deps: if plan B depends_on plan A (frontmatter)
        // and plan A has no task breakdown, all tasks in B depend on A's __whole__ node.
        for plan in plans {
            if let Some(ref fm) = plan.frontmatter {
                for dep_ref in &fm.depends_on {
                    // Resolve the dep_ref to a plan base
                    let dep_plan = num_to_base
                        .get(dep_ref)
                        .or_else(|| plan_bases.get(dep_ref).map(|s| s));
                    if let Some(dep_plan) = dep_plan {
                        // Only create implicit edges if dep plan has NO task file
                        // (i.e. it has a synthetic __whole__ node)
                        if !task_files.contains_key(dep_plan) {
                            let whole_gid = GlobalTaskId {
                                plan: dep_plan.clone(),
                                task: WHOLE_PLAN_TASK.to_string(),
                            };
                            if deps.contains_key(&whole_gid) {
                                // All tasks in this plan depend on the whole node
                                let this_plan_tasks: Vec<GlobalTaskId> = nodes
                                    .iter()
                                    .filter(|n| n.plan == plan.base)
                                    .cloned()
                                    .collect();
                                for task_gid in this_plan_tasks {
                                    deps.entry(task_gid.clone())
                                        .or_default()
                                        .insert(whole_gid.clone());
                                    reverse_deps
                                        .entry(whole_gid.clone())
                                        .or_default()
                                        .insert(task_gid);
                                }
                            }
                        }
                    }
                }
            }
        }

        // G3: Validate that all cross-plan depends_on reference existing plans.
        // When running a subset of plans, deps on plans not in the current set
        // are treated as already-satisfied (the dep was completed in a prior run).
        for plan in plans {
            if let Some(ref fm) = plan.frontmatter {
                for dep_ref in &fm.depends_on {
                    let resolved = num_to_base.get(dep_ref).or_else(|| plan_bases.get(dep_ref));
                    if resolved.is_none() {
                        tracing::warn!(
                            "Plan {} depends_on '{}' which is not in the current run — \
                             treating as already complete",
                            plan.base,
                            dep_ref
                        );
                    }
                }
            }
        }

        let dag = UnifiedTaskDag {
            nodes,
            deps,
            reverse_deps,
            file_sets,
            estimates,
            plan_for_task,
        };
        dag.validate_no_cycles()?;
        Ok(dag)
    }

    /// Return tasks that are ready to run: all deps completed, not in-flight,
    /// and no file overlap with any in-flight task.
    pub fn next_runnable(
        &self,
        completed: &HashSet<GlobalTaskId>,
        in_flight: &HashSet<GlobalTaskId>,
    ) -> Vec<&GlobalTaskId> {
        // Collect all files touched by in-flight tasks
        let in_flight_files: HashSet<&String> = in_flight
            .iter()
            .flat_map(|gid| self.file_sets.get(gid).into_iter().flatten())
            .collect();

        self.nodes
            .iter()
            .filter(|gid| {
                // Not already done or running
                if completed.contains(gid) || in_flight.contains(gid) {
                    return false;
                }
                // All deps satisfied
                if let Some(dep_set) = self.deps.get(gid) {
                    if !dep_set.iter().all(|d| completed.contains(d)) {
                        return false;
                    }
                }
                // No file overlap with in-flight
                if let Some(my_files) = self.file_sets.get(gid) {
                    if my_files.iter().any(|f| in_flight_files.contains(f)) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Theoretical max concurrency: the largest set of tasks with no mutual
    /// dependencies and no file overlap. This is a heuristic (greedy) since
    /// the exact problem is NP-hard.
    pub fn max_width(&self) -> usize {
        // Use the simpler approach: find the max number of tasks at any
        // "level" of the DAG (computed via longest-path-from-source).
        let levels = self.compute_levels();
        let mut level_counts: HashMap<usize, usize> = HashMap::new();
        for (_, level) in &levels {
            *level_counts.entry(*level).or_insert(0) += 1;
        }
        level_counts.values().copied().max().unwrap_or(1)
    }

    /// Longest weighted path through the DAG in estimated minutes.
    pub fn critical_path_minutes(&self) -> u32 {
        let topo = self.topological_order();
        if topo.is_empty() {
            return 0;
        }

        let mut dist: HashMap<&GlobalTaskId, u32> = HashMap::new();

        for gid in &topo {
            let my_time = self.estimates.get(gid).copied().unwrap_or(0);
            dist.insert(gid, my_time);
        }

        for gid in &topo {
            let current_dist = dist[gid];
            if let Some(dependents) = self.reverse_deps.get(gid) {
                for dep in dependents {
                    let dep_time = self.estimates.get(dep).copied().unwrap_or(0);
                    let new_dist = current_dist + dep_time;
                    let entry = dist.entry(dep).or_insert(0);
                    if new_dist > *entry {
                        *entry = new_dist;
                    }
                }
            }
        }

        dist.values().copied().max().unwrap_or(0)
    }

    /// Find cross-plan references that don't resolve to any known task.
    pub fn find_dangling_references(
        plans: &[PlanInfo],
        task_files: &HashMap<String, TaskFile>,
    ) -> Vec<String> {
        let plan_bases: HashSet<String> = plans.iter().map(|p| p.base.clone()).collect();
        let num_to_base: HashMap<String, String> = plans
            .iter()
            .map(|p| (p.num.clone(), p.base.clone()))
            .collect();

        // Collect all known task IDs per plan
        let mut known_tasks: HashMap<String, HashSet<String>> = HashMap::new();
        for plan in plans {
            if let Some(tf) = task_files.get(&plan.base) {
                let ids: HashSet<String> = tf.tasks.iter().map(|t| t.id.clone()).collect();
                known_tasks.insert(plan.base.clone(), ids);
            } else {
                let mut ids = HashSet::new();
                ids.insert(WHOLE_PLAN_TASK.to_string());
                known_tasks.insert(plan.base.clone(), ids);
            }
        }

        let mut dangling = Vec::new();

        for plan in plans {
            if let Some(tf) = task_files.get(&plan.base) {
                for task in &tf.tasks {
                    for dep_ref in &task.depends_on {
                        if let Some((plan_prefix, task_id)) = dep_ref.split_once(':') {
                            let resolved_plan = num_to_base
                                .get(plan_prefix)
                                .or_else(|| plan_bases.get(plan_prefix).map(|s| s));
                            match resolved_plan {
                                Some(dep_plan) => {
                                    if let Some(tasks) = known_tasks.get(dep_plan) {
                                        if !tasks.contains(task_id) {
                                            dangling.push(format!(
                                                "{}:{} references unknown task {} in plan {}",
                                                plan.base, task.id, task_id, dep_plan
                                            ));
                                        }
                                    } else {
                                        dangling.push(format!(
                                            "{}:{} references unknown plan {}",
                                            plan.base, task.id, plan_prefix
                                        ));
                                    }
                                }
                                None => {
                                    dangling.push(format!(
                                        "{}:{} references unknown plan prefix {}",
                                        plan.base, task.id, plan_prefix
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        dangling
    }

    /// Return all GlobalTaskIds belonging to a given plan.
    pub fn tasks_for_plan(&self, plan: &str) -> Vec<&GlobalTaskId> {
        self.nodes.iter().filter(|gid| gid.plan == plan).collect()
    }

    /// True when every task for the given plan is in the completed set.
    pub fn all_plan_tasks_complete(&self, plan: &str, completed: &HashSet<GlobalTaskId>) -> bool {
        self.nodes
            .iter()
            .filter(|gid| gid.plan == plan)
            .all(|gid| completed.contains(gid))
    }

    /// Get the dependency set for a specific task node.
    pub fn task_deps(&self, gid: &GlobalTaskId) -> Option<&HashSet<GlobalTaskId>> {
        self.deps.get(gid)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn validate_no_cycles(&self) -> Result<()> {
        let mut in_degree: HashMap<&GlobalTaskId, usize> = HashMap::new();
        for gid in &self.nodes {
            in_degree.insert(gid, self.deps.get(gid).map_or(0, |s| s.len()));
        }

        let mut queue: VecDeque<&GlobalTaskId> = VecDeque::new();
        for (&gid, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(gid);
            }
        }

        let mut visited = 0usize;
        while let Some(gid) = queue.pop_front() {
            visited += 1;
            if let Some(dependents) = self.reverse_deps.get(gid) {
                for dep in dependents {
                    if let Some(d) = in_degree.get_mut(dep) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }

        if visited != self.nodes.len() {
            bail!(
                "Unified task DAG contains a cycle -- {} tasks could not be sorted",
                self.nodes.len() - visited
            );
        }
        Ok(())
    }

    fn topological_order(&self) -> Vec<&GlobalTaskId> {
        let mut in_degree: HashMap<&GlobalTaskId, usize> = HashMap::new();
        for gid in &self.nodes {
            in_degree.insert(gid, self.deps.get(gid).map_or(0, |s| s.len()));
        }

        let mut queue: VecDeque<&GlobalTaskId> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&gid, _)| gid)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(gid) = queue.pop_front() {
            order.push(gid);
            if let Some(dependents) = self.reverse_deps.get(gid) {
                for dep in dependents {
                    if let Some(d) = in_degree.get_mut(dep) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }
        order
    }

    /// Total number of distinct plans in the DAG.
    pub fn plan_count(&self) -> usize {
        let plans: HashSet<&str> = self.nodes.iter().map(|g| g.plan.as_str()).collect();
        plans.len()
    }

    /// Total number of task nodes in the DAG.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// All distinct plan base names, sorted.
    pub fn all_plans(&self) -> Vec<&str> {
        let mut plans: Vec<&str> = self
            .nodes
            .iter()
            .map(|g| g.plan.as_str())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        plans.sort();
        plans
    }

    /// Plan-level dependencies for a given plan: which other plans must have
    /// at least one task completed before any task in this plan can start.
    ///
    /// This is derived from the task-level edges: if any task in `plan` depends
    /// on a task in another plan, that other plan is a dependency.
    pub fn plan_dependencies(&self, plan: &str) -> HashSet<String> {
        let mut dep_plans = HashSet::new();
        for gid in self.nodes.iter().filter(|g| g.plan == plan) {
            if let Some(dep_set) = self.deps.get(gid) {
                for dep in dep_set {
                    if dep.plan != plan {
                        dep_plans.insert(dep.plan.clone());
                    }
                }
            }
        }
        dep_plans
    }

    /// Assign each node a "level" = longest path from any source to that node.
    fn compute_levels(&self) -> HashMap<&GlobalTaskId, usize> {
        let topo = self.topological_order();
        let mut levels: HashMap<&GlobalTaskId, usize> = HashMap::new();

        for gid in &topo {
            levels.insert(gid, 0);
        }

        for gid in &topo {
            let my_level = levels[gid];
            if let Some(dependents) = self.reverse_deps.get(gid) {
                for dep in dependents {
                    let entry = levels.entry(dep).or_insert(0);
                    if my_level + 1 > *entry {
                        *entry = my_level + 1;
                    }
                }
            }
        }

        levels
    }

    /// Partition a set of task IDs (all from the same plan) into independent groups.
    /// Two tasks are in the same group if they share at least one file (conflict edge).
    /// Tasks with no file set are singletons.
    /// Returns a Vec of groups; if only one group, caller should use the single-batch path.
    pub fn independent_groups(&self, tasks: &[GlobalTaskId]) -> Vec<Vec<GlobalTaskId>> {
        if tasks.is_empty() {
            return vec![];
        }
        if tasks.len() == 1 {
            return vec![tasks.to_vec()];
        }
        // Union-Find over task indices
        let n = tasks.len();
        let mut parent: Vec<usize> = (0..n).collect();

        fn find(parent: &mut Vec<usize>, x: usize) -> usize {
            if parent[x] != x {
                parent[x] = find(parent, parent[x]);
            }
            parent[x]
        }
        fn union(parent: &mut Vec<usize>, x: usize, y: usize) {
            let rx = find(parent, x);
            let ry = find(parent, y);
            if rx != ry {
                parent[rx] = ry;
            }
        }

        // Join tasks that share at least one file
        for i in 0..n {
            let files_i = self.file_sets.get(&tasks[i]);
            for j in (i + 1)..n {
                let files_j = self.file_sets.get(&tasks[j]);
                if let (Some(fi), Some(fj)) = (files_i, files_j) {
                    if fi.iter().any(|f| fj.contains(f)) {
                        union(&mut parent, i, j);
                    }
                }
            }
        }

        // Collect groups
        let mut groups: HashMap<usize, Vec<GlobalTaskId>> = HashMap::new();
        for (i, task) in tasks.iter().enumerate() {
            let root = find(&mut parent, i);
            groups.entry(root).or_default().push(task.clone());
        }

        groups.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::plan::{PlanFrontmatter, PlanInfo};
    use super::super::tasks::{Task, TaskFile, TaskMeta, TaskStatus};
    use super::*;
    use std::path::PathBuf;

    fn make_plan(base: &str, depends_on: Vec<&str>) -> PlanInfo {
        PlanInfo {
            base: base.to_string(),
            num: base.split('-').next().unwrap_or(base).to_string(),
            path: PathBuf::from(format!("plans/{base}.md")),
            frontmatter: Some(PlanFrontmatter {
                plan: Some(base.to_string()),
                depends_on: depends_on.into_iter().map(String::from).collect(),
                parallel_safe: true,
                tasks: vec![],
                ..Default::default()
            }),
        }
    }

    fn make_task(id: &str, deps: &[&str], files: &[&str]) -> Task {
        make_task_with_estimate(id, deps, files, None)
    }

    fn make_task_with_estimate(id: &str, deps: &[&str], files: &[&str], est: Option<u32>) -> Task {
        Task {
            id: id.to_string(),
            title: id.to_string(),
            status: TaskStatus::Pending,
            files: files.iter().map(|s| s.to_string()).collect(),
            acceptance: vec![],
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            parallel_group: None,
            exclusive_files: true,
            estimated_minutes: est,
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

    fn make_task_file(plan: &str, tasks: Vec<Task>) -> TaskFile {
        TaskFile {
            meta: TaskMeta {
                plan: plan.to_string(),
                iteration: 1,
                total: tasks.len(),
                done: 0,
                max_parallel: None,
                estimated_total_minutes: None,
            },
            tasks,
        }
    }

    fn gid(plan: &str, task: &str) -> GlobalTaskId {
        GlobalTaskId {
            plan: plan.to_string(),
            task: task.to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[test]
    fn linear_chain_within_one_plan() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![
            make_task("T1", &[], &["a.rs"]),
            make_task("T2", &["T1"], &["b.rs"]),
            make_task("T3", &["T2"], &["c.rs"]),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        let completed = HashSet::new();
        let in_flight = HashSet::new();
        let runnable = dag.next_runnable(&completed, &in_flight);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("01-alpha", "T1"));

        let completed: HashSet<GlobalTaskId> = [gid("01-alpha", "T1")].into();
        let runnable = dag.next_runnable(&completed, &in_flight);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("01-alpha", "T2"));

        let completed: HashSet<GlobalTaskId> =
            [gid("01-alpha", "T1"), gid("01-alpha", "T2")].into();
        let runnable = dag.next_runnable(&completed, &in_flight);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("01-alpha", "T3"));
    }

    #[test]
    fn cross_plan_dependency() {
        let plans = vec![
            make_plan("01-alpha", vec![]),
            make_plan("02-beta", vec!["01-alpha"]),
        ];
        let tasks_01 = vec![
            make_task("T1", &[], &["a.rs"]),
            make_task("T2", &["T1"], &["b.rs"]),
            make_task("T3", &["T2"], &["c.rs"]),
            make_task("T4", &["T3"], &["d.rs"]),
        ];
        // 02's T1 depends on 01's T4 via cross-plan ref "01:T4"
        let tasks_02 = vec![
            make_task("T1", &["01:T4"], &["e.rs"]),
            make_task("T2", &["T1"], &["f.rs"]),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks_01));
        tf.insert("02-beta".to_string(), make_task_file("02-beta", tasks_02));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        // Initially only 01:T1 is runnable
        let runnable = dag.next_runnable(&HashSet::new(), &HashSet::new());
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("01-alpha", "T1"));

        // After completing all of plan 01, 02:T1 becomes runnable
        let completed: HashSet<GlobalTaskId> = [
            gid("01-alpha", "T1"),
            gid("01-alpha", "T2"),
            gid("01-alpha", "T3"),
            gid("01-alpha", "T4"),
        ]
        .into();
        let runnable = dag.next_runnable(&completed, &HashSet::new());
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("02-beta", "T1"));
    }

    #[test]
    fn synthetic_whole_node_for_plan_without_tasks() {
        // Plan A has no task file -> synthetic __whole__ node
        // Plan B depends on plan A -> all B tasks depend on A:__whole__
        let plans = vec![
            make_plan("01-alpha", vec![]),
            make_plan("02-beta", vec!["01-alpha"]),
        ];
        let tasks_02 = vec![
            make_task("T1", &[], &["e.rs"]),
            make_task("T2", &["T1"], &["f.rs"]),
        ];
        let mut tf = HashMap::new();
        // No task file for 01-alpha
        tf.insert("02-beta".to_string(), make_task_file("02-beta", tasks_02));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        // Only the __whole__ node is runnable initially
        let runnable = dag.next_runnable(&HashSet::new(), &HashSet::new());
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("01-alpha", WHOLE_PLAN_TASK));

        // After __whole__ completes, 02:T1 becomes runnable
        let completed: HashSet<GlobalTaskId> = [gid("01-alpha", WHOLE_PLAN_TASK)].into();
        let runnable = dag.next_runnable(&completed, &HashSet::new());
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("02-beta", "T1"));
    }

    #[test]
    fn file_conflict_blocks_concurrent_scheduling() {
        let plans = vec![make_plan("01-alpha", vec![])];
        // T1 and T2 are independent but touch the same file
        let tasks = vec![
            make_task("T1", &[], &["shared.rs"]),
            make_task("T2", &[], &["shared.rs"]),
            make_task("T3", &[], &["other.rs"]),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        // All three are runnable when nothing is in flight
        let runnable = dag.next_runnable(&HashSet::new(), &HashSet::new());
        assert_eq!(runnable.len(), 3);

        // With T1 in flight, T2 is blocked (file conflict) but T3 is still runnable
        let in_flight: HashSet<GlobalTaskId> = [gid("01-alpha", "T1")].into();
        let runnable = dag.next_runnable(&HashSet::new(), &in_flight);
        let runnable_ids: HashSet<String> = runnable.iter().map(|g| g.task.clone()).collect();
        assert!(runnable_ids.contains("T3"));
        assert!(
            !runnable_ids.contains("T2"),
            "T2 should be blocked by file conflict with T1"
        );
        assert!(!runnable_ids.contains("T1"), "T1 is in flight");
    }

    #[test]
    fn next_runnable_respects_in_flight_exclusion() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![
            make_task("T1", &[], &["a.rs"]),
            make_task("T2", &[], &["b.rs"]),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        // Both runnable initially
        let runnable = dag.next_runnable(&HashSet::new(), &HashSet::new());
        assert_eq!(runnable.len(), 2);

        // T1 in flight -> only T2 runnable
        let in_flight: HashSet<GlobalTaskId> = [gid("01-alpha", "T1")].into();
        let runnable = dag.next_runnable(&HashSet::new(), &in_flight);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0], &gid("01-alpha", "T2"));
    }

    #[test]
    fn critical_path_minutes_calculation() {
        // T1(5) -> T2(10) -> T3(3)
        // T4(20) independent
        // Critical path = T1 + T2 + T3 = 18? No, T4=20 is longer if standalone.
        // Actually T4 has no deps, so its path is just 20.
        // T1->T2->T3 = 5+10+3 = 18. So critical path = 20.
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![
            make_task_with_estimate("T1", &[], &[], Some(5)),
            make_task_with_estimate("T2", &["T1"], &[], Some(10)),
            make_task_with_estimate("T3", &["T2"], &[], Some(3)),
            make_task_with_estimate("T4", &[], &[], Some(20)),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();
        assert_eq!(dag.critical_path_minutes(), 20);
    }

    #[test]
    fn critical_path_minutes_chain() {
        // Linear chain: T1(5) -> T2(10) -> T3(15) = 30
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![
            make_task_with_estimate("T1", &[], &[], Some(5)),
            make_task_with_estimate("T2", &["T1"], &[], Some(10)),
            make_task_with_estimate("T3", &["T2"], &[], Some(15)),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();
        assert_eq!(dag.critical_path_minutes(), 30);
    }

    #[test]
    fn dangling_reference_detection() {
        let plans = vec![make_plan("01-alpha", vec![]), make_plan("02-beta", vec![])];
        let tasks_01 = vec![make_task("T1", &[], &[])];
        // Reference a task that doesn't exist in 01
        let tasks_02 = vec![make_task("T1", &["01:T99"], &[])];

        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks_01));
        tf.insert("02-beta".to_string(), make_task_file("02-beta", tasks_02));

        let dangling = UnifiedTaskDag::find_dangling_references(&plans, &tf);
        assert_eq!(dangling.len(), 1);
        assert!(dangling[0].contains("T99"));
    }

    #[test]
    fn dangling_reference_unknown_plan() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![make_task("T1", &["99:T1"], &[])];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dangling = UnifiedTaskDag::find_dangling_references(&plans, &tf);
        assert_eq!(dangling.len(), 1);
        assert!(dangling[0].contains("99"));
    }

    #[test]
    fn all_plan_tasks_complete_check() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![make_task("T1", &[], &[]), make_task("T2", &["T1"], &[])];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        let empty: HashSet<GlobalTaskId> = HashSet::new();
        assert!(!dag.all_plan_tasks_complete("01-alpha", &empty));

        let partial: HashSet<GlobalTaskId> = [gid("01-alpha", "T1")].into();
        assert!(!dag.all_plan_tasks_complete("01-alpha", &partial));

        let full: HashSet<GlobalTaskId> = [gid("01-alpha", "T1"), gid("01-alpha", "T2")].into();
        assert!(dag.all_plan_tasks_complete("01-alpha", &full));
    }

    #[test]
    fn max_width_parallel_tasks() {
        let plans = vec![make_plan("01-alpha", vec![])];
        // 3 independent tasks, then 1 dependent on all 3
        let tasks = vec![
            make_task("T1", &[], &[]),
            make_task("T2", &[], &[]),
            make_task("T3", &[], &[]),
            make_task("T4", &["T1", "T2", "T3"], &[]),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();
        assert_eq!(dag.max_width(), 3);
    }

    #[test]
    fn tasks_for_plan_returns_correct_set() {
        let plans = vec![make_plan("01-alpha", vec![]), make_plan("02-beta", vec![])];
        let mut tf = HashMap::new();
        tf.insert(
            "01-alpha".to_string(),
            make_task_file(
                "01-alpha",
                vec![make_task("T1", &[], &[]), make_task("T2", &[], &[])],
            ),
        );
        tf.insert(
            "02-beta".to_string(),
            make_task_file("02-beta", vec![make_task("T1", &[], &[])]),
        );

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        let alpha_tasks = dag.tasks_for_plan("01-alpha");
        assert_eq!(alpha_tasks.len(), 2);

        let beta_tasks = dag.tasks_for_plan("02-beta");
        assert_eq!(beta_tasks.len(), 1);

        let unknown = dag.tasks_for_plan("99-nope");
        assert!(unknown.is_empty());
    }

    #[test]
    fn independent_groups_partitions_by_file_conflict() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![
            // Group A: T1, T2 share file "a.rs"
            make_task("T1", &[], &["a.rs"]),
            make_task("T2", &[], &["a.rs"]),
            // Group B: T3, T4 share file "b.rs"
            make_task("T3", &[], &["b.rs"]),
            make_task("T4", &[], &["b.rs"]),
            // Singleton: T5 has no files
            make_task("T5", &[], &[]),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        let all_tasks = dag.tasks_for_plan("01-alpha");
        let groups =
            dag.independent_groups(&all_tasks.iter().map(|t| (*t).clone()).collect::<Vec<_>>());

        // Should have 3 groups: {T1, T2}, {T3, T4}, {T5}
        assert_eq!(groups.len(), 3);

        // Each group should have the expected tasks
        let mut group_sizes: Vec<usize> = groups.iter().map(|g| g.len()).collect();
        group_sizes.sort();
        assert_eq!(group_sizes, vec![1, 2, 2]); // One singleton, two pairs
    }

    #[test]
    fn independent_groups_single_group_for_shared_files() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let tasks = vec![
            // All three tasks share the same file
            make_task("T1", &[], &["a.rs"]),
            make_task("T2", &[], &["a.rs"]),
            make_task("T3", &[], &["a.rs"]),
        ];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", tasks));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        let all_tasks = dag.tasks_for_plan("01-alpha");
        let groups =
            dag.independent_groups(&all_tasks.iter().map(|t| (*t).clone()).collect::<Vec<_>>());

        // Should have 1 group with all 3 tasks
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn independent_groups_empty_returns_empty() {
        let plans = vec![make_plan("01-alpha", vec![])];
        let mut tf = HashMap::new();
        tf.insert("01-alpha".to_string(), make_task_file("01-alpha", vec![]));

        let dag = UnifiedTaskDag::from_plans(&plans, &tf).unwrap();

        let groups = dag.independent_groups(&[]);
        assert_eq!(groups.len(), 0);
    }
}
