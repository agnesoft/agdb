use crate::db_error::DbError;
use std::mem::size_of;

pub(crate) trait Serialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for i64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(i64::from_le_bytes(bytes.try_into().map_err(|_| {
            DbError::Storage("i64 deserialization error: out of bounds".to_string())
        })?))
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(u64::from_le_bytes(bytes.try_into().map_err(|_| {
            DbError::Storage("u64 deserialization error: out of bounds".to_string())
        })?))
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        const SIZE_OFFSET: usize = size_of::<usize>();
        let value_offset = size_of::<T>();
        let size = u64::deserialize(bytes.get(0..SIZE_OFFSET).ok_or_else(|| {
            DbError::Storage("Vec deserialization error: size out of bounds".to_string())
        })?)? as usize;
        let mut data: Self = vec![];

        data.reserve(size);

        for i in 0..size {
            let offset = SIZE_OFFSET + value_offset * i;
            let end = offset + value_offset;
            data.push(T::deserialize(bytes.get(offset..end).ok_or_else(
                || DbError::Storage("Vec deserialization error: value out of bounds".to_string()),
            )?)?);
        }

        Ok(data)
    }

    fn serialize(&self) -> Vec<u8> {
        const SIZE_OFFSET: usize = size_of::<usize>();
        let value_offset: usize = size_of::<T>();
        let mut bytes: Vec<u8> = vec![];

        bytes.reserve(SIZE_OFFSET + value_offset * self.len());
        bytes.extend((self.len() as u64).serialize());

        for value in self {
            bytes.extend(value.serialize());
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64() {
        let number = -10_i64;
        let bytes = number.serialize();
        let actual = i64::deserialize(&bytes);

        assert_eq!(actual, Ok(number))
    }

    #[test]
    fn i64_out_of_bounds() {
        let bytes = vec![0_u8; 4];

        assert_eq!(
            i64::deserialize(&bytes),
            Err(DbError::Storage(
                "i64 deserialization error: out of bounds".to_string()
            ))
        );
    }

    #[test]
    fn u64() {
        let number = 10_u64;
        let bytes = number.serialize();
        let actual = u64::deserialize(&bytes);

        assert_eq!(actual, Ok(number))
    }

    #[test]
    fn u64_out_of_bounds() {
        let bytes = vec![0_u8; 4];

        assert_eq!(
            u64::deserialize(&bytes),
            Err(DbError::Storage(
                "u64 deserialization error: out of bounds".to_string()
            ))
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
            Err(DbError::Storage(
                "Vec deserialization error: size out of bounds".to_string()
            ))
        );
    }

    #[test]
    fn vec_value_out_of_bounds() {
        let bytes = 1_u64.serialize();

        assert_eq!(
            Vec::<i64>::deserialize(&bytes),
            Err(DbError::Storage(
                "Vec deserialization error: value out of bounds".to_string()
            ))
        );
    }
}
