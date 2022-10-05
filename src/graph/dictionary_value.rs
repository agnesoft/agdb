use crate::storage::Serialize;
use crate::storage::StableHash;
use crate::DbError;

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct DictionaryValue<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(super) count: u64,
    pub(super) value: T,
}

impl<T> Serialize for DictionaryValue<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(DictionaryValue::<T> {
            count: u64::deserialize(bytes)?,
            value: T::deserialize(&bytes[(u64::serialized_size() as usize)..])?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = self.count.serialize();
        bytes.extend(self.value.serialize());

        bytes
    }
}
