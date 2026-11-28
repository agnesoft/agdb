use super::StorageConfig;
use super::StorageData;
use super::StorageSlice;
use super::StorageSnapshot;
use super::SyncMode;
use super::pages::{Overlay, Reader, View};
use super::root::{Meta, Root};
use super::segments::Segments;
use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::Transaction;
use crate::TransactionMut;
use crate::db::id_pool::IdPool;
use crate::db::id_pool::changed;
use crate::transaction::ReadView;
use crate::transaction::Start;
use crate::transaction::Writes;
use crate::transaction_mut::Pending;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock, RwLockWriteGuard};

// WHY: lock order (deadlock avoidance): `write_mutex`, `root`, `snapshots.current`, `writing`.
pub(crate) struct ConcurrentRoot<S: StorageData> {
    root: RwLock<Root<S>>,

    write_mutex: Mutex<Writers>,

    snapshots: Option<Snapshots>,

    info: Mutex<StorageInfo>,

    ids: IdPool,

    version: AtomicU64,

    writing: Mutex<BTreeMap<u64, usize>>,

    spaces: Mutex<Vec<(Overlay, Meta)>>,
}

const SPACES: usize = 16;

const READERS_WAIT: std::time::Duration = std::time::Duration::from_secs(1);

const IN_PLACE: u64 = 1 << 20;

#[derive(Default)]
struct Writers {
    removed: HashMap<DbId, u64>,
    pruned: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StorageInfo {
    pub(crate) name: String,
    pub(crate) len: u64,
    pub(crate) config: StorageConfig,
}

impl StorageInfo {
    fn of<S: StorageData>(storage: &S) -> Self {
        Self {
            name: storage.name().to_string(),
            len: storage.len(),
            config: StorageConfig {
                sync_mode: storage.sync_mode(),
                overlay_limit: storage.overlay_limit(),
                memory_limit: storage.memory_limit(),
            },
        }
    }

    fn refresh<S: StorageData>(&mut self, storage: &S) {
        if self.name != storage.name() {
            storage.name().clone_into(&mut self.name);
        }

        self.len = storage.len();
        self.config = StorageConfig {
            sync_mode: storage.sync_mode(),
            overlay_limit: storage.overlay_limit(),
            memory_limit: storage.memory_limit(),
        };
    }
}

struct Snapshot {
    bytes: StorageSnapshot,
    meta: Result<Arc<Meta>, DbError>,
    version: u64,
    generation: u64,
}

impl Snapshot {
    fn of<S: StorageData>(
        root: &Root<S>,
        (version, generation): (u64, u64),
        meta: impl FnOnce(&Meta) -> Arc<Meta>,
    ) -> Option<Self> {
        Some(Self {
            bytes: root.storage().snapshot()?,
            meta: root.read().map(|(_, committed)| meta(&committed)),
            version,
            generation,
        })
    }
}

struct Snapshots {
    current: Mutex<Arc<Snapshot>>,

    retired: Mutex<Vec<Arc<Snapshot>>>,

    writing: Arc<Snapshot>,

    spare: Mutex<Option<Arc<Meta>>>,
}

impl Snapshots {
    fn new(snapshot: Snapshot) -> Self {
        Self {
            current: Mutex::new(Arc::new(snapshot)),
            retired: Mutex::new(Vec::new()),
            writing: Arc::new(Snapshot {
                bytes: StorageSnapshot::Segments(Segments::default()),
                meta: Err(DbError::storage(
                    DbErrorType::NotAllowed,
                    "Storage is being written",
                )),
                version: u64::MAX,
                generation: u64::MAX,
            }),
            spare: Mutex::new(None),
        }
    }

    fn meta(&self, meta: &Meta) -> Arc<Meta> {
        let spare = self
            .spare
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        Self::reuse(spare, meta)
    }

    fn reuse(mut spare: Option<Arc<Meta>>, meta: &Meta) -> Arc<Meta> {
        match spare.as_mut().and_then(Arc::get_mut) {
            Some(spare) => spare.copy_detached(meta),
            None => return Arc::new(meta.detached()),
        }

        spare.expect("spare is set")
    }

    fn spare(snapshot: Arc<Snapshot>) -> Option<Arc<Meta>> {
        match Arc::try_unwrap(snapshot) {
            Ok(Snapshot { meta: Ok(meta), .. }) => Some(meta),
            _ => None,
        }
    }

    fn recycle(&self, snapshot: Arc<Snapshot>) {
        if let Some(meta) = Self::spare(snapshot) {
            *self.spare.lock().unwrap_or_else(PoisonError::into_inner) = Some(meta);
        }
    }

    fn current(&self) -> MutexGuard<'_, Arc<Snapshot>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn retired(&self) -> MutexGuard<'_, Vec<Arc<Snapshot>>> {
        let mut retired = self.retired.lock().unwrap_or_else(PoisonError::into_inner);

        for snapshot in retired.extract_if(.., |snapshot| Arc::strong_count(snapshot) == 1) {
            self.recycle(snapshot);
        }

        retired
    }

    fn retire(&self, snapshot: Arc<Snapshot>) {
        if Arc::ptr_eq(&snapshot, &self.writing) {
            return;
        }

        let mut retired = self.retired();

        if Arc::strong_count(&snapshot) > 1 {
            retired.push(snapshot);
        } else {
            drop(retired);
            self.recycle(snapshot);
        }
    }

    fn publish_of<S: StorageData>(&self, root: &Root<S>, versions: (u64, u64)) {
        let mut current = self.current();

        if let Some(snapshot) = Arc::get_mut(&mut current)
            && let Some(bytes) = root.storage().snapshot()
        {
            snapshot.bytes = bytes;

            match root.read() {
                Ok((_, committed)) => match snapshot.meta.as_mut().ok().and_then(Arc::get_mut) {
                    Some(meta) => meta.copy_detached(&committed),
                    None => snapshot.meta = Ok(Arc::new(committed.detached())),
                },
                Err(e) => snapshot.meta = Err(e),
            }

            (snapshot.version, snapshot.generation) = versions;
            return;
        }

        drop(current);

        if let Some(snapshot) = Snapshot::of(root, versions, |meta| self.meta(meta)) {
            self.publish(snapshot);
        }
    }

    fn publish(&self, snapshot: Snapshot) {
        let old = std::mem::replace(&mut *self.current(), Arc::new(snapshot));
        self.retire(old);
    }

    fn oldest_held(&self) -> Option<u64> {
        let retired = self.retired().iter().map(|snapshot| snapshot.version).min();
        let current = self.current();
        let current = (Arc::strong_count(&current) > 1 && !Arc::ptr_eq(&current, &self.writing))
            .then_some(current.version);
        retired.into_iter().chain(current).min()
    }
}

struct WriteGuard<'a, S: StorageData> {
    parent: &'a ConcurrentRoot<S>,
    root: RwLockWriteGuard<'a, Root<S>>,
    published: bool,
    removed: Option<(Vec<DbId>, u64)>,
    writers: MutexGuard<'a, Writers>,
}

impl<S: StorageData> Drop for WriteGuard<'_, S> {
    fn drop(&mut self) {
        let parent = self.parent;
        parent
            .info
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .refresh(self.root.storage());

        if !parent.ids.made()
            && let Ok((_, meta)) = self.root.read()
        {
            parent.ids.init(&meta.graph);
        }

        if !self.published
            && let Some(snapshots) = &parent.snapshots
        {
            snapshots.publish_of(&self.root, parent.versions());
        }

        if let Some((removed, version)) = self.removed.take() {
            parent.keep_removed(&mut self.writers, removed, version);
        }
    }
}

impl<S: StorageData> ConcurrentRoot<S> {
    pub(crate) fn new(mut root: Root<S>) -> Self {
        let info = StorageInfo::of(root.storage());
        let ids = IdPool::default();

        if let Ok((_, meta)) = root.read() {
            ids.init(&meta.graph);
        }

        let snapshots = Snapshot::of(&root, (0, ids.generation()), |meta| {
            Arc::new(meta.detached())
        })
        .map(|snapshot| {
            root.storage_mut().disable_spill();
            Snapshots::new(snapshot)
        });

        Self {
            info: Mutex::new(info),
            root: RwLock::new(root),
            write_mutex: Mutex::new(Writers::default()),
            snapshots,
            ids,
            version: AtomicU64::new(0),
            writing: Mutex::new(BTreeMap::new()),
            spaces: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn transaction<T, E>(
        &self,
        f: impl FnOnce(&Transaction<S>) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<DbError>,
    {
        let registered = Cell::new(None);
        let _registered = Registered {
            parent: self,
            version: &registered,
        };
        let mut space = None;

        let (result, pending) = if let Some(snapshots) = &self.snapshots {
            let snapshot = Arc::clone(&snapshots.current());
            let view = SnapshotView(&snapshot.bytes);
            let started = || registered.set(Some(self.register(snapshot.version)));

            match &snapshot.meta {
                Ok(meta) => {
                    let writes = Writes::new(Start {
                        base: &view,
                        meta,
                        pool: &self.ids,
                        version: snapshot.version,
                        generation: snapshot.generation,
                        space: &mut space,
                        spaces: &self.spaces,
                        started: &started,
                    });
                    let result = f(&Transaction::new(
                        ReadView::Concurrent(&view),
                        Cow::Borrowed(&**meta),
                    )
                    .writable(&writes));
                    (result, writes.into_pending(&self.ids))
                }
                Err(error) => (
                    f(&Transaction::failed(
                        ReadView::Concurrent(&view),
                        error.clone(),
                    )),
                    None,
                ),
            }
        } else {
            let root = self.root.read().expect("ConcurrentRoot root lock poisoned");
            let base = Reader::new(root.storage());
            let version = self.version.load(Ordering::Relaxed);
            let started = || registered.set(Some(self.register(version)));

            match root.read() {
                Ok((reader, meta)) => {
                    let writes = Writes::new(Start {
                        base: &base,
                        meta: &meta,
                        pool: &self.ids,
                        version,
                        generation: self.ids.generation(),
                        space: &mut space,
                        spaces: &self.spaces,
                        started: &started,
                    });
                    let result = f(&Transaction::new(
                        ReadView::Committed(reader),
                        Cow::Borrowed(&*meta),
                    )
                    .writable(&writes));
                    (result, writes.into_pending(&self.ids))
                }
                Err(error) => {
                    let txn = Transaction::failed(ReadView::Committed(base), error);
                    (f(&txn), None)
                }
            }
        };

        let committed = match (&result, pending) {
            (Ok(_), Some(pending)) => self.commit_pending(pending, &mut space),
            _ => Ok(()),
        };

        if let Some((mut overlay, meta)) = space {
            let mut spaces = self.spaces.lock().unwrap_or_else(PoisonError::into_inner);

            if spaces.len() < SPACES {
                overlay.recycle();
                spaces.push((overlay, meta));
            }
        }

        let value = result?;
        committed.map(|()| value).map_err(E::from)
    }

    fn commit_pending(
        &self,
        pending: Pending<'_>,
        space: &mut Option<(Overlay, Meta)>,
    ) -> Result<(), DbError> {
        let Pending {
            failed,
            log,
            ids,
            lens,
        } = pending;

        if let Some(error) = failed {
            return Err(error);
        }

        if log.is_empty() {
            return Ok(());
        }

        let mut guard = self.lock_write();

        if ids.generation() != self.ids.generation() {
            return Err(changed());
        }

        let version = ids.version();
        let replay = self.version.load(Ordering::Relaxed) != version;
        let root_mut = guard.root.begin()?;
        let mut transaction = TransactionMut::new(root_mut).committing(ids, replay);

        if replay {
            transaction.replay(&log, version, &guard.writers.removed)?;
        } else {
            let (overlay, meta) = space.as_mut().expect("the transaction wrote");
            transaction.adopt(overlay, lens, meta);
        }

        let (committed, in_place) = self.commit(transaction, &mut guard.removed);
        self.publish_in_place(in_place, &mut guard);
        committed
    }

    /// Run a mutable (write) transaction.
    pub(crate) fn transaction_mut<T, E: From<DbError>>(
        &self,
        f: impl FnOnce(&mut TransactionMut<S>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut guard = self.lock_write();
        let root_mut = guard.root.begin().map_err(E::from)?;
        let mut transaction = TransactionMut::new(root_mut).with_ids(&self.ids);
        let result = f(&mut transaction)?;
        let (committed, in_place) = self.commit(transaction, &mut guard.removed);
        self.publish_in_place(in_place, &mut guard);
        committed.map_err(E::from)?;
        Ok(result)
    }

    fn commit(
        &self,
        transaction: TransactionMut<'_, S>,
        removed: &mut Option<(Vec<DbId>, u64)>,
    ) -> (Result<(), DbError>, Option<InPlace<'_>>) {
        let version = self.version.load(Ordering::Relaxed) + 1;
        let elements = transaction.removed().to_vec();
        let in_place = self.write_in_place(&transaction);
        let committed = transaction.commit(version);
        self.version.store(version, Ordering::Relaxed);

        if committed.is_ok() {
            *removed = Some((elements, version));
        }

        (committed, in_place)
    }

    fn write_in_place(&self, transaction: &TransactionMut<'_, S>) -> Option<InPlace<'_>> {
        let snapshots = self.snapshots.as_ref()?;
        let mut current = snapshots.current();

        // WHY: with no reader (the slot and the storage hold the bytes) a commit writes
        // in place. The slot stays locked until `publish_in_place`, so readers wait
        // rather than see the placeholder or unflushed bytes, hence only small unsynced commits.
        if Arc::strong_count(&current) == 1
            && let StorageSnapshot::Segments(bytes) = &current.bytes
            && bytes.holders() == 2
            && let Some((written, SyncMode::None)) = transaction.written()
            && written <= IN_PLACE
        {
            let spare = Snapshots::spare(std::mem::replace(
                &mut *current,
                Arc::clone(&snapshots.writing),
            ));
            return Some(InPlace { current, spare });
        }

        None
    }

    fn publish_in_place(&self, in_place: Option<InPlace<'_>>, guard: &mut WriteGuard<'_, S>) {
        let (
            Some(InPlace {
                mut current,
                mut spare,
            }),
            Some(snapshots),
        ) = (in_place, &self.snapshots)
        else {
            return;
        };

        if let Some(snapshot) = Snapshot::of(&guard.root, self.versions(), |meta| {
            Snapshots::reuse(spare.take(), meta)
        }) {
            *current = Arc::new(snapshot);
            guard.published = true;
        }

        drop(current);

        if let Some(meta) = spare {
            *snapshots
                .spare
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = Some(meta);
        }

        drop(snapshots.retired());
    }

    fn register(&self, version: u64) -> u64 {
        *self
            .writing
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(version)
            .or_default() += 1;
        version
    }

    fn unregister(&self, version: u64) {
        let mut writing = self.writing.lock().unwrap_or_else(PoisonError::into_inner);

        if let Some(count) = writing.get_mut(&version) {
            *count -= 1;

            if *count == 0 {
                writing.remove(&version);
            }
        }
    }

    fn keep_removed(&self, writers: &mut Writers, removed: Vec<DbId>, version: u64) {
        if removed.is_empty() && writers.removed.is_empty() {
            return;
        }

        let registered = self
            .writing
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .first_key_value()
            .map(|(version, _)| *version);
        let held = self.snapshots.as_ref().and_then(Snapshots::oldest_held);
        let oldest = registered.into_iter().chain(held).min();

        match oldest.filter(|oldest| *oldest < version) {
            None => {
                writers.removed.clear();
                writers.pruned = version;
            }
            Some(oldest) => {
                if writers.pruned < oldest {
                    writers.removed.retain(|_, removed| oldest < *removed);
                    writers.pruned = oldest;
                }

                writers
                    .removed
                    .extend(removed.into_iter().map(|id| (id, version)));
            }
        }
    }

    fn versions(&self) -> (u64, u64) {
        (self.version.load(Ordering::Relaxed), self.ids.generation())
    }

    pub(crate) fn info(&self) -> StorageInfo {
        self.info
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn with_storage<T>(&self, f: impl FnOnce(&S) -> T) -> T {
        let _write = self
            .write_mutex
            .lock()
            .expect("ConcurrentRoot write_mutex poisoned");
        let root = self.root.read().expect("ConcurrentRoot root lock poisoned");
        f(root.storage())
    }

    pub(crate) fn with_storage_mut<T>(&self, f: impl FnOnce(&mut S) -> T) -> T {
        let mut guard = self.lock_write();
        let result = f(guard.root.storage_mut());

        if self.snapshots.is_some() {
            guard.root.storage_mut().disable_spill();
        }

        result
    }

    pub(crate) fn with_root_mut<T, E: From<DbError>>(
        &self,
        f: impl FnOnce(&mut Root<S>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut guard = self.lock_write();
        let current = self.exclude_file_readers();
        let result = f(&mut guard.root);
        self.replaced(&mut guard);
        self.republish(&mut guard, current);
        result
    }

    /// Atomically replaces the internal [`Root`] (and its storage) with
    pub(crate) fn restore(&self, mut new_root: Root<S>) -> Result<(), DbError> {
        let mut guard = self.lock_write();

        if new_root.page_shift() != guard.root.page_shift() {
            return Err(DbError::storage(
                DbErrorType::NotAllowed,
                "restore: page shift mismatch".to_string(),
            ));
        }

        if self.snapshots.is_some() {
            new_root.storage_mut().disable_spill();
        }

        let current = self.exclude_file_readers();
        *guard.root = new_root;
        self.replaced(&mut guard);
        self.republish(&mut guard, current);
        Ok(())
    }

    fn exclude_file_readers(&self) -> Option<MutexGuard<'_, Arc<Snapshot>>> {
        let snapshots = self.snapshots.as_ref()?;
        let mut current = snapshots.current();

        if !matches!(current.bytes, StorageSnapshot::File(_)) {
            return None;
        }

        let start = std::time::Instant::now();
        let mut waited = 0_u32;

        // WHY: a reader nested in the caller's own read would never end; after the wait
        // replacing the file fails cleanly instead of deadlocking.
        while (Arc::strong_count(&current) > 1 || !snapshots.retired().is_empty())
            && start.elapsed() < READERS_WAIT
        {
            waited += 1;

            if waited < 64 {
                std::thread::yield_now();
            } else {
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
        }

        // WHY: drop the old snapshot now (unless readers hold it) so `f` can replace its file.
        let old = std::mem::replace(&mut *current, Arc::clone(&snapshots.writing));
        snapshots.retire(old);
        Some(current)
    }

    fn republish(
        &self,
        guard: &mut WriteGuard<'_, S>,
        current: Option<MutexGuard<'_, Arc<Snapshot>>>,
    ) {
        if let Some(mut current) = current
            && let Some(snapshot) = Snapshot::of(&guard.root, self.versions(), |meta| {
                Arc::new(meta.detached())
            })
        {
            *current = Arc::new(snapshot);
            guard.published = true;
        }
    }

    fn replaced(&self, guard: &mut WriteGuard<'_, S>) {
        self.version.fetch_add(1, Ordering::Relaxed);
        self.ids.reset();
        guard.writers.removed.clear();
    }

    fn lock_write(&self) -> WriteGuard<'_, S> {
        let writers = self.writers();
        let root = self
            .root
            .write()
            .expect("ConcurrentRoot root lock poisoned");

        WriteGuard {
            parent: self,
            root,
            published: false,
            removed: None,
            writers,
        }
    }
}

impl<S: StorageData> ConcurrentRoot<S> {
    fn writers(&self) -> MutexGuard<'_, Writers> {
        // WHY: commits hold the lock for microseconds; spinning avoids paying the
        // scheduler wake-up of a blocked writer on every handoff.
        for attempt in 0..SPINS + YIELDS {
            if let Ok(writers) = self.write_mutex.try_lock() {
                return writers;
            }

            if attempt < SPINS {
                for _ in 0..1 << attempt.min(6) {
                    std::hint::spin_loop();
                }
            } else {
                std::thread::yield_now();
            }
        }

        self.write_mutex
            .lock()
            .expect("ConcurrentRoot write_mutex poisoned")
    }
}

const SPINS: u32 = 32;
const YIELDS: u32 = 32;

struct Registered<'a, S: StorageData> {
    parent: &'a ConcurrentRoot<S>,
    version: &'a Cell<Option<u64>>,
}

impl<S: StorageData> Drop for Registered<'_, S> {
    fn drop(&mut self) {
        if let Some(version) = self.version.get() {
            self.parent.unregister(version);
        }
    }
}

struct InPlace<'a> {
    current: MutexGuard<'a, Arc<Snapshot>>,
    spare: Option<Arc<Meta>>,
}

struct SnapshotView<'a>(&'a StorageSnapshot);

impl View for SnapshotView<'_> {
    fn len(&self) -> u64 {
        match self.0 {
            StorageSnapshot::Segments(bytes) => bytes.len(),
            StorageSnapshot::File(file) => file.len(),
        }
    }

    fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
        let end = pos.saturating_add(len);

        if end > self.len() {
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!(
                    "Snapshot read ({pos}..{end}) out of bounds ({})",
                    self.len()
                ),
            ));
        }

        match self.0 {
            StorageSnapshot::Segments(bytes) => Ok(bytes.read(pos, len)),
            StorageSnapshot::File(file) => Ok(StorageSlice::Owned(file.read(pos, len)?)),
        }
    }
}
