# mori-mcp

MCP (Model Context Protocol) server and CLI for Rust code intelligence. Exposes `mori-index` and `mori-context` as MCP tools over stdio JSON-RPC 2.0, so Claude and other MCP clients can search and navigate the codebase directly.

The binary is `mori-mcp`.

## MCP Server

```bash
# Start the context server (reads from stdin, writes to stdout)
mori-mcp context-server --root /path/to/project

# Logs go to stderr so they don't corrupt the MCP stream
RUST_LOG=info mori-mcp context-server --root .
```

Wire it into Claude Desktop by adding to `claude_desktop_config.json`:

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

The server reads `Content-Length` framed JSON-RPC messages from stdin and writes framed responses to stdout. On startup it opens the index at `{root}/.mori/index.db`, runs `update()` on first use, and builds the symbol graph lazily.

## MCP Tools

**`search_code`** — keyword search by symbol name, with optional `kind` and `visibility` filters.

**`get_symbol_context`** — full details for a named symbol plus related symbols found in its signature.

**`get_file_ast`** — all indexed symbols from a given file path (project-relative).

**`find_similar_patterns`** — find symbols structurally similar to a named symbol via HDC fingerprint similarity. Accepts a `threshold` (0.0–1.0, default 0.6).

**`get_context`** — assemble rich context blocks with source snippets, related symbols, and docs. Supports `strategy: "keyword" | "similar" | "hybrid"` and returns JSON or markdown.

**`find_references`** — all references to a symbol: callers, importers, type users, implementors.

**`find_implementations`** — all types that implement a given trait.

**`get_callers`** — transitive call/dependency graph up to `depth` hops. `direction: "callers"` traces what uses the symbol; `"callees"` traces what the symbol uses.

**`workspace_map`** — workspace crate graph with inter-crate dependencies and symbol counts per crate. `detail_level: "full"` adds public symbol lists.

**`get_index_stats`** — file count, symbol count, reference count, resolved reference count.

## CLI Commands

```bash
# Initialize the index (parse all .rs files)
mori-mcp index init --root .

# Show index stats
mori-mcp index stats --root .

# Search
mori-mcp index search "MortalityClock" --root . --limit 10

# Analyze episodes and extract patterns from build history
mori-mcp learn --root .

# Enrich plan artifacts — generates implementation briefs, tasks, tests, etc.
# Plans live at .mori/plans/<plan-name>/
mori-mcp enrich briefs --plan 01-workspace-scaffold --root .
mori-mcp enrich tasks  --plan 01-workspace-scaffold --root .
mori-mcp enrich tests  --plan 01-workspace-scaffold --root .
mori-mcp enrich all    --plan 01-workspace-scaffold --root .

# Show cost and usage statistics
mori-mcp stats --root .
mori-mcp stats --root . --by-plan
```

## Enrichment

The `enrich` subcommands call an LLM to generate plan artifacts. Configure the gateway endpoint and key via flags or environment variables:

```bash
export MORI_GATEWAY_URL=https://your-gateway/v1
export MORI_GATEWAY_KEY=your-key

mori-mcp enrich all --plan 01 --batch  # 50% cost with async batch API
mori-mcp enrich briefs --plan 01 --force  # regenerate even if output exists
mori-mcp enrich tasks  --plan 01 --dry-run  # print what would be done
```

Supported steps: `briefs`, `tasks`, `verify`, `review`, `prd`, `decompose`, `tests`, `invariants`, `scribe`, `all`.

## Cost Tracking

Cost data is written to `.mori/runs/costs/summary.json`. `mori-mcp stats --by-plan` shows per-plan cost and iteration counts. Episode logs at `.mori/memory/episodes.jsonl` count total episodes logged.

## Dependencies

```toml
[dependencies]
mori-mcp = { path = "../../crates/mori-mcp" }
```

Or just install the binary and use it as a tool. Depends on `mori-index` and `mori-context`. Uses `clap` for the CLI, `tokio` for the async server loop, and `reqwest` for LLM gateway calls during enrichment.
