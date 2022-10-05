use crate::storage::Serialize;
use crate::storage::StableHash;
use crate::DbError;

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct DictionaryValue<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(super) meta: i64,
    pub(super) hash: u64,
    pub(super) value: T,
}

impl<T> Serialize for DictionaryValue<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(DictionaryValue::<T> {
            meta: i64::deserialize(bytes)?,
            hash: u64::deserialize(&bytes[(i64::serialized_size() as usize)..])?,
            value: T::deserialize(
                &bytes[((i64::serialized_size() + u64::serialized_size()) as usize)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = self.meta.serialize();
        bytes.extend(self.hash.serialize());
        bytes.extend(self.value.serialize());

        bytes
    }
}
