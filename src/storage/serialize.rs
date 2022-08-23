pub(crate) trait Serialize {
    fn deserialize(bytes: Vec<u8>) -> Self;
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for i64 {
    fn deserialize(bytes: Vec<u8>) -> Self {
        i64::from_le_bytes(bytes.try_into().unwrap())
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Serialize for u64 {
    fn deserialize(bytes: Vec<u8>) -> Self {
        u64::from_le_bytes(bytes.try_into().unwrap())
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
        let actual = i64::deserialize(bytes);

        assert_eq!(actual, number)
    }

    #[test]
    fn u64() {
        let number = 10_u64;
        let bytes = number.serialize();
        let actual = u64::deserialize(bytes);

        assert_eq!(actual, number)
    }
}
