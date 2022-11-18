use crate::utilities::serialize::Serialize;
use crate::DbError;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string() {
        let original = "This string has 24 bytes".to_string();
        let serialized_size = original.serialized_size();
        let bytes = original.serialize();

        assert_eq!(bytes.len(), serialized_size);

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
}
