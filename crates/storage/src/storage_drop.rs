use crate::storage_data::StorageData;
use crate::storage_impl::StorageImpl;

impl<T: StorageData> Drop for StorageImpl<T> {
    fn drop(&mut self) {
        if self.apply_wal().is_ok() {
            let _ = self.data.clear_wal();
        }
    }
}
