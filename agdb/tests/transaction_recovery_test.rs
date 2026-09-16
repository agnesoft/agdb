mod test_db;

use agdb::DbFile;
use agdb::QueryBuilder;
use test_db::TestFile;

#[test]
fn crash_recovery_restores_pre_transaction_state() {
    const N: usize = 50;

    if std::env::var("__AGDB_CRASH_TEST").is_ok() {
        let path = std::env::var("__AGDB_CRASH_DB").unwrap();
        let mut db = DbFile::new(&path).unwrap();
        let _ = db.transaction_mut(|t| -> Result<(), agdb::DbError> {
            for i in 0..N {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values(vec![vec![(format!("k{i}"), format!("corrupt_{i}")).into()]])
                        .ids("node")
                        .query(),
                )?;
            }
            t.exec_mut(
                QueryBuilder::insert()
                    .values(vec![vec![("k0", "overwritten_again").into()]])
                    .ids("node")
                    .query(),
            )?;
            t.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases("ghost")
                    .values(vec![vec![("x", "should_not_exist").into()]])
                    .query(),
            )?;
            std::process::exit(1);
        });
    }

    let test_file = TestFile::new();
    let db_path = test_file.file_name().clone();

    let mut original_values: Vec<agdb::DbKeyValue> = (0..N)
        .map(|i| (format!("k{i}"), format!("orig_{i}")).into())
        .collect();
    original_values.sort_by_key(|a| a.key.to_string());

    {
        let mut db = DbFile::new(&db_path).unwrap();
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases("node")
                .values(vec![original_values.clone()])
                .query(),
        )
        .unwrap();
    }

    let status = spawn_crash_child(&db_path, "crash_recovery_restores_pre_transaction_state");
    assert!(!status.success());

    let db = DbFile::new(&db_path).unwrap();
    let r = db.exec(QueryBuilder::select().ids("node").query()).unwrap();
    let mut values = r.elements[0].values.clone();
    values.sort_by_key(|a| a.key.to_string());
    assert_eq!(
        values, original_values,
        "all values must be restored to originals after crash"
    );

    assert!(
        db.exec(QueryBuilder::select().ids("ghost").query())
            .is_err(),
        "node created inside crashed transaction must not exist"
    );
}

fn spawn_crash_child(db_path: &str, test_name: &str) -> std::process::ExitStatus {
    let exe = std::env::current_exe().unwrap();
    std::process::Command::new(&exe)
        .env("__AGDB_CRASH_TEST", "1")
        .env("__AGDB_CRASH_DB", db_path)
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture")
        .status()
        .expect("failed to spawn child")
}
