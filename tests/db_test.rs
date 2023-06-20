mod test_db;

use agdb::Comparison;
use agdb::Db;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryId;
use test_db::test_file::TestFile;

#[allow(unused_imports)]
#[test]
fn public_types() {
    use agdb::Comparison;
    use agdb::CountComparison;
    use agdb::Db;
    use agdb::DbElement;
    use agdb::DbError;
    use agdb::DbId;
    use agdb::DbKey;
    use agdb::DbKeyOrder;
    use agdb::DbKeyValue;
    use agdb::DbValue;
    use agdb::InsertAliasesQuery;
    use agdb::InsertEdgesQuery;
    use agdb::InsertNodesQuery;
    use agdb::InsertValuesQuery;
    use agdb::Query;
    use agdb::QueryBuilder;
    use agdb::QueryCondition;
    use agdb::QueryError;
    use agdb::QueryId;
    use agdb::QueryIds;
    use agdb::QueryMut;
    use agdb::QueryResult;
    use agdb::QueryValues;
    use agdb::RemoveAliasesQuery;
    use agdb::RemoveQuery;
    use agdb::RemoveValuesQuery;
    use agdb::SearchQuery;
    use agdb::SelectAllAliases;
    use agdb::SelectKeyCountQuery;
    use agdb::SelectKeysQuery;
    use agdb::SelectQuery;
    use agdb::SelectValuesQuery;
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
            &QueryBuilder::insert()
                .nodes()
                .aliases(vec!["alias", "alias2"])
                .values_uniform(values.clone())
                .query(),
        )
        .unwrap();
        db.exec_mut(&QueryBuilder::insert().edges().from(1).to(2).query())
            .unwrap();
        let result = db
            .exec(
                &QueryBuilder::select()
                    .ids(vec![QueryId::from("alias"), "alias2".into(), (-3).into()])
                    .query(),
            )
            .unwrap();

        assert_eq!(
            result.elements,
            &[
                DbElement {
                    id: DbId(1),
                    values: values.clone(),
                },
                DbElement {
                    id: DbId(2),
                    values: values.clone(),
                },
                DbElement {
                    id: DbId(-3),
                    values: vec![],
                }
            ]
        );
    }

    let db = Db::new(test_file.file_name()).unwrap();
    let result = db
        .exec(
            &QueryBuilder::select()
                .ids(vec![QueryId::from("alias"), "alias2".into(), (-3).into()])
                .query(),
        )
        .unwrap();

    assert_eq!(
        result.elements,
        &[
            DbElement {
                id: DbId(1),
                values: values.clone(),
            },
            DbElement {
                id: DbId(2),
                values,
            },
            DbElement {
                id: DbId(-3),
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
            &QueryBuilder::insert()
                .nodes()
                .aliases(vec!["alias", "alias2"])
                .values_uniform(vec![("key", 100).into()])
                .query(),
        )
        .unwrap();
        db.exec_mut(&QueryBuilder::insert().edges().from(1).to(2).query())
            .unwrap();
        let result = db
            .exec(
                &QueryBuilder::select()
                    .ids(vec![QueryId::from("alias"), "alias2".into(), (-3).into()])
                    .query(),
            )
            .unwrap();

        assert_eq!(
            result.elements,
            &[
                DbElement {
                    id: DbId(1),
                    values: vec![("key", 100).into()],
                },
                DbElement {
                    id: DbId(2),
                    values: vec![("key", 100).into()],
                },
                DbElement {
                    id: DbId(-3),
                    values: vec![],
                }
            ]
        );

        db.exec_mut(&QueryBuilder::remove().ids(-3).query())
            .unwrap();
        db.exec_mut(
            &QueryBuilder::remove()
                .values(vec!["key".into()])
                .ids(1)
                .query(),
        )
        .unwrap();
    }

    let db = Db::new(test_file.file_name()).unwrap();
    let result = db
        .exec(&QueryBuilder::select().ids(vec!["alias", "alias2"]).query())
        .unwrap();

    assert_eq!(
        result.elements,
        &[
            DbElement {
                id: DbId(1),
                values: vec![],
            },
            DbElement {
                id: DbId(2),
                values: vec![("key", 100).into()],
            }
        ]
    );

    let error = db
        .exec(&QueryBuilder::select().ids(-3).query())
        .unwrap_err();
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
        format!(
            "File '{}' is not a valid database file and is not empty.",
            test_file.file_name()
        )
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
                &QueryBuilder::insert()
                    .nodes()
                    .count(1000)
                    .values_uniform(vec![("key", "value").into()])
                    .query(),
            )
            .unwrap();
        db.exec_mut(&QueryBuilder::remove().ids(result.ids()).query())
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

#[rustfmt::skip]
#[test]
fn quickstart() {
    let _ = std::fs::remove_file("db_file.agdb");
    let mut db = Db::new("db_file.agdb").unwrap();

    //create a nodes with data
    db.exec_mut(&QueryBuilder::insert().nodes().aliases("users").query()).unwrap();
    let users = db.exec_mut(&QueryBuilder::insert().nodes().values(vec![
        vec![("id", 1).into(), ("username", "user_1").into()],
        vec![("id", 2).into(), ("username", "user_2").into()],
        vec![("id", 3).into(), ("username", "user_3").into()]]
    ).query()).unwrap();

    //connect nodes
    db.exec_mut(&QueryBuilder::insert().edges().from("users").to(users.ids()).query()).unwrap();

    //select nodes
    let user_elements = db.exec(&QueryBuilder::select().ids(users.ids()).query()).unwrap();

    for element in user_elements.elements {
        println!("{:?}: {:?}", element.id, element.values);
    }

    // DbId(2): [DbKeyValue { key: String("id"), value: Int(1) }, DbKeyValue { key: String("username"), value: String("user_1") }]
    // DbId(3): [DbKeyValue { key: String("id"), value: Int(2) }, DbKeyValue { key: String("username"), value: String("user_2") }]
    // DbId(4): [DbKeyValue { key: String("id"), value: Int(3) }, DbKeyValue { key: String("username"), value: String("user_3") }]

    //search with conditions
    let user_id = db.exec(&QueryBuilder::search().from("users").where_().key("username").value(Comparison::Equal("user_2".into())).query()).unwrap();

    println!("{:?}", user_id.elements);
    //[DbElement { id: DbId(3), values: [] }]
    let _ = std::fs::remove_file("db_file.agdb");
}
