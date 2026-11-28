pub(crate) mod alloc;
pub mod any_storage;
pub(crate) mod concurrent_root;
pub mod file_snapshot;
pub mod file_storage;
pub mod file_storage_memory_mapped;
pub mod memory_storage;

mod overlay;
mod page_cache;
#[allow(dead_code)]
pub(crate) mod pages;
mod ranges;
pub(crate) mod root;
pub(crate) mod segments;
#[allow(dead_code)]
pub(crate) mod storage_simulator;
mod write_ahead_log;

use crate::DbError;
use file_snapshot::FileSnapshot;
use segments::Segments;
use std::borrow::Cow;

/// The read-only bytes of a storage as of its last commit that the read
/// transactions run on without waiting for the writers, see
/// [`StorageData::snapshot()`].
pub enum StorageSnapshot {
    /// The copy-on-write bytes of an in-memory storage.
    Segments(Segments),
    /// The file of a [`FileStorage`](crate::FileStorage) with the old
    /// value of the bytes written since.
    File(FileSnapshot),
}

pub const DEFAULT_OVERLAY_LIMIT: u64 = 64 * 1024 * 1024;

/// Controls whether the underlying storage layer performs synchronization
/// on every storage commit. For example issuing `fsync`/`fdatasync`
/// for physical filesystems.
///
/// - `None` — no explicit sync; relies on the OS to flush dirty pages.
///   Suitable for local filesystems or when running in a replicated cluster.
/// - `Commit` — calls `sync_data()` on the WAL before writing the main file
///   and on the main file before clearing the WAL at each transaction
///   commit. The clearing is not synced, so a crash may roll back the last
///   commit. Required for correctness on network filesystems with
///   client-side write caching (e.g. WekaFS with `writecache`).
/// - `Strict` — like `Commit` and also syncs the cleared WAL so that a
///   commit survives a crash.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyncMode {
    #[default]
    None,
    Commit,
    Strict,
}

impl std::fmt::Display for SyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncMode::None => f.write_str("none"),
            SyncMode::Commit => f.write_str("commit"),
            SyncMode::Strict => f.write_str("strict"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageConfig {
    pub sync_mode: SyncMode,
    pub overlay_limit: u64,
    /// The memory the database may use, `None` (or `0`) for unlimited.
    /// The memory and memory mapped databases fail the transactions that
    /// would grow them past it, the file database caches its reads in
    /// that much memory.
    pub memory_limit: Option<u64>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            sync_mode: SyncMode::None,
            overlay_limit: DEFAULT_OVERLAY_LIMIT,
            memory_limit: None,
        }
    }
}

/// Convenience alias for `Cow<'a, [u8]>`.
pub type StorageSlice<'a> = Cow<'a, [u8]>;

/// Minimum set of data operations required by the database
/// to store & retrieve data.
pub trait StorageData: Sized {
    /// Writes the current database state to `name` as a backup.
    /// File implementations copy the underlying file(s); the
    /// memory implementation dumps the buffer to disk.
    fn backup(&self, _name: &str) -> Result<(), DbError>;

    /// Copies the storage to a new `name`.
    fn copy(&self, name: &str) -> Result<Self, DbError>;

    /// Creates an empty storage of the same kind as this one named `name`,
    /// discarding what was stored under that name. The default
    /// implementation opens `name` with [`new()`](Self::new) and empties it.
    fn create(&self, name: &str) -> Result<Self, DbError> {
        let mut storage = Self::new(name)?;
        storage.resize(0)?;
        storage.flush()?;
        Ok(storage)
    }

    /// Flushes any buffers to the underlying storage (e.g. file). The
    /// default implementation does nothing.
    fn flush(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    /// Convenience method that returns `len() == 0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the underlying storage in bytes.
    fn len(&self) -> u64;

    /// Returns the name this storage was constructed with.
    fn name(&self) -> &str;

    /// Constructs or loads the storage `name`. The `name` might be
    /// a file name or other identifier.
    fn new(name: &str) -> Result<Self, DbError>;

    /// Reads `value_len` bytes starting at `pos`. Returns [`StorageSlice`]
    /// (COW).
    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError>;

    /// Changes the name of the storage changing also the names of the files
    /// (if the storage is file based).
    fn rename(&mut self, new_name: &str) -> Result<(), DbError>;

    /// Resizes the underlying storage to `new_len`. If the storage is enlarged as
    /// a result the new bytes should be initialized to `0_u8`.
    fn resize(&mut self, new_len: u64) -> Result<(), DbError>;

    /// Sets how many bytes of a transaction the storage holds in memory
    /// before writing them ahead of the commit. The default implementation
    /// does nothing which is appropriate for in-memory storages.
    fn set_overlay_limit(&mut self, _limit: u64) {}

    /// Returns the overlay limit. The default is [`DEFAULT_OVERLAY_LIMIT`].
    fn overlay_limit(&self) -> u64 {
        DEFAULT_OVERLAY_LIMIT
    }

    /// The read-only snapshot of the bytes of the storage that the read
    /// transactions run on without waiting for the writers, `None` (the
    /// default) when they have to wait (they read from the storage).
    /// [`MemoryStorage`](crate::MemoryStorage) and
    /// [`FileStorageMemoryMapped`](crate::FileStorageMemoryMapped) take
    /// it from their in-memory bytes, [`FileStorage`](crate::FileStorage)
    /// reads its file. It is taken between the transactions only.
    fn snapshot(&self) -> Option<StorageSnapshot> {
        None
    }

    /// Disables the page-overlay spill so that the storage only changes
    /// on commit. Used for the storages with a [`snapshot()`](Self::snapshot)
    /// as their writes cannot be rolled back. The default is a no-op —
    /// file-backed storages rely on the WAL for recovery.
    fn disable_spill(&mut self) {}

    /// Fails if the storage may not grow to `len` bytes, e.g. past the
    /// [`memory_limit()`](Self::memory_limit) of an in-memory storage.
    /// Checked before a transaction grows the storage so that it fails
    /// without changing it. The default always succeeds.
    fn check_len(&self, _len: u64) -> Result<(), DbError> {
        Ok(())
    }

    /// Sets the memory the storage may use, `None` (or `0`) for
    /// unlimited. The storages holding the bytes in memory fail to grow
    /// past it (see [`check_len()`](Self::check_len)), the file storage
    /// caches its reads in that much memory. The default does nothing.
    fn set_memory_limit(&mut self, _limit: Option<u64>) {}

    /// Returns the memory limit, see
    /// [`set_memory_limit()`](Self::set_memory_limit). The default is `None`.
    fn memory_limit(&self) -> Option<u64> {
        None
    }

    /// Sets the sync mode for this storage. The default implementation
    /// does nothing which is appropriate for in-memory storages.
    fn set_sync_mode(&mut self, _mode: SyncMode) {}

    /// Returns the current sync mode. The default is [`SyncMode::None`].
    fn sync_mode(&self) -> SyncMode {
        SyncMode::None
    }

    /// Ensures all previously written data reaches durable storage (e.g.
    /// calls `fdatasync`). The default implementation does nothing which
    /// is appropriate for in-memory storages.
    fn sync(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    /// Writes the `bytes` to the underlying storage at `pos`. The implementation
    /// must handle the case where the `pos + bytes.len()` exceeds the current
    /// [`len()`](#method.len).
    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError>;

    /// Replays the WAL in reverse to restore the pre-transaction file state,
    /// then clears the WAL and restores len.  Default is a no-op (no WAL).
    fn recover_from_wal(&mut self) -> Result<(), DbError> {
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    mod sync_mode {
        use super::*;

        mod std_fmt_display {
            use super::*;

            #[test]
            fn fmt_should_show_lowercase_name() {
                assert_eq!(SyncMode::None.to_string(), "none");
                assert_eq!(SyncMode::Commit.to_string(), "commit");
                assert_eq!(SyncMode::Strict.to_string(), "strict");
            }
        }
    }

    mod storage_config {
        use super::*;

        mod default {
            use super::*;

            #[test]
            fn default_should_not_sync_and_use_default_overlay_limit() {
                assert_eq!(
                    StorageConfig::default(),
                    StorageConfig {
                        sync_mode: SyncMode::None,
                        overlay_limit: DEFAULT_OVERLAY_LIMIT,
                        memory_limit: None,
                    }
                );
            }
        }
    }

    pub(crate) mod storage_data {
        use super::*;

        const LIMIT: u64 = 8192;

        pub(crate) mod backup {
            use super::*;

            pub(crate) fn backup_should_write_bytes_to_name<S: StorageData>(
                mut storage: S,
                name: &str,
                read: impl Fn(&str) -> Vec<u8>,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();

                storage.backup(name).unwrap();

                assert_eq!(read(name), vec![1, 2, 3]);
            }
        }

        pub(crate) mod copy {
            use super::*;

            pub(crate) fn copy_should_create_independent_storage_named_name<S: StorageData>(
                mut storage: S,
                name: &str,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();

                let mut copy = storage.copy(name).unwrap();

                assert_eq!(copy.name(), name);
                assert_eq!(copy.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
                copy.write(0, &[4]).unwrap();
                assert_eq!(storage.read(0, 1).unwrap().to_vec(), vec![1]);
            }
        }

        pub(crate) mod create {
            use super::*;

            pub(crate) fn create_should_replace_name_with_empty_storage<S: StorageData>(
                mut storage: S,
                name: &str,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                drop(storage.copy(name).unwrap());

                let mut created = storage.create(name).unwrap();

                assert_eq!(created.name(), name);
                assert_eq!(created.len(), 0);
                created.write(0, &[4]).unwrap();
                assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
            }
        }

        pub(crate) mod flush {
            use super::*;

            pub(crate) fn flush_should_keep_written_bytes_on_recover<S: StorageData>(
                mut storage: S,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();

                storage.flush().unwrap();

                storage.recover_from_wal().unwrap();
                assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
            }
        }

        pub(crate) mod is_empty {
            use super::*;

            pub(crate) fn is_empty_should_be_true_only_without_bytes<S: StorageData>(
                mut storage: S,
            ) {
                let empty = storage.is_empty();

                storage.write(0, &[1]).unwrap();

                assert!(empty);
                assert!(!storage.is_empty());
            }
        }

        pub(crate) mod len {
            use super::*;

            pub(crate) fn len_should_cover_written_bytes<S: StorageData>(mut storage: S) {
                storage.write(3, &[1, 2]).unwrap();

                assert_eq!(storage.len(), 5);
            }
        }

        pub(crate) mod name {
            use super::*;

            pub(crate) fn name_should_return_name<S: StorageData>(storage: S, name: &str) {
                assert_eq!(storage.name(), name);
            }
        }

        pub(crate) mod new {
            use super::*;

            pub(crate) fn new_should_create_empty_storage_named_name<S: StorageData>() {
                let storage = S::new("/new").unwrap();

                assert_eq!(storage.name(), "/new");
                assert_eq!(storage.len(), 0);
            }
        }

        pub(crate) mod read {
            use super::*;

            pub(crate) fn read_should_return_bytes_at_pos<S: StorageData>(mut storage: S) {
                storage.write(0, &[1, 2, 3, 4]).unwrap();

                assert_eq!(storage.read(1, 2).unwrap().to_vec(), vec![2, 3]);
                assert!(storage.read(4, 0).unwrap().is_empty());
            }

            pub(crate) fn read_out_of_bounds_should_fail<S: StorageData>(mut storage: S) {
                storage.write(0, &[1, 2, 3, 4]).unwrap();

                for (pos, len) in [(3, 2), (u64::MAX, 1)] {
                    assert!(storage.read(pos, len).is_err(), "pos: {pos}, len: {len}");
                }
            }
        }

        pub(crate) mod rename {
            use super::*;

            pub(crate) fn rename_should_change_name_and_keep_bytes<S: StorageData>(
                mut storage: S,
                new_name: &str,
            ) {
                storage.write(0, &[1, 2]).unwrap();
                storage.flush().unwrap();

                storage.rename(new_name).unwrap();

                assert_eq!(storage.name(), new_name);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![1, 2]);
            }
        }

        pub(crate) mod resize {
            use super::*;

            pub(crate) fn resize_should_zero_new_bytes_and_cut_off_others<S: StorageData>(
                mut storage: S,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();

                storage.resize(5).unwrap();
                let enlarged = storage.read(0, 5).unwrap().to_vec();
                storage.resize(5).unwrap();
                storage.resize(2).unwrap();

                assert_eq!(enlarged, vec![1, 2, 3, 0, 0]);
                assert_eq!(storage.len(), 2);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![1, 2]);
            }

            pub(crate) fn resize_past_memory_limit_should_fail_if_bounded<S: StorageData>(
                mut storage: S,
                bounded: bool,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.set_memory_limit(Some(2));

                let grown = storage.resize(LIMIT);
                storage.set_memory_limit(None);
                storage.resize(3).unwrap();

                assert_eq!(grown.is_err(), bounded);
                assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
                storage.set_memory_limit(Some(2));
                storage.resize(1).unwrap();
                assert_eq!(storage.len(), 1);
            }
        }

        pub(crate) mod check_len {
            use super::*;

            pub(crate) fn check_len_should_fail_past_memory_limit_if_bounded<S: StorageData>(
                mut storage: S,
                bounded: bool,
            ) {
                storage.set_memory_limit(Some(LIMIT));

                assert!(storage.check_len(LIMIT).is_ok());
                assert_eq!(storage.check_len(LIMIT + 1).is_err(), bounded);
                storage.set_memory_limit(None);
                assert!(storage.check_len(LIMIT + 1).is_ok());
            }
        }

        pub(crate) mod set_memory_limit {
            use super::*;

            pub(crate) fn set_memory_limit_should_set_limit_with_zero_as_none<S: StorageData>(
                mut storage: S,
            ) {
                storage.set_memory_limit(Some(LIMIT));
                let limit = storage.memory_limit();
                storage.set_memory_limit(Some(0));

                assert_eq!(limit, Some(LIMIT));
                assert_eq!(storage.memory_limit(), None);
            }
        }

        pub(crate) mod memory_limit {
            use super::*;

            pub(crate) fn memory_limit_should_default_to_none<S: StorageData>(storage: S) {
                assert_eq!(storage.memory_limit(), None);
            }
        }

        pub(crate) mod set_overlay_limit {
            use super::*;

            pub(crate) fn set_overlay_limit_should_set_limit_if_supported<S: StorageData>(
                mut storage: S,
                supported: bool,
            ) {
                storage.set_overlay_limit(5);

                let expected = if supported { 5 } else { DEFAULT_OVERLAY_LIMIT };
                assert_eq!(storage.overlay_limit(), expected);
            }
        }

        pub(crate) mod overlay_limit {
            use super::*;

            pub(crate) fn overlay_limit_should_default_to_default_overlay_limit<S: StorageData>(
                storage: S,
            ) {
                assert_eq!(storage.overlay_limit(), DEFAULT_OVERLAY_LIMIT);
            }
        }

        pub(crate) mod set_sync_mode {
            use super::*;

            pub(crate) fn set_sync_mode_should_set_mode_if_supported<S: StorageData>(
                mut storage: S,
                supported: bool,
            ) {
                storage.set_sync_mode(SyncMode::Commit);

                let expected = if supported {
                    SyncMode::Commit
                } else {
                    SyncMode::None
                };
                assert_eq!(storage.sync_mode(), expected);
            }
        }

        pub(crate) mod sync_mode {
            use super::*;

            pub(crate) fn sync_mode_should_default_to_none<S: StorageData>(storage: S) {
                assert_eq!(storage.sync_mode(), SyncMode::None);
            }
        }

        pub(crate) mod sync {
            use super::*;

            pub(crate) fn sync_should_succeed<S: StorageData>(mut storage: S) {
                storage.write(0, &[1]).unwrap();

                storage.sync().unwrap();
            }
        }

        pub(crate) mod write {
            use super::*;

            pub(crate) fn write_should_overwrite_and_extend<S: StorageData>(mut storage: S) {
                storage.write(0, &[1, 2, 3]).unwrap();

                storage.write(2, &[4, 5]).unwrap();
                storage.write(0, &[6]).unwrap();

                assert_eq!(storage.len(), 4);
                assert_eq!(storage.read(0, 4).unwrap().to_vec(), vec![6, 2, 4, 5]);
            }

            pub(crate) fn write_past_end_should_zero_gap<S: StorageData>(mut storage: S) {
                storage.write(2, &[1]).unwrap();

                assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![0, 0, 1]);
            }

            pub(crate) fn write_past_memory_limit_should_fail_if_bounded<S: StorageData>(
                mut storage: S,
                bounded: bool,
            ) {
                storage.write(0, &[1; LIMIT as usize]).unwrap();
                storage.set_memory_limit(Some(LIMIT));

                let overwritten = storage.write(0, &[2, 2]);
                let extended = storage.write(LIMIT - 1, &[3, 3]);

                assert!(overwritten.is_ok());
                assert_eq!(extended.is_err(), bounded);
                let expected_len = if bounded { LIMIT } else { LIMIT + 1 };
                assert_eq!(storage.len(), expected_len);
                assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![2, 2, 1]);
            }

            pub(crate) fn write_empty_should_do_nothing<S: StorageData>(mut storage: S) {
                storage.write(5, &[]).unwrap();

                assert_eq!(storage.len(), 0);
            }
        }

        pub(crate) mod recover_from_wal {
            use super::*;

            pub(crate) fn recover_from_wal_should_restore_flushed_bytes_if_logged<
                S: StorageData,
            >(
                mut storage: S,
                logged: bool,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.write(1, &[4, 5, 6]).unwrap();

                storage.recover_from_wal().unwrap();

                let (len, expected) = if logged {
                    (3, vec![1, 2, 3])
                } else {
                    (4, vec![1, 4, 5, 6])
                };
                assert_eq!(storage.len(), len);
                assert_eq!(storage.read(0, len).unwrap().to_vec(), expected);
            }

            fn bytes<S: StorageData>(storage: &S) -> Vec<u8> {
                if storage.is_empty() {
                    return Vec::new();
                }

                storage.read(0, storage.len()).unwrap().to_vec()
            }

            pub(crate) fn recover_from_wal_should_restore_flushed_bytes_after_random_changes<
                S: StorageData,
            >(
                mut open: impl FnMut() -> S,
                kill: impl Fn(),
                fault: impl Fn(u64),
            ) {
                let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
                let mut random = move |n: u64| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    seed % n
                };
                let mut storage = open();
                let mut flushed = Vec::new();
                let mut expected = Vec::new();

                for round in 0..500 {
                    storage
                        .set_overlay_limit([0, 64, 512, DEFAULT_OVERLAY_LIMIT][random(4) as usize]);
                    storage.set_sync_mode(
                        [SyncMode::None, SyncMode::Commit, SyncMode::Strict][random(3) as usize],
                    );

                    for _ in 0..1 + random(8) {
                        if random(4) == 0 {
                            let len = random(64);
                            storage.resize(len).unwrap();
                            expected.resize(len as usize, 0);
                        } else {
                            let pos = random(expected.len() as u64 + 8) as usize;
                            let value = vec![1 + random(255) as u8; 1 + random(16) as usize];
                            storage.write(pos as u64, &value).unwrap();
                            expected.resize(expected.len().max(pos + value.len()), 0);
                            expected[pos..pos + value.len()].copy_from_slice(&value);
                        }
                    }

                    assert_eq!(bytes(&storage), expected, "round: {round}");

                    match random(4) {
                        0 => {
                            storage.flush().unwrap();
                            flushed = expected.clone();
                        }
                        1 => {
                            storage.recover_from_wal().unwrap();
                            expected = flushed.clone();
                        }
                        2 => {
                            kill();
                            storage = open();
                            expected = flushed.clone();
                        }
                        _ => {
                            fault(random(u64::MAX));

                            if storage.flush().is_ok() {
                                flushed = expected.clone();
                            }

                            kill();
                            storage = open();
                            expected = flushed.clone();
                        }
                    }

                    assert_eq!(bytes(&storage), expected, "round: {round}");
                }
            }
        }
    }
}
