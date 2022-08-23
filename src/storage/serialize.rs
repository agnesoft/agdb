use std::mem::size_of;

pub(crate) trait Serialize {
    fn deserialize(bytes: &[u8]) -> Self;
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for i64 {
    fn deserialize(bytes: &[u8]) -> Self {
        const END: usize = size_of::<i64>();
        i64::from_le_bytes((&bytes[0..END]).try_into().unwrap())
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u64 {
    fn deserialize(bytes: &[u8]) -> Self {
        const END: usize = size_of::<u64>();
        u64::from_le_bytes((&bytes[0..END]).try_into().unwrap())
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn deserialize(bytes: &[u8]) -> Self {
        const SIZE_OFFSET: usize = size_of::<usize>();
        let value_offset = size_of::<T>();
        let size = u64::deserialize(&bytes[0..SIZE_OFFSET]) as usize;
        let mut data: Self = vec![];

        data.reserve(size);

        for i in 0..(size) {
            let offset = SIZE_OFFSET + value_offset * i;
            let end = offset + value_offset;
            data.push(T::deserialize(&bytes[offset..end]));
        }

        data
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

        assert_eq!(actual, number)
    }

    #[test]
    fn u64() {
        let number = 10_u64;
        let bytes = number.serialize();
        let actual = u64::deserialize(&bytes);

        assert_eq!(actual, number)
    }

    #[test]
    fn vec_i64() {
        let data = vec![1_i64, 2_i64, 3_i64];
        let bytes = data.serialize();
        let actual = Vec::<i64>::deserialize(&bytes);

        assert_eq!(actual, data);
    }
}
