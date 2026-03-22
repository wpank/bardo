//! `mori` -- Code intelligence CLI and MCP server for Rust projects.

mod batch_client;
mod direct_client;
mod enrich;
mod init;
mod learn;
mod prompts;
mod protocol;
mod server;
mod tools;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use mori_index::Index;

use crate::enrich::{EnrichContext, EnrichStep};
use crate::server::McpServer;

/// Code intelligence for Rust projects.
#[derive(Parser)]
#[command(name = "mori", about = "Code intelligence for Rust projects")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Top-level commands.
#[derive(Subcommand)]
enum Command {
    /// Start MCP context server on stdio.
    ContextServer {
        /// Project root directory.
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Initialize mori in a project directory.
    Init {
        /// Project root directory.
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Index management.
    Index {
        #[command(subcommand)]
        action: IndexAction,
    },
    /// Analyze episodes and extract patterns from build history.
    Learn {
        /// Project root directory.
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Enrich plan artifacts (briefs, tasks, tests, invariants).
    Enrich {
        #[command(subcommand)]
        action: EnrichAction,
    },
}

/// Subcommands for `enrich`.
#[derive(Subcommand)]
enum EnrichAction {
    /// Generate implementation brief from plan.
    Briefs(EnrichArgs),
    /// Generate tasks.toml from plan.
    Tasks(EnrichArgs),
    /// Generate verify-tasks.toml (compile gates, test tasks).
    Verify(EnrichArgs),
    /// Generate review-tasks.toml (invariant, contract, acceptance checks).
    Review(EnrichArgs),
    /// Extract PRD context relevant to the plan.
    Prd(EnrichArgs),
    /// Generate step-by-step decomposition.
    Decompose(EnrichArgs),
    /// Generate testing backlog.
    Tests(EnrichArgs),
    /// Generate review rubric / invariants.
    Invariants(EnrichArgs),
    /// Generate scribe-tasks.toml (documentation tasks).
    Scribe(EnrichArgs),
    /// Run all enrichment steps in dependency order.
    All(EnrichArgs),
}

/// Shared arguments for enrichment subcommands.
#[derive(Parser)]
struct EnrichArgs {
    /// Plan name or prefix (e.g. "01-workspace-scaffold" or "01").
    #[arg(long)]
    plan: String,
    /// Project root directory.
    #[arg(long, default_value = ".")]
    root: String,
    /// Gateway URL for LLM calls (uses claude CLI if not set).
    #[arg(long, env = "MORI_GATEWAY_URL")]
    gateway_url: Option<String>,
    /// Gateway API key.
    #[arg(long, env = "MORI_GATEWAY_KEY")]
    gateway_key: Option<String>,
    /// Use batch API (50% cost, async).
    #[arg(long)]
    batch: bool,
    /// Override the default model for this step.
    #[arg(long)]
    model: Option<String>,
    /// Regenerate even if output file already exists.
    #[arg(long)]
    force: bool,
    /// Print what would be done without doing it.
    #[arg(long)]
    dry_run: bool,
}

/// Subcommands for `index`.
#[derive(Subcommand)]
enum IndexAction {
    /// Initialize or update the index.
    Init {
        /// Project root directory.
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Show index statistics.
    Stats {
        /// Project root directory.
        #[arg(long, default_value = ".")]
        root: String,
    },
    /// Search the index.
    Search {
        /// Search query.
        query: String,
        /// Project root directory.
        #[arg(long, default_value = ".")]
        root: String,
        /// Maximum number of results.
        #[arg(long, default_value = "20")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up tracing to stderr so it doesn't interfere with MCP stdio.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::ContextServer { root } => {
            let path = PathBuf::from(&root);
            let mut server = McpServer::new(&path)?;
            server.run().await?;
        }

        Command::Init { root } => {
            let path = PathBuf::from(&root);
            init::run(&path)?;
        }

        Command::Learn { root } => {
            let path = PathBuf::from(&root);
            learn::run(&path)?;
        }

        Command::Enrich { action } => {
            let (args, step) = match action {
                EnrichAction::Briefs(a) => (a, Some(EnrichStep::Briefs)),
                EnrichAction::Tasks(a) => (a, Some(EnrichStep::Tasks)),
                EnrichAction::Verify(a) => (a, Some(EnrichStep::Verify)),
                EnrichAction::Review(a) => (a, Some(EnrichStep::Review)),
                EnrichAction::Prd(a) => (a, Some(EnrichStep::Prd)),
                EnrichAction::Decompose(a) => (a, Some(EnrichStep::Decompose)),
                EnrichAction::Tests(a) => (a, Some(EnrichStep::Tests)),
                EnrichAction::Invariants(a) => (a, Some(EnrichStep::Invariants)),
                EnrichAction::Scribe(a) => (a, Some(EnrichStep::Scribe)),
                EnrichAction::All(a) => (a, None),
            };

            let ctx = EnrichContext {
                root: PathBuf::from(&args.root),
                gateway_url: args.gateway_url,
                gateway_key: args.gateway_key,
                batch_mode: args.batch,
                model_override: args.model,
                force: args.force,
                dry_run: args.dry_run,
            };

            if let Some(s) = step {
                enrich::run_step(&ctx, s, &args.plan).await?;
            } else {
                enrich::run_all(&ctx, &args.plan).await?;
            }
        }

        Command::Index { action } => match action {
            IndexAction::Init { root } => {
                let path = PathBuf::from(&root);
                let mut index = Index::open(&path)?;
                let stats = index.update()?;
                let idx_stats = index.stats()?;

                println!("Index updated.");
                println!(
                    "  Scanned: {} files, {} changed, {} added, {} removed",
                    stats.files_scanned,
                    stats.files_changed,
                    stats.files_added,
                    stats.files_removed,
                );
                println!(
                    "  Symbols added: {}, parse time: {}ms, db time: {}ms",
                    stats.symbols_added, stats.parse_time_ms, stats.db_time_ms,
                );
                println!(
                    "  Total: {} files, {} symbols, {} refs ({} resolved)",
                    idx_stats.files, idx_stats.symbols, idx_stats.refs, idx_stats.resolved_refs,
                );
            }

            IndexAction::Stats { root } => {
                let path = PathBuf::from(&root);
                let index = Index::open(&path)?;
                let stats = index.stats()?;

                println!("Index statistics:");
                println!("  Files:          {}", stats.files);
                println!("  Symbols:        {}", stats.symbols);
                println!("  References:     {}", stats.refs);
                println!("  Resolved refs:  {}", stats.resolved_refs);
            }

            IndexAction::Search { query, root, limit } => {
                let path = PathBuf::from(&root);
                let mut index = Index::open(&path)?;
                index.update()?;

                let results = index.search(&query, limit)?;

                if results.is_empty() {
                    println!("No results for \"{query}\".");
                    return Ok(());
                }

                println!(
                    "{:<40} {:<12} {:<6} {:<6} {}",
                    "NAME", "KIND", "LINE", "SCORE", "FILE"
                );
                println!("{}", "-".repeat(90));
                for r in &results {
                    println!(
                        "{:<40} {:<12} {:<6} {:<6.2} {}",
                        truncate(&r.symbol.name, 39),
                        format!("{:?}", r.symbol.kind),
                        r.symbol.line,
                        r.score,
                        r.symbol.file,
                    );
                }
                println!("\n{} result(s).", results.len());
            }
        },
    }

    Ok(())
}

/// Truncate a string to `max_len`, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let end = max_len.saturating_sub(3);
        format!("{}...", &s[..end])
    }
}
