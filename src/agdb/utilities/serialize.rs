use crate::db::db_error::DbError;
use std::mem::size_of;

pub trait FixedSize {}

pub trait OldSerialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialize(&self) -> Vec<u8>;
    fn fixed_size() -> u64 {
        size_of::<Self>() as u64
    }
}

impl OldSerialize for f64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let bits = u64::deserialize(bytes)?;
        Ok(f64::from_bits(bits))
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_bits().serialize()
    }
}

impl OldSerialize for i64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let buffer: [u8; size_of::<Self>()] = bytes
            .get(0..size_of::<Self>())
            .ok_or_else(|| DbError::from("i64 deserialization error: out of bounds"))?
            .try_into()
            .unwrap();
        Ok(Self::from_le_bytes(buffer))
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().into()
    }
}

impl OldSerialize for u64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let buffer: [u8; size_of::<Self>()] = bytes
            .get(0..size_of::<Self>())
            .ok_or_else(|| DbError::from("u64 deserialization error: out of bounds"))?
            .try_into()
            .unwrap();
        Ok(Self::from_le_bytes(buffer))
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().into()
    }
}

impl OldSerialize for String {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() <= 16 {
            let size = bytes[0] as usize;
            Ok(String::from_utf8(bytes[1..(size + 1)].to_vec())?)
        } else {
            let size = u64::deserialize(bytes)? as usize;
            Ok(String::from_utf8(
                bytes[u64::fixed_size() as usize..(u64::fixed_size() as usize + size)].to_vec(),
            )?)
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        if self.len() < 16 {
            bytes.reserve(1 + self.len());
            bytes.push(self.len() as u8);
            bytes.extend(self.as_bytes());
        } else {
            bytes.reserve(u64::fixed_size() as usize + self.len());
            bytes.extend((self.len() as u64).serialize());
            bytes.extend(self.as_bytes());
        }

        bytes
    }

    fn fixed_size() -> u64 {
        0
    }
}

struct VecSerializer;

impl VecSerializer {
    fn deserialize_dynamic<T: OldSerialize>(bytes: &[u8]) -> Result<Vec<T>, DbError> {
        const LEN_OFFSET: usize = size_of::<u64>();
        let len = u64::deserialize(bytes)? as usize;
        let mut data = Vec::<T>::new();
        data.reserve(len);
        let mut offset = LEN_OFFSET;

        for _i in 0..len {
            let value_len = u64::deserialize(&bytes[offset..(offset + LEN_OFFSET)])? as usize;
            offset += LEN_OFFSET;
            let value = T::deserialize(&bytes[offset..(offset + value_len)])?;
            offset += value_len;
            data.push(value);
        }

        Ok(data)
    }

    fn serialize_dynamic<T: OldSerialize>(vec: &Vec<T>) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        bytes.extend((vec.len() as u64).serialize());

        for value in vec {
            let serialized_value = value.serialize();
            bytes.extend((serialized_value.len() as u64).serialize());
            bytes.extend(serialized_value);
        }

        bytes
    }

    fn deserialize_fixed<T: OldSerialize>(bytes: &[u8]) -> Result<Vec<T>, DbError> {
        const LEN_OFFSET: usize = size_of::<u64>();
        let len = u64::deserialize(bytes)? as usize;
        let mut data = Vec::<T>::new();
        data.reserve(len);

        let value_offset = T::fixed_size();

        for i in 0..len {
            let offset = LEN_OFFSET + value_offset as usize * i;
            data.push(T::deserialize(&bytes[offset..])?);
        }

        Ok(data)
    }

    fn serialize_fixed<T: OldSerialize>(vec: &Vec<T>) -> Vec<u8> {
        const LEN_OFFSET: usize = size_of::<u64>();
        let mut bytes = Vec::<u8>::new();

        let value_offset = T::fixed_size();
        bytes.reserve(LEN_OFFSET + (value_offset as usize) * vec.len());
        bytes.extend((vec.len() as u64).serialize());

        for value in vec {
            bytes.extend(value.serialize());
        }

        bytes
    }
}

impl<T> OldSerialize for Vec<T>
where
    T: OldSerialize,
{
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if T::fixed_size() == 0 {
            return VecSerializer::deserialize_dynamic(bytes);
        }

        VecSerializer::deserialize_fixed(bytes)
    }

    fn serialize(&self) -> Vec<u8> {
        if T::fixed_size() == 0 {
            return VecSerializer::serialize_dynamic(self);
        }

        VecSerializer::serialize_fixed(self)
    }

    fn fixed_size() -> u64 {
        0
    }
}

impl OldSerialize for Vec<u8> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(bytes.to_vec())
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn fixed_size() -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn fixed_size() {
        assert_eq!(i64::fixed_size(), 8);
        assert_eq!(u64::fixed_size(), 8);
        assert_eq!(Vec::<i64>::fixed_size(), 0);
        assert_eq!(String::fixed_size(), 0);
    }

    #[test]
    fn f64() {
        let f = -3.333_f64;
        let bytes = f.serialize();
        let actual = f64::deserialize(&bytes).unwrap();

        assert_eq!(f.total_cmp(&actual), Ordering::Equal);

        let nan = f64::NAN;
        let bytes = nan.serialize();
        let actual_nan = f64::deserialize(&bytes).unwrap();

        assert_eq!(nan.total_cmp(&actual_nan), Ordering::Equal);
    }

    #[test]
    fn i64() {
        let number = -10_i64;
        let bytes = number.serialize();
        let actual = i64::deserialize(&bytes);

        assert_eq!(actual, Ok(number));
    }

    #[test]
    fn i64_out_of_bounds() {
        let bytes = vec![0_u8; 4];

        assert_eq!(
            i64::deserialize(&bytes),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn small_string_optimization() {
        let value = "Hello, World!".to_string();
        let bytes = value.serialize();

        assert_eq!(bytes.len(), 14);

        let actual = String::deserialize(&bytes);

        assert_eq!(actual, Ok(value));
    }

    #[test]
    fn string() {
        let value = "Hello, World! This string is not short. No sir!".to_string();
        let bytes = value.serialize();

        assert_eq!(bytes.len(), 8 + value.len());

        let actual = String::deserialize(&bytes);

        assert_eq!(actual, Ok(value));
    }

    #[test]
    fn string_bad_bytes() {
        let bad_bytes = vec![2_u8, 0xdf, 0xff];

        assert!(String::deserialize(&bad_bytes).is_err());
    }

    #[test]
    fn u64() {
        let number = 10_u64;
        let bytes = number.serialize();
        let actual = u64::deserialize(&bytes);

        assert_eq!(actual, Ok(number));
    }

    #[test]
    fn u64_out_of_bounds() {
        let bytes = vec![0_u8; 4];

        assert_eq!(
            u64::deserialize(&bytes),
            Err(DbError::from("u64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn vec_i64() {
        let data = vec![1_i64, 2_i64, 3_i64];
        let bytes = data.serialize();
        let actual = Vec::<i64>::deserialize(&bytes);

        assert_eq!(actual, Ok(data));
    }

    #[test]
    fn vec_size_out_of_bounds() {
        let bytes = vec![0_u8; 4];

        assert_eq!(
            Vec::<i64>::deserialize(&bytes),
            Err(DbError::from("u64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn vec_u8() {
        let data = vec![1_u8, 2_u8, 3_u8];
        let bytes = data.serialize();
        let actual = Vec::<u8>::deserialize(&bytes);

        assert_eq!(Vec::<u8>::fixed_size(), 0);
        assert_eq!(actual, Ok(data));
    }

    #[test]
    fn vec_value_out_of_bounds() {
        let bytes = 1_u64.serialize();

        assert_eq!(
            Vec::<i64>::deserialize(&bytes),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }
}
