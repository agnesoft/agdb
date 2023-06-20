mod test_db;

use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryResult;
use test_db::TestDb;

#[test]
fn remove_nodes_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query(), 1);
    db.transaction_mut_error(
        |t| {
            t.exec_mut(&QueryBuilder::remove().ids("alias").query())
                .unwrap();
            t.exec(&QueryBuilder::select().ids(1).query())
        },
        "Id '1' not found".into(),
    );
    db.exec(QueryBuilder::select().ids("alias").query(), 1);
}

#[test]
fn remove_nodes() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["alias", "alias2"])
            .query(),
        2,
    );
    db.exec_mut(
        QueryBuilder::remove().ids(vec!["alias", "alias2"]).query(),
        -2,
    );
}

#[test]
fn remove_missing_nodes() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().ids(1).query(), 0);
}

#[test]
fn remove_missing_nodes_aliases() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::remove().ids("alias").query(), 0);
}

#[test]
fn remove_nodes_with_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query(), 1);
    db.exec_mut(QueryBuilder::remove().ids(1).query(), -1);
    db.exec_error(
        QueryBuilder::select().ids("alias").query(),
        "Alias 'alias' not found",
    );
}

#[test]
fn remove_nodes_no_alias_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().ids(1).query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_ids(QueryBuilder::select().ids(1).query(), &[1]);
}

#[test]
fn remove_missing_nodes_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().ids(1).query())?;
            Err("error".into())
        },
        "error".into(),
    );
}

#[test]
fn remove_missing_nodes_alias_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().ids("alias").query())?;
            Err("error".into())
        },
        "error".into(),
    );
}

#[test]
fn remove_nodes_with_edges() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from(vec![1, 2])
            .to(vec![2, 1])
            .query(),
        2,
    );
    db.exec_mut(QueryBuilder::remove().ids(1).query(), -1);
    db.exec_error(QueryBuilder::select().ids(-3).query(), "Id '-3' not found");
}

#[test]
fn remove_nodes_with_edges_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(QueryBuilder::insert().edges().from(1).to(1).query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(&QueryBuilder::remove().ids(1).query())?;
            Err("error".into())
        },
        "error".into(),
    );
    db.exec_ids(QueryBuilder::select().ids(-2).query(), &[-2]);
}

#[test]
fn remove_nodes_with_values() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![vec![("key", "value").into()]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            values: vec![("key", "value").into()],
        }],
    );
    db.exec_mut(QueryBuilder::remove().ids(1).query(), -1);
    db.exec_error(QueryBuilder::select().ids(1).query(), "Id '1' not found");
}

#[test]
fn remove_nodes_with_values_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values(vec![vec![("key", vec![1, 2, 3]).into()]])
            .query(),
        1,
    );
    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            values: vec![("key", vec![1, 2, 3]).into()],
        }],
    );

    db.transaction_mut_error(
        |t| -> Result<QueryResult, QueryError> {
            t.exec_mut(&QueryBuilder::remove().ids(1).query()).unwrap();
            t.exec(&QueryBuilder::select().ids(1).query())
        },
        QueryError::from("Id '1' not found"),
    );

    db.exec_elements(
        QueryBuilder::select().ids(1).query(),
        &[DbElement {
            id: DbId(1),
            values: vec![("key", vec![1, 2, 3]).into()],
        }],
    );
}

#[test]
fn remove_nodes_search() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut_ids(QueryBuilder::insert().edges().from(1).to(2).query(), &[-3]);

    db.exec_mut(
        QueryBuilder::remove()
            .search(QueryBuilder::search().from(1).query())
            .query(),
        -2,
    );
    db.exec_error(QueryBuilder::select().ids(1).query(), "Id '1' not found");
    db.exec_error(QueryBuilder::select().ids(2).query(), "Id '2' not found");
}
