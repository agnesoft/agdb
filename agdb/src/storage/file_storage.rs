use super::DEFAULT_OVERLAY_LIMIT;
use super::StorageData;
use super::StorageSlice;
use super::StorageSnapshot;
use super::SyncMode;
use super::file_snapshot::FileShared;
use super::file_snapshot::FileSnapshot;
use super::file_snapshot::Records;
use super::overlay::Overlay;
use super::page_cache::PageCache;
use super::ranges::Ranges;
use super::segments::Segments;
use super::write_ahead_log::WriteAheadLog;
use super::write_ahead_log::remove_wal;
use super::write_ahead_log::wal_filename;
use crate::DbError;
use crate::DbErrorType;
use agplatform::Fs;
use agplatform::OpenWriteOptions;
use agplatform::Platform;
use agplatform::PlatformImpl;
use agplatform::ReadFile;
use agplatform::WriteFile;
use std::io::Seek;
use std::io::SeekFrom;
use std::sync::Arc;

type FileHandle<P> = <<P as Platform>::Fs as Fs>::WriteHandle;

/// Single file based storage with write ahead log (WAL) for resiliency
/// implementing [`StorageData`]. It uses the storage name as the file
/// name (`.{name}` for WAL). The files are accessed through the platform
/// `P`, see [`FileStorage`] for the production one. Multiple readers
/// share the single file handle reading it positionally.
///
/// The [`StorageData::backup()`] is implemented so that it copies the file
/// to the new name.
///
/// The changes are held in memory until [`StorageData::flush()`] unless
/// they grow past [`StorageData::overlay_limit()`].
///
/// With a [`StorageData::memory_limit()`] of at least a page the reads
/// go through a bounded LRU cache of that size, sparing the file reads
/// of the recently used pages.
///
/// Its [`StorageData::snapshot()`] reads the file while it is written,
/// see [`FileSnapshot`].
pub struct FileStorageImpl<P: Platform> {
    platform: Arc<P>,
    shared: Arc<FileShared<P>>,
    version: u64,
    pending: Records,
    filename: String,
    len: u64,
    sync_mode: SyncMode,
    wal: WriteAheadLog<P>,
    undo: Undo,
    file_len: u64,
    dirty: Ranges,
    shortest: Option<u64>,
    overlay: Overlay,
    overlay_limit: u64,
    wal_unsynced: bool,
    memory_limit: Option<u64>,
}

struct Undo {
    len: u64,
    logged: Ranges,
    truncated: bool,
}

type UndoRecords = (Option<(u64, u64)>, Option<u64>);

impl Undo {
    fn new(len: u64) -> Self {
        Self {
            len,
            logged: Ranges::default(),
            truncated: false,
        }
    }

    fn records(&self, pos: u64, end: u64, len: u64) -> UndoRecords {
        let old_end = end.min(len).min(self.len);
        let old = (pos < old_end && !self.logged.contains(pos, old_end)).then_some((pos, old_end));
        let truncate = (self.len < end && !self.truncated).then_some(self.len);
        (old, truncate)
    }

    fn logged(&mut self, undo_records: UndoRecords) {
        let (old, truncate) = undo_records;

        if let Some((start, end)) = old {
            self.logged.insert(start, end);
        }

        self.truncated |= truncate.is_some();
    }
}

/// The [`FileStorageImpl`] on the production [`PlatformImpl`].
pub type FileStorage = FileStorageImpl<PlatformImpl>;

impl<P: Platform> std::fmt::Debug for FileStorageImpl<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileStorage")
            .field("filename", &self.filename)
            .field("len", &self.len)
            .field("sync_mode", &self.sync_mode)
            .field("wal", &self.wal)
            .finish_non_exhaustive()
    }
}

impl<P: Platform> FileStorageImpl<P> {
    /// Constructs or loads the storage `name` in the file system of the
    /// `platform`, see [`StorageData::new()`].
    pub fn with_platform(platform: Arc<P>, name: &str) -> Result<Self, DbError> {
        let mut file = platform
            .fs()
            .open_write_options(name, OpenWriteOptions::new().create())?;
        let mut wal = WriteAheadLog::new(&*platform, name)?;
        let sync_mode = SyncMode::None;

        Self::apply_wal(&file, &mut wal)?;

        let len = file.seek(SeekFrom::End(0))?;

        Ok(Self {
            platform,
            shared: Arc::new(FileShared::new(file)),
            version: 1,
            pending: Records::default(),
            filename: name.to_string(),
            len,
            sync_mode,
            wal,
            undo: Undo::new(len),
            file_len: len,
            dirty: Ranges::default(),
            shortest: None,
            overlay: Overlay::default(),
            overlay_limit: DEFAULT_OVERLAY_LIMIT,
            wal_unsynced: false,
            memory_limit: None,
        })
    }

    pub(crate) fn defer_write(&mut self, pos: u64, len: u64, bytes: &Segments) {
        if len == 0 {
            return;
        }

        let end = pos + len;
        self.stage_from(pos, end, bytes);
        self.dirty.insert(pos, end);
        self.len = self.len.max(end);
    }

    pub(crate) fn defer_resize(&mut self, new_len: u64, bytes: &Segments) {
        self.stage_from(new_len.min(self.len), new_len.max(self.len), bytes);
        self.hold_len(new_len);
    }

    pub(crate) fn flush_deferred(&mut self, bytes: &Segments) -> Result<(), DbError> {
        self.write_held(bytes)?;
        self.commit_log()?;
        self.shared.commit(self.version);
        self.version += 1;
        Ok(())
    }

    pub(crate) fn is_deferring(&self) -> bool {
        self.shortest.is_some()
            || !self.dirty.is_empty()
            || !self.overlay.is_empty()
            || self.file_len != self.len
    }

    #[cfg(test)]
    pub(crate) fn shared(&self) -> &FileShared<P> {
        &self.shared
    }

    pub(crate) fn platform(&self) -> &Arc<P> {
        &self.platform
    }

    pub(crate) fn read_segments(&self) -> Result<Segments, DbError> {
        Segments::load(self.file_len, |buffer, pos| {
            self.shared.file.read_exact_at(buffer, pos)
        })
        .map_err(DbError::from)
    }

    #[cfg(test)]
    pub(crate) fn read_all(&self) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; self.file_len as usize];
        self.shared.file.read_exact_at(&mut buffer, 0)?;
        Ok(buffer)
    }

    pub(crate) fn spill(&mut self, bytes: &Segments) -> Result<(), DbError> {
        if self.overlay_limit < self.wal.staged_len() + self.overlay.size() {
            self.write_held(bytes)?;
        }

        Ok(())
    }

    fn apply_wal_record(file: &FileHandle<P>, pos: u64, value: &[u8]) -> Result<(), DbError> {
        if value.is_empty() {
            file.set_len(pos)?;
        } else {
            file.write_all_at(value, pos)?;
        }

        Ok(())
    }

    fn apply_wal(file: &FileHandle<P>, wal: &mut WriteAheadLog<P>) -> Result<(), DbError> {
        if wal.is_empty() {
            return Ok(());
        }

        wal.rollback(|pos, value| Self::apply_wal_record(file, pos, value))?;
        file.sync_data()?;
        wal.truncate()?;
        wal.sync()
    }

    fn commit_log(&mut self) -> Result<(), DbError> {
        // Reset before anything can fail: logging more than needed is safe,
        // missing a record that a cleared log would need is not.
        self.undo = Undo::new(self.len);

        if self.wal.is_empty() {
            return Ok(());
        }

        if self.sync_mode != SyncMode::None {
            self.shared.file.sync_data()?;
        }

        self.wal_unsynced = true;
        self.wal.clear()?;

        if self.sync_mode == SyncMode::Strict {
            self.wal.sync()?;
            self.wal_unsynced = false;
        }

        Ok(())
    }

    fn disk_end(&self) -> u64 {
        self.shortest
            .map_or(self.file_len, |shortest| shortest.min(self.file_len))
    }

    fn hold_len(&mut self, new_len: u64) {
        if new_len < self.len {
            self.overlay.truncate(new_len);
            self.shortest = Some(self.shortest.map_or(new_len, |len| len.min(new_len)));
        }

        self.len = new_len;
    }

    fn mark_logged(&mut self, records: UndoRecords) {
        if let (_, Some(len)) = records {
            self.wal.stage(len, &[]);
        }

        self.undo.logged(records);
    }

    fn read_into(
        shared: &FileShared<P>,
        overlay: &Overlay,
        disk_end: u64,
        pos: u64,
        buffer: &mut [u8],
        cached: bool,
    ) -> Result<(), DbError> {
        let from_file = disk_end.saturating_sub(pos).min(buffer.len() as u64) as usize;
        shared.read(pos, &mut buffer[..from_file], cached, None)?;
        buffer[from_file..].fill(0);
        overlay.patch(pos, buffer);
        Ok(())
    }

    fn stage_from(&mut self, pos: u64, end: u64, bytes: &Segments) {
        let records = self.undo.records(pos, end, self.len);

        if let (Some((start, end)), _) = records {
            self.wal.stage(start, &bytes.read(start, end - start));
        }

        self.mark_logged(records);
    }

    fn stage_read(&mut self, pos: u64, end: u64) -> Result<(), DbError> {
        let records = self.undo.records(pos, end, self.len);

        if let (Some((start, end)), _) = records {
            let (shared, overlay, disk_end) = (&*self.shared, &self.overlay, self.disk_end());

            let old = self.wal.stage_with(start, end - start, &|buffer| {
                Self::read_into(shared, overlay, disk_end, start, buffer, false)
            })?;

            if shared.recording() {
                self.pending.add(start, old);
            }
        }

        self.mark_logged(records);
        Ok(())
    }

    fn write_held(&mut self, bytes: &Segments) -> Result<(), DbError> {
        if !self.pending.is_empty() {
            self.shared.publish(self.version, &mut self.pending);
        }

        if self.wal.staged_len() != 0 {
            self.wal_unsynced = true;
            self.wal.write_staged()?;
        }

        if self.wal_unsynced && self.sync_mode != SyncMode::None && !self.wal.is_empty() {
            self.wal.sync()?;
            self.wal_unsynced = false;
        }

        if let Some(shortest) = self.shortest {
            if shortest < self.file_len {
                self.shared
                    .write(shortest, u64::MAX, |file| file.set_len(shortest))?;
                self.file_len = shortest;
            }

            self.shortest = None;
        }

        for (start, end) in self.dirty.iter() {
            if self.len <= start {
                break;
            }

            let end = end.min(self.len);
            let written = self.shared.write(start, end, |file| {
                bytes
                    .chunks(start, end)
                    .try_for_each(|(pos, chunk)| file.write_all_at(chunk, pos))
            });
            self.file_len = self.file_len.max(end);
            written?;
        }

        self.dirty.clear();

        for (start, range) in self.overlay.iter() {
            let end = start + range.len() as u64;
            let written = self
                .shared
                .write(start, end, |file| file.write_all_at(&range, start));
            self.file_len = self.file_len.max(start + range.len() as u64);
            written?;
        }

        self.overlay.clear();

        if self.file_len != self.len {
            let len = self.len;
            self.shared
                .write(len.min(self.file_len), u64::MAX, |file| file.set_len(len))?;
            self.file_len = len;
        }

        Ok(())
    }
}

impl<P: Platform + Default + 'static> StorageData for FileStorageImpl<P> {
    /// The file is copied through its handle, which keeps the borrow of a
    /// file open for writing, and the log of a file previously at `name`
    /// is removed so that it is not replayed onto the copy.
    fn backup(&self, name: &str) -> Result<(), DbError> {
        let fs = self.platform.fs();

        if fs.exists(name)? && fs.canonicalize(name)? == fs.canonicalize(&self.filename)? {
            return Err(DbError::storage(
                DbErrorType::NotAllowed,
                format!("Cannot back up '{}' onto itself", self.filename),
            ));
        }

        remove_wal(&*self.platform, name)?;
        fs.copy_from(&self.shared.file, name)?;
        Ok(())
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        self.backup(name)?;
        Self::with_platform(self.platform.clone(), name)
    }

    fn create(&self, name: &str) -> Result<Self, DbError> {
        let mut storage = Self::with_platform(self.platform.clone(), name)?;
        storage.resize(0)?;
        storage.flush()?;
        Ok(storage)
    }

    fn flush(&mut self) -> Result<(), DbError> {
        self.flush_deferred(&Segments::default())
    }

    fn set_overlay_limit(&mut self, limit: u64) {
        self.overlay_limit = limit;
    }

    fn overlay_limit(&self) -> u64 {
        self.overlay_limit
    }

    fn set_sync_mode(&mut self, mode: SyncMode) {
        self.sync_mode = mode;
    }

    fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }

    fn sync(&mut self) -> Result<(), DbError> {
        Ok(self.shared.file.sync_data()?)
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn name(&self) -> &str {
        &self.filename
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Self::with_platform(Arc::new(P::default()), name)
    }

    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError> {
        let end = pos.saturating_add(value_len);

        if self.len < end {
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!("Read ({pos}..{end}) out of bounds ({})", self.len),
            ));
        }

        if let Some(bytes) = self.overlay.get(pos, end) {
            return Ok(StorageSlice::Borrowed(bytes));
        }

        let mut buffer = vec![0_u8; value_len as usize];
        Self::read_into(
            &self.shared,
            &self.overlay,
            self.disk_end(),
            pos,
            &mut buffer,
            true,
        )?;
        Ok(StorageSlice::Owned(buffer))
    }

    /// The open files are renamed with their handles kept, so that they
    /// stay valid if the renaming fails half way. The log goes first: a
    /// file already named `new_name` is replaced only once both moved.
    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        let fs = self.platform.fs();
        fs.rename(wal_filename(&self.filename), wal_filename(new_name))?;

        if let Err(error) = fs.rename(&self.filename, new_name) {
            let _ = fs.rename(wal_filename(new_name), wal_filename(&self.filename));
            return Err(error.into());
        }

        self.filename = new_name.to_string();
        Ok(())
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        let (low, high) = (new_len.min(self.len), new_len.max(self.len));
        self.stage_read(low, high)?;
        self.hold_len(new_len);
        self.spill(&Segments::default())
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        if bytes.is_empty() {
            return Ok(());
        }

        let end = pos + bytes.len() as u64;
        self.stage_read(pos, end)?;
        self.overlay.write(pos, bytes);
        self.len = self.len.max(end);
        self.spill(&Segments::default())
    }

    fn set_memory_limit(&mut self, limit: Option<u64>) {
        let limit = limit.filter(|&limit| limit != 0);

        if limit != self.memory_limit {
            self.memory_limit = limit;
            self.shared.set_cache(limit.and_then(PageCache::new));
        }
    }

    fn memory_limit(&self) -> Option<u64> {
        self.memory_limit
    }

    fn snapshot(&self) -> Option<StorageSnapshot> {
        Some(StorageSnapshot::File(FileSnapshot::new(
            &self.shared,
            self.version - 1,
            self.len,
        )))
    }

    fn recover_from_wal(&mut self) -> Result<(), DbError> {
        self.version += 1;
        self.wal.discard_staged();
        let wal = &mut self.wal;

        // WHY: each write drops the cached pages it overwrites, only the rollback of a
        // non-empty log changes the file behind the cache.
        if !wal.is_empty() {
            self.shared
                .write(0, u64::MAX, |file| Self::apply_wal(file, wal))?;
        }

        self.dirty.clear();
        self.overlay.clear();
        self.shortest = None;
        self.len = self.shared.file.metadata()?.len;
        self.file_len = self.len;
        self.undo = Undo::new(self.len);
        Ok(())
    }
}

/// Syncs the clearing of the log so that a clean close keeps the last commit.
impl<P: Platform> Drop for FileStorageImpl<P> {
    fn drop(&mut self) {
        let wal = &mut self.wal;
        let _ = self
            .shared
            .write(0, u64::MAX, |file| Self::apply_wal(file, wal));

        if self.wal_unsynced && self.sync_mode != SyncMode::None {
            let _ = self.wal.sync();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::file_snapshot::MAX_CACHED_READ_PAGES;
    use crate::storage::page_cache::CACHE_PAGE_SIZE;

    fn seg(bytes: &[u8]) -> Segments {
        Segments::from(bytes)
    }
    use crate::storage::overlay::RUN_OVERHEAD;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    type TestFileStorage = FileStorageImpl<TestPlatform>;

    fn platform() -> Arc<TestPlatform> {
        Arc::new(test_platform())
    }

    fn open(platform: &Arc<TestPlatform>) -> TestFileStorage {
        FileStorageImpl::with_platform(platform.clone(), NAME).unwrap()
    }

    fn kill(platform: &Arc<TestPlatform>) {
        platform.fs().sync();
        platform.fs().crash();
    }

    fn read(storage: &TestFileStorage) -> Vec<u8> {
        storage.read_all().unwrap()
    }

    fn spilling(platform: &Arc<TestPlatform>) -> TestFileStorage {
        let mut storage = open(platform);
        storage.set_overlay_limit(0);
        storage
    }

    /// Leaves the log of the storage `name` holding the pre-image of an
    /// unflushed write as a process killed while writing it does.
    fn stale_storage(platform: &Arc<TestPlatform>, name: &str) {
        let mut storage = FileStorageImpl::with_platform(platform.clone(), name).unwrap();
        storage.set_overlay_limit(0);
        storage.write(0, &[9, 9, 9, 9]).unwrap();
        storage.flush().unwrap();
        storage.write(0, &[8]).unwrap();
        kill(platform);
    }

    fn written(platform: &Arc<TestPlatform>) -> TestFileStorage {
        let mut storage = open(platform);
        storage.write(0, &[1, 2, 3]).unwrap();
        storage.flush().unwrap();
        storage
    }

    const PAGE: usize = CACHE_PAGE_SIZE as usize;

    fn cached(platform: &Arc<TestPlatform>) -> TestFileStorage {
        let mut storage = open(platform);
        storage
            .write(0, &[[1; PAGE], [2; PAGE], [3; PAGE]].concat())
            .unwrap();
        storage.flush().unwrap();
        storage.set_memory_limit(Some(2 * CACHE_PAGE_SIZE));
        storage
    }

    fn fail_reads(platform: &Arc<TestPlatform>) {
        platform
            .fs()
            .set_io_error(FsOp::Read, NAME, ErrorKind::Other);
    }

    mod file_storage_impl {
        use super::*;

        #[test]
        fn file_storage_impl_should_roll_back_unflushed_writes_on_reopen() {
            for (killed, limit) in [
                (false, 0),
                (true, 0),
                (false, DEFAULT_OVERLAY_LIMIT),
                (true, DEFAULT_OVERLAY_LIMIT),
            ] {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.set_overlay_limit(limit);
                storage.write(1, &[4, 5, 6]).unwrap();
                storage.resize(2).unwrap();

                if killed {
                    kill(&platform);
                } else {
                    drop(storage);
                }

                assert_eq!(
                    read(&open(&platform)),
                    vec![1, 2, 3],
                    "killed: {killed}, limit: {limit}"
                );
            }
        }

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_name_len_sync_mode_and_wal() {
                let platform = platform();
                let mut storage = spilling(&platform);
                storage.write(0, &[1]).unwrap();

                assert_eq!(
                    format!("{storage:?}"),
                    "FileStorage { filename: \"/db\", len: 1, sync_mode: None, wal: WriteAheadLog { generation: Some(4611686018427387904), len: 48, .. }, .. }"
                );
            }
        }

        mod with_platform {
            use super::*;

            #[test]
            fn with_platform_should_load_file() {
                let platform = platform();
                platform.fs().write(NAME, [1, 2, 3]).unwrap();

                let storage = open(&platform);

                assert_eq!(storage.len(), 3);
                assert_eq!(read(&storage), vec![1, 2, 3]);
            }

            #[test]
            fn with_platform_should_roll_back_writes_logged_before_crash() {
                let platform = platform();
                let mut storage = spilling(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.write(2, &[4, 5]).unwrap();
                kill(&platform);

                let storage = open(&platform);

                assert_eq!(read(&storage), vec![1, 2, 3]);
                assert!(storage.wal.is_empty());
            }

            #[test]
            fn with_platform_should_roll_back_writes_logged_in_legacy_log() {
                // GIVEN a write of [9, 9, 9] at 2 into [1, 2, 3, 4] logged by a version without the log header
                let platform = platform();
                platform.fs().write(NAME, [1, 2, 9, 9, 9]).unwrap();
                let legacy_record = |pos: u64, value: &[u8]| {
                    [
                        &pos.to_le_bytes()[..],
                        &(value.len() as u64).to_le_bytes(),
                        value,
                    ]
                    .concat()
                };
                platform
                    .fs()
                    .write(
                        WAL,
                        [legacy_record(2, &[3, 4]), legacy_record(4, &[])].concat(),
                    )
                    .unwrap();

                // WHEN the storage is opened
                let storage = open(&platform);

                // THEN the write is rolled back and the log is emptied
                assert_eq!(read(&storage), vec![1, 2, 3, 4]);
                assert!(storage.wal.is_empty());
                drop(storage);
                assert_eq!(platform.fs().read(NAME).unwrap(), [1, 2, 3, 4]);
                assert!(platform.fs().read(WAL).unwrap().is_empty());
            }

            #[test]
            fn with_platform_failure_should_fail() {
                for (op, path) in [
                    (FsOp::Open, NAME),
                    (FsOp::Open, WAL),
                    (FsOp::Read, WAL),
                    (FsOp::Write, NAME),
                    (FsOp::SetLen, NAME),
                    (FsOp::SyncData, NAME),
                    (FsOp::SetLen, WAL),
                    (FsOp::SyncData, WAL),
                    (FsOp::Seek, NAME),
                ] {
                    let platform = platform();
                    let mut storage = spilling(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    storage.write(2, &[4, 5]).unwrap();
                    kill(&platform);
                    platform.fs().set_io_error(op, path, ErrorKind::Other);

                    assert!(
                        FileStorageImpl::with_platform(platform.clone(), NAME).is_err(),
                        "op: {op:?}, path: {path}"
                    );
                }
            }
        }

        mod defer_write {
            use super::*;

            #[test]
            fn defer_write_should_stage_old_bytes_without_writing_file() {
                // GIVEN flushed bytes the storage holds differently
                let platform = platform();
                let mut storage = written(&platform);

                // WHEN deferring a write over and past them
                storage.defer_write(2, 3, &seg(&[1, 2, 9]));

                // THEN the file is not written
                assert_eq!(storage.len(), 5);
                assert_eq!(read(&storage), vec![1, 2, 3]);
                // AND the log rolls back to the old bytes of the storage
                platform
                    .fs()
                    .set_io_error(FsOp::Write, NAME, ErrorKind::Other);
                assert!(storage.flush_deferred(&seg(&[1, 2, 4, 5, 6])).is_err());
                platform.fs().clear_faults();
                storage.recover_from_wal().unwrap();
                assert_eq!(read(&storage), vec![1, 2, 9]);
            }

            #[test]
            fn defer_write_empty_should_do_nothing() {
                let platform = platform();
                let mut storage = written(&platform);

                storage.defer_write(5, 0, &seg(&[1, 2, 3]));

                assert!(!storage.is_deferring());
                assert_eq!(storage.len(), 3);
            }
        }

        mod defer_resize {
            use super::*;

            #[test]
            fn defer_resize_should_stage_old_bytes_without_resizing_file() {
                let platform = platform();
                let mut storage = written(&platform);

                storage.defer_resize(1, &seg(&[1, 9, 9]));

                assert_eq!(storage.len(), 1);
                assert_eq!(read(&storage), vec![1, 2, 3]);
                platform
                    .fs()
                    .set_io_error(FsOp::SetLen, NAME, ErrorKind::Other);
                assert!(storage.flush_deferred(&seg(&[1])).is_err());
                platform.fs().clear_faults();
                storage.recover_from_wal().unwrap();
                assert_eq!(read(&storage), vec![1, 9, 9]);
            }

            #[test]
            fn defer_resize_to_len_should_do_nothing() {
                let platform = platform();
                let mut storage = written(&platform);

                storage.defer_resize(3, &seg(&[1, 2, 3]));

                assert!(!storage.is_deferring());
            }
        }

        mod flush_deferred {
            use super::*;

            #[test]
            fn flush_deferred_should_write_changed_ranges_of_bytes() {
                // GIVEN deferred writes and a shrink then growth past the file length
                let platform = platform();
                let mut storage = written(&platform);
                storage.defer_write(0, 1, &seg(&[1, 2, 3]));
                storage.defer_resize(1, &seg(&[7, 2, 3]));
                storage.defer_resize(4, &seg(&[7]));
                storage.defer_write(5, 1, &seg(&[7, 0, 0, 0]));

                // WHEN flushing them with bytes differing also outside the changed ranges
                storage.flush_deferred(&seg(&[7, 8, 8, 8, 8, 9])).unwrap();

                // THEN only the changed ranges are written, the rest is zeroed by the resizes
                assert_eq!(read(&storage), vec![7, 0, 0, 0, 0, 9]);
                assert!(!storage.is_deferring());
                assert!(storage.wal.is_empty());
            }

            #[test]
            fn flush_deferred_should_not_write_ranges_cut_off_by_shrink() {
                let platform = platform();
                let mut storage = written(&platform);
                storage.defer_write(2, 4, &seg(&[1, 2, 3]));
                storage.defer_resize(1, &seg(&[1, 2, 4, 4, 4, 4]));

                storage.flush_deferred(&seg(&[1])).unwrap();

                assert_eq!(read(&storage), vec![1]);
            }

            #[test]
            fn flush_deferred_failure_should_keep_writes_deferred() {
                for (op, path) in [
                    (FsOp::Write, WAL),
                    (FsOp::SetLen, NAME),
                    (FsOp::Write, NAME),
                ] {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.defer_write(5, 1, &seg(&[1, 2, 3]));
                    storage.defer_resize(1, &seg(&[1, 2, 3, 0, 0, 1]));
                    storage.defer_write(0, 1, &seg(&[1]));
                    platform.fs().set_io_error(op, path, ErrorKind::Other);

                    assert!(
                        storage.flush_deferred(&seg(&[4])).is_err(),
                        "op: {op:?}, path: {path}"
                    );

                    assert!(storage.is_deferring(), "op: {op:?}, path: {path}");
                    platform.fs().clear_faults();
                    storage.flush_deferred(&seg(&[4])).unwrap();
                    assert_eq!(read(&storage), vec![4], "op: {op:?}, path: {path}");
                }
            }
        }

        mod is_deferring {
            use super::*;

            #[test]
            fn is_deferring_should_be_true_until_flush_deferred() {
                for defer in [
                    |storage: &mut TestFileStorage| storage.defer_write(0, 1, &seg(&[1, 2, 3])),
                    |storage: &mut TestFileStorage| storage.defer_resize(5, &seg(&[1, 2, 3])),
                    |storage: &mut TestFileStorage| storage.defer_resize(1, &seg(&[1, 2, 3])),
                    |storage: &mut TestFileStorage| storage.write(0, &[1]).unwrap(),
                ] {
                    let platform = platform();
                    let mut storage = written(&platform);
                    assert!(!storage.is_deferring());

                    defer(&mut storage);

                    assert!(storage.is_deferring());
                    let bytes = vec![1; storage.len() as usize];
                    storage.flush_deferred(&seg(&bytes)).unwrap();
                    assert!(!storage.is_deferring());
                }
            }
        }

        mod platform {
            use super::*;

            #[test]
            fn platform_should_return_platform_of_storage() {
                let platform = platform();
                let storage = open(&platform);

                assert!(Arc::ptr_eq(storage.platform(), &platform));
            }
        }

        mod read_all {
            use super::*;

            #[test]
            fn read_all_should_return_bytes_of_file_without_held_changes() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(1, &[1, 2]).unwrap();
                storage.flush().unwrap();

                storage.write(0, &[3]).unwrap();

                assert_eq!(storage.read_all(), Ok(vec![0, 1, 2]));
            }

            #[test]
            fn read_all_failure_should_fail() {
                let platform = platform();
                let storage = written(&platform);
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(storage.read_all().is_err());
            }
        }

        mod read_segments {
            use super::*;

            #[test]
            fn read_segments_should_return_bytes_of_file_without_held_changes() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(1, &[1, 2]).unwrap();
                storage.flush().unwrap();

                storage.write(0, &[3]).unwrap();

                assert_eq!(storage.read_segments().unwrap().to_vec(), vec![0, 1, 2]);
            }

            #[test]
            fn read_segments_failure_should_fail() {
                let platform = platform();
                let storage = written(&platform);
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(storage.read_segments().is_err());
            }
        }

        mod spill {
            use super::*;

            #[test]
            fn spill_past_overlay_limit_should_write_held_changes_keeping_log() {
                // GIVEN held changes past the overlay limit
                let platform = platform();
                let mut storage = written(&platform);
                storage.write(1, &[4, 5]).unwrap();
                storage.resize(4).unwrap();
                storage.set_overlay_limit(0);

                // WHEN spilling them
                storage.spill(&seg(&[])).unwrap();

                // THEN they are in the file
                assert!(!storage.is_deferring());
                assert_eq!(read(&storage), vec![1, 4, 5, 0]);
                // AND the log still undoes them
                storage.recover_from_wal().unwrap();
                assert_eq!(read(&storage), vec![1, 2, 3]);
            }

            #[test]
            fn spill_within_overlay_limit_should_keep_changes_held() {
                let platform = platform();
                let mut storage = written(&platform);
                storage.write(1, &[4, 5]).unwrap();

                storage.spill(&seg(&[])).unwrap();

                assert!(storage.is_deferring());
                assert_eq!(read(&storage), vec![1, 2, 3]);
            }

            #[test]
            fn spill_should_leave_flush_nothing_more_to_log() {
                // GIVEN changes spilled in commit mode
                let platform = platform();
                let mut storage = written(&platform);
                storage.set_sync_mode(SyncMode::Commit);
                storage.set_overlay_limit(0);
                storage.write(1, &[4]).unwrap();

                // WHEN flushing with the log failing to sync
                platform
                    .fs()
                    .set_io_error(FsOp::SyncData, WAL, ErrorKind::Other);

                // THEN the flush neither writes nor syncs the log again
                assert_eq!(storage.flush(), Ok(()));
                assert_eq!(read(&storage), vec![1, 4, 3]);
            }

            #[test]
            fn spill_failure_should_keep_changes_held() {
                for (mode, op, path) in [
                    (SyncMode::None, FsOp::Write, WAL),
                    (SyncMode::Commit, FsOp::SyncData, WAL),
                    (SyncMode::None, FsOp::SetLen, NAME),
                    (SyncMode::None, FsOp::Write, NAME),
                ] {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.set_sync_mode(mode);
                    storage.resize(1).unwrap();
                    storage.write(4, &[7]).unwrap();
                    storage.set_overlay_limit(0);
                    platform.fs().set_io_error(op, path, ErrorKind::Other);

                    assert!(
                        storage.spill(&seg(&[])).is_err(),
                        "mode: {mode}, op: {op:?}, path: {path}"
                    );

                    assert!(
                        storage.is_deferring(),
                        "mode: {mode}, op: {op:?}, path: {path}"
                    );
                    platform.fs().clear_faults();
                    storage.flush().unwrap();
                    assert_eq!(
                        read(&storage),
                        vec![1, 0, 0, 0, 7],
                        "mode: {mode}, op: {op:?}, path: {path}"
                    );
                }
            }
        }

        mod storage_data {
            use super::*;

            mod backup {
                use super::*;

                #[test]
                fn backup_should_write_bytes_to_name() {
                    let platform = platform();

                    crate::storage::tests::storage_data::backup::backup_should_write_bytes_to_name(
                        open(&platform),
                        "/backup",
                        |name| platform.fs().read(name).unwrap(),
                    );
                }

                #[test]
                fn backup_should_not_replay_log_of_storage_previously_at_name() {
                    // GIVEN a storage killed with an unflushed write at the name
                    let platform = platform();
                    stale_storage(&platform, "/backup");
                    let storage = written(&platform);

                    // WHEN the storage is backed up to the name
                    storage.backup("/backup").unwrap();

                    // THEN the backup opens with the bytes of the storage
                    assert!(!platform.fs().exists("/.backup").unwrap());
                    let backup =
                        FileStorageImpl::with_platform(platform.clone(), "/backup").unwrap();
                    assert_eq!(read(&backup), vec![1, 2, 3]);
                }

                #[test]
                fn backup_onto_itself_should_fail_keeping_bytes() {
                    let platform = platform();
                    let storage = written(&platform);
                    platform.fs().create_symlink_file("/link", NAME).unwrap();

                    for name in [NAME, "/link"] {
                        assert_eq!(
                            storage.backup(name),
                            Err(DbError::storage(
                                DbErrorType::NotAllowed,
                                "Cannot back up '/db' onto itself"
                            )),
                            "name: {name}"
                        );
                        assert_eq!(read(&storage), vec![1, 2, 3], "name: {name}");
                    }
                }

                #[test]
                fn backup_failure_should_fail() {
                    for (op, path) in [
                        (FsOp::Metadata, "/backup"),
                        (FsOp::Canonicalize, "/backup"),
                        (FsOp::Canonicalize, NAME),
                        (FsOp::Metadata, "/.backup"),
                        (FsOp::Remove, "/.backup"),
                        (FsOp::Open, "/backup"),
                        (FsOp::Write, "/backup"),
                        (FsOp::Read, NAME),
                    ] {
                        let platform = platform();
                        platform.fs().write("/backup", [9]).unwrap();
                        platform.fs().write("/.backup", [9]).unwrap();
                        let storage = written(&platform);
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.backup("/backup").is_err(),
                            "op: {op:?}, path: {path}"
                        );
                    }
                }
            }

            mod copy {
                use super::*;

                #[test]
                fn copy_should_create_independent_storage_named_name() {
                    crate::storage::tests::storage_data::copy::copy_should_create_independent_storage_named_name(
                        open(&platform()),
                        "/copy",
                    );
                }

                #[test]
                fn copy_should_not_replay_log_of_storage_previously_at_name() {
                    // GIVEN a storage killed with an unflushed write at the name
                    let platform = platform();
                    stale_storage(&platform, "/copy");
                    let storage = written(&platform);

                    // WHEN the storage is copied to the name
                    let copy = storage.copy("/copy").unwrap();

                    // THEN the copy holds the bytes of the storage
                    assert_eq!(read(&copy), vec![1, 2, 3]);
                }

                #[test]
                fn copy_failure_should_fail() {
                    let platform = platform();
                    let storage = written(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::Open, "/copy", ErrorKind::Other);

                    assert!(storage.copy("/copy").is_err());
                }
            }

            mod create {
                use super::*;

                #[test]
                fn create_should_replace_name_with_empty_storage() {
                    crate::storage::tests::storage_data::create::create_should_replace_name_with_empty_storage(
                        open(&platform()),
                        "/created",
                    );
                }

                #[test]
                fn create_should_empty_file_at_name() {
                    let platform = platform();
                    let storage = written(&platform);
                    drop(storage.copy("/created").unwrap());

                    drop(storage.create("/created").unwrap());

                    assert!(platform.fs().read("/created").unwrap().is_empty());
                }

                #[test]
                fn create_failure_should_fail() {
                    for (op, path) in [
                        (FsOp::Open, "/created"),
                        (FsOp::Read, "/created"),
                        (FsOp::Write, "/.created"),
                    ] {
                        let platform = platform();
                        let storage = written(&platform);
                        drop(storage.copy("/created").unwrap());
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.create("/created").is_err(),
                            "op: {op:?}, path: {path}"
                        );
                    }
                }
            }

            mod flush {
                use super::*;

                #[test]
                fn flush_should_keep_written_bytes_on_recover() {
                    crate::storage::tests::storage_data::flush::flush_should_keep_written_bytes_on_recover(open(
                        &platform(),
                    ));
                }

                #[test]
                fn flush_should_make_writes_survive_crash_by_sync_mode() {
                    // GIVEN two flushed writes
                    for (mode, expected) in [
                        (SyncMode::None, vec![]),
                        (SyncMode::Commit, vec![1, 2, 3]),
                        (SyncMode::Strict, vec![4, 2, 3]),
                    ] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.set_sync_mode(mode);
                        storage.write(0, &[1, 2, 3]).unwrap();
                        storage.flush().unwrap();
                        storage.write(0, &[4]).unwrap();

                        storage.flush().unwrap();

                        // WHEN the system crashes
                        platform.fs().crash();

                        // THEN nothing survives without syncs, the commit mode rolls back the last write and the strict one keeps it
                        assert_eq!(read(&open(&platform)), expected, "mode: {mode}");
                    }
                }

                #[test]
                fn flush_should_sync_log_before_writing_file_unless_sync_mode_none() {
                    for (mode, flushed) in [
                        (SyncMode::None, true),
                        (SyncMode::Commit, false),
                        (SyncMode::Strict, false),
                    ] {
                        let platform = platform();
                        let mut storage = written(&platform);
                        storage.set_sync_mode(mode);
                        storage.write(0, &[4]).unwrap();
                        platform
                            .fs()
                            .set_io_error(FsOp::SyncData, WAL, ErrorKind::Other);

                        assert_eq!(storage.flush().is_ok(), flushed, "mode: {mode}");

                        let expected = if flushed { [4, 2, 3] } else { [1, 2, 3] };
                        assert_eq!(read(&storage), expected, "mode: {mode}");
                    }
                }

                #[test]
                fn flush_without_writes_should_do_nothing() {
                    for flushed in [false, true] {
                        let platform = platform();
                        let mut storage = if flushed {
                            written(&platform)
                        } else {
                            open(&platform)
                        };
                        storage.set_sync_mode(SyncMode::Strict);
                        platform
                            .fs()
                            .set_io_error(FsOp::SyncData, "", ErrorKind::Other);
                        platform
                            .fs()
                            .set_io_error(FsOp::Write, "", ErrorKind::Other);

                        assert_eq!(storage.flush(), Ok(()), "flushed: {flushed}");
                    }
                }

                #[test]
                fn flush_failure_should_fail() {
                    for (mode, op, path, spilled) in [
                        (SyncMode::None, FsOp::Write, WAL, false),
                        (SyncMode::Commit, FsOp::SyncData, WAL, false),
                        (SyncMode::Commit, FsOp::Write, NAME, false),
                        (SyncMode::Commit, FsOp::SyncData, NAME, false),
                        (SyncMode::None, FsOp::Write, WAL, true),
                        (SyncMode::Strict, FsOp::SyncData, WAL, true),
                    ] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.set_sync_mode(mode);

                        if spilled {
                            storage.set_overlay_limit(0);
                        }

                        storage.write(0, &[1]).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.flush().is_err(),
                            "mode: {mode}, op: {op:?}, path: {path}, spilled: {spilled}"
                        );
                    }
                }
            }

            mod is_empty {
                use super::*;

                #[test]
                fn is_empty_should_be_true_only_without_bytes() {
                    crate::storage::tests::storage_data::is_empty::is_empty_should_be_true_only_without_bytes(open(
                        &platform(),
                    ));
                }
            }

            mod check_len {
                use super::*;

                #[test]
                fn check_len_should_fail_past_memory_limit_if_bounded() {
                    crate::storage::tests::storage_data::check_len::check_len_should_fail_past_memory_limit_if_bounded(
                        open(&platform()), false,
                    );
                }
            }

            mod set_memory_limit {
                use super::*;

                #[test]
                fn set_memory_limit_same_should_keep_cached_pages() {
                    let platform = platform();
                    let mut storage = cached(&platform);
                    storage.read(0, 1).unwrap();
                    fail_reads(&platform);

                    storage.set_memory_limit(Some(2 * CACHE_PAGE_SIZE));

                    assert_eq!(storage.read(0, 1).unwrap().to_vec(), vec![1]);
                }

                #[test]
                fn set_memory_limit_below_page_should_not_cache() {
                    let platform = platform();
                    let mut storage = cached(&platform);

                    storage.set_memory_limit(Some(CACHE_PAGE_SIZE - 1));

                    assert!(!storage.shared.has_cache());
                    assert_eq!(storage.memory_limit(), Some(CACHE_PAGE_SIZE - 1));
                }

                #[test]
                fn set_memory_limit_should_set_limit_with_zero_as_none() {
                    crate::storage::tests::storage_data::set_memory_limit::set_memory_limit_should_set_limit_with_zero_as_none(
                        open(&platform()),
                    );
                }
            }

            mod memory_limit {
                use super::*;

                #[test]
                fn memory_limit_should_default_to_none() {
                    crate::storage::tests::storage_data::memory_limit::memory_limit_should_default_to_none(
                        open(&platform()),
                    );
                }
            }

            mod set_overlay_limit {
                use super::*;

                #[test]
                fn set_overlay_limit_should_set_limit_if_supported() {
                    crate::storage::tests::storage_data::set_overlay_limit::set_overlay_limit_should_set_limit_if_supported(
                        open(&platform()),
                        true,
                    );
                }
            }

            mod overlay_limit {
                use super::*;

                #[test]
                fn overlay_limit_should_default_to_default_overlay_limit() {
                    crate::storage::tests::storage_data::overlay_limit::overlay_limit_should_default_to_default_overlay_limit(
                        open(&platform()),
                    );
                }
            }

            mod set_sync_mode {
                use super::*;

                #[test]
                fn set_sync_mode_should_set_mode_if_supported() {
                    crate::storage::tests::storage_data::set_sync_mode::set_sync_mode_should_set_mode_if_supported(
                        open(&platform()),
                        true,
                    );
                }
            }

            mod sync_mode {
                use super::*;

                #[test]
                fn sync_mode_should_default_to_none() {
                    crate::storage::tests::storage_data::sync_mode::sync_mode_should_default_to_none(open(&platform()));
                }
            }

            mod sync {
                use super::*;

                #[test]
                fn sync_should_succeed() {
                    crate::storage::tests::storage_data::sync::sync_should_succeed(open(
                        &platform(),
                    ));
                }

                #[test]
                fn sync_should_make_bytes_survive_crash() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2]).unwrap();
                    storage.flush().unwrap();

                    storage.sync().unwrap();

                    platform.fs().crash();
                    assert_eq!(read(&open(&platform)), vec![1, 2]);
                }

                #[test]
                fn sync_failure_should_fail() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::SyncData, NAME, ErrorKind::Other);

                    assert!(storage.sync().is_err());
                }
            }

            mod len {
                use super::*;

                #[test]
                fn len_should_cover_written_bytes() {
                    crate::storage::tests::storage_data::len::len_should_cover_written_bytes(open(
                        &platform(),
                    ));
                }
            }

            mod name {
                use super::*;

                #[test]
                fn name_should_return_name() {
                    crate::storage::tests::storage_data::name::name_should_return_name(
                        open(&platform()),
                        NAME,
                    );
                }
            }

            mod new {
                use super::*;

                #[test]
                fn new_should_create_empty_storage_named_name() {
                    crate::storage::tests::storage_data::new::new_should_create_empty_storage_named_name::<TestFileStorage>();
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_cached_should_not_read_file_again() {
                    let platform = platform();
                    let storage = cached(&platform);
                    storage.read(1, 2).unwrap();
                    fail_reads(&platform);

                    let bytes = storage.read(PAGE as u64 - 2, 2).unwrap();

                    assert_eq!(bytes.to_vec(), vec![1, 1]);
                    assert!(storage.read(2 * PAGE as u64, 1).is_err());
                }

                #[test]
                fn read_cached_across_pages_should_return_bytes_of_each() {
                    let platform = platform();
                    let storage = cached(&platform);

                    let bytes = storage.read(PAGE as u64 - 1, PAGE as u64 + 2).unwrap();

                    assert_eq!(bytes.len(), PAGE + 2);
                    assert_eq!((bytes[0], bytes[1], bytes[PAGE + 1]), (1, 2, 3));
                }

                #[test]
                fn read_cached_should_evict_least_recently_used_page() {
                    let platform = platform();
                    let storage = cached(&platform);
                    storage.read(0, 1).unwrap();
                    storage.read(PAGE as u64, 1).unwrap();
                    storage.read(0, 1).unwrap();
                    storage.read(2 * PAGE as u64, 1).unwrap();
                    fail_reads(&platform);

                    assert_eq!(storage.read(0, 1).unwrap().to_vec(), vec![1]);
                    assert_eq!(storage.read(2 * PAGE as u64, 1).unwrap().to_vec(), vec![3]);
                    assert!(storage.read(PAGE as u64, 1).is_err());
                }

                #[test]
                fn read_cached_should_return_bytes_written_after_caching() {
                    let platform = platform();
                    let mut storage = cached(&platform);
                    storage.read(0, 3).unwrap();

                    storage.write(1, &[7]).unwrap();
                    storage.flush().unwrap();

                    assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 7, 1]);
                }

                #[test]
                fn read_cached_failure_should_fail_without_caching() {
                    let platform = platform();
                    let storage = cached(&platform);
                    fail_reads(&platform);

                    let failed = storage.read(0, 1);
                    platform.fs().clear_faults();

                    assert!(failed.is_err());
                    assert_eq!(storage.read(0, 1).unwrap().to_vec(), vec![1]);
                }

                #[test]
                fn read_cached_poisoned_should_clear_cache() {
                    let platform = platform();
                    let storage = cached(&platform);
                    storage.read(0, 1).unwrap();
                    storage.shared.poison();
                    fail_reads(&platform);

                    assert!(storage.read(0, 1).is_err());
                    assert!(!storage.shared.is_poisoned());
                }

                #[test]
                fn read_longer_than_cached_read_should_read_file() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    let len = (MAX_CACHED_READ_PAGES + 1) * CACHE_PAGE_SIZE;
                    storage.resize(len).unwrap();
                    storage.flush().unwrap();
                    storage.set_memory_limit(Some(len));
                    storage.read(0, len).unwrap();
                    fail_reads(&platform);

                    assert!(storage.read(0, 1).is_err());
                }

                #[test]
                fn read_should_return_bytes_at_pos() {
                    crate::storage::tests::storage_data::read::read_should_return_bytes_at_pos(
                        open(&platform()),
                    );
                }

                #[test]
                fn read_out_of_bounds_should_fail() {
                    crate::storage::tests::storage_data::read::read_out_of_bounds_should_fail(
                        open(&platform()),
                    );
                }

                #[test]
                fn read_out_of_bounds_should_report_range() {
                    let platform = platform();
                    let storage = written(&platform);

                    assert_eq!(
                        storage.read(1, 3),
                        Err(DbError::storage(
                            DbErrorType::OutOfBounds,
                            "Read (1..4) out of bounds (3)"
                        ))
                    );
                }

                #[test]
                fn read_within_held_write_should_borrow_its_bytes() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.write(1, &[4, 5]).unwrap();
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    let bytes = storage.read(1, 2).unwrap();

                    assert_eq!(bytes.to_vec(), vec![4, 5]);
                }

                #[test]
                fn read_should_combine_file_and_held_writes() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.write(1, &[9]).unwrap();
                    storage.write(4, &[8]).unwrap();

                    let bytes = storage.read(0, 5).unwrap();

                    assert!(matches!(bytes, StorageSlice::Owned(_)));
                    assert_eq!(bytes.to_vec(), vec![1, 9, 3, 0, 8]);
                }

                #[test]
                fn read_after_shrink_should_not_return_bytes_cut_off_from_file() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.resize(1).unwrap();
                    storage.resize(3).unwrap();

                    assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 0, 0]);
                }

                #[test]
                fn read_failure_should_fail() {
                    let platform = platform();
                    let storage = written(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    assert!(storage.read(0, 1).is_err());
                }
            }

            mod rename {
                use super::*;

                #[test]
                fn rename_should_change_name_and_keep_bytes() {
                    crate::storage::tests::storage_data::rename::rename_should_change_name_and_keep_bytes(
                        open(&platform()),
                        "/renamed",
                    );
                }

                #[test]
                fn rename_should_move_files_with_log() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2]).unwrap();
                    storage.flush().unwrap();
                    storage.write(0, &[3]).unwrap();

                    storage.rename("/renamed").unwrap();

                    for (path, exists) in [
                        (NAME, false),
                        (WAL, false),
                        ("/renamed", true),
                        ("/.renamed", true),
                    ] {
                        assert_eq!(platform.fs().exists(path).unwrap(), exists, "path: {path}");
                    }
                    storage.recover_from_wal().unwrap();
                    assert_eq!(read(&storage), vec![1, 2]);
                }

                #[test]
                fn rename_failure_should_keep_name_and_files() {
                    for path in [NAME, WAL] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        platform
                            .fs()
                            .set_io_error(FsOp::Rename, path, ErrorKind::Other);

                        assert!(storage.rename("/renamed").is_err(), "path: {path}");

                        assert_eq!(storage.name(), NAME, "path: {path}");
                        assert!(platform.fs().exists(NAME).unwrap(), "path: {path}");
                        assert!(!platform.fs().exists("/renamed").unwrap(), "path: {path}");
                    }
                }

                #[test]
                fn rename_failure_over_existing_file_should_keep_it() {
                    for path in [NAME, WAL] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        platform.fs().write("/existing", [7]).unwrap();
                        platform
                            .fs()
                            .set_io_error(FsOp::Rename, path, ErrorKind::Other);

                        assert!(storage.rename("/existing").is_err(), "path: {path}");

                        assert_eq!(
                            platform.fs().read("/existing").unwrap(),
                            vec![7],
                            "path: {path}"
                        );
                    }
                }
            }

            mod resize {
                use super::*;

                #[test]
                fn resize_cached_shrunk_and_grown_should_zero_dropped_bytes() {
                    let platform = platform();
                    let mut storage = cached(&platform);
                    storage.read(PAGE as u64, 2).unwrap();

                    storage.resize(PAGE as u64 + 1).unwrap();
                    storage.resize(PAGE as u64 + 2).unwrap();

                    assert_eq!(storage.read(PAGE as u64, 2).unwrap().to_vec(), vec![2, 0]);
                }

                #[test]
                fn resize_should_zero_new_bytes_and_cut_off_others() {
                    crate::storage::tests::storage_data::resize::resize_should_zero_new_bytes_and_cut_off_others(open(
                        &platform(),
                    ));
                }

                #[test]
                fn resize_should_hold_new_len_until_flush() {
                    let platform = platform();
                    let mut storage = written(&platform);

                    storage.resize(5).unwrap();

                    assert_eq!(storage.len(), 5);
                    assert_eq!(read(&storage), vec![1, 2, 3]);
                    storage.flush().unwrap();
                    assert_eq!(read(&storage), vec![1, 2, 3, 0, 0]);
                }

                #[test]
                fn resize_past_overlay_limit_should_write_file() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.set_overlay_limit(0);

                    storage.resize(1).unwrap();

                    assert_eq!(read(&storage), vec![1]);
                }

                #[test]
                fn resize_failure_should_fail() {
                    for (op, path, len) in [(FsOp::Read, NAME, 1), (FsOp::Write, WAL, 5)] {
                        let platform = platform();
                        let mut storage = spilling(&platform);
                        storage.write(0, &[1, 2, 3]).unwrap();
                        storage.flush().unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.resize(len).is_err(),
                            "op: {op:?}, path: {path}, len: {len}"
                        );
                    }
                }

                #[test]
                fn resize_failure_to_read_old_bytes_should_fail_without_resizing() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    assert!(storage.resize(1).is_err());

                    assert_eq!(storage.len(), 3);
                    assert_eq!(storage.wal.staged_len(), 0);
                }

                #[test]
                fn resize_should_log_old_bytes_once_since_flush() {
                    // GIVEN a shrink logging the bytes it cuts off
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.resize(1).unwrap();
                    let logged = storage.wal.staged_len();

                    // WHEN growing and shrinking within the logged bytes
                    storage.resize(3).unwrap();
                    storage.resize(2).unwrap();

                    // THEN nothing more is logged
                    assert_eq!(storage.wal.staged_len(), logged);
                    // AND recovering restores the flushed bytes
                    storage.recover_from_wal().unwrap();
                    assert_eq!(read(&storage), vec![1, 2, 3]);
                }

                #[test]
                fn resize_past_len_at_flush_should_log_truncation_once() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.resize(4).unwrap();
                    let logged = storage.wal.staged_len();

                    storage.resize(8).unwrap();
                    storage.resize(5).unwrap();

                    assert_eq!(storage.wal.staged_len(), logged);
                    storage.recover_from_wal().unwrap();
                    assert_eq!(read(&storage), vec![1, 2, 3]);
                }

                #[test]
                fn resize_past_memory_limit_should_fail_if_bounded() {
                    crate::storage::tests::storage_data::resize::resize_past_memory_limit_should_fail_if_bounded(
                        open(&platform()), false,
                    );
                }
            }

            mod write {
                use super::*;

                #[test]
                fn write_should_log_old_bytes_once_since_flush() {
                    // GIVEN a write logging the old byte it overwrites
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.write(0, &[4]).unwrap();
                    let logged = storage.wal.staged_len();

                    // WHEN overwriting it again
                    storage.write(0, &[5]).unwrap();

                    // THEN nothing more is logged
                    assert_eq!(storage.wal.staged_len(), logged);
                    // AND a write over the rest of the old bytes logs them
                    storage.write(0, &[6, 6, 6]).unwrap();
                    assert!(logged < storage.wal.staged_len());
                    storage.recover_from_wal().unwrap();
                    assert_eq!(read(&storage), vec![1, 2, 3]);
                }

                #[test]
                fn write_past_len_at_flush_should_log_truncation_once() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.write(3, &[4]).unwrap();
                    let logged = storage.wal.staged_len();

                    storage.write(4, &[5]).unwrap();
                    storage.write(3, &[6, 7, 8]).unwrap();

                    assert_eq!(storage.wal.staged_len(), logged);
                    storage.recover_from_wal().unwrap();
                    assert_eq!(read(&storage), vec![1, 2, 3]);
                }

                #[test]
                fn write_after_shrink_should_roll_back_on_reopen() {
                    for killed in [false, true] {
                        let platform = platform();
                        let mut storage = written(&platform);
                        storage.set_overlay_limit(0);
                        storage.resize(1).unwrap();
                        storage.write(2, &[9]).unwrap();
                        storage.write(4, &[8]).unwrap();

                        if killed {
                            kill(&platform);
                        } else {
                            drop(storage);
                        }

                        assert_eq!(read(&open(&platform)), vec![1, 2, 3], "killed: {killed}");
                    }
                }

                #[test]
                fn write_should_overwrite_and_extend() {
                    crate::storage::tests::storage_data::write::write_should_overwrite_and_extend(
                        open(&platform()),
                    );
                }

                #[test]
                fn write_past_end_should_zero_gap() {
                    crate::storage::tests::storage_data::write::write_past_end_should_zero_gap(
                        open(&platform()),
                    );
                }

                #[test]
                fn write_empty_should_do_nothing() {
                    crate::storage::tests::storage_data::write::write_empty_should_do_nothing(
                        open(&platform()),
                    );
                }

                #[test]
                fn write_should_hold_bytes_until_flush() {
                    let platform = platform();
                    let mut storage = written(&platform);

                    storage.write(1, &[4, 5, 6]).unwrap();

                    assert_eq!(storage.read(0, 4).unwrap().to_vec(), vec![1, 4, 5, 6]);
                    assert_eq!(read(&storage), vec![1, 2, 3]);
                    assert!(storage.wal.is_empty());
                    storage.flush().unwrap();
                    assert_eq!(read(&storage), vec![1, 4, 5, 6]);
                }

                #[test]
                fn write_past_overlay_limit_should_write_file_keeping_log() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    storage.set_overlay_limit(40 + RUN_OVERHEAD);
                    storage.write(0, &[4]).unwrap();
                    assert_eq!(read(&storage), vec![1, 2, 3]);

                    storage.write(2, &[5, 6]).unwrap();

                    assert_eq!(read(&storage), vec![4, 2, 5, 6]);
                    storage.recover_from_wal().unwrap();
                    assert_eq!(read(&storage), vec![1, 2, 3]);
                }

                #[test]
                fn write_failure_should_fail() {
                    for (op, path, pos) in [
                        (FsOp::Read, NAME, 0),
                        (FsOp::Write, WAL, 3),
                        (FsOp::Write, NAME, 0),
                    ] {
                        let platform = platform();
                        let mut storage = spilling(&platform);
                        storage.write(0, &[1, 2, 3]).unwrap();
                        storage.flush().unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.write(pos, &[4, 5]).is_err(),
                            "op: {op:?}, path: {path}, pos: {pos}"
                        );

                        platform.fs().clear_faults();
                        storage.recover_from_wal().unwrap();
                        assert_eq!(
                            read(&storage),
                            vec![1, 2, 3],
                            "op: {op:?}, path: {path}, pos: {pos}"
                        );
                    }
                }

                #[test]
                fn write_failure_to_read_old_bytes_should_fail_without_writing() {
                    let platform = platform();
                    let mut storage = written(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    assert!(storage.write(0, &[4, 5]).is_err());

                    assert_eq!(storage.len(), 3);
                    assert!(!storage.is_deferring());
                    assert_eq!(storage.wal.staged_len(), 0);
                }

                #[test]
                fn write_past_memory_limit_should_fail_if_bounded() {
                    crate::storage::tests::storage_data::write::write_past_memory_limit_should_fail_if_bounded(
                        open(&platform()), false,
                    );
                }
            }

            mod recover_from_wal {
                use super::*;

                #[test]
                fn recover_from_wal_should_drop_cached_unflushed_bytes() {
                    let platform = platform();
                    let mut storage = cached(&platform);
                    storage.set_overlay_limit(0);
                    storage.write(0, &[9]).unwrap();
                    storage.read(0, 1).unwrap();

                    storage.recover_from_wal().unwrap();

                    assert_eq!(storage.read(0, 1).unwrap().to_vec(), vec![1]);
                }

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_after_random_changes() {
                    let platform = platform();

                    crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_after_random_changes(
                        || open(&platform),
                        || {
                            platform.fs().clear_faults();
                            platform.fs().set_capacity(None);
                            kill(&platform);
                        },
                        |choice| {
                            let fs = platform.fs();
                            let used: u64 = [NAME, WAL]
                                .iter()
                                .map(|path| fs.metadata(path).map_or(0, |metadata| metadata.len))
                                .sum();

                            match choice % 3 {
                                0 => fs.set_io_error(FsOp::Write, NAME, ErrorKind::Other),
                                1 => fs.set_io_error(FsOp::Write, WAL, ErrorKind::Other),
                                _ => {
                                    fs.set_short_io(FsOp::Write, "", 8);
                                    fs.set_capacity(Some(used + choice / 3 % 64));
                                }
                            }
                        },
                    );
                }

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_if_logged() {
                    crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_if_logged(
                        open(&platform()),
                        true,
                    );
                    crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_if_logged(
                        spilling(&platform()),
                        true,
                    );
                }

                #[test]
                fn recover_from_wal_failure_should_fail() {
                    for (op, path) in [(FsOp::Read, WAL), (FsOp::Metadata, NAME)] {
                        let platform = platform();
                        let mut storage = spilling(&platform);
                        storage.write(0, &[1]).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.recover_from_wal().is_err(),
                            "op: {op:?}, path: {path}"
                        );
                    }
                }
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_should_make_last_flush_survive_crash_unless_sync_mode_none() {
                for (mode, expected) in
                    [(SyncMode::None, vec![]), (SyncMode::Commit, vec![1, 2, 3])]
                {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.set_sync_mode(mode);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();

                    drop(storage);

                    platform.fs().crash();
                    assert_eq!(read(&open(&platform)), expected, "mode: {mode}");
                }
            }

            #[test]
            fn drop_failure_should_keep_log_for_next_open() {
                let platform = platform();
                let mut storage = spilling(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.write(0, &[4]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Write, NAME, ErrorKind::Other);

                drop(storage);

                platform.fs().clear_faults();
                assert_eq!(read(&open(&platform)), vec![1, 2, 3]);
            }
        }
    }
}
