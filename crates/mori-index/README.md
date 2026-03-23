# mori-index

Incremental Rust source code indexer backed by SQLite. Parses `.rs` files via tree-sitter, stores symbols and cross-references in `.mori/index.db`, and supports keyword search, HDC fingerprint similarity, and PageRank-ranked symbol lookup.

The index is designed for code intelligence use cases: fast incremental updates on large Rust workspaces, structural similarity between functions and types, and reference graph traversal. It is also the storage layer behind `mori-context` and `mori-mcp`.

## Opening an Index

```rust
use mori_index::Index;

// Opens (or creates) .mori/index.db at the project root.
// Runs schema migrations automatically.
let mut index = Index::open("/path/to/project")?;
```

`Index::open_with_db(db, root)` accepts a pre-configured `Db` for testing — pass `Db::open_in_memory()` to keep the index entirely in memory.

## Incremental Updates

```rust
let stats = index.update()?;
println!(
    "scanned={} changed={} added={} removed={} symbols_added={} parse={}ms db={}ms",
    stats.files_scanned,
    stats.files_changed,
    stats.files_added,
    stats.files_removed,
    stats.symbols_added,
    stats.parse_time_ms,
    stats.db_time_ms,
);
```

`update()` walks the project root for `.rs` files, computes Blake3 hashes, and re-parses only files whose content changed. Files not seen since last run are removed from the index. `fingerprint_cache_hits` in `UpdateStats` counts symbols whose signature hash matched the stored value, skipping re-fingerprinting.

## Searching

```rust
// Keyword: LIKE match on symbol names
let results = index.search("process_block", 20)?;

// By kind and visibility
use mori_index::symbol::{SymbolKind, Visibility};
let traits = index.search_kind(SymbolKind::Trait, Some(Visibility::Public), 10)?;

// HDC similarity: find structurally similar symbols
use bardo_primitives::HdcVector;
let fp = HdcVector::from_seed(b"some-seed");
let similar = index.search_similar(&fp, 0.6, 10)?;

// Hybrid: keyword + HDC fused via Reciprocal Rank Fusion
let hybrid = index.search_hybrid("authentication", None, 0.5, 15)?;
```

`SearchResult` carries `symbol: Symbol`, `score: f32`, and `match_kind: MatchKind`. `MatchKind` is `Keyword`, `Structural`, `Similarity { similarity }`, or `Hybrid { sources }`.

`Symbol` has `name` (fully qualified, e.g. `golem_core::id::GolemId`), `kind: SymbolKind`, `file` (project-relative path), `line`, `signature`, `visibility`, and `doc`.

`SymbolKind` covers: `Function`, `Struct`, `Enum`, `Trait`, `TypeAlias`, `Const`, `Module`, `Impl`, `Use`, `Macro`.

## HDC Fingerprints

Each symbol gets a 10,240-bit `HdcVector` (from `bardo_primitives`) built from its kind, name trigrams, and parameter types. Similarity search uses Hamming distance. Fingerprints are stored as blobs in SQLite and skipped if the signature hash hasn't changed since last index.

## PageRank

```rust
// Build the in-memory symbol graph first (required before ranked_symbols)
index.rebuild_graph()?;

// Get symbols ranked by cross-file reference frequency,
// biased toward files in the given list
let ranked = index.ranked_symbols(&[file_id_1, file_id_2], 30, 0.85)?;
// returns Vec<(symbol_id: i64, pagerank_score: f32)>
```

`SymbolGraph` is built from cross-reference edges stored in the database. `rebuild_graph()` is lazy — it skips the rebuild if the graph is clean. `graph_dirty` is set to `true` on any `update()` call that found changes.

## Features

**`embedding`** — adds semantic search via `fastembed`. Call `index.embed_symbols()` to generate vectors for un-embedded symbols in batches of 64. Pass an `EmbeddingStore` to `search_hybrid` to include embedding results in the fusion.

**`salsa-memo`** — Salsa incremental memoization. Re-analysis of unchanged files hits the memoization cache rather than re-running tree-sitter parsing.

**`snapshot`** — mmap'd read-only snapshot at `.mori/index.snapshot.rkyv`. Written automatically after each `update()` that finds changes. Reads are faster than heap-deserialized SQLite queries on cold start. `index.has_snapshot()` returns whether one is loaded.

```toml
[dependencies]
mori-index = { path = "../../crates/mori-index" }
# Optional features:
mori-index = { path = "../../crates/mori-index", features = ["salsa-memo", "snapshot"] }
```

## Database

`Db` wraps a `rusqlite::Connection` opened in WAL mode with `foreign_keys=ON`. Schema covers `files`, `symbols`, `refs`, and `fingerprints` tables. `Db::open_in_memory()` is the standard test setup. `IndexStats` reports file count, symbol count, ref count, and resolved ref count.
