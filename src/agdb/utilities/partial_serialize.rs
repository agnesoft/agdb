use crate::utilities::serialize::Serialize;
use crate::DbError;
use std::any::type_name;

pub trait PartialSerialize: Sized {
    fn serialized_size(&self) -> usize;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
}

impl PartialSerialize for String {
    fn serialized_size(&self) -> usize {
        usize::serialized_size() + self.len()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size());
        bytes.extend(self.len().serialize());
        bytes.extend(self.as_bytes());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let begin = usize::serialized_size();
        let end = begin + len;

        Ok(String::from_utf8(
            bytes
                .get(begin..end)
                .ok_or_else(|| DbError::from("String deserialization error: out of bounds"))?
                .to_vec(),
        )?)
    }
}

impl<T: Serialize> PartialSerialize for Vec<T> {
    fn serialized_size(&self) -> usize {
        usize::serialized_size() + self.len() * T::serialized_size()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size());
        bytes.extend(self.len().serialize());

        for value in self {
            bytes.extend(value.serialize());
        }

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let mut begin = usize::serialized_size();
        let mut end = begin + T::serialized_size();
        let mut vec = Self::new();
        vec.reserve(len);

        for _ in 0..len {
            vec.push(T::deserialize(bytes.get(begin..end).ok_or_else(|| {
                DbError::from(format!(
                    "Vec<{}> deserialization error: out of bounds",
                    type_name::<T>()
                ))
            })?)?);
            begin += T::serialized_size();
            end += T::serialized_size();
        }

        Ok(vec)
    }
}

impl PartialSerialize for Vec<u8> {
    fn serialized_size(&self) -> usize {
        usize::serialized_size() + self.len()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size());
        bytes.extend(self.len().serialize());
        bytes.extend(self);

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let begin = usize::serialized_size();
        let end = begin + len;

        Ok(bytes
            .get(begin..end)
            .ok_or_else(|| DbError::from("Vec<u8> deserialization error: out of bounds"))?
            .to_vec())
    }
}

impl PartialSerialize for Vec<String> {
    fn serialized_size(&self) -> usize {
        let mut size = usize::serialized_size();

        for value in self {
            size += value.serialized_size();
        }

        size
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size());
        bytes.extend(self.len().serialize());

        for value in self {
            bytes.extend(value.serialize());
        }

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let mut begin = usize::serialized_size();
        let mut vec = Self::new();
        vec.reserve(len);

        for _ in 0..len {
            let value = String::deserialize(&bytes[begin..])
                .map_err(|_| DbError::from("Vec<String> deserialization error: out of bounds"))?;
            begin += value.serialized_size();
            vec.push(value);
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string() {
        let original = "This string has 24 bytes".to_string();
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len(), serialized_size);

        bytes.push(0);
        let deserialized = String::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn string_invalid_utf8() {
        let len: usize = 2;
        let mut bytes = len.serialize();
        bytes.push(0xdf);
        bytes.push(0xff);

        assert!(String::deserialize(&bytes).is_err());
    }

    #[test]
    fn string_out_of_bounds() {
        let mut bytes = "This string has 24 bytes".to_string().serialize();
        bytes.pop();

        assert_eq!(
            String::deserialize(&bytes),
            Err(DbError::from("String deserialization error: out of bounds"))
        );
    }

    #[test]
    fn vec_u8() {
        let original = vec![1_u8, 2_u8, 3_u8];
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len(), serialized_size);

        bytes.push(0);
        let deserialized = Vec::<u8>::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn vec_u8_out_of_bounds() {
        let mut bytes = vec![1_u8, 2_u8, 3_u8].serialize();
        bytes.pop();

        assert_eq!(
            Vec::<u8>::deserialize(&bytes),
            Err(DbError::from(
                "Vec<u8> deserialization error: out of bounds"
            ))
        );
    }

    #[test]
    fn vec_u64() {
        let original = vec![1_u64, 2_u64, 3_u64];
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len(), serialized_size);

        bytes.push(0);
        let deserialized = Vec::<u64>::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn vec_u64_out_of_bounds() {
        let mut bytes = vec![1_u64, 2_u64, 3_u64].serialize();
        bytes.pop();

        assert_eq!(
            Vec::<u64>::deserialize(&bytes),
            Err(DbError::from(
                "Vec<u64> deserialization error: out of bounds"
            ))
        );
    }

    #[test]
    fn vec_string() {
        let original = vec!["Hello".to_string(), "World".to_string()];
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len(), serialized_size);
        bytes.push(0);
        let deserialized = Vec::<String>::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn vec_string_out_of_bounds() {
        let mut bytes = vec!["Hello".to_string(), "World".to_string()].serialize();
        bytes.pop();

        assert_eq!(
            Vec::<String>::deserialize(&bytes),
            Err(DbError::from(
                "Vec<String> deserialization error: out of bounds"
            ))
        );
    }
}
