mod test_db;

use agdb::DbAny;
use agdb::DbElement;
use agdb::DbError;
use agdb::DbErrorType;
use agdb::DbId;
use agdb::DbValue;
use agdb::QueryBuilder;
use agdb::QueryId;
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;
use test_db::TestFile;

#[track_caller]
fn with_dbs(test: impl Fn(&DbAny)) {
    let file = TestFile::new();
    let mapped = TestFile::new();

    for db in [
        DbAny::new_memory("memory").unwrap(),
        DbAny::new_mapped(mapped.file_name()).unwrap(),
        DbAny::new_file(file.file_name()).unwrap(),
    ] {
        test(&db);
    }
}

fn insert_nodes(db: &DbAny, count: u64) -> Vec<DbId> {
    db.exec_mut(QueryBuilder::insert().nodes().count(count).query())
        .unwrap()
        .ids()
}

fn value(db: &DbAny, id: impl Into<QueryId>, key: &str) -> Option<DbValue> {
    db.exec(QueryBuilder::select().values([key]).ids(id.into()).query())
        .ok()?
        .elements
        .pop()?
        .values
        .pop()
        .map(|key_value| key_value.value)
}

fn exists(db: &DbAny, id: DbId) -> bool {
    db.exec(QueryBuilder::select().ids(id).query()).is_ok()
}

fn conflict() -> DbErrorType {
    DbErrorType::Conflict
}

fn dump(db: &DbAny) -> (Vec<DbElement>, Vec<(String, DbId)>, Vec<String>) {
    let mut elements = Vec::new();

    for id in (1..64).flat_map(|id| [id, -id]) {
        if let Ok(mut result) = db.exec(QueryBuilder::select().ids(id).query()) {
            elements.append(&mut result.elements);
        }
    }

    let aliases = db
        .exec(QueryBuilder::select().aliases().query())
        .unwrap()
        .elements
        .into_iter()
        .map(|element| (element.values[0].value.to_string(), element.id))
        .collect();
    let indexes = db
        .exec(QueryBuilder::select().indexes().query())
        .unwrap()
        .elements
        .into_iter()
        .flat_map(|element| element.values)
        .map(|key_value| format!("{}={}", key_value.key, key_value.value))
        .collect();

    (elements, aliases, indexes)
}

#[test]
fn transaction_should_commit_its_writes() {
    with_dbs(|db| {
        let ids = db
            .transaction(|t| -> Result<Vec<DbId>, DbError> {
                let ids = t
                    .exec_mut(QueryBuilder::insert().nodes().count(2).query())?
                    .ids();
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(ids[0])
                        .to(ids[1])
                        .query(),
                )?;
                t.exec_mut(
                    QueryBuilder::insert()
                        .values_uniform([("key", 1).into()])
                        .ids(ids.clone())
                        .query(),
                )?;
                Ok(ids)
            })
            .unwrap();

        assert_eq!(ids, [DbId(1), DbId(2)]);
        assert_eq!(value(db, 2, "key"), Some(1.into()));
        assert!(exists(db, DbId(-1)));
    });
}

#[test]
fn transaction_should_read_its_writes() {
    with_dbs(|db| {
        insert_nodes(db, 1);

        db.transaction(|t| -> Result<(), DbError> {
            let users = t
                .exec_mut(QueryBuilder::insert().nodes().aliases("users").query())?
                .ids();
            t.exec_mut(QueryBuilder::insert().edges().from(1).to(users).query())?;
            t.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases("users")
                    .values([[("upserted", true).into()]])
                    .query(),
            )?;
            let found = t
                .exec(QueryBuilder::search().from(1).where_().node().query())?
                .ids();
            assert_eq!(found, [DbId(1), DbId(2)]);
            assert_eq!(
                t.exec(
                    QueryBuilder::select()
                        .values(["upserted"])
                        .ids("users")
                        .query()
                )?
                .elements[0]
                    .values[0]
                    .value,
                true.into()
            );

            t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .from(1)
                    .where_()
                    .edge()
                    .query(),
            )?;
            let found = t.exec(QueryBuilder::search().from(1).query())?.ids();
            assert_eq!(found, [DbId(1)]);
            Ok(())
        })
        .unwrap();

        assert_eq!(value(db, "users", "upserted"), Some(true.into()));
        assert!(!exists(db, DbId(-1)));
    });
}

#[test]
fn transaction_writes_should_be_visible_only_once_committed() {
    with_dbs(|db| {
        db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().nodes().count(1).query())?;
            assert_eq!(
                db.exec(QueryBuilder::select().node_count().query())?.result,
                0
            );
            Ok(())
        })
        .unwrap();

        assert_eq!(
            db.exec(QueryBuilder::select().node_count().query())
                .unwrap()
                .result,
            1
        );
    });
}

#[test]
fn transaction_error_should_drop_writes_and_ids() {
    with_dbs(|db| {
        let aborted = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().nodes().count(3).query())?;
            Err(DbError::db(DbErrorType::NotAllowed, "abort"))
        });
        let failed = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().nodes().count(3).query())?;
            let _ = t.exec_mut(QueryBuilder::insert().edges().from(1).to(10).query());
            Ok(())
        });

        assert!(aborted.is_err());
        assert_eq!(failed.unwrap_err().ty, DbErrorType::NotFound);
        assert_eq!(insert_nodes(db, 1), [DbId(1)]);
    });
}

#[test]
fn concurrent_inserts_should_both_commit_with_their_ids() {
    with_dbs(|db| {
        let first = db
            .transaction(|t| -> Result<Vec<DbId>, DbError> {
                let ids = t
                    .exec_mut(QueryBuilder::insert().nodes().count(2).query())?
                    .ids();
                let nested = db
                    .transaction(|t| -> Result<Vec<DbId>, DbError> {
                        let ids = t
                            .exec_mut(QueryBuilder::insert().nodes().count(2).query())?
                            .ids();
                        t.exec_mut(
                            QueryBuilder::insert()
                                .edges()
                                .from(ids[0])
                                .to(ids[1])
                                .query(),
                        )?;
                        Ok(ids)
                    })
                    .unwrap();
                assert_eq!(nested, [DbId(3), DbId(4)]);
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(ids[0])
                        .to(ids[1])
                        .query(),
                )?;
                Ok(ids)
            })
            .unwrap();

        assert_eq!(first, [DbId(1), DbId(2)]);
        assert_eq!(
            db.exec(QueryBuilder::search().from(1).query())
                .unwrap()
                .ids(),
            [DbId(1), DbId(-2), DbId(2)]
        );
        assert_eq!(
            db.exec(QueryBuilder::search().from(3).query())
                .unwrap()
                .ids(),
            [DbId(3), DbId(-1), DbId(4)]
        );
    });
}

#[test]
fn concurrent_values_should_be_last_writer_wins_per_key() {
    with_dbs(|db| {
        insert_nodes(db, 1);

        db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(
                QueryBuilder::insert()
                    .values([[("a", 1).into()]])
                    .ids(1)
                    .query(),
            )?;
            db.transaction(|t| -> Result<(), DbError> {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values([[("a", 2).into(), ("b", 2).into()]])
                        .ids(1)
                        .query(),
                )?;
                Ok(())
            })
        })
        .unwrap();

        assert_eq!(value(db, 1, "a"), Some(1.into()));
        assert_eq!(value(db, 1, "b"), Some(2.into()));
    });
}

#[test]
fn concurrent_aliases_should_be_last_writer_wins() {
    with_dbs(|db| {
        insert_nodes(db, 2);

        db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().aliases("x").ids(1).query())?;
            db.transaction(|t| -> Result<(), DbError> {
                t.exec_mut(QueryBuilder::insert().aliases("x").ids(2).query())?;
                Ok(())
            })
        })
        .unwrap();

        assert_eq!(
            db.exec(QueryBuilder::select().aliases().query())
                .unwrap()
                .elements
                .into_iter()
                .map(|element| element.id)
                .collect::<Vec<_>>(),
            [DbId(1)]
        );
    });
}

#[test]
fn concurrent_amends_should_all_count() {
    with_dbs(|db| {
        insert_nodes(db, 1);

        db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(
                QueryBuilder::insert()
                    .amend([[("counter", 1).into()]])
                    .ids(1)
                    .query(),
            )?;
            db.transaction(|t| -> Result<(), DbError> {
                t.exec_mut(
                    QueryBuilder::insert()
                        .amend([[("counter", 10).into()]])
                        .ids(1)
                        .query(),
                )?;
                Ok(())
            })
        })
        .unwrap();

        assert_eq!(value(db, 1, "counter"), Some(11.into()));
    });
}

#[test]
fn concurrent_amends_of_threads_should_all_count() {
    with_dbs(|db| {
        const THREADS: usize = 8;
        const ADDS: i64 = 50;
        insert_nodes(db, 1);
        let barrier = Barrier::new(THREADS);

        thread::scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| {
                    barrier.wait();

                    for _ in 0..ADDS {
                        db.transaction(|t| -> Result<(), DbError> {
                            t.exec_mut(
                                QueryBuilder::insert()
                                    .amend([[("counter", 1).into()]])
                                    .ids(1)
                                    .query(),
                            )?;
                            t.exec_mut(QueryBuilder::insert().nodes().count(1).query())?;
                            Ok(())
                        })
                        .unwrap();
                    }
                });
            }
        });

        assert_eq!(
            value(db, 1, "counter"),
            Some((THREADS as i64 * ADDS).into())
        );
        assert_eq!(
            db.exec(QueryBuilder::select().node_count().query())
                .unwrap()
                .result,
            THREADS as u64 * ADDS as u64 + 1
        );
    });
}

#[test]
fn edge_to_node_removed_concurrently_should_conflict() {
    with_dbs(|db| {
        insert_nodes(db, 2);

        let result = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query())?;
            db.transaction(|t| -> Result<(), DbError> {
                t.exec_mut(QueryBuilder::remove().ids(2).query())?;
                Ok(())
            })
        });

        assert_eq!(result.unwrap_err().ty, conflict());
        assert!(exists(db, DbId(1)));
        assert!(!exists(db, DbId(2)));
        assert!(!exists(db, DbId(-1)));
    });
}

#[test]
fn node_removal_should_remove_edge_added_concurrently() {
    with_dbs(|db| {
        insert_nodes(db, 2);

        db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::remove().ids(2).query())?;
            db.transaction(|t| -> Result<(), DbError> {
                t.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query())?;
                Ok(())
            })
        })
        .unwrap();

        assert!(!exists(db, DbId(2)));
        assert!(!exists(db, DbId(-1)));
        assert_eq!(
            db.exec(QueryBuilder::search().from(1).query())
                .unwrap()
                .ids(),
            [DbId(1)]
        );
    });
}

#[test]
fn update_of_element_removed_concurrently_should_conflict() {
    with_dbs(|db| {
        insert_nodes(db, 2);

        let result = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(
                QueryBuilder::insert()
                    .values([[("key", 1).into()]])
                    .ids(2)
                    .query(),
            )?;
            db.transaction(|t| -> Result<(), DbError> {
                t.exec_mut(QueryBuilder::remove().ids(2).query())?;
                Ok(())
            })
        });

        assert_eq!(result.unwrap_err().ty, conflict());
        assert!(!exists(db, DbId(2)));
    });
}

#[test]
fn update_of_element_whose_slot_was_taken_again_should_conflict() {
    with_dbs(|db| {
        insert_nodes(db, 2);

        let result = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(
                QueryBuilder::insert()
                    .values([[("key", 1).into()]])
                    .ids(2)
                    .query(),
            )?;
            db.exec_mut(QueryBuilder::remove().ids(2).query())?;
            assert_eq!(insert_nodes(db, 1), [DbId(2)]);
            Ok(())
        });

        assert_eq!(result.unwrap_err().ty, conflict());
        assert_eq!(value(db, 2, "key"), None);
    });
}

#[test]
fn removal_of_element_removed_concurrently_should_commit() {
    with_dbs(|db| {
        insert_nodes(db, 3);

        db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::remove().ids([2, 3]).query())?;
            t.exec_mut(QueryBuilder::remove().aliases("none").query())?;
            db.exec_mut(QueryBuilder::remove().ids(2).query())?;
            Ok(())
        })
        .unwrap();

        assert!(exists(db, DbId(1)));
        assert!(!exists(db, DbId(2)));
        assert!(!exists(db, DbId(3)));
    });
}

#[test]
fn removal_of_values_of_element_removed_concurrently_should_commit() {
    with_dbs(|db| {
        insert_nodes(db, 2);
        db.exec_mut(
            QueryBuilder::insert()
                .values_uniform([("k", 1).into()])
                .ids([1, 2])
                .query(),
        )
        .unwrap();

        db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::remove().values("k").ids([1, 2]).query())?;
            db.exec_mut(QueryBuilder::remove().ids(2).query())?;
            Ok(())
        })
        .unwrap();

        assert_eq!(value(db, 1, "k"), None);
        assert!(!exists(db, DbId(2)));
    });
}

#[test]
fn edge_from_node_removed_concurrently_should_conflict() {
    with_dbs(|db| {
        insert_nodes(db, 2);

        let result = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query())?;
            db.exec_mut(QueryBuilder::remove().ids(1).query())?;
            Ok(())
        });

        assert_eq!(result.unwrap_err().ty, conflict());
        assert!(!exists(db, DbId(1)));
        assert!(exists(db, DbId(2)));
        assert!(!exists(db, DbId(-1)));
    });
}

#[test]
fn index_created_concurrently_should_conflict() {
    with_dbs(|db| {
        let result = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().index("key").query())?;
            db.exec_mut(QueryBuilder::insert().index("key").query())?;
            Ok(())
        });

        assert_eq!(result.unwrap_err().ty, conflict());
        assert_eq!(dump(db).2, ["key=0"]);
    });
}

#[test]
fn optimize_storage_should_conflict_with_transaction() {
    let mapped = TestFile::new();

    for db in [
        DbAny::new_memory("memory").unwrap(),
        DbAny::new_mapped(mapped.file_name()).unwrap(),
    ] {
        let db = &db;
        insert_nodes(db, 1);

        let result = db.transaction(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().nodes().count(1).query())?;
            db.optimize_storage()?;
            Ok(())
        });

        assert_eq!(result.unwrap_err().ty, conflict());
        assert_eq!(insert_nodes(db, 1), [DbId(2)]);
    }
}

#[test]
fn transaction_without_writes_should_not_commit() {
    with_dbs(|db| {
        insert_nodes(db, 1);

        let count = db
            .transaction(|t| -> Result<u64, DbError> {
                db.exec_mut(QueryBuilder::remove().ids(1).query())?;
                Ok(t.exec(QueryBuilder::select().node_count().query())?.result)
            })
            .unwrap();

        assert_eq!(count, 1);
        assert!(!exists(db, DbId(1)));
    });
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
enum Script {
    Nodes(agdb::InsertNodesQuery),
    Edges(agdb::InsertEdgesQuery),
    Values(agdb::InsertValuesQuery),
    Aliases(agdb::InsertAliasesQuery),
    Index(agdb::InsertIndexQuery),
    Remove(agdb::RemoveQuery),
    RemoveValues(agdb::RemoveValuesQuery),
    RemoveIndex(agdb::RemoveIndexQuery),
}

macro_rules! run {
    ($query:expr, $exec:expr) => {
        match $query {
            Script::Nodes(query) => $exec(query),
            Script::Edges(query) => $exec(query),
            Script::Values(query) => $exec(query),
            Script::Aliases(query) => $exec(query),
            Script::Index(query) => $exec(query),
            Script::Remove(query) => $exec(query),
            Script::RemoveValues(query) => $exec(query),
            Script::RemoveIndex(query) => $exec(query),
        }
    };
}

fn script(seed: u64) -> Vec<Script> {
    let mut state = seed | 1;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    let mut queries = Vec::new();

    for _ in 0..40 {
        let id = next(12) as i64 + 1;
        let other = next(12) as i64 + 1;
        let query = match next(9) {
            0 | 1 => Script::Nodes(QueryBuilder::insert().nodes().count(next(3) + 1).query()),
            2 => Script::Nodes(
                QueryBuilder::insert()
                    .nodes()
                    .aliases(format!("a{}", next(4)))
                    .values([[("n", next(5)).into()]])
                    .query(),
            ),
            3 => Script::Edges(QueryBuilder::insert().edges().from(id).to(other).query()),
            4 => Script::Values(
                QueryBuilder::insert()
                    .values_uniform([
                        ("k", next(100)).into(),
                        (format!("k{}", next(3)), id).into(),
                    ])
                    .ids(id)
                    .query(),
            ),
            5 => Script::Values(
                QueryBuilder::insert()
                    .amend([[("c", 1).into()]])
                    .ids(id)
                    .query(),
            ),
            6 => Script::Remove(QueryBuilder::remove().ids(id).query()),
            7 => Script::RemoveValues(QueryBuilder::remove().values(["k"]).ids(id).query()),
            _ => match next(3) {
                0 => Script::Index(QueryBuilder::insert().index("k").query()),
                1 => Script::RemoveIndex(QueryBuilder::remove().index("k").query()),
                _ => Script::Aliases(
                    QueryBuilder::insert()
                        .aliases(format!("a{}", next(4)))
                        .ids(id)
                        .query(),
                ),
            },
        };
        queries.push(query);
    }

    queries
}

struct Runs(usize);

impl From<DbError> for Runs {
    fn from(_: DbError) -> Self {
        Self(0)
    }
}

#[test]
fn transaction_should_write_as_transaction_mut_on_fast_path_and_replay() {
    let mut compared = 0;

    for seed in 1..40 {
        let exclusive = DbAny::new_memory("exclusive").unwrap();
        let fast = DbAny::new_memory("fast").unwrap();
        let replayed = DbAny::new_memory("replayed").unwrap();

        let Err(Runs(runs)) = exclusive.transaction_mut(|t| -> Result<(), Runs> {
            Err(Runs(
                script(seed)
                    .into_iter()
                    .take_while(|query| run!(query.clone(), |q| t.exec_mut(q)).is_ok())
                    .count(),
            ))
        }) else {
            unreachable!()
        };
        let results = exclusive
            .transaction_mut(|t| -> Result<Vec<_>, DbError> {
                script(seed)
                    .into_iter()
                    .take(runs)
                    .map(|query| run!(query, |q| t.exec_mut(q)).map(|result| result.ids()))
                    .collect()
            })
            .unwrap();

        let mut others = Vec::new();

        for db in [&fast, &replayed] {
            others.push(
                db.transaction(|t| -> Result<Vec<_>, DbError> {
                    let results = script(seed)
                        .into_iter()
                        .take(runs)
                        .map(|query| run!(query, |q| t.exec_mut(q)).map(|result| result.ids()))
                        .collect::<Result<Vec<_>, DbError>>()?;

                    if std::ptr::eq(db, &replayed) {
                        // WHY: an empty commit makes the transaction replay.
                        db.transaction_mut(|_| -> Result<(), DbError> { Ok(()) })?;
                    }

                    Ok(results)
                })
                .unwrap(),
            );
        }

        assert_eq!(others[0], others[1], "seed {seed}");
        assert_eq!(dump(&fast), dump(&replayed), "seed {seed}");

        // WHY: a concurrent transaction does not reuse freed ids and the script
        // addresses elements by id, so after a reuse they diverge.
        if others[0] == results {
            assert_eq!(dump(&fast), dump(&exclusive), "seed {seed}");
            compared += 1;
        }
    }

    assert!(compared >= 30, "{compared}");
}

#[test]
fn concurrent_writers_with_readers_should_keep_snapshots() {
    with_dbs(|db| {
        const WRITERS: usize = 4;
        let db = Arc::new(db);
        insert_nodes(&db, 1);
        let barrier = Barrier::new(WRITERS + 1);

        thread::scope(|scope| {
            for writer in 0..WRITERS {
                let db = &db;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();

                    for i in 0..30_i64 {
                        db.transaction(|t| -> Result<(), DbError> {
                            let id = t
                                .exec_mut(
                                    QueryBuilder::insert()
                                        .nodes()
                                        .values([[("w", writer as i64).into(), ("i", i).into()]])
                                        .query(),
                                )?
                                .ids();
                            t.exec_mut(QueryBuilder::insert().edges().from(1).to(id).query())?;
                            Ok(())
                        })
                        .unwrap();
                    }
                });
            }

            let db = &db;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();

                for _ in 0..200 {
                    db.transaction(|t| -> Result<(), DbError> {
                        let nodes = t
                            .exec(QueryBuilder::search().from(1).where_().node().query())?
                            .ids();
                        let edges = t
                            .exec(QueryBuilder::search().from(1).where_().edge().query())?
                            .ids();
                        assert_eq!(nodes.len(), edges.len() + 1);
                        Ok(())
                    })
                    .unwrap();
                }
            });
        });

        assert_eq!(
            db.exec(QueryBuilder::search().from(1).where_().node().query())
                .unwrap()
                .ids()
                .len(),
            WRITERS * 30 + 1
        );
    });
}
