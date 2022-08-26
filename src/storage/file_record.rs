use super::serialize::Serialize;
use crate::db_error::DbError;
use std::mem::size_of;

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct FileRecord {
    pub(crate) index: i64,
    pub(crate) position: u64,
    pub(crate) size: u64,
}

#[allow(dead_code)]
impl FileRecord {
    pub(crate) const fn size() -> usize {
        size_of::<i64>() + size_of::<u64>()
    }
}

impl Serialize for FileRecord {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        const SIZE_OFFSET: usize = size_of::<i64>();
        const END: usize = SIZE_OFFSET + size_of::<u64>();

        Ok(FileRecord {
            index: i64::deserialize(&bytes[0..SIZE_OFFSET])?,
            position: 0,
            size: u64::deserialize(&bytes[SIZE_OFFSET..END])?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = self.index.serialize();
        bytes.extend(self.size.serialize());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn default_constructed() {
        let _record = FileRecord::default();
    }

    #[test]
    fn derived_from_debug() {
        let record = FileRecord::default();
        format!("{:?}", record);
    }

    #[test]
    fn derived_from_ord() {
        let record = FileRecord::default();
        assert_eq!(record.cmp(&record), Ordering::Equal);
    }

    #[test]
    fn deserialization_out_of_bounds() {
        let bytes = vec![0_u8; 4];
        let actual_record = FileRecord::deserialize(&bytes);

        assert_eq!(actual_record, Err(DbError::Storage("".to_string())));
    }

    #[test]
    fn serialization() {
        let record = FileRecord {
            index: 1,
            position: 0,
            size: 4,
        };

        let bytes = record.serialize();
        let actual_record = FileRecord::deserialize(&bytes);

        assert_eq!(actual_record, Ok(record));
    }

    #[test]
    fn size() {
        assert_eq!(FileRecord::size(), size_of::<i64>() + size_of::<u64>());
    }
}
