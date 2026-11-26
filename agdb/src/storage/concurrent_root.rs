//! MVCC concurrent root: internal synchronization for `DbImpl`.
//!
//! Wraps [`Root`] so that read transactions are lock-free (epoch-based)
//! for [`MemoryStorage`] and [`FileStorageMemoryMapped`], and serialized
//! via an internal `RwLock` for [`FileStorage`]. Write transactions are
//! always serialized via a `Mutex`.
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

use std::collections::HashMap;

/// Holds old page contents for active reader snapshots.
///
/// Keyed by page number; each page maps to a `Vec` of `(epoch, data)`
/// pairs in ascending epoch order (writers always push with monotonically
/// increasing epochs).  `find` uses `partition_point` for O(log V)
/// lookup where V is the number of versions for a single page (typically
/// 1–3), eliminating the previous O(N) scan over all pages.
struct VersionStore {
    entries: HashMap<u64, Vec<(u64, Vec<u8>)>>,
}

impl VersionStore {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Push entries for the pages replaced by the commit at `epoch`.
    fn push(&mut self, epoch: u64, captured: Vec<(u64, Vec<u8>)>) {
        for (page, data) in captured {
            self.entries.entry(page).or_default().push((epoch, data));
        }
    }

    /// Find the page content that was current at `snap_epoch`.
    /// Returns the data from the **earliest** version whose epoch is
    /// greater than `snap_epoch` (that version replaced the page the
    /// reader should see).
    fn find(&self, page: u64, snap_epoch: u64) -> Option<&[u8]> {
        let page_entries = self.entries.get(&page)?;
        // Entries are in epoch order. Find the first with epoch > snap_epoch.
        let idx = page_entries.partition_point(|(e, _)| *e <= snap_epoch);
        page_entries.get(idx).map(|(_, data)| data.as_slice())
    }

    /// Remove entries no longer needed: those whose epoch is ≤ the
    /// oldest active reader epoch. Empty pages are removed from the map.
    fn trim(&mut self, oldest_epoch: u64) {
        self.entries.retain(|_, versions| {
            versions.retain(|(e, _)| *e > oldest_epoch);
            !versions.is_empty()
        });
    }

    /// Total number of version entries across all pages.
    #[cfg(test)]
    fn total_entries(&self) -> usize {
        self.entries.values().map(|v| v.len()).sum()
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

    /// `true` for `MemoryStorage` and `FileStorageMemoryMapped` (epoch
    /// reads), `false` for `FileStorage`.
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

        // Only allocate page_epochs for epoch-based storages (MemoryStorage,
        // FileStorageMemoryMapped). FileStorage uses the RwLock path.
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

    // -- Reader (epoch-based, for MemoryStorage / FileStorageMemoryMapped) --

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

        // Load epoch, meta and storage_len under a single
        // committed_meta lock so the reader sees values from the
        // same commit.  The writer updates all three under this lock
        // in post_commit / refresh_mvcc_state, so the triple is
        // always consistent.
        //
        // Also register in reader_epochs while still holding
        // committed_meta.  The writer's trim (in post_commit) locks
        // reader_epochs THEN later locks committed_meta — never both
        // at once — so there is no deadlock.  Registering here
        // closes the gap where a reader has snap_epoch but isn't yet
        // visible to trim: by the time the writer updates
        // committed_meta (after trim + push + page_epochs), this
        // reader is already registered, so a future trim won't
        // discard entries it needs.
        let (snap_epoch, meta, snap_len) = {
            let committed = self.committed_meta.lock().unwrap();
            let snap_epoch = self.epoch.load(Ordering::Acquire);
            let meta = Arc::clone(&committed);
            let snap_len = self.storage_len.load(Ordering::Acquire);

            *self
                .reader_epochs
                .lock()
                .unwrap()
                .entry(snap_epoch)
                .or_insert(0) += 1;

            (snap_epoch, meta, snap_len)
        };

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

        // Always record which pages this transaction modified and
        // capture their old contents BEFORE the commit overwrites them.
        //
        // Both are needed even when active_readers == 0 right now: a
        // new epoch reader can register between our check and the
        // post_commit epoch bump. That reader's snap_epoch predates
        // the bump, so it will consult the version store for any page
        // whose page_epoch exceeds its snap_epoch — and those entries
        // must already exist.
        let dirty_pages = if self.epoch_reads && !transaction.failed() {
            transaction.root_view().dirty_pages()
        } else {
            Vec::new()
        };
        let captured = if self.epoch_reads && !transaction.failed() {
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
            self.drain_epoch_readers();
        }

        // Push old pages to version store and update page_epochs BEFORE
        // commit writes new data. This prevents a race where a reader
        // sees new page data via the storage buffer but old page_epochs,
        // and reads from committed storage instead of version store.
        if self.epoch_reads {
            self.pre_commit_mvcc(&dirty_pages, captured);
        }

        // Commit through the existing Root/TransactionMut mechanism.
        transaction.commit().map_err(E::from)?;

        if self.epoch_reads {
            self.post_commit(dirty_pages);
        }

        // Clear the resizing flag (if it was set). Must happen after
        // post_commit updates storage_len/meta/epoch so new epoch
        // readers see the correct state.
        if will_resize {
            self.resizing.store(false, Ordering::Release);
        }

        Ok(result)
    }

    /// BEFORE commit: trim + push old pages to version store and update
    /// page_epochs. This must run before `transaction.commit()` writes
    /// new data, so epoch readers always see modified pages via
    /// page_epochs and go to the version store for old data.
    fn pre_commit_mvcc(&self, dirty_pages: &[u64], captured: Vec<(u64, Vec<u8>)>) {
        let new_epoch = self.epoch.load(Ordering::Relaxed) + 1;

        let page_epochs = unsafe { &mut *self.page_epochs.get() };

        // Grow page_epochs if needed (before we index into it).
        // SAFETY: we hold write_mutex, no other writer can grow it,
        // and readers only access within their snapshot length.
        let root = unsafe { &*self.root.get() };
        let new_len = root.storage().len();
        let new_pages = new_len.div_ceil(1u64 << self.page_shift) as usize;
        if new_pages > page_epochs.len() {
            page_epochs.resize_with(new_pages, || AtomicU64::new(0));
        }

        // Trim the version store BEFORE pushing new entries. If we
        // pushed first and then trimmed with no readers registered,
        // the oldest epoch would default to new_epoch and immediately
        // discard the entries we just added — racing with a reader
        // that is mid-registration (active_readers bumped but not yet
        // in reader_epochs).
        if !dirty_pages.is_empty() {
            let oldest = {
                let epochs = self.reader_epochs.lock().unwrap();
                epochs.keys().next().copied().unwrap_or(new_epoch)
            };
            self.versions.lock().unwrap().trim(oldest);
        }

        // Store old pages in version store (after trim so they survive).
        if !captured.is_empty() {
            self.versions.lock().unwrap().push(new_epoch, captured);
        }

        // Update per-page epochs for every modified page BEFORE the
        // commit writes new data. This prevents the race where a
        // reader sees new data via the storage buffer but old
        // page_epochs, causing it to read from committed storage
        // instead of the version store.
        //
        // SAFETY: we hold write_mutex. Epoch readers load these
        // atomically and only act on pages whose epoch > snap_epoch.
        for &page in dirty_pages {
            if (page as usize) < page_epochs.len() {
                page_epochs[page as usize].store(new_epoch, Ordering::Release);
            }
        }
    }

    /// After a successful commit: bump epoch, update committed meta.
    /// The version store and page_epochs were already handled by
    /// `pre_commit_mvcc`.
    fn post_commit(&self, dirty_pages: Vec<u64>) {
        let new_epoch = self.epoch.load(Ordering::Relaxed) + 1;

        // Grow page_epochs for newly allocated pages (commit may have
        // added pages beyond what pre_commit_mvcc sized).
        let page_epochs = unsafe { &mut *self.page_epochs.get() };
        let root = unsafe { &*self.root.get() };
        let new_len = root.storage().len();
        let new_pages = new_len.div_ceil(1u64 << self.page_shift) as usize;
        if new_pages > page_epochs.len() {
            page_epochs.resize_with(new_pages, || AtomicU64::new(0));
        }

        // Update page_epochs for any pages that were newly created by
        // commit (not in dirty_pages from before commit).
        for &page in &dirty_pages {
            if (page as usize) < page_epochs.len() {
                page_epochs[page as usize].store(new_epoch, Ordering::Release);
            }
        }

        // Update epoch, meta and storage_len under a single
        // committed_meta lock so that begin_read_epoch always sees a
        // consistent triple.
        {
            let meta = unsafe { &*self.root.get() }.committed_meta_clone();
            let mut committed = self.committed_meta.lock().unwrap();
            self.storage_len.store(new_len, Ordering::Release);
            *committed = Arc::new(meta);
            self.epoch.store(new_epoch, Ordering::Release);
        }
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

        // Wait for epoch readers before mutating root. Set the
        // resizing flag so that new epoch readers fall back to the
        // locked path (which blocks on read_lock we hold).
        if self.epoch_reads {
            self.resizing.store(true, Ordering::Release);
            self.drain_epoch_readers();
        }

        let root = unsafe { &mut *self.root.get() };
        let result = f(root)?;

        // The closure may have rebuilt the storage (e.g. optimize),
        // so refresh all MVCC state from the current root.
        if self.epoch_reads {
            self.refresh_mvcc_state();
            self.resizing.store(false, Ordering::Release);
        }

        Ok(result)
    }

    /// Spin-waits until all epoch-based readers have finished.
    ///
    /// Must be called with `resizing` already set to `true` and both
    /// `write_mutex` and `read_lock` held.  Uses progressive backoff:
    /// spin-loop for the first batch, then [`std::thread::yield_now`].
    /// Panics after 10 seconds if readers never drain (indicates a leaked
    /// reader guard).
    fn drain_epoch_readers(&self) {
        const SPIN_ITERS: u32 = 64;
        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

        let deadline = std::time::Instant::now() + TIMEOUT;
        let mut i = 0u32;
        while self.active_readers.load(Ordering::Acquire) > 0 {
            if i < SPIN_ITERS {
                std::hint::spin_loop();
                i += 1;
            } else {
                std::thread::yield_now();
                if std::time::Instant::now() >= deadline {
                    panic!(
                        "ConcurrentRoot: timed out ({TIMEOUT:?}) waiting for {} \
                         epoch reader(s) to drain",
                        self.active_readers.load(Ordering::Relaxed),
                    );
                }
            }
        }
    }

    /// Resets all MVCC bookkeeping from the current root state.
    /// Called after operations that bypass `transaction_mut` (e.g.
    /// `optimize`, `rename`).
    fn refresh_mvcc_state(&self) {
        let root = unsafe { &*self.root.get() };
        let new_len = root.storage().len();
        let new_pages = new_len.div_ceil(1u64 << self.page_shift) as usize;

        // Reset page epochs FIRST (all pages are "committed at epoch 0").
        let page_epochs = unsafe { &mut *self.page_epochs.get() };
        page_epochs.resize_with(new_pages, || AtomicU64::new(0));
        for pe in page_epochs.iter() {
            pe.store(0, Ordering::Relaxed);
        }

        // Update all committed state atomically (see post_commit).
        {
            let mut committed = self.committed_meta.lock().unwrap();
            *committed = Arc::new(root.committed_meta_clone());
            self.storage_len.store(new_len, Ordering::Release);
            self.epoch.store(0, Ordering::Release);
        }

        // Clear version store.
        self.versions.lock().unwrap().entries.clear();  // HashMap::clear()
    }

    /// Atomically replaces the internal [`Root`] (and its storage) with
    /// `new_root`.  Acquires exclusive access exactly like a write
    /// transaction, waits for all epoch readers to drain, then swaps.
    /// The old `Root` is dropped — releasing file handles / memory maps.
    /// All MVCC bookkeeping is reset via [`refresh_mvcc_state`].
    ///
    /// Returns an error when the new root has a different `page_shift`
    /// (the field is not atomic and must stay invariant while readers
    /// can observe it).
    pub(crate) fn restore(&self, mut new_root: Root<S>) -> Result<(), DbError> {
        if new_root.page_shift() != self.page_shift {
            return Err(DbError::storage(
                DbErrorType::NotAllowed,
                "restore: page shift mismatch".to_string(),
            ));
        }

        let _wguard = self
            .write_mutex
            .lock()
            .expect("ConcurrentRoot write_mutex poisoned");
        let _rguard = self
            .read_lock
            .write()
            .expect("ConcurrentRoot read_lock poisoned");

        // Wait for epoch readers (same protocol as with_root_mut).
        if self.epoch_reads {
            self.resizing.store(true, Ordering::Release);
            self.drain_epoch_readers();
        }

        if self.epoch_reads {
            new_root.storage_mut().disable_spill();
        }

        // Replace the root — dropping the old storage.
        let root = unsafe { &mut *self.root.get() };
        *root = new_root;

        // Reset all MVCC bookkeeping from the new root.
        self.refresh_mvcc_state();

        if self.epoch_reads {
            self.resizing.store(false, Ordering::Release);
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    mod version_store {
        use super::*;

        mod find {
            use super::*;

            #[test]
            fn find_should_return_earliest_superseding_entry() {
                let mut vs = VersionStore::new();
                vs.push(2, vec![(0, vec![10, 11])]);
                vs.push(5, vec![(0, vec![20, 21])]);
                vs.push(8, vec![(0, vec![30, 31])]);

                assert_eq!(vs.find(0, 1), Some(&[10, 11][..]));
                assert_eq!(vs.find(0, 4), Some(&[20, 21][..]));
                assert_eq!(vs.find(0, 7), Some(&[30, 31][..]));
            }

            #[test]
            fn find_at_or_past_latest_should_return_none() {
                let mut vs = VersionStore::new();
                vs.push(2, vec![(0, vec![10])]);

                assert_eq!(vs.find(0, 2), None);
                assert_eq!(vs.find(0, 100), None);
            }

            #[test]
            fn find_wrong_page_should_return_none() {
                let mut vs = VersionStore::new();
                vs.push(2, vec![(0, vec![10])]);

                assert_eq!(vs.find(1, 0), None);
            }

            #[test]
            fn find_empty_should_return_none() {
                let vs = VersionStore::new();

                assert_eq!(vs.find(0, 0), None);
            }
        }

        mod trim {
            use super::*;

            #[test]
            fn trim_should_remove_entries_at_or_below_oldest_epoch() {
                let mut vs = VersionStore::new();
                vs.push(1, vec![(0, vec![1])]);
                vs.push(2, vec![(0, vec![2])]);
                vs.push(5, vec![(0, vec![5])]);
                vs.push(10, vec![(0, vec![10])]);

                vs.trim(4);

                assert_eq!(vs.total_entries(), 2);
                assert_eq!(vs.find(0, 4), Some(&[5][..]));
                assert_eq!(vs.find(0, 9), Some(&[10][..]));
            }

            #[test]
            fn trim_all_should_leave_empty() {
                let mut vs = VersionStore::new();
                vs.push(1, vec![(0, vec![1])]);
                vs.push(2, vec![(0, vec![2])]);

                vs.trim(100);

                assert_eq!(vs.total_entries(), 0);
            }

            #[test]
            fn trim_zero_should_keep_all() {
                let mut vs = VersionStore::new();
                vs.push(5, vec![(0, vec![5])]);

                vs.trim(0);

                assert_eq!(vs.total_entries(), 1);
            }
        }

        mod push {
            use super::*;

            #[test]
            fn push_should_add_entries_for_each_page() {
                let mut vs = VersionStore::new();

                vs.push(3, vec![(0, vec![1, 2]), (5, vec![3, 4])]);

                assert_eq!(vs.total_entries(), 2);
                assert_eq!(vs.find(0, 2), Some(&[1, 2][..]));
                assert_eq!(vs.find(5, 2), Some(&[3, 4][..]));
            }

            #[test]
            fn push_multiple_epochs_same_page_should_keep_all() {
                let mut vs = VersionStore::new();
                vs.push(1, vec![(0, vec![10])]);
                vs.push(2, vec![(0, vec![20])]);

                assert_eq!(vs.total_entries(), 2);
                assert_eq!(vs.find(0, 0), Some(&[10][..]));
            }
        }
    }
}
