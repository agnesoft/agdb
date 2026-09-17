#[path = "../src/test_utilities/test_file.rs"]
mod test_file;

use agdb::DbError;
use agdb::DbImpl;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::QueryBuilder;
use agdb::StorageData;
use agdb::SyncMode;
use std::process::Command;
use test_file::TestFile;

fn open<Store: StorageData>(path: &str, sync: &str) -> DbImpl<Store> {
    let mut db = DbImpl::<Store>::new(path).unwrap();
    if sync == "commit" {
        db.set_sync_mode(SyncMode::Commit);
    }
    db
}

fn child<Store: StorageData>(path: &str, sync: &str, action: &str) {
    let mut db = open::<Store>(path, sync);
    if action == "seed" {
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases("state")
                .values_uniform(vec![("a", 0_i64).into(), ("b", 0_i64).into()])
                .query(),
        )
        .unwrap();
        return;
    }
    if action == "read" || action == "read-committed" {
        let expected = if action == "read-committed" { 2 } else { 0 };
        let result = db
            .exec(QueryBuilder::select().ids("state").query())
            .unwrap();
        assert_eq!(result.elements.len(), 1);
        assert_eq!(
            result.elements[0].values,
            vec![("a", expected).into(), ("b", expected).into()]
        );
        return;
    }
    let result = db.transaction_mut(|tx| -> Result<(), DbError> {
        tx.exec_mut(
            QueryBuilder::insert()
                .values_uniform(vec![("a", 1_i64).into()])
                .ids("state")
                .query(),
        )?;
        match action {
            "after-first" => std::process::exit(86),
            "panic-first" => panic!("intentional transaction interruption"),
            _ => {}
        }
        tx.exec_mut(
            QueryBuilder::insert()
                .values_uniform(vec![("a", 2_i64).into()])
                .ids("state")
                .query(),
        )?;
        if action == "after-repeat" {
            std::process::exit(86);
        }
        tx.exec_mut(
            QueryBuilder::insert()
                .values_uniform(vec![("b", 2_i64).into()])
                .ids("state")
                .query(),
        )?;
        if action == "before-commit" {
            std::process::exit(86);
        }
        if action == "return-error" {
            return Err(DbError::from(std::io::Error::other("intentional rollback")));
        }
        Ok(())
    });
    if action == "return-error" {
        assert!(result.is_err());
    } else {
        result.unwrap();
    }
}

#[test]
fn transaction_child() {
    let Ok(path) = std::env::var("AGDB_RECOVERY_TEST_PATH") else {
        return;
    };
    let backend = std::env::var("AGDB_RECOVERY_TEST_BACKEND").unwrap();
    let sync = std::env::var("AGDB_RECOVERY_TEST_SYNC").unwrap();
    let action = std::env::var("AGDB_RECOVERY_TEST_ACTION").unwrap();
    match backend.as_str() {
        "file" => child::<FileStorage>(&path, &sync, &action),
        "mapped" => child::<FileStorageMemoryMapped>(&path, &sync, &action),
        _ => panic!("unknown backend"),
    }
}

fn subprocess(path: &str, backend: &str, sync: &str, action: &str, expected: i32) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "transaction_child", "--nocapture"])
        .env("AGDB_RECOVERY_TEST_PATH", path)
        .env("AGDB_RECOVERY_TEST_BACKEND", backend)
        .env("AGDB_RECOVERY_TEST_SYNC", sync)
        .env("AGDB_RECOVERY_TEST_ACTION", action)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{backend}/{sync}/{action}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn recovery(action: &str) {
    for backend in ["file", "mapped"] {
        for sync in ["none", "commit"] {
            let file = TestFile::from(format!(
                "transaction-recovery-{backend}-{sync}-{action}.testfile"
            ));
            subprocess(file.file_name(), backend, sync, "seed", 0);
            let exit_code = match action {
                "panic-first" => 101,
                "commit" | "return-error" => 0,
                _ => 86,
            };
            subprocess(file.file_name(), backend, sync, action, exit_code);
            let read = if action == "commit" {
                "read-committed"
            } else {
                "read"
            };
            subprocess(file.file_name(), backend, sync, read, 0);
        }
    }
}

#[test]
fn exit_after_first_write() {
    recovery("after-first");
}

#[test]
fn exit_after_repeated_write() {
    recovery("after-repeat");
}

#[test]
fn exit_before_commit() {
    recovery("before-commit");
}

#[test]
fn panic_after_first_write() {
    recovery("panic-first");
}

#[test]
fn returned_error_rolls_back() {
    recovery("return-error");
}

#[test]
fn successful_commit_persists() {
    recovery("commit");
}
