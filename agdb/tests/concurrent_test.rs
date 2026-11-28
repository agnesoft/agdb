//! Concurrent correctness tests for the MVCC implementation in
//! `ConcurrentRoot`. These exercise the public `DbMemory` API from
//! multiple threads and assert snapshot isolation, writer serialization,
//! version-store trimming, resize safety, and transaction atomicity.

mod test_db;

use agdb::{DbError, DbErrorType, DbFile, DbMemory, DbValue, QueryBuilder, StorageConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use test_db::TestFile;

/// Force the storage buffer to grow well beyond what any test needs.
/// This prevents deadlocks where a writer spin-waits for
/// `active_readers == 0` during resize while a reader is blocked
/// inside its transaction on a test barrier.
///
/// Mechanism: insert many nodes to grow the buffer, then remove them.
/// The `Vec<u8>` buffer stays at its peak size; freed pages are reused
/// by subsequent writes without further growth.
fn pre_grow(db: &DbMemory) {
    // Bulk-insert nodes to grow the buffer, then remove them. The
    // Vec<u8> stays at its peak size; freed pages are reused later.
    // 2000 nodes gives enough headroom for tests that insert ~100-200
    // additional nodes+edges during the concurrent phase.
    let ids = db
        .exec_mut(QueryBuilder::insert().nodes().count(2000).query())
        .unwrap()
        .ids();
    db.exec_mut(QueryBuilder::remove().ids(ids).query())
        .unwrap();
    // The allocator's internal structures may need extra pages the
    // first time values/edges are written after bulk-remove. Force
    // those allocations now so the concurrent phase never resizes.
    // (Without this, the writer's spin-wait for active_readers==0
    // during resize deadlocks against barrier-blocked readers.)
    db.transaction_mut(|t| -> Result<(), DbError> {
        let root = t
            .exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases("_warmup_root")
                    .query(),
            )?
            .ids();
        let ids = t
            .exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .values([[("_warmup", "settle_allocator").into()]])
                    .query(),
            )?
            .ids();
        t.exec_mut(QueryBuilder::insert().edges().from(root).to(ids).query())?;
        Ok(())
    })
    .unwrap();
    // Update a value to settle the value-update allocator path too
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("_warmup", "updated").into()]])
            .ids("_warmup_root")
            .query(),
    )
    .unwrap();
    // Clean up warmup data
    let warmup_ids = db
        .exec(QueryBuilder::search().from("_warmup_root").query())
        .unwrap()
        .ids();
    db.exec_mut(QueryBuilder::remove().ids(warmup_ids).query())
        .unwrap();
    db.exec_mut(QueryBuilder::remove().aliases("_warmup_root").query())
        .unwrap();
}

/// Creates a DbMemory with a "root" node and 10 data nodes connected to
/// it, each with `("name", "node_{i}")` and `("value", i)` properties.
/// Total: 11 nodes, 10 edges. Storage is pre-grown for concurrent use.
fn setup_db() -> DbMemory {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    db.transaction_mut(|t| -> Result<(), DbError> {
        let values: Vec<Vec<agdb::DbKeyValue>> = (0..10)
            .map(|i| vec![("name", format!("node_{i}")).into(), ("value", i).into()])
            .collect();
        let ids = t
            .exec_mut(QueryBuilder::insert().nodes().values(values).query())?
            .ids();
        t.exec_mut(QueryBuilder::insert().edges().from("root").to(ids).query())?;
        Ok(())
    })
    .unwrap();
    pre_grow(&db);
    db
}

// ---------------------------------------------------------------------------
// Snapshot isolation
// ---------------------------------------------------------------------------

/// A reader that starts before a writer commits must see the database
/// state as it was when the reader began, not the writer's changes.
/// Tests the `Arc<Meta>` snapshot path (node_count comes from Meta).
#[test]
fn snapshot_isolation_reader_sees_pre_commit_state() {
    let db = Arc::new(setup_db());
    let initial_count = db
        .exec(QueryBuilder::select().node_count().query())
        .unwrap()
        .result;
    assert_eq!(initial_count, 11);

    let reader_started = Arc::new(Barrier::new(2));
    let writer_committed = Arc::new(Barrier::new(2));

    let db_r = db.clone();
    let rs = reader_started.clone();
    let wc = writer_committed.clone();

    let reader = thread::spawn(move || {
        db_r.transaction(|t| -> Result<u64, DbError> {
            rs.wait(); // signal: reader is inside transaction
            wc.wait(); // wait for writer to commit

            // Reader still sees the old node count
            let count = t.exec(QueryBuilder::select().node_count().query())?.result;
            Ok(count)
        })
        .unwrap()
    });

    // Wait for reader to enter its transaction
    reader_started.wait();

    // Commit 5 new nodes
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query())
        .unwrap();

    // Signal: writer has committed
    writer_committed.wait();

    let reader_saw = reader.join().unwrap();
    assert_eq!(reader_saw, 11, "reader must see pre-commit snapshot");

    // After both finish, the database has the writer's changes
    let final_count = db
        .exec(QueryBuilder::select().node_count().query())
        .unwrap()
        .result;
    assert_eq!(final_count, 16);
}

/// A reader sees old property values even after a writer has overwritten
/// them. This exercises the `SnapshotView` and `VersionStore` code paths.
#[test]
fn snapshot_isolation_reader_sees_pre_commit_values() {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("target")
            .values(vec![vec![("key", "original").into()]])
            .query(),
    )
    .unwrap();
    pre_grow(&db);
    // Warmup: one value update absorbs the one-time page allocation
    // for value operations, then reset. After this, further value
    // updates on "target" won't trigger storage resize.
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("key", "warmup").into()]])
            .ids("target")
            .query(),
    )
    .unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("key", "original").into()]])
            .ids("target")
            .query(),
    )
    .unwrap();
    let db = Arc::new(db);

    let reader_started = Arc::new(Barrier::new(2));
    let writer_committed = Arc::new(Barrier::new(2));

    let db_r = db.clone();
    let rs = reader_started.clone();
    let wc = writer_committed.clone();

    let reader = thread::spawn(move || {
        db_r.transaction(|t| -> Result<DbValue, DbError> {
            rs.wait(); // reader is inside transaction
            wc.wait(); // writer has committed

            let result = t.exec(QueryBuilder::select().ids("target").query())?;
            Ok(result.elements[0].values[0].value.clone())
        })
        .unwrap()
    });

    reader_started.wait();

    // Overwrite the value
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("key", "modified").into()]])
            .ids("target")
            .query(),
    )
    .unwrap();

    writer_committed.wait();

    let reader_saw = reader.join().unwrap();
    assert_eq!(
        reader_saw,
        DbValue::from("original"),
        "reader must see pre-commit value"
    );

    let current = db
        .exec(QueryBuilder::select().ids("target").query())
        .unwrap()
        .elements[0]
        .values[0]
        .value
        .clone();
    assert_eq!(current, DbValue::from("modified"));
}

// ---------------------------------------------------------------------------
// Writer serialization
// ---------------------------------------------------------------------------

/// N threads each insert a unique node with a unique alias. After all
/// threads finish, every alias exists and every node is present.
#[test]
fn writer_serialization_all_writes_visible() {
    let db = DbMemory::new("").unwrap();
    pre_grow(&db);
    let db = Arc::new(db);
    let n_writers: usize = 8;
    let nodes_per_writer: usize = 10;
    let barrier = Arc::new(Barrier::new(n_writers));

    let handles: Vec<_> = (0..n_writers)
        .map(|w| {
            let db = db.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                for i in 0..nodes_per_writer {
                    let alias = format!("w{w}_n{i}");
                    db.exec_mut(QueryBuilder::insert().nodes().aliases(alias).query())
                        .unwrap();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let aliases = db.exec(QueryBuilder::select().aliases().query()).unwrap();
    assert_eq!(aliases.result, (n_writers * nodes_per_writer) as u64);

    let count = db
        .exec(QueryBuilder::select().node_count().query())
        .unwrap()
        .result;
    assert_eq!(count, (n_writers * nodes_per_writer) as u64);
}

// ---------------------------------------------------------------------------
// Version store trimming
// ---------------------------------------------------------------------------

/// A reader holds a snapshot at an old epoch while multiple writers
/// commit. The reader must still see correct data. After the reader
/// drops, subsequent writer commits trim the version store.
#[test]
fn version_store_trimming_after_reader_drops() {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("target")
            .values(vec![vec![("v", 0).into()]])
            .query(),
    )
    .unwrap();
    pre_grow(&db);
    // Warmup: absorb one-time page allocation for value updates
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", -1).into()]])
            .ids("target")
            .query(),
    )
    .unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", 0).into()]])
            .ids("target")
            .query(),
    )
    .unwrap();
    let db = Arc::new(db);

    let reader_started = Arc::new(Barrier::new(2));
    let reader_should_read = Arc::new(Barrier::new(2));

    let db_r = db.clone();
    let rs = reader_started.clone();
    let rr = reader_should_read.clone();

    let reader = thread::spawn(move || {
        db_r.transaction(|t| -> Result<DbValue, DbError> {
            rs.wait(); // signal: reader is inside transaction at epoch E
            rr.wait(); // wait for multiple writes to happen

            let result = t.exec(QueryBuilder::select().ids("target").query())?;
            Ok(result.elements[0].values[0].value.clone())
        })
        .unwrap()
    });

    reader_started.wait();

    // Commit many writes while reader holds epoch E
    for i in 1..=20 {
        db.exec_mut(
            QueryBuilder::insert()
                .values(vec![vec![("v", i).into()]])
                .ids("target")
                .query(),
        )
        .unwrap();
    }

    reader_should_read.wait();
    let reader_saw = reader.join().unwrap();
    assert_eq!(
        reader_saw,
        DbValue::from(0_i64),
        "reader must see original value"
    );

    // Reader has dropped. The next writer commit should trim version
    // store entries. Verify no errors and the value is current.
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", 999).into()]])
            .ids("target")
            .query(),
    )
    .unwrap();

    let current = db
        .exec(QueryBuilder::select().ids("target").query())
        .unwrap()
        .elements[0]
        .values[0]
        .value
        .clone();
    assert_eq!(current, DbValue::from(999_i64));
}

// ---------------------------------------------------------------------------
// BTreeMap refcount (multiple readers at same epoch)
// ---------------------------------------------------------------------------

/// Two readers start at the same epoch. One drops while a writer
/// commits (triggering trim). The other must still see correct data.
#[test]
fn multiple_readers_same_epoch_refcount() {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("target")
            .values(vec![vec![("v", "initial").into()]])
            .query(),
    )
    .unwrap();
    pre_grow(&db);
    // Warmup: absorb one-time page allocation for value updates
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", "warmup").into()]])
            .ids("target")
            .query(),
    )
    .unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", "initial").into()]])
            .ids("target")
            .query(),
    )
    .unwrap();
    let db = Arc::new(db);

    // Both readers enter their transactions at the same epoch
    let both_started = Arc::new(Barrier::new(3)); // 2 readers + main
    let writer_committed = Arc::new(Barrier::new(3));
    let reader1_may_finish = Arc::new(Barrier::new(2)); // reader1 + main
    let reader2_reads = Arc::new(Barrier::new(2)); // reader2 + main

    let db1 = db.clone();
    let bs1 = both_started.clone();
    let wc1 = writer_committed.clone();
    let r1f = reader1_may_finish.clone();

    let reader1 = thread::spawn(move || {
        db1.transaction(|t| -> Result<DbValue, DbError> {
            bs1.wait(); // both readers started
            wc1.wait(); // writer committed
            let result = t.exec(QueryBuilder::select().ids("target").query())?;
            let val = result.elements[0].values[0].value.clone();
            r1f.wait(); // signal: reader1 is about to drop
            Ok(val)
        })
        .unwrap()
    });

    let db2 = db.clone();
    let bs2 = both_started.clone();
    let wc2 = writer_committed.clone();
    let r2r = reader2_reads.clone();

    let reader2 = thread::spawn(move || {
        db2.transaction(|t| -> Result<DbValue, DbError> {
            bs2.wait(); // both readers started
            wc2.wait(); // writer committed
            r2r.wait(); // wait for reader1 to drop + writer trim

            let result = t.exec(QueryBuilder::select().ids("target").query())?;
            Ok(result.elements[0].values[0].value.clone())
        })
        .unwrap()
    });

    // Wait for both readers to start
    both_started.wait();

    // Commit a change
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", "changed").into()]])
            .ids("target")
            .query(),
    )
    .unwrap();

    // Signal writer committed
    writer_committed.wait();

    // Wait for reader1 to finish and drop
    reader1_may_finish.wait();
    let r1_saw = reader1.join().unwrap();
    assert_eq!(r1_saw, DbValue::from("initial"));

    // Another write to trigger trim
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", "changed_again").into()]])
            .ids("target")
            .query(),
    )
    .unwrap();

    // Let reader2 read now (after reader1 dropped and trim ran)
    reader2_reads.wait();
    let r2_saw = reader2.join().unwrap();
    assert_eq!(
        r2_saw,
        DbValue::from("initial"),
        "reader2 must still see initial value after reader1 dropped"
    );
}

// ---------------------------------------------------------------------------
// Resize safety
// ---------------------------------------------------------------------------

/// When enough data is inserted to trigger a storage resize, active
/// readers must either complete successfully or fall back to the locked
/// path. Neither path should panic or corrupt data.
#[test]
fn resize_safety_readers_survive_or_fallback() {
    let db = Arc::new(setup_db());
    let barrier = Arc::new(Barrier::new(6)); // 4 readers + 1 writer + main
    let running = Arc::new(AtomicBool::new(true));

    let readers: Vec<_> = (0..4)
        .map(|_| {
            let db = db.clone();
            let barrier = barrier.clone();
            let running = running.clone();
            thread::spawn(move || {
                let mut count = 0u64;
                barrier.wait();
                while running.load(Ordering::Relaxed) {
                    let result = db.exec(QueryBuilder::select().node_count().query());
                    assert!(result.is_ok(), "reader failed: {:?}", result.err());
                    let n = result.unwrap().result;
                    assert!(n >= 11, "node count went backwards: {n}");
                    count += 1;
                }
                count
            })
        })
        .collect();

    let db_w = db.clone();
    let barrier_w = barrier.clone();
    let writer = thread::spawn(move || {
        barrier_w.wait();
        for i in 0..200 {
            db_w.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .values([[("data", format!("resize_test_{i}")).into()]])
                    .query(),
            )
            .unwrap();
        }
    });

    barrier.wait();
    writer.join().unwrap();
    running.store(false, Ordering::Relaxed);

    for r in readers {
        let ops = r.join().unwrap();
        assert!(ops > 0, "reader should have completed at least one read");
    }

    let final_count = db
        .exec(QueryBuilder::select().node_count().query())
        .unwrap()
        .result;
    assert_eq!(final_count, 211);
}

// ---------------------------------------------------------------------------
// Mixed read/write stress
// ---------------------------------------------------------------------------

/// Multiple threads doing reads and writes simultaneously. Asserts no
/// panics, no errors, and the database is consistent after all threads
/// finish.
#[test]
fn mixed_read_write_stress() {
    let db = Arc::new(setup_db());
    let n_readers: usize = 4;
    let n_writers: usize = 2;
    let iterations: usize = 50;
    let barrier = Arc::new(Barrier::new(n_readers + n_writers));

    let readers: Vec<_> = (0..n_readers)
        .map(|_| {
            let db = db.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                for _ in 0..iterations {
                    db.transaction(|t| -> Result<(), DbError> {
                        let count = t.exec(QueryBuilder::select().node_count().query())?.result;
                        assert!(count >= 11);
                        t.exec(QueryBuilder::search().from("root").query())?;
                        Ok(())
                    })
                    .unwrap();
                }
            })
        })
        .collect();

    let writers: Vec<_> = (0..n_writers)
        .map(|w| {
            let db = db.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                for i in 0..iterations {
                    db.exec_mut(
                        QueryBuilder::insert()
                            .nodes()
                            .values([[("writer", w as i64).into(), ("iter", i as i64).into()]])
                            .query(),
                    )
                    .unwrap();
                }
            })
        })
        .collect();

    for r in readers {
        r.join().unwrap();
    }
    for w in writers {
        w.join().unwrap();
    }

    let expected = 11 + (n_writers * iterations) as u64;
    let actual = db
        .exec(QueryBuilder::select().node_count().query())
        .unwrap()
        .result;
    assert_eq!(actual, expected);
}

// ---------------------------------------------------------------------------
// Transaction rollback
// ---------------------------------------------------------------------------

/// A writer that fails/rolls back must not affect concurrent readers.
#[test]
fn transaction_rollback_under_concurrency() {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("stable")
            .values(vec![vec![("v", "untouched").into()]])
            .query(),
    )
    .unwrap();
    pre_grow(&db);
    // Warmup: absorb one-time page allocation for value updates
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", "warmup").into()]])
            .ids("stable")
            .query(),
    )
    .unwrap();
    db.exec_mut(
        QueryBuilder::insert()
            .values(vec![vec![("v", "untouched").into()]])
            .ids("stable")
            .query(),
    )
    .unwrap();
    let db = Arc::new(db);

    let reader_started = Arc::new(Barrier::new(2));
    let writer_done = Arc::new(Barrier::new(2));

    let db_r = db.clone();
    let rs = reader_started.clone();
    let wd = writer_done.clone();

    let reader = thread::spawn(move || {
        db_r.transaction(|t| -> Result<DbValue, DbError> {
            rs.wait(); // reader is inside transaction
            wd.wait(); // writer has attempted and rolled back

            let result = t.exec(QueryBuilder::select().ids("stable").query())?;
            Ok(result.elements[0].values[0].value.clone())
        })
        .unwrap()
    });

    reader_started.wait();

    // Writer transaction that rolls back
    let result = db.transaction_mut(|t| -> Result<(), DbError> {
        t.exec_mut(
            QueryBuilder::insert()
                .values(vec![vec![("v", "corrupted").into()]])
                .ids("stable")
                .query(),
        )?;
        Err(DbError::db(DbErrorType::NotAllowed, "deliberate rollback"))
    });
    assert!(result.is_err());

    writer_done.wait();

    let reader_saw = reader.join().unwrap();
    assert_eq!(reader_saw, DbValue::from("untouched"));

    let current = db
        .exec(QueryBuilder::select().ids("stable").query())
        .unwrap()
        .elements[0]
        .values[0]
        .value
        .clone();
    assert_eq!(current, DbValue::from("untouched"));
}

// ---------------------------------------------------------------------------
// Read consistency (multi-query transaction)
// ---------------------------------------------------------------------------

/// A read transaction sees a consistent snapshot even when it executes
/// multiple queries — the node count and actual search results agree.
#[test]
fn concurrent_transactions_read_consistency() {
    let db = Arc::new(setup_db());
    let running = Arc::new(AtomicBool::new(true));
    let barrier = Arc::new(Barrier::new(3));

    let readers: Vec<_> = (0..2)
        .map(|_| {
            let db = db.clone();
            let running = running.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                while running.load(Ordering::Relaxed) {
                    db.transaction(|t| -> Result<(), DbError> {
                        let count = t.exec(QueryBuilder::select().node_count().query())?.result;
                        let search = t.exec(QueryBuilder::search().from("root").query())?;
                        // Nodes have positive IDs, edges have negative IDs
                        let node_count_in_search =
                            search.elements.iter().filter(|e| e.id.0 > 0).count() as u64;
                        assert!(
                            node_count_in_search <= count,
                            "search found {node_count_in_search} nodes but count is {count}",
                        );
                        Ok(())
                    })
                    .unwrap();
                }
            })
        })
        .collect();

    let db_w = db.clone();
    let barrier_w = barrier.clone();
    let writer = thread::spawn(move || {
        barrier_w.wait();
        for i in 0..100 {
            let ids = db_w
                .exec_mut(
                    QueryBuilder::insert()
                        .nodes()
                        .values([[("n", i as i64).into()]])
                        .query(),
                )
                .unwrap()
                .ids();
            db_w.exec_mut(QueryBuilder::insert().edges().from("root").to(ids).query())
                .unwrap();
        }
    });

    writer.join().unwrap();
    running.store(false, Ordering::Relaxed);
    for r in readers {
        r.join().unwrap();
    }
}

// ---------------------------------------------------------------------------
// Writer transaction atomicity
// ---------------------------------------------------------------------------

/// A multi-query write transaction is atomic — concurrent readers see
/// either the state before the transaction or the state after, never
/// a partial commit.
#[test]
fn writer_transaction_atomicity() {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    pre_grow(&db);
    let db = Arc::new(db);

    let running = Arc::new(AtomicBool::new(true));
    let barrier = Arc::new(Barrier::new(3));

    let readers: Vec<_> = (0..2)
        .map(|_| {
            let db = db.clone();
            let running = running.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                while running.load(Ordering::Relaxed) {
                    db.transaction(|t| -> Result<(), DbError> {
                        let count = t.exec(QueryBuilder::select().node_count().query())?.result;
                        let edges = t
                            .exec(QueryBuilder::select().edge_count_from().ids("root").query())?
                            .elements[0]
                            .values[0]
                            .value
                            .to_u64()
                            .unwrap();
                        // Each transaction_mut inserts a node AND an edge
                        // atomically, so edges must always equal count - 1.
                        assert_eq!(
                            edges,
                            count - 1,
                            "edges ({edges}) != nodes-1 ({}) — partial commit visible",
                            count - 1
                        );
                        Ok(())
                    })
                    .unwrap();
                }
            })
        })
        .collect();

    let db_w = db.clone();
    let barrier_w = barrier.clone();
    let writer = thread::spawn(move || {
        barrier_w.wait();
        for i in 0..100 {
            db_w.transaction_mut(|t| -> Result<(), DbError> {
                let ids = t
                    .exec_mut(
                        QueryBuilder::insert()
                            .nodes()
                            .values([[("n", i as i64).into()]])
                            .query(),
                    )?
                    .ids();
                t.exec_mut(QueryBuilder::insert().edges().from("root").to(ids).query())?;
                Ok(())
            })
            .unwrap();
        }
    });

    writer.join().unwrap();
    running.store(false, Ordering::Relaxed);
    for r in readers {
        r.join().unwrap();
    }
}

// ---------------------------------------------------------------------------
// Restore tests
// ---------------------------------------------------------------------------

fn get_node_count(db: &DbMemory) -> u64 {
    db.exec(QueryBuilder::select().node_count().query())
        .unwrap()
        .result
}

#[test]
fn restore_should_replace_data() {
    let db = DbMemory::new("").unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query())
        .unwrap();
    assert_eq!(get_node_count(&db), 5);

    // Backup to a temp file.
    let dir = std::env::temp_dir().join("agdb_restore_test_replace");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let backup_path = dir.join("backup.agdb");
    db.backup(backup_path.to_str().unwrap()).unwrap();

    // Insert more nodes AFTER the backup.
    db.exec_mut(QueryBuilder::insert().nodes().count(10).query())
        .unwrap();
    assert_eq!(get_node_count(&db), 15);

    // Restore from the backup — should revert to 5 nodes.
    db.restore(backup_path.to_str().unwrap()).unwrap();
    assert_eq!(get_node_count(&db), 5);

    // The database is fully usable after restore.
    db.exec_mut(QueryBuilder::insert().nodes().count(3).query())
        .unwrap();
    assert_eq!(get_node_count(&db), 8);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn restore_should_be_visible_to_new_readers() {
    let db = Arc::new(DbMemory::new("").unwrap());
    pre_grow(&db);

    // Insert 10 aliased nodes.
    for i in 0..10 {
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(format!("node_{i}"))
                .query(),
        )
        .unwrap();
    }
    assert_eq!(get_node_count(&db), 10);

    // Backup.
    let dir = std::env::temp_dir().join("agdb_restore_test_readers");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let backup_path = dir.join("backup.agdb");
    db.backup(backup_path.to_str().unwrap()).unwrap();

    // Insert more nodes.
    db.exec_mut(QueryBuilder::insert().nodes().count(20).query())
        .unwrap();
    assert_eq!(get_node_count(&db), 30);

    // Restore to 10 nodes.
    db.restore(backup_path.to_str().unwrap()).unwrap();

    // Concurrent readers should all see 10 nodes.
    let barrier = Arc::new(Barrier::new(5));
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let db = db.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                let count = get_node_count(&db);
                assert!(count >= 10, "reader saw {count} nodes, expected >= 10");
            })
        })
        .collect();

    barrier.wait();
    for h in handles {
        h.join().unwrap();
    }

    // Write after restore works.
    db.exec_mut(QueryBuilder::insert().nodes().count(5).query())
        .unwrap();
    assert_eq!(get_node_count(&db), 15);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_snapshot_reader_sees_pre_commit_values() {
    let test_file = TestFile::new();
    let db = Arc::new(DbFile::new(test_file.file_name()).unwrap());
    db.set_storage_config(StorageConfig {
        overlay_limit: 0,
        memory_limit: Some(1024 * 1024),
        ..db.storage_config()
    });
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases("target")
            .values([[("key", "original").into()]])
            .query(),
    )
    .unwrap();
    let reader_started = Arc::new(Barrier::new(2));
    let writer_committed = Arc::new(Barrier::new(2));
    let (db_r, rs, wc) = (db.clone(), reader_started.clone(), writer_committed.clone());

    let reader = thread::spawn(move || {
        db_r.transaction(|t| -> Result<(DbValue, u64), DbError> {
            rs.wait();
            wc.wait();
            let value = t.exec(QueryBuilder::select().values("key").ids("target").query())?;
            let count = t.exec(QueryBuilder::select().node_count().query())?.result;
            Ok((value.elements[0].values[0].value.clone(), count))
        })
        .unwrap()
    });
    reader_started.wait();
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("key", "x".repeat(10_000)).into()]])
            .ids("target")
            .query(),
    )
    .unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().count(100).query())
        .unwrap();
    writer_committed.wait();

    assert_eq!(reader.join().unwrap(), ("original".into(), 1));
    assert_eq!(
        db.exec(QueryBuilder::select().values("key").ids("target").query())
            .unwrap()
            .elements[0]
            .values[0]
            .value,
        "x".repeat(10_000).into()
    );
}

#[test]
fn file_nested_read_during_optimize_should_not_deadlock() {
    let test_file = TestFile::new();
    let db = Arc::new(DbFile::new(test_file.file_name()).unwrap());
    db.exec_mut(QueryBuilder::insert().nodes().aliases("n").query())
        .unwrap();
    let reading = Arc::new(Barrier::new(2));
    let (db_r, reading_r) = (db.clone(), reading.clone());

    let reader = thread::spawn(move || {
        db_r.transaction(|_| -> Result<u64, DbError> {
            reading_r.wait();
            thread::sleep(std::time::Duration::from_millis(50));
            db_r.transaction(|t| -> Result<u64, DbError> {
                Ok(t.exec(QueryBuilder::select().ids("n").query())?.result)
            })
        })
    });
    reading.wait();

    db.optimize_storage().unwrap();

    assert_eq!(reader.join().unwrap(), Ok(1));
    assert_eq!(
        db.exec(QueryBuilder::select().ids("n").query())
            .unwrap()
            .result,
        1
    );
}

#[test]
fn file_concurrent_transactions_read_consistency() {
    let test_file = TestFile::new();
    let db = Arc::new(DbFile::new(test_file.file_name()).unwrap());
    db.set_storage_config(StorageConfig {
        overlay_limit: 4096,
        memory_limit: Some(64 * 1024),
        ..db.storage_config()
    });
    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    let running = Arc::new(AtomicBool::new(true));

    let readers: Vec<_> = (0..3)
        .map(|_| {
            let (db, running) = (db.clone(), running.clone());
            thread::spawn(move || {
                let mut checks = 0_u64;

                while running.load(Ordering::Relaxed) || checks == 0 {
                    db.transaction(|t| -> Result<(), DbError> {
                        let count = t.exec(QueryBuilder::select().node_count().query())?.result;
                        let search = t.exec(
                            QueryBuilder::select()
                                .search()
                                .from("root")
                                .where_()
                                .node()
                                .query(),
                        )?;
                        assert_eq!(search.elements.len() as u64, count);

                        for element in &search.elements[1..] {
                            assert_eq!(element.values.len(), 2, "{element:?}");
                        }

                        Ok(())
                    })
                    .unwrap();
                    checks += 1;
                }
            })
        })
        .collect();

    for i in 0..200_i64 {
        db.transaction_mut(|t| -> Result<(), DbError> {
            let id = t
                .exec_mut(
                    QueryBuilder::insert()
                        .nodes()
                        .values([[("n", i).into(), ("blob", vec![i as u8; 600]).into()]])
                        .query(),
                )?
                .ids();
            t.exec_mut(QueryBuilder::insert().edges().from("root").to(id).query())?;
            Ok(())
        })
        .unwrap();
    }

    running.store(false, Ordering::Relaxed);

    for reader in readers {
        reader.join().unwrap();
    }

    assert_eq!(
        db.exec(QueryBuilder::select().node_count().query())
            .unwrap()
            .result,
        201
    );
}
