use crate::storage::Storage;
use crate::storage_data::StorageData;

impl<T: StorageData> Drop for Storage<T> {
    fn drop(&mut self) {
        if self.apply_wal().is_ok() {
            let _ = self.data.clear_wal();
        }
    }
}
