mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::DbKeyValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use test_db::TestDb;

#[test]
fn insert_edges_from_to_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(QueryBuilder::insert().edges().from("alias1").to(2).query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_error(QueryBuilder::select().ids(-3).query(), "Id '-3' not found");
}

#[test]
fn insert_edges_missing_from() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut_error(
        QueryBuilder::insert().edges().from("alias").to(2).query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn insert_edges_from_to() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3", "alias4"])
            .query(),
        4,
    );
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to(["alias3", "alias4"])
            .each()
            .query(),
        &[-5, -6, -7, -8],
    );
}

#[test]
fn insert_edges_from_to_each() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3"])
            .query(),
        3,
    );
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to("alias3")
            .query(),
        &[-4, -5],
    );
}

#[test]
fn insert_edges_from_to_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3", "alias4"])
            .query(),
        4,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to(["alias3", "alias4"])
            .values([[("key", "value").into()], [("key", "value2").into()]])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids([-5, -6]).query(),
        &[
            DbElement {
                id: DbId(-5),
                from: Some(DbId(1)),
                to: Some(DbId(3)),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(-6),
                from: Some(DbId(2)),
                to: Some(DbId(4)),
                values: vec![("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_each_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3", "alias4"])
            .query(),
        4,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to(["alias3", "alias4"])
            .each()
            .values([
                [("key", "value1").into()],
                [("key", "value2").into()],
                [("key", "value3").into()],
                [("key", "value4").into()],
            ])
            .query(),
        4,
    );
    db.exec_elements(
        QueryBuilder::select().ids([-5, -6, -7, -8]).query(),
        &[
            DbElement {
                id: DbId(-5),
                from: Some(DbId(1)),
                to: Some(DbId(3)),
                values: vec![("key", "value1").into()],
            },
            DbElement {
                id: DbId(-6),
                from: Some(DbId(1)),
                to: Some(DbId(4)),
                values: vec![("key", "value2").into()],
            },
            DbElement {
                id: DbId(-7),
                from: Some(DbId(2)),
                to: Some(DbId(3)),
                values: vec![("key", "value3").into()],
            },
            DbElement {
                id: DbId(-8),
                from: Some(DbId(2)),
                to: Some(DbId(4)),
                values: vec![("key", "value4").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_each_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3", "alias4"])
            .query(),
        4,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to(["alias3", "alias4"])
            .each()
            .values_uniform([("key", "value").into(), ("key", "value2").into()])
            .query(),
        4,
    );
    db.exec_elements(
        QueryBuilder::select().ids([-5, -6, -7, -8]).query(),
        &[
            DbElement {
                id: DbId(-5),
                from: Some(DbId(1)),
                to: Some(DbId(3)),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-6),
                from: Some(DbId(1)),
                to: Some(DbId(4)),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-7),
                from: Some(DbId(2)),
                to: Some(DbId(3)),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-8),
                from: Some(DbId(2)),
                to: Some(DbId(4)),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_values_bad_length() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3", "alias4"])
            .query(),
        4,
    );
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to(["alias3", "alias4"])
            .values([[("key", "value").into()]])
            .query(),
        "Values len '1' do not match the insert count '2'",
    );
}

#[test]
fn insert_edges_from_to_values_each_bad_length() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3", "alias4"])
            .query(),
        4,
    );
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to(["alias3", "alias4"])
            .each()
            .values([[("key", "value").into()]])
            .query(),
        "Values len '1' do not match the insert count '4'",
    );
}

#[test]
fn insert_edges_from_to_values_asymmetric() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3"])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to("alias3")
            .values([[("key", "value").into()], [("key", "value2").into()]])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids([-4, -5]).query(),
        &[
            DbElement {
                id: DbId(-4),
                from: Some(DbId(1)),
                to: Some(DbId(3)),
                values: vec![("key", "value").into()],
            },
            DbElement {
                id: DbId(-5),
                from: Some(DbId(2)),
                to: Some(DbId(3)),
                values: vec![("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_values_uniform() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(["alias1", "alias2", "alias3"])
            .query(),
        3,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(["alias1", "alias2"])
            .to("alias3")
            .values_uniform([("key", "value").into(), ("key", "value2").into()])
            .query(),
        2,
    );
    db.exec_elements(
        QueryBuilder::select().ids([-4, -5]).query(),
        &[
            DbElement {
                id: DbId(-4),
                from: Some(DbId(1)),
                to: Some(DbId(3)),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
            DbElement {
                id: DbId(-5),
                from: Some(DbId(2)),
                to: Some(DbId(3)),
                values: vec![("key", "value").into(), ("key", "value2").into()],
            },
        ],
    );
}

#[test]
fn insert_edges_from_to_search() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(4).query(), 4);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from([1, 2])
            .to([3, 4])
            .query(),
        2,
    );
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from(QueryBuilder::search().from(1).query())
            .to(QueryBuilder::search().from(3).query())
            .query(),
        &[-7, -8],
    );
}

#[test]
fn insert_edges_from_to_inserted_nodes() {
    let mut db = TestDb::new();
    let from = db.exec_mut_result(QueryBuilder::insert().nodes().count(2).query());
    let to = db.exec_mut_result(QueryBuilder::insert().nodes().count(2).query());

    db.exec_mut_ids(
        QueryBuilder::insert().edges().from(from).to(to).query(),
        &[-5, -6],
    );
}

#[test]
fn insert_or_update_new_edge() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .ids(QueryBuilder::search().from(1).where_().edge().query())
            .from(1)
            .to(2)
            .query(),
        &[-3],
    );
}

#[test]
fn insert_or_update_existing_edge() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query(), 1);
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .ids(-3)
            .from(1)
            .to(2)
            .values([[("key", 1).into()]])
            .query(),
        &[-3],
    );
}

#[test]
fn insert_or_update_mismatch_length() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(2).query(), 1);
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .ids(-3)
            .from(1)
            .to(2)
            .values(Vec::<Vec<DbKeyValue>>::new())
            .query(),
        "Values len '0' do not match the insert count '1'",
    );
}

#[test]
fn insert_or_update_unknown_edge() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .ids(-3)
            .from(1)
            .to(2)
            .values([[("k", 1).into()]])
            .query(),
        "Id '-3' not found",
    );
}

#[test]
fn insert_or_update_node_id() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut_error(
        QueryBuilder::insert()
            .edges()
            .ids(1)
            .from(1)
            .to(2)
            .values([[("k", 1).into()]])
            .query(),
        "The ids for insert or update must all refer to edges - node id '1' found",
    );
}
