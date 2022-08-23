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
}
