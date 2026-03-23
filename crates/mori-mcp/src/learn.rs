//! `mori learn` — read episodes, aggregate stats, and surface patterns.

use std::collections::HashMap;
use std::path::Path;

/// Run the learn command: read `.mori/memory/episodes.jsonl`, group by plan,
/// and print summary statistics. Pattern extraction via LLM is a future step;
/// this gives operators immediate visibility into episode data.
pub fn run(root: &Path) -> anyhow::Result<()> {
    let episodes_path = root.join(".mori/memory/episodes.jsonl");
    if !episodes_path.exists() {
        println!(
            "No episodes found at {}. Run some plans first.",
            episodes_path.display()
        );
        return Ok(());
    }

    let content = std::fs::read_to_string(&episodes_path)?;
    let episodes: Vec<serde_json::Value> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    println!("Found {} episodes.", episodes.len());

    if episodes.is_empty() {
        return Ok(());
    }

    // Group by plan_id
    let mut by_plan: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();
    for ep in &episodes {
        if let Some(plan) = ep.get("plan_id").and_then(|v| v.as_str()) {
            by_plan.entry(plan.to_string()).or_default().push(ep);
        }
    }

    println!("\nEpisodes by plan:");
    let mut plans: Vec<_> = by_plan.iter().collect();
    plans.sort_by_key(|(_, eps)| std::cmp::Reverse(eps.len()));
    for (plan, eps) in &plans {
        let total_cost: f64 = eps
            .iter()
            .filter_map(|e| e.get("cost_usd").and_then(|v| v.as_f64()))
            .sum();
        let gate_fails = eps
            .iter()
            .filter(|e| e.get("gate_passed").and_then(|v| v.as_bool()) == Some(false))
            .count();
        let total_duration: u64 = eps
            .iter()
            .filter_map(|e| e.get("duration_secs").and_then(|v| v.as_u64()))
            .sum();
        let avg_duration = if eps.is_empty() {
            0
        } else {
            total_duration / eps.len() as u64
        };
        println!(
            "  {plan}: {} episodes, ${total_cost:.2}, {gate_fails} gate failures, avg {avg_duration}s/task",
            eps.len()
        );
    }

    // Group by role
    let mut by_role: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();
    for ep in &episodes {
        if let Some(role) = ep.get("role").and_then(|v| v.as_str()) {
            by_role.entry(role.to_string()).or_default().push(ep);
        }
    }

    println!("\nEpisodes by role:");
    let mut roles: Vec<_> = by_role.iter().collect();
    roles.sort_by_key(|(_, eps)| std::cmp::Reverse(eps.len()));
    for (role, eps) in &roles {
        let total_cost: f64 = eps
            .iter()
            .filter_map(|e| e.get("cost_usd").and_then(|v| v.as_f64()))
            .sum();
        let total_tokens: u64 = eps
            .iter()
            .filter_map(|e| {
                let inp = e.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let out = e.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                Some(inp + out)
            })
            .sum();
        println!(
            "  {role}: {} episodes, ${total_cost:.2}, {total_tokens} total tokens",
            eps.len()
        );
    }

    // Overall summary
    let total_cost: f64 = episodes
        .iter()
        .filter_map(|e| e.get("cost_usd").and_then(|v| v.as_f64()))
        .sum();
    let total_input: u64 = episodes
        .iter()
        .filter_map(|e| e.get("input_tokens").and_then(|v| v.as_u64()))
        .sum();
    let total_output: u64 = episodes
        .iter()
        .filter_map(|e| e.get("output_tokens").and_then(|v| v.as_u64()))
        .sum();

    println!("\nTotals:");
    println!("  Cost:          ${total_cost:.2}");
    println!("  Input tokens:  {total_input}");
    println!("  Output tokens: {total_output}");
    println!("  Plans:         {}", by_plan.len());
    println!("  Episodes:      {}", episodes.len());

    println!("\nPattern extraction requires LLM analysis (not yet wired).");
    println!("Episodes are logged and ready for analysis.");

    Ok(())
}
