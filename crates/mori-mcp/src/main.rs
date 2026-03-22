//! `mori` -- Code intelligence CLI and MCP server for Rust projects.

mod protocol;
mod server;
mod tools;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use mori_index::Index;

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
    /// Index management.
    Index {
        #[command(subcommand)]
        action: IndexAction,
    },
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
