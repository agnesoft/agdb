use crate::db_error::DbError;

pub(crate) trait Serialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for i64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let buffer: [u8; std::mem::size_of::<Self>()] = bytes
            .get(0..std::mem::size_of::<Self>())
            .ok_or_else(|| DbError::from("i64 deserialization error: out of bounds"))?
            .try_into()
            .unwrap();
        Ok(Self::from_le_bytes(buffer))
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u64 {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let buffer: [u8; std::mem::size_of::<Self>()] = bytes
            .get(0..std::mem::size_of::<Self>())
            .ok_or_else(|| DbError::from("u64 deserialization error: out of bounds"))?
            .try_into()
            .unwrap();
        Ok(Self::from_le_bytes(buffer))
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        const SIZE_OFFSET: usize = std::mem::size_of::<usize>();
        let value_offset = std::mem::size_of::<T>();
        let size = u64::deserialize(bytes)? as usize;
        let mut data: Self = vec![];

        data.reserve(size);

        for i in 0..size {
            let offset = SIZE_OFFSET + value_offset * i;
            data.push(T::deserialize(&bytes[offset..])?);
        }

        Ok(data)
    }

    fn serialize(&self) -> Vec<u8> {
        const SIZE_OFFSET: usize = std::mem::size_of::<usize>();
        let value_offset: usize = std::mem::size_of::<T>();
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
            Err(DbError::from("i64 deserialization error: out of bounds"))
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
    fn vec_value_out_of_bounds() {
        let bytes = 1_u64.serialize();

        assert_eq!(
            Vec::<i64>::deserialize(&bytes),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }
}
