use super::plan::PlanInfo;
use super::tasks::TaskFile;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet, VecDeque};

/// A group of plans that can execute in parallel (no mutual dependencies).
#[derive(Debug, Clone)]
pub struct ExecutionWave {
    pub index: usize,
    pub plans: Vec<String>, // plan base names
    /// Estimated total minutes for this wave (max of constituent plan estimates)
    pub estimated_minutes: u32,
}

/// Directed acyclic graph of plan dependencies, built from frontmatter
/// `depends_on` fields.
pub struct PlanDag {
    /// plan_base -> set of plan_bases it depends on
    deps: HashMap<String, HashSet<String>>,
    /// plan_base -> set of plan_bases that depend on it
    reverse_deps: HashMap<String, HashSet<String>>,
    /// All plan bases in sorted order
    all_plans: Vec<String>,
    /// plan_base -> estimated_minutes from frontmatter
    estimates: HashMap<String, u32>,
}

impl PlanDag {
    /// Build a DAG from a slice of discovered plans.
    ///
    /// Plans that have frontmatter with `depends_on` use those explicit
    /// dependencies. Plans without frontmatter fall back to sequential
    /// ordering: each plan depends on the one before it (by sorted position).
    pub fn from_plans(plans: &[PlanInfo]) -> Result<Self> {
        let all_plans: Vec<String> = {
            let mut v: Vec<String> = plans.iter().map(|p| p.base.clone()).collect();
            v.sort();
            v
        };

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
        let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::new();
        let mut estimates: HashMap<String, u32> = HashMap::new();

        // Initialize entries for every plan
        for base in &all_plans {
            deps.entry(base.clone()).or_default();
            reverse_deps.entry(base.clone()).or_default();
        }

        for plan in plans {
            let base = &plan.base;
            let has_frontmatter = plan.frontmatter.is_some();

            if let Some(ref fm) = plan.frontmatter {
                if let Some(mins) = fm.estimated_minutes {
                    estimates.insert(base.clone(), mins);
                }

                if fm.depends_on.is_empty() && fm.plan.is_none() {
                    // Frontmatter present but no plan id and no deps: sequential fallback
                    let sorted_pos = all_plans.iter().position(|b| b == base);
                    if let Some(pos) = sorted_pos {
                        if pos > 0 {
                            let prev = &all_plans[pos - 1];
                            deps.entry(base.clone()).or_default().insert(prev.clone());
                            reverse_deps
                                .entry(prev.clone())
                                .or_default()
                                .insert(base.clone());
                        }
                    }
                } else {
                    // Explicit dependencies
                    for dep in &fm.depends_on {
                        if let Some(resolved) = resolve_plan_name(dep, &all_plans) {
                            deps.entry(base.clone())
                                .or_default()
                                .insert(resolved.clone());
                            reverse_deps
                                .entry(resolved)
                                .or_default()
                                .insert(base.clone());
                        }
                    }
                }
            }

            if !has_frontmatter {
                // Backward-compat: depend on the previous plan in sorted order
                let sorted_pos = all_plans.iter().position(|b| b == base);
                if let Some(pos) = sorted_pos {
                    if pos > 0 {
                        let prev = &all_plans[pos - 1];
                        deps.entry(base.clone()).or_default().insert(prev.clone());
                        reverse_deps
                            .entry(prev.clone())
                            .or_default()
                            .insert(base.clone());
                    }
                }
            }
        }

        let dag = PlanDag {
            deps,
            reverse_deps,
            all_plans,
            estimates,
        };
        dag.validate_no_cycles()?;
        Ok(dag)
    }

    /// Build a DAG using task files to derive plan-level dependencies.
    ///
    /// For each plan that has a task file, cross-plan `dep_ref` entries of the
    /// form `"planNum:taskId"` are collapsed to plan-level edges. This replaces
    /// the conservative sequential fallback with the real dependency graph
    /// encoded in the task files, enabling maximum plan-level parallelism.
    ///
    /// Plans without task files fall back to the same logic as `from_plans`
    /// (frontmatter `depends_on`, or sequential order if neither is present).
    pub fn from_plans_and_tasks(
        plans: &[PlanInfo],
        task_files: &HashMap<String, TaskFile>,
    ) -> Result<Self> {
        let all_plans: Vec<String> = {
            let mut v: Vec<String> = plans.iter().map(|p| p.base.clone()).collect();
            v.sort();
            v
        };

        let num_to_base: HashMap<String, String> = plans
            .iter()
            .map(|p| (p.num.clone(), p.base.clone()))
            .collect();

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
        let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::new();
        let mut estimates: HashMap<String, u32> = HashMap::new();

        for base in &all_plans {
            deps.entry(base.clone()).or_default();
            reverse_deps.entry(base.clone()).or_default();
        }

        for plan in plans {
            if let Some(ref fm) = plan.frontmatter {
                if let Some(mins) = fm.estimated_minutes {
                    estimates.insert(plan.base.clone(), mins);
                }
            }
        }

        for plan in plans {
            let base = &plan.base;

            if let Some(tf) = task_files.get(base) {
                // Derive plan-level deps from cross-plan task references.
                // A "planNum:taskId" reference in any task means this plan
                // depends on that plan at the plan level.
                let mut cross_plan_deps: HashSet<String> = HashSet::new();
                for task in &tf.tasks {
                    for dep_ref in &task.depends_on {
                        if let Some((plan_prefix, _)) = dep_ref.split_once(':') {
                            let dep_base = num_to_base.get(plan_prefix).cloned().or_else(|| {
                                if all_plans.contains(&plan_prefix.to_string()) {
                                    Some(plan_prefix.to_string())
                                } else {
                                    None
                                }
                            });
                            if let Some(dep_base) = dep_base {
                                if &dep_base != base {
                                    cross_plan_deps.insert(dep_base);
                                }
                            }
                        }
                    }
                }
                // If no cross-plan deps found, the plan is treated as
                // independent (placed in the earliest possible wave).
                for dep in cross_plan_deps {
                    deps.entry(base.clone()).or_default().insert(dep.clone());
                    reverse_deps.entry(dep).or_default().insert(base.clone());
                }
            } else {
                // No task file: mirror from_plans fallback logic.
                let has_frontmatter = plan.frontmatter.is_some();
                if let Some(ref fm) = plan.frontmatter {
                    if fm.depends_on.is_empty() && fm.plan.is_none() {
                        let pos = all_plans.iter().position(|b| b == base);
                        if let Some(pos) = pos {
                            if pos > 0 {
                                let prev = &all_plans[pos - 1];
                                deps.entry(base.clone()).or_default().insert(prev.clone());
                                reverse_deps
                                    .entry(prev.clone())
                                    .or_default()
                                    .insert(base.clone());
                            }
                        }
                    } else {
                        for dep in &fm.depends_on {
                            if let Some(resolved) = resolve_plan_name(dep, &all_plans) {
                                deps.entry(base.clone())
                                    .or_default()
                                    .insert(resolved.clone());
                                reverse_deps
                                    .entry(resolved)
                                    .or_default()
                                    .insert(base.clone());
                            }
                        }
                    }
                }
                if !has_frontmatter {
                    let pos = all_plans.iter().position(|b| b == base);
                    if let Some(pos) = pos {
                        if pos > 0 {
                            let prev = &all_plans[pos - 1];
                            deps.entry(base.clone()).or_default().insert(prev.clone());
                            reverse_deps
                                .entry(prev.clone())
                                .or_default()
                                .insert(base.clone());
                        }
                    }
                }
            }
        }

        let dag = PlanDag {
            deps,
            reverse_deps,
            all_plans,
            estimates,
        };
        dag.validate_no_cycles()?;
        Ok(dag)
    }

    /// Compute execution waves using Kahn's algorithm. Each wave contains
    /// plans whose dependencies are all satisfied by earlier waves.
    pub fn compute_waves(&self) -> Vec<ExecutionWave> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for base in &self.all_plans {
            in_degree.insert(base.clone(), self.deps[base].len());
        }

        let mut waves = Vec::new();
        let mut remaining: HashSet<String> = self.all_plans.iter().cloned().collect();

        let mut wave_idx = 0;
        loop {
            let ready: Vec<String> = remaining
                .iter()
                .filter(|p| in_degree[p.as_str()] == 0)
                .cloned()
                .collect::<Vec<_>>();

            if ready.is_empty() {
                break;
            }

            let mut sorted_ready = ready;
            // Sort numeric-prefixed plans first, then letter-prefixed (R/Q/W/X/A)
            sorted_ready.sort_by(|a, b| {
                let a_digit = a.starts_with(|c: char| c.is_ascii_digit());
                let b_digit = b.starts_with(|c: char| c.is_ascii_digit());
                match (a_digit, b_digit) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.cmp(b),
                }
            });

            let est = sorted_ready
                .iter()
                .map(|p| self.estimates.get(p).copied().unwrap_or(15))
                .max()
                .unwrap_or(15);

            for plan in &sorted_ready {
                remaining.remove(plan);
                if let Some(dependents) = self.reverse_deps.get(plan) {
                    for dep in dependents {
                        if let Some(d) = in_degree.get_mut(dep.as_str()) {
                            *d -= 1;
                        }
                    }
                }
            }

            waves.push(ExecutionWave {
                index: wave_idx,
                plans: sorted_ready,
                estimated_minutes: est,
            });
            wave_idx += 1;
        }

        waves
    }

    /// Find the critical path — the longest sequential dependency chain
    /// through the graph, weighted by estimated time. Returns plan bases
    /// in execution order.
    pub fn critical_path(&self) -> Vec<String> {
        // Longest path in a DAG via dynamic programming on topological order,
        // weighted by estimated_minutes.
        let topo = self.topological_order();

        // dist[node] = total estimated minutes on the longest path ending at node
        let mut dist: HashMap<&str, u32> = HashMap::new();
        let mut pred: HashMap<&str, Option<&str>> = HashMap::new();

        for node in &topo {
            let my_time = self.estimates.get(node.as_str()).copied().unwrap_or(15);
            dist.insert(node.as_str(), my_time);
            pred.insert(node.as_str(), None);
        }

        for node in &topo {
            if let Some(dependents) = self.reverse_deps.get(node.as_str()) {
                for dep in dependents {
                    let dep_time = self.estimates.get(dep.as_str()).copied().unwrap_or(15);
                    let new_dist = dist[node.as_str()] + dep_time;
                    if new_dist > dist[dep.as_str()] {
                        dist.insert(dep.as_str(), new_dist);
                        pred.insert(dep.as_str(), Some(node.as_str()));
                    }
                }
            }
        }

        // Find the node with the maximum distance
        let end = topo
            .iter()
            .max_by_key(|n| dist[n.as_str()])
            .map(|s| s.as_str());

        let mut path = Vec::new();
        let mut cur = end;
        while let Some(node) = cur {
            path.push(node.to_string());
            cur = pred[node];
        }
        path.reverse();
        path
    }

    /// Check whether a plan can start given a set of already-completed plans.
    /// Returns true if every dependency of `plan` is in `completed`.
    pub fn can_start(&self, plan: &str, completed: &HashSet<String>) -> bool {
        match self.deps.get(plan) {
            Some(dep_set) => dep_set.iter().all(|d| completed.contains(d)),
            None => true, // unknown plan — no deps recorded, allow it
        }
    }

    /// Estimated total wall-clock minutes (sum of wave estimates)
    pub fn estimated_total_minutes(&self) -> u32 {
        self.compute_waves()
            .iter()
            .map(|w| w.estimated_minutes)
            .sum()
    }

    /// Get dependencies for a plan. Returns an empty set if the plan is unknown.
    pub fn dependencies(&self, plan: &str) -> HashSet<String> {
        self.deps.get(plan).cloned().unwrap_or_default()
    }

    /// Analyze file overlap between plans in a wave.
    /// Returns triplets of (plan_a, plan_b, shared_crates) for plans that touch the same crates.
    pub fn file_overlap_analysis(
        &self,
        wave_plans: &[String],
        plan_crates: &HashMap<String, Vec<String>>,
    ) -> Vec<(String, String, Vec<String>)> {
        let mut overlaps = Vec::new();
        for i in 0..wave_plans.len() {
            for j in (i + 1)..wave_plans.len() {
                let a = &wave_plans[i];
                let b = &wave_plans[j];
                let crates_a: HashSet<&String> = plan_crates
                    .get(a)
                    .map(|v| v.iter().collect())
                    .unwrap_or_default();
                let crates_b: HashSet<&String> = plan_crates
                    .get(b)
                    .map(|v| v.iter().collect())
                    .unwrap_or_default();
                let shared: Vec<String> = crates_a
                    .intersection(&crates_b)
                    .map(|s| (*s).clone())
                    .collect();
                if !shared.is_empty() {
                    overlaps.push((a.clone(), b.clone(), shared));
                }
            }
        }
        overlaps
    }

    /// Split a wave into sub-waves respecting a max_parallel limit.
    pub fn split_wave(&self, wave: &ExecutionWave, max_parallel: usize) -> Vec<ExecutionWave> {
        if wave.plans.len() <= max_parallel {
            return vec![wave.clone()];
        }
        wave.plans
            .chunks(max_parallel)
            .enumerate()
            .map(|(i, chunk)| ExecutionWave {
                index: wave.index * 100 + i,
                plans: chunk.to_vec(),
                estimated_minutes: chunk
                    .iter()
                    .map(|p| self.estimates.get(p).copied().unwrap_or(15))
                    .max()
                    .unwrap_or(15),
            })
            .collect()
    }

    /// Validate no cycles exist via topological sort attempt.
    fn validate_no_cycles(&self) -> Result<()> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for base in &self.all_plans {
            in_degree.insert(base.as_str(), self.deps[base].len());
        }

        let mut queue: VecDeque<&str> = VecDeque::new();
        for (&node, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(node);
            }
        }

        let mut visited = 0usize;
        while let Some(node) = queue.pop_front() {
            visited += 1;
            if let Some(dependents) = self.reverse_deps.get(node) {
                for dep in dependents {
                    if let Some(d) = in_degree.get_mut(dep.as_str()) {
                        *d -= 1;
                        if *d == 0 {
                            queue.push_back(dep.as_str());
                        }
                    }
                }
            }
        }

        if visited != self.all_plans.len() {
            let stuck: Vec<&str> = self
                .all_plans
                .iter()
                .map(|s| s.as_str())
                .filter(|s| in_degree.get(s).copied().unwrap_or(0) > 0)
                .collect();
            if let Some(first) = stuck.first() {
                bail!("Cycle detected involving plan: {first}");
            }
            bail!(
                "Plan dependency graph contains a cycle — {} plans could not be sorted",
                self.all_plans.len() - visited
            );
        }
        Ok(())
    }

    /// Topological ordering via Kahn's algorithm (deterministic: ties broken
    /// alphabetically).
    fn topological_order(&self) -> Vec<String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for base in &self.all_plans {
            in_degree.insert(base.clone(), self.deps[base].len());
        }

        // Use a sorted vec as a poor-man's priority queue for determinism
        let mut ready: Vec<String> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(k, _)| k.clone())
            .collect();
        ready.sort();

        let mut order = Vec::with_capacity(self.all_plans.len());

        while let Some(node) = ready.first().cloned() {
            ready.remove(0);
            order.push(node.clone());
            if let Some(dependents) = self.reverse_deps.get(&node) {
                let mut new_ready = Vec::new();
                for dep in dependents {
                    if let Some(d) = in_degree.get_mut(dep) {
                        *d -= 1;
                        if *d == 0 {
                            new_ready.push(dep.clone());
                        }
                    }
                }
                new_ready.sort();
                for nr in new_ready {
                    // Insert in sorted position
                    let pos = ready.binary_search(&nr).unwrap_or_else(|p| p);
                    ready.insert(pos, nr);
                }
            }
        }

        order
    }
}

/// Resolve a plan reference (could be "02" or "02-core-types") to a full base name
fn resolve_plan_name(reference: &str, all_plans: &[String]) -> Option<String> {
    // Exact match
    if all_plans.contains(&reference.to_string()) {
        return Some(reference.to_string());
    }
    // Prefix match (e.g. "02" matches "02-core-types")
    let prefix = format!("{}-", reference);
    all_plans.iter().find(|p| p.starts_with(&prefix)).cloned()
}

#[cfg(test)]
mod tests {
    use super::super::plan::{PlanFrontmatter, PlanInfo};
    use super::*;
    use std::path::PathBuf;

    fn make_plan(base: &str, depends_on: Vec<&str>) -> PlanInfo {
        make_plan_with_estimate(base, depends_on, None)
    }

    fn make_plan_with_estimate(
        base: &str,
        depends_on: Vec<&str>,
        estimated_minutes: Option<u32>,
    ) -> PlanInfo {
        PlanInfo {
            base: base.to_string(),
            num: base.split('-').next().unwrap_or(base).to_string(),
            path: PathBuf::from(format!("plans/{base}.md")),
            frontmatter: Some(PlanFrontmatter {
                plan: Some(base.to_string()),
                depends_on: depends_on.into_iter().map(String::from).collect(),
                parallel_with: Vec::new(),
                crates_touched: Vec::new(),
                estimated_tasks: None,
                estimated_parallel_width: None,
                estimated_minutes,
                refactor_after: false,
                parallel_safe: true,
                tasks: vec![],
            }),
        }
    }

    fn make_plan_no_fm(base: &str) -> PlanInfo {
        PlanInfo {
            base: base.to_string(),
            num: base.split('-').next().unwrap_or(base).to_string(),
            path: PathBuf::from(format!("plans/{base}.md")),
            frontmatter: None,
        }
    }

    #[test]
    fn test_diamond_dag() {
        // A -> B, A -> C, B -> D, C -> D
        let plans = vec![
            make_plan("01-a", vec![]),
            make_plan("02-b", vec!["01-a"]),
            make_plan("03-c", vec!["01-a"]),
            make_plan("04-d", vec!["02-b", "03-c"]),
        ];
        let dag = PlanDag::from_plans(&plans).unwrap();
        let waves = dag.compute_waves();
        assert_eq!(waves.len(), 3);
        assert_eq!(waves[0].plans, vec!["01-a"]);
        assert!(waves[1].plans.contains(&"02-b".to_string()));
        assert!(waves[1].plans.contains(&"03-c".to_string()));
        assert_eq!(waves[2].plans, vec!["04-d"]);
    }

    #[test]
    fn test_no_frontmatter_sequential() {
        let plans = vec![
            make_plan_no_fm("01-first"),
            make_plan_no_fm("02-second"),
            make_plan_no_fm("03-third"),
        ];
        let dag = PlanDag::from_plans(&plans).unwrap();
        let waves = dag.compute_waves();
        // Sequential: one plan per wave
        assert_eq!(waves.len(), 3);
        assert_eq!(waves[0].plans, vec!["01-first"]);
        assert_eq!(waves[1].plans, vec!["02-second"]);
        assert_eq!(waves[2].plans, vec!["03-third"]);
    }

    #[test]
    fn test_can_start() {
        let plans = vec![
            make_plan("01-a", vec![]),
            make_plan("02-b", vec!["01-a"]),
            make_plan("03-c", vec!["01-a", "02-b"]),
        ];
        let dag = PlanDag::from_plans(&plans).unwrap();

        let mut completed = HashSet::new();
        assert!(dag.can_start("01-a", &completed));
        assert!(!dag.can_start("02-b", &completed));

        completed.insert("01-a".to_string());
        assert!(dag.can_start("02-b", &completed));
        assert!(!dag.can_start("03-c", &completed));

        completed.insert("02-b".to_string());
        assert!(dag.can_start("03-c", &completed));
    }

    #[test]
    fn test_critical_path() {
        let plans = vec![
            make_plan("01-a", vec![]),
            make_plan("02-b", vec!["01-a"]),
            make_plan("03-c", vec!["01-a"]),
            make_plan("04-d", vec!["02-b", "03-c"]),
        ];
        let dag = PlanDag::from_plans(&plans).unwrap();
        let cp = dag.critical_path();
        // Longest chain is 3 nodes: 01-a -> 02-b -> 04-d (or 01-a -> 03-c -> 04-d)
        assert_eq!(cp.len(), 3);
        assert_eq!(cp[0], "01-a");
        assert_eq!(cp[2], "04-d");
    }

    #[test]
    fn test_all_independent() {
        let plans = vec![
            make_plan("01-a", vec![]),
            make_plan("02-b", vec![]),
            make_plan("03-c", vec![]),
        ];
        let dag = PlanDag::from_plans(&plans).unwrap();
        let waves = dag.compute_waves();
        // All independent — single wave
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0].plans.len(), 3);
    }

    #[test]
    fn test_estimated_total_minutes() {
        let plans = vec![
            make_plan_with_estimate("01-a", vec![], Some(10)),
            make_plan_with_estimate("02-b", vec!["01-a"], Some(20)),
            make_plan_with_estimate("03-c", vec!["01-a"], Some(30)),
            make_plan_with_estimate("04-d", vec!["02-b", "03-c"], Some(5)),
        ];
        let dag = PlanDag::from_plans(&plans).unwrap();
        // Wave 0: [01-a] est=10, Wave 1: [02-b, 03-c] est=max(20,30)=30, Wave 2: [04-d] est=5
        assert_eq!(dag.estimated_total_minutes(), 10 + 30 + 5);
    }

    #[test]
    fn test_dependencies() {
        let plans = vec![make_plan("01-a", vec![]), make_plan("02-b", vec!["01-a"])];
        let dag = PlanDag::from_plans(&plans).unwrap();
        let deps = dag.dependencies("02-b");
        assert!(deps.contains("01-a"));
        assert_eq!(deps.len(), 1);
        assert!(dag.dependencies("01-a").is_empty());
        // Unknown plan returns empty
        assert!(dag.dependencies("99-nonexistent").is_empty());
    }

    #[test]
    fn test_resolve_plan_name_prefix() {
        // Test that short references like "02" resolve correctly
        let plans = vec![
            make_plan("01-first", vec![]),
            make_plan("02-second", vec!["01"]), // short reference
        ];
        let dag = PlanDag::from_plans(&plans).unwrap();
        let deps = dag.dependencies("02-second");
        assert!(deps.contains("01-first"));
    }

    #[test]
    fn test_wave_estimates_default() {
        // Plans without estimated_minutes should default to 15
        let plans = vec![make_plan("01-a", vec![]), make_plan("02-b", vec![])];
        let dag = PlanDag::from_plans(&plans).unwrap();
        let waves = dag.compute_waves();
        assert_eq!(waves[0].estimated_minutes, 15);
    }
}
