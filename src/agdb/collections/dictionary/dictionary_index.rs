use crate::db_error::DbError;
use crate::utilities::serialize::Serialize;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DictionaryIndex {
    pub(crate) index: i64,
}

impl DictionaryIndex {
    pub fn as_u64(&self) -> u64 {
        self.value() as u64
    }

    pub fn as_usize(&self) -> usize {
        self.value() as usize
    }

    pub fn is_valid(&self) -> bool {
        0 < self.index
    }

    pub fn value(&self) -> i64 {
        self.index
    }
}

impl From<i64> for DictionaryIndex {
    fn from(index: i64) -> Self {
        Self { index }
    }
}

impl Serialize for DictionaryIndex {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            index: i64::deserialize(bytes)?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        self.index.serialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let index = DictionaryIndex::default();

        format!("{:?}", index);
    }
}
