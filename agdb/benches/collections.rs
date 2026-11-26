use agbench::Bench;
use agdb::DbImpl;
use agdb::DbKeyValue;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::QueryBuilder;
use agdb::StorageData;
use agplatform::TestDir;
use std::process::ExitCode;

const ALIASES: u64 = 1000;
const INDEXED: u64 = 10_000;

fn main() -> ExitCode {
    agbench::main(&[aliases, index_update])
}

/// Each database kind in its own file of a temporary directory.
struct Dbs {
    mapped: DbImpl<FileStorageMemoryMapped>,
    memory: DbImpl<MemoryStorage>,
    file: DbImpl<FileStorage>,
    next: [u64; 3],
    _dir: TestDir,
}

fn db<S: StorageData>(dir: &TestDir, name: &str) -> DbImpl<S> {
    let path = dir.path().join(name);
    DbImpl::new(path.to_str().expect("temp dir should be valid UTF-8"))
        .expect("db should be created")
}

fn dbs() -> Dbs {
    let dir = TestDir::temp();

    Dbs {
        mapped: db(&dir, "mapped.agdb"),
        memory: db(&dir, "memory.agdb"),
        file: db(&dir, "file.agdb"),
        next: [INDEXED; 3],
        _dir: dir,
    }
}

fn insert_remove_aliases<S: StorageData>(db: &DbImpl<S>) {
    let aliases: Vec<String> = (0..ALIASES).map(|i| format!("alias{i}")).collect();
    db.exec_mut(QueryBuilder::insert().nodes().aliases(&aliases).query())
        .unwrap();
    db.exec_mut(QueryBuilder::remove().ids(&aliases).query())
        .unwrap();
}

fn aliases(bench: &mut Bench) {
    bench
        .benchmark("collections/insert_remove_1000_aliases", dbs)
        .target("mapped", |d| insert_remove_aliases(&d.mapped))
        .target("memory", |d| insert_remove_aliases(&d.memory))
        .target("file", |d| insert_remove_aliases(&d.file))
        .run();
}

fn indexed<S: StorageData>(db: &DbImpl<S>) {
    db.exec_mut(QueryBuilder::insert().index("key").query())
        .unwrap();
    let values: Vec<Vec<DbKeyValue>> = (0..INDEXED)
        .map(|i| vec![("key", spread(i)).into()])
        .collect();
    db.exec_mut(QueryBuilder::insert().nodes().values(values).query())
        .unwrap();
}

/// Indexed values spread over the hash space rather than sequential ones,
/// which would each land in their own slot.
fn spread(i: u64) -> u64 {
    i.wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

fn update_and_search<S: StorageData>(db: &DbImpl<S>, next: &mut u64) {
    let id = (*next % INDEXED) as i64 + 1;
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("key", spread(*next)).into()]])
            .ids(id)
            .query(),
    )
    .unwrap();
    *next += 1;
    db.exec(
        QueryBuilder::search()
            .index("key")
            .value(spread(u64::MAX))
            .query(),
    )
    .unwrap();
}

fn index_update(bench: &mut Bench) {
    bench
        .benchmark("collections/index_update_and_search_miss", || {
            let d = dbs();
            indexed(&d.mapped);
            indexed(&d.memory);
            indexed(&d.file);
            d
        })
        .target("mapped", |d| update_and_search(&d.mapped, &mut d.next[0]))
        .target("memory", |d| update_and_search(&d.memory, &mut d.next[1]))
        .target("file", |d| update_and_search(&d.file, &mut d.next[2]))
        .run();
}
