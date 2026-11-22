use super::StorageData;
use super::alloc::ALLOCATOR_SIZE;
use super::alloc::Allocator;
use super::alloc::ROOT_BYTES;
use super::alloc::u32_at;
use super::pages::DEFAULT_PAGE_SHIFT;
use super::pages::MAX_PAGE_SHIFT;
use super::pages::Overlay;
use super::pages::Reader;
use super::pages::View;
use super::pages::ViewMut;
use super::pages::Writer;
use crate::DbError;
use crate::DbErrorType;
use crate::collections::aliases::Aliases;
use crate::collections::graph::GRAPH_SIZE;
use crate::collections::graph::GraphHeader;
use crate::collections::indexes::INDEXES_SIZE;
use crate::collections::indexes::Indexes;
use crate::collections::keys::KEYS_SIZE;
use crate::collections::keys::Keys;
use crate::collections::table::TABLE_SIZE;
use crate::utilities::serialize::u64_at;
use std::borrow::Cow;

/// Read by agdb ≤ 0.17 as the storage version record, which makes it
/// refuse the file.
const VERSION: u64 = 2;
/// Tells the root from a pre-release version 2 file, whose records start
/// where it is.
/// `agdbroot` in little-endian bytes.
const MAGIC: u64 = 0x746f_6f72_6264_6761;
/// `[0 u64][8 u64][VERSION u64][MAGIC u64][meta_len u32][page_shift u8]
/// [id_bytes u8][0 u16]`
const HEADER: usize = 40;
const GRAPH_AT: usize = HEADER + ALLOCATOR_SIZE;
const ALIASES_AT: usize = GRAPH_AT + GRAPH_SIZE;
const KEYS_AT: usize = ALIASES_AT + TABLE_SIZE;
const INDEXES_AT: usize = KEYS_AT + KEYS_SIZE;
const META_SIZE: usize = INDEXES_AT + INDEXES_SIZE - HEADER;
const ROOT_SIZE: usize = HEADER + META_SIZE;
const _: () = assert!(ROOT_SIZE as u64 <= ROOT_BYTES);

/// The width of the ids of the graph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum IdWidth {
    /// At most `u32::MAX` nodes and as many edges, 32 B per edge.
    U32,
    /// Limited by the storage only, 64 B per edge.
    U64,
}

/// What a storage is created with and keeps for its life: changing it
/// takes a rebuild.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Layout {
    pub(crate) page_shift: u32,
    pub(crate) ids: IdWidth,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            page_shift: DEFAULT_PAGE_SHIFT,
            ids: IdWidth::U32,
        }
    }
}

impl Layout {
    /// The layout if the storage supports it.
    fn validate(self) -> Result<Self, DbError> {
        if (DEFAULT_PAGE_SHIFT..=MAX_PAGE_SHIFT).contains(&self.page_shift) {
            return Ok(self);
        }

        Err(DbError::storage(
            DbErrorType::InvalidIndex,
            format!(
                "Unsupported page shift {} ({DEFAULT_PAGE_SHIFT}..={MAX_PAGE_SHIFT})",
                self.page_shift
            ),
        ))
    }

    fn decode(bytes: &[u8]) -> Result<Self, DbError> {
        let ids = match (bytes[37], u16::from_le_bytes([bytes[38], bytes[39]])) {
            (4, 0) => IdWidth::U32,
            (8, 0) => IdWidth::U64,
            (id_bytes, reserved) => {
                return Err(DbError::storage(
                    DbErrorType::InvalidIndex,
                    format!("Invalid root id width ({id_bytes} bytes, reserved {reserved})"),
                ));
            }
        };

        Self {
            page_shift: u32::from(bytes[36]),
            ids,
        }
        .validate()
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes[36] = self.page_shift as u8;
        bytes[37] = match self.ids {
            IdWidth::U32 => 4,
            IdWidth::U64 => 8,
        };
    }
}

/// What a storage holds.
#[derive(Debug, PartialEq)]
pub(crate) enum Format {
    Empty,
    Root,
    /// The records of the given storage version, read by the migration.
    Legacy(u64),
}

impl Format {
    pub(crate) fn detect(view: &impl View) -> Result<Self, DbError> {
        if view.len() == 0 {
            return Ok(Self::Empty);
        }

        let bytes = view.read(0, view.len().min(HEADER as u64))?;
        let mut header = [0_u8; HEADER];
        header[..bytes.len()].copy_from_slice(&bytes);
        let version = if u64_at(&header, 0) == 0 && u64_at(&header, 1) == 8 {
            u64_at(&header, 2)
        } else {
            0
        };

        match version {
            0 | 1 => Ok(Self::Legacy(version)),
            VERSION if u64_at(&header, 3) == MAGIC => Ok(Self::Root),
            VERSION => Err(DbError::storage(
                DbErrorType::NotAllowed,
                "Storage is a pre-release version '2' that is not supported",
            )),
            _ => Err(DbError::storage(
                DbErrorType::NotAllowed,
                format!(
                    "Storage version '{version}' is higher than the current version '{VERSION}'"
                ),
            )),
        }
    }
}

/// The state of the storage kept in its root, read on open and written
/// by each commit.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Meta {
    pub(crate) layout: Layout,
    pub(crate) alloc: Allocator,
    pub(crate) graph: GraphHeader,
    pub(crate) aliases: Aliases,
    pub(crate) keys: Keys,
    pub(crate) indexes: Indexes,
}

impl Meta {
    fn new(layout: Layout) -> Self {
        Self {
            layout,
            alloc: Allocator::new(layout.page_shift),
            graph: GraphHeader::new(layout),
            aliases: Aliases::default(),
            keys: Keys::new(layout.page_shift),
            indexes: Indexes::default(),
        }
    }

    fn load(view: &impl View) -> Result<Self, DbError> {
        Self::decode(view, &view.read(0, ROOT_SIZE as u64)?)
    }

    /// Decodes the root `bytes` of the storage `view`.
    fn decode(view: &impl View, bytes: &[u8]) -> Result<Self, DbError> {
        let meta_len = u32_at(bytes, 32);

        if meta_len as usize != META_SIZE {
            return Err(DbError::storage(
                DbErrorType::InvalidIndex,
                format!("Invalid root meta length ({meta_len} != {META_SIZE})"),
            ));
        }

        let layout = Layout::decode(bytes)?;
        let mut alloc = [0_u8; ALLOCATOR_SIZE];
        alloc.copy_from_slice(&bytes[HEADER..GRAPH_AT]);
        let alloc = Allocator::decode(&alloc, layout.page_shift);

        if alloc.len() != view.len() {
            return Err(DbError::storage(
                DbErrorType::InvalidIndex,
                format!(
                    "Invalid root storage length ({} != {})",
                    alloc.len(),
                    view.len()
                ),
            ));
        }

        let keys = Keys::decode(&bytes[KEYS_AT..INDEXES_AT], layout.page_shift);

        Ok(Self {
            layout,
            alloc,
            graph: GraphHeader::decode(&bytes[GRAPH_AT..ALIASES_AT], layout),
            aliases: Aliases::decode(&bytes[ALIASES_AT..KEYS_AT]),
            indexes: Indexes::load(view, &keys, &bytes[INDEXES_AT..ROOT_SIZE])?,
            keys,
        })
    }

    fn encode(&self) -> [u8; ROOT_SIZE] {
        let mut bytes = [0_u8; ROOT_SIZE];
        bytes[8..16].copy_from_slice(&8_u64.to_le_bytes());
        bytes[16..24].copy_from_slice(&VERSION.to_le_bytes());
        bytes[24..32].copy_from_slice(&MAGIC.to_le_bytes());
        bytes[32..36].copy_from_slice(&(META_SIZE as u32).to_le_bytes());
        self.layout.encode(&mut bytes);
        bytes[HEADER..GRAPH_AT].copy_from_slice(&self.alloc.encode());
        bytes[GRAPH_AT..ALIASES_AT].copy_from_slice(&self.graph.encode());
        bytes[ALIASES_AT..KEYS_AT].copy_from_slice(&self.aliases.encode());
        bytes[KEYS_AT..INDEXES_AT].copy_from_slice(&self.keys.encode());
        bytes[INDEXES_AT..].copy_from_slice(&self.indexes.encode());
        bytes
    }
}

/// Owns the storage, the overlay its transactions reuse and the
/// committed [`Meta`].
pub(crate) struct Root<S: StorageData> {
    storage: S,
    overlay: Overlay,
    /// `None` after a failed commit, after which the storage may hold
    /// either meta, so it is read from the root until it is reloaded.
    meta: Option<Meta>,
}

/// A transaction over the [`Root`]: its writes and meta reach the
/// storage on [`commit`](Self::commit) and are dropped otherwise.
pub(crate) struct RootMut<'a, S: StorageData> {
    pub(crate) view: Writer<'a, S>,
    pub(crate) meta: Meta,
    committed: &'a mut Option<Meta>,
}

impl<S: StorageData> Root<S> {
    /// Creates the root of the `layout` in an empty storage or loads it
    /// with the layout it has. Legacy storages are refused as they need a
    /// migration.
    pub(crate) fn open(storage: S, layout: Layout) -> Result<Self, DbError> {
        let format = Format::detect(&Reader::new(&storage))?;
        let meta = match format {
            Format::Empty => Meta::new(layout.validate()?),
            Format::Root => Meta::load(&Reader::new(&storage))?,
            Format::Legacy(version) => {
                return Err(DbError::storage(
                    DbErrorType::NotAllowed,
                    format!("Storage version '{version}' needs a migration"),
                ));
            }
        };
        let mut root = Self {
            storage,
            overlay: Overlay::new(meta.layout.page_shift),
            meta: Some(meta),
        };

        if format == Format::Empty {
            root.begin().and_then(|mut transaction| {
                transaction.view.set_len(transaction.meta.alloc.len());
                transaction.view.write(0, &transaction.meta.encode())?;
                transaction.commit()
            })?;
        }

        Ok(root)
    }

    /// The committed view and meta. It is refused while the storage may
    /// hold the writes of a transaction it failed to restore.
    pub(crate) fn read(&self) -> Result<(Reader<'_, S>, Cow<'_, Meta>), DbError> {
        if self.overlay.poisoned() {
            return Err(DbError::storage(
                DbErrorType::Poisoned,
                "Storage was not restored after a failed transaction",
            ));
        }

        let reader = Reader::new(&self.storage);
        let meta = match &self.meta {
            Some(meta) => Cow::Borrowed(meta),
            None => Cow::Owned(Meta::load(&reader)?),
        };

        Ok((reader, meta))
    }

    /// Starts a transaction, restoring the storage first if a previous
    /// one failed to.
    pub(crate) fn begin(&mut self) -> Result<RootMut<'_, S>, DbError> {
        let view = Writer::new(&mut self.storage, &mut self.overlay)?;
        let meta = match self.meta.take() {
            Some(meta) => meta,
            None => Meta::load(&view)?,
        };
        self.meta = Some(meta.clone());

        Ok(RootMut {
            view,
            meta,
            committed: &mut self.meta,
        })
    }
}

impl<S: StorageData> RootMut<'_, S> {
    /// Writes the meta to the root only if it changed: the root already
    /// holds the committed one.
    pub(crate) fn commit(mut self) -> Result<(), DbError> {
        let root = self.meta.encode();

        if self.committed.as_ref().map(Meta::encode) != Some(root) {
            self.view.write(0, &root)?;
        }

        let result = self.view.commit();
        *self.committed = result.is_ok().then_some(self.meta);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbId;
    use crate::storage::DEFAULT_OVERLAY_LIMIT;
    use crate::storage::SyncMode;
    use crate::storage::alloc::tests::pattern;
    use crate::storage::alloc::tests::random_op;
    use crate::storage::alloc::tests::sizes;
    use crate::storage::alloc::tests::verify;
    use crate::storage::file_storage::FileStorageImpl;
    use crate::storage::file_storage_memory_mapped::FileStorageMemoryMappedImpl;
    use crate::storage::pages::tests::XorShift;
    use agplatform::Platform;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;
    use std::sync::Arc;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    type TestFile = FileStorageImpl<TestPlatform>;
    type Live = Vec<(u64, u64, u64)>;

    fn platform() -> Arc<TestPlatform> {
        Arc::new(test_platform())
    }

    fn storage(platform: &Arc<TestPlatform>) -> TestFile {
        FileStorageImpl::with_platform(platform.clone(), NAME).unwrap()
    }

    fn with_bytes(platform: &Arc<TestPlatform>, bytes: &[u8]) -> TestFile {
        let mut storage = storage(platform);
        storage.write(0, bytes).unwrap();
        storage.flush().unwrap();
        storage
    }

    fn open(platform: &Arc<TestPlatform>) -> Root<TestFile> {
        Root::open(storage(platform), Layout::default()).unwrap()
    }

    fn strict(platform: &Arc<TestPlatform>) -> Root<TestFile> {
        let mut storage = storage(platform);
        storage.set_sync_mode(SyncMode::Strict);
        Root::open(storage, Layout::default()).unwrap()
    }

    fn words(words: &[u64]) -> Vec<u8> {
        words.iter().flat_map(|word| word.to_le_bytes()).collect()
    }

    fn content(view: &impl View) -> Vec<u8> {
        view.read(0, view.len()).unwrap().to_vec()
    }

    fn assert_live(view: &impl View, live: &Live) {
        for (pos, size, id) in live.iter().copied() {
            assert_eq!(view.read(pos, size).unwrap().to_vec(), pattern(id, size));
        }
    }

    /// Allocates `sizes` holding their patterns in the transaction.
    fn alloc<S: StorageData>(transaction: &mut RootMut<'_, S>, live: &mut Live, sizes: &[u64]) {
        for size in sizes.iter().copied() {
            let id = live.len() as u64 + 1;
            let pos = transaction
                .meta
                .alloc
                .alloc(&mut transaction.view, size)
                .unwrap();
            transaction.view.write(pos, &pattern(id, size)).unwrap();
            live.push((pos, size, id));
        }
    }

    /// Inserts a node with a self-loop, an alias and an indexed value.
    fn collections<S: StorageData>(transaction: &mut RootMut<'_, S>) {
        let meta = &mut transaction.meta;
        let view = &mut transaction.view;
        let node = meta.graph.insert_node(view, &mut meta.alloc).unwrap();
        meta.graph
            .insert_edge(view, &mut meta.alloc, node, node)
            .unwrap();
        meta.aliases
            .insert(view, &mut meta.alloc, &meta.graph, node, "root")
            .unwrap();
        meta.indexes
            .insert_index(view, &mut meta.alloc, &mut meta.keys, &"key".into())
            .unwrap();
        let key = meta.indexes.list()[0].key_id;
        meta.indexes
            .insert(view, &mut meta.alloc, key, &"value".into(), node)
            .unwrap();
    }

    /// Commits the allocations of `sizes`, returning the meta and the
    /// live allocations.
    fn committed(root: &mut Root<TestFile>, sizes: &[u64]) -> (Meta, Live) {
        let mut transaction = root.begin().unwrap();
        let mut live = Vec::new();
        alloc(&mut transaction, &mut live, sizes);
        let meta = transaction.meta.clone();
        transaction.commit().unwrap();
        (meta, live)
    }

    /// A transaction that allocates, frees and grows, returning the meta
    /// and the live allocations it would commit.
    fn changed(transaction: &mut RootMut<'_, TestFile>, live: &Live) -> (Meta, Live) {
        let mut live = live.clone();
        alloc(transaction, &mut live, &[24, 3000, 600]);
        let (pos, size, _) = live.swap_remove(0);
        transaction
            .meta
            .alloc
            .free(&mut transaction.view, pos, size)
            .unwrap();
        (transaction.meta.clone(), live)
    }

    /// Asserts that the committed meta is the one of either state and
    /// that the storage holds its allocations. Returns whether it is the
    /// second one.
    fn assert_either(root: &Root<TestFile>, states: [&(Meta, Live); 2]) -> bool {
        let (reader, meta) = root.read().unwrap();
        let second = *meta == states[1].0;

        if !second {
            assert_eq!(*meta, states[0].0);
        }

        let live = &states[usize::from(second)].1;
        verify(&meta.alloc, &reader, &sizes(live));
        assert_live(&reader, live);
        second
    }

    fn assert_state(root: &Root<TestFile>, state: &(Meta, Live)) {
        assert_either(root, [state, state]);
    }

    /// Drops a spilled transaction while the WAL cannot be read.
    fn poison(platform: &Arc<TestPlatform>, root: &mut Root<TestFile>) {
        root.storage.set_overlay_limit(0);
        let mut transaction = root.begin().unwrap();
        alloc(&mut transaction, &mut Vec::new(), &[3000]);
        platform
            .fs()
            .set_io_error(FsOp::Read, WAL, ErrorKind::Other);
        drop(transaction);
        platform.fs().clear_faults();
        root.storage.set_overlay_limit(DEFAULT_OVERLAY_LIMIT);
    }

    /// Runs transactions of random operations over the root, dropping
    /// about a third of them and reopening the storage at random.
    fn random_transactions<S: StorageData>(open: impl Fn() -> S, seed: u64, layout: Layout) {
        let mut random = XorShift(seed);
        let mut root = Root::open(open(), layout).unwrap();
        let mut live = Vec::new();
        let mut id = 0;

        for _ in 0..10 {
            let mut transaction = root.begin().unwrap();
            let mut changed = live.clone();

            for _ in 0..50 {
                id += 1;
                random_op(
                    &mut transaction.meta.alloc,
                    &mut transaction.view,
                    &mut changed,
                    &mut random,
                    id,
                )
                .unwrap();
            }

            if random.next(3) == 0 {
                drop(transaction);
            } else {
                transaction.commit().unwrap();
                live = changed;
            }

            if random.next(3) == 0 {
                drop(root);
                root = Root::open(open(), Layout::default()).unwrap();
            }

            let (reader, meta) = root.read().unwrap();
            verify(&meta.alloc, &reader, &sizes(&live));
            assert_live(&reader, &live);
        }
    }

    mod format {
        use super::*;

        mod detect {
            use super::*;

            #[test]
            fn detect_empty_should_return_empty() {
                let platform = platform();

                let format = Format::detect(&Reader::new(&storage(&platform))).unwrap();

                assert_eq!(format, Format::Empty);
            }

            #[test]
            fn detect_root_should_return_root() {
                let platform = platform();
                drop(open(&platform));

                let format = Format::detect(&Reader::new(&storage(&platform))).unwrap();

                assert_eq!(format, Format::Root);
            }

            #[test]
            fn detect_legacy_fixtures_should_return_legacy_1() {
                for fixture in ["tests/test_db_prior_0_11_0.agdb", "tests/test_db_v1.agdb"] {
                    let platform = platform();
                    let storage = with_bytes(&platform, &std::fs::read(fixture).unwrap());

                    let format = Format::detect(&Reader::new(&storage)).unwrap();

                    assert_eq!(format, Format::Legacy(1));
                }
            }

            #[test]
            fn detect_without_version_record_should_return_legacy_0() {
                for bytes in [words(&[1, 8, 2]), words(&[0, 16, 2]), vec![0; 5]] {
                    let platform = platform();
                    let storage = with_bytes(&platform, &bytes);

                    let format = Format::detect(&Reader::new(&storage)).unwrap();

                    assert_eq!(format, Format::Legacy(0));
                }
            }

            #[test]
            fn detect_pre_release_version_2_should_fail() {
                let platform = platform();
                let storage = with_bytes(&platform, &words(&[0, 8, 2, 1, 16, 0]));

                let error = Format::detect(&Reader::new(&storage)).unwrap_err();

                assert_eq!(
                    error.description,
                    "Storage is a pre-release version '2' that is not supported"
                );
            }

            #[test]
            fn detect_higher_version_should_fail() {
                let platform = platform();
                let storage = with_bytes(&platform, &words(&[0, 8, 3, MAGIC]));

                let error = Format::detect(&Reader::new(&storage)).unwrap_err();

                assert_eq!(
                    error.description,
                    "Storage version '3' is higher than the current version '2'"
                );
            }

            #[test]
            fn detect_read_failure_should_fail() {
                let platform = platform();
                let storage = with_bytes(&platform, &words(&[0, 8, 1]));
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(Format::detect(&Reader::new(&storage)).is_err());
            }
        }
    }

    mod meta {
        use super::*;

        mod decode {
            use super::*;

            #[test]
            fn decode_should_return_encoded_meta() {
                let platform = platform();
                let mut root = open(&platform);
                let mut transaction = root.begin().unwrap();
                collections(&mut transaction);
                transaction.commit().unwrap();
                let (reader, meta) = root.read().unwrap();

                let decoded = Meta::decode(&reader, &meta.encode()).unwrap();

                assert_eq!(decoded, *meta);
            }

            #[test]
            fn decode_meta_len_mismatch_should_fail() {
                let platform = platform();
                let root = open(&platform);
                let mut bytes = Meta::new(Layout::default()).encode();
                bytes[32] += 1;

                let error = Meta::decode(&root.read().unwrap().0, &bytes).unwrap_err();

                assert_eq!(
                    error.description,
                    format!(
                        "Invalid root meta length ({} != {META_SIZE})",
                        META_SIZE + 1
                    )
                );
            }

            #[test]
            fn decode_invalid_layout_should_fail() {
                let platform = platform();
                let root = open(&platform);
                let reader = root.read().unwrap().0;
                let encoded = Meta::new(Layout::default()).encode();
                let mut shift = encoded;
                shift[36] = 16;
                let mut ids = encoded;
                ids[37] = 5;
                let mut reserved = encoded;
                reserved[39] = 1;

                let errors = [shift, ids, reserved]
                    .map(|bytes| Meta::decode(&reader, &bytes).unwrap_err().description);

                assert_eq!(
                    errors,
                    [
                        "Unsupported page shift 16 (10..=15)",
                        "Invalid root id width (5 bytes, reserved 0)",
                        "Invalid root id width (4 bytes, reserved 256)"
                    ]
                );
            }

            #[test]
            fn decode_storage_len_mismatch_should_fail() {
                let platform = platform();
                let mut root = open(&platform);
                committed(&mut root, &[9000]);
                let reader = root.read().unwrap().0;

                let error =
                    Meta::decode(&reader, &Meta::new(Layout::default()).encode()).unwrap_err();

                assert_eq!(
                    error.description,
                    format!("Invalid root storage length (4096 != {})", reader.len())
                );
            }

            #[test]
            fn decode_indexes_outside_storage_should_fail() {
                let platform = platform();
                let root = open(&platform);
                let mut bytes = Meta::new(Layout::default()).encode();
                bytes[INDEXES_AT..].copy_from_slice(&words(&[4096, 1]));

                let error = Meta::decode(&root.read().unwrap().0, &bytes).unwrap_err();

                assert_eq!(error.description, "Read (4096..4136) out of bounds (4096)");
            }
        }
    }

    mod root {
        use super::*;

        #[test]
        fn root_should_keep_allocations_over_random_transactions_and_reopens() {
            let platform = platform();

            for seed in 1..=4 {
                for limit in [0, 4096, DEFAULT_OVERLAY_LIMIT] {
                    let name = format!("/file{seed}_{limit}");
                    let file = || {
                        let mut file =
                            FileStorageImpl::with_platform(platform.clone(), &name).unwrap();
                        file.set_overlay_limit(limit);
                        file
                    };
                    random_transactions(file, seed, Layout::default());
                    let name = format!("/mapped{seed}_{limit}");
                    let mapped = || {
                        let mut mapped =
                            FileStorageMemoryMappedImpl::with_platform(platform.clone(), &name)
                                .unwrap();
                        mapped.set_overlay_limit(limit);
                        mapped
                    };
                    random_transactions(mapped, seed, Layout::default());
                }
            }
        }

        #[test]
        fn root_with_largest_pages_should_keep_allocations_over_random_transactions_and_reopens() {
            let platform = platform();
            let layout = Layout {
                page_shift: MAX_PAGE_SHIFT,
                ids: IdWidth::U32,
            };

            for seed in 1..=2 {
                let name = format!("/file{seed}");
                let file = || FileStorageImpl::with_platform(platform.clone(), &name).unwrap();
                random_transactions(file, seed, layout);
            }
        }

        mod open {
            use super::*;

            #[test]
            fn open_empty_should_create_root() {
                let platform = platform();

                let root = open(&platform);

                let mut expected = Meta::new(Layout::default()).encode().to_vec();
                expected.resize(4096, 0);
                assert_eq!(content(&root.read().unwrap().0), expected);
                assert_eq!(*root.read().unwrap().1, Meta::new(Layout::default()));
                assert_eq!(expected[..32], words(&[0, 8, 2, MAGIC]));
            }

            #[test]
            fn open_should_load_committed_meta() {
                let platform = platform();
                let mut root = open(&platform);
                let state = committed(&mut root, &[16, 600, 9000]);
                drop(root);

                let root = open(&platform);

                assert_state(&root, &state);
            }

            #[test]
            fn open_should_keep_layout_of_storage() {
                let platform = platform();
                let layout = Layout {
                    page_shift: MAX_PAGE_SHIFT,
                    ids: IdWidth::U64,
                };
                let mut root = Root::open(storage(&platform), layout).unwrap();
                let state = committed(&mut root, &[16, 600, 40000]);
                drop(root);

                let root = open(&platform);

                assert_eq!(root.read().unwrap().1.layout, layout);
                assert_eq!(root.overlay.page_size(), 1 << MAX_PAGE_SHIFT);
                assert_state(&root, &state);
            }

            #[test]
            fn open_empty_with_unsupported_page_shift_should_fail() {
                let platform = platform();

                let errors = [DEFAULT_PAGE_SHIFT - 1, MAX_PAGE_SHIFT + 1].map(|page_shift| {
                    let layout = Layout {
                        page_shift,
                        ids: IdWidth::U32,
                    };
                    Root::open(storage(&platform), layout)
                        .err()
                        .unwrap()
                        .description
                });

                assert_eq!(
                    errors,
                    [
                        "Unsupported page shift 9 (10..=15)",
                        "Unsupported page shift 16 (10..=15)"
                    ]
                );
                assert_eq!(storage(&platform).len(), 0);
            }

            #[test]
            fn open_should_load_committed_collections() {
                let platform = platform();
                let mut root = open(&platform);
                let mut transaction = root.begin().unwrap();
                collections(&mut transaction);
                let meta = transaction.meta.clone();
                transaction.commit().unwrap();
                drop(root);

                let root = open(&platform);

                let (reader, loaded) = root.read().unwrap();
                assert_eq!(*loaded, meta);
                assert_eq!(
                    loaded.aliases.id(&reader, &loaded.graph, "root").unwrap(),
                    Some(DbId(1))
                );
                assert_eq!(
                    loaded
                        .indexes
                        .search(&reader, &"key".into(), &"value".into())
                        .unwrap(),
                    Some(vec![DbId(1)])
                );
                assert_eq!(loaded.graph.edge_count(), 1);
            }

            #[test]
            fn open_legacy_should_fail() {
                let platform = platform();
                let storage = with_bytes(&platform, &words(&[0, 8, 1]));

                let error = Root::open(storage, Layout::default()).err().unwrap();

                assert_eq!(error.description, "Storage version '1' needs a migration");
            }

            #[test]
            fn open_higher_version_should_fail() {
                let platform = platform();
                let storage = with_bytes(&platform, &words(&[0, 8, 3]));

                assert!(Root::open(storage, Layout::default()).is_err());
            }

            #[test]
            fn open_short_root_should_fail() {
                let platform = platform();
                let storage = with_bytes(&platform, &words(&[0, 8, 2, MAGIC]));

                assert!(Root::open(storage, Layout::default()).is_err());
            }

            #[test]
            fn open_create_failure_should_leave_storage_empty() {
                for limit in [0, DEFAULT_OVERLAY_LIMIT] {
                    let platform = platform();
                    let mut storage = storage(&platform);
                    storage.set_overlay_limit(limit);
                    platform
                        .fs()
                        .set_io_error(FsOp::Write, NAME, ErrorKind::Other);

                    assert!(Root::open(storage, Layout::default()).is_err());

                    platform.fs().clear_faults();
                    assert!(self::storage(&platform).is_empty(), "limit: {limit}");
                }
            }
        }

        mod read {
            use super::*;

            #[test]
            fn read_should_return_committed_meta() {
                let platform = platform();
                let mut root = open(&platform);

                let state = committed(&mut root, &[16, 600, 9000]);

                assert_state(&root, &state);
            }

            #[test]
            fn read_poisoned_should_fail() {
                let platform = platform();
                let mut root = open(&platform);
                poison(&platform, &mut root);

                let error = root.read().err().unwrap();

                assert_eq!(error.ty, DbErrorType::Poisoned);
            }

            #[test]
            fn read_stale_should_load_meta_from_storage() {
                let platform = platform();
                let mut root = open(&platform);
                let state = committed(&mut root, &[16, 600]);
                root.meta = None;

                assert_state(&root, &state);
            }

            #[test]
            fn read_stale_failure_should_fail() {
                let platform = platform();
                let mut root = open(&platform);
                root.meta = None;
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(root.read().is_err());
            }
        }

        mod begin {
            use super::*;

            #[test]
            fn begin_poisoned_should_restore_storage() {
                let platform = platform();
                let mut root = open(&platform);
                let state = committed(&mut root, &[16, 600, 9000]);
                poison(&platform, &mut root);

                drop(root.begin().unwrap());

                assert_state(&root, &state);
            }

            #[test]
            fn begin_restore_failure_should_fail() {
                let platform = platform();
                let mut root = open(&platform);
                poison(&platform, &mut root);
                platform
                    .fs()
                    .set_io_error(FsOp::Read, WAL, ErrorKind::Other);

                assert!(root.begin().is_err());
                assert!(root.overlay.poisoned());
            }

            #[test]
            fn begin_stale_should_reload_meta() {
                let platform = platform();
                let mut root = open(&platform);
                let state = committed(&mut root, &[16, 600]);
                root.meta = None;

                let transaction = root.begin().unwrap();

                assert_eq!(transaction.meta, state.0);
                drop(transaction);
                assert_eq!(root.meta, Some(state.0));
            }

            #[test]
            fn begin_stale_failure_should_fail() {
                let platform = platform();
                let mut root = open(&platform);
                root.meta = None;
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(root.begin().is_err());
                assert!(root.meta.is_none());
            }
        }
    }

    mod root_mut {
        use super::*;

        #[test]
        fn root_mut_should_roll_back_when_dropped() {
            let platform = platform();
            let mut root = open(&platform);
            let state = committed(&mut root, &[16, 600, 9000]);
            let mut transaction = root.begin().unwrap();
            let after = changed(&mut transaction, &state.1);

            drop(transaction);

            assert!(!assert_either(&root, [&state, &after]));
        }

        mod commit {
            use super::*;

            #[test]
            fn commit_should_store_meta() {
                let platform = platform();
                let mut root = open(&platform);
                let state = committed(&mut root, &[16, 600, 9000]);
                let mut transaction = root.begin().unwrap();
                let after = changed(&mut transaction, &state.1);

                transaction.commit().unwrap();

                assert!(assert_either(&root, [&state, &after]));
                assert_eq!(
                    root.read().unwrap().0.read(0, ROOT_SIZE as u64).unwrap(),
                    &after.0.encode()[..]
                );
                drop(root);
                assert!(assert_either(&open(&platform), [&state, &after]));
            }

            #[test]
            fn commit_root_write_failure_should_roll_back() {
                let platform = platform();
                let mut root = open(&platform);
                let state = committed(&mut root, &[16, 600, 9000]);
                let mut transaction = root.begin().unwrap();
                let after = changed(&mut transaction, &state.1);
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(transaction.commit().is_err());

                platform.fs().clear_faults();
                assert!(root.meta.is_some());
                assert!(!assert_either(&root, [&state, &after]));
            }

            #[test]
            fn commit_unchanged_meta_should_skip_root() {
                let platform = platform();
                let mut root = strict(&platform);
                let state = committed(&mut root, &[16, 600, 9000]);
                for op in [FsOp::Read, FsOp::Write, FsOp::SetLen, FsOp::SyncData] {
                    platform.fs().set_io_error(op, "/", ErrorKind::Other);
                }

                root.begin().unwrap().commit().unwrap();

                platform.fs().clear_faults();
                assert_state(&root, &state);
            }

            #[test]
            fn commit_unchanged_meta_should_store_writes() {
                let platform = platform();
                let mut root = open(&platform);
                let mut state = committed(&mut root, &[16, 600, 9000]);
                let mut transaction = root.begin().unwrap();
                let (pos, size, _) = state.1[1];
                transaction.view.write(pos, &pattern(7, size)).unwrap();
                state.1[1].2 = 7;

                transaction.commit().unwrap();

                assert_state(&root, &state);
                drop(root);
                assert_state(&open(&platform), &state);
            }

            #[test]
            fn commit_failure_should_keep_meta_of_storage() {
                for (mode, op) in
                    [SyncMode::Commit, SyncMode::Strict]
                        .into_iter()
                        .flat_map(|mode| {
                            [FsOp::Read, FsOp::Write, FsOp::SetLen, FsOp::SyncData]
                                .map(|op| (mode, op))
                        })
                {
                    for nth in 0.. {
                        let platform = platform();
                        let mut root = open(&platform);
                        root.storage.set_sync_mode(mode);
                        let state = committed(&mut root, &[16, 600, 9000]);
                        let mut transaction = root.begin().unwrap();
                        let after = changed(&mut transaction, &state.1);
                        platform.fs().set_io_errors(
                            op,
                            "/",
                            (0..nth).map(|_| None).chain([Some(ErrorKind::Other), None]),
                        );

                        let result = transaction.commit();

                        platform.fs().clear_faults();
                        drop(root.begin().unwrap());
                        let second = assert_either(&root, [&state, &after]);
                        drop(root);
                        let reopened = assert_either(&open(&platform), [&state, &after]);
                        assert_eq!(reopened, second, "mode: {mode:?}, op: {op:?}, nth: {nth}");

                        if result.is_ok() {
                            assert!(second);
                            break;
                        }
                    }
                }
            }

            #[test]
            fn commit_crash_should_reopen_either_state() {
                for op in [FsOp::Write, FsOp::SetLen, FsOp::SyncData] {
                    for nth in 0.. {
                        let platform = platform();
                        let mut root = strict(&platform);
                        let state = committed(&mut root, &[16, 600, 9000]);
                        let mut transaction = root.begin().unwrap();
                        let after = changed(&mut transaction, &state.1);
                        platform.fs().set_io_errors(
                            op,
                            "/",
                            (0..nth).map(|_| None).chain([Some(ErrorKind::Other)]),
                        );

                        let result = transaction.commit();

                        platform.fs().crash();
                        platform.fs().clear_faults();
                        drop(root);
                        let second = assert_either(&strict(&platform), [&state, &after]);

                        if result.is_ok() {
                            assert!(second, "op: {op:?}, nth: {nth}");
                            break;
                        }
                    }
                }
            }
        }
    }
}
