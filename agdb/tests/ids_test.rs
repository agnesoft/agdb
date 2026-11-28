mod test_db;

use agdb::Db;
use agdb::DbError;
use agdb::DbErrorType;
use agdb::DbId;
use agdb::QueryBuilder;
use test_db::TestFile;

fn insert_nodes(db: &mut Db, count: u64) -> Vec<i64> {
    db.exec_mut(QueryBuilder::insert().nodes().count(count).query())
        .unwrap()
        .ids()
        .into_iter()
        .map(|id| id.0)
        .collect()
}

fn remove(db: &mut Db, ids: &[i64]) {
    for id in ids {
        db.exec_mut(QueryBuilder::remove().ids(DbId(*id)).query())
            .unwrap();
    }
}

fn abort() -> DbError {
    DbError::db(DbErrorType::NotAllowed, "abort")
}

#[test]
fn insert_after_removes_should_reuse_last_removed_first() {
    let test_file = TestFile::new();
    let mut db = Db::new(test_file.file_name()).unwrap();
    insert_nodes(&mut db, 5);
    remove(&mut db, &[2, 4]);

    assert_eq!(insert_nodes(&mut db, 3), [4, 2, 6]);
}

#[test]
fn insert_in_transaction_of_removes_should_reuse_them_first() {
    let test_file = TestFile::new();
    let mut db = Db::new(test_file.file_name()).unwrap();
    insert_nodes(&mut db, 5);
    remove(&mut db, &[1]);

    let ids = db
        .transaction_mut(|t| -> Result<Vec<i64>, DbError> {
            t.exec_mut(QueryBuilder::remove().ids([3, 5]).query())?;
            let ids = t
                .exec_mut(QueryBuilder::insert().nodes().count(4).query())?
                .ids();
            Ok(ids.into_iter().map(|id| id.0).collect())
        })
        .unwrap();

    assert_eq!(ids, [5, 3, 1, 6]);
}

#[test]
fn aborted_transaction_should_release_ids() {
    let test_file = TestFile::new();
    let mut db = Db::new(test_file.file_name()).unwrap();
    insert_nodes(&mut db, 5);
    remove(&mut db, &[2, 4]);

    let aborted = db.transaction_mut(|t| -> Result<(), DbError> {
        t.exec_mut(QueryBuilder::insert().nodes().count(4).query())?;
        t.exec_mut(QueryBuilder::insert().edges().from(1).to([3, 5]).query())?;
        Err(abort())
    });

    assert_eq!(aborted, Err(abort()));
    assert_eq!(insert_nodes(&mut db, 3), [4, 2, 6]);
    assert_eq!(
        db.exec_mut(QueryBuilder::insert().edges().from(1).to(3).query())
            .unwrap()
            .ids(),
        [DbId(-1)]
    );
}

#[test]
fn failed_query_should_release_ids() {
    let test_file = TestFile::new();
    let mut db = Db::new(test_file.file_name()).unwrap();
    insert_nodes(&mut db, 2);

    let failed = db.transaction_mut(|t| -> Result<(), DbError> {
        t.exec_mut(QueryBuilder::insert().nodes().count(2).query())?;
        t.exec_mut(QueryBuilder::insert().edges().from(1).to(10).query())?;
        Ok(())
    });

    assert!(failed.is_err());
    assert_eq!(insert_nodes(&mut db, 1), [3]);
}

#[test]
fn reopen_after_aborted_transaction_should_keep_free_slots() {
    let test_file = TestFile::new();
    {
        let mut db = Db::new(test_file.file_name()).unwrap();
        insert_nodes(&mut db, 5);
        remove(&mut db, &[2, 4]);
        let _ = db.transaction_mut(|t| -> Result<(), DbError> {
            t.exec_mut(QueryBuilder::insert().nodes().count(3).query())?;
            Err(abort())
        });
    }

    let mut db = Db::new(test_file.file_name()).unwrap();

    assert_eq!(insert_nodes(&mut db, 3), [4, 2, 6]);
}

#[test]
fn optimize_storage_should_hand_out_free_slots_of_optimized() {
    let test_file = TestFile::new();
    {
        let mut db = Db::new(test_file.file_name()).unwrap();
        insert_nodes(&mut db, 8);
        remove(&mut db, &[7, 5, 3, 1]);
    }
    let mut db = Db::new(test_file.file_name()).unwrap();
    assert_eq!(insert_nodes(&mut db, 1), [1]);

    db.optimize_storage().unwrap();

    assert_eq!(insert_nodes(&mut db, 4), [7, 5, 3, 9]);
}

#[test]
fn restore_should_hand_out_free_slots_of_restored() {
    let test_file = TestFile::new();
    let backup_file = TestFile::from("./restore_should_hand_out_free_slots_of_restored.agdb");
    let mut db = Db::new(test_file.file_name()).unwrap();
    insert_nodes(&mut db, 4);
    remove(&mut db, &[2]);
    db.backup(backup_file.file_name()).unwrap();
    remove(&mut db, &[3]);

    db.restore(backup_file.file_name()).unwrap();

    assert_eq!(insert_nodes(&mut db, 2), [2, 5]);
}
