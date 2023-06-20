mod test_db;

use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryId;
use test_db::TestDb;

#[test]
fn insert_aliases_of() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().count(2).query(), 2);
    db.exec_mut(
        QueryBuilder::insert()
            .aliases(vec!["alias", "alias2"])
            .of(vec![1, 2])
            .query(),
        2,
    );
}

#[test]
fn insert_aliases_of_alias() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query(), 1);
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .aliases(vec!["alias1", "alias2"])
            .of(vec![QueryId::from("alias"), 2.into()])
            .query(),
        2,
    );
}

#[test]
fn insert_aliases_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().nodes().aliases("alias").query(), 1);
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(
                &QueryBuilder::insert()
                    .aliases(vec!["alias1", "alias2"])
                    .of(vec![QueryId::from("alias"), 2.into()])
                    .query(),
            )?;

            // This fails and causes a rollback
            // since the alias was overwritten
            // in the transaction.
            t.exec(&QueryBuilder::select().ids("alias").query())?;
            Ok(())
        },
        "Alias 'alias' not found".into(),
    );

    db.exec(QueryBuilder::select().ids("alias").query(), 1);
}

#[test]
fn insert_aliases_empty_alias() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert().aliases(String::new()).of(1).query(),
        "Empty alias is not allowed",
    );
}

#[test]
fn insert_aliases_ids_mismatched_length() {
    let mut db = TestDb::new();
    db.exec_mut_error(
        QueryBuilder::insert()
            .aliases(String::new())
            .of(vec![1, 2])
            .query(),
        "Ids and aliases must have the same length",
    );
}
