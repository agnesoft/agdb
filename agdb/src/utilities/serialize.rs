use crate::DbError;
use std::any::type_name;

pub trait Serialize: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialized_size(&self) -> u64;
}

pub trait SerializeStatic: Serialize {
    fn serialized_size_static() -> u64 {
        std::mem::size_of::<Self>() as u64
    }
}

impl SerializeStatic for i64 {}
impl SerializeStatic for u64 {}
impl SerializeStatic for f64 {}

impl Serialize for i64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self::from_le_bytes(
            bytes
                .get(0..std::mem::size_of::<Self>())
                .ok_or_else(|| DbError::from("i64 deserialization error: out of bounds"))?
                .try_into()?,
        ))
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl Serialize for u64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self::from_le_bytes(
            bytes
                .get(0..std::mem::size_of::<Self>())
                .ok_or_else(|| DbError::from("u64 deserialization error: out of bounds"))?
                .try_into()?,
        ))
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl Serialize for f64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self::from_le_bytes(
            bytes
                .get(0..std::mem::size_of::<Self>())
                .ok_or_else(|| DbError::from("f64 deserialization error: out of bounds"))?
                .try_into()?,
        ))
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl Serialize for usize {
    fn serialize(&self) -> Vec<u8> {
        (*self as u64).serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let value = u64::deserialize(bytes)?;
        Ok(usize::try_from(value)?)
    }

    fn serialized_size(&self) -> u64 {
        u64::serialized_size_static()
    }
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.len().serialize());
        bytes.extend(self.as_bytes());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let begin = len.serialized_size() as usize;
        let end = begin + len;

        Ok(String::from_utf8(
            bytes
                .get(begin..end)
                .ok_or_else(|| DbError::from("String deserialization error: out of bounds"))?
                .to_vec(),
        )?)
    }

    fn serialized_size(&self) -> u64 {
        self.len().serialized_size() + self.len() as u64
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.len().serialize());

        for value in self {
            bytes.extend(value.serialize());
        }

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let mut begin = len.serialized_size() as usize;
        let mut vec = Self::new();
        vec.reserve(len);

        for _ in 0..len {
            let value = T::deserialize(&bytes[begin..]).map_err(|_| {
                DbError::from(format!(
                    "Vec<{}> deserialization error: out of bounds",
                    type_name::<T>()
                ))
            })?;
            begin += value.serialized_size() as usize;
            vec.push(value);
        }

        Ok(vec)
    }

    fn serialized_size(&self) -> u64 {
        let mut len = self.len().serialized_size();

        for value in self {
            len += value.serialized_size();
        }

        len
    }
}

impl Serialize for Vec<u8> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.len().serialize());
        bytes.extend(self);

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let begin = len.serialized_size() as usize;
        let end = begin + len;

        Ok(bytes
            .get(begin..end)
            .ok_or_else(|| DbError::from("Vec<u8> deserialization error: out of bounds"))?
            .to_vec())
    }

    fn serialized_size(&self) -> u64 {
        self.len().serialized_size() + self.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn i64() {
        let original = -10_i64;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = i64::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn i64_out_of_bounds() {
        assert_eq!(
            i64::deserialize(&Vec::<u8>::new()),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn u64() {
        let original = 10_u64;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = u64::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn u64_out_of_bounds() {
        assert_eq!(
            u64::deserialize(&Vec::<u8>::new()),
            Err(DbError::from("u64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn f64() {
        let original = -PI;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = f64::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn f64_out_of_bounds() {
        assert_eq!(
            f64::deserialize(&Vec::<u8>::new()),
            Err(DbError::from("f64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn usize() {
        let original: usize = 10;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = usize::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn string() {
        let original = "This string has 24 bytes".to_string();
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

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
    fn ar_u8() {
        let original = vec![1_u8, 2_u8, 3_u8];
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = Vec::<u8>::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn ar_u8_out_of_bounds() {
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

        assert_eq!(bytes.len() as u64, serialized_size);

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

        assert_eq!(bytes.len() as u64, serialized_size);
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
            Err(DbError::from(format!(
                "Vec<{}> deserialization error: out of bounds",
                type_name::<String>()
            )))
        );

        let len: usize = 1;
        bytes = len.serialize();

        assert_eq!(
            Vec::<String>::deserialize(&bytes),
            Err(DbError::from(format!(
                "Vec<{}> deserialization error: out of bounds",
                type_name::<String>()
            )))
        );
    }
}
