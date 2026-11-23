use agbench::Bench;
use agplatform::TestDir;
use std::path::PathBuf;
use std::process::ExitCode;

mod old {
    use agdb_base as agdb;
    include!("workload.rs");
}

mod new {
    include!("workload.rs");
}

const NODES: u64 = 1000;
const INDEXED: u64 = 100_000;
const LARGE: u64 = 64 << 10;

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
        select_element,
        select_value,
        select_alias,
        search_index,
        search_condition,
        search_order_by,
        select_search_values,
        open,
    ])
}

/// The same database created by each version in its own file.
struct Pair<O, N> {
    old: O,
    new: N,
    counters: [u64; 2],
    _dir: TestDir,
}

type Op<D> = fn(&mut D, &mut u64, u64) -> u64;

fn pair<O: agdb_base::StorageData, N: agdb::StorageData>(
    nodes: u64,
    index: bool,
) -> Pair<agdb_base::DbImpl<O>, agdb::DbImpl<N>> {
    let dir = TestDir::temp();

    Pair {
        old: old::create(&dir.path().join("old.agdb"), nodes, index),
        new: new::create(&dir.path().join("new.agdb"), nodes, index),
        counters: [nodes; 2],
        _dir: dir,
    }
}

fn compare<O: agdb_base::StorageData, N: agdb::StorageData>(
    bench: &mut Bench,
    name: &str,
    nodes: u64,
    index: bool,
    ops: (Op<agdb_base::DbImpl<O>>, Op<agdb::DbImpl<N>>),
    n: u64,
) {
    bench
        .benchmark(name, || pair::<O, N>(nodes, index))
        .target("base", move |p| (ops.0)(&mut p.old, &mut p.counters[0], n))
        .target("current", move |p| {
            (ops.1)(&mut p.new, &mut p.counters[1], n)
        })
        .run();
}

/// Compares `op` of both versions on each storage kind over `nodes` nodes
/// (indexed on `k` when `index`), `n` is passed to `op`.
macro_rules! compare {
    ($bench:ident, $name:literal, $nodes:expr, $index:expr, $op:ident, $n:expr) => {
        compare::<agdb_base::MemoryStorage, agdb::MemoryStorage>(
            $bench,
            concat!($name, "/memory"),
            $nodes,
            $index,
            (old::$op, new::$op),
            $n,
        );
        compare::<agdb_base::FileStorageMemoryMapped, agdb::FileStorageMemoryMapped>(
            $bench,
            concat!($name, "/mapped"),
            $nodes,
            $index,
            (old::$op, new::$op),
            $n,
        );
        compare::<agdb_base::FileStorage, agdb::FileStorage>(
            $bench,
            concat!($name, "/file"),
            $nodes,
            $index,
            (old::$op, new::$op),
            $n,
        );
    };
}

fn insert_node(bench: &mut Bench) {
    compare!(bench, "insert_node", NODES, false, insert_node, 0);
}

fn insert_indexed_node(bench: &mut Bench) {
    compare!(bench, "insert_indexed_node", NODES, true, insert_node, 0);
}

fn insert_edge(bench: &mut Bench) {
    compare!(bench, "insert_edge", NODES, false, insert_edge, NODES);
}

fn update_value(bench: &mut Bench) {
    compare!(bench, "update_value", NODES, false, update_value, NODES);
}

fn update_value_resize(bench: &mut Bench) {
    compare!(
        bench,
        "update_value_resize",
        NODES,
        false,
        update_value_resize,
        NODES
    );
}

fn insert_remove_node(bench: &mut Bench) {
    compare!(
        bench,
        "insert_remove_node",
        NODES,
        false,
        insert_remove_node,
        0
    );
}

fn insert_large_value(bench: &mut Bench) {
    compare!(
        bench,
        "insert_64kib_value",
        NODES,
        false,
        insert_large_value,
        LARGE
    );
}

fn transaction_nodes_edges(bench: &mut Bench) {
    compare!(
        bench,
        "transaction_100_nodes_100_edges",
        NODES,
        false,
        transaction,
        100
    );
}

fn insert_nodes_bulk(bench: &mut Bench) {
    compare!(
        bench,
        "insert_1000_nodes",
        NODES,
        false,
        insert_nodes,
        NODES
    );
}

fn remove_nodes_bulk(bench: &mut Bench) {
    compare!(
        bench,
        "insert_remove_1000_nodes",
        NODES,
        false,
        insert_remove_nodes,
        NODES
    );
}

fn select_element(bench: &mut Bench) {
    compare!(
        bench,
        "select_element",
        INDEXED,
        true,
        select_element,
        INDEXED
    );
}

fn select_value(bench: &mut Bench) {
    compare!(bench, "select_value", INDEXED, true, select_value, INDEXED);
}

fn select_alias(bench: &mut Bench) {
    compare!(bench, "select_alias", INDEXED, true, select_alias, 0);
}

fn search_index(bench: &mut Bench) {
    compare!(
        bench,
        "search_index_100k",
        INDEXED,
        true,
        search_index,
        INDEXED
    );
}

fn search_condition(bench: &mut Bench) {
    compare!(
        bench,
        "search_condition_1000",
        NODES,
        false,
        search_condition,
        NODES / 2
    );
}

fn search_order_by(bench: &mut Bench) {
    compare!(
        bench,
        "search_order_by_1000_limit_10",
        NODES,
        false,
        search_order_by,
        10
    );
}

fn select_search_values(bench: &mut Bench) {
    compare!(
        bench,
        "select_search_values_1000",
        NODES,
        false,
        select_search_values,
        0
    );
}

/// Both files closed, each target opens its own.
struct Files {
    old: PathBuf,
    new: PathBuf,
    _dir: TestDir,
}

fn files<O: agdb_base::StorageData, N: agdb::StorageData>(nodes: u64) -> Files {
    let dir = TestDir::temp();
    let old = dir.path().join("old.agdb");
    let new = dir.path().join("new.agdb");
    drop(old::create::<O>(&old, nodes, true));
    drop(new::create::<N>(&new, nodes, true));

    Files {
        old,
        new,
        _dir: dir,
    }
}

fn open_kind<O: agdb_base::StorageData, N: agdb::StorageData>(bench: &mut Bench, name: &str) {
    bench
        .benchmark(name, || files::<O, N>(INDEXED))
        .target("base", |f| drop(old::open::<O>(&f.old)))
        .target("current", |f| drop(new::open::<N>(&f.new)))
        .run();
}

fn open(bench: &mut Bench) {
    open_kind::<agdb_base::FileStorageMemoryMapped, agdb::FileStorageMemoryMapped>(
        bench,
        "open_100k/mapped",
    );
    open_kind::<agdb_base::FileStorage, agdb::FileStorage>(bench, "open_100k/file");
}
