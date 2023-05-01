mod framework;

use agdb::QueryBuilder;
use agdb::QueryError;
use framework::TestDb;

#[test]
fn remove_node() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut(QueryBuilder::remove().id(1).query(), -1);
}

#[test]
fn remove_node_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias").query(), 1);
    db.transaction_mut_error(
        |t| {
            t.exec_mut(&QueryBuilder::remove().id("alias").query())
                .unwrap();
            t.exec(&QueryBuilder::select().id(1).query())
        },
        "Id '1' not found".into(),
    );
    db.exec(QueryBuilder::select().id("alias").query(), 1);
}

#[test]
fn remove_nodes() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias".to_string(), "alias2".to_string()])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .ids(&["alias".into(), "alias2".into()])
            .query(),
        -2,
    );
}

#[test]
fn remove_missing_node() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().id(1).query(), 0);
}

#[test]
fn remove_missing_node_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().id("alias").query(), 0);
}

#[test]
fn remove_node_with_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias").query(), 1);
    db.exec_mut(QueryBuilder::remove().id(1).query(), -1);
    db.exec_error(
        QueryBuilder::select().id("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn remove_node_no_alias_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().id(1).query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_ids(QueryBuilder::select().id(1).query(), &[1]);
}

#[test]
fn remove_missing_node_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().id(1).query())?;
            Err("error".into())
        },
        "error".into(),
    );
}

#[test]
fn remove_missing_node_alias_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().id("alias").query())?;
            Err("error".into())
        },
        "error".into(),
    );
}

#[test]
fn remove_node_with_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(QueryBuilder::insert().edge().from(1).to(2).query(), 1);
    db.exec_mut(QueryBuilder::insert().edge().from(2).to(1).query(), 1);
    db.exec_mut(QueryBuilder::remove().id(1).query(), -1);
    db.exec_error(QueryBuilder::select().id(-3).query(), "Id '-3' not found");
}

#[test]
fn remove_node_with_edges_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut(QueryBuilder::insert().edge().from(1).to(1).query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().id(1).query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_ids(QueryBuilder::select().id(-2).query(), &[-2]);
}

#[test]
fn remove_search() {
    let _query = QueryBuilder::remove()
        .search(QueryBuilder::search().from("origin".into()).query())
        .query();
}
