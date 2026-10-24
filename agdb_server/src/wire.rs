use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::AgdbSerialize;

// Cluster traffic uses agdb's binary serialization instead of JSON, which has
// no byte type and sent binary payloads as arrays of numbers (~3.5x larger).
// The version is part of the content type so a future format change is
// negotiated the same way as binary vs JSON.
pub(crate) const CONTENT_TYPE: &str = "application/vnd.agdb.cluster.v1";

pub(crate) fn encode<T: AgdbSerialize>(value: &T) -> Vec<u8> {
    value.serialize()
}

pub(crate) fn decode<T: AgdbSerialize>(bytes: &[u8]) -> ServerResult<T> {
    let value = T::deserialize(bytes)?;
    let size = value.serialized_size();

    if size != bytes.len() as u64 {
        return Err(ServerError::from(format!(
            "cluster message has {} bytes, expected {size}",
            bytes.len()
        )));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() -> ServerResult<()> {
        let value = vec![("owner".to_string(), Some(7_u64))];
        assert_eq!(
            decode::<Vec<(String, Option<u64>)>>(&encode(&value))?,
            value
        );
        Ok(())
    }

    #[test]
    fn rejects_malformed_input() {
        let mut bytes = encode(&7_u64);
        bytes.push(0);

        assert!(decode::<u64>(&bytes).is_err());
        assert!(decode::<u64>(&bytes[..4]).is_err());
        assert!(decode::<Vec<u64>>(&encode(&u64::MAX)).is_err());
    }
}
