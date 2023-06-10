mod test_db;

use agdb::QueryBuilder;
use agdb::QueryError;
use test_db::TestDb;

#[test]
fn remove_edges_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into()])
            .query(),
        1,
    );
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
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
            t.exec_mut(&QueryBuilder::remove().ids(&[(-3).into()]).query())?;
            t.exec(&QueryBuilder::select().ids(&[(-3).into()]).query())
        },
        "Id '-3' not found".into(),
    );
    db.exec(QueryBuilder::select().ids(&[(-3).into()]).query(), 1);
}

#[test]
fn remove_edges() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(&["alias1".into()])
            .query(),
        1,
    );
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
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
fn remove_missing_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().ids(&[(-3).into()]).query(), 0);
}

#[test]
fn remove_missing_edges_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |transaction| -> Result<(), QueryError> {
            let query = QueryBuilder::remove().ids(&[(-3).into()]).query();
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
    db.exec_mut_ids(
        QueryBuilder::insert()
            .edges()
            .from(&[1.into()])
            .to(&[2.into()])
            .query(),
        &[-3],
    );

    db.exec_mut(
        QueryBuilder::remove()
            .search(QueryBuilder::search().from(1.into()).query())
            .query(),
        -2,
    );
    db.exec_error(
        QueryBuilder::select().ids(&[(-3).into()]).query(),
        "Id '-3' not found",
    );
}
