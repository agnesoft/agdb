//! MVCC concurrent root: internal synchronization for `DbImpl`.
//!
//! Wraps [`Root`] so that read transactions are lock-free (epoch-based)
//! for [`MemoryStorage`], and serialized via an internal `RwLock` for
//! file-backed storages. Write transactions are always serialized via a
//! `Mutex`.
//!
//! The epoch protocol matches the benchmark prototype in `/tmp/s1bench`:
//! per-page epoch tracking, a version store for old pages, and reader
//! epoch registration for version-store trimming.

use super::StorageData;
use super::StorageSlice;
use super::pages::{Reader, View};
use super::root::{Meta, Root};
use crate::DbError;
use crate::DbErrorType;
use crate::Transaction;
use crate::TransactionMut;
use crate::transaction::ReadView;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

// ---------------------------------------------------------------------------
// Version store
// ---------------------------------------------------------------------------

/// A single old-page entry saved before a commit overwrites the page.
struct VersionEntry {
    /// The epoch at which this old content was superseded.
    epoch: u64,
    /// The page number (byte offset = `page << page_shift`).
    page: u64,
    /// The old page content.
    data: Vec<u8>,
}

/// Holds old page contents for active reader snapshots.
struct VersionStore {
    entries: Vec<VersionEntry>,
}

impl VersionStore {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Push entries for the pages replaced by the commit at `epoch`.
    fn push(&mut self, epoch: u64, captured: Vec<(u64, Vec<u8>)>) {
        self.entries
            .reserve(captured.len().saturating_sub(self.entries.capacity()));
        for (page, data) in captured {
            self.entries.push(VersionEntry { epoch, page, data });
        }
    }

    /// Find the page content that was current at `snap_epoch`.
    /// Returns the data from the **earliest** version whose epoch is
    /// greater than `snap_epoch` (that version replaced the page the
    /// reader should see).
    fn find(&self, page: u64, snap_epoch: u64) -> Option<&[u8]> {
        // We want the entry where entry.epoch > snap_epoch with the
        // smallest epoch (the first commit after the reader started).
        let mut best: Option<&VersionEntry> = None;

        for entry in &self.entries {
            if entry.page == page && entry.epoch > snap_epoch {
                match best {
                    Some(b) if entry.epoch < b.epoch => best = Some(entry),
                    None => best = Some(entry),
                    _ => {}
                }
            }
        }

        best.map(|e| e.data.as_slice())
    }

    /// Remove entries no longer needed: those whose epoch is ≤ the
    /// oldest active reader epoch.
    fn trim(&mut self, oldest_epoch: u64) {
        self.entries.retain(|e| e.epoch > oldest_epoch);
    }
}

// ---------------------------------------------------------------------------
// ConcurrentRoot
// ---------------------------------------------------------------------------

/// Internal synchronization wrapper around [`Root`].
///
/// # Safety
///
/// `UnsafeCell<Root<S>>` allows aliased access from readers and writers.
/// Correctness relies on:
///
/// 1. Writers are serialized via `write_mutex`.
/// 2. Epoch-based readers only read pages whose `page_epoch` is ≤ their
///    snapshot epoch — the writer never modifies those pages.
/// 3. Locked readers hold `read_lock` which excludes writers.
/// 4. Buffer reallocation (storage resize) spin-waits until
///    `active_readers == 0` before proceeding.
pub(crate) struct ConcurrentRoot<S: StorageData> {
    root: UnsafeCell<Root<S>>,

    /// Serializes write transactions.
    write_mutex: Mutex<()>,

    /// For file-backed storages: readers take a read lock, writers take
    /// a write lock. For epoch-based storages the writer also takes the
    /// write lock (to synchronize with potential locked readers on
    /// `AnyStorage`).
    read_lock: RwLock<()>,

    /// Committed meta, shared with epoch-based readers via `Arc`.
    /// Readers clone the `Arc` (cheap atomic increment) instead of
    /// deep-cloning the entire `Meta` structure.
    committed_meta: Mutex<Arc<Meta>>,

    /// Global epoch, bumped on each commit.
    epoch: AtomicU64,

    page_shift: u32,

    /// Per-page epoch. `page_epochs[p]` = the epoch at which page `p`
    /// was last committed. Accessed via `UnsafeCell` because it is read
    /// by epoch readers (atomically per element) and grown by writers
    /// (under `write_mutex`).
    page_epochs: UnsafeCell<Vec<AtomicU64>>,

    versions: Mutex<VersionStore>,

    /// Number of active epoch-based readers.
    active_readers: AtomicU64,

    /// The snapshot epochs held by active readers, for trimming.
    /// Maps epoch → count of readers at that epoch. Using a counted
    /// map instead of a set prevents dedup: two readers at the same
    /// epoch must both complete before that epoch can be trimmed.
    reader_epochs: Mutex<BTreeMap<u64, usize>>,

    /// Length of the committed storage, used for epoch-reader bounds.
    storage_len: AtomicU64,

    /// Set by a writer before it waits for epoch readers to drain for a
    /// resize. New epoch readers that see this flag fall back to the
    /// locked path, preventing them from obtaining raw pointers into
    /// the buffer that is about to be reallocated.
    resizing: std::sync::atomic::AtomicBool,

    /// `true` for `MemoryStorage` (epoch reads), `false` otherwise.
    epoch_reads: bool,
}

// SAFETY: see the doc-comment on ConcurrentRoot.
unsafe impl<S: StorageData + Send> Send for ConcurrentRoot<S> {}
unsafe impl<S: StorageData + Send> Sync for ConcurrentRoot<S> {}

impl<S: StorageData> ConcurrentRoot<S> {
    /// Wraps an already-opened `Root`. When `epoch_reads` is true the
    /// overlay limit of the storage is set to `u64::MAX` so that the
    /// transaction overlay never spills (the committed buffer always
    /// holds committed data).
    pub(crate) fn new(mut root: Root<S>, epoch_reads: bool) -> Self {
        let page_shift = root.page_shift();
        let storage_len = root.storage().len();
        let meta = root.committed_meta_clone();

        if epoch_reads {
            root.storage_mut().disable_spill();
        }

        // Only allocate page_epochs for epoch-based storages (MemoryStorage).
        // File-backed storages use the RwLock path and never read page_epochs.
        let page_epochs = if epoch_reads {
            let num_pages = storage_len.div_ceil(1u64 << page_shift) as usize;
            let capacity = num_pages.max(1024) * 4;
            let mut v = Vec::with_capacity(capacity);
            v.resize_with(num_pages, || AtomicU64::new(0));
            v
        } else {
            Vec::new()
        };

        Self {
            root: UnsafeCell::new(root),
            write_mutex: Mutex::new(()),
            read_lock: RwLock::new(()),
            committed_meta: Mutex::new(Arc::new(meta)),
            epoch: AtomicU64::new(0),
            page_shift,
            page_epochs: UnsafeCell::new(page_epochs),
            versions: Mutex::new(VersionStore::new()),
            active_readers: AtomicU64::new(0),
            reader_epochs: Mutex::new(BTreeMap::new()),
            storage_len: AtomicU64::new(storage_len),
            resizing: std::sync::atomic::AtomicBool::new(false),
            epoch_reads,
        }
    }

    // -- Reader (epoch-based, for MemoryStorage) --

    /// Start an epoch-based read. Returns the committed meta (as a
    /// cheap `Arc` clone) and a [`ReadGuard`] that must live for the
    /// duration of the read.
    /// Try to start an epoch-based read. Returns `None` when a resize
    /// is in progress (the caller should fall back to the locked path).
    fn begin_read_epoch(&self) -> Option<(ReadGuard<'_, S>, Arc<Meta>)> {
        // Register as a reader FIRST, so the writer's spin-wait will
        // see us and not proceed with the resize.
        self.active_readers.fetch_add(1, Ordering::AcqRel);

        // Check the resizing flag. The writer sets it BEFORE waiting
        // for active_readers to drain. If it's set, we must not hold
        // raw pointers into the buffer — back off and let the caller
        // use the locked path instead.
        if self.resizing.load(Ordering::Acquire) {
            self.active_readers.fetch_sub(1, Ordering::AcqRel);
            return None;
        }

        // Capture epoch first — the epoch is bumped AFTER meta and len
        // are updated, so snap_epoch <= committed epoch means our meta
        // and len are at least as recent as this epoch.
        let snap_epoch = self.epoch.load(Ordering::Acquire);
        // Arc::clone is ~2ns (atomic increment) vs Meta::clone ~100-300ns.
        // Meta is updated BEFORE epoch is bumped, so this is consistent
        // with snap_epoch.
        let meta = Arc::clone(&self.committed_meta.lock().unwrap());
        // storage_len is updated BEFORE meta, so len >= what meta needs.
        let snap_len = self.storage_len.load(Ordering::Acquire);

        *self
            .reader_epochs
            .lock()
            .unwrap()
            .entry(snap_epoch)
            .or_insert(0) += 1;

        Some((
            ReadGuard {
                parent: self,
                snap_epoch,
                snap_len,
            },
            meta,
        ))
    }

    // -- Public transaction entry points --

    /// Run an immutable (read) transaction.
    ///
    /// For epoch-based storages: the closure runs with a lock-free
    /// snapshot view. For file-backed: runs under the internal RwLock.
    pub(crate) fn transaction<T, E>(
        &self,
        f: impl FnOnce(&Transaction<S>) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<DbError>,
    {
        if self.epoch_reads
            && let Some((guard, meta_arc)) = self.begin_read_epoch()
        {
            let view = SnapshotView { guard: &guard };
            // Borrow from the Arc — the Arc lives for this scope,
            // which outlives the Transaction.
            let txn = Transaction::new(ReadView::Concurrent(&view), Cow::Borrowed(&meta_arc));
            return f(&txn);
        }
        // Not epoch_reads, or resize in progress — use locked path.
        {
            let _rguard = self
                .read_lock
                .read()
                .expect("ConcurrentRoot read_lock poisoned");
            // SAFETY: read_lock excludes writers, so &Root is unaliased.
            let root = unsafe { &*self.root.get() };
            let txn = match root.read() {
                Ok((reader, meta)) => Transaction::new(ReadView::Committed(reader), meta),
                Err(error) => Transaction::failed(Reader::new(root.storage()), error),
            };
            f(&txn)
        }
    }

    /// Run a mutable (write) transaction.
    ///
    /// Writers are always serialized. The closure receives a
    /// `&mut TransactionMut` and runs against the existing `Root`
    /// mechanism. After commit, the MVCC layer captures old pages for
    /// the version store and bumps the epoch.
    pub(crate) fn transaction_mut<T, E: From<DbError>>(
        &self,
        f: impl FnOnce(&mut TransactionMut<S>) -> Result<T, E>,
    ) -> Result<T, E> {
        // Serialize writers.
        let _wguard = self
            .write_mutex
            .lock()
            .expect("ConcurrentRoot write_mutex poisoned");
        // Also take the write side of read_lock to exclude locked readers.
        let _rguard = self
            .read_lock
            .write()
            .expect("ConcurrentRoot read_lock poisoned");

        // SAFETY: write_mutex + read_lock.write() ensure exclusive access.
        let root = unsafe { &mut *self.root.get() };
        let root_mut = root.begin().map_err(E::from)?;

        let mut transaction = TransactionMut::new(root_mut);
        let result = f(&mut transaction)?;

        // Always record which pages this transaction modified so that
        // post_commit can update page_epochs (needed even when no readers
        // are active — a reader arriving between commit and epoch bump
        // must see the correct page_epochs).
        //
        // Only capture the *old page data* when there are active epoch
        // readers that need snapshot data for the version store. This
        // avoids the per-page read + allocation overhead on the common
        // single-threaded write path.
        let has_readers = self.active_readers.load(Ordering::Acquire) > 0;
        let dirty_pages = if self.epoch_reads && !transaction.failed() {
            transaction.root_view().dirty_pages()
        } else {
            Vec::new()
        };
        let captured = if self.epoch_reads && !transaction.failed() && has_readers {
            transaction.root_view().capture_pages().map_err(E::from)?
        } else {
            Vec::new()
        };

        let will_resize =
            self.epoch_reads && !transaction.failed() && transaction.root_view().storage_resized();

        // If the storage will resize and epoch readers are active, we
        // must wait for them to finish first (they may hold references
        // into the buffer that would be invalidated by reallocation).
        if will_resize {
            // Set the resizing flag BEFORE draining. New epoch readers
            // that see this flag will fall back to the locked path.
            // The locked path blocks on read_lock (which we still hold
            // in write mode via _rguard), so fallback readers will wait
            // until we finish the commit and _rguard drops.
            self.resizing.store(true, Ordering::Release);
            while self.active_readers.load(Ordering::Acquire) > 0 {
                std::hint::spin_loop();
            }
        }

        // Commit through the existing Root/TransactionMut mechanism.
        transaction.commit().map_err(E::from)?;

        if self.epoch_reads {
            self.post_commit(dirty_pages, captured);
        }

        // Clear the resizing flag (if it was set). Must happen after
        // post_commit updates storage_len/meta/epoch so new epoch
        // readers see the correct state.
        if will_resize {
            self.resizing.store(false, Ordering::Release);
        }

        Ok(result)
    }

    /// After a successful commit: populate the version store, bump
    /// epoch, update page epochs, update committed meta.
    ///
    /// `dirty_pages` — page numbers modified by the transaction (always
    /// collected). `captured` — old page data (only collected when epoch
    /// readers were active at capture time).
    fn post_commit(&self, dirty_pages: Vec<u64>, captured: Vec<(u64, Vec<u8>)>) {
        let new_epoch = self.epoch.load(Ordering::Relaxed) + 1;

        let page_epochs = unsafe { &mut *self.page_epochs.get() };

        // Grow page_epochs if storage grew.
        // SAFETY: we hold write_mutex, no other writer can grow it,
        // and readers only access within their snapshot length.
        let root = unsafe { &*self.root.get() };
        let new_len = root.storage().len();
        let new_pages = new_len.div_ceil(1u64 << self.page_shift) as usize;

        if new_pages > page_epochs.len() {
            page_epochs.resize_with(new_pages, || AtomicU64::new(0));
        }

        // Store old pages in version store when readers were active.
        if !captured.is_empty() {
            self.versions.lock().unwrap().push(new_epoch, captured);
        }

        // ALWAYS update per-page epochs for every modified page.
        // This is critical: a reader that registers between our capture
        // check (active_readers == 0) and the epoch bump MUST see the
        // correct page_epochs so it knows which pages need the version
        // store. Without this, that reader would see page_epoch == 0
        // for a page that was just modified and read stale/concurrent data.
        //
        // SAFETY: we hold write_mutex. Epoch readers load these
        // atomically and only act on pages whose epoch > snap_epoch.
        for &page in &dirty_pages {
            if (page as usize) < page_epochs.len() {
                page_epochs[page as usize].store(new_epoch, Ordering::Release);
            }
        }

        // Trim version store.
        if !dirty_pages.is_empty() {
            let oldest = {
                let epochs = self.reader_epochs.lock().unwrap();
                epochs.keys().next().copied().unwrap_or(new_epoch)
            };
            self.versions.lock().unwrap().trim(oldest);
        }

        // Update storage length FIRST — readers snapshot len before meta,
        // so the len must be >= what the meta references.
        self.storage_len.store(new_len, Ordering::Release);

        // Update committed meta (wrap in Arc for cheap reader clones).
        {
            let meta = unsafe { &*self.root.get() }.committed_meta_clone();
            *self.committed_meta.lock().unwrap() = Arc::new(meta);
        }

        // Bump global epoch LAST — readers snapshot epoch first and use
        // it to decide which pages need version-store fallback.
        self.epoch.store(new_epoch, Ordering::Release);
    }

    // -- Accessors for DbImpl --

    pub(crate) fn storage(&self) -> &S {
        // SAFETY: returns a shared reference; callers must not use this
        // while a writer is active unless they hold the appropriate lock.
        unsafe { &*self.root.get() }.storage()
    }

    pub(crate) fn with_storage_mut<T>(&self, f: impl FnOnce(&mut S) -> T) -> T {
        let _wguard = self
            .write_mutex
            .lock()
            .expect("ConcurrentRoot write_mutex poisoned");
        let _rguard = self
            .read_lock
            .write()
            .expect("ConcurrentRoot read_lock poisoned");
        let root = unsafe { &mut *self.root.get() };
        f(root.storage_mut())
    }

    pub(crate) fn with_root_mut<T, E: From<DbError>>(
        &self,
        f: impl FnOnce(&mut Root<S>) -> Result<T, E>,
    ) -> Result<T, E> {
        let _wguard = self
            .write_mutex
            .lock()
            .expect("ConcurrentRoot write_mutex poisoned");
        let _rguard = self
            .read_lock
            .write()
            .expect("ConcurrentRoot read_lock poisoned");

        // Wait for epoch readers before mutating root.
        if self.epoch_reads {
            while self.active_readers.load(Ordering::Acquire) > 0 {
                std::hint::spin_loop();
            }
        }

        let root = unsafe { &mut *self.root.get() };
        let result = f(root)?;

        // The closure may have rebuilt the storage (e.g. optimize),
        // so refresh all MVCC state from the current root.
        if self.epoch_reads {
            self.refresh_mvcc_state();
        }

        Ok(result)
    }

    /// Resets all MVCC bookkeeping from the current root state.
    /// Called after operations that bypass `transaction_mut` (e.g.
    /// `optimize`, `rename`).
    fn refresh_mvcc_state(&self) {
        let root = unsafe { &*self.root.get() };
        let new_len = root.storage().len();
        let new_pages = new_len.div_ceil(1u64 << self.page_shift) as usize;

        // Update committed meta (wrap in Arc for cheap reader clones).
        *self.committed_meta.lock().unwrap() = Arc::new(root.committed_meta_clone());

        // Update storage length.
        self.storage_len.store(new_len, Ordering::Release);

        // Reset page epochs (all pages are "committed at epoch 0").
        let page_epochs = unsafe { &mut *self.page_epochs.get() };
        page_epochs.resize_with(new_pages, || AtomicU64::new(0));
        for pe in page_epochs.iter() {
            pe.store(0, Ordering::Relaxed);
        }

        // Reset epoch to 0 — no active versions.
        self.epoch.store(0, Ordering::Release);

        // Clear version store.
        self.versions.lock().unwrap().entries.clear();
    }

    #[allow(dead_code)]
    pub(crate) fn into_storage(self) -> S {
        self.root.into_inner().into_storage()
    }
}

// ---------------------------------------------------------------------------
// ReadGuard — RAII guard for epoch-based readers
// ---------------------------------------------------------------------------

/// Tracks an active epoch-based reader. On drop, unregisters the reader
/// and may trigger version-store trimming.
pub(crate) struct ReadGuard<'a, S: StorageData> {
    parent: &'a ConcurrentRoot<S>,
    snap_epoch: u64,
    snap_len: u64,
}

impl<S: StorageData> Drop for ReadGuard<'_, S> {
    fn drop(&mut self) {
        self.parent.active_readers.fetch_sub(1, Ordering::AcqRel);

        let mut epochs = self.parent.reader_epochs.lock().unwrap();
        if let Some(count) = epochs.get_mut(&self.snap_epoch) {
            *count -= 1;
            if *count == 0 {
                epochs.remove(&self.snap_epoch);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SnapshotView — epoch-aware View for read transactions
// ---------------------------------------------------------------------------

/// A [`View`] implementation that reads committed pages via the epoch
/// protocol. Pages not modified since the snapshot epoch are read
/// directly from committed storage; pages modified later are read from
/// the version store.
///
/// On the fast path (all pages committed), reads return **borrowed**
/// slices directly from the storage buffer — zero-copy. This is safe
/// because the [`ReadGuard`] keeps `active_readers > 0`, and the
/// writer spin-waits for `active_readers == 0` before any storage
/// resize that could reallocate the buffer.
struct SnapshotView<'a, S: StorageData> {
    guard: &'a ReadGuard<'a, S>,
}

impl<S: StorageData> View for SnapshotView<'_, S> {
    fn len(&self) -> u64 {
        self.guard.snap_len
    }

    fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
        let end = pos.saturating_add(len);

        if end > self.guard.snap_len {
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!(
                    "Snapshot read ({pos}..{end}) out of bounds ({})",
                    self.guard.snap_len
                ),
            ));
        }

        if len == 0 {
            return Ok(StorageSlice::Borrowed(&[]));
        }

        let parent = self.guard.parent;
        let shift = parent.page_shift;
        let first = pos >> shift;
        let last = (end - 1) >> shift;

        let page_epochs = unsafe { &*parent.page_epochs.get() };

        // Fast path: all pages are committed (not modified since snapshot).
        let all_committed = (first..=last).all(|p| {
            let idx = p as usize;
            idx < page_epochs.len()
                && page_epochs[idx].load(Ordering::Acquire) <= self.guard.snap_epoch
        });

        if all_committed {
            // Read directly from committed storage — zero-copy.
            //
            // SAFETY: The ReadGuard keeps active_readers > 0, and the
            // writer spin-waits for active_readers == 0 before any
            // storage resize (Vec reallocation). Therefore the buffer
            // address is stable for the entire ReadGuard lifetime.
            //
            // The returned StorageSlice borrows from &self (the
            // SnapshotView), which borrows from ReadGuard. The raw
            // pointer cast extends the storage borrow's lifetime to
            // match, which is sound because the buffer outlives the
            // ReadGuard.
            let storage = unsafe { &*parent.root.get() }.storage();
            let slice = storage.read(pos, len)?;
            Ok(match slice {
                Cow::Borrowed(bytes) => {
                    // Extend the lifetime from the temporary storage ref
                    // to &self. Sound because the buffer is pinned by the
                    // ReadGuard (see above).
                    let ptr = bytes.as_ptr();
                    let blen = bytes.len();
                    StorageSlice::Borrowed(unsafe { std::slice::from_raw_parts(ptr, blen) })
                }
                Cow::Owned(vec) => StorageSlice::Owned(vec),
            })
        } else {
            self.read_mixed(pos, len, first, last)
        }
    }
}

impl<S: StorageData> SnapshotView<'_, S> {
    /// Read that spans both committed and version-store pages.
    fn read_mixed(
        &self,
        pos: u64,
        len: u64,
        first: u64,
        last: u64,
    ) -> Result<StorageSlice<'_>, DbError> {
        let parent = self.guard.parent;
        let shift = parent.page_shift;
        let page_size = 1u64 << shift;
        let page_epochs = unsafe { &*parent.page_epochs.get() };
        let snap_epoch = self.guard.snap_epoch;
        let snap_len = self.guard.snap_len;
        let end = pos + len;

        let mut buffer = vec![0u8; len as usize];

        for page in first..=last {
            let page_start = page << shift;
            let read_start = page_start.max(pos);
            let read_end = (page_start + page_size).min(end).min(snap_len);

            if read_start >= read_end {
                continue;
            }

            let page_offset = (read_start - page_start) as usize;
            let buf_offset = (read_start - pos) as usize;
            let read_len = (read_end - read_start) as usize;

            let idx = page as usize;
            let modified =
                idx < page_epochs.len() && page_epochs[idx].load(Ordering::Acquire) > snap_epoch;

            if !modified {
                // Page is committed — read from storage.
                let storage = unsafe { &*parent.root.get() }.storage();
                let data = storage.read(read_start, read_len as u64)?;
                buffer[buf_offset..buf_offset + read_len].copy_from_slice(&data);
            } else {
                // Page was modified after our snapshot — look up version store.
                let versions = parent.versions.lock().unwrap();
                if let Some(old_data) = versions.find(page, snap_epoch) {
                    let available = old_data.len().min(page_offset + read_len);
                    if page_offset < available {
                        let copy_len = available - page_offset;
                        buffer[buf_offset..buf_offset + copy_len]
                            .copy_from_slice(&old_data[page_offset..page_offset + copy_len]);
                    }
                } else {
                    // The version store has no entry for this page at our
                    // snapshot epoch. This means the old data was not
                    // captured (no readers were active when the commit ran)
                    // but a reader registered after the commit. This is a
                    // consistency violation — return an error rather than
                    // silently serving zero-filled data.
                    return Err(DbError::storage(
                        DbErrorType::NotFound,
                        format!(
                            "Snapshot read: version store missing page {page} \
                             at epoch {snap_epoch} (page_epoch > snap_epoch)"
                        ),
                    ));
                }
            }
        }

        Ok(StorageSlice::Owned(buffer))
    }
}
