//! Benchmarks for mori-index.
//!
//! Tests fingerprint computation vs cache hits, HDC/keyword search at varying
//! symbol counts, snapshot-backed variants, and graph rebuild cost.

#![allow(missing_docs)]

use std::collections::HashMap;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use bardo_primitives::HdcVector;
use mori_index::db::Db;
use mori_index::symbol::{Symbol, SymbolKind, Visibility};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn synthetic_symbol(idx: usize) -> Symbol {
    Symbol {
        name: format!("symbol_{idx}"),
        kind: match idx % 4 {
            0 => SymbolKind::Function,
            1 => SymbolKind::Struct,
            2 => SymbolKind::Enum,
            _ => SymbolKind::Trait,
        },
        file: format!("file_{}.rs", idx / 20),
        line: (idx % 100) as u32 + 1,
        signature: format!(
            "fn symbol_{idx}(arg{}: u32, arg{}: String) -> Result<Vec<u8>>",
            idx % 5,
            (idx + 1) % 5
        ),
        visibility: Visibility::Public,
        doc: Some(format!("Documentation for symbol {idx}")),
        content_hash: blake3::hash(format!("sig-{idx}").as_bytes()).into(),
    }
}

/// Populate a test DB with `n` symbols, each with an HDC fingerprint.
/// Returns the symbol row ids.
fn populate_db(db: &Db, n: usize) -> Vec<i64> {
    let hash = blake3::hash(b"x").as_bytes().to_vec();
    let mut all_ids = Vec::new();
    let files_needed = (n / 20).max(1);
    for f in 0..files_needed {
        let file_id = db.upsert_file(&format!("file_{f}.rs"), &hash, 0).unwrap();
        let start = f * 20;
        let end = (start + 20).min(n);
        let syms: Vec<Symbol> = (start..end).map(synthetic_symbol).collect();
        let ids = db.insert_symbols(file_id, &syms).unwrap();
        for (sym, &id) in syms.iter().zip(ids.iter()) {
            let fp = mori_index::fingerprint::fingerprint(sym);
            db.set_hdc(id, &fp).unwrap();
        }
        all_ids.extend(ids);
    }
    all_ids
}

/// Create a fresh in-memory DB with migrations applied.
fn fresh_db() -> Db {
    let db = Db::open_in_memory().unwrap();
    db.migrate().unwrap();
    db
}

/// Insert synthetic cross-references so the graph has edges.
/// Each symbol references the next two symbols (wrapping), giving 2*n edges.
fn populate_refs(db: &Db, ids: &[i64]) {
    use mori_index::symbol::{RefKind, SymbolRef};
    let n = ids.len();
    if n < 2 {
        return;
    }
    for (i, &from_id) in ids.iter().enumerate() {
        let refs = vec![
            SymbolRef {
                from_file: format!("file_{}.rs", i / 20),
                from_line: (i % 100) as u32 + 1,
                target: format!("symbol_{}", (i + 1) % n),
                ref_kind: RefKind::Call,
            },
            SymbolRef {
                from_file: format!("file_{}.rs", i / 20),
                from_line: (i % 100) as u32 + 2,
                target: format!("symbol_{}", (i + 2) % n),
                ref_kind: RefKind::TypeRef,
            },
        ];
        db.insert_refs(&refs, &[from_id, from_id]).unwrap();
    }
    // Resolve the refs so to_symbol gets populated.
    db.resolve_refs().unwrap();
}

// ---------------------------------------------------------------------------
// Fingerprint benchmarks
// ---------------------------------------------------------------------------

fn bench_fingerprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint");

    let sym = synthetic_symbol(42);

    group.bench_function("compute", |b| {
        b.iter(|| mori_index::fingerprint::fingerprint(&sym));
    });

    // Simulate a cache hit: pre-compute the fingerprint, then measure HashMap
    // lookup cost (keyed by content_hash).
    let fp = mori_index::fingerprint::fingerprint(&sym);
    let mut cache: HashMap<[u8; 32], HdcVector> = HashMap::new();
    cache.insert(sym.content_hash, fp);

    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            let _ = cache.get(&sym.content_hash).unwrap();
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// HDC search benchmarks
// ---------------------------------------------------------------------------

fn bench_hdc_search_sqlite(c: &mut Criterion) {
    let mut group = c.benchmark_group("hdc_search/sqlite");

    for &n in &[100, 1000, 5000] {
        let db = fresh_db();
        populate_db(&db, n);
        let query_fp = mori_index::fingerprint::fingerprint(&synthetic_symbol(0));

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                mori_index::search::search_similar(&db, &query_fp, 0.5, 10).unwrap();
            });
        });
    }

    group.finish();
}

#[cfg(feature = "snapshot")]
fn bench_hdc_search_snapshot(c: &mut Criterion) {
    use mori_index::snapshot;

    let mut group = c.benchmark_group("hdc_search/snapshot");

    for &n in &[100, 1000, 5000] {
        let db = fresh_db();
        populate_db(&db, n);
        let query_fp = mori_index::fingerprint::fingerprint(&synthetic_symbol(0));

        let tmp_dir = std::env::temp_dir().join(format!("mori-bench-hdc-snap-{n}"));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let snap_path = tmp_dir.join("bench.snapshot.rkyv");
        snapshot::write_snapshot(&db, &snap_path).unwrap();
        let snap = snapshot::MmapSnapshot::open(&snap_path).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let _ = snap.search_similar(&query_fp, 0.5, 10);
            });
        });

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Keyword search benchmarks
// ---------------------------------------------------------------------------

fn bench_keyword_search_sqlite(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyword_search/sqlite");

    for &n in &[100, 1000, 5000] {
        let db = fresh_db();
        populate_db(&db, n);

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                mori_index::search::search_keyword(&db, "symbol_4", 10).unwrap();
            });
        });
    }

    group.finish();
}

#[cfg(feature = "snapshot")]
fn bench_keyword_search_snapshot(c: &mut Criterion) {
    use mori_index::snapshot;

    let mut group = c.benchmark_group("keyword_search/snapshot");

    for &n in &[100, 1000, 5000] {
        let db = fresh_db();
        populate_db(&db, n);

        let tmp_dir = std::env::temp_dir().join(format!("mori-bench-kw-snap-{n}"));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let snap_path = tmp_dir.join("bench.snapshot.rkyv");
        snapshot::write_snapshot(&db, &snap_path).unwrap();
        let snap = snapshot::MmapSnapshot::open(&snap_path).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let _ = snap.search_by_name("symbol_4", 10);
            });
        });

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Graph rebuild benchmark
// ---------------------------------------------------------------------------

fn bench_graph_rebuild(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_rebuild");

    // Use 1000 symbols with 2000 edges (each symbol -> next two).
    let db = fresh_db();
    let ids = populate_db(&db, 1000);
    populate_refs(&db, &ids);

    group.bench_function("full", |b| {
        b.iter(|| {
            let edges = db.load_graph_edges().unwrap();
            let node_count = db.symbol_count().unwrap();
            let _ = mori_index::graph::SymbolGraph::from_edges(&edges, node_count);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_fingerprint,
    bench_hdc_search_sqlite,
    bench_keyword_search_sqlite,
    bench_graph_rebuild,
);

#[cfg(feature = "snapshot")]
criterion_group!(
    snapshot_benches,
    bench_hdc_search_snapshot,
    bench_keyword_search_snapshot,
);

#[cfg(feature = "snapshot")]
criterion_main!(benches, snapshot_benches);

#[cfg(not(feature = "snapshot"))]
criterion_main!(benches);
