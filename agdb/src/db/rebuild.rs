//! Copies a database into a new storage element by element, keeping the
//! ids, in transactions of [`BATCH`] items: the migration of the legacy
//! format and the optimization of the storage.

use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::StorageData;
use crate::Transaction;
use crate::collections::graph::Edge;
use crate::collections::graph::Node;
use crate::collections::key_values::KeyValues;
use crate::collections::keys::ALIAS;
use crate::db::db_migration::DbMigration;
use crate::db::legacy::Legacy;
use crate::storage::pages::Reader;
use crate::storage::root::Layout;
use crate::storage::root::Root;
use crate::storage::root::RootMut;
use crate::transaction::ReadView;

const BATCH: u64 = 4096;
/// The database version written by the rebuild.
const VERSION: u64 = 2;

/// Runs `f` in a transaction of the `root` that is committed if it
/// succeeds. `f` is dynamic so every batch shares one instantiation.
fn batch<S: StorageData>(
    root: &mut Root<S>,
    f: &mut dyn FnMut(&mut RootMut<'_, S>) -> Result<(), DbError>,
) -> Result<(), DbError> {
    let mut transaction = root.begin()?;
    f(&mut transaction)?;
    transaction.commit()
}

/// The node and the edge of a graph slot: ids are slots, so a slot holds
/// at most one of each.
pub(crate) type Element = (Option<Node>, Option<Edge>);

/// A database read by the rebuild.
pub(crate) trait Source {
    /// The graph slots, the unused slot 0 included.
    fn slots(&self) -> u64;

    /// The `count` elements from the slot `start`, none past the graph.
    /// The links of the edges to the previous ones and the edge counts of
    /// the nodes are ignored.
    fn graph(&self, start: u64, count: u64) -> Result<Vec<Element>, DbError>;

    /// The keys of the indexes in their order.
    fn indexes(&self) -> Result<Vec<DbValue>, DbError>;

    /// The aliases of nodes, reporting what it drops as `fixed`.
    fn aliases(&self, fixed: &mut Vec<String>) -> Result<Vec<(DbId, String)>, DbError>;

    /// The slots that [`values`](Self::values) reads.
    fn value_slots(&self) -> Result<u64, DbError>;

    /// The key-values of the existing elements in the value slots
    /// `start..start + count`, reporting what it drops as `fixed`.
    fn values(
        &self,
        start: u64,
        count: u64,
        fixed: &mut Vec<String>,
    ) -> Result<Vec<(DbId, DbKeyValue)>, DbError>;
}

/// Copies the `source` into the empty `root`, reporting what it drops as
/// `fixed`. Each array of the graph ends at its highest element, so the
/// free slots above it are dropped.
pub(crate) fn rebuild<S: StorageData>(
    source: &impl Source,
    root: &mut Root<S>,
    fixed: &mut Vec<String>,
) -> Result<(), DbError> {
    let slots = source.slots();
    let mut nodes = 0;
    let mut edges = 0;
    let mut edge_count = 0;

    for start in (1..slots).step_by(BATCH as usize) {
        for (slot, element) in (start..).zip(source.graph(start, BATCH)?) {
            if element.0.is_some() {
                nodes = slot;
            }

            if element.1.is_some() {
                edges = slot;
                edge_count += 1;
            }
        }
    }

    for start in (1..=nodes.max(edges)).step_by(BATCH as usize) {
        let elements = source.graph(start, BATCH)?;
        batch(root, &mut |transaction| {
            let graph = &mut transaction.meta.graph;
            let alloc = &mut transaction.meta.alloc;

            for (slot, element) in (start..).zip(&elements) {
                if slot <= nodes {
                    graph.push_node(&mut transaction.view, alloc, element.0.as_ref())?;
                }

                if slot <= edges {
                    graph.push_edge(&mut transaction.view, alloc, element.1.as_ref())?;
                }
            }

            Ok(())
        })?;
    }

    relink(root, nodes, edge_count)?;
    let indexes = source.indexes()?;
    batch(root, &mut |transaction| {
        let meta = &mut transaction.meta;

        for key in &indexes {
            if meta.indexes.index(key).is_some() {
                fixed.push(format!("duplicate index '{key}' removed"));
            } else {
                meta.indexes.insert_index(
                    &mut transaction.view,
                    &mut meta.alloc,
                    &mut meta.keys,
                    key,
                )?;
            }
        }

        Ok(())
    })?;

    for aliases in source.aliases(fixed)?.chunks(BATCH as usize) {
        batch(root, &mut |transaction| {
            let meta = &mut transaction.meta;
            let view = &mut transaction.view;

            for (id, alias) in aliases {
                let value = DbValue::from(alias.as_str());
                KeyValues(&meta.graph).set(
                    view,
                    &mut meta.alloc,
                    &mut meta.keys,
                    *id,
                    ALIAS,
                    &value,
                )?;
                meta.indexes
                    .insert(view, &mut meta.alloc, ALIAS, &value, *id)?;
            }

            Ok(())
        })?;
    }

    for start in (0..source.value_slots()?).step_by(BATCH as usize) {
        let values = source.values(start, BATCH, fixed)?;
        batch(root, &mut |transaction| {
            let meta = &mut transaction.meta;
            let view = &mut transaction.view;

            for element in values.chunk_by(|a, b| a.0 == b.0) {
                KeyValues(&meta.graph).reserve(
                    view,
                    &mut meta.alloc,
                    element[0].0,
                    element.len() as u64,
                )?;
            }

            for (id, key_value) in &values {
                let (key, old) = KeyValues(&meta.graph).insert(
                    view,
                    &mut meta.alloc,
                    &mut meta.keys,
                    *id,
                    key_value,
                )?;

                if let Some(old) = old {
                    fixed.push(format!(
                        "value '{old}' of duplicate key '{}' of element {} removed",
                        key_value.key, id.0
                    ));
                    meta.indexes.remove(view, &mut meta.alloc, key, &old, *id)?;
                }

                meta.indexes
                    .insert(view, &mut meta.alloc, key, &key_value.value, *id)?;
            }

            Ok(())
        })?;
    }

    Ok(())
}

/// Links the edges of the nodes up to the slot `nodes` back and counts
/// them, failing unless each of the `edges` is in the lists of both of its
/// nodes.
fn relink<S: StorageData>(root: &mut Root<S>, nodes: u64, edges: u64) -> Result<(), DbError> {
    let mut linked = [0, 0];

    for start in (1..=nodes).step_by(BATCH as usize) {
        batch(root, &mut |transaction| {
            for slot in start..(start + BATCH).min(nodes + 1) {
                let id = DbId(slot as i64);
                let graph = &transaction.meta.graph;

                if graph.node(&transaction.view, id)?.is_some() {
                    let counts = graph.relink(&mut transaction.view, id)?;
                    linked[0] += counts[0];
                    linked[1] += counts[1];
                }
            }

            Ok(())
        })?;
    }

    if linked != [edges, edges] {
        return Err(DbError::graph(
            DbErrorType::InvalidIndex,
            format!(
                "Graph has edges not linked by their nodes ({} from, {} to of {edges})",
                linked[0], linked[1]
            ),
        ));
    }

    Ok(())
}

/// Migrates the legacy `storage` into a new one named after it, see
/// [`DbMigration`].
pub(crate) fn migrate<S: StorageData>(storage: S) -> Result<(Root<S>, DbMigration), DbError> {
    let mut fixed = vec![];
    let from;
    let mut root;

    {
        let reader = Reader::new(&storage);
        let legacy = Legacy::open(&reader)?;
        from = legacy.version();
        root = Root::open(target(&storage)?, Layout::default())?;
        rebuild(&legacy, &mut root, &mut fixed)?;
        root.storage_mut().sync()?;
    }

    swap(storage, &mut root)?;
    Ok((
        root,
        DbMigration {
            from,
            to: VERSION,
            fixed,
        },
    ))
}

/// Rebuilds the `root` into a new storage replacing its own once it is
/// durable. If only the replacement fails, the `root` is left on the new
/// storage with the old one unchanged.
pub(crate) fn optimize<S: StorageData>(root: &mut Root<S>) -> Result<(), DbError> {
    let mut rebuilt = {
        let (reader, meta) = root.read()?;
        let mut rebuilt = Root::open(target(root.storage())?, meta.layout)?;
        let transaction = Transaction::new(ReadView::Committed(reader), meta);
        rebuild(&transaction, &mut rebuilt, &mut vec![])?;
        rebuilt
    };

    rebuilt.storage_mut().sync()?;
    swap(std::mem::replace(root, rebuilt).into_storage(), root)
}

/// The empty storage `{name}.rebuild` of the same kind and configuration
/// as the `storage`.
fn target<S: StorageData>(storage: &S) -> Result<S, DbError> {
    let mut target = storage.create(&format!("{}.rebuild", storage.name()))?;
    target.set_sync_mode(storage.sync_mode());
    target.set_overlay_limit(storage.overlay_limit());
    Ok(target)
}

/// Replaces the `source` by the durable rebuilt storage of the `root`.
/// The source is closed first as only a closed file can be replaced
/// everywhere; on failure it is left as it was.
fn swap<S: StorageData>(source: S, root: &mut Root<S>) -> Result<(), DbError> {
    let name = source.name().to_string();
    drop(source);
    root.storage_mut().rename(&name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbKeyValue;
    use crate::QueryBuilder;
    use crate::QueryResult;
    use crate::Transaction;
    use crate::TransactionMut;
    use crate::db::legacy::tests::File;
    use crate::db::legacy::tests::GRAPH;
    use crate::db::legacy::tests::database;
    use crate::db::legacy::tests::database_v0;
    use crate::query::Query;
    use crate::query::QueryMut;
    use crate::storage::SyncMode;
    use crate::storage::file_storage::FileStorageImpl;
    use crate::storage::memory_storage::MemoryStorageImpl;
    use crate::storage::pages::View;
    use crate::transaction::ReadView;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::cell::Cell;
    use std::io::ErrorKind;
    use std::sync::Arc;

    type TestFile = FileStorageImpl<TestPlatform>;

    const NAME: &str = "/db";
    const REBUILD: &str = "/db.rebuild";
    const FREE: i64 = i64::MIN;

    /// The `legacy` source with other `indexes` and `values` whose call
    /// `fail` fails.
    struct Fake<'a, V> {
        legacy: Legacy<'a, V>,
        indexes: Vec<DbValue>,
        values: Vec<(DbId, DbKeyValue)>,
        fail: u64,
        calls: Cell<u64>,
    }

    impl<'a, V> Fake<'a, V> {
        fn new(
            legacy: Legacy<'a, V>,
            indexes: Vec<DbValue>,
            values: Vec<(DbId, DbKeyValue)>,
            fail: u64,
        ) -> Self {
            Self {
                legacy,
                indexes,
                values,
                fail,
                calls: Cell::new(0),
            }
        }

        fn call(&self) -> Result<(), DbError> {
            self.calls.set(self.calls.get() + 1);

            if self.calls.get() == self.fail {
                return Err(DbError::storage(
                    DbErrorType::NotFound,
                    format!("Call {} failed", self.fail),
                ));
            }

            Ok(())
        }
    }

    impl<V: View> Source for Fake<'_, V> {
        fn slots(&self) -> u64 {
            self.legacy.slots()
        }

        fn graph(&self, start: u64, count: u64) -> Result<Vec<Element>, DbError> {
            self.call()?;
            self.legacy.graph(start, count)
        }

        fn indexes(&self) -> Result<Vec<DbValue>, DbError> {
            self.call()?;
            Ok(self.indexes.clone())
        }

        fn aliases(&self, fixed: &mut Vec<String>) -> Result<Vec<(DbId, String)>, DbError> {
            self.call()?;
            self.legacy.aliases(fixed)
        }

        fn value_slots(&self) -> Result<u64, DbError> {
            self.call()?;
            Ok(1)
        }

        fn values(
            &self,
            _start: u64,
            _count: u64,
            _fixed: &mut Vec<String>,
        ) -> Result<Vec<(DbId, DbKeyValue)>, DbError> {
            self.call()?;
            Ok(self.values.clone())
        }
    }

    /// The legacy test database read by the `reader` with a duplicate index
    /// and a duplicate key-value, whose call `fail` fails.
    fn duplicates<'a>(
        reader: &'a Reader<'a, MemoryStorageImpl<TestPlatform>>,
        fail: u64,
    ) -> Fake<'a, Reader<'a, MemoryStorageImpl<TestPlatform>>> {
        let legacy = Legacy::open(reader).unwrap();
        let mut indexes = legacy.indexes().unwrap();
        indexes.push("k".into());
        let mut values = database().1;
        values.push((DbId(1), ("k", 2_u64).into()));
        Fake::new(legacy, indexes, values, fail)
    }

    fn memory(file: &File) -> MemoryStorageImpl<TestPlatform> {
        file.storage()
    }

    fn empty_root(platform: &Arc<TestPlatform>) -> Root<TestFile> {
        let mut storage = TestFile::with_platform(platform.clone(), "/target").unwrap();
        storage.set_overlay_limit(0);
        Root::open(storage, Layout::default()).unwrap()
    }

    /// A legacy file of the version 1 with the graph `slots` only.
    fn graph_only(slots: &[[i64; 4]]) -> File {
        let mut file = File::new(1);
        let graph = file.graph(slots);
        let aliases = file.map(&[]);
        let empty = file.vec(0, &[]);
        file.put(
            1,
            &crate::db::legacy::tests::words(&[1, graph, aliases, 0, empty, empty]),
        );
        file
    }

    fn exec<S: StorageData>(root: &Root<S>, query: impl Query) -> QueryResult {
        let (reader, meta) = root.read().unwrap();
        Transaction::new(ReadView::Committed(reader), meta)
            .exec(query)
            .unwrap()
    }

    fn exec_mut<S: StorageData>(root: &mut Root<S>, query: impl QueryMut) -> QueryResult {
        let mut transaction = TransactionMut::new(root.begin().unwrap());
        let result = transaction.exec_mut(query).unwrap();
        transaction.commit().unwrap();
        result
    }

    fn ids(result: QueryResult) -> Vec<i64> {
        result.elements.iter().map(|element| element.id.0).collect()
    }

    /// Writes the bytes of the legacy `file` as `NAME` of the `platform`.
    fn legacy_file(platform: &Arc<TestPlatform>, file: &File) -> TestFile {
        platform.fs().write(NAME, &file.bytes).unwrap();
        TestFile::with_platform(platform.clone(), NAME).unwrap()
    }

    mod rebuild {
        use super::*;

        #[test]
        fn rebuild_should_copy_elements_with_their_ids() {
            let platform = Arc::new(test_platform());
            let (file, values) = database();
            let storage = memory(&file);
            let reader = Reader::new(&storage);
            let legacy = Legacy::open(&reader).unwrap();
            let mut root = empty_root(&platform);
            let mut fixed = vec![];

            rebuild(&legacy, &mut root, &mut fixed).unwrap();

            assert_eq!(fixed.len(), 11);
            assert_eq!(
                ids(exec(&root, QueryBuilder::search().elements().query())),
                vec![1, 2, -3, -5]
            );
            assert_eq!(
                ids(exec(&root, QueryBuilder::search().from(1).query())),
                vec![1, -3, 2, -5]
            );
            assert_eq!(
                exec(
                    &root,
                    QueryBuilder::select().edge_count().ids([1, 2]).query()
                )
                .elements
                .iter()
                .map(|element| element.values[0].value.to_u64().unwrap())
                .collect::<Vec<_>>(),
                vec![1, 3]
            );
            assert_eq!(
                exec(&root, QueryBuilder::select().aliases().query())
                    .elements
                    .iter()
                    .map(|element| (element.id.0, element.values[0].value.to_string()))
                    .collect::<Vec<_>>(),
                vec![(1, "a".to_string()), (2, "c".to_string())]
            );
            assert_eq!(
                exec(&root, QueryBuilder::select().ids([1, 2, -3]).query())
                    .elements
                    .into_iter()
                    .flat_map(|element| element
                        .values
                        .into_iter()
                        .map(move |key_value| (element.id, key_value)))
                    .collect::<Vec<_>>(),
                values
            );
            assert_eq!(
                ids(exec(
                    &root,
                    QueryBuilder::search().index("k").value(1_u64).query()
                )),
                vec![1]
            );
        }

        #[test]
        fn rebuild_should_drop_free_slots_past_highest_elements() {
            let platform = Arc::new(test_platform());
            let storage = memory(&database().0);
            let reader = Reader::new(&storage);
            let legacy = Legacy::open(&reader).unwrap();
            let mut root = empty_root(&platform);
            rebuild(&legacy, &mut root, &mut vec![]).unwrap();

            let nodes = exec_mut(&mut root, QueryBuilder::insert().nodes().count(2).query());
            let edges = exec_mut(
                &mut root,
                QueryBuilder::insert()
                    .edges()
                    .from([1, 1, 1, 1])
                    .to([2, 2, 2, 2])
                    .query(),
            );

            assert_eq!(ids(nodes), vec![3, 4]);
            assert_eq!(ids(edges), vec![-4, -2, -1, -6]);
        }

        #[test]
        fn rebuild_duplicates_should_be_fixed() {
            let platform = Arc::new(test_platform());
            let storage = memory(&graph_only(&GRAPH));
            let reader = Reader::new(&storage);
            let source = Fake::new(
                Legacy::open(&reader).unwrap(),
                vec!["k".into(), "k".into()],
                vec![(DbId(1), ("k", 1).into()), (DbId(1), ("k", 2).into())],
                0,
            );
            let mut root = empty_root(&platform);
            let mut fixed = vec![];

            rebuild(&source, &mut root, &mut fixed).unwrap();

            assert_eq!(
                fixed,
                vec![
                    "duplicate index 'k' removed",
                    "value '1' of duplicate key 'k' of element 1 removed",
                ]
            );
            assert_eq!(
                ids(exec(
                    &root,
                    QueryBuilder::search().index("k").value(2).query()
                )),
                vec![1]
            );
            assert_eq!(
                ids(exec(
                    &root,
                    QueryBuilder::search().index("k").value(1).query()
                )),
                Vec::<i64>::new()
            );
        }

        #[test]
        fn rebuild_failed_source_should_fail() {
            let storage = memory(&database().0);
            let reader = Reader::new(&storage);
            let mut fail = 1;

            loop {
                let platform = Arc::new(test_platform());
                let source = duplicates(&reader, fail);
                let mut root = empty_root(&platform);

                match rebuild(&source, &mut root, &mut vec![]) {
                    Ok(()) => break,
                    Err(error) => assert_eq!(error.description, format!("Call {fail} failed")),
                }

                fail += 1;
            }

            assert_eq!(fail, 7);
        }

        #[test]
        fn rebuild_into_unrecoverable_root_should_fail() {
            let platform = Arc::new(test_platform());
            let storage = memory(&database().0);
            let reader = Reader::new(&storage);
            let legacy = Legacy::open(&reader).unwrap();
            let mut root = empty_root(&platform);
            let mut transaction = TransactionMut::new(root.begin().unwrap());
            transaction
                .exec_mut(QueryBuilder::insert().nodes().count(1).query())
                .unwrap();
            platform
                .fs()
                .set_io_error(FsOp::Read, "/.target", ErrorKind::Other);
            drop(transaction);

            assert!(rebuild(&legacy, &mut root, &mut vec![]).is_err());
        }

        #[test]
        fn rebuild_should_keep_free_slots_below_highest_elements() {
            let platform = Arc::new(test_platform());
            let storage = memory(&graph_only(&[
                [0, 0, -2, 2],
                [0, 0, 0, 0],
                [0, 0, FREE, 0],
                [0, 0, 0, 0],
            ]));
            let reader = Reader::new(&storage);
            let legacy = Legacy::open(&reader).unwrap();
            let mut root = empty_root(&platform);
            rebuild(&legacy, &mut root, &mut vec![]).unwrap();

            let nodes = exec_mut(&mut root, QueryBuilder::insert().nodes().count(2).query());

            assert_eq!(ids(nodes), vec![2, 4]);
        }

        #[test]
        fn rebuild_failed_target_io_should_fail() {
            for op in [FsOp::Read, FsOp::Write] {
                let mut fail = 0;

                loop {
                    let platform = Arc::new(test_platform());
                    let storage = memory(&database().0);
                    let reader = Reader::new(&storage);
                    let source = duplicates(&reader, 0);
                    let mut root = empty_root(&platform);
                    platform.fs().set_io_errors(
                        op,
                        "/",
                        (0..=fail)
                            .map(|call| (call == fail).then_some(ErrorKind::Other))
                            .chain([None]),
                    );

                    if rebuild(&source, &mut root, &mut vec![]).is_ok() {
                        break;
                    }

                    fail += 1;
                    assert!(fail < 10_000, "op: {op:?}");
                }

                assert!(10 < fail, "op: {op:?}, fail: {fail}");
            }
        }
    }

    mod relink {
        use super::*;

        #[test]
        fn relink_invalid_graph_should_fail() {
            for (slots, description) in [
                (
                    vec![[0, 0, FREE, 2], [0, 0, 0, 0], [0, 0, 0, 0], [-1, -2, 0, 0]],
                    "Graph has edges not linked by their nodes (0 from, 0 to of 1)",
                ),
                (
                    vec![[0, 0, FREE, 2], [3, 0, 1, 0], [0, 3, 0, 1], [-2, -2, 0, 0]],
                    "Node '1' links invalid edge '-3'",
                ),
            ] {
                let platform = Arc::new(test_platform());
                let storage = memory(&graph_only(&slots));
                let reader = Reader::new(&storage);
                let legacy = Legacy::open(&reader).unwrap();
                let mut root = empty_root(&platform);

                assert_eq!(
                    rebuild(&legacy, &mut root, &mut vec![])
                        .unwrap_err()
                        .description,
                    description
                );
            }
        }
    }

    mod migrate {
        use super::*;

        #[test]
        fn migrate_should_replace_legacy_file_with_rebuilt_one() {
            for (file, from) in [(database().0, 1), (database_v0(1), 0), (database_v0(0), 0)] {
                let platform = Arc::new(test_platform());
                let storage = legacy_file(&platform, &file);

                let (root, migration) = migrate(storage).unwrap();

                assert_eq!(root.storage().name(), NAME);
                assert_eq!((migration.from, migration.to), (from, VERSION));
                assert!(!platform.fs().exists(REBUILD).unwrap());
                assert!(!platform.fs().exists("/.db.rebuild").unwrap());
                drop(root);
                let root = Root::open(
                    TestFile::with_platform(platform.clone(), NAME).unwrap(),
                    Layout::default(),
                )
                .unwrap();
                assert_eq!(
                    ids(exec(&root, QueryBuilder::search().from(1).query()))[..2],
                    [1, -3]
                );
            }
        }

        #[test]
        fn migrate_invalid_should_fail_before_rebuilding() {
            let platform = Arc::new(test_platform());
            let mut file = File::new(1);
            file.put(1, &[0; 4]);
            let storage = legacy_file(&platform, &file);

            assert!(migrate(storage).is_err());
            assert!(!platform.fs().exists(REBUILD).unwrap());
        }

        #[test]
        fn migrate_unlinked_graph_should_keep_legacy_file() {
            let platform = Arc::new(test_platform());
            let file = graph_only(&[[0, 0, FREE, 2], [0, 0, 0, 0], [0, 0, 0, 0], [-1, -2, 0, 0]]);
            let storage = legacy_file(&platform, &file);

            assert!(migrate(storage).is_err());

            assert_eq!(platform.fs().read(NAME).unwrap(), file.bytes);
        }

        #[test]
        fn migrate_failed_io_should_keep_legacy_file() {
            for (op, path) in [
                (FsOp::Open, REBUILD),
                (FsOp::Write, REBUILD),
                (FsOp::SyncData, REBUILD),
                (FsOp::Rename, REBUILD),
                (FsOp::Rename, "/.db.rebuild"),
            ] {
                let platform = Arc::new(test_platform());
                let file = database().0;
                let storage = legacy_file(&platform, &file);
                platform.fs().set_io_error(op, path, ErrorKind::Other);

                assert!(migrate(storage).is_err(), "op: {op:?}, path: {path}");

                assert_eq!(platform.fs().read(NAME).unwrap(), file.bytes, "op: {op:?}");
            }
        }
    }

    mod optimize {
        use super::*;

        /// A database at `NAME` with an index, aliases, values and an edge,
        /// whose removed node leaves free space.
        fn populated(platform: &Arc<TestPlatform>) -> Root<TestFile> {
            let storage = TestFile::with_platform(platform.clone(), NAME).unwrap();
            let mut root = Root::open(storage, Layout::default()).unwrap();
            exec_mut(&mut root, QueryBuilder::insert().index("k").query());
            exec_mut(
                &mut root,
                QueryBuilder::insert()
                    .nodes()
                    .aliases(["a", "b", "c"])
                    .values([
                        vec![("k", 1).into()],
                        vec![("k", 2).into()],
                        vec![("k", vec![3_u64; 64]).into()],
                    ])
                    .query(),
            );
            exec_mut(
                &mut root,
                QueryBuilder::insert().edges().from("a").to("b").query(),
            );
            exec_mut(&mut root, QueryBuilder::remove().ids("c").query());
            root
        }

        fn assert_populated(root: &Root<TestFile>) {
            assert_eq!(
                ids(exec(root, QueryBuilder::search().from("a").query())),
                vec![1, -1, 2]
            );
            assert_eq!(
                ids(exec(
                    root,
                    QueryBuilder::search().index("k").value(2).query()
                )),
                vec![2]
            );
            assert_eq!(
                exec(root, QueryBuilder::select().aliases().query()).result,
                2
            );
        }

        #[test]
        fn optimize_should_replace_storage_with_rebuilt_one() {
            let platform = Arc::new(test_platform());
            let mut root = populated(&platform);
            let len = root.storage().len();

            optimize(&mut root).unwrap();

            assert_eq!(root.storage().name(), NAME);
            assert!(root.storage().len() < len);
            assert!(!platform.fs().exists(REBUILD).unwrap());
            assert_populated(&root);
        }

        #[test]
        fn optimize_failed_io_should_keep_storage() {
            for (op, path) in [
                (FsOp::Read, NAME),
                (FsOp::Open, REBUILD),
                (FsOp::Write, REBUILD),
                (FsOp::SyncData, REBUILD),
            ] {
                let mut fail = 0;

                loop {
                    let platform = Arc::new(test_platform());
                    let mut root = populated(&platform);
                    platform.fs().set_io_errors(
                        op,
                        path,
                        (0..=fail)
                            .map(|call| (call == fail).then_some(ErrorKind::Other))
                            .chain([None]),
                    );

                    if optimize(&mut root).is_ok() {
                        break;
                    }

                    platform.fs().clear_faults();
                    assert_eq!(root.storage().name(), NAME, "op: {op:?}, fail: {fail}");
                    assert_populated(&root);
                    fail += 1;
                    assert!(fail < 10_000, "op: {op:?}");
                }
            }
        }

        #[test]
        fn optimize_failed_rename_should_keep_rebuilt_storage() {
            let platform = Arc::new(test_platform());
            let mut root = populated(&platform);
            let len = root.storage().len();
            platform
                .fs()
                .set_io_error(FsOp::Rename, REBUILD, ErrorKind::Other);

            assert!(optimize(&mut root).is_err());

            assert_eq!(root.storage().name(), REBUILD);
            assert_populated(&root);
            let old = TestFile::with_platform(platform.clone(), NAME).unwrap();
            assert_eq!(old.len(), len);
            assert_populated(&Root::open(old, Layout::default()).unwrap());
        }

        #[test]
        fn optimize_poisoned_root_should_fail() {
            let platform = Arc::new(test_platform());
            let mut root = populated(&platform);
            root.storage_mut().set_overlay_limit(0);
            let mut transaction = TransactionMut::new(root.begin().unwrap());
            transaction
                .exec_mut(QueryBuilder::insert().nodes().count(1).query())
                .unwrap();
            platform
                .fs()
                .set_io_error(FsOp::Read, "/.db", ErrorKind::Other);
            drop(transaction);

            assert_eq!(optimize(&mut root).unwrap_err().ty, DbErrorType::Poisoned);
        }
    }

    mod target {
        use super::*;

        #[test]
        fn target_should_create_empty_storage_with_config() {
            let platform = Arc::new(test_platform());
            let mut storage = legacy_file(&platform, &database().0);
            storage.set_sync_mode(SyncMode::Commit);
            storage.set_overlay_limit(7);

            let target = target(&storage).unwrap();

            assert_eq!(target.name(), REBUILD);
            assert_eq!(target.len(), 0);
            assert_eq!(target.sync_mode(), SyncMode::Commit);
            assert_eq!(target.overlay_limit(), 7);
        }
    }
}
