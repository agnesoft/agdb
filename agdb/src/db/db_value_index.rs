use crate::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DbValueIndex {
    value: [u8; 16],
}

impl DbValueIndex {
    pub fn new() -> Self {
        Self { value: [0_u8; 16] }
    }

    pub fn data(&self) -> [u8; 16] {
        self.value
    }

    pub fn get_type(&self) -> u8 {
        self.value[15] >> 4
    }

    pub fn index(&self) -> u64 {
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(&self.value[0..std::mem::size_of::<u64>()]);
        u64::from_le_bytes(bytes)
    }

    pub fn is_value(&self) -> bool {
        self.size() != 0 || self.index() == 0
    }

    pub fn set_type(&mut self, value: u8) {
        let v = (value << 4) | self.size();
        self.value[15] = v;
    }

    pub fn set_index(&mut self, index: u64) {
        self.set_size(0);
        self.value[0..std::mem::size_of::<u64>()].copy_from_slice(&index.to_le_bytes());
    }

    pub fn set_value(&mut self, value: &[u8]) -> bool {
        if value.len() > 15 {
            return false;
        }

        self.set_size(value.len() as u8);
        self.value[0..(value.len())].copy_from_slice(value);

        true
    }

    pub fn size(&self) -> u8 {
        self.value[15] & 0b00001111
    }

    pub fn value(&self) -> &[u8] {
        let pos = self.size();
        &self.value[0..(pos as usize)]
    }

    fn set_size(&mut self, size: u8) {
        let v = (size & 0b00001111) | (self.value[15] & 0b11110000);
        self.value[15] = v;
    }
}

impl Serialize for DbValueIndex {
    fn serialize(&self) -> Vec<u8> {
        self.value.to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() < 16 {
            return Err(DbError::from(
                "DbValueIndex deserialization error: out of bounds",
            ));
        }

        let mut index = Self::default();
        index.value.copy_from_slice(&bytes[0..16]);
        Ok(index)
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl SerializeStatic for DbValueIndex {
    fn serialized_size_static() -> u64 {
        16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", DbValueIndex::new());
    }

    #[test]
    fn derived_from_clone_partial_eq() {
        let index = DbValueIndex::new();
        assert_eq!(index, index.clone());
    }

    #[test]
    fn value_type() {
        let mut index = DbValueIndex::default();
        index.set_type(3);

        assert_eq!(index.get_type(), 3);
    }

    #[test]
    fn value() {
        let mut index = DbValueIndex::default();
        let value = vec![1_u8, 2_u8, 3_u8];

        assert!(index.set_value(&value));
        assert_eq!(index.value(), value);
        assert_eq!(index.size(), 3);
        assert!(index.is_value());
    }

    #[test]
    fn value_type_size() {
        let mut index = DbValueIndex::default();
        let value = vec![1_u8, 2_u8, 3_u8];

        index.set_type(5);

        assert!(index.set_value(&value));
        assert_eq!(index.value(), value);
        assert_eq!(index.size(), 3);
        assert_eq!(index.get_type(), 5);
        assert!(index.is_value());
    }

    #[test]
    fn value_max() {
        let mut index = DbValueIndex::default();
        let value = vec![1_u8; 15];
        assert!(index.set_value(&value));

        assert_eq!(index.value(), value);
        assert_eq!(index.size(), 15);
        assert!(index.is_value());
    }

    #[test]
    fn value_too_large() {
        let mut index = DbValueIndex::default();
        let value = vec![1_u8; 16];
        assert!(!index.set_value(&value));

        assert_eq!(index.value(), Vec::<u8>::new());
        assert_eq!(index.size(), 0);
        assert!(index.is_value());
    }

    #[test]
    fn set_index() {
        let mut index = DbValueIndex::default();
        index.set_index(10_u64);

        assert_eq!(index.value(), Vec::<u8>::new());
        assert_eq!(index.size(), 0);
        assert_eq!(index.index(), 10_u64);
        assert!(!index.is_value());
    }

    #[test]
    fn serialize() {
        let mut index = DbValueIndex::new();
        assert!(index.set_value(&[1_u8, 2_u8, 3_u8]));

        let data = index.serialize();
        let other_index = DbValueIndex::deserialize(&data).unwrap();

        assert_eq!(index, other_index);
    }

    #[test]
    fn bad_deserialize() {
        let bad_data = vec![1_u8; 15];
        assert_eq!(
            DbValueIndex::deserialize(&bad_data)
                .unwrap_err()
                .description,
            "DbValueIndex deserialization error: out of bounds"
        );
    }

    #[test]
    fn serialized_size() {
        assert_eq!(DbValueIndex::default().serialized_size(), 16);
    }
}
