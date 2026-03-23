# mori-context

Assembles `mori-index` search results into structured context blocks for LLM consumption. Takes a `ContextQuery`, searches the index, pulls source snippets, resolves related symbols found in signatures, and returns a `ContextResponse` you can inject directly into a prompt as markdown or JSON.

## Install

```toml
[dependencies]
mori-context = { git = "https://github.com/uniswap/bardo", path = "crates/mori-context" }
```

Depends on `mori-index` (which depends on `bardo-primitives`). No feature flags of its own — the `embedding` and `snapshot` features propagate from `mori-index` if you enable them.

If you're building an LLM-powered code assistant and need to turn a search query into a prompt-ready context block with source snippets and related types, this handles the pipeline.

## Quick start

```rust
use mori_context::{assemble, ContextQuery};
use mori_index::Index;

let mut index = Index::open("/path/to/project")?;
index.update()?;

let query = ContextQuery::keyword("process_block", 10);
let response = assemble(&mut index, &query)?;

// Inject directly into an LLM prompt
let markdown = response.to_markdown();

// Or get structured JSON
let json = response.to_json();
```

## Search strategies

Three ways to search, controlled by `ContextQuery`:

```rust
use mori_context::{ContextQuery, SearchStrategy};

// Keyword: fast name-based LIKE search
let q = ContextQuery::keyword("AuthMiddleware", 10);

// Hybrid: keyword + HDC fingerprint similarity, fused via RRF
let q = ContextQuery::hybrid("authentication middleware", 15, 0.5);

// Full control
let q = ContextQuery {
    query: "MortalityClock".into(),
    strategy: SearchStrategy::Similar { threshold: 0.6 },
    limit: 10,
    context_lines: 15,
    include_related: true,
};
```

`SearchStrategy::Keyword` hits the SQLite index directly. `SearchStrategy::Similar` searches by HDC fingerprint. `SearchStrategy::Hybrid` runs both and merges via Reciprocal Rank Fusion (K=60).

## The assembly pipeline

`assemble()` runs a five-step pipeline:

1. **Search** — dispatches to `mori-index` using the query's strategy. Internally fetches `limit * 3` candidates to have room after deduplication.

2. **Deduplication** — tracks seen `(file, line)` pairs to eliminate duplicates from multi-strategy merges.

3. **Snippet extraction** — reads source files from disk, extracts a window of context lines around each symbol. The window is biased toward code after the symbol: `n/3` lines before, `n - n/3` lines after.

4. **Related symbol resolution** — parses each result's signature for capitalized identifiers (likely type names), searches the index for each, returns up to 5 related symbols per block. This answers "what types does this function use?" without a separate query.

5. **Assembly** — packs everything into `ContextBlock` structs ordered by search relevance.

The pipeline degrades gracefully. If a source file can't be read, the block is still returned without a snippet. If no initial symbol is found for similarity search, you get a `ContextError::NoResults`.

## Output formats

### Markdown (for LLM prompts)

```rust
let md = response.to_markdown();
```

Produces:

````markdown
# Code Context: authentication middleware

## Function `require_auth` (src/auth.rs:42)
```rust
pub async fn require_auth(state: &AppState, headers: &HeaderMap) -> Result<(), AuthError>
```

Validates x-api-key or Bearer token against configured gateway key.

### Source (lines 38-55)
```rust
// ... extracted source code ...
```

### Related
- Struct `AppState` (src/state.rs:12): `pub struct AppState { ... }`
- Enum `AuthError` (src/error.rs:8): `pub enum AuthError { ... }`
````

This format is tested with Claude and produces good results for code understanding tasks. The `related` section gives the model type context without requiring separate file reads.

### JSON (for programmatic use)

```rust
let json = response.to_json();
```

Returns pretty-printed JSON via `serde_json`. All types implement `Serialize`.

## Token efficiency

The context pipeline is designed to minimize tokens sent to the LLM:

- **Snippets over full files** — a 15-line window around a symbol is ~200 tokens vs 2,000+ for the full file
- **Signatures over implementations** — the signature captures the contract; implementation details come from the snippet
- **Related symbols** — 5 related types at ~50 tokens each replaces reading 5 separate files at 2,000+ tokens each
- **Ranked results** — RRF fusion surfaces the most relevant symbols first, so you can use smaller limits

For a typical code understanding task, `mori-context` delivers ~500-800 tokens of targeted context vs 8,000+ tokens from naive file reads. That's a 10-16x reduction.

## Core types

### ContextBlock

```rust
pub struct ContextBlock {
    pub name: String,         // symbol name
    pub kind: String,         // "Function", "Struct", "Trait", etc.
    pub file: String,         // project-relative path
    pub line: u32,            // line number
    pub signature: String,    // declaration text
    pub doc: Option<String>,  // doc comments
    pub snippet: Option<Snippet>,    // source context window
    pub related: Vec<RelatedSymbol>, // types from the signature
    pub score: f32,           // search relevance
}
```

### ContextResponse

```rust
pub struct ContextResponse {
    pub query: String,              // original query text
    pub blocks: Vec<ContextBlock>,  // ordered by relevance
    pub total_candidates: usize,    // total symbols considered before limit
}
```

### Snippet

```rust
pub struct Snippet {
    pub code: String,       // extracted source lines
    pub start_line: u32,    // first line (1-indexed)
    pub end_line: u32,      // last line (1-indexed)
}
```

### SnippetConfig

Controls the extraction window. Default: 5 lines before, 10 lines after. `SnippetConfig::from_context_lines(n)` splits `n` lines into `n/3` before and `n - n/3` after (biased toward showing code after the symbol, where the implementation lives).

## Error handling

`ContextError` wraps `IndexError` from `mori-index` and I/O errors from reading source files.

`assemble()` never fails because a single snippet can't be read — that block just has `snippet: None`. The response is always as complete as the available data allows.

## Use cases

- **MCP context server** — `mori-mcp` uses this crate behind the `get_context` tool, serving structured code context to Claude, Cursor, and other MCP clients
- **Agent context injection** — the build orchestrator uses `assemble()` to prepare task-specific context for implementer agents, reducing their token footprint by 10-16x
- **Code review** — reviewer agents get targeted context about the symbols under review plus their dependents, without reading entire files
- **IDE integration** — embed context assembly in an LSP or editor plugin to provide AI-powered code understanding

## Architecture

```
src/
├── lib.rs       # re-exports public API
├── assemble.rs  # assemble() pipeline, ContextBlock, ContextResponse, RRF merge
├── query.rs     # ContextQuery, SearchStrategy
├── snippet.rs   # SnippetConfig, Snippet, extract_snippet()
└── error.rs     # ContextError
```

## License

MIT/Apache-2.0
