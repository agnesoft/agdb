mod test_db;

use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryId;
use test_db::TestDb;

#[test]
fn remove_edges_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(
        QueryBuilder::insert().edges().from("alias1").to(2).query(),
        1,
    );
    db.transaction_mut_error(
        |t| {
            t.exec_mut(QueryBuilder::remove().ids(-3).query())?;
            t.exec(QueryBuilder::select().ids(-3).query())
        },
        "Id '-3' not found".into(),
    );
    db.exec(QueryBuilder::select().ids(-3).query(), 1);
}

#[test]
fn remove_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias1").query(), 1);
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from([QueryId::from("alias1"), 2.into()])
            .to([QueryId::from(2), "alias1".into()])
            .query(),
        2,
    );
    db.exec_mut(QueryBuilder::remove().ids([-3, -4]).query(), -2);
}

#[test]
fn remove_missing_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().ids(-3).query(), 0);
}

#[test]
fn remove_missing_edges_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |transaction| -> Result<(), QueryError> {
            let query = QueryBuilder::remove().ids(-3).query();
            transaction.exec_mut(&query).unwrap();
            Err("error".into())
        },
        "error".into(),
    );
}

#[test]
fn remove_edges_search() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut_ids(QueryBuilder::insert().edges().from(1).to(2).query(), &[-3]);

    db.exec_mut(
        QueryBuilder::remove()
            .ids(QueryBuilder::search().from(1).query())
            .query(),
        -2,
    );
    db.exec_error(QueryBuilder::select().ids(-3).query(), "Id '-3' not found");
}
