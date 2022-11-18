use crate::db::db_error::DbError;
use crate::utilities::serialize::OldSerialize;

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

impl OldSerialize for DictionaryIndex {
    fn old_deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            index: i64::old_deserialize(bytes)?,
        })
    }

    fn old_serialize(&self) -> Vec<u8> {
        self.index.old_serialize()
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
