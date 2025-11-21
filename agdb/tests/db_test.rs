#[cfg(feature = "derive")]
mod derive_feature_test;
#[cfg(feature = "derive")]
mod efficient_agdb;
#[cfg(feature = "openapi")]
mod openapi_feature_test;
#[cfg(feature = "derive")]
mod quickstart;
#[cfg(feature = "serde")]
mod serde_feature_test;
mod test_db;

use agdb::Db;
use agdb::DbAny;
use agdb::DbElement;
use agdb::DbFile;
use agdb::DbId;
use agdb::DbMemory;
use agdb::MemoryStorage;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::StorageData;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;
use test_db::TestDb;
use test_db::test_file::TestFile;

#[allow(unused_imports)]
#[test]
fn public_types() {
    use agdb::AgdbSerialize;
    use agdb::Comparison;
    use agdb::CountComparison;
    use agdb::Db;
    use agdb::DbElement;
    use agdb::DbError;
    use agdb::DbF64;
    use agdb::DbFile;
    use agdb::DbFileTransaction;
    use agdb::DbFileTransactionMut;
    use agdb::DbId;
    use agdb::DbImpl;
    use agdb::DbKeyOrder;
    use agdb::DbKeyValue;
    use agdb::DbMemory;
    use agdb::DbMemoryTransaction;
    use agdb::DbMemoryTransactionMut;
    use agdb::DbTransaction;
    use agdb::DbTransactionMut;
    use agdb::DbType;
    use agdb::DbValue;
    use agdb::FileStorage;
    use agdb::FileStorageMemoryMapped;
    use agdb::InsertAliasesQuery;
    use agdb::InsertEdgesQuery;
    use agdb::InsertIndexQuery;
    use agdb::InsertNodesQuery;
    use agdb::InsertValuesQuery;
    use agdb::MemoryStorage;
    use agdb::Query;
    use agdb::QueryBuilder;
    use agdb::QueryCondition;
    use agdb::QueryConditionData;
    use agdb::QueryConditionLogic;
    use agdb::QueryConditionModifier;
    use agdb::QueryId;
    use agdb::QueryIds;
    use agdb::QueryMut;
    use agdb::QueryResult;
    use agdb::QueryValues;
    use agdb::RemoveAliasesQuery;
    use agdb::RemoveQuery;
    use agdb::RemoveValuesQuery;
    use agdb::SearchQuery;
    use agdb::SearchQueryAlgorithm;
    use agdb::SelectAliasesQuery;
    use agdb::SelectAllAliasesQuery;
    use agdb::SelectEdgeCountQuery;
    use agdb::SelectIndexesQuery;
    use agdb::SelectKeyCountQuery;
    use agdb::SelectKeysQuery;
    use agdb::SelectNodeCountQuery;
    use agdb::SelectValuesQuery;
    use agdb::StableHash;
    use agdb::StorageData;
    use agdb::StorageSlice;
    use agdb::Transaction;
    use agdb::TransactionMut;
}

#[test]
fn data_persistence() {
    let test_file = TestFile::new();
    let values = vec![
        ("key", "String that is much longer than 15 characters").into(),
        (-10_i64, 2000000000000_i64).into(),
        (10_u64, 1.1_f64).into(),
        (vec!["Some", "List"], vec![1_u64, 2_u64]).into(),
        (
            vec![-1_i64, -2_i64, -3_i64],
            vec![-3.3_f64, -3.4_f64, -720.984_f64],
        )
            .into(),
        (vec![3_u8; 5], vec![15_u8; 20]).into(),
    ];

    {
        let mut db = Db::new(test_file.file_name()).unwrap();
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(["alias", "alias2"])
                .values_uniform(values.clone())
                .query(),
        )
        .unwrap();
        db.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query())
            .unwrap();
        let result = db
            .exec(
                QueryBuilder::select()
                    .ids([QueryId::from("alias"), "alias2".into(), (-3).into()])
                    .query(),
            )
            .unwrap();

        assert_eq!(
            result.elements,
            &[
                DbElement {
                    id: DbId(1),
                    from: None,
                    to: None,
                    values: values.clone(),
                },
                DbElement {
                    id: DbId(2),
                    from: None,
                    to: None,
                    values: values.clone(),
                },
                DbElement {
                    id: DbId(-3),
                    from: Some(DbId(1)),
                    to: Some(DbId(2)),
                    values: vec![],
                }
            ]
        );
    }

    let db = Db::new(test_file.file_name()).unwrap();
    let result = db
        .exec(
            QueryBuilder::select()
                .ids([QueryId::from("alias"), "alias2".into(), (-3).into()])
                .query(),
        )
        .unwrap();

    assert_eq!(
        result.elements,
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: values.clone(),
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values,
            },
            DbElement {
                id: DbId(-3),
                from: Some(DbId(1)),
                to: Some(DbId(2)),
                values: vec![],
            }
        ]
    );
}

#[test]
fn data_remove_persistence() {
    let test_file = TestFile::new();

    {
        let mut db = Db::new(test_file.file_name()).unwrap();
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(["alias", "alias2"])
                .values_uniform([("key", 100).into()])
                .query(),
        )
        .unwrap();
        db.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query())
            .unwrap();
        let result = db
            .exec(
                QueryBuilder::select()
                    .ids([QueryId::from("alias"), "alias2".into(), (-3).into()])
                    .query(),
            )
            .unwrap();

        assert_eq!(
            result.elements,
            &[
                DbElement {
                    id: DbId(1),
                    from: None,
                    to: None,
                    values: vec![("key", 100).into()],
                },
                DbElement {
                    id: DbId(2),
                    from: None,
                    to: None,
                    values: vec![("key", 100).into()],
                },
                DbElement {
                    id: DbId(-3),
                    from: Some(DbId(1)),
                    to: Some(DbId(2)),
                    values: vec![],
                }
            ]
        );

        db.exec_mut(QueryBuilder::remove().ids(-3).query()).unwrap();
        db.exec_mut(QueryBuilder::remove().values("key").ids(1).query())
            .unwrap();
    }

    let db = Db::new(test_file.file_name()).unwrap();
    let result = db
        .exec(QueryBuilder::select().ids(["alias", "alias2"]).query())
        .unwrap();

    assert_eq!(
        result.elements,
        &[
            DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![],
            },
            DbElement {
                id: DbId(2),
                from: None,
                to: None,
                values: vec![("key", 100).into()],
            }
        ]
    );

    let error = db.exec(QueryBuilder::select().ids(-3).query()).unwrap_err();
    assert_eq!(error.description, "Id '-3' not found");
}

#[test]
fn invalid_db_file() {
    let test_file = TestFile::new();
    let bytes = [0_u64.to_le_bytes(), 0_u64.to_le_bytes()].concat();
    std::fs::write(test_file.file_name(), bytes).unwrap();
    let error = Db::new(test_file.file_name()).unwrap_err();

    assert_eq!(error.description, "Failed to create database");
    assert_eq!(
        error.cause.unwrap().description,
        "Storage error: invalid version record size (0 < 8)"
    );
}

#[test]
fn optimize_on_drop() {
    let test_file = TestFile::new();
    let db_file_size;

    {
        let mut db = Db::new(test_file.file_name()).unwrap();
        let result = db
            .exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .count(1000)
                    .values_uniform([("key", "value").into()])
                    .query(),
            )
            .unwrap();
        db.exec_mut(QueryBuilder::remove().ids(result).query())
            .unwrap();
        db_file_size = std::fs::File::open(test_file.file_name())
            .unwrap()
            .metadata()
            .unwrap()
            .len();
    }

    let optimized_file_size = std::fs::File::open(test_file.file_name())
        .unwrap()
        .metadata()
        .unwrap()
        .len();

    assert!(optimized_file_size < db_file_size);
}

#[test]
fn share_between_threads() {
    let test_file = TestFile::new();
    let db = Arc::new(RwLock::new(Db::new(test_file.file_name()).unwrap()));
    db.write()
        .unwrap()
        .exec_mut(QueryBuilder::insert().nodes().count(1).query())
        .unwrap();
    let db2 = db.clone();

    let t1 = std::thread::spawn(move || {
        db.read()
            .unwrap()
            .exec(QueryBuilder::search().from(1).query())
            .unwrap()
    });
    let t2 = std::thread::spawn(move || {
        db2.read()
            .unwrap()
            .exec(QueryBuilder::search().from(1).query())
            .unwrap()
    });

    t1.join().unwrap();
    t2.join().unwrap();
}

#[test]
fn hot_backup() {
    let test_file = TestFile::new();
    let signal = Arc::new(RwLock::new(true));
    let db = Arc::new(RwLock::new(Db::new(test_file.file_name()).unwrap()));
    db.write()
        .unwrap()
        .exec_mut(QueryBuilder::insert().nodes().count(1).query())
        .unwrap();

    let db2 = db.clone();
    let signal2 = signal.clone();
    let t1 = std::thread::spawn(move || {
        while *signal2.read().unwrap() {
            db2.write()
                .unwrap()
                .exec_mut(QueryBuilder::insert().nodes().count(1).query())
                .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let test_file2 = TestFile::new();
    db.write().unwrap().backup(test_file2.file_name()).unwrap();
    *signal.write().unwrap() = false;
    let db = Db::new(test_file2.file_name()).unwrap();
    assert_eq!(
        db.exec(QueryBuilder::select().ids(1).query())
            .unwrap()
            .result,
        1
    );
    t1.join().unwrap();
}

#[test]
fn memory_backup() {
    let test_file = TestFile::new();

    {
        let mut db = DbMemory::new("memdb").unwrap();
        db.exec_mut(QueryBuilder::insert().nodes().count(1).query())
            .unwrap();
        db.backup(test_file.file_name()).unwrap();
        assert!(std::fs::exists(test_file.file_name()).unwrap());
        db.exec_mut(QueryBuilder::insert().nodes().count(1).query())
            .unwrap();
    }

    let db = DbMemory::new(test_file.file_name()).unwrap();
    assert_eq!(
        db.exec(QueryBuilder::select().node_count().query())
            .unwrap()
            .elements[0]
            .values[0]
            .value
            .to_u64()
            .unwrap(),
        1
    );
}

#[test]
fn filename() {
    let test_file = TestFile::new();
    let db = Db::new(test_file.file_name()).unwrap();
    assert_eq!(db.filename(), test_file.file_name());
}

#[test]
fn optimize_storage() {
    let mut db = TestDb::new();

    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .count(100)
            .values_uniform([("key", 123).into()])
            .query(),
        100,
    );

    let size = db.db.size();
    db.db.optimize_storage().unwrap();
    let optimized_size = db.db.size();

    assert!(optimized_size < size);
}

#[test]
fn rename_memory() {
    let mut db = DbMemory::new("memdb").unwrap();
    db.rename("mydb").unwrap();
    assert_eq!(db.filename(), "mydb");
}

#[test]
fn rename_mapped() {
    let test_file = TestFile::new();
    let test_file2 = TestFile::new();
    let mut db = Db::new(test_file.file_name()).unwrap();
    db.rename(test_file2.file_name()).unwrap();
    assert_eq!(db.filename(), test_file2.file_name());
}

#[test]
fn rename_file() {
    let test_file = TestFile::new();
    let test_file2 = TestFile::new();
    {
        let mut db = DbFile::new(test_file.file_name()).unwrap();
        db.exec_mut(QueryBuilder::insert().nodes().count(1).query())
            .unwrap();
        db.rename(test_file2.file_name()).unwrap();
        assert_eq!(
            db.exec(QueryBuilder::select().ids(1).query())
                .unwrap()
                .result,
            1
        );
    }
    let db = DbFile::new(test_file2.file_name()).unwrap();
    assert_eq!(
        db.exec(QueryBuilder::select().ids(1).query())
            .unwrap()
            .result,
        1
    );
}

#[test]
fn copy_memory() {
    let mut db = DbMemory::new("memdb").unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    let other = db.copy("mydb").unwrap();
    assert_eq!(other.filename(), "mydb");
    assert_eq!(
        other
            .exec(QueryBuilder::select().ids("root").query())
            .unwrap()
            .result,
        1
    );
}

#[test]
fn copy_mapped() {
    let test_file = TestFile::new();
    let test_file2 = TestFile::new();
    let mut db = Db::new(test_file.file_name()).unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    let other = db.copy(test_file2.file_name()).unwrap();
    assert_eq!(other.filename(), test_file2.file_name());
    assert_eq!(
        other
            .exec(QueryBuilder::select().ids("root").query())
            .unwrap()
            .result,
        1
    );
}

#[test]
fn copy_file() {
    let test_file = TestFile::new();
    let test_file2 = TestFile::new();
    let mut db = DbFile::new(test_file.file_name()).unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    let other = db.copy(test_file2.file_name()).unwrap();
    assert_eq!(other.filename(), test_file2.file_name());
    assert_eq!(
        other
            .exec(QueryBuilder::select().ids("root").query())
            .unwrap()
            .result,
        1
    );
}

#[test]
fn queries_as_reference() {
    let test_file = TestFile::new();
    let mut db = DbFile::new(test_file.file_name()).unwrap();

    let query = QueryBuilder::insert()
        .nodes()
        .aliases(["root", "users"])
        .query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::insert().aliases("root").ids("root").query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::insert()
        .edges()
        .from("root")
        .to("users")
        .query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::insert().index("username").query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::insert()
        .values([[("username", "admin").into()]])
        .ids("users")
        .query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::remove().aliases("new_root").query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::remove().index("username").query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::remove()
        .values("username")
        .ids("users")
        .query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::remove().ids("users").query();
    db.exec_mut(&query).unwrap();

    let query = QueryBuilder::search().from("root").query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().aliases().query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().aliases().ids(1).query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().edge_count().ids("root").query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().edge_count_from().ids("root").query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().edge_count_to().ids("root").query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().indexes().query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().key_count().ids("root").query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().keys().ids("root").query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().node_count().query();
    db.exec(&query).unwrap();

    let query = QueryBuilder::select().ids("root").query();
    db.exec(&query).unwrap();
}

#[test]
fn large_queries() {
    const COUNT: i64 = 10000;
    let mut db = DbMemory::new("test").unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().count(COUNT as u64).query())
        .unwrap();

    let mut now = Instant::now();
    let mut times = vec![];

    for (from, to) in (1..COUNT).step_by(2).zip((2..COUNT).step_by(2)) {
        let r = db
            .exec_mut(QueryBuilder::insert().edges().from(from).to(to).query())
            .unwrap();
        let vals: Vec<agdb::DbKeyValue> = (1..10)
            .map(|j| (format!("key{}", from + j), format!("value{}", from + j)).into())
            .collect();
        db.exec_mut(
            QueryBuilder::insert()
                .values([vals])
                .ids(r.elements[0].id)
                .query(),
        )
        .unwrap();

        if to % 1000 == 0 {
            times.push(now.elapsed());
            now = Instant::now();
        }
    }

    times.push(now.elapsed());

    let min = times.iter().min().unwrap();
    let max = times.iter().max().unwrap();
    let avg = times.iter().map(|t| t.as_millis() as u64).sum::<u64>() / times.len() as u64;

    println!("Min: {}ms", min.as_millis());
    println!("Max: {}ms", max.as_millis());
    println!("Avg: {avg}ms");
    println!("{times:?}");
}

#[test]
fn convert_db_before_0_11_0() {
    let test_file = TestFile::new();
    std::fs::copy("tests/test_db_prior_0_11_0.agdb", test_file.file_name()).unwrap();
    let db = Db::new(test_file.file_name()).unwrap();
    let result = db
        .exec(QueryBuilder::select().search().elements().query())
        .unwrap();
    assert_eq!(result.elements.len(), 3);
    assert_eq!(
        result.elements[0].values,
        vec![("key1", "value1").into(), ("key2", 123).into()]
    );
    assert_eq!(result.elements[1].values, vec![(1, 2).into()]);
    assert_eq!(result.elements[2].values, vec![("tag", "label").into()]);
}

#[test]
fn db_any() {
    let test_file1 = TestFile::new();
    let test_file2 = TestFile::new();

    let dbs = [
        DbAny::new_file(test_file1.file_name()).unwrap(),
        DbAny::new_mapped(test_file2.file_name()).unwrap(),
        DbAny::new_memory("memdb").unwrap(),
    ];

    let names = dbs.iter().map(|db| db.filename()).collect::<Vec<&str>>();

    assert_eq!(
        names,
        vec![test_file1.file_name(), test_file2.file_name(), "memdb"]
    );
}

#[test]
fn db_any_struct() {
    struct MyDb {
        db: DbAny,
    }

    let test_file = TestFile::new();
    let db = DbAny::new(test_file.file_name()).unwrap();
    let mut my_db = MyDb { db };

    assert_eq!(my_db.db.filename(), test_file.file_name());

    my_db = MyDb {
        db: DbAny::new_memory("memdb").unwrap(),
    };

    assert_eq!(my_db.db.filename(), "memdb");
}

#[test]
fn with_data() {
    let mut db = DbMemory::with_data(MemoryStorage::new("test").unwrap()).unwrap();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query())
        .unwrap();
    let count = db
        .exec(QueryBuilder::select().node_count().query())
        .unwrap()
        .elements[0]
        .values[0]
        .value
        .to_u64()
        .unwrap();

    assert_eq!(count, 1);
}

#[test]
fn using_ref_insert_element() {
    struct S {}

    impl agdb::DbType for S {
        type ValueType = Self;

        fn db_id(&self) -> Option<QueryId> {
            None
        }

        fn db_keys() -> Vec<agdb::DbValue> {
            vec![]
        }

        fn from_db_element(_element: &agdb::DbElement) -> Result<Self::ValueType, agdb::DbError> {
            Ok(S {})
        }

        fn to_db_values(&self) -> Vec<agdb::DbKeyValue> {
            vec![]
        }
    }

    let mut db = DbMemory::new("test").unwrap();
    let s = S {};
    db.exec_mut(QueryBuilder::insert().element(&s).query())
        .unwrap();
    db.exec_mut(QueryBuilder::insert().element(s).query())
        .unwrap();
}
