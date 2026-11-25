//! Concurrent MVCC benchmark for agdb.
//!
//! Compares throughput of:
//!   1. `Arc<RwLock<DbMemory>>` — the old external-lock approach
//!   2. `Arc<DbMemory>` — the new MVCC internal-sync approach
//!
//! Workloads:
//!   - Pure reads (N reader threads)
//!   - Mixed read/write (N readers + 1 writer)
//!   - Write-heavy (N writer threads)
//!
//! Run with: `cargo bench -p agdb --bench concurrent`

use agdb::{DbMemory, QueryBuilder};
use std::sync::{Arc, Barrier, RwLock};
use std::time::Duration;

const WARM_UP_NODES: u64 = 10_000;
const BENCH_DURATION: Duration = Duration::from_secs(2);
const THREAD_COUNTS: &[usize] = &[1, 2, 4, 8];

// ---- helpers ---------------------------------------------------------------

fn setup_db() -> DbMemory {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["root", "users"])
            .query(),
    )
    .unwrap();

    // Insert nodes so reads have data to query
    db.transaction_mut(|t| -> Result<(), agdb::DbError> {
        let values: Vec<Vec<agdb::DbKeyValue>> = (0..WARM_UP_NODES)
            .map(|i| vec![("name", format!("user_{i}")).into(), ("score", i).into()])
            .collect();
        let ids = t
            .exec_mut(QueryBuilder::insert().nodes().values(values).query())?
            .ids();
        t.exec_mut(QueryBuilder::insert().edges().from("users").to(ids).query())?;
        Ok(())
    })
    .unwrap();

    // Create an index for fast lookups
    db.exec_mut(QueryBuilder::insert().index("score").query())
        .unwrap();

    db
}

// ---- RwLock baseline -------------------------------------------------------

fn rwlock_write(db: &RwLock<DbMemory>, i: u64) -> u64 {
    // Use write() — the old API required &mut self for exec_mut, so
    // the external RwLock needed exclusive access.  This is what MVCC
    // replaces; read() here would benchmark "MVCC + useless lock" vs
    // "MVCC", hiding the actual improvement.
    #[allow(clippy::readonly_write_lock)]
    let guard = db.write().unwrap();
    guard
        .exec_mut(
            QueryBuilder::insert()
                .nodes()
                .values([[("name", format!("new_{i}")).into(), ("score", i).into()]])
                .query(),
        )
        .unwrap()
        .result
}

/// Heavier read: search with condition scan (holds lock longer, shows contention)
fn rwlock_heavy_read(db: &RwLock<DbMemory>, i: u64) -> u64 {
    let guard = db.read().unwrap();
    guard
        .exec(
            QueryBuilder::search()
                .from("users")
                .where_()
                .node()
                .and()
                .key("score")
                .value(agdb::Comparison::LessThan((i % 100 + 1).into()))
                .query(),
        )
        .unwrap()
        .result
}

/// Transaction read: multiple queries in a transaction (even longer lock hold)
fn rwlock_txn_read(db: &RwLock<DbMemory>, i: u64) -> u64 {
    let guard = db.read().unwrap();
    guard
        .transaction(|t| -> Result<u64, agdb::DbError> {
            let r1 = t
                .exec(
                    QueryBuilder::search()
                        .index("score")
                        .value(i % WARM_UP_NODES)
                        .query(),
                )?
                .result;
            let r2 = t.exec(QueryBuilder::select().ids("root").query())?.result;
            Ok(r1 + r2)
        })
        .unwrap()
}

// ---- MVCC (new API) --------------------------------------------------------

fn mvcc_heavy_read(db: &DbMemory, i: u64) -> u64 {
    db.exec(
        QueryBuilder::search()
            .from("users")
            .where_()
            .node()
            .and()
            .key("score")
            .value(agdb::Comparison::LessThan((i % 100 + 1).into()))
            .query(),
    )
    .unwrap()
    .result
}

fn mvcc_txn_read(db: &DbMemory, i: u64) -> u64 {
    db.transaction(|t| -> Result<u64, agdb::DbError> {
        let r1 = t
            .exec(
                QueryBuilder::search()
                    .index("score")
                    .value(i % WARM_UP_NODES)
                    .query(),
            )?
            .result;
        let r2 = t.exec(QueryBuilder::select().ids("root").query())?.result;
        Ok(r1 + r2)
    })
    .unwrap()
}

fn mvcc_write(db: &DbMemory, i: u64) -> u64 {
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("name", format!("new_{i}")).into(), ("score", i).into()]])
            .query(),
    )
    .unwrap()
    .result
}

// ---- benchmark harness -----------------------------------------------------

struct BenchResult {
    name: String,
    threads: usize,
    rwlock_ops: u64,
    mvcc_ops: u64,
    speedup: f64,
}

fn bench_readers(threads: usize) -> BenchResult {
    // RwLock approach
    let db = Arc::new(RwLock::new(setup_db()));
    let rwlock_ops = run_threaded(threads, BENCH_DURATION, {
        let db = db.clone();
        move |i| {
            rwlock_heavy_read(&db, i);
        }
    });

    // MVCC approach
    let db = Arc::new(setup_db());
    let mvcc_ops = run_threaded(threads, BENCH_DURATION, {
        let db = db.clone();
        move |i| {
            mvcc_heavy_read(&db, i);
        }
    });

    let speedup = mvcc_ops as f64 / rwlock_ops as f64;
    BenchResult {
        name: "pure_reads".into(),
        threads,
        rwlock_ops,
        mvcc_ops,
        speedup,
    }
}

fn bench_mixed(reader_threads: usize) -> BenchResult {
    // RwLock: N heavy readers + 1 writer
    let db = Arc::new(RwLock::new(setup_db()));
    let rwlock_ops = run_mixed_rwlock(reader_threads, BENCH_DURATION, db, false);

    // MVCC: N heavy readers + 1 writer
    let db = Arc::new(setup_db());
    let mvcc_ops = run_mixed_mvcc(reader_threads, BENCH_DURATION, db, false);

    let speedup = mvcc_ops as f64 / rwlock_ops as f64;
    BenchResult {
        name: "mixed_rw".into(),
        threads: reader_threads + 1,
        rwlock_ops,
        mvcc_ops,
        speedup,
    }
}

fn bench_mixed_txn(reader_threads: usize) -> BenchResult {
    // RwLock: N transaction readers + 1 writer
    let db = Arc::new(RwLock::new(setup_db()));
    let rwlock_ops = run_mixed_rwlock(reader_threads, BENCH_DURATION, db, true);

    // MVCC: N transaction readers + 1 writer
    let db = Arc::new(setup_db());
    let mvcc_ops = run_mixed_mvcc(reader_threads, BENCH_DURATION, db, true);

    let speedup = mvcc_ops as f64 / rwlock_ops as f64;
    BenchResult {
        name: "mixed_txn".into(),
        threads: reader_threads + 1,
        rwlock_ops,
        mvcc_ops,
        speedup,
    }
}

fn bench_writers(threads: usize) -> BenchResult {
    // RwLock approach — writers must take write lock
    let db = Arc::new(RwLock::new(setup_db()));
    let rwlock_ops = run_threaded(threads, BENCH_DURATION, {
        let db = db.clone();
        move |i| {
            rwlock_write(&db, i);
        }
    });

    // MVCC approach — writers are serialized internally
    let db = Arc::new(setup_db());
    let mvcc_ops = run_threaded(threads, BENCH_DURATION, {
        let db = db.clone();
        move |i| {
            mvcc_write(&db, i);
        }
    });

    let speedup = mvcc_ops as f64 / rwlock_ops as f64;
    BenchResult {
        name: "pure_writes".into(),
        threads,
        rwlock_ops,
        mvcc_ops,
        speedup,
    }
}

fn run_threaded<F>(threads: usize, duration: Duration, op: F) -> u64
where
    F: Fn(u64) + Send + Sync + 'static,
{
    let op = Arc::new(op);
    let barrier = Arc::new(Barrier::new(threads + 1));
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let op = op.clone();
            let barrier = barrier.clone();
            let running = running.clone();
            std::thread::spawn(move || {
                let mut count = 0u64;
                let mut i = 0u64;
                barrier.wait();
                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    op(i);
                    i += 1;
                    count += 1;
                }
                count
            })
        })
        .collect();

    barrier.wait();
    std::thread::sleep(duration);
    running.store(false, std::sync::atomic::Ordering::Relaxed);

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

fn run_mixed_rwlock(
    reader_threads: usize,
    duration: Duration,
    db: Arc<RwLock<DbMemory>>,
    use_txn: bool,
) -> u64 {
    let barrier = Arc::new(Barrier::new(reader_threads + 2)); // readers + 1 writer + main
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Spawn readers
    let reader_handles: Vec<_> = (0..reader_threads)
        .map(|_| {
            let db = db.clone();
            let barrier = barrier.clone();
            let running = running.clone();
            std::thread::spawn(move || {
                let mut count = 0u64;
                let mut i = 0u64;
                barrier.wait();
                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    if use_txn {
                        rwlock_txn_read(&db, i);
                    } else {
                        rwlock_heavy_read(&db, i);
                    }
                    i += 1;
                    count += 1;
                }
                count
            })
        })
        .collect();

    // Spawn 1 writer
    let writer_handle = {
        let db = db.clone();
        let barrier = barrier.clone();
        let running = running.clone();
        std::thread::spawn(move || {
            let mut count = 0u64;
            let mut i = 0u64;
            barrier.wait();
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                rwlock_write(&db, i);
                i += 1;
                count += 1;
            }
            count
        })
    };

    barrier.wait();
    std::thread::sleep(duration);
    running.store(false, std::sync::atomic::Ordering::Relaxed);

    let read_ops: u64 = reader_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let write_ops = writer_handle.join().unwrap();
    read_ops + write_ops
}

fn run_mixed_mvcc(
    reader_threads: usize,
    duration: Duration,
    db: Arc<DbMemory>,
    use_txn: bool,
) -> u64 {
    let barrier = Arc::new(Barrier::new(reader_threads + 2));
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let reader_handles: Vec<_> = (0..reader_threads)
        .map(|_| {
            let db = db.clone();
            let barrier = barrier.clone();
            let running = running.clone();
            std::thread::spawn(move || {
                let mut count = 0u64;
                let mut i = 0u64;
                barrier.wait();
                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    if use_txn {
                        mvcc_txn_read(&db, i);
                    } else {
                        mvcc_heavy_read(&db, i);
                    }
                    i += 1;
                    count += 1;
                }
                count
            })
        })
        .collect();

    let writer_handle = {
        let db = db.clone();
        let barrier = barrier.clone();
        let running = running.clone();
        std::thread::spawn(move || {
            let mut count = 0u64;
            let mut i = 0u64;
            barrier.wait();
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                mvcc_write(&db, i);
                i += 1;
                count += 1;
            }
            count
        })
    };

    barrier.wait();
    std::thread::sleep(duration);
    running.store(false, std::sync::atomic::Ordering::Relaxed);

    let read_ops: u64 = reader_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let write_ops = writer_handle.join().unwrap();
    read_ops + write_ops
}

// ---- main ------------------------------------------------------------------

fn main() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║              agdb MVCC Concurrent Benchmark (DbMemory)                      ║");
    println!("║  Comparing: Arc<RwLock<Db>> (old)  vs  Arc<Db> (MVCC, new)                  ║");
    println!(
        "║  Duration: {BENCH_DURATION:?} per measurement, {WARM_UP_NODES} pre-loaded nodes + score index       ║"
    );
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    let mut results = Vec::new();

    // Pure reads
    println!("── Pure Reads (select + search_index) ──────────────────────────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_readers(t);
        println!(
            "  {t:>2} threads │ RwLock: {:>10} ops │ MVCC: {:>10} ops │ speedup: {:.2}×",
            r.rwlock_ops, r.mvcc_ops, r.speedup,
        );
        results.push(r);
    }

    // Mixed read/write (heavy search reads)
    println!();
    println!("── Mixed: N search readers + 1 writer ──────────────────────────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_mixed(t);
        println!(
            "  {:>2} total  │ RwLock: {:>10} ops │ MVCC: {:>10} ops │ speedup: {:.2}×",
            r.threads, r.rwlock_ops, r.mvcc_ops, r.speedup,
        );
        results.push(r);
    }

    // Mixed transaction read/write
    println!();
    println!("── Mixed: N transaction readers + 1 writer ─────────────────────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_mixed_txn(t);
        println!(
            "  {:>2} total  │ RwLock: {:>10} ops │ MVCC: {:>10} ops │ speedup: {:.2}×",
            r.threads, r.rwlock_ops, r.mvcc_ops, r.speedup,
        );
        results.push(r);
    }

    // Pure writes
    println!();
    println!("── Pure Writes (insert_node) ────────────────────────────────────────────────");
    for &t in THREAD_COUNTS {
        let r = bench_writers(t);
        println!(
            "  {t:>2} threads │ RwLock: {:>10} ops │ MVCC: {:>10} ops │ speedup: {:.2}×",
            r.rwlock_ops, r.mvcc_ops, r.speedup,
        );
        results.push(r);
    }

    // Summary
    println!();
    println!("── Summary ────────────────────────────────────────────────────────────────────");

    for category in &["pure_reads", "mixed_rw", "mixed_txn", "pure_writes"] {
        if let Some(best) = results
            .iter()
            .filter(|r| r.name == *category)
            .max_by(|a, b| a.speedup.partial_cmp(&b.speedup).unwrap())
        {
            let label = match *category {
                "pure_reads" => format!(
                    "Pure reads:     {:.2}× at {} threads",
                    best.speedup, best.threads
                ),
                "mixed_rw" => format!(
                    "Mixed search:   {:.2}× at {} threads ({}R + 1W)",
                    best.speedup,
                    best.threads,
                    best.threads - 1
                ),
                "mixed_txn" => format!(
                    "Mixed txn:      {:.2}× at {} threads ({}R + 1W)",
                    best.speedup,
                    best.threads,
                    best.threads - 1
                ),
                "pure_writes" => format!(
                    "Pure writes:    {:.2}× at {} threads",
                    best.speedup, best.threads
                ),
                _ => unreachable!(),
            };
            println!("  {label}");
        }
    }
    println!();
}
