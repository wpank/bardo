# mori-context

Assembles `mori-index` search results into structured context blocks for LLM consumption. Takes a `ContextQuery`, searches the index, pulls source snippets, resolves related symbols referenced in signatures, and returns a `ContextResponse` you can inject directly into a prompt.

## Core Types

**`ContextQuery`** — what to search for and how:

```rust
use mori_context::{ContextQuery, SearchStrategy};

// Keyword search (default)
let q = ContextQuery::keyword("process_block", 10);

// Hybrid: keyword + HDC fingerprint similarity, fused via RRF
let q = ContextQuery::hybrid("authentication middleware", 15, 0.5);

// Full control
let q = ContextQuery {
    query: "MortalityClock".into(),
    strategy: SearchStrategy::Similar { threshold: 0.6 },
    limit: 10,
    context_lines: 15,   // source lines around each symbol
    include_related: true,
};
```

`SearchStrategy` variants: `Keyword` (name LIKE match), `Similar { threshold }` (HDC Hamming distance), `Hybrid { threshold }` (both, fused).

**`ContextBlock`** — one result:

```rust
pub struct ContextBlock {
    pub name: String,
    pub kind: String,         // "Function", "Struct", etc.
    pub file: String,         // project-relative path
    pub line: u32,
    pub signature: String,
    pub doc: Option<String>,
    pub snippet: Option<Snippet>,    // source lines around the symbol
    pub related: Vec<RelatedSymbol>, // types referenced in the signature
    pub score: f32,
}
```

**`ContextResponse`** — the full result set:

```rust
pub struct ContextResponse {
    pub query: String,
    pub blocks: Vec<ContextBlock>,  // ordered by relevance
    pub total_candidates: usize,    // total symbols considered before limit
}
```

`ContextResponse::to_markdown()` renders the blocks as a markdown string suitable for direct injection into an LLM prompt.

## Usage

```rust
use mori_context::{assemble, ContextQuery, SearchStrategy};
use mori_index::Index;

let index = Index::open("/path/to/project")?;

let query = ContextQuery {
    query: "tick cost economic".into(),
    strategy: SearchStrategy::Hybrid { threshold: 0.5 },
    limit: 8,
    context_lines: 12,
    include_related: true,
};

let response = assemble(&index, query)?;

for block in &response.blocks {
    println!("{} {} ({}:{}) score={:.2}",
        block.kind, block.name, block.file, block.line, block.score);
}

// Or get a markdown blob for LLM injection:
let md = response.to_markdown();
```

## `SnippetConfig`

Controls how source snippets are extracted. Configured via `ContextQuery::context_lines` (lines of context around the symbol). `Snippet` carries `start_line`, `end_line`, and `content`.

## Errors

`ContextError` wraps `IndexError` from `mori-index` and I/O errors from reading source files. The `assemble` function degrades gracefully — if a source file can't be read, the block is still returned without a snippet.

## Dependencies

```toml
[dependencies]
mori-context = { path = "../../crates/mori-context" }
```

Depends on `mori-index`. No feature flags of its own — the `embedding` and `snapshot` features propagate from `mori-index` if you enable them there.
