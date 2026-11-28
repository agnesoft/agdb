use super::page_cache::CACHE_PAGE_SIZE;
use super::page_cache::PageCache;
use crate::DbError;
use agplatform::Fs;
use agplatform::Platform;
use agplatform::ReadFile;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub(crate) const MAX_CACHED_READ_PAGES: u64 = 16;

const MAX_SPARE_BYTES: usize = 1 << 20;

type FileHandle<P> = <<P as Platform>::Fs as Fs>::WriteHandle;

pub(crate) struct FileShared<P: Platform> {
    pub(crate) file: FileHandle<P>,
    // WHY: set by the first snapshot, before the first write transaction, so that no
    // write goes unrecorded.
    recording: AtomicBool,
    caching: AtomicBool,
    newest: AtomicU64,
    state: Mutex<FileState>,
}

struct FileState {
    cache: Option<PageCache>,
    // WHY: odd while the writer writes the file and changes with every write, so a reader
    // does not cache a page that a write overlapped.
    seq: u64,
    versions: VecDeque<(u64, Records)>,
    spare: Option<Records>,
    pins: BTreeMap<u64, usize>,
    committed: u64,
}

#[derive(Default)]
pub(crate) struct Records {
    ranges: BTreeMap<u64, (u64, usize)>,
    bytes: Vec<u8>,
}

impl Records {
    pub(crate) fn add(&mut self, pos: u64, old: &[u8]) {
        let end = pos + old.len() as u64;

        if !self
            .ranges
            .range(..end)
            .next_back()
            .is_some_and(|(_, (stop, _))| pos < *stop)
        {
            self.insert(pos, old);
            return;
        }

        let mut gaps = Vec::new();
        let mut at = pos;

        for (start, stop) in self.overlapping(pos, end) {
            if at < start {
                gaps.push((at, start));
            }

            at = at.max(stop);
        }

        if at < end {
            gaps.push((at, end));
        }

        for (from, to) in gaps {
            self.insert(from, &old[(from - pos) as usize..(to - pos) as usize]);
        }
    }

    fn apply(&self, pos: u64, buffer: &mut [u8]) {
        let end = pos + buffer.len() as u64;

        for (start, stop) in self.overlapping(pos, end) {
            let offset = self.ranges[&start].1;
            let (from, to) = (pos.max(start), end.min(stop));
            let at = offset + (from - start) as usize;
            buffer[(from - pos) as usize..(to - pos) as usize]
                .copy_from_slice(&self.bytes[at..at + (to - from) as usize]);
        }
    }

    fn insert(&mut self, pos: u64, old: &[u8]) {
        self.ranges
            .insert(pos, (pos + old.len() as u64, self.bytes.len()));
        self.bytes.extend_from_slice(old);
    }

    fn overlapping(&self, pos: u64, end: u64) -> impl Iterator<Item = (u64, u64)> + '_ {
        let first = self
            .ranges
            .range(..pos)
            .next_back()
            .filter(|(_, (stop, _))| pos < *stop);

        first
            .into_iter()
            .chain(self.ranges.range(pos..end))
            .map(|(&start, &(stop, _))| (start, stop))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    fn merge(&mut self, records: &Records) {
        for (&start, &(stop, offset)) in &records.ranges {
            self.add(
                start,
                &records.bytes[offset..offset + (stop - start) as usize],
            );
        }
    }

    fn clear(&mut self) {
        self.ranges.clear();
        self.bytes.clear();
    }
}

impl FileState {
    fn prune(&mut self) -> u64 {
        let oldest = self
            .pins
            .keys()
            .next()
            .map_or(self.committed, |&oldest| oldest.min(self.committed));

        while self
            .versions
            .front()
            .is_some_and(|(version, _)| *version <= oldest)
        {
            let (_, mut records) = self.versions.pop_front().expect("front exists");

            if records.bytes.capacity() <= MAX_SPARE_BYTES {
                records.clear();
                self.spare = Some(records);
            }
        }

        self.versions.back().map_or(0, |(version, _)| *version)
    }
}

fn cacheable(pos: u64, end: u64) -> bool {
    pos < end && (end - 1) / CACHE_PAGE_SIZE - pos / CACHE_PAGE_SIZE < MAX_CACHED_READ_PAGES
}

impl<P: Platform> FileShared<P> {
    pub(crate) fn new(file: FileHandle<P>) -> Self {
        Self {
            file,
            recording: AtomicBool::new(false),
            caching: AtomicBool::new(false),
            newest: AtomicU64::new(0),
            state: Mutex::new(FileState {
                cache: None,
                seq: 0,
                versions: VecDeque::new(),
                spare: None,
                pins: BTreeMap::new(),
                committed: 0,
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, FileState> {
        self.state.lock().unwrap_or_else(|poisoned| {
            self.state.clear_poison();
            let mut state = poisoned.into_inner();

            if let Some(cache) = &mut state.cache {
                cache.clear();
            }

            state
        })
    }

    #[cfg(test)]
    pub(crate) fn has_cache(&self) -> bool {
        self.lock().cache.is_some()
    }

    #[cfg(test)]
    pub(crate) fn poison(&self) {
        let _ = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let _guard = self.state.lock().unwrap();
                    panic!("poison");
                })
                .join()
        });
    }

    #[cfg(test)]
    pub(crate) fn versions(&self) -> Vec<u64> {
        self.lock()
            .versions
            .iter()
            .map(|(version, _)| *version)
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn is_poisoned(&self) -> bool {
        self.state.is_poisoned()
    }

    pub(crate) fn set_cache(&self, cache: Option<PageCache>) {
        let mut state = self.lock();
        self.caching.store(cache.is_some(), Ordering::Relaxed);
        state.cache = cache;
    }

    pub(crate) fn recording(&self) -> bool {
        self.recording.load(Ordering::Relaxed)
    }

    pub(crate) fn commit(&self, version: u64) {
        let mut state = self.lock();
        state.committed = version;
        self.newest.store(state.prune(), Ordering::Release);
    }

    pub(crate) fn publish(&self, version: u64, pending: &mut Records) {
        let mut state = self.lock();
        let state = &mut *state;

        match state.versions.back_mut() {
            Some((last, records)) if *last == version => {
                records.merge(pending);
                pending.clear();
            }
            _ => {
                let spare = state.spare.take().unwrap_or_default();
                let records = std::mem::replace(pending, spare);
                state.versions.push_back((version, records));
            }
        }

        self.newest.fetch_max(version, Ordering::Release);
    }

    pub(crate) fn write<T>(&self, pos: u64, end: u64, f: impl FnOnce(&FileHandle<P>) -> T) -> T {
        // WHY: only the writer sets the cache, so it cannot appear meanwhile.
        if !self.caching.load(Ordering::Relaxed) {
            return f(&self.file);
        }

        self.lock().seq += 1;
        let result = f(&self.file);
        let mut state = self.lock();

        if let Some(cache) = &mut state.cache {
            cache.invalidate(pos, end);
        }

        state.seq += 1;
        result
    }

    pub(crate) fn read(
        &self,
        pos: u64,
        buffer: &mut [u8],
        cached: bool,
        version: Option<u64>,
    ) -> Result<(), DbError> {
        if buffer.is_empty() {
            return Ok(());
        }

        let mut state = None;

        if cached
            && self.caching.load(Ordering::Relaxed)
            && cacheable(pos, pos + buffer.len() as u64)
        {
            state = Some(self.read_cached(self.lock(), pos, buffer)?);
        } else {
            self.read_file(pos, buffer)?;
        }

        // WHY: the records are published before the file is written, so a read that sees
        // the new bytes also sees the records to put the old ones back.
        if let Some(version) = version
            && version < self.newest.load(Ordering::Acquire)
        {
            let state = state.get_or_insert_with(|| self.lock());

            for (_, records) in state
                .versions
                .iter()
                .rev()
                .take_while(|(newer, _)| version < *newer)
            {
                records.apply(pos, buffer);
            }
        }

        Ok(())
    }

    fn read_cached<'a>(
        &'a self,
        mut guard: MutexGuard<'a, FileState>,
        pos: u64,
        buffer: &mut [u8],
    ) -> Result<MutexGuard<'a, FileState>, DbError> {
        let end = pos + buffer.len() as u64;
        let mut page_buffer = Vec::new();

        for page in pos / CACHE_PAGE_SIZE..=(end - 1) / CACHE_PAGE_SIZE {
            let page_pos = page * CACHE_PAGE_SIZE;
            let from = pos.max(page_pos);
            let to = end.min(page_pos + CACHE_PAGE_SIZE);
            let target = &mut buffer[(from - pos) as usize..(to - pos) as usize];
            let in_page = (from - page_pos) as usize..(to - page_pos) as usize;

            if let Some(bytes) = guard.cache.as_mut().and_then(|cache| cache.get(page)) {
                target.copy_from_slice(&bytes[in_page]);
                continue;
            }

            let seq = guard.seq;
            drop(guard);
            page_buffer.resize(CACHE_PAGE_SIZE as usize, 0);
            self.read_file(page_pos, &mut page_buffer)?;
            target.copy_from_slice(&page_buffer[in_page]);
            guard = self.lock();

            if guard.seq == seq
                && seq.is_multiple_of(2)
                && let Some(cache) = &mut guard.cache
            {
                cache.insert(page, &page_buffer);
            }
        }

        Ok(guard)
    }

    fn read_file(&self, pos: u64, buffer: &mut [u8]) -> Result<(), DbError> {
        let mut read = 0;

        while read < buffer.len() {
            match self.file.read_at(&mut buffer[read..], pos + read as u64)? {
                0 => break,
                n => read += n,
            }
        }

        buffer[read..].fill(0);
        Ok(())
    }
}

trait Source: Send + Sync {
    fn read(&self, version: u64, pos: u64, buffer: &mut [u8]) -> Result<(), DbError>;
    fn unpin(&self, version: u64);
}

impl<P: Platform> Source for FileShared<P> {
    fn read(&self, version: u64, pos: u64, buffer: &mut [u8]) -> Result<(), DbError> {
        FileShared::read(self, pos, buffer, true, Some(version))
    }

    fn unpin(&self, version: u64) {
        let mut state = self.lock();

        if let Some(count) = state.pins.get_mut(&version) {
            *count -= 1;

            if *count == 0 {
                state.pins.remove(&version);
                self.newest.store(state.prune(), Ordering::Release);
            }
        }
    }
}

/// The bytes of a [`FileStorage`](crate::FileStorage) as of a commit.
pub struct FileSnapshot {
    source: Arc<dyn Source>,
    version: u64,
    len: u64,
}

impl FileSnapshot {
    pub(crate) fn new<P: Platform + 'static>(
        shared: &Arc<FileShared<P>>,
        version: u64,
        len: u64,
    ) -> Self {
        // WHY: only the first snapshot stores, so the others do not write a cache line
        // shared by all the readers.
        if !shared.recording() {
            shared.recording.store(true, Ordering::Relaxed);
        }

        // WHY: a pin only keeps more records, nothing to prune.
        *shared.lock().pins.entry(version).or_default() += 1;

        Self {
            source: Arc::clone(shared) as Arc<dyn Source>,
            version,
            len,
        }
    }

    pub(crate) fn len(&self) -> u64 {
        self.len
    }

    pub(crate) fn read(&self, pos: u64, len: u64) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; len as usize];
        self.source.read(self.version, pos, &mut buffer)?;
        Ok(buffer)
    }
}

impl Drop for FileSnapshot {
    fn drop(&mut self) {
        self.source.unpin(self.version);
    }
}

impl std::fmt::Debug for FileSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileSnapshot")
            .field("version", &self.version)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StorageData;
    use crate::StorageSnapshot;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    const NAME: &str = "/db";
    const PAGE: u64 = CACHE_PAGE_SIZE;

    type TestFileStorage = FileStorageImpl<TestPlatform>;

    fn open(platform: &Arc<TestPlatform>) -> TestFileStorage {
        FileStorageImpl::with_platform(platform.clone(), NAME).unwrap()
    }

    fn storage(platform: &Arc<TestPlatform>) -> TestFileStorage {
        let mut storage = open(platform);
        storage
            .write(
                0,
                &[[1; PAGE as usize], [2; PAGE as usize], [3; PAGE as usize]].concat(),
            )
            .unwrap();
        storage.flush().unwrap();
        storage
    }

    fn take_snapshot(storage: &TestFileStorage) -> FileSnapshot {
        match storage.snapshot() {
            Some(StorageSnapshot::File(snapshot)) => snapshot,
            _ => panic!("file snapshot"),
        }
    }

    fn fail_reads(platform: &Arc<TestPlatform>) {
        platform
            .fs()
            .set_io_error(FsOp::Read, NAME, ErrorKind::Other);
    }

    mod records {
        use super::*;

        mod add {
            use super::*;

            #[test]
            fn add_should_keep_first_old_value_of_bytes() {
                let mut records = Records::default();
                records.add(2, &[1, 1, 1]);

                records.add(0, &[2, 2, 2, 2, 2, 2, 2]);
                records.add(3, &[3]);

                let mut buffer = [0; 8];
                records.apply(0, &mut buffer);
                assert_eq!(buffer, [2, 2, 1, 1, 1, 2, 2, 0]);
            }

            #[test]
            fn add_inside_recorded_range_should_do_nothing() {
                let mut records = Records::default();
                records.add(0, &[1, 1, 1, 1]);

                records.add(1, &[2, 2]);

                assert_eq!(records.ranges.len(), 1);
            }
        }

        mod apply {
            use super::*;

            #[test]
            fn apply_should_put_only_overlapping_bytes() {
                let mut records = Records::default();
                records.add(0, &[1, 1, 1]);
                records.add(6, &[2, 2]);

                let mut buffer = [9; 5];
                records.apply(2, &mut buffer);

                assert_eq!(buffer, [1, 9, 9, 9, 2]);
            }
        }
    }

    mod file_snapshot {
        use super::*;

        mod read {
            use super::*;

            #[test]
            fn read_should_keep_bytes_of_its_commit_after_later_commits() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                let snapshot = take_snapshot(&storage);

                storage.write(PAGE - 1, &[7, 7]).unwrap();
                storage.flush().unwrap();
                storage.write(PAGE, &[8]).unwrap();
                storage.flush().unwrap();

                assert_eq!(snapshot.read(PAGE - 1, 3).unwrap(), vec![1, 2, 2]);
                assert_eq!(storage.read(PAGE - 1, 3).unwrap().to_vec(), vec![7, 8, 2]);
                assert_eq!(
                    take_snapshot(&storage).read(PAGE - 1, 3).unwrap(),
                    vec![7, 8, 2]
                );
            }

            #[test]
            fn read_should_not_see_spilled_writes() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                storage.set_overlay_limit(0);
                let snapshot = take_snapshot(&storage);

                storage.write(0, &[7; 10]).unwrap();

                assert_eq!(snapshot.read(0, 2).unwrap(), vec![1, 1]);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![7, 7]);
            }

            #[test]
            fn read_should_keep_bytes_after_rolled_back_transaction() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                storage.set_overlay_limit(0);
                let snapshot = take_snapshot(&storage);
                storage.write(0, &[7]).unwrap();

                storage.recover_from_wal().unwrap();
                storage.write(1, &[8]).unwrap();
                storage.flush().unwrap();

                assert_eq!(snapshot.read(0, 2).unwrap(), vec![1, 1]);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![1, 8]);
            }

            #[test]
            fn read_should_keep_bytes_cut_off_by_shrink() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                let snapshot = take_snapshot(&storage);

                storage.resize(PAGE).unwrap();
                storage.flush().unwrap();

                assert_eq!(snapshot.len(), 3 * PAGE);
                assert_eq!(snapshot.read(2 * PAGE, 2).unwrap(), vec![3, 3]);
            }

            #[test]
            fn read_cached_should_keep_bytes_of_its_commit() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                storage.set_memory_limit(Some(2 * PAGE));
                let snapshot = take_snapshot(&storage);
                snapshot.read(0, 1).unwrap();

                storage.write(0, &[7]).unwrap();
                storage.flush().unwrap();

                assert_eq!(snapshot.read(0, 2).unwrap(), vec![1, 1]);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![7, 1]);
            }

            #[test]
            fn read_cached_should_not_read_file_again() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                storage.set_memory_limit(Some(2 * PAGE));
                let snapshot = take_snapshot(&storage);
                snapshot.read(0, 1).unwrap();
                fail_reads(&platform);

                assert_eq!(snapshot.read(1, 2).unwrap(), vec![1, 1]);
                assert!(snapshot.read(PAGE, 1).is_err());
            }

            #[test]
            fn read_during_write_should_not_cache_page() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                storage.set_memory_limit(Some(2 * PAGE));
                let snapshot = take_snapshot(&storage);

                storage
                    .shared()
                    .write(0, 1, |_| snapshot.read(0, 1).unwrap());
                fail_reads(&platform);

                assert!(snapshot.read(0, 1).is_err());
            }

            #[test]
            fn read_failure_should_fail() {
                let platform = test_platform().into();
                let storage = storage(&platform);
                let snapshot = take_snapshot(&storage);
                fail_reads(&platform);

                assert!(snapshot.read(0, 1).is_err());
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_should_release_records_no_snapshot_needs() {
                let platform = test_platform().into();
                let mut storage = storage(&platform);
                let old = take_snapshot(&storage);
                storage.write(0, &[7]).unwrap();
                storage.flush().unwrap();
                let current = take_snapshot(&storage);
                storage.write(1, &[8]).unwrap();
                assert_eq!(storage.shared().versions(), vec![2]);

                drop(old);

                assert!(storage.shared().versions().is_empty());
                storage.flush().unwrap();
                assert_eq!(storage.shared().versions(), vec![3]);
                drop(current);
                assert!(storage.shared().versions().is_empty());
            }
        }
    }
}
