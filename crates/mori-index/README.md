# mori-index

Incremental Rust source code indexer backed by SQLite. Parses `.rs` files via tree-sitter, stores symbols and cross-references in `.mori/index.db`, and supports keyword search, HDC fingerprint similarity, embedding-based semantic search, hybrid search with reciprocal rank fusion, and PageRank-ranked symbol lookup.

The only internal dependency is `bardo-primitives` (for HDC vectors). No golem dependencies. This is the storage and search layer behind `mori-context` and `mori-mcp`.

## Install

```toml
[dependencies]
mori-index = { git = "https://github.com/uniswap/bardo", path = "crates/mori-index" }

# With optional features
mori-index = { git = "https://github.com/uniswap/bardo", path = "crates/mori-index", features = ["embedding", "snapshot", "salsa-memo"] }
```

External deps: `tree-sitter`, `rusqlite`, `blake3`, `dashmap`. If you're building code search, an LSP, or an LLM-powered code tool, this gives you an index out of the box.

## Quick start

```rust
use mori_index::Index;

// Opens (or creates) .mori/index.db at the project root.
let mut index = Index::open("/path/to/project")?;

// Incremental update — only re-parses changed files.
let stats = index.update()?;
println!("{} files scanned, {} changed, {} symbols added",
    stats.files_scanned, stats.files_changed, stats.symbols_added);

// Search by name
let results = index.search("process_block", 20)?;
for r in &results {
    println!("{} {} ({}:{}) score={:.2}",
        r.symbol.kind, r.symbol.name, r.symbol.file, r.symbol.line, r.score);
}
```

For testing, `Index::open_with_db(Db::open_in_memory()?, root)` keeps the index entirely in memory.

## Incremental updates

The indexer tracks file changes at two levels:

### File-level change detection

1. Walk the project root for `.rs` files, skipping hidden dirs and `target/`
2. Blake3-hash each file's content
3. Compare against stored hashes in SQLite
4. Re-parse only files whose hash changed. Files not seen since last run are removed (cascades to symbols and refs).

### Symbol-level fingerprint caching

Within a changed file, most symbols haven't actually changed. Before clearing old symbols, the indexer loads all existing `(content_hash -> hdc_blob)` pairs into a HashMap. After re-parsing, if a symbol's signature hash matches the stored value, the old HDC fingerprint is reused instead of recomputing.

```rust
let stats = index.update()?;
println!("fingerprint cache hits: {}", stats.fingerprint_cache_hits);
// Typical edit: 1 changed function in a 50-function file = 49 cache hits
```

All updates happen in a single SQLite transaction. Blake3 hashing runs at ~1 GB/s (SIMD-accelerated), so scanning thousands of files takes milliseconds.

## Search strategies

### Keyword search

```rust
let results = index.search("AuthMiddleware", 20)?;
```

SQL `LIKE` match on symbol names. Fast, exact. Good when you know what you're looking for.

### Structural search

```rust
use mori_index::symbol::{SymbolKind, Visibility};

// All public traits
let traits = index.search_kind(SymbolKind::Trait, Some(Visibility::Public), 10)?;

// All functions (any visibility)
let fns = index.search_kind(SymbolKind::Function, None, 50)?;
```

### HDC similarity search

```rust
use bardo_primitives::HdcVector;

// Find symbols structurally similar to a fingerprint
let fp = some_symbol_fingerprint;
let similar = index.search_similar(&fp, 0.6, 10)?;
// Returns symbols above 0.6 similarity threshold
```

HDC similarity catches structural matches that keyword search misses. Two functions with the same parameter types and return shape match even if they have different names. Search runs at ~50 us for 10K symbols — all in-memory after first load.

### Hybrid search (keyword + HDC, fused via RRF)

```rust
let results = index.search_hybrid("authentication", None, 0.5, 15)?;
```

Runs both keyword and HDC searches, then merges results using Reciprocal Rank Fusion (K=60). Each result's score is `1 / (60 + rank + 1)`, summed across all lists where the symbol appears. Deduplicates by `(file, name, line)`.

With the `embedding` feature, embedding results are also fused into the hybrid merge.

### Embedding search (optional)

```rust
// Generate embeddings for un-embedded symbols (batches of 64)
let embedded = index.embed_symbols()?;
println!("embedded {embedded} new symbols");
```

Uses `fastembed` with BGE-small-en-v1.5 (384-dim, 33M params, ~50MB model download). Cosine similarity on dense vectors gives higher accuracy for natural language queries ("what handles authentication?") but costs ~3-5ms per embedding vs ~50ns for HDC.

## HDC fingerprints

Every symbol gets a 10,240-bit `HdcVector` (from `bardo-primitives`) built from its structural features:

1. **Role vector** — deterministic seed per `SymbolKind` (`b"mori:role:function"`, etc.)
2. **Name trigrams** — overlapping 3-char windows, each seeded and bundled
3. **Parameter types** — extracted from the signature (String, Vec, Option, Result, HashMap, primitives, etc.), each seeded and bundled
4. **Binding** — bundle name + params, bind with role

The fingerprint captures what a symbol *does* (structurally), not what it's *called*. Stored as 1,280-byte blobs in SQLite.

## Symbol graph and PageRank

```rust
// Build the in-memory dependency graph (lazy, skips if clean)
index.rebuild_graph()?;

// PageRank-scored symbols, biased toward specific files
let ranked = index.ranked_symbols(&[file_id_1, file_id_2], 30, 0.85)?;
for (sym_id, score) in ranked {
    println!("symbol {} score={:.4}", sym_id, score);
}
```

The graph is built from cross-reference edges stored in the `refs` table: imports, type references, function calls, trait implementations. `rebuild_graph()` is lazy — it skips the rebuild if no files changed since the last build (tracked by a `graph_dirty` flag).

PageRank with file bias finds the most "important" symbols relative to the files you care about. Top-50 symbols typically cover 80% of cross-file references.

### Transitive traversal

```rust
use mori_index::graph::Direction;

let graph = index.graph().unwrap();

// Everything that depends on symbol #42, up to 3 hops
let dependents = graph.transitive(42, 3, Direction::Reverse);

// Everything symbol #42 depends on
let deps = graph.transitive(42, 3, Direction::Forward);
```

## Symbol types

```rust
use mori_index::symbol::{Symbol, SymbolKind, Visibility, SymbolRef, RefKind};
```

`SymbolKind`: Function, Struct, Enum, Trait, TypeAlias, Const, Module, Impl, Use, Macro.

`Visibility`: Public, Crate, Restricted, Private.

`RefKind`: Import, TypeRef, Call, ImplTrait.

`Symbol` carries: fully-qualified `name`, `kind`, project-relative `file`, `line`, `signature` (declaration text up to the opening brace), `visibility`, `doc` (doc comments), and `content_hash` (Blake3 of the signature for change detection).

## Feature flags

| Feature | What it adds | Dependencies | Tradeoff |
|---------|-------------|-------------|----------|
| `embedding` | Dense vector semantic search via fastembed BGE-small-en-v1.5 | `fastembed` | ~50MB model download, ~3-5ms per embedding |
| `snapshot` | Zero-copy mmap'd read-only index snapshots | `rkyv`, `memmap2` | Instant cold start (<1ms vs ~400ms), but snapshot is read-only |
| `salsa-memo` | Incremental memoization for parsing and fingerprinting | `salsa` | Cache hit on unchanged files is <1us vs milliseconds to re-parse |

All features are composable and opt-in.

### Snapshot details

With the `snapshot` feature, after each `update()` that finds changes, the index writes a `.mori/index.snapshot.rkyv` file. On next startup, searches can read directly from the mmap'd snapshot without deserializing into heap objects.

```rust
if index.has_snapshot() {
    // Searches can bypass SQLite for reads
}
```

At ~1.5KB per symbol, a 122K-symbol codebase produces a ~180MB snapshot. The OS handles paging — only accessed symbols live in RAM.

### Salsa memoization

With `salsa-memo`, three tracked functions are memoized by file content hash:
- `parsed_symbols(file)` — tree-sitter symbol extraction
- `parsed_refs(file)` — reference extraction
- `file_fingerprints(file)` — HDC fingerprint computation

Repeat `update()` calls with no actual file changes skip parsing entirely.

## Database schema

SQLite in WAL mode with foreign keys enforced.

| Table | Purpose |
|-------|---------|
| `files` | Indexed files with Blake3 content hash and mtime |
| `symbols` | Extracted symbols with name, kind, signature, visibility, doc, HDC blob |
| `refs` | Cross-references between symbols (imports, calls, type refs, trait impls) |
| `embeddings` | Dense vector embeddings (with `embedding` feature) |

Indexes on `symbols(name)`, `symbols(file_id)`, `symbols(kind)`, `refs(from_symbol)`, `refs(to_symbol)`, `files(path)`.

`Db::open_in_memory()` is the standard test setup.

## Performance

On the bardo repo itself (4,470 files, 122K symbols, 430K references):

| Operation | Time | Notes |
|-----------|------|-------|
| Full index from scratch | ~2-3s | Tree-sitter parse + SQLite inserts |
| Incremental update (1 file changed) | ~10ms | Hash check + re-parse 1 file |
| Incremental update (no changes) | ~5ms | Hash check only |
| Keyword search | <1ms | SQLite LIKE with index |
| HDC similarity (10K symbols) | ~50us | In-memory after first load |
| Embedding search (10K symbols) | ~3-5ms | Cosine similarity on dense vectors |
| Hybrid search (keyword + HDC + embedding) | ~5-10ms | RRF merge |
| PageRank (30 iterations) | ~20ms | In-memory graph |
| Snapshot load (cold start) | <1ms | mmap, no deserialization |

## Architecture

```
src/
├── lib.rs           # Index: main entry point, public API
├── db.rs            # Db: SQLite wrapper, 100+ methods, WAL mode
├── parser.rs        # RustParser: tree-sitter extraction
├── update.rs        # Incremental update with hash-based change detection
├── search.rs        # Keyword, structural, HDC, embedding, hybrid + RRF
├── fingerprint.rs   # HDC fingerprinting: role + name trigrams + param types
├── graph.rs         # SymbolGraph: PageRank, transitive traversal
├── symbol.rs        # Symbol, SymbolKind, Visibility, SymbolRef, RefKind
├── error.rs         # IndexError
├── embedding.rs     # [embedding] fastembed integration
├── snapshot.rs      # [snapshot] rkyv + memmap2 zero-copy snapshots
└── memo.rs          # [salsa-memo] incremental memoization
```

## License

MIT/Apache-2.0
