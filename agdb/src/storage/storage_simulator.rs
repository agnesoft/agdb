use super::DEFAULT_OVERLAY_LIMIT;
use super::StorageData;
use super::StorageSlice;
use super::pages::DEFAULT_PAGE_SHIFT;
use crate::DbError;
use crate::DbErrorType;
use agplatform::Fs;
use agplatform::Platform;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

/// The bytes of a chunk: the smallest page.
const CHUNK: u64 = 1 << DEFAULT_PAGE_SHIFT;

/// The I/O of a [`StorageSimulator`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Io {
    pub(crate) reads: u64,
    pub(crate) writes: u64,
    /// The chunks the reads and writes spanned.
    pub(crate) chunks: u64,
    /// The chunks first changed since the last flush, whose bytes a log
    /// would hold.
    pub(crate) logged: u64,
    pub(crate) flushes: u64,
}

/// A storage of zero bytes that holds only the chunks written to it, so
/// that a file of terabytes costs the chunks it uses, and counts its
/// [`Io`]. A chunk first changed since the last flush keeps its bytes
/// as of the flush, which [`recover_from_wal`](StorageData::recover_from_wal)
/// restores like a log. A backup writes the bytes to the file system of
/// the platform `P`.
pub(crate) struct StorageSimulator<P: Platform> {
    platform: Arc<P>,
    name: String,
    len: u64,
    chunks: BTreeMap<u64, Vec<u8>>,
    /// The length and the chunks as of the last flush, of the chunks
    /// changed since.
    flushed: (u64, HashMap<u64, Option<Vec<u8>>>),
    overlay_limit: u64,
    io: Cell<Io>,
}

impl<P: Platform> StorageSimulator<P> {
    fn with_platform(platform: Arc<P>, name: &str) -> Self {
        Self {
            platform,
            name: name.to_string(),
            len: 0,
            chunks: BTreeMap::new(),
            flushed: (0, HashMap::new()),
            overlay_limit: DEFAULT_OVERLAY_LIMIT,
            io: Cell::new(Io::default()),
        }
    }

    fn bytes(&self, pos: u64, end: u64) -> Vec<u8> {
        let mut bytes = vec![0; (end - pos) as usize];
        let mut at = pos;

        while at < end {
            let offset = at % CHUNK;
            let size = (CHUNK - offset).min(end - at);

            if let Some(chunk) = self.chunks.get(&(at / CHUNK)) {
                bytes[(at - pos) as usize..][..size as usize]
                    .copy_from_slice(&chunk[offset as usize..][..size as usize]);
            }

            at += size;
        }

        bytes
    }

    /// Returns the I/O since the last call.
    pub(crate) fn take_io(&self) -> Io {
        self.io.take()
    }

    fn count(&self, pos: u64, len: u64, write: bool) {
        let mut io = self.io.get();
        let first = pos / CHUNK;
        io.chunks += (pos + len).div_ceil(CHUNK).max(first + 1) - first;

        if write {
            io.writes += 1;
        } else {
            io.reads += 1;
        }

        self.io.set(io);
    }

    /// Keeps the bytes of the `chunk` as of the last flush.
    fn log(&mut self, chunk: u64) {
        if !self.flushed.1.contains_key(&chunk) {
            self.flushed
                .1
                .insert(chunk, self.chunks.get(&chunk).cloned());
            let mut io = self.io.get();
            io.logged += 1;
            self.io.set(io);
        }
    }
}

impl<P: Platform + Default> StorageData for StorageSimulator<P> {
    fn backup(&self, name: &str) -> Result<(), DbError> {
        self.platform.fs().write(name, self.bytes(0, self.len))?;
        Ok(())
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        let mut copy = Self::with_platform(self.platform.clone(), name);
        copy.len = self.len;
        copy.chunks = self.chunks.clone();
        copy.flushed.0 = self.len;
        Ok(copy)
    }

    fn create(&self, name: &str) -> Result<Self, DbError> {
        Ok(Self::with_platform(self.platform.clone(), name))
    }

    fn flush(&mut self) -> Result<(), DbError> {
        self.flushed.0 = self.len;
        self.flushed.1.clear();
        let mut io = self.io.get();
        io.flushes += 1;
        self.io.set(io);
        Ok(())
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Ok(Self::with_platform(Arc::new(P::default()), name))
    }

    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError> {
        let end = pos.saturating_add(value_len);

        if end > self.len {
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!("Read ({pos}..{end}) out of bounds ({})", self.len),
            ));
        }

        self.count(pos, value_len, false);
        Ok(StorageSlice::Owned(self.bytes(pos, end)))
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        self.name = new_name.to_string();
        Ok(())
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        let cut = self
            .chunks
            .range(new_len.div_ceil(CHUNK)..)
            .map(|(chunk, _)| *chunk)
            .collect::<Vec<_>>();

        for chunk in cut {
            self.log(chunk);
            self.chunks.remove(&chunk);
        }

        let last = new_len / CHUNK;

        if self.chunks.contains_key(&last) {
            self.log(last);
            self.chunks.get_mut(&last).unwrap()[(new_len % CHUNK) as usize..].fill(0);
        }

        self.len = new_len;
        Ok(())
    }

    fn set_overlay_limit(&mut self, limit: u64) {
        self.overlay_limit = limit;
    }

    fn overlay_limit(&self) -> u64 {
        self.overlay_limit
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        if bytes.is_empty() {
            return Ok(());
        }

        self.count(pos, bytes.len() as u64, true);
        let mut at = pos;
        let mut rest = bytes;

        while !rest.is_empty() {
            let chunk = at / CHUNK;
            let offset = (at % CHUNK) as usize;
            let size = (CHUNK as usize - offset).min(rest.len());
            self.log(chunk);
            self.chunks
                .entry(chunk)
                .or_insert_with(|| vec![0; CHUNK as usize])[offset..offset + size]
                .copy_from_slice(&rest[..size]);
            at += size as u64;
            rest = &rest[size..];
        }

        self.len = self.len.max(pos + bytes.len() as u64);
        Ok(())
    }

    fn recover_from_wal(&mut self) -> Result<(), DbError> {
        let (len, chunks) = std::mem::take(&mut self.flushed);

        for (chunk, bytes) in chunks {
            match bytes {
                Some(bytes) => self.chunks.insert(chunk, bytes),
                None => self.chunks.remove(&chunk),
            };
        }

        self.len = len;
        self.flushed.0 = len;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::storage::alloc::Allocator;
    use crate::storage::pages::Overlay;
    use crate::storage::pages::Writer;
    use crate::storage::pages::tests::XorShift;
    use crate::storage::tests::storage_data as contract;
    use agplatform::test_platform::TestPlatform;

    pub(crate) type TestSimulator = StorageSimulator<TestPlatform>;

    /// The elements the collections of the scale tests grow to.
    pub(crate) const SCALE: u64 = 100_000;

    /// The I/O of the operations by the decade of the size of the collection
    /// they ran on: the most chunks one spanned, their sum and count.
    #[derive(Default)]
    pub(crate) struct Scale([(u64, u64, u64); 7]);

    impl Scale {
        fn add(&mut self, io: Io, size: u64) {
            let decade = &mut self.0[size.max(1).ilog10() as usize];
            *decade = (decade.0.max(io.chunks), decade.1 + io.chunks, decade.2 + 1);
        }

        /// Asserts that the operations ran on 1k to [`SCALE`] elements, that
        /// in the full decades past 10k they spanned on average at most 10 %
        /// more chunks than from 1k to 10k, and that none spanned twice as
        /// many as the most there: the longest table overflow chain grows a
        /// little with the buckets, an operation copying its collection would
        /// grow tenfold per decade.
        pub(crate) fn assert_flat(&self) {
            let mean = |decade: &(u64, u64, u64)| decade.1 as f64 / decade.2.max(1) as f64;
            let base = self.0[3];
            let top = (SCALE - 1).ilog10() as usize;
            assert!(
                self.0[3..=top].iter().all(|decade| decade.2 != 0)
                    && self.0[4..=top]
                        .iter()
                        .all(|decade| mean(decade) <= 1.1 * mean(&base))
                    && self.0[4..].iter().all(|decade| decade.0 <= 2 * base.0),
                "{:?}",
                self.0
            );
        }
    }

    /// Runs each operation in a transaction of its own over a
    /// [`StorageSimulator`] holding an empty heap.
    pub(crate) struct Simulation {
        pub(crate) storage: TestSimulator,
        overlay: Overlay,
        pub(crate) alloc: Allocator,
    }

    impl Simulation {
        pub(crate) fn new() -> Self {
            let alloc = Allocator::new(DEFAULT_PAGE_SHIFT);
            let mut storage = TestSimulator::new("/simulation").unwrap();
            storage.resize(alloc.len()).unwrap();
            storage.flush().unwrap();

            Self {
                storage,
                overlay: Overlay::new(DEFAULT_PAGE_SHIFT),
                alloc,
            }
        }

        /// Runs and commits the `op`, adding its I/O to the `scale` as that
        /// of an operation on `size` elements.
        pub(crate) fn run<T>(
            &mut self,
            scale: &mut Scale,
            size: u64,
            op: impl FnOnce(&mut Writer<TestSimulator>, &mut Allocator) -> Result<T, DbError>,
        ) -> T {
            self.storage.take_io();
            let mut writer = Writer::new(&mut self.storage, &mut self.overlay).unwrap();
            let result = op(&mut writer, &mut self.alloc).unwrap();
            writer.commit().unwrap();
            scale.add(self.storage.take_io(), size);
            result
        }
    }

    const NAME: &str = "/simulator";
    const TIB: u64 = 1 << 40;

    fn open() -> TestSimulator {
        TestSimulator::new(NAME).unwrap()
    }

    fn bytes(storage: &TestSimulator) -> Vec<u8> {
        storage.read(0, storage.len()).unwrap().to_vec()
    }

    mod storage_simulator {
        use super::*;

        #[test]
        fn storage_simulator_should_recover_flushed_bytes_over_random_changes() {
            let mut random = XorShift(1);
            let mut storage = open();
            let mut expected = Vec::new();
            let mut flushed = Vec::new();

            for round in 0..2000 {
                match random.next(8) {
                    0 => {
                        let len = random.next(6 * CHUNK);
                        storage.resize(len).unwrap();
                        expected.resize(len as usize, 0);
                    }
                    1 => {
                        storage.flush().unwrap();
                        flushed = expected.clone();
                    }
                    2 => {
                        storage.recover_from_wal().unwrap();
                        expected = flushed.clone();
                    }
                    _ => {
                        let pos = random.next(expected.len() as u64 + CHUNK) as usize;
                        let value =
                            vec![1 + random.next(255) as u8; 1 + random.next(3000) as usize];
                        storage.write(pos as u64, &value).unwrap();
                        expected.resize(expected.len().max(pos + value.len()), 0);
                        expected[pos..pos + value.len()].copy_from_slice(&value);
                    }
                }

                assert_eq!(bytes(&storage), expected, "round: {round}");
            }
        }

        #[test]
        fn storage_simulator_should_hold_only_chunks_written() {
            let mut storage = open();

            storage.write(TIB, &[1, 2]).unwrap();
            storage.resize(4 * TIB).unwrap();

            assert_eq!(storage.len(), 4 * TIB);
            assert_eq!(storage.chunks.len(), 1);
            assert_eq!(storage.read(TIB - 1, 4).unwrap().to_vec(), vec![0, 1, 2, 0]);
            storage.recover_from_wal().unwrap();
            assert_eq!(storage.len(), 0);
            assert!(storage.chunks.is_empty());
        }

        mod take_io {
            use super::*;

            #[test]
            fn take_io_should_return_io_since_last_call() {
                let mut storage = open();
                storage.write(CHUNK - 1, &[1, 2]).unwrap();
                storage.flush().unwrap();
                storage.write(0, &[3]).unwrap();
                storage.read(0, CHUNK + 1).unwrap();
                storage.resize(CHUNK).unwrap();

                assert_eq!(
                    storage.take_io(),
                    Io {
                        reads: 1,
                        writes: 2,
                        chunks: 5,
                        logged: 4,
                        flushes: 1,
                    }
                );
                assert_eq!(storage.take_io(), Io::default());
            }
        }

        mod storage_data {
            use super::*;

            mod backup {
                use super::*;

                #[test]
                fn backup_should_write_bytes_to_name() {
                    let storage = open();
                    let platform = storage.platform.clone();

                    contract::backup::backup_should_write_bytes_to_name(
                        storage,
                        "/backup",
                        |name| platform.fs().read(name).unwrap(),
                    );
                }

                #[test]
                fn backup_failure_should_fail() {
                    assert!(open().backup("/missing/backup").is_err());
                }
            }

            mod copy {
                use super::*;

                #[test]
                fn copy_should_create_independent_storage_named_name() {
                    contract::copy::copy_should_create_independent_storage_named_name(
                        open(),
                        "/copy",
                    );
                }
            }

            mod create {
                use super::*;

                #[test]
                fn create_should_replace_name_with_empty_storage() {
                    contract::create::create_should_replace_name_with_empty_storage(
                        open(),
                        "/created",
                    );
                }
            }

            mod flush {
                use super::*;

                #[test]
                fn flush_should_keep_written_bytes_on_recover() {
                    contract::flush::flush_should_keep_written_bytes_on_recover(open());
                }
            }

            mod is_empty {
                use super::*;

                #[test]
                fn is_empty_should_be_true_only_without_bytes() {
                    contract::is_empty::is_empty_should_be_true_only_without_bytes(open());
                }
            }

            mod len {
                use super::*;

                #[test]
                fn len_should_cover_written_bytes() {
                    contract::len::len_should_cover_written_bytes(open());
                }
            }

            mod name {
                use super::*;

                #[test]
                fn name_should_return_name() {
                    contract::name::name_should_return_name(open(), NAME);
                }
            }

            mod new {
                use super::*;

                #[test]
                fn new_should_create_empty_storage_named_name() {
                    contract::new::new_should_create_empty_storage_named_name::<TestSimulator>();
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_should_return_bytes_at_pos() {
                    contract::read::read_should_return_bytes_at_pos(open());
                }

                #[test]
                fn read_out_of_bounds_should_fail() {
                    contract::read::read_out_of_bounds_should_fail(open());
                }
            }

            mod rename {
                use super::*;

                #[test]
                fn rename_should_change_name_and_keep_bytes() {
                    contract::rename::rename_should_change_name_and_keep_bytes(open(), "/renamed");
                }
            }

            mod resize {
                use super::*;

                #[test]
                fn resize_should_zero_new_bytes_and_cut_off_others() {
                    contract::resize::resize_should_zero_new_bytes_and_cut_off_others(open());
                }
            }

            mod set_overlay_limit {
                use super::*;

                #[test]
                fn set_overlay_limit_should_set_limit_if_supported() {
                    contract::set_overlay_limit::set_overlay_limit_should_set_limit_if_supported(
                        open(),
                        true,
                    );
                }
            }

            mod overlay_limit {
                use super::*;

                #[test]
                fn overlay_limit_should_default_to_default_overlay_limit() {
                    contract::overlay_limit::overlay_limit_should_default_to_default_overlay_limit(
                        open(),
                    );
                }
            }

            mod set_sync_mode {
                use super::*;

                #[test]
                fn set_sync_mode_should_set_mode_if_supported() {
                    contract::set_sync_mode::set_sync_mode_should_set_mode_if_supported(
                        open(),
                        false,
                    );
                }
            }

            mod sync_mode {
                use super::*;

                #[test]
                fn sync_mode_should_default_to_none() {
                    contract::sync_mode::sync_mode_should_default_to_none(open());
                }
            }

            mod sync {
                use super::*;

                #[test]
                fn sync_should_succeed() {
                    contract::sync::sync_should_succeed(open());
                }
            }

            mod write {
                use super::*;

                #[test]
                fn write_should_overwrite_and_extend() {
                    contract::write::write_should_overwrite_and_extend(open());
                }

                #[test]
                fn write_past_end_should_zero_gap() {
                    contract::write::write_past_end_should_zero_gap(open());
                }

                #[test]
                fn write_empty_should_do_nothing() {
                    contract::write::write_empty_should_do_nothing(open());
                }
            }

            mod recover_from_wal {
                use super::*;

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_if_logged() {
                    contract::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_if_logged(
                        open(),
                        true,
                    );
                }

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_after_random_changes() {
                    // WHY: the simulator holds its bytes in memory, so there is nothing to reopen after a crash; storage_simulator_should_recover_flushed_bytes_over_random_changes covers it
                }
            }
        }
    }
}
