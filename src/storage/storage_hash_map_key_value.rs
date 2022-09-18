use crate::DbError;

use super::serialize::Serialize;

#[derive(Debug, PartialEq)]
pub(crate) struct StorageHashMapKeyValue<K: Serialize, T: Serialize> {
    key: K,
    value: T,
    empty: bool,
}

impl<K: Serialize, T: Serialize> Serialize for StorageHashMapKeyValue<K, T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(Self {
            key: K::deserialize(&bytes[0..])?,
            value: T::deserialize(&bytes[std::mem::size_of::<K>()..])?,
            empty: *bytes
                .get(std::mem::size_of::<K>() + std::mem::size_of::<T>())
                .ok_or_else(|| DbError::from("value out of bounds"))?
                != 0,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.reserve(std::mem::size_of::<K>() + std::mem::size_of::<T>() + 1);
        data.append(&mut self.key.serialize());
        data.append(&mut self.value.serialize());
        data.push(self.empty as u8);

        data
    }
}

impl<K: Serialize + Default, T: Serialize + Default> Default for StorageHashMapKeyValue<K, T> {
    fn default() -> Self {
        Self {
            key: K::default(),
            value: T::default(),
            empty: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let key_value = StorageHashMapKeyValue::<i64, i64>::default();

        format!("{:?}", key_value);
    }

    #[test]
    fn derived_from_default() {
        let key_value = StorageHashMapKeyValue::<i64, i64>::default();

        assert_eq!(
            key_value,
            StorageHashMapKeyValue::<i64, i64> {
                key: 0,
                value: 0,
                empty: true
            }
        )
    }

    #[test]
    fn i64_i64() {
        let key_value = StorageHashMapKeyValue {
            key: 1_i64,
            value: 10_i64,
            empty: false,
        };
        let bytes = key_value.serialize();
        let other = StorageHashMapKeyValue::deserialize(&bytes);

        assert_eq!(other, Ok(key_value));
    }

    #[test]
    fn out_of_bounds() {
        let bytes = vec![0_u8; 16];

        assert_eq!(
            StorageHashMapKeyValue::<i64, i64>::deserialize(&bytes)
                .unwrap_err()
                .description,
            "value out of bounds"
        );
    }
}
