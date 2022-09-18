use crate::db_error::DbError;

pub(crate) trait Serialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialize(&self) -> Vec<u8>;

    fn serialized_size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }
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
        self.to_le_bytes().into()
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
        self.to_le_bytes().into()
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        const SIZE_OFFSET: usize = std::mem::size_of::<u64>();
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
        const SIZE_OFFSET: usize = std::mem::size_of::<u64>();
        let value_offset: usize = std::mem::size_of::<T>();
        let mut bytes: Vec<u8> = vec![];

        bytes.reserve(SIZE_OFFSET + value_offset * self.len());
        bytes.extend((self.len() as u64).serialize());

        for value in self {
            bytes.extend(value.serialize());
        }

        bytes
    }

    fn serialized_size() -> u64 {
        0
    }
}

impl Serialize for Vec<u8> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(bytes.to_vec())
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl Serialize for String {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        String::from_utf8(bytes.to_vec())
    }

    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn serialized_size() -> u64 {
        0
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
    fn serialized_size() {
        assert_eq!(i64::serialized_size(), 8);
        assert_eq!(u64::serialized_size(), 8);
        assert_eq!(Vec::<i64>::serialized_size(), 0);
        assert_eq!(String::serialized_size(), 0);
    }

    #[test]
    fn string() {
        let value = "Hello, World!".to_string();
        let bytes = value.serialize();
        let actual = String::deserialize(&bytes);

        assert_eq!(actual, Ok(value));
    }

    #[test]
    fn string_out_of_bounds() {
        let value = "Hello, World!".to_string();
        let bytes = vec![20_u8; 10];

        assert_eq!(
            String::deserialize(&bytes),
            Err(DbError::from("String deserialization error: out of bounds"))
        );
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
