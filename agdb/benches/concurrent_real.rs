//! Concurrent MVCC benchmark on real production databases.
//!
//! Opens copies of:
//!   - ~/evidence.bak  (957 MB, 12K nodes, 1.3M edges — EP evidence store)
//!   - ~/cidb.bak      (127 MB, 119K nodes, 154K edges — CI database)
//!
//! Compares throughput of:
//!   1. `Arc<RwLock<Db>>` — the old external-lock approach
//!   2. `Arc<Db>` — the new MVCC internal-sync approach
//!
//! Workloads model real ciapi access patterns:
//!   - Evidence: dashboard searches + pipeline evidence writes
//!   - CIDB: app lookups + build counter updates
//!
//! Run with: `cargo bench -p agdb --bench concurrent_real`

use agdb::{Db, DbId, QueryBuilder};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier, RwLock};
use std::time::{Duration, Instant};

const BENCH_DURATION: Duration = Duration::from_secs(3);
const THREAD_COUNTS: &[usize] = &[1, 2, 4, 8];

const EVIDENCE_PATH: &str = "/root/evidence.bak";
const CIDB_PATH: &str = "/root/cidb.bak";

// ---- helpers ---------------------------------------------------------------

/// Copy a database file to a temp path and open it.
fn open_copy(src: &str, label: &str, suffix: &str) -> Db {
    let dst = format!("/tmp/bench_{label}_{suffix}");
    let _ = std::fs::remove_file(&dst);
    std::fs::copy(src, &dst).unwrap_or_else(|e| panic!("copy {src} → {dst}: {e}"));
    Db::new(&dst).unwrap_or_else(|e| panic!("open {dst}: {e}"))
}

fn cleanup(label: &str, suffix: &str) {
    let dst = format!("/tmp/bench_{label}_{suffix}");
    let _ = std::fs::remove_file(&dst);
}

// ---- Evidence workloads ----------------------------------------------------
// Real pattern: dashboards search evidence by identity, pipeline writes new evidence

/// Search: find identities matching a pattern (dashboard read)
fn evidence_search_identities(db: &Db, i: u64) -> u64 {
    // Search from "identities" for nodes at distance 2 (direct children),
    // limited to avoid scanning all 12K nodes every time
    db.exec(
        QueryBuilder::search()
            .from("identities")
            .offset(i % 200)
            .limit(50)
            .where_()
            .node()
            .and()
            .distance(2)
            .query(),
    )
    .unwrap()
    .result
}

/// Search: get versions from root (dashboard read)
fn evidence_search_versions(db: &Db, _i: u64) -> u64 {
    db.exec(
        QueryBuilder::search()
            .from("versions")
            .limit(20)
            .where_()
            .node()
            .and()
            .distance(2)
            .query(),
    )
    .unwrap()
    .result
}

/// Select: read properties of a specific identity node
fn evidence_select_node(db: &Db, i: u64) -> u64 {
    // Node IDs in evidence start at ~3 (after root/identities/versions aliases)
    let node_id = DbId(4 + (i % 12000) as i64);
    db.exec(QueryBuilder::select().ids(node_id).query())
        .unwrap_or_else(|_| {
            // Node might not exist at this ID, fall back to alias
            db.exec(QueryBuilder::select().ids("root").query()).unwrap()
        })
        .result
}

/// Write: insert a new evidence node + edge (pipeline reporting)
fn evidence_write(db: &Db, i: u64) -> u64 {
    db.transaction_mut(|t| -> Result<u64, agdb::DbError> {
        let ids = t
            .exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .values([[
                        ("id", format!("bench.test.evidence.{i}")).into(),
                        ("status", "pass").into(),
                        ("score", (i % 100)).into(),
                    ]])
                    .query(),
            )?
            .ids();
        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from("root")
                .to(ids)
                .query(),
        )?;
        Ok(1)
    })
    .unwrap()
}

// ---- CIDB workloads --------------------------------------------------------
// Real pattern: lookups from apps/repos/itbas, updates to build counters

/// Search: find apps (the most common ciapi query)
fn cidb_search_apps(db: &Db, i: u64) -> u64 {
    db.exec(
        QueryBuilder::search()
            .from("apps")
            .offset(i % 400)
            .limit(50)
            .where_()
            .node()
            .and()
            .distance(2)
            .query(),
    )
    .unwrap()
    .result
}

/// Search: find repos
fn cidb_search_repos(db: &Db, i: u64) -> u64 {
    db.exec(
        QueryBuilder::search()
            .from("repos")
            .offset(i % 200)
            .limit(30)
            .where_()
            .node()
            .and()
            .distance(2)
            .query(),
    )
    .unwrap()
    .result
}

/// Select: read a specific app node's properties
fn cidb_select_app(db: &Db, i: u64) -> u64 {
    // Apps start around ID 15000 based on the probe
    let node_id = DbId(15000 + (i % 5000) as i64);
    db.exec(QueryBuilder::select().ids(node_id).query())
        .unwrap_or_else(|_| {
            db.exec(QueryBuilder::select().ids("root").query()).unwrap()
        })
        .result
}

/// Write: update a build counter (pipeline reporting to cidb)
fn cidb_write_update(db: &Db, i: u64) -> u64 {
    let node_id = DbId(15000 + (i % 5000) as i64);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("last_build_ts", i).into()]])
            .ids(node_id)
            .query(),
    )
    .unwrap_or_else(|_| {
        // Node might not exist, insert a new one instead
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .values([[
                    ("name", format!("bench/test/app-{i}")).into(),
                    ("builds", i).into(),
                ]])
                .query(),
        )
        .unwrap()
    })
    .result
}

/// Write: insert a new app node + edge (new repo onboarding)
fn cidb_write_insert(db: &Db, i: u64) -> u64 {
    db.transaction_mut(|t| -> Result<u64, agdb::DbError> {
        let ids = t
            .exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .values([[
                        ("name", format!("bench/test/new-app-{i}")).into(),
                        ("gitlab", format!("barclays/bench/app-{i}")).into(),
                        ("builds", 0_u64).into(),
                        ("deploys", 0_u64).into(),
                    ]])
                    .query(),
            )?
            .ids();
        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from("apps")
                .to(ids)
                .query(),
        )?;
        Ok(1)
    })
    .unwrap()
}

// ---- RwLock wrappers -------------------------------------------------------

fn rwlock_read<F: Fn(&Db, u64) -> u64>(db: &RwLock<Db>, i: u64, f: F) -> u64 {
    let guard = db.read().unwrap();
    f(&guard, i)
}

fn rwlock_write<F: Fn(&Db, u64) -> u64>(db: &RwLock<Db>, i: u64, f: F) -> u64 {
    let guard = db.write().unwrap();
    f(&guard, i)
}

// ---- benchmark harness -----------------------------------------------------

struct BenchResult {
    name: String,
    db_label: String,
    threads: usize,
    rwlock_ops: u64,
    mvcc_ops: u64,
    speedup: f64,
    rwlock_ms: u64,
    mvcc_ms: u64,
}

fn run_threaded<F>(threads: usize, duration: Duration, op: F) -> (u64, u64)
where
    F: Fn(u64) + Send + Sync + 'static,
{
    let op = Arc::new(op);
    let barrier = Arc::new(Barrier::new(threads + 1));
    let running = Arc::new(AtomicBool::new(true));

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let op = op.clone();
            let barrier = barrier.clone();
            let running = running.clone();
            std::thread::spawn(move || {
                let mut count = 0u64;
                let mut i = 0u64;
                barrier.wait();
                while running.load(Ordering::Relaxed) {
                    op(i);
                    i += 1;
                    count += 1;
                }
                count
            })
        })
        .collect();

    barrier.wait();
    let start = Instant::now();
    std::thread::sleep(duration);
    running.store(false, Ordering::Relaxed);

    let total: u64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    (total, start.elapsed().as_millis() as u64)
}

fn run_mixed<R, W>(
    reader_threads: usize,
    duration: Duration,
    read_op: R,
    write_op: W,
) -> (u64, u64)
where
    R: Fn(u64) + Send + Sync + 'static,
    W: Fn(u64) + Send + Sync + 'static,
{
    let read_op = Arc::new(read_op);
    let write_op = Arc::new(write_op);
    let total_threads = reader_threads + 1;
    let barrier = Arc::new(Barrier::new(total_threads + 1));
    let running = Arc::new(AtomicBool::new(true));

    let reader_handles: Vec<_> = (0..reader_threads)
        .map(|_| {
            let op = read_op.clone();
            let barrier = barrier.clone();
            let running = running.clone();
            std::thread::spawn(move || {
                let mut count = 0u64;
                let mut i = 0u64;
                barrier.wait();
                while running.load(Ordering::Relaxed) {
                    op(i);
                    i += 1;
                    count += 1;
                }
                count
            })
        })
        .collect();

    let writer_handle = {
        let op = write_op.clone();
        let barrier = barrier.clone();
        let running = running.clone();
        std::thread::spawn(move || {
            let mut count = 0u64;
            let mut i = 0u64;
            barrier.wait();
            while running.load(Ordering::Relaxed) {
                op(i);
                i += 1;
                count += 1;
            }
            count
        })
    };

    barrier.wait();
    let start = Instant::now();
    std::thread::sleep(duration);
    running.store(false, Ordering::Relaxed);

    let read_ops: u64 = reader_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let write_ops = writer_handle.join().unwrap();
    (read_ops + write_ops, start.elapsed().as_millis() as u64)
}

// ---- scenario runners ------------------------------------------------------

fn bench_evidence_mixed(reader_threads: usize) -> BenchResult {
    // RwLock: readers search identities, 1 writer inserts evidence
    let db = Arc::new(RwLock::new(open_copy(EVIDENCE_PATH, "evidence", "rwlock")));
    let (rwlock_ops, rwlock_ms) = {
        let db_r = db.clone();
        let db_w = db.clone();
        run_mixed(
            reader_threads,
            BENCH_DURATION,
            move |i| {
                if i % 3 == 0 {
                    rwlock_read(&db_r, i, evidence_search_identities);
                } else if i % 3 == 1 {
                    rwlock_read(&db_r, i, evidence_search_versions);
                } else {
                    rwlock_read(&db_r, i, evidence_select_node);
                }
            },
            move |i| {
                rwlock_write(&db_w, i, evidence_write);
            },
        )
    };
    cleanup("evidence", "rwlock");

    // MVCC: same workload, no external lock
    let db = Arc::new(open_copy(EVIDENCE_PATH, "evidence", "mvcc"));
    let (mvcc_ops, mvcc_ms) = {
        let db_r = db.clone();
        let db_w = db.clone();
        run_mixed(
            reader_threads,
            BENCH_DURATION,
            move |i| {
                if i % 3 == 0 {
                    evidence_search_identities(&db_r, i);
                } else if i % 3 == 1 {
                    evidence_search_versions(&db_r, i);
                } else {
                    evidence_select_node(&db_r, i);
                }
            },
            move |i| {
                evidence_write(&db_w, i);
            },
        )
    };
    cleanup("evidence", "mvcc");

    let speedup = mvcc_ops as f64 / rwlock_ops.max(1) as f64;
    BenchResult {
        name: "evidence_mixed".into(),
        db_label: "evidence (957 MB)".into(),
        threads: reader_threads + 1,
        rwlock_ops,
        mvcc_ops,
        speedup,
        rwlock_ms,
        mvcc_ms,
    }
}

fn bench_evidence_reads(threads: usize) -> BenchResult {
    let db = Arc::new(RwLock::new(open_copy(EVIDENCE_PATH, "evidence", "rwlock_r")));
    let (rwlock_ops, rwlock_ms) = {
        let db = db.clone();
        run_threaded(threads, BENCH_DURATION, move |i| {
            if i % 2 == 0 {
                rwlock_read(&db, i, evidence_search_identities);
            } else {
                rwlock_read(&db, i, evidence_select_node);
            }
        })
    };
    cleanup("evidence", "rwlock_r");

    let db = Arc::new(open_copy(EVIDENCE_PATH, "evidence", "mvcc_r"));
    let (mvcc_ops, mvcc_ms) = {
        let db = db.clone();
        run_threaded(threads, BENCH_DURATION, move |i| {
            if i % 2 == 0 {
                evidence_search_identities(&db, i);
            } else {
                evidence_select_node(&db, i);
            }
        })
    };
    cleanup("evidence", "mvcc_r");

    let speedup = mvcc_ops as f64 / rwlock_ops.max(1) as f64;
    BenchResult {
        name: "evidence_reads".into(),
        db_label: "evidence (957 MB)".into(),
        threads,
        rwlock_ops,
        mvcc_ops,
        speedup,
        rwlock_ms,
        mvcc_ms,
    }
}

fn bench_cidb_mixed(reader_threads: usize) -> BenchResult {
    let db = Arc::new(RwLock::new(open_copy(CIDB_PATH, "cidb", "rwlock")));
    let (rwlock_ops, rwlock_ms) = {
        let db_r = db.clone();
        let db_w = db.clone();
        run_mixed(
            reader_threads,
            BENCH_DURATION,
            move |i| {
                if i % 3 == 0 {
                    rwlock_read(&db_r, i, cidb_search_apps);
                } else if i % 3 == 1 {
                    rwlock_read(&db_r, i, cidb_search_repos);
                } else {
                    rwlock_read(&db_r, i, cidb_select_app);
                }
            },
            move |i| {
                if i % 2 == 0 {
                    rwlock_write(&db_w, i, cidb_write_update);
                } else {
                    rwlock_write(&db_w, i, cidb_write_insert);
                }
            },
        )
    };
    cleanup("cidb", "rwlock");

    let db = Arc::new(open_copy(CIDB_PATH, "cidb", "mvcc"));
    let (mvcc_ops, mvcc_ms) = {
        let db_r = db.clone();
        let db_w = db.clone();
        run_mixed(
            reader_threads,
            BENCH_DURATION,
            move |i| {
                if i % 3 == 0 {
                    cidb_search_apps(&db_r, i);
                } else if i % 3 == 1 {
                    cidb_search_repos(&db_r, i);
                } else {
                    cidb_select_app(&db_r, i);
                }
            },
            move |i| {
                if i % 2 == 0 {
                    cidb_write_update(&db_w, i);
                } else {
                    cidb_write_insert(&db_w, i);
                }
            },
        )
    };
    cleanup("cidb", "mvcc");

    let speedup = mvcc_ops as f64 / rwlock_ops.max(1) as f64;
    BenchResult {
        name: "cidb_mixed".into(),
        db_label: "cidb (127 MB)".into(),
        threads: reader_threads + 1,
        rwlock_ops,
        mvcc_ops,
        speedup,
        rwlock_ms,
        mvcc_ms,
    }
}

fn bench_cidb_reads(threads: usize) -> BenchResult {
    let db = Arc::new(RwLock::new(open_copy(CIDB_PATH, "cidb", "rwlock_r")));
    let (rwlock_ops, rwlock_ms) = {
        let db = db.clone();
        run_threaded(threads, BENCH_DURATION, move |i| {
            if i % 2 == 0 {
                rwlock_read(&db, i, cidb_search_apps);
            } else {
                rwlock_read(&db, i, cidb_select_app);
            }
        })
    };
    cleanup("cidb", "rwlock_r");

    let db = Arc::new(open_copy(CIDB_PATH, "cidb", "mvcc_r"));
    let (mvcc_ops, mvcc_ms) = {
        let db = db.clone();
        run_threaded(threads, BENCH_DURATION, move |i| {
            if i % 2 == 0 {
                cidb_search_apps(&db, i);
            } else {
                cidb_select_app(&db, i);
            }
        })
    };
    cleanup("cidb", "mvcc_r");

    let speedup = mvcc_ops as f64 / rwlock_ops.max(1) as f64;
    BenchResult {
        name: "cidb_reads".into(),
        db_label: "cidb (127 MB)".into(),
        threads,
        rwlock_ops,
        mvcc_ops,
        speedup,
        rwlock_ms,
        mvcc_ms,
    }
}

// ---- main ------------------------------------------------------------------

fn print_result(r: &BenchResult) {
    let ops_per_sec_rw = r.rwlock_ops as f64 / (r.rwlock_ms as f64 / 1000.0);
    let ops_per_sec_mv = r.mvcc_ops as f64 / (r.mvcc_ms as f64 / 1000.0);
    println!(
        "  {:>2} threads │ RwLock: {:>8} ops ({:>7.0}/s) │ MVCC: {:>8} ops ({:>7.0}/s) │ {:.2}×",
        r.threads, r.rwlock_ops, ops_per_sec_rw, r.mvcc_ops, ops_per_sec_mv, r.speedup,
    );
}

fn main() {
    // Check that the database files exist
    for (path, label) in [(EVIDENCE_PATH, "evidence"), (CIDB_PATH, "cidb")] {
        if !std::path::Path::new(path).exists() {
            eprintln!("⚠ {label} database not found at {path} — skipping");
            eprintln!("  Copy from the server: scp <host>:{label}.bak {path}");
            return;
        }
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════════╗");
    println!("║          agdb MVCC Benchmark — Real Production Databases                        ║");
    println!("║  Comparing: Arc<RwLock<Db>> (old)  vs  Arc<Db> (MVCC, new)                      ║");
    println!("║  evidence.bak: 12K nodes, 1.3M edges, 957 MB (EP evidence store)                ║");
    println!("║  cidb.bak:    119K nodes, 154K edges, 127 MB (CI database)                      ║");
    println!("║  Duration: {BENCH_DURATION:?} per measurement, FileStorageMemoryMapped                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════════╝");

    let mut results = Vec::new();

    // === Evidence database ===
    println!();
    println!("── Evidence: Pure Reads (search identities + select nodes) ────────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_evidence_reads(t);
        print_result(&r);
        results.push(r);
    }

    println!();
    println!("── Evidence: Mixed (N readers + 1 writer inserting evidence) ──────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_evidence_mixed(t);
        print_result(&r);
        results.push(r);
    }

    // === CIDB database ===
    println!();
    println!("── CIDB: Pure Reads (search apps + select app nodes) ─────────────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_cidb_reads(t);
        print_result(&r);
        results.push(r);
    }

    println!();
    println!("── CIDB: Mixed (N readers + 1 writer updating/inserting) ─────────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_cidb_mixed(t);
        print_result(&r);
        results.push(r);
    }

    // Summary
    println!();
    println!("── Summary (best speedup per category) ──────────────────────────────────────────");
    for category in &["evidence_reads", "evidence_mixed", "cidb_reads", "cidb_mixed"] {
        if let Some(best) = results
            .iter()
            .filter(|r| r.name == *category)
            .max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        {
            let label = match *category {
                "evidence_reads" => format!(
                    "Evidence reads:  {:.2}× at {} threads",
                    best.speedup, best.threads
                ),
                "evidence_mixed" => format!(
                    "Evidence mixed:  {:.2}× at {} threads ({}R + 1W)",
                    best.speedup,
                    best.threads,
                    best.threads - 1
                ),
                "cidb_reads" => format!(
                    "CIDB reads:      {:.2}× at {} threads",
                    best.speedup, best.threads
                ),
                "cidb_mixed" => format!(
                    "CIDB mixed:      {:.2}× at {} threads ({}R + 1W)",
                    best.speedup,
                    best.threads,
                    best.threads - 1
                ),
                _ => unreachable!(),
            };
            println!("  {label}");
        }
    }
    println!();
}
