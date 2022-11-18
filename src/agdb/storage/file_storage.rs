use super::file_records::FileRecord;
use super::file_storage_impl::FileStorageImpl;
use super::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeFixedSized;
use crate::DbError;
use crate::DbIndex;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cmp::max;
use std::cmp::min;

pub struct FileStorage {
    data: RefCell<FileStorageImpl>,
}

impl FileStorage {
    pub fn new(filename: &String) -> Result<FileStorage, DbError> {
        Ok(FileStorage {
            data: RefCell::new(FileStorageImpl::new(filename)?),
        })
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        let len = self.len()?;
        self.data_mut().write(len, bytes)
    }

    fn data(&self) -> Ref<FileStorageImpl> {
        self.data.borrow()
    }

    fn data_mut(&self) -> RefMut<FileStorageImpl> {
        self.data.borrow_mut()
    }

    fn enlarge_value(&mut self, record: &mut FileRecord, new_size: u64) -> Result<u64, DbError> {
        let mut bytes = self.read_value(record)?;
        bytes.resize(new_size as usize, 0_u8);

        let len = self.len()?;
        self.update_record(record, len, new_size)?;

        self.append(&DbIndex::from_values(record.index, record.size).serialize())?;
        self.append(&bytes)?;

        Ok(new_size)
    }

    fn ensure_size(
        &mut self,
        record: &mut FileRecord,
        offset: u64,
        size: u64,
    ) -> Result<u64, DbError> {
        let new_size = offset + size;

        if new_size == record.size {
            return Ok(new_size);
        }

        if new_size > record.size {
            self.enlarge_value(record, new_size)?;
        }

        Ok(record.pos)
    }

    fn erase_bytes(
        &mut self,
        record: &FileRecord,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        if offset_from < offset_to {
            self.data_mut().write(
                record.pos + offset_from,
                &vec![0_u8; min(size, offset_to - offset_from) as usize],
            )
        } else {
            let position = max(offset_to + size, offset_from);
            self.data_mut().write(
                record.pos + position,
                &vec![0_u8; (offset_from + size - position) as usize],
            )
        }
    }

    fn invalidate_record(&mut self, pos: u64) -> Result<(), DbError> {
        self.data_mut().write(pos, &0_u64.serialize())
    }

    fn read_value(&self, record: &FileRecord) -> Result<Vec<u8>, DbError> {
        self.data_mut().read_exact(record.pos, record.size)
    }

    fn shrink_index(&mut self, record: &FileRecord, mut current_pos: u64) -> Result<u64, DbError> {
        if record.pos != current_pos {
            let bytes = self.read_value(record)?;
            self.data_mut().set_pos(record.index, current_pos);
            self.data_mut().write(
                current_pos,
                &DbIndex::from_values(record.index, record.size).serialize(),
            )?;
            self.data_mut()
                .write(current_pos + DbIndex::fixed_serialized_size(), &bytes)?;
        }

        current_pos += DbIndex::fixed_serialized_size() + record.size;

        Ok(current_pos)
    }

    fn shrink_records(&mut self, records: Vec<FileRecord>) -> Result<u64, DbError> {
        let mut current_pos = 0_u64;

        for record in records {
            current_pos = self.shrink_index(&record, current_pos)?;
        }

        Ok(current_pos)
    }

    fn shrink_value(&mut self, record: &mut FileRecord, new_size: u64) -> Result<u64, DbError> {
        let bytes = self.read_value(record)?;

        let len = self.len()?;
        self.update_record(record, len, new_size)?;

        self.append(&DbIndex::from_values(record.index, record.size).serialize())?;
        self.append(&bytes)?;

        Ok(new_size)
    }

    fn update_record(
        &mut self,
        record: &mut FileRecord,
        new_pos: u64,
        new_size: u64,
    ) -> Result<(), DbError> {
        self.invalidate_record(record.pos)?;
        self.data_mut().set_pos(record.index, new_pos);
        self.data_mut().set_size(record.index, new_size);
        record.pos = new_pos;
        record.size = new_size;

        Ok(())
    }

    fn validate_read_size(offset: u64, read_size: u64, value_size: u64) -> Result<(), DbError> {
        if offset > value_size {
            return Err(DbError::from(format!(
                "FileStorage read error: offset ({}) out of bounds ({})",
                offset, value_size
            )));
        }

        if (offset + read_size) > value_size {
            return Err(DbError::from(format!(
                "FileStorage read error: value ({}) out of bounds ({})",
                offset + read_size,
                value_size
            )));
        }

        Ok(())
    }
}

impl Storage for FileStorage {
    fn commit(&mut self) -> Result<(), DbError> {
        self.data_mut().end_transaction()
    }

    fn insert<T: Serialize>(&mut self, value: &T) -> Result<DbIndex, DbError> {
        self.insert_bytes(&value.serialize())
    }

    fn insert_at<T: Serialize>(
        &mut self,
        index: &DbIndex,
        offset: u64,
        value: &T,
    ) -> Result<u64, DbError> {
        self.insert_bytes_at(index, offset, &value.serialize())
    }

    fn insert_bytes(&mut self, bytes: &[u8]) -> Result<DbIndex, DbError> {
        let record = self.data_mut().new_record(self.len()?, bytes.len() as u64);
        let index = DbIndex::from_values(record.index, record.size);

        self.transaction();
        self.append(&index.serialize())?;
        self.append(bytes)?;
        self.commit()?;

        Ok(index)
    }

    fn insert_bytes_at(
        &mut self,
        index: &DbIndex,
        offset: u64,
        bytes: &[u8],
    ) -> Result<u64, DbError> {
        let mut record = self.data().record(index.value())?;

        self.transaction();
        let pos = self.ensure_size(&mut record, offset, bytes.len() as u64)?;
        self.data_mut().write(pos + offset, bytes)?;
        self.commit()?;

        Ok(record.size)
    }

    fn len(&self) -> Result<u64, DbError> {
        self.data_mut().len()
    }

    fn move_at<T: Serialize>(
        &mut self,
        index: &DbIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<u64, DbError> {
        let bytes = self.value_as_bytes_at_size(index, offset_from, size)?;
        let record = self.data().record(index.value())?;

        self.transaction();
        let value_len = self.insert_bytes_at(index, offset_to, &bytes)?;
        self.erase_bytes(&record, offset_from, offset_to, size)?;
        self.commit()?;

        Ok(value_len)
    }

    fn remove(&mut self, index: &DbIndex) -> Result<(), DbError> {
        let record = self.data().record(index.value())?;
        self.data_mut().remove_index(index.value());

        self.transaction();
        self.invalidate_record(record.pos)?;
        self.commit()
    }

    fn replace<T: Serialize>(&mut self, index: &DbIndex, value: &T) -> Result<u64, DbError> {
        self.replace_with_bytes(index, &value.serialize())
    }

    fn replace_with_bytes(&mut self, index: &DbIndex, bytes: &[u8]) -> Result<u64, DbError> {
        self.transaction();
        self.insert_bytes_at(index, 0, bytes)?;
        let len = self.resize_value(index, bytes.len() as u64)?;
        self.commit()?;

        Ok(len)
    }

    fn resize_value(&mut self, index: &DbIndex, new_size: u64) -> Result<u64, DbError> {
        let mut record = self.data().record(index.value())?;

        self.transaction();

        if new_size > record.size {
            self.enlarge_value(&mut record, new_size)?;
        } else if new_size < record.size {
            self.shrink_value(&mut record, new_size)?;
        }

        self.commit()?;

        Ok(record.size)
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.transaction();
        let records = self.data().records();
        let size = self.shrink_records(records)?;
        self.data_mut().truncate(size)?;

        self.commit()
    }

    fn transaction(&mut self) {
        self.data_mut().begin_transaction();
    }

    fn value<T: Serialize>(&self, index: &DbIndex) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes(index)?)
    }

    fn value_as_bytes(&self, index: &DbIndex) -> Result<Vec<u8>, DbError> {
        self.value_as_bytes_at(index, 0)
    }

    fn value_as_bytes_at(&self, index: &DbIndex, offset: u64) -> Result<Vec<u8>, DbError> {
        self.value_as_bytes_at_size(index, offset, self.value_size(index)?)
    }

    fn value_as_bytes_at_size(
        &self,
        index: &DbIndex,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, DbError> {
        let record = self.data().record(index.value())?;
        Self::validate_read_size(offset, size, record.size)?;
        let pos = record.pos + DbIndex::fixed_serialized_size() + offset;

        self.data_mut().read_exact(pos, record.size)
    }

    fn value_at<T: Serialize>(&self, index: &DbIndex, offset: u64) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes_at(index, offset)?)
    }

    fn value_size(&self, index: &DbIndex) -> Result<u64, DbError> {
        Ok(self.data().record(index.value())?.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use crate::utilities::serialize::SerializeDynamicSized;
    use crate::utilities::serialize::SerializeFixedSized;

    #[test]
    fn insert_value() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let value1 = "Hello, World!".to_string();
        let index1 = storage.insert(&value1).unwrap();
        assert!(index1.is_valid());
        assert_eq!(
            storage.value_size(&index1),
            Ok(value1.serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index1), Ok(index1.meta()));
        assert_eq!(storage.value(&index1), Ok(value1));

        let value2 = 10_i64;
        let index2 = storage.insert(&value2).unwrap();
        assert!(index2.is_valid());
        assert_eq!(
            storage.value_size(&index2),
            Ok(i64::fixed_serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index2), Ok(index2.meta()));
        assert_eq!(storage.value(&index2), Ok(value2));

        let value3 = vec![1_u64, 2_u64, 3_u64];
        let index3 = storage.insert(&value3).unwrap();
        assert!(index3.is_valid());
        assert_eq!(
            storage.value_size(&index3),
            Ok(value3.serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index3), Ok(index3.meta()));
        assert_eq!(storage.value(&index3), Ok(value3));

        let value4 = vec!["Hello".to_string(), "World".to_string()];
        let index4 = storage.insert(&value4).unwrap();
        assert!(index4.is_valid());
        assert_eq!(
            storage.value_size(&index4),
            Ok(value4.serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index4), Ok(index4.meta()));
        assert_eq!(storage.value(&index4), Ok(value4));
    }
}
