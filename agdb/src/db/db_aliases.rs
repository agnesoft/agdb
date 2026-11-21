use crate::DbError;
use crate::DbId;
use crate::StorageData;
use crate::collections::map::MapData;
use crate::collections::map::MapValueState;
use crate::collections::multi_map::MultiMapStorage;
use crate::db::db_value_index::DbValueIndex;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;

/// Type of the inline value marking that a node has an alias.
const ALIAS_TYPE: u8 = 1;

/// The aliases of the nodes. A node keeps its alias in the graph,
/// in place if it fits into 15 bytes and as a separate value
/// otherwise. The hashes of the aliases map to the nodes carrying
/// them to find a node by its alias.
pub(crate) struct DbAliases<D>
where
    D: StorageData,
{
    ids: MultiMapStorage<u64, DbId, D>,
}

impl<D> DbAliases<D>
where
    D: StorageData,
{
    pub(crate) fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        Ok(Self {
            ids: MultiMapStorage::new(storage)?,
        })
    }

    pub(crate) fn from_storage(storage: &Storage<D>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            ids: MultiMapStorage::from_storage(storage, index)?,
        })
    }

    pub(crate) fn storage_index(&self) -> StorageIndex {
        self.ids.storage_index()
    }

    /// The alias of the node `id`, `None` for edges and nodes without one.
    pub(crate) fn alias(
        &self,
        storage: &Storage<D>,
        graph: &DbGraph<D>,
        id: DbId,
    ) -> Result<Option<String>, DbError> {
        Ok(self.find(storage, graph, id)?.map(|(_, alias)| alias))
    }

    /// All aliases with their nodes in the slot order of the map.
    pub(crate) fn aliases(
        &self,
        storage: &Storage<D>,
        graph: &DbGraph<D>,
    ) -> Result<Vec<(String, DbId)>, DbError> {
        let mut aliases = Vec::with_capacity(self.ids.len() as usize);

        for pos in 0..self.ids.data.capacity() {
            if self.ids.data.state(storage, pos)? == MapValueState::Valid {
                let id = self.ids.data.value(storage, pos)?;

                if let Some(index) = Self::stored(storage, graph, id)? {
                    aliases.push((Self::decode(storage, &index)?, id));
                }
            }
        }

        Ok(aliases)
    }

    /// The node with the `alias`.
    pub(crate) fn id(
        &self,
        storage: &Storage<D>,
        graph: &DbGraph<D>,
        alias: &str,
    ) -> Result<Option<DbId>, DbError> {
        let hash = Self::hash(alias);
        let inline = Self::inline(alias);

        for entry in self.ids.iter_key(storage, &hash) {
            let id = entry?.1;
            let Some(index) = Self::stored(storage, graph, id)? else {
                continue;
            };

            let found = match inline {
                Some(inline) => index == inline,
                None => !index.is_value() && Self::decode(storage, &index)? == alias,
            };

            if found {
                return Ok(Some(id));
            }
        }

        Ok(None)
    }

    /// Sets the `alias` of the node `id` replacing its current alias.
    /// The alias must not be used by another node.
    pub(crate) fn insert(
        &mut self,
        storage: &mut Storage<D>,
        graph: &mut DbGraph<D>,
        id: DbId,
        alias: &str,
    ) -> Result<(), DbError> {
        let transaction = storage.transaction();
        self.remove(storage, graph, id)?;

        let index = match Self::inline(alias) {
            Some(index) => index,
            None => {
                let mut index = DbValueIndex::new();
                index.set_index(storage.insert(&alias.to_string())?.0);
                index.set_type(ALIAS_TYPE);
                index
            }
        };

        graph.set_alias(storage, GraphIndex(id.0), &index.data())?;
        self.ids.insert(storage, &Self::hash(alias), &id)?;
        storage.commit(transaction)
    }

    /// Removes the alias of the node `id` returning it.
    pub(crate) fn remove(
        &mut self,
        storage: &mut Storage<D>,
        graph: &mut DbGraph<D>,
        id: DbId,
    ) -> Result<Option<String>, DbError> {
        let Some((index, alias)) = self.find(storage, graph, id)? else {
            return Ok(None);
        };

        let transaction = storage.transaction();

        if !index.is_value() {
            storage.remove(StorageIndex(index.index()))?;
        }

        self.ids.remove_value(storage, &Self::hash(&alias), &id)?;
        graph.set_alias(storage, GraphIndex(id.0), &[0; 16])?;
        storage.commit(transaction)?;

        Ok(Some(alias))
    }

    pub(crate) fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.ids.shrink_to_fit(storage)
    }

    fn decode(storage: &Storage<D>, index: &DbValueIndex) -> Result<String, DbError> {
        if index.is_value() {
            Ok(String::from_utf8_lossy(index.value()).into_owned())
        } else {
            storage.value(StorageIndex(index.index()))
        }
    }

    fn find(
        &self,
        storage: &Storage<D>,
        graph: &DbGraph<D>,
        id: DbId,
    ) -> Result<Option<(DbValueIndex, String)>, DbError> {
        match Self::stored(storage, graph, id)? {
            Some(index) if index.get_type() == ALIAS_TYPE => {
                Ok(Some((index, Self::decode(storage, &index)?)))
            }
            _ => Ok(None),
        }
    }

    fn hash(alias: &str) -> u64 {
        alias.as_bytes().stable_hash()
    }

    fn inline(alias: &str) -> Option<DbValueIndex> {
        let mut index = DbValueIndex::new();

        if index.set_value(alias.as_bytes()) {
            index.set_type(ALIAS_TYPE);
            Some(index)
        } else {
            None
        }
    }

    fn stored(
        storage: &Storage<D>,
        graph: &DbGraph<D>,
        id: DbId,
    ) -> Result<Option<DbValueIndex>, DbError> {
        graph
            .alias(storage, GraphIndex(id.0))?
            .map(|alias| DbValueIndex::deserialize(&alias))
            .transpose()
    }
}

/// The aliases of the databases of version 1: a map of the aliases to
/// the ids and a map of the ids to the aliases, both with the aliases
/// as separate values.
pub(crate) mod v1 {
    use super::*;

    /// Moves the aliases of the nodes at `index` into the `graph` and
    /// removes the maps. The aliases of the edges and of the elements
    /// that no longer exist are dropped and described in `fixed`.
    pub(crate) fn convert<D: StorageData>(
        storage: &mut Storage<D>,
        graph: &mut DbGraph<D>,
        index: (StorageIndex, StorageIndex),
        fixed: &mut Vec<String>,
    ) -> Result<DbAliases<D>, DbError> {
        let transaction = storage.transaction();
        let mut aliases = DbAliases::new(storage)?;
        let keys = MultiMapStorage::<String, DbId, D>::from_storage(storage, index.0)?;

        for pos in 0..keys.data.capacity() {
            if keys.data.state(storage, pos)? != MapValueState::Valid {
                continue;
            }

            let alias = keys.data.key(storage, pos)?;
            let id = keys.data.value(storage, pos)?;

            if graph.node(storage, GraphIndex(id.0))?.is_none() {
                fixed.push(if graph.edge(storage, GraphIndex(id.0))?.is_some() {
                    format!(
                        "alias '{alias}' of edge {} removed: edges cannot have aliases",
                        id.0
                    )
                } else {
                    format!("alias '{alias}' of missing element {} removed", id.0)
                });
            } else if alias.is_empty() {
                fixed.push(format!(
                    "empty alias of node {} removed: aliases cannot be empty",
                    id.0
                ));
            } else if let Some(other) = aliases.alias(storage, graph, id)? {
                fixed.push(format!(
                    "alias '{alias}' of node {} removed: the node already has alias '{other}'",
                    id.0
                ));
            } else {
                aliases.insert(storage, graph, id, &alias)?;
            }
        }

        keys.remove_from_storage(storage)?;
        MultiMapStorage::<DbId, String, D>::from_storage(storage, index.1)?
            .remove_from_storage(storage)?;
        storage.commit(transaction)?;

        Ok(aliases)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::collections::tests::TestStorage;
        use crate::collections::tests::assert_io_failures_recover;
        use crate::collections::tests::assert_read_failures;
        use crate::collections::tests::storage;
        use crate::db::db_aliases::tests::graph;

        fn maps(
            storage: &mut TestStorage,
            aliases: &[(&str, i64)],
        ) -> (StorageIndex, StorageIndex) {
            let mut keys = MultiMapStorage::<String, DbId, _>::new(storage).unwrap();
            let mut values = MultiMapStorage::<DbId, String, _>::new(storage).unwrap();

            for (alias, id) in aliases {
                keys.insert(storage, &alias.to_string(), &DbId(*id))
                    .unwrap();
                values
                    .insert(storage, &DbId(*id), &alias.to_string())
                    .unwrap();
            }

            (keys.storage_index(), values.storage_index())
        }

        mod convert {
            use super::*;

            #[test]
            fn convert_should_move_aliases_into_graph() {
                let mut storage = storage();
                let mut graph = graph(&mut storage, 2);
                let index = maps(
                    &mut storage,
                    &[("root", 1), ("a_much_longer_alias_than_15_bytes", 2)],
                );
                let mut fixed = vec![];

                let aliases = convert(&mut storage, &mut graph, index, &mut fixed).unwrap();

                let mut all = aliases.aliases(&storage, &graph).unwrap();
                all.sort();
                assert_eq!(
                    all,
                    vec![
                        ("a_much_longer_alias_than_15_bytes".to_string(), DbId(2)),
                        ("root".to_string(), DbId(1)),
                    ]
                );
                assert_eq!(aliases.id(&storage, &graph, "root").unwrap(), Some(DbId(1)));
                assert!(fixed.is_empty());
                assert!(storage.value_size(index.0).is_err());
                assert!(storage.value_size(index.1).is_err());
            }

            #[test]
            fn convert_should_drop_aliases_of_edges_and_missing_elements() {
                let mut storage = storage();
                let mut graph = graph(&mut storage, 3);
                let edge = graph
                    .insert_edge(&mut storage, GraphIndex(1), GraphIndex(2))
                    .unwrap();
                graph.remove_node(&mut storage, GraphIndex(3)).unwrap();
                let index = maps(
                    &mut storage,
                    &[
                        ("edge", edge.0),
                        ("removed", 3),
                        ("unknown", 10),
                        ("node", 1),
                    ],
                );
                let mut fixed = vec![];

                let aliases = convert(&mut storage, &mut graph, index, &mut fixed).unwrap();

                fixed.sort();
                assert_eq!(
                    fixed,
                    vec![
                        "alias 'edge' of edge -4 removed: edges cannot have aliases",
                        "alias 'removed' of missing element 3 removed",
                        "alias 'unknown' of missing element 10 removed",
                    ]
                );
                assert_eq!(
                    aliases.aliases(&storage, &graph).unwrap(),
                    vec![("node".to_string(), DbId(1))]
                );
            }

            #[test]
            fn convert_should_drop_empty_aliases() {
                let mut storage = storage();
                let mut graph = graph(&mut storage, 2);
                let index = maps(&mut storage, &[("", 1), ("node", 2)]);
                let mut fixed = vec![];

                let aliases = convert(&mut storage, &mut graph, index, &mut fixed).unwrap();

                assert_eq!(
                    fixed,
                    vec!["empty alias of node 1 removed: aliases cannot be empty"]
                );
                assert_eq!(aliases.alias(&storage, &graph, DbId(1)), Ok(None));
                assert_eq!(
                    aliases.aliases(&storage, &graph).unwrap(),
                    vec![("node".to_string(), DbId(2))]
                );
            }

            #[test]
            fn convert_should_keep_one_alias_per_node() {
                let mut storage = storage();
                let mut graph = graph(&mut storage, 1);
                let index = maps(&mut storage, &[("first", 1), ("second", 1)]);
                let mut fixed = vec![];

                let aliases = convert(&mut storage, &mut graph, index, &mut fixed).unwrap();

                let kept = aliases.alias(&storage, &graph, DbId(1)).unwrap().unwrap();
                let dropped = ["first", "second"]
                    .into_iter()
                    .find(|alias| *alias != kept)
                    .unwrap();
                assert_eq!(
                    fixed,
                    vec![format!(
                        "alias '{dropped}' of node 1 removed: the node already has alias '{kept}'"
                    )]
                );
                assert_eq!(aliases.id(&storage, &graph, dropped).unwrap(), None);
            }

            #[test]
            fn convert_read_failure_should_return_error() {
                assert_read_failures(
                    |storage| {
                        let mut graph = graph(storage, 2);
                        let edge = graph
                            .insert_edge(storage, GraphIndex(1), GraphIndex(2))
                            .unwrap();
                        let index = maps(storage, &[("edge", edge.0), ("unknown", 10)]);
                        (graph, index)
                    },
                    |storage, subject| {
                        let mut fixed = vec![];
                        convert(storage, &mut subject.0, subject.1, &mut fixed)?;
                        fixed.sort();
                        Ok(fixed)
                    },
                );
            }

            #[test]
            fn convert_should_keep_version_1_maps_on_io_failure() {
                assert_io_failures_recover(
                    |storage| {
                        let graph = graph(storage, 2);
                        let index = maps(
                            storage,
                            &[("a", 1), ("a_much_longer_alias_than_15_bytes", 2)],
                        );
                        (graph, index)
                    },
                    |storage, (graph, index)| convert(storage, graph, *index, &mut vec![]),
                    |storage, (graph, index)| {
                        let mut graph =
                            DbGraph::from_storage(storage, graph.storage_index()).unwrap();
                        assert_eq!(graph.alias(storage, GraphIndex(1)).unwrap(), Some([0; 16]));
                        let aliases = convert(storage, &mut graph, *index, &mut vec![]).unwrap();
                        assert_eq!(aliases.id(storage, &graph, "a").unwrap(), Some(DbId(1)));
                        assert_eq!(
                            aliases
                                .id(storage, &graph, "a_much_longer_alias_than_15_bytes")
                                .unwrap(),
                            Some(DbId(2))
                        );
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::tests::TestStorage;
    use crate::collections::tests::assert_io_failures_recover;
    use crate::collections::tests::storage;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    type TestGraph = DbGraph<FileStorageImpl<TestPlatform>>;
    type TestAliases = DbAliases<FileStorageImpl<TestPlatform>>;

    const LONG: &str = "sixteen_bytes_ab";
    const INLINE: &str = "fifteen_bytes_a";

    pub(super) fn graph(storage: &mut TestStorage, nodes: usize) -> TestGraph {
        let mut graph = DbGraph::new(storage).unwrap();

        for _ in 0..nodes {
            graph.insert_node(storage).unwrap();
        }

        graph
    }

    fn aliased(storage: &mut TestStorage, aliases: &[&str]) -> (TestGraph, TestAliases) {
        let mut graph = graph(storage, aliases.len());
        let mut db_aliases = DbAliases::new(storage).unwrap();

        for (i, alias) in aliases.iter().enumerate() {
            db_aliases
                .insert(storage, &mut graph, DbId(i as i64 + 1), alias)
                .unwrap();
        }

        (graph, db_aliases)
    }

    fn sorted(
        storage: &TestStorage,
        graph: &TestGraph,
        aliases: &TestAliases,
    ) -> Vec<(String, DbId)> {
        let mut all = aliases.aliases(storage, graph).unwrap();
        all.sort();
        all
    }

    fn stored(storage: &TestStorage, graph: &TestGraph, id: i64) -> DbValueIndex {
        DbValueIndex::deserialize(&graph.alias(storage, GraphIndex(id)).unwrap().unwrap()).unwrap()
    }

    mod db_aliases {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_have_no_aliases() {
                let mut storage = storage();
                let graph = graph(&mut storage, 1);

                let aliases = DbAliases::new(&mut storage).unwrap();

                assert!(aliases.aliases(&storage, &graph).unwrap().is_empty());
                assert_eq!(aliases.id(&storage, &graph, "").unwrap(), None);
            }
        }

        mod from_storage {
            use super::*;

            #[test]
            fn from_storage_should_restore_aliases() {
                let mut storage = storage();
                let (graph, aliases) = aliased(&mut storage, &["a", LONG]);

                let restored = DbAliases::from_storage(&storage, aliases.storage_index()).unwrap();

                assert_eq!(
                    sorted(&storage, &graph, &restored),
                    sorted(&storage, &graph, &aliases)
                );
                assert_eq!(restored.id(&storage, &graph, LONG).unwrap(), Some(DbId(2)));
            }

            #[test]
            fn from_storage_missing_value_should_fail() {
                let storage = storage();

                assert!(DbAliases::from_storage(&storage, StorageIndex(10)).is_err());
            }
        }

        mod alias {
            use super::*;

            #[test]
            fn alias_should_return_alias_of_node() {
                let mut storage = storage();
                let aliases = ["a", INLINE, LONG, "žluťoučký kůň"];

                let (graph, db_aliases) = aliased(&mut storage, &aliases);

                for (i, alias) in aliases.iter().enumerate() {
                    assert_eq!(
                        db_aliases
                            .alias(&storage, &graph, DbId(i as i64 + 1))
                            .unwrap()
                            .as_deref(),
                        Some(*alias)
                    );
                }
                assert!(stored(&storage, &graph, 2).is_value());
                assert!(!stored(&storage, &graph, 3).is_value());
            }

            #[test]
            fn alias_no_alias_should_return_none() {
                let mut storage = storage();
                let (mut graph, aliases) = aliased(&mut storage, &["a"]);
                graph.insert_node(&mut storage).unwrap();
                let edge = graph
                    .insert_edge(&mut storage, GraphIndex(1), GraphIndex(2))
                    .unwrap();

                for id in [2, edge.0, 0, 10] {
                    assert_eq!(
                        aliases.alias(&storage, &graph, DbId(id)).unwrap(),
                        None,
                        "{id}"
                    );
                }
            }

            #[test]
            fn alias_should_return_read_failure() {
                let mut storage = storage();
                let (graph, aliases) = aliased(&mut storage, &[LONG]);
                storage
                    .remove(StorageIndex(stored(&storage, &graph, 1).index()))
                    .unwrap();

                assert!(aliases.alias(&storage, &graph, DbId(1)).is_err());
            }
        }

        mod aliases {
            use super::*;

            #[test]
            fn aliases_should_return_all_aliases() {
                let mut storage = storage();
                let (mut graph, mut aliases) = aliased(&mut storage, &["b", LONG, "a", "c"]);
                aliases.remove(&mut storage, &mut graph, DbId(4)).unwrap();

                assert_eq!(
                    sorted(&storage, &graph, &aliases),
                    vec![
                        ("a".to_string(), DbId(3)),
                        ("b".to_string(), DbId(1)),
                        (LONG.to_string(), DbId(2)),
                    ]
                );
            }

            #[test]
            fn aliases_should_return_read_failures() {
                crate::collections::tests::assert_io_failures(
                    |storage| aliased(storage, &["a", LONG]),
                    |storage, (graph, aliases)| aliases.aliases(storage, graph),
                );
            }

            #[test]
            fn aliases_should_skip_entries_of_missing_nodes() {
                let mut storage = storage();
                let (graph, mut aliases) = aliased(&mut storage, &["a"]);
                aliases
                    .ids
                    .insert(
                        &mut storage,
                        &DbAliases::<FileStorageImpl<TestPlatform>>::hash("b"),
                        &DbId(10),
                    )
                    .unwrap();

                assert_eq!(
                    aliases.aliases(&storage, &graph).unwrap(),
                    vec![("a".to_string(), DbId(1))]
                );
            }
        }

        mod id {
            use super::*;

            #[test]
            fn id_should_return_node_with_alias() {
                let mut storage = storage();
                let (graph, aliases) = aliased(&mut storage, &[INLINE, LONG]);

                for (alias, id) in [(INLINE, 1), (LONG, 2)] {
                    assert_eq!(aliases.id(&storage, &graph, alias).unwrap(), Some(DbId(id)));
                }
                assert_eq!(aliases.id(&storage, &graph, "x").unwrap(), None);
            }

            #[test]
            fn id_colliding_hashes_should_compare_aliases() {
                let mut storage = storage();
                let (graph, mut aliases) =
                    aliased(&mut storage, &["a", LONG, "b", "a_different_long_alias"]);
                type Aliases = DbAliases<FileStorageImpl<TestPlatform>>;

                for (alias, id) in [
                    ("b", 1),
                    ("b", 2),
                    ("a_different_long_alias", 1),
                    ("a_different_long_alias", 2),
                    ("b", 10),
                ] {
                    aliases
                        .ids
                        .insert(&mut storage, &Aliases::hash(alias), &DbId(id))
                        .unwrap();
                }

                assert_eq!(aliases.id(&storage, &graph, "b").unwrap(), Some(DbId(3)));
                assert_eq!(
                    aliases
                        .id(&storage, &graph, "a_different_long_alias")
                        .unwrap(),
                    Some(DbId(4))
                );
            }

            #[test]
            fn id_entries_of_other_aliases_should_return_none() {
                let mut storage = storage();
                let (graph, mut aliases) = aliased(&mut storage, &["a", LONG]);
                type Aliases = DbAliases<FileStorageImpl<TestPlatform>>;

                for alias in ["x", "a_different_long_alias"] {
                    for id in [1, 2, 10] {
                        aliases
                            .ids
                            .insert(&mut storage, &Aliases::hash(alias), &DbId(id))
                            .unwrap();
                    }
                }

                assert_eq!(aliases.id(&storage, &graph, "x").unwrap(), None);
                assert_eq!(
                    aliases
                        .id(&storage, &graph, "a_different_long_alias")
                        .unwrap(),
                    None
                );
            }

            #[test]
            fn id_should_return_read_failures() {
                crate::collections::tests::assert_io_failures(
                    |storage| aliased(storage, &["a", LONG]),
                    |storage, (graph, aliases)| aliases.id(storage, graph, LONG),
                );
            }

            #[test]
            fn id_should_return_read_failure() {
                let mut storage = storage();
                let (graph, aliases) = aliased(&mut storage, &[LONG]);
                storage
                    .remove(StorageIndex(stored(&storage, &graph, 1).index()))
                    .unwrap();

                assert!(aliases.id(&storage, &graph, LONG).is_err());
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_replace_alias_of_node() {
                let mut storage = storage();
                let (mut graph, mut aliases) = aliased(&mut storage, &[LONG]);
                let old = StorageIndex(stored(&storage, &graph, 1).index());

                aliases
                    .insert(&mut storage, &mut graph, DbId(1), "a")
                    .unwrap();

                assert_eq!(
                    sorted(&storage, &graph, &aliases),
                    vec![("a".to_string(), DbId(1))]
                );
                assert_eq!(aliases.id(&storage, &graph, LONG).unwrap(), None);
                assert!(storage.value_size(old).is_err());
            }

            #[test]
            fn insert_edge_should_fail() {
                let mut storage = storage();
                let (mut graph, mut aliases) = aliased(&mut storage, &["a", "b"]);
                let edge = graph
                    .insert_edge(&mut storage, GraphIndex(1), GraphIndex(2))
                    .unwrap();

                assert!(
                    aliases
                        .insert(&mut storage, &mut graph, DbId(edge.0), "c")
                        .is_err()
                );
                assert_eq!(aliases.id(&storage, &graph, "c").unwrap(), None);
            }

            #[test]
            fn insert_should_restore_aliases_on_io_failure() {
                for alias in ["c", "a_much_longer_alias_than_15_bytes"] {
                    assert_io_failures_recover(
                        |storage| aliased(storage, &[LONG, "b"]),
                        |storage, (graph, aliases)| aliases.insert(storage, graph, DbId(1), alias),
                        |storage, (graph, aliases)| {
                            let graph =
                                DbGraph::from_storage(storage, graph.storage_index()).unwrap();
                            let aliases =
                                DbAliases::from_storage(storage, aliases.storage_index()).unwrap();
                            assert_eq!(
                                sorted(storage, &graph, &aliases),
                                vec![("b".to_string(), DbId(2)), (LONG.to_string(), DbId(1))]
                            );
                        },
                    );
                }
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_should_return_and_remove_alias() {
                let mut storage = storage();
                let (mut graph, mut aliases) = aliased(&mut storage, &[LONG, "b"]);
                let value = StorageIndex(stored(&storage, &graph, 1).index());

                let removed = aliases.remove(&mut storage, &mut graph, DbId(1)).unwrap();

                assert_eq!(removed.as_deref(), Some(LONG));
                assert_eq!(graph.alias(&storage, GraphIndex(1)).unwrap(), Some([0; 16]));
                assert_eq!(aliases.id(&storage, &graph, LONG).unwrap(), None);
                assert_eq!(
                    sorted(&storage, &graph, &aliases),
                    vec![("b".to_string(), DbId(2))]
                );
                assert!(storage.value_size(value).is_err());
            }

            #[test]
            fn remove_no_alias_should_return_none() {
                let mut storage = storage();
                let (mut graph, mut aliases) = aliased(&mut storage, &["a"]);
                graph.insert_node(&mut storage).unwrap();

                for id in [2, 0, -1, 10] {
                    assert_eq!(
                        aliases.remove(&mut storage, &mut graph, DbId(id)).unwrap(),
                        None
                    );
                }
                assert_eq!(
                    sorted(&storage, &graph, &aliases),
                    vec![("a".to_string(), DbId(1))]
                );
            }

            #[test]
            fn remove_should_restore_aliases_on_io_failure() {
                for id in [1, 2] {
                    assert_io_failures_recover(
                        |storage| aliased(storage, &[LONG, "b"]),
                        |storage, (graph, aliases)| aliases.remove(storage, graph, DbId(id)),
                        |storage, (graph, aliases)| {
                            let graph =
                                DbGraph::from_storage(storage, graph.storage_index()).unwrap();
                            let aliases =
                                DbAliases::from_storage(storage, aliases.storage_index()).unwrap();
                            assert_eq!(
                                sorted(storage, &graph, &aliases),
                                vec![("b".to_string(), DbId(2)), (LONG.to_string(), DbId(1))]
                            );
                        },
                    );
                }
            }
        }

        mod shrink_to_fit {
            use super::*;

            #[test]
            fn shrink_to_fit_should_keep_aliases() {
                let mut storage = storage();
                let names: Vec<String> = (0..100).map(|i| format!("alias_{i}")).collect();
                let names: Vec<&str> = names.iter().map(String::as_str).collect();
                let (mut graph, mut aliases) = aliased(&mut storage, &names);

                for id in 1..=90 {
                    aliases.remove(&mut storage, &mut graph, DbId(id)).unwrap();
                }

                let size = storage.value_size(aliases.ids.data.storage_index()).ok();
                aliases.shrink_to_fit(&mut storage).unwrap();

                assert_eq!(sorted(&storage, &graph, &aliases).len(), 10);
                assert_eq!(
                    aliases.id(&storage, &graph, "alias_95").unwrap(),
                    Some(DbId(96))
                );
                assert!(aliases.ids.data.capacity() < 100, "{size:?}");
            }
        }
    }
}
