use super::StorageData;
use super::StorageSlice;
use super::SyncMode;
use crate::DbError;
use crate::DbErrorType;

/// 1 KiB pages unless the storage was created with another size.
pub(crate) const DEFAULT_PAGE_SHIFT: u32 = 10;
/// 32 KiB, the largest page: the offsets in a page are `u16`.
pub(crate) const MAX_PAGE_SHIFT: u32 = 15;

/// The arena capacity an overlay keeps after a transaction.
const KEPT_ARENA: usize = 1 << 20;
/// The smallest page table: room for 32 pages at half load.
const MIN_TABLE: usize = 64;
/// 2^64 / φ: its products spread consecutive and strided pages over the
/// top bits.
const FIBONACCI: u64 = 0x9e37_79b9_7f4a_7c15;

pub(crate) trait View {
    fn len(&self) -> u64;
    fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError>;
}

pub(crate) trait ViewMut: View {
    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError>;
    fn set_len(&mut self, len: u64);
}

fn out_of_bounds(op: &str, pos: u64, end: u64, len: u64) -> DbError {
    DbError::storage(
        DbErrorType::OutOfBounds,
        format!("{op} ({pos}..{end}) out of bounds ({len})"),
    )
}

/// The committed bytes of the storage.
pub(crate) struct Reader<'a, S: StorageData> {
    storage: &'a S,
}

impl<'a, S: StorageData> Reader<'a, S> {
    pub(crate) fn new(storage: &'a S) -> Self {
        Self { storage }
    }
}

impl<S: StorageData> View for Reader<'_, S> {
    fn len(&self) -> u64 {
        self.storage.len()
    }

    fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
        self.storage.read(pos, len)
    }
}

/// The pages written by a transaction, each copied from the storage on
/// its first write. It is kept by the owner of the storage and reused by
/// its transactions, so that its buffers are not reallocated each time.
pub(crate) struct Overlay {
    shift: u32,
    /// Open addressing by page with linear probing, at most half full:
    /// the slot + 1 of a page, 0 = free.
    table: Vec<u32>,
    arena: Vec<u8>,
    /// The page of each slot and its position in the table.
    pages: Vec<(u64, usize)>,
    poisoned: bool,
}

impl Overlay {
    /// The overlay of pages of `1 << shift` bytes.
    pub(crate) fn new(shift: u32) -> Self {
        Self {
            shift,
            table: vec![0; MIN_TABLE],
            arena: Vec::new(),
            pages: Vec::new(),
            poisoned: false,
        }
    }

    pub(crate) fn page_size(&self) -> u64 {
        1 << self.shift
    }

    pub(crate) fn shift(&self) -> u32 {
        self.shift
    }

    /// The storage failed to restore after a transaction and may hold
    /// some of its writes until the next [`Writer::new`] restores it.
    pub(crate) fn poisoned(&self) -> bool {
        self.poisoned
    }

    fn page(&self, page: u64) -> Option<&[u8]> {
        self.slot(page).map(|slot| self.bytes(slot))
    }

    fn bytes(&self, slot: usize) -> &[u8] {
        let size = self.page_size() as usize;
        &self.arena[slot * size..][..size]
    }

    fn slot(&self, page: u64) -> Option<usize> {
        if self.pages.is_empty() {
            return None;
        }

        self.find(page).ok()
    }

    /// The slot of the `page`, or the free position it would take.
    fn find(&self, page: u64) -> Result<usize, usize> {
        let mask = self.table.len() - 1;
        let mut pos =
            (page.wrapping_mul(FIBONACCI) >> (self.table.len().leading_zeros() + 1)) as usize;

        loop {
            match self.table[pos] {
                0 => return Err(pos),
                entry if self.pages[entry as usize - 1].0 == page => {
                    return Ok(entry as usize - 1);
                }
                _ => pos = (pos + 1) & mask,
            }
        }
    }

    /// Adds the `page` that is not in the table as the next slot.
    fn insert(&mut self, page: u64) -> usize {
        if self.table.len() < 2 * (self.pages.len() + 1) {
            self.rehash(2 * self.table.len());
        }

        let slot = self.pages.len();
        let pos = self.find(page).unwrap_err();
        self.table[pos] = slot as u32 + 1;
        self.pages.push((page, pos));
        slot
    }

    fn rehash(&mut self, len: usize) {
        self.table = vec![0; len];

        for slot in 0..self.pages.len() {
            let pos = self.find(self.pages[slot].0).unwrap_err();
            self.table[pos] = slot as u32 + 1;
            self.pages[slot].1 = pos;
        }
    }

    fn size(&self) -> u64 {
        self.arena.len() as u64
    }

    fn reset(&mut self) {
        for (_, pos) in self.pages.drain(..) {
            self.table[pos] = 0;
        }

        self.arena.clear();
    }

    pub(crate) fn recycle(&mut self) {
        self.reset();
        self.shrink();
    }

    /// Keeps the capacity for [`KEPT_ARENA`] bytes of pages after a reset.
    fn shrink(&mut self) {
        let kept = ((2 * KEPT_ARENA) >> self.shift).max(MIN_TABLE);

        if kept < self.table.len() {
            self.table = vec![0; kept];
        }

        self.arena.shrink_to(KEPT_ARENA);
        self.pages.shrink_to(kept / 2);
    }

    /// Zeroes the bytes from `len` to `end`, looking up either the pages
    /// cut off or the pages held, whichever are fewer.
    fn cut(&mut self, len: u64, end: u64) {
        let first = len >> self.shift;
        let cut = end.div_ceil(self.page_size()) - first;

        if cut <= self.pages.len() as u64 {
            for page in first..first + cut {
                if let Some(slot) = self.slot(page) {
                    self.zero(slot, page, len);
                }
            }
        } else {
            for slot in 0..self.pages.len() {
                let page = self.pages[slot].0;

                if first <= page {
                    self.zero(slot, page, len);
                }
            }
        }
    }

    fn zero(&mut self, slot: usize, page: u64, len: u64) {
        let size = self.page_size() as usize;
        let from = len.saturating_sub(page << self.shift) as usize;
        self.arena[slot * size..][from..size].fill(0);
    }
}

/// Holds the writes of a transaction in pages over the committed bytes
/// of the storage so that they reach the storage only on commit, or
/// when they grow past the overlay limit of the storage (spill). The
/// storage is restored from its WAL if the transaction does not commit
/// after it was written to.
pub(crate) struct Writer<'a, S: StorageData> {
    base: Base<'a, S>,
    overlay: &'a mut Overlay,
    len: u64,
    /// The shortest length of the transaction: the storage bytes past
    /// it are no longer read.
    low: u64,
    limit: u64,
    touched: bool,
}

enum Base<'a, S: StorageData> {
    Storage(&'a mut S),
    Snapshot(&'a dyn View),
}

impl<S: StorageData> Base<'_, S> {
    fn len(&self) -> u64 {
        match self {
            Self::Storage(storage) => storage.len(),
            Self::Snapshot(snapshot) => snapshot.len(),
        }
    }

    fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
        match self {
            Self::Storage(storage) => storage.read(pos, len),
            Self::Snapshot(snapshot) => snapshot.read(pos, len),
        }
    }

    fn check_len(&self, len: u64) -> Result<(), DbError> {
        match self {
            Self::Storage(storage) => storage.check_len(len),
            Self::Snapshot(_) => Ok(()),
        }
    }

    fn storage(&mut self) -> &mut S {
        match self {
            Self::Storage(storage) => storage,
            Self::Snapshot(_) => unreachable!("the writes over a snapshot are adopted"),
        }
    }
}

impl<'a, S: StorageData> Writer<'a, S> {
    pub(crate) fn new(storage: &'a mut S, overlay: &'a mut Overlay) -> Result<Self, DbError> {
        if overlay.poisoned {
            storage.recover_from_wal()?;
            overlay.poisoned = false;
        }

        let len = storage.len();
        let limit = storage.overlay_limit();

        Ok(Self {
            base: Base::Storage(storage),
            overlay,
            len,
            low: len,
            limit,
            touched: false,
        })
    }

    pub(crate) fn over(snapshot: &'a dyn View, overlay: &'a mut Overlay) -> Self {
        let len = snapshot.len();

        Self {
            base: Base::Snapshot(snapshot),
            overlay,
            len,
            low: len,
            limit: u64::MAX,
            touched: false,
        }
    }

    pub(crate) fn written(&self) -> (u64, SyncMode) {
        let pages = self.overlay.pages.len() as u64 * self.overlay.page_size();
        let sync = match &self.base {
            Base::Storage(storage) => storage.sync_mode(),
            Base::Snapshot(_) => SyncMode::None,
        };
        (pages + self.len.saturating_sub(self.base.len()), sync)
    }

    pub(crate) fn lens(&self) -> (u64, u64) {
        (self.len, self.low)
    }

    pub(crate) fn adopt(&mut self, overlay: &mut Overlay, (len, low): (u64, u64)) {
        std::mem::swap(&mut self.overlay.table, &mut overlay.table);
        std::mem::swap(&mut self.overlay.arena, &mut overlay.arena);
        std::mem::swap(&mut self.overlay.pages, &mut overlay.pages);
        self.len = len;
        self.low = low;
    }

    pub(crate) fn commit(mut self) -> Result<(), DbError> {
        let result = self.apply().and_then(|()| self.base.storage().flush());

        match result {
            Ok(()) => {
                self.touched = false;
                Ok(())
            }
            Err(error) => Err(match self.restore() {
                Ok(()) => error,
                Err(restore) => DbError::storage(
                    DbErrorType::Poisoned,
                    "Storage was not restored after a failed transaction",
                )
                .caused_by(restore.caused_by(error)),
            }),
        }
    }

    pub(crate) fn rollback(mut self) -> Result<(), DbError> {
        self.restore()
    }

    fn apply(&mut self) -> Result<(), DbError> {
        let base_end = self.base_end();
        let storage = self.base.storage();

        if base_end < self.len {
            storage.check_len(self.len)?;
        }

        self.touched = true;

        for len in [base_end, self.len] {
            if storage.len() != len {
                storage.resize(len)?;
            }
        }

        let mut slots: Vec<(u64, usize)> = self
            .overlay
            .pages
            .iter()
            .enumerate()
            .map(|(slot, (page, _))| (*page, slot))
            .collect();
        slots.sort_unstable();

        for (page, slot) in slots {
            let start = page << self.overlay.shift;

            if start < self.len {
                let len = self.overlay.page_size().min(self.len - start) as usize;
                storage.write(start, &self.overlay.bytes(slot)[..len])?;
            }
        }

        self.overlay.reset();
        self.low = self.len;
        Ok(())
    }

    fn base_end(&self) -> u64 {
        self.low.min(self.base.len())
    }

    fn lift(&mut self, page: u64) -> Result<usize, DbError> {
        if let Some(slot) = self.overlay.slot(page) {
            return Ok(slot);
        }

        let size = self.overlay.page_size();
        let start = page << self.overlay.shift;
        let from_base = self.base_end().saturating_sub(start).min(size);
        let arena_len = self.overlay.arena.len();

        if from_base != 0 {
            let bytes = self.base.read(start, from_base)?;
            self.overlay.arena.extend_from_slice(&bytes);
        }

        self.overlay.arena.resize(arena_len + size as usize, 0);
        Ok(self.overlay.insert(page))
    }

    fn restore(&mut self) -> Result<(), DbError> {
        if self.touched {
            self.touched = false;
            self.overlay.poisoned = true;
            self.base.storage().recover_from_wal()?;
            self.overlay.poisoned = false;
        }

        Ok(())
    }
}

impl<S: StorageData> View for Writer<'_, S> {
    fn len(&self) -> u64 {
        self.len
    }

    fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
        let end = pos.saturating_add(len);

        if self.len < end {
            return Err(out_of_bounds("Read", pos, end, self.len));
        }

        if len == 0 {
            return Ok(StorageSlice::Borrowed(&[]));
        }

        let shift = self.overlay.shift;
        let first = pos >> shift;
        let last = (end - 1) >> shift;

        if first == last
            && let Some(page) = self.overlay.page(first)
        {
            let offset = (pos - (first << shift)) as usize;
            return Ok(StorageSlice::Borrowed(&page[offset..][..len as usize]));
        }

        let base_end = self.base_end();
        let dirty = (first..=last).any(|page| self.overlay.slot(page).is_some());

        if !dirty && end <= base_end {
            return self.base.read(pos, len);
        }

        let mut buffer = vec![0_u8; len as usize];
        let from_base = base_end.saturating_sub(pos).min(len);

        if from_base != 0 {
            buffer[..from_base as usize].copy_from_slice(&self.base.read(pos, from_base)?);
        }

        for page in first..=last {
            if let Some(bytes) = self.overlay.page(page) {
                let start = (page << shift).max(pos);
                let stop = ((page + 1) << shift).min(end);
                let offset = (start - (page << shift)) as usize;
                buffer[(start - pos) as usize..(stop - pos) as usize]
                    .copy_from_slice(&bytes[offset..][..(stop - start) as usize]);
            }
        }

        Ok(StorageSlice::Owned(buffer))
    }
}

impl<S: StorageData> ViewMut for Writer<'_, S> {
    fn write(&mut self, mut pos: u64, mut bytes: &[u8]) -> Result<(), DbError> {
        let end = pos.saturating_add(bytes.len() as u64);

        if self.len < end {
            return Err(out_of_bounds("Write", pos, end, self.len));
        }

        if self.base_end() < end {
            self.base.check_len(self.len)?;
        }

        let shift = self.overlay.shift;
        let size = self.overlay.page_size() as usize;

        while !bytes.is_empty() {
            let page = pos >> shift;
            let offset = (pos - (page << shift)) as usize;
            let len = (size - offset).min(bytes.len());
            let slot = self.lift(page)?;
            self.overlay.arena[slot * size + offset..][..len].copy_from_slice(&bytes[..len]);
            pos += len as u64;
            bytes = &bytes[len..];
        }

        if self.limit < self.overlay.size() {
            self.apply()?;
        }

        Ok(())
    }

    fn set_len(&mut self, len: u64) {
        if len < self.len {
            self.overlay.cut(len, self.len);
            self.low = self.low.min(len);
        }

        self.len = len;
    }
}

/// Restores the storage if the transaction wrote to it and did not
/// commit, e.g. on an error or a panic.
impl<S: StorageData> Drop for Writer<'_, S> {
    fn drop(&mut self) {
        if let Base::Storage(_) = self.base {
            let _ = self.restore();
            self.overlay.reset();
            self.overlay.shrink();
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::storage::DEFAULT_OVERLAY_LIMIT;
    use crate::storage::SyncMode;
    use crate::storage::file_storage::FileStorageImpl;
    use crate::storage::file_storage_memory_mapped::FileStorageMemoryMappedImpl;
    use crate::storage::memory_storage::MemoryStorageImpl;
    use crate::storage::storage_simulator::tests::TestSimulator;
    use agplatform::Platform;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;
    use std::sync::Arc;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    type TestFileStorage = FileStorageImpl<TestPlatform>;

    fn platform() -> Arc<TestPlatform> {
        Arc::new(test_platform())
    }

    fn bytes(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8 + 1).collect()
    }

    fn file(platform: &Arc<TestPlatform>, len: usize, limit: u64) -> TestFileStorage {
        file_at(platform, NAME, len, limit)
    }

    fn file_at(
        platform: &Arc<TestPlatform>,
        name: &str,
        len: usize,
        limit: u64,
    ) -> TestFileStorage {
        let mut storage = FileStorageImpl::with_platform(platform.clone(), name).unwrap();
        storage.write(0, &bytes(len)).unwrap();
        storage.flush().unwrap();
        storage.set_overlay_limit(limit);
        storage
    }

    fn reopened(platform: &Arc<TestPlatform>) -> Vec<u8> {
        FileStorageImpl::with_platform(platform.clone(), NAME)
            .unwrap()
            .read_all()
            .unwrap()
    }

    fn content(view: &impl View) -> Vec<u8> {
        view.read(0, view.len()).unwrap().to_vec()
    }

    fn committed<S: StorageData>(storage: &S) -> Vec<u8> {
        content(&Reader::new(storage))
    }

    /// Drops a spilled writer while the WAL cannot be read.
    fn poison(platform: &Arc<TestPlatform>, storage: &mut TestFileStorage, overlay: &mut Overlay) {
        let mut writer = Writer::new(storage, overlay).unwrap();
        writer.write(1000, &[9; 100]).unwrap();
        platform
            .fs()
            .set_io_error(FsOp::Read, WAL, ErrorKind::Other);
        drop(writer);
        platform.fs().clear_faults();
    }

    pub(crate) struct XorShift(pub(crate) u64);

    impl XorShift {
        pub(crate) fn next(&mut self, bound: u64) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0 % bound
        }
    }

    /// Runs random writes, resizes and reads against a plain buffer and
    /// commits or rolls back, returning the storage bytes expected after it.
    /// Lengths and writes scale with the page size of the `overlay`.
    fn random_transaction<S: StorageData>(
        storage: &mut S,
        overlay: &mut Overlay,
        seed: u64,
    ) -> Vec<u8> {
        let scale = overlay.page_size() >> DEFAULT_PAGE_SHIFT;
        let mut random = XorShift(seed);
        let original = committed(storage);
        let mut model = original.clone();
        let mut writer = Writer::new(storage, overlay).unwrap();

        for _ in 0..200 {
            match random.next(4) {
                0 => {
                    model.resize(random.next(8000 * scale) as usize, 0);
                    writer.set_len(model.len() as u64);
                }
                1 => {
                    let pos = random.next(model.len() as u64 + 1);
                    let len = random.next(model.len() as u64 - pos + 1);
                    let read = writer.read(pos, len).unwrap();
                    assert_eq!(*read, model[pos as usize..][..len as usize], "seed: {seed}");
                }
                _ => {
                    let pos = random.next(model.len() as u64 + 1);
                    let len = random.next((3000 * scale).min(model.len() as u64 - pos) + 1);
                    let value = vec![random.next(255) as u8 + 1; len as usize];
                    model[pos as usize..][..len as usize].copy_from_slice(&value);
                    writer.write(pos, &value).unwrap();
                }
            }
        }

        assert_eq!(content(&writer), model, "seed: {seed}");

        if random.next(2) == 0 {
            writer.commit().unwrap();
            model
        } else {
            writer.rollback().unwrap();
            original
        }
    }

    mod reader {
        use super::*;

        mod view {
            use super::*;

            mod len {
                use super::*;

                #[test]
                fn len_should_return_storage_len() {
                    let platform = platform();
                    let storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);

                    assert_eq!(Reader::new(&storage).len(), 3000);
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_should_return_storage_bytes() {
                    let platform = platform();
                    let storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);

                    let read = Reader::new(&storage).read(1000, 100).unwrap().to_vec();

                    assert_eq!(read, bytes(3000)[1000..1100]);
                }
            }
        }
    }

    mod writer {
        use super::*;

        #[test]
        fn writer_should_match_buffer_over_random_transactions() {
            for seed in 1..=20 {
                let platform = platform();
                let mut memory = MemoryStorageImpl::from_buffer_with_platform(
                    platform.clone(),
                    "/memory",
                    bytes(3000),
                );
                let mut mapped =
                    FileStorageMemoryMappedImpl::with_platform(platform.clone(), "/mapped")
                        .unwrap();
                mapped.write(0, &bytes(3000)).unwrap();
                mapped.flush().unwrap();
                let mut files: Vec<(TestFileStorage, Overlay)> = [0, 4096, DEFAULT_OVERLAY_LIMIT]
                    .into_iter()
                    .map(|limit| {
                        let storage = file_at(&platform, &format!("/file{limit}"), 3000, limit);
                        (storage, Overlay::new(DEFAULT_PAGE_SHIFT))
                    })
                    .collect();
                let mut memory_overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut mapped_overlay = Overlay::new(DEFAULT_PAGE_SHIFT);

                for round in 0..3 {
                    let seed = seed * 10 + round;
                    let expected = random_transaction(&mut memory, &mut memory_overlay, seed);
                    assert_eq!(committed(&memory), expected, "memory, seed: {seed}");
                    let expected = random_transaction(&mut mapped, &mut mapped_overlay, seed);
                    assert_eq!(committed(&mapped), expected, "mapped, seed: {seed}");

                    for (storage, overlay) in &mut files {
                        let expected = random_transaction(storage, overlay, seed);
                        assert_eq!(
                            storage.read_all().unwrap(),
                            expected,
                            "{}, seed: {seed}",
                            storage.name()
                        );
                    }
                }
            }
        }

        #[test]
        fn writer_with_largest_pages_should_match_buffer_over_random_transactions() {
            let size = 3000 << (MAX_PAGE_SHIFT - DEFAULT_PAGE_SHIFT);

            for seed in 1..=5 {
                let platform = platform();
                let mut memory = MemoryStorageImpl::from_buffer_with_platform(
                    platform.clone(),
                    "/memory",
                    bytes(size),
                );
                let mut file = file(&platform, size, 65536);
                let mut memory_overlay = Overlay::new(MAX_PAGE_SHIFT);
                let mut file_overlay = Overlay::new(MAX_PAGE_SHIFT);

                for round in 0..3 {
                    let seed = seed * 10 + round;
                    let expected = random_transaction(&mut memory, &mut memory_overlay, seed);
                    assert_eq!(committed(&memory), expected, "memory, seed: {seed}");
                    let expected = random_transaction(&mut file, &mut file_overlay, seed);
                    assert_eq!(file.read_all().unwrap(), expected, "file, seed: {seed}");
                }
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_view_storage_bytes() {
                let platform = platform();
                let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);

                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let writer = Writer::new(&mut storage, &mut overlay).unwrap();

                assert_eq!(writer.len(), 3000);
                assert_eq!(content(&writer), bytes(3000));
            }

            #[test]
            fn new_poisoned_should_restore_storage() {
                let platform = platform();
                let mut storage = file(&platform, 3000, 0);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                poison(&platform, &mut storage, &mut overlay);

                let writer = Writer::new(&mut storage, &mut overlay).unwrap();

                assert_eq!(content(&writer), bytes(3000));
                drop(writer);
                assert!(!overlay.poisoned());
                assert_eq!(storage.read_all().unwrap(), bytes(3000));
            }

            #[test]
            fn new_poisoned_restore_failure_should_fail() {
                let platform = platform();
                let mut storage = file(&platform, 3000, 0);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                poison(&platform, &mut storage, &mut overlay);
                platform
                    .fs()
                    .set_io_error(FsOp::Read, WAL, ErrorKind::Other);

                assert!(Writer::new(&mut storage, &mut overlay).is_err());
                assert!(overlay.poisoned());
            }
        }

        mod commit {
            use super::*;

            #[test]
            fn commit_should_write_pages_to_storage() {
                let platform = platform();
                let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.set_len(4000);
                writer.write(1000, &[9; 100]).unwrap();
                writer.write(3900, &[8; 100]).unwrap();

                writer.commit().unwrap();

                let mut expected = bytes(3000);
                expected.resize(4000, 0);
                expected[1000..1100].fill(9);
                expected[3900..].fill(8);
                assert_eq!(storage.read_all().unwrap(), expected);
                drop(storage);
                assert_eq!(reopened(&platform), expected);
            }

            #[test]
            fn commit_shrunk_should_drop_pages_past_len() {
                let platform = platform();
                let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1100, &[9]).unwrap();
                writer.write(2100, &[9]).unwrap();
                writer.set_len(1500);

                writer.commit().unwrap();

                let mut expected = bytes(1500);
                expected[1100] = 9;
                assert_eq!(storage.read_all().unwrap(), expected);
            }

            #[test]
            fn commit_shrunk_and_grown_should_zero_dropped_bytes() {
                let platform = platform();
                let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.set_len(100);
                writer.set_len(2500);
                writer.write(2400, &[5]).unwrap();

                writer.commit().unwrap();

                let mut expected = bytes(100);
                expected.resize(2500, 0);
                expected[2400] = 5;
                assert_eq!(storage.read_all().unwrap(), expected);
            }

            #[test]
            fn commit_spilled_should_keep_writes() {
                let platform = platform();
                let mut storage = file(&platform, 3000, 0);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9; 100]).unwrap();

                writer.commit().unwrap();

                let mut expected = bytes(3000);
                expected[1000..1100].fill(9);
                assert_eq!(storage.read_all().unwrap(), expected);
                drop(storage);
                assert_eq!(reopened(&platform), expected);
            }

            #[test]
            fn commit_failure_should_restore_storage() {
                for (limit, mode, op) in [0, 2048, DEFAULT_OVERLAY_LIMIT]
                    .into_iter()
                    .flat_map(|limit| [SyncMode::None, SyncMode::Commit].map(|mode| (limit, mode)))
                    .flat_map(|(limit, mode)| {
                        [FsOp::Read, FsOp::Write, FsOp::SetLen, FsOp::SyncData]
                            .map(|op| (limit, mode, op))
                    })
                {
                    for nth in 0.. {
                        let platform = platform();
                        let mut storage = file(&platform, 3000, limit);
                        storage.set_sync_mode(mode);
                        let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                        let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                        writer.write(100, &[9; 2000]).unwrap();
                        writer.write(0, &[7]).unwrap();
                        writer.set_len(5000);
                        writer.write(4500, &[8]).unwrap();
                        writer.set_len(1500);
                        writer.set_len(4000);
                        platform.fs().set_io_errors(
                            op,
                            "/",
                            (0..nth).map(|_| None).chain([Some(ErrorKind::Other), None]),
                        );

                        let result = writer.commit();

                        platform.fs().clear_faults();
                        let expected = if result.is_ok() {
                            let mut expected = bytes(1500);
                            expected[100..].fill(9);
                            expected[0] = 7;
                            expected.resize(4000, 0);
                            expected
                        } else {
                            bytes(3000)
                        };
                        let case =
                            format!("limit: {limit}, mode: {mode:?}, op: {op:?}, nth: {nth}");
                        assert_eq!(committed(&storage), expected, "{case}");
                        Writer::new(&mut storage, &mut overlay)
                            .unwrap()
                            .commit()
                            .unwrap();
                        drop(storage);
                        assert_eq!(reopened(&platform), expected, "{case}");

                        if result.is_ok() {
                            break;
                        }
                    }
                }
            }

            #[test]
            fn commit_restore_failure_should_poison_overlay() {
                let platform = platform();
                let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Write, NAME, ErrorKind::Other);

                assert!(writer.commit().is_err());
                assert!(overlay.poisoned());
            }
        }

        mod rollback {
            use super::*;

            #[test]
            fn rollback_should_discard_writes() {
                let platform = platform();
                let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9; 100]).unwrap();
                writer.set_len(5000);

                writer.rollback().unwrap();

                assert_eq!(storage.read_all().unwrap(), bytes(3000));
            }

            #[test]
            fn rollback_spilled_should_restore_storage() {
                let platform = platform();
                let mut storage = file(&platform, 3000, 0);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9; 100]).unwrap();
                writer.set_len(5000);
                writer.write(4500, &[9]).unwrap();

                writer.rollback().unwrap();

                assert_eq!(storage.read_all().unwrap(), bytes(3000));
                drop(storage);
                assert_eq!(reopened(&platform), bytes(3000));
            }

            #[test]
            fn rollback_spilled_failure_should_fail() {
                let platform = platform();
                let mut storage = file(&platform, 3000, 0);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Read, WAL, ErrorKind::Other);

                assert!(writer.rollback().is_err());
            }
        }

        mod view {
            use super::*;

            mod len {
                use super::*;

                #[test]
                fn len_should_return_transaction_len() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();

                    writer.set_len(5000);

                    assert_eq!(writer.len(), 5000);
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_dirty_page_should_borrow_overlay() {
                    let mut storage = MemoryStorageImpl::from_buffer_with_platform(
                        platform(),
                        "/memory",
                        bytes(3000),
                    );
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.write(1100, &[1, 2, 3]).unwrap();

                    let read = writer.read(1099, 5).unwrap();

                    assert!(matches!(read, StorageSlice::Borrowed(_)));
                    assert_eq!(*read, [bytes(3000)[1099], 1, 2, 3, bytes(3000)[1103]]);
                }

                #[test]
                fn read_clean_pages_should_borrow_storage() {
                    let mut storage = MemoryStorageImpl::from_buffer_with_platform(
                        platform(),
                        "/memory",
                        bytes(3000),
                    );
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.write(2100, &[1]).unwrap();

                    let read = writer.read(0, 2048).unwrap();

                    assert!(matches!(read, StorageSlice::Borrowed(_)));
                    assert_eq!(*read, bytes(3000)[..2048]);
                }

                #[test]
                fn read_across_dirty_page_should_patch_storage_bytes() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.write(1500, &[9; 10]).unwrap();

                    let read = writer.read(500, 2000).unwrap().to_vec();

                    let mut expected = bytes(3000);
                    expected[1500..1510].fill(9);
                    assert_eq!(read, expected[500..2500]);
                }

                #[test]
                fn read_past_storage_len_should_return_zeros() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.set_len(5000);

                    let read = writer.read(2900, 200).unwrap().to_vec();

                    let mut expected = bytes(3000)[2900..].to_vec();
                    expected.resize(200, 0);
                    assert_eq!(read, expected);
                }

                #[test]
                fn read_after_shrink_should_return_zeros_past_shortest_len() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.set_len(1000);
                    writer.set_len(3000);

                    let read = writer.read(900, 200).unwrap().to_vec();

                    let mut expected = bytes(3000)[900..1000].to_vec();
                    expected.resize(200, 0);
                    assert_eq!(read, expected);
                }

                #[test]
                fn read_empty_should_return_nothing() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let writer = Writer::new(&mut storage, &mut overlay).unwrap();

                    assert!(writer.read(3000, 0).unwrap().is_empty());
                }

                #[test]
                fn read_past_len_should_fail() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let writer = Writer::new(&mut storage, &mut overlay).unwrap();

                    let error = writer.read(2990, 20).unwrap_err();

                    assert_eq!(error.description, "Read (2990..3010) out of bounds (3000)");
                }

                #[test]
                fn read_storage_failure_should_fail() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.write(1500, &[9]).unwrap();
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    assert!(writer.read(0, 2000).is_err());
                }
            }
        }

        mod view_mut {
            use super::*;

            mod write {
                use super::*;

                #[test]
                fn write_should_span_pages() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();

                    writer.write(900, &[7; 2100]).unwrap();

                    let mut expected = bytes(3000);
                    expected[900..].fill(7);
                    assert_eq!(content(&writer), expected);
                    let pages: Vec<u64> = writer.overlay.pages.iter().map(|page| page.0).collect();
                    assert_eq!(pages, [0, 1, 2]);
                }

                #[test]
                fn write_past_limit_should_spill_to_storage() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, 2048);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.write(0, &[7]).unwrap();
                    writer.write(1024, &[7]).unwrap();

                    writer.write(2048, &[7]).unwrap();

                    let mut expected = bytes(3000);
                    expected[0] = 7;
                    expected[1024] = 7;
                    expected[2048] = 7;
                    assert!(writer.touched);
                    assert_eq!(writer.overlay.size(), 0);
                    assert_eq!(content(&writer), expected);
                }

                #[test]
                fn write_past_u32_pages_should_hold_only_written_pages() {
                    let mut storage = TestSimulator::new(NAME).unwrap();
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    let last = (4 << 40) - 1;
                    writer.set_len(last + 1);

                    writer.write(last, &[7]).unwrap();
                    writer.write(0, &[8]).unwrap();
                    writer.commit().unwrap();

                    assert_eq!(overlay.table.len(), MIN_TABLE);
                    assert_eq!(storage.len(), last + 1);
                    assert_eq!(storage.read(last, 1).unwrap()[..], [7]);
                    assert_eq!(storage.read(0, 1).unwrap()[..], [8]);
                }

                #[test]
                fn write_past_len_should_fail() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();

                    let error = writer.write(2999, &[1, 2]).unwrap_err();

                    assert_eq!(error.description, "Write (2999..3001) out of bounds (3000)");
                }

                #[test]
                fn write_page_read_failure_should_fail() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    assert!(writer.write(0, &[1]).is_err());
                }

                #[test]
                fn write_spill_failure_should_fail() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, 0);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    platform
                        .fs()
                        .set_io_errors(FsOp::Read, NAME, [None, Some(ErrorKind::Other)]);

                    assert!(writer.write(0, &[1]).is_err());
                }
            }

            mod set_len {
                use super::*;

                #[test]
                fn set_len_should_zero_new_bytes() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();

                    writer.set_len(5000);

                    let mut expected = bytes(3000);
                    expected.resize(5000, 0);
                    assert_eq!(content(&writer), expected);
                }

                #[test]
                fn set_len_shorter_should_zero_written_bytes_past_len() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.write(1000, &[9; 100]).unwrap();
                    writer.write(2500, &[9]).unwrap();

                    writer.set_len(1050);
                    writer.set_len(3000);

                    let mut expected = bytes(1000);
                    expected.resize(1050, 9);
                    expected.resize(3000, 0);
                    assert_eq!(content(&writer), expected);
                }

                #[test]
                fn set_len_shorter_than_written_pages_should_zero_them_past_len() {
                    let platform = platform();
                    let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                    let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                    let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                    writer.write(0, &[9; 100]).unwrap();

                    writer.set_len(50);
                    writer.set_len(3000);

                    let mut expected = vec![9; 50];
                    expected.resize(3000, 0);
                    assert_eq!(content(&writer), expected);
                }
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_should_leave_storage_unchanged() {
                let platform = platform();
                let mut storage = file(&platform, 3000, DEFAULT_OVERLAY_LIMIT);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9; 100]).unwrap();

                drop(writer);

                assert_eq!(storage.read_all().unwrap(), bytes(3000));
            }

            #[test]
            fn drop_spilled_should_restore_storage() {
                let platform = platform();
                let mut storage = file(&platform, 3000, 0);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9; 100]).unwrap();
                writer.set_len(1500);

                drop(writer);

                assert_eq!(storage.read_all().unwrap(), bytes(3000));
            }

            #[test]
            fn drop_spilled_restore_failure_should_poison_overlay() {
                let platform = platform();
                let mut storage = file(&platform, 3000, 0);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(1000, &[9]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Read, WAL, ErrorKind::Other);

                drop(writer);

                assert!(overlay.poisoned());
            }

            #[test]
            fn drop_should_reset_overlay() {
                let platform = platform();
                let mut storage = file(&platform, 3_000_000, DEFAULT_OVERLAY_LIMIT);
                let mut overlay = Overlay::new(DEFAULT_PAGE_SHIFT);
                let mut writer = Writer::new(&mut storage, &mut overlay).unwrap();
                writer.write(0, &[9; 2_000_000]).unwrap();
                assert_eq!(writer.overlay.table.len(), 4096);

                drop(writer);

                assert!(overlay.pages.is_empty());
                assert_eq!(overlay.table.len(), (2 * KEPT_ARENA) >> DEFAULT_PAGE_SHIFT);
                assert!(overlay.table.iter().all(|slot| *slot == 0));
                assert!(overlay.arena.is_empty());
                assert!(overlay.arena.capacity() <= KEPT_ARENA);
            }
        }
    }
}
