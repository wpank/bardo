# mori-mcp

MCP server and CLI for Rust code intelligence. Exposes `mori-index` and `mori-context` as MCP tools over stdio JSON-RPC 2.0, so Claude, Cursor, and other MCP clients can search and navigate Rust codebases directly. Also includes a 9-step enrichment pipeline that generates implementation artifacts from plans.

## Install

```bash
cargo install --git https://github.com/uniswap/bardo --path crates/mori-mcp
```

Or add as a library:

```toml
[dependencies]
mori-mcp = { git = "https://github.com/uniswap/bardo", path = "crates/mori-mcp" }
```

Depends on `mori-index` and `mori-context`. Uses `clap` for the CLI, `tokio` for async I/O, and `reqwest` for gateway calls during enrichment.

## MCP server

Start the context server and wire it into any MCP client:

```bash
# Reads JSON-RPC from stdin, writes to stdout (logs to stderr)
RUST_LOG=info mori-mcp context-server --root /path/to/project
```

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "mori": {
      "command": "mori-mcp",
      "args": ["context-server", "--root", "/path/to/your/project"]
    }
  }
}
```

### Claude Code

Add to `.claude/settings.json`:

```json
{
  "mcpServers": {
    "mori": {
      "command": "mori-mcp",
      "args": ["context-server", "--root", "."]
    }
  }
}
```

On startup, the server opens the index at `{root}/.mori/index.db`, runs `update()` when the client sends `notifications/initialized`, and builds the symbol graph lazily on first graph query.

Protocol: MCP version `2024-11-05`, JSON-RPC 2.0 with Content-Length framing.

## MCP tools

10 tools available via `tools/list` and `tools/call`:

### search_code

Search the indexed codebase by symbol name with optional kind and visibility filters.

```json
{ "query": "AuthMiddleware", "kind": "struct", "visibility": "public", "limit": 20 }
```

Returns ranked symbol matches with name, file, line, kind, signature, score.

### get_symbol_context

Full details for a named symbol plus related symbols extracted from its signature.

```json
{ "symbol_name": "require_auth", "limit": 10 }
```

Returns the primary symbol's definition, docs, and up to N related types referenced in the signature.

### get_context

Rich context assembly with source snippets, related symbols, and docs. This is the main tool for LLM context injection.

```json
{ "query": "authentication", "strategy": "hybrid", "limit": 8, "context_lines": 12, "format": "markdown" }
```

Strategies: `"keyword"` (fast name match), `"similar"` (HDC fingerprint), `"hybrid"` (both fused via RRF). Output: `"json"` or `"markdown"`.

A single `get_context` call replaces 5-10 grep + file read tool calls. ~500-800 tokens of targeted context vs 8,000+ from naive file reads.

### get_file_ast

All indexed symbols from a file path (project-relative).

```json
{ "file_path": "src/auth.rs" }
```

Returns public API surface: names, lines, kinds, signatures, visibility. No implementation bodies. Useful for understanding a file's interface without reading the whole thing.

### find_similar_patterns

Symbols structurally similar to a named symbol via HDC fingerprint.

```json
{ "symbol_name": "process_block", "threshold": 0.6, "limit": 10 }
```

Catches renamed functions, cross-crate duplicates, and similar patterns across the codebase. Runs at ~50us.

### find_references

All references to a symbol: callers, importers, type users, implementors.

```json
{ "symbol_name": "AppState", "limit": 50 }
```

### find_implementations

All types implementing a given trait.

```json
{ "trait_name": "Provider" }
```

### get_callers

Transitive call/dependency graph up to N hops.

```json
{ "symbol_name": "start_server", "depth": 3, "direction": "callers" }
```

`direction`: `"callers"` traces what uses the symbol, `"callees"` traces what the symbol uses. Depth capped at 10.

### workspace_map

Workspace crate graph with inter-crate dependencies and symbol counts.

```json
{ "detail_level": "full" }
```

`"summary"` gives crate names and deps. `"full"` adds public symbol lists per crate.

### get_index_stats

File count, symbol count, reference count, resolved reference count.

```json
{}
```

## CLI commands

### Index management

```bash
# Initialize or update the code index
mori-mcp index init --root .

# Show index statistics
mori-mcp index stats --root .

# Search symbols
mori-mcp index search "MortalityClock" --root . --limit 10
```

### Enrichment pipeline

Generate implementation artifacts from plan files. Plans live at `.mori/plans/<plan-name>/plan.md` (or `plans/<plan-name>/plan.md`).

```bash
# Run all 9 enrichment steps
mori-mcp enrich all --plan 01-workspace-scaffold --root .

# Run individual steps
mori-mcp enrich briefs    --plan 01 --root .
mori-mcp enrich tasks     --plan 01 --root .
mori-mcp enrich decompose --plan 01 --root .
mori-mcp enrich verify    --plan 01 --root .
mori-mcp enrich review    --plan 01 --root .
mori-mcp enrich tests     --plan 01 --root .
mori-mcp enrich invariants --plan 01 --root .
mori-mcp enrich scribe    --plan 01 --root .
mori-mcp enrich prd       --plan 01 --root .
```

Options:

```
--gateway-url <url>   bardo-gateway URL (env: MORI_GATEWAY_URL)
--gateway-key <key>   Gateway API key (env: MORI_GATEWAY_KEY)
--batch               Use batch API (50% cost, async processing)
--model <model>       Override default model for this step
--force               Regenerate even if output already exists
--dry-run             Print what would be done without executing
```

### Enrichment steps

| Step | Output | LLM? | Default model | What it produces |
|------|--------|------|---------------|-----------------|
| `prd` | `prd-extract.md` | No | — | PRD context references extracted from plan |
| `briefs` | `brief.md` | Yes | haiku | Implementation brief: deps, imports, exports, execution order |
| `tasks` | `tasks.toml` | No | — | Task list parsed from `##` headings with file paths |
| `decompose` | `decomposition.md` | Yes | sonnet | Step-by-step instructions with file/creates/action/checkpoint |
| `verify` | `verify-tasks.toml` | Yes | sonnet | Compile gates, test tasks, lint tasks |
| `review` | `review-tasks.toml` | Yes | sonnet | Review checklists: gates, invariants, contracts, acceptance |
| `tests` | `testing-backlog.md` | Yes | sonnet | Testing backlog organized by task |
| `invariants` | `rubric.md` | Yes | haiku | Review rubric: invariant blocks, contracts, APIs |
| `scribe` | `scribe-tasks.toml` | Yes | sonnet | Documentation tasks |

Steps that don't need an LLM (prd, tasks) use pure extraction. Steps that do can call the gateway's real-time API, batch API (50% cost), or shell out to the `claude` CLI as a fallback.

### LLM backends

Three backends, selected by configuration:

1. **Claude CLI** (default, no gateway needed) — shells out to `claude --model <model> --print`
2. **Gateway real-time** — HTTP POST to `{gateway_url}/v1/messages`
3. **Gateway batch** (with `--batch`) — submits to `/v1/batch/submit`, flushes, polls for results with 5-second intervals (5-minute timeout)

```bash
# Use the gateway (cheaper with caching)
export MORI_GATEWAY_URL=http://localhost:4000
export MORI_GATEWAY_KEY=your-key
mori-mcp enrich all --plan 01 --root .

# Use batch API (50% cost, async)
mori-mcp enrich all --plan 01 --root . --batch
```

### Cost tracking and stats

```bash
# Overall cost summary
mori-mcp stats --root .

# Per-plan breakdown
mori-mcp stats --root . --by-plan

# Analyze episodes and extract patterns
mori-mcp learn --root .
```

Cost data lives at `.mori/runs/costs/summary.json`. Episode logs at `.mori/memory/episodes.jsonl` track per-task execution history: plan, role, cost, tokens, duration, gate pass/fail.

### Project initialization

```bash
mori-mcp init --root .
```

Creates `.mori/` scaffold with `config.toml`, `mcp-config.json`, `plans/`, and `runs/` directories. Appends gitignore rules to keep the index out of version control while preserving config files.

## Token reduction

Using MCP tools vs raw file reads:

| Task | Without MCP | With MCP | Reduction |
|------|------------|----------|-----------|
| Find relevant code | 5,000+ tokens (grep + reads) | ~500 tokens (`search_code`) | 10x |
| Understand a type | 8,000+ tokens (read full file) | ~800 tokens (`get_symbol_context`) | 10x |
| Check change impact | 15,000+ tokens (read dependents) | ~200 tokens (`get_callers`) | 75x |
| Full context for a task | 20,000+ tokens | ~2,000 tokens (`get_context` hybrid) | 10x |

The MCP tools give the LLM structured, relevant context instead of raw file dumps. Fewer tokens means lower cost, faster responses, and less context pollution.

## Architecture

```
src/
├── main.rs          # CLI entry point (clap), command routing
├── server.rs        # Stdio MCP server with Content-Length framing
├── protocol.rs      # JSON-RPC 2.0 types, MCP protocol structures
├── tools.rs         # 10 MCP tool definitions and dispatch
├── enrich.rs        # 9-step enrichment orchestration
├── prompts.rs       # Prompt templates for all enrichment steps
├── batch_client.rs  # bardo-gateway batch API + real-time client
├── direct_client.rs # Claude CLI invocation fallback
├── init.rs          # .mori/ directory scaffolding
└── learn.rs         # Episode analysis and pattern extraction
```

## License

MIT/Apache-2.0
