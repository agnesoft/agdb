#[allow(dead_code)]
pub(crate) mod aliases;
#[allow(dead_code)]
pub(crate) mod array;
pub mod bit_set;
#[allow(dead_code)]
pub(crate) mod graph;
#[allow(dead_code)]
pub(crate) mod indexes;
#[allow(dead_code)]
pub(crate) mod key_values;
pub mod map;
pub mod multi_map;
#[allow(dead_code)]
pub(crate) mod table;
#[allow(dead_code)]
pub(crate) mod value;
pub mod vec;

#[cfg(test)]
pub(crate) mod tests {
    use crate::DbError;
    use crate::collections::map::MapDataIndex;
    use crate::storage::Storage;
    use crate::storage::StorageIndex;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::Platform;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::fmt::Debug;
    use std::io::ErrorKind;
    use std::sync::Arc;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    pub(crate) type TestStorage = Storage<FileStorageImpl<TestPlatform>>;

    pub(crate) fn storage() -> TestStorage {
        new_storage(&Arc::new(test_platform()))
    }

    fn new_storage(platform: &Arc<TestPlatform>) -> TestStorage {
        FileStorageImpl::with_platform(platform.clone(), NAME)
            .and_then(Storage::with_data)
            .unwrap()
    }

    /// Fails each read, write and truncation of the files the `action`
    /// does, one at a time, on the subject the `setup` prepared, and checks
    /// that the failure is returned until the `action` runs out of calls to
    /// fail.
    pub(crate) fn assert_io_failures<T, R>(
        setup: impl Fn(&mut TestStorage) -> T,
        action: impl Fn(&mut TestStorage, &mut T) -> Result<R, DbError>,
    ) {
        assert_io_failures_recover(setup, action, |_, _| ());
    }

    /// As [`assert_io_failures`], also checking with `verify` what
    /// recovering the storage from each failure restores.
    pub(crate) fn assert_io_failures_recover<T, R>(
        setup: impl Fn(&mut TestStorage) -> T,
        action: impl Fn(&mut TestStorage, &mut T) -> Result<R, DbError>,
        verify: impl Fn(&mut TestStorage, &T),
    ) {
        io_failures(setup, action, |storage, subject| {
            storage.recover().unwrap();
            verify(storage, subject);
        });
    }

    /// As [`assert_io_failures`], also checking with `verify` the subject
    /// right after each failure, before recovering the storage.
    pub(crate) fn assert_io_failures_keep<T, R>(
        setup: impl Fn(&mut TestStorage) -> T,
        action: impl Fn(&mut TestStorage, &mut T) -> Result<R, DbError>,
        verify: impl Fn(&mut TestStorage, &T),
    ) {
        io_failures(setup, action, |storage, subject| {
            verify(storage, subject);
            storage.recover().unwrap();
        });
    }

    /// Fails each read of the database file the `action` does, one at a
    /// time, on the subject the `setup` prepared, and checks that the
    /// failure is returned until the `action` runs out of reads and returns
    /// what it returns without failures.
    pub(crate) fn assert_read_failures<T, R: Debug + PartialEq>(
        setup: impl Fn(&mut TestStorage) -> T,
        action: impl Fn(&mut TestStorage, &mut T) -> Result<R, DbError>,
    ) {
        assert_read_failures_on(
            |platform| {
                let mut storage = new_storage(platform);
                let subject = setup(&mut storage);
                (storage, subject)
            },
            |subject| action(&mut subject.0, &mut subject.1),
        );
    }

    /// As [`assert_read_failures`] for a subject that `open` creates with
    /// its own storage on the platform.
    pub(crate) fn assert_read_failures_on<T, R: Debug + PartialEq>(
        open: impl Fn(&Arc<TestPlatform>) -> T,
        action: impl Fn(&mut T) -> Result<R, DbError>,
    ) {
        let expected = action(&mut open(&Arc::new(test_platform()))).unwrap();
        let mut failures = 0;

        for nth in 0.. {
            let platform = Arc::new(test_platform());
            let mut subject = open(&platform);
            let mut results = vec![None; nth];
            results.push(Some(ErrorKind::Other));
            platform.fs().set_io_errors(FsOp::Read, NAME, results);

            let result = action(&mut subject);

            platform.fs().clear_faults();

            if let Ok(result) = result {
                assert_eq!(result, expected, "read {nth} failed silently");
                break;
            }

            failures += 1;
        }

        assert!(failures > 0, "the action should read");
    }

    fn io_failures<T, R>(
        setup: impl Fn(&mut TestStorage) -> T,
        action: impl Fn(&mut TestStorage, &mut T) -> Result<R, DbError>,
        after: impl Fn(&mut TestStorage, &T),
    ) {
        let mut failures = 0;

        for (op, path) in [
            (FsOp::Read, NAME),
            (FsOp::Write, NAME),
            (FsOp::SetLen, NAME),
            (FsOp::Write, WAL),
            (FsOp::SetLen, WAL),
        ] {
            for nth in 0.. {
                let platform = Arc::new(test_platform());
                let mut storage = new_storage(&platform);
                let mut subject = setup(&mut storage);
                let mut results = vec![None; nth];
                results.push(Some(ErrorKind::Other));
                platform.fs().set_io_errors(op, path, results);

                let result = action(&mut storage, &mut subject);

                platform.fs().clear_faults();

                if result.is_ok() {
                    break;
                }

                after(&mut storage, &subject);
                failures += 1;
            }
        }

        assert!(failures > 0, "the action should do I/O");
    }

    /// Doubles the vecs of the map at `index`, as maps rehashed in place by
    /// earlier versions can have spare capacity.
    pub(crate) fn add_spare_capacity(storage: &mut TestStorage, index: StorageIndex) {
        let data = storage.value::<MapDataIndex>(index).unwrap();

        for vec in [data.states_index, data.keys_index, data.values_index] {
            let size = storage.value_size(vec).unwrap();
            storage.resize_value(vec, size * 2).unwrap();
        }
    }
}
