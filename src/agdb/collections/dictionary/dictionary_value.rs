use crate::db::db_error::DbError;
use crate::utilities::old_serialize::OldSerialize;
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Default)]
pub struct DictionaryValue<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + OldSerialize,
{
    pub(crate) meta: i64,
    pub(crate) hash: u64,
    pub(crate) value: T,
}

impl<T> OldSerialize for DictionaryValue<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + OldSerialize,
{
    fn old_deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(DictionaryValue::<T> {
            meta: i64::old_deserialize(bytes)?,
            hash: u64::old_deserialize(&bytes[(i64::fixed_size() as usize)..])?,
            value: T::old_deserialize(
                &bytes[((i64::fixed_size() + u64::fixed_size()) as usize)..],
            )?,
        })
    }

    fn old_serialize(&self) -> Vec<u8> {
        let mut bytes = self.meta.old_serialize();
        bytes.extend(self.hash.old_serialize());
        bytes.extend(self.value.old_serialize());

        bytes
    }
}
