use crate::Serialize;
use db_error::DbError;

impl<T: Serialize> Serialize for Vec<T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        const SIZE_OFFSET: usize = std::mem::size_of::<u64>();
        let value_offset = T::serialized_size();
        let size = u64::deserialize(bytes)? as usize;
        let mut data: Self = vec![];

        data.reserve(size);

        for i in 0..size {
            let offset = SIZE_OFFSET + value_offset as usize * i;
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
