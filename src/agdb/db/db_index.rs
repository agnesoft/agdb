use crate::utilities::serialize::{Serialize, SerializeFixedSized};
use crate::DbError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DbIndex {
    value: u64,
    meta: u64,
}

impl DbIndex {
    pub fn as_usize(&self) -> usize {
        usize::try_from(self.value).unwrap_or(0)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<DbIndex, DbError> {
        if bytes.len() as u64 > DbIndex::fixed_serialized_size() {
            return Err(DbError::from(format!(
                "DbIndex::from_bytes error: value ({}) too long (>{})",
                bytes.len(),
                DbIndex::fixed_serialized_size()
            )));
        }

        DbIndex::deserialize(bytes)
    }

    pub fn from_values(value: u64, meta: u64) -> DbIndex {
        DbIndex { value, meta }
    }

    pub fn is_valid(&self) -> bool {
        self.value != 0
    }

    pub fn meta(&self) -> u64 {
        self.meta
    }

    pub fn meta_as_usize(&self) -> usize {
        usize::try_from(self.meta).unwrap_or(0)
    }

    pub fn new() -> DbIndex {
        DbIndex::default()
    }

    pub fn set_meta(&mut self, meta: u64) {
        self.meta = meta
    }

    pub fn set_value(&mut self, value: u64) {
        self.value = value
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

impl Serialize for DbIndex {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(Self::fixed_serialized_size() as usize);
        bytes.extend(self.value.serialize());
        bytes.extend(self.meta.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(DbIndex {
            value: u64::deserialize(bytes)?,
            meta: u64::deserialize(&bytes[u64::fixed_serialized_size() as usize..])?,
        })
    }
}

impl SerializeFixedSized for DbIndex {}

impl From<usize> for DbIndex {
    fn from(value: usize) -> Self {
        Self {
            value: value as u64,
            meta: 0,
        }
    }
}

impl From<u64> for DbIndex {
    fn from(value: u64) -> Self {
        Self { value, meta: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", DbIndex::from(1_u64));
    }

    #[test]
    fn derived_from_clone() {
        let mut index = DbIndex::from(1_u64);
        index.set_meta(2_u64);

        assert_eq!(index.clone(), index);
    }

    #[test]
    fn derived_from_default() {
        assert_eq!(DbIndex::default(), DbIndex::new())
    }

    #[test]
    fn derived_from_eq() {
        assert_eq!(DbIndex::default(), DbIndex::default())
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut indexes = vec![DbIndex::from(1_u64), DbIndex::new(), DbIndex::from(2_u64)];
        indexes.sort();

        assert_eq!(
            indexes,
            vec![DbIndex::new(), DbIndex::from(1_u64), DbIndex::from(2_u64)]
        );
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DbIndex::default().cmp(&DbIndex::default()), Ordering::Equal);
    }

    #[test]
    fn from_usize() {
        let value = 1_usize;
        let index = DbIndex::from(value);

        assert_eq!(index.as_usize(), value);
    }

    #[test]
    fn from_u64() {
        let value = 1_u64;
        let index = DbIndex::from(value);

        assert_eq!(index.value(), value);
    }

    #[test]
    fn from_bytes() {
        let bytes_full = [
            15_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 10_u8, 11_u8, 12_u8,
            13_u8, 14_u8, 15_u8,
        ];

        assert_eq!(
            DbIndex::from_bytes(&bytes_full),
            DbIndex::deserialize(&bytes_full)
        );

        let partial_bytes = [1_u8];

        assert_eq!(
            DbIndex::from_bytes(&partial_bytes),
            DbIndex::deserialize(&partial_bytes)
        );
    }

    #[test]
    fn from_bytes_too_long() {
        let bytes = vec![0_u8; 17];

        assert_eq!(
            DbIndex::from_bytes(&bytes),
            Err(DbError::from(
                "DbIndex::from_bytes error: value (17) too long (>16)"
            ))
        );
    }

    #[test]
    fn from_values() {
        let mut left = DbIndex::new();
        left.set_value(1_u64);
        left.set_meta(2_u64);

        let right = DbIndex::from_values(1_u64, 2_u64);

        assert_eq!(left, right);
    }

    #[test]
    fn is_valid() {
        let mut index = DbIndex::default();

        assert!(!index.is_valid());

        index.set_meta(1_u64);

        assert!(!index.is_valid());

        index.set_value(1_u64);

        assert!(index.is_valid());
    }

    #[test]
    fn meta() {
        let mut index = DbIndex::default();

        assert_eq!(index.meta(), 0);
        assert_eq!(index.meta_as_usize(), 0);

        index.set_meta(1_u64);

        assert_eq!(index.meta(), 1_u64);
        assert_eq!(index.meta_as_usize(), 1_usize);
    }

    #[test]
    fn value() {
        let mut index = DbIndex::default();

        assert_eq!(index.value(), 0);

        index.set_value(1_u64);

        assert_eq!(index.value(), 1_u64);
    }

    #[test]
    fn serialize() {
        let mut original = DbIndex::default();
        original.set_value(1_u64);
        original.set_meta(2_u64);

        let serialized_size = DbIndex::fixed_serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = DbIndex::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }
}
