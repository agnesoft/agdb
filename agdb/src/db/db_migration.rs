use crate::DbError;
use crate::DbErrorType;
use crate::StorageData;
use crate::db::DB_VERSION;
use crate::db::DbStorageIndex;
use crate::db::db_aliases;
use crate::graph;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::serialize::u64_at;

/// The conversion of a database of an older version done when it was
/// opened, see [`DbImpl::migration()`](crate::DbImpl::migration).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DbMigration {
    /// The version of the database before the conversion, `0` for the
    /// databases created by agdb 0.10.0 and older.
    pub from: u64,

    /// The version of the database after the conversion.
    pub to: u64,

    /// The inconsistencies of the data the conversion found and fixed.
    pub fixed: Vec<String>,
}

/// Converts the database in the `storage` to the current version in
/// a single transaction that also sets the storage version, so that
/// the older versions of agdb refuse to open the database afterwards.
/// Only for storage versions below [`DB_VERSION`]: the legacy index
/// is told apart by its size, which the current index shares.
pub(crate) fn migrate<D: StorageData>(storage: &mut Storage<D>) -> Result<DbMigration, DbError> {
    let transaction = storage.transaction();
    let size = storage.value_size(StorageIndex(1))?;
    let from = if size == legacy::DbStorageIndex::serialized_size_static() {
        legacy::convert(storage)?;
        legacy::VERSION
    } else {
        v1::VERSION
    };

    let index = storage.value::<v1::DbStorageIndex>(StorageIndex(1))?;

    if index.version != v1::VERSION {
        return Err(DbError::db(
            DbErrorType::NotAllowed,
            format!(
                "Database version '{}' is not supported by storage version '{}'",
                index.version,
                storage.version()
            ),
        ));
    }

    let mut fixed = vec![];
    let mut graph = graph::v1::convert(storage, index.graph)?;
    let aliases = db_aliases::v1::convert(storage, &mut graph, index.aliases, &mut fixed)?;

    storage.replace(
        StorageIndex(1),
        &DbStorageIndex {
            version: DB_VERSION,
            graph: graph.storage_index(),
            aliases: aliases.storage_index(),
            indexes: index.indexes,
            values: index.values,
        },
    )?;
    storage.set_version(DB_VERSION)?;
    storage.commit(transaction)?;
    storage.optimize_storage()?;

    Ok(DbMigration {
        from,
        to: DB_VERSION,
        fixed,
    })
}

/// The databases of agdb 0.11.0 to 0.17.x.
pub(crate) mod v1 {
    use super::*;

    pub(crate) const VERSION: u64 = 1;

    pub(crate) struct DbStorageIndex {
        pub(crate) version: u64,
        pub(crate) graph: StorageIndex,
        pub(crate) aliases: (StorageIndex, StorageIndex),
        pub(crate) indexes: StorageIndex,
        pub(crate) values: StorageIndex,
    }

    impl Serialize for DbStorageIndex {
        fn serialize(&self) -> Vec<u8> {
            [
                self.version,
                self.graph.0,
                self.aliases.0.0,
                self.aliases.1.0,
                self.indexes.0,
                self.values.0,
            ]
            .iter()
            .flat_map(|field| field.to_le_bytes())
            .collect()
        }

        fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
            if bytes.len() < Self::serialized_size_static() as usize {
                return Err(DbError::db(
                    DbErrorType::NotEnoughData,
                    format!(
                        "DbStorageIndex v1 deserialization error: got {} bytes, expected at least {} bytes",
                        bytes.len(),
                        Self::serialized_size_static(),
                    ),
                ));
            }

            let field = |i: usize| u64_at(bytes, i);

            Ok(Self {
                version: field(0),
                graph: StorageIndex(field(1)),
                aliases: (StorageIndex(field(2)), StorageIndex(field(3))),
                indexes: StorageIndex(field(4)),
                values: StorageIndex(field(5)),
            })
        }

        fn serialized_size(&self) -> u64 {
            Self::serialized_size_static()
        }
    }

    impl SerializeStatic for DbStorageIndex {
        fn serialized_size_static() -> u64 {
            u64::serialized_size_static() * 6
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        mod db_storage_index {
            use super::*;

            mod serialize {
                use super::*;

                mod deserialize {
                    use super::*;

                    #[test]
                    fn deserialize_should_restore_serialized_index() {
                        let index = DbStorageIndex {
                            version: 1,
                            graph: StorageIndex(2),
                            aliases: (StorageIndex(3), StorageIndex(4)),
                            indexes: StorageIndex(5),
                            values: StorageIndex(6),
                        };

                        let bytes = index.serialize();
                        let deserialized = DbStorageIndex::deserialize(&bytes).unwrap();

                        assert_eq!(bytes.len() as u64, index.serialized_size());
                        assert_eq!(deserialized.serialize(), bytes);
                    }

                    #[test]
                    fn deserialize_short_bytes_should_fail() {
                        assert_eq!(
                            DbStorageIndex::deserialize(&[0; 47]).err(),
                            Some(DbError::db(
                                DbErrorType::NotEnoughData,
                                "DbStorageIndex v1 deserialization error: got 47 bytes, expected at least 48 bytes"
                            ))
                        );
                    }
                }
            }
        }
    }
}

// TODO: Remove this at some point in the future as it provides support for databases created in <= 0.10.0.
mod legacy {
    use super::*;
    use crate::DbId;
    use crate::DbKeyValue;
    use crate::collections::map::MapIterator;
    use crate::collections::multi_map::MultiMapStorage;
    use crate::db::db_key_value::DbKeyValues;
    use std::marker::PhantomData;

    pub(crate) const VERSION: u64 = 0;

    pub(crate) struct DbStorageIndex {
        pub(crate) graph: StorageIndex,
        pub(crate) aliases: (StorageIndex, StorageIndex),
        pub(crate) indexes: StorageIndex,
        pub(crate) values: StorageIndex,
    }

    impl Serialize for DbStorageIndex {
        fn serialize(&self) -> Vec<u8> {
            [
                self.graph.serialize(),
                self.aliases.0.serialize(),
                self.aliases.1.serialize(),
                self.indexes.serialize(),
                self.values.serialize(),
            ]
            .concat()
        }

        fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
            if bytes.len() < Self::serialized_size_static() as usize {
                return Err(DbError::db(
                    DbErrorType::NotEnoughData,
                    format!(
                        "DbStorageIndex legacy deserialization error: got {} bytes, expected at least {} bytes",
                        bytes.len(),
                        Self::serialized_size_static(),
                    ),
                ));
            }

            let field = |i: usize| StorageIndex(u64_at(bytes, i));

            Ok(Self {
                graph: field(0),
                aliases: (field(1), field(2)),
                indexes: field(3),
                values: field(4),
            })
        }

        fn serialized_size(&self) -> u64 {
            Self::serialized_size_static()
        }
    }

    impl SerializeStatic for DbStorageIndex {
        fn serialized_size_static() -> u64 {
            i64::serialized_size_static() * 5
        }
    }

    /// Converts the values of a database of agdb 0.10.0 and older to
    /// the version 1.
    pub(crate) fn convert<D: StorageData>(storage: &mut Storage<D>) -> Result<(), DbError> {
        let legacy_index = storage.value::<DbStorageIndex>(StorageIndex(1))?;
        let legacy_values =
            MultiMapStorage::<DbId, DbKeyValue, _>::from_storage(storage, legacy_index.values)?;
        let t = storage.transaction();
        let mut values = DbKeyValues::new(storage)?;
        let mut pos = 0;

        loop {
            let mut it = MapIterator {
                pos,
                data: &legacy_values.data,
                storage,
                phantom_data: PhantomData,
            };

            if let Some((db_id, kv)) = it.next() {
                pos = it.pos;
                values.insert_value(storage, db_id.as_index(), &kv)?;
            } else {
                break;
            }
        }

        legacy_values.remove_from_storage(storage)?;

        let db_storage_index = v1::DbStorageIndex {
            version: v1::VERSION,
            graph: legacy_index.graph,
            aliases: legacy_index.aliases,
            indexes: legacy_index.indexes,
            values: values.storage_index(),
        };

        storage.replace(StorageIndex(1), &db_storage_index)?;
        storage.commit(t)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        mod db_storage_index {
            use super::*;

            mod serialize {
                use super::*;

                mod deserialize {
                    use super::*;

                    #[test]
                    fn deserialize_should_restore_serialized_index() {
                        let index = DbStorageIndex {
                            graph: StorageIndex(1),
                            aliases: (StorageIndex(2), StorageIndex(3)),
                            indexes: StorageIndex(4),
                            values: StorageIndex(5),
                        };

                        let bytes = index.serialize();
                        let deserialized = DbStorageIndex::deserialize(&bytes).unwrap();

                        assert_eq!(bytes.len() as u64, index.serialized_size());
                        assert_eq!(deserialized.serialize(), bytes);
                    }

                    #[test]
                    fn deserialize_short_bytes_should_fail() {
                        assert_eq!(
                            DbStorageIndex::deserialize(&[0; 39]).err(),
                            Some(DbError::db(
                                DbErrorType::NotEnoughData,
                                "DbStorageIndex legacy deserialization error: got 39 bytes, expected at least 40 bytes"
                            ))
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbId;
    use crate::DbKeyValue;
    use crate::collections::multi_map::MultiMapStorage;
    use crate::collections::tests::TestStorage;
    use crate::collections::tests::assert_io_failures_recover;
    use crate::collections::tests::storage;
    use crate::db::DbImpl;
    use crate::db::db_index::DbIndexes;
    use crate::db::db_key_value::DbKeyValues;
    use crate::graph::GraphIndex;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    type TestDb = DbImpl<FileStorageImpl<TestPlatform>>;

    const GRAPH: [[i64; 4]; 4] = [
        [0, 0, i64::MIN, 2],
        [3, 0, 1, 0],
        [0, 3, 0, 1],
        [-1, -2, 0, 0],
    ];

    fn alias_maps(storage: &mut TestStorage) -> (StorageIndex, StorageIndex) {
        let mut keys = MultiMapStorage::<String, DbId, _>::new(storage).unwrap();
        let mut values = MultiMapStorage::<DbId, String, _>::new(storage).unwrap();

        for (alias, id) in [("root", 1), ("edge", -3)] {
            keys.insert(storage, &alias.to_string(), &DbId(id)).unwrap();
            values
                .insert(storage, &DbId(id), &alias.to_string())
                .unwrap();
        }

        (keys.storage_index(), values.storage_index())
    }

    /// A database of version 1: nodes 1 and 2, edge -3 from 1 to 2, the
    /// alias "root" of node 1, the alias "edge" of the edge and a value
    /// of node 1.
    fn v1_db(storage: &mut TestStorage) -> v1::DbStorageIndex {
        storage.insert_bytes(&[0; 48]).unwrap();
        let graph = graph::v1::write(storage, &GRAPH);
        let aliases = alias_maps(storage);
        let indexes = DbIndexes::new(storage).unwrap().storage_index();
        let mut values = DbKeyValues::new(storage).unwrap();
        values.insert_value(storage, 1, &("key", 1).into()).unwrap();
        let index = v1::DbStorageIndex {
            version: v1::VERSION,
            graph,
            aliases,
            indexes,
            values: values.storage_index(),
        };
        storage.replace(StorageIndex(1), &index).unwrap();
        index
    }

    /// The same database as [`v1_db`] created by agdb 0.10.0 or older.
    fn legacy_db(storage: &mut TestStorage) {
        storage.insert_bytes(&[0; 40]).unwrap();
        let graph = graph::v1::write(storage, &GRAPH);
        let aliases = alias_maps(storage);
        let indexes = DbIndexes::new(storage).unwrap().storage_index();
        let mut values = MultiMapStorage::<DbId, DbKeyValue, _>::new(storage).unwrap();
        values
            .insert(storage, &DbId(1), &("key", 1).into())
            .unwrap();
        let index = legacy::DbStorageIndex {
            graph,
            aliases,
            indexes,
            values: values.storage_index(),
        };
        storage.replace(StorageIndex(1), &index).unwrap();
    }

    fn assert_migrated(storage: &TestStorage) {
        assert_eq!(storage.version(), DB_VERSION);
        let (graph, aliases, _, values) = TestDb::load_data_structures(storage).unwrap();
        assert_eq!(graph.node_count(storage).unwrap(), 2);
        assert_eq!(graph.edge_from(storage, GraphIndex(-3)), GraphIndex(1));
        assert_eq!(
            aliases.aliases(storage, &graph).unwrap(),
            vec![("root".to_string(), DbId(1))]
        );
        assert_eq!(values.values(storage, 1).unwrap(), vec![("key", 1).into()]);
    }

    fn report(from: u64) -> DbMigration {
        DbMigration {
            from,
            to: DB_VERSION,
            fixed: vec!["alias 'edge' of edge -3 removed: edges cannot have aliases".to_string()],
        }
    }

    mod migrate {
        use super::*;

        #[test]
        fn migrate_v1_should_convert_to_current_version() {
            let mut storage = storage();
            let index = v1_db(&mut storage);

            let migration = migrate(&mut storage).unwrap();

            assert_eq!(migration, report(1));
            assert_migrated(&storage);
            assert_eq!(
                storage
                    .value::<DbStorageIndex>(StorageIndex(1))
                    .unwrap()
                    .indexes,
                index.indexes
            );

            for removed in [index.graph, index.aliases.0, index.aliases.1] {
                assert!(storage.value_size(removed).is_err(), "{removed:?}");
            }
        }

        #[test]
        fn migrate_legacy_should_convert_to_current_version() {
            let mut storage = storage();
            legacy_db(&mut storage);

            let migration = migrate(&mut storage).unwrap();

            assert_eq!(migration, report(0));
            assert_migrated(&storage);
        }

        #[test]
        fn migrate_legacy_should_keep_legacy_or_complete_on_io_failure() {
            assert_io_failures_recover(
                legacy_db,
                |storage, _| migrate(storage),
                |storage, _| {
                    if storage.version() == v1::VERSION {
                        assert_eq!(migrate(storage).unwrap(), report(0));
                    }

                    assert_migrated(storage);
                },
            );
        }

        #[test]
        fn migrate_unsupported_version_should_fail() {
            let mut storage = storage();
            let mut index = v1_db(&mut storage);
            index.version = 5;
            storage.replace(StorageIndex(1), &index).unwrap();

            assert_eq!(
                migrate(&mut storage),
                Err(DbError::db(
                    DbErrorType::NotAllowed,
                    "Database version '5' is not supported by storage version '1'"
                ))
            );
            assert_eq!(storage.version(), 1);
        }

        #[test]
        fn migrate_missing_index_should_fail() {
            let mut storage = storage();

            assert!(migrate(&mut storage).is_err());
            assert_eq!(storage.version(), 1);
        }

        #[test]
        fn migrate_should_keep_version_1_or_complete_on_io_failure() {
            assert_io_failures_recover(
                |storage| {
                    v1_db(storage);
                },
                |storage, _| migrate(storage),
                |storage, _| {
                    if storage.version() == v1::VERSION {
                        assert_eq!(migrate(storage).unwrap(), report(1));
                    }

                    assert_migrated(storage);
                },
            );
        }
    }
}
