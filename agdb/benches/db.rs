use agbench::Bench;
use agdb::DbId;
use agdb::DbImpl;
use agdb::DbKeyValue;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::QueryBuilder;
use agdb::StorageData;
use agplatform::TestDir;
use std::process::ExitCode;

const NODES: u64 = 1000;
const LARGE: usize = 64 << 10;

fn main() -> ExitCode {
    agbench::main(&[
        insert_node,
        insert_indexed_node,
        insert_edge,
        update_value,
        update_value_resize,
        insert_remove_node,
        insert_large_value,
        transaction_nodes_edges,
        insert_nodes_bulk,
        remove_nodes_bulk,
    ])
}

/// Each database kind in its own file of a temporary directory holding
/// the `root` node and `NODES` nodes with values.
struct Dbs {
    memory: DbImpl<MemoryStorage>,
    mapped: DbImpl<FileStorageMemoryMapped>,
    file: DbImpl<FileStorage>,
    counters: [u64; 3],
    _dir: TestDir,
}

fn values(i: u64) -> Vec<DbKeyValue> {
    vec![
        ("name", format!("node{i}")).into(),
        ("k", i).into(),
        ("tag", "abc").into(),
    ]
}

fn db<S: StorageData>(dir: &TestDir, name: &str, index: bool) -> DbImpl<S> {
    let path = dir.path().join(name);
    let db = DbImpl::new(path.to_str().expect("temp dir should be valid UTF-8"))
        .expect("db should be created");

    if index {
        db.exec_mut(QueryBuilder::insert().index("k").query())
            .unwrap();
    }

    db.exec_mut(QueryBuilder::insert().nodes().aliases("root").query())
        .unwrap();
    let nodes: Vec<Vec<DbKeyValue>> = (0..NODES).map(values).collect();
    db.exec_mut(QueryBuilder::insert().nodes().values(nodes).query())
        .unwrap();
    db
}

fn dbs_with(index: bool) -> Dbs {
    let dir = TestDir::temp();

    Dbs {
        memory: db(&dir, "memory.agdb", index),
        mapped: db(&dir, "mapped.agdb", index),
        file: db(&dir, "file.agdb", index),
        counters: [NODES; 3],
        _dir: dir,
    }
}

fn dbs() -> Dbs {
    dbs_with(false)
}

/// Runs `op` on each database kind with its own counter.
macro_rules! targets {
    ($bench:expr, $op:ident) => {
        $bench
            .target("memory", |d| $op(&mut d.memory, &mut d.counters[0]))
            .target("mapped", |d| $op(&mut d.mapped, &mut d.counters[1]))
            .target("file", |d| $op(&mut d.file, &mut d.counters[2]))
            .run()
    };
}

fn node<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) -> Vec<DbId> {
    *counter += 1;
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([values(*counter)])
            .query(),
    )
    .unwrap()
    .ids()
}

fn insert_node(bench: &mut Bench) {
    targets!(bench.benchmark("db/insert_node", dbs), node);
}

fn insert_indexed_node(bench: &mut Bench) {
    targets!(
        bench.benchmark("db/insert_indexed_node", || dbs_with(true)),
        node
    );
}

fn edge<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) {
    *counter += 1;
    let to = DbId(2 + (*counter % NODES) as i64);
    db.exec_mut(QueryBuilder::insert().edges().from("root").to(to).query())
        .unwrap();
}

fn insert_edge(bench: &mut Bench) {
    targets!(bench.benchmark("db/insert_edge", dbs), edge);
}

fn update<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) {
    *counter += 1;
    let id = DbId(2 + (*counter % NODES) as i64);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("k", *counter).into()]])
            .ids(id)
            .query(),
    )
    .unwrap();
}

fn update_value(bench: &mut Bench) {
    targets!(bench.benchmark("db/update_value", dbs), update);
}

fn update_resize<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) {
    *counter += 1;
    let id = DbId(2 + (*counter % NODES) as i64);
    let name = "x".repeat(16 + (*counter % 64) as usize);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("name", name).into()]])
            .ids(id)
            .query(),
    )
    .unwrap();
}

fn update_value_resize(bench: &mut Bench) {
    targets!(
        bench.benchmark("db/update_value_resize", dbs),
        update_resize
    );
}

fn insert_remove<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) {
    let ids = node(db, counter);
    db.exec_mut(QueryBuilder::remove().ids(ids).query())
        .unwrap();
}

fn insert_remove_node(bench: &mut Bench) {
    targets!(bench.benchmark("db/insert_remove_node", dbs), insert_remove);
}

fn large<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) {
    *counter += 1;
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("blob", vec![(*counter % 256) as u8; LARGE]).into()]])
            .query(),
    )
    .unwrap();
}

fn insert_large_value(bench: &mut Bench) {
    targets!(bench.benchmark("db/insert_64kib_value", dbs), large);
}

fn transaction<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) {
    *counter += 100;
    let start = *counter;
    db.transaction_mut(|t| -> Result<(), agdb::DbError> {
        let nodes: Vec<Vec<DbKeyValue>> = (start..start + 100).map(values).collect();
        let ids = t.exec_mut(QueryBuilder::insert().nodes().values(nodes).query())?;
        t.exec_mut(QueryBuilder::insert().edges().from("root").to(ids).query())?;
        Ok(())
    })
    .unwrap();
}

fn transaction_nodes_edges(bench: &mut Bench) {
    targets!(
        bench.benchmark("db/transaction_100_nodes_100_edges", dbs),
        transaction
    );
}

fn bulk<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) -> Vec<DbId> {
    *counter += NODES;
    let start = *counter;
    let nodes: Vec<Vec<DbKeyValue>> = (start..start + NODES).map(values).collect();
    db.exec_mut(QueryBuilder::insert().nodes().values(nodes).query())
        .unwrap()
        .ids()
}

fn insert_nodes_bulk(bench: &mut Bench) {
    targets!(bench.benchmark("db/insert_1000_nodes", dbs), bulk);
}

fn bulk_remove<S: StorageData>(db: &mut DbImpl<S>, counter: &mut u64) {
    let ids = bulk(db, counter);
    db.exec_mut(QueryBuilder::remove().ids(ids).query())
        .unwrap();
}

fn remove_nodes_bulk(bench: &mut Bench) {
    targets!(
        bench.benchmark("db/insert_remove_1000_nodes", dbs),
        bulk_remove
    );
}
