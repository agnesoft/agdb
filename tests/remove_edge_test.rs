mod framework;

use agdb::QueryBuilder;
use agdb::QueryError;
use framework::TestDb;

#[test]
fn remove_edges_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into()])
            .to(&[2.into()])
            .query(),
        1,
    );
    db.transaction_mut_error(
        |t| {
            t.exec_mut(&QueryBuilder::remove().id(-3).query())?;
            t.exec(&QueryBuilder::select().id(-3).query())
        },
        "Id '-3' not found".into(),
    );
    db.exec(QueryBuilder::select().id(-3).query(), 1);
}

#[test]
fn remove_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().node().alias("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().node().query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(&["alias1".into(), 2.into()])
            .to(&[2.into(), "alias1".into()])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove()
            .ids(&[(-3).into(), (-4).into()])
            .query(),
        -2,
    );
}

#[test]
fn remove_missing_edge() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().id(-3).query(), 0);
}

#[test]
fn remove_missing_edge_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |transaction| -> Result<(), QueryError> {
            let query = QueryBuilder::remove().id(-3).query();
            transaction.exec_mut(&query).unwrap();
            Err("error".into())
        },
        "error".into(),
    );
}
