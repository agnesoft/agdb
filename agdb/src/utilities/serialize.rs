use crate::DbError;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub trait Serialize: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialized_size(&self) -> u64;
}

pub trait SerializeStatic: Serialize {
    fn serialized_size_static() -> u64 {
        std::mem::size_of::<Self>() as u64
    }
}

impl SerializeStatic for i64 {}
impl SerializeStatic for u64 {}
impl SerializeStatic for f64 {}

impl Serialize for i64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self::from_le_bytes(
            bytes
                .get(0..std::mem::size_of::<Self>())
                .ok_or_else(|| DbError::from("i64 deserialization error: out of bounds"))?
                .try_into()?,
        ))
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl Serialize for u64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self::from_le_bytes(
            bytes
                .get(0..std::mem::size_of::<Self>())
                .ok_or_else(|| DbError::from("u64 deserialization error: out of bounds"))?
                .try_into()?,
        ))
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl Serialize for f64 {
    fn serialize(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self::from_le_bytes(
            bytes
                .get(0..std::mem::size_of::<Self>())
                .ok_or_else(|| DbError::from("f64 deserialization error: out of bounds"))?
                .try_into()?,
        ))
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl Serialize for usize {
    fn serialize(&self) -> Vec<u8> {
        (*self as u64).serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let value = u64::deserialize(bytes)?;
        Ok(usize::try_from(value)?)
    }

    fn serialized_size(&self) -> u64 {
        u64::serialized_size_static()
    }
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_size() as usize);
        bytes.extend(self.len().serialize());
        bytes.extend(self.as_bytes());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let begin = len.serialized_size() as usize;
        let end = begin + len;

        Ok(String::from_utf8(
            bytes
                .get(begin..end)
                .ok_or_else(|| DbError::from("String deserialization error: out of bounds"))?
                .to_vec(),
        )?)
    }

    fn serialized_size(&self) -> u64 {
        self.len().serialized_size() + self.len() as u64
    }
}

impl Serialize for bool {
    fn serialize(&self) -> Vec<u8> {
        vec![*self as u8]
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        bytes
            .first()
            .ok_or(DbError::from("bool deserialization error: out of bounds"))
            .map(|&b| b != 0)
    }

    fn serialized_size(&self) -> u64 {
        1
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_size() as usize);
        bytes.extend(self.len().serialize());

        for value in self {
            bytes.extend(value.serialize());
        }

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let mut begin = len.serialized_size() as usize;
        let mut vec = Self::with_capacity(len);

        for _ in 0..len {
            let value = T::deserialize(&bytes[begin..]).map_err(|_| {
                DbError::from(format!(
                    "Vec<{}> deserialization error: out of bounds",
                    std::any::type_name::<T>()
                ))
            })?;
            begin += value.serialized_size() as usize;
            vec.push(value);
        }

        Ok(vec)
    }

    fn serialized_size(&self) -> u64 {
        let mut len = self.len().serialized_size();

        for value in self {
            len += value.serialized_size();
        }

        len
    }
}

impl Serialize for Vec<u8> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_size() as usize);
        bytes.extend(self.len().serialize());
        bytes.extend(self);

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let len = usize::deserialize(bytes)?;
        let begin = len.serialized_size() as usize;
        let end = begin + len;

        Ok(bytes
            .get(begin..end)
            .ok_or_else(|| DbError::from("Vec<u8> deserialization error: out of bounds"))?
            .to_vec())
    }

    fn serialized_size(&self) -> u64 {
        self.len().serialized_size() + self.len() as u64
    }
}

impl Serialize for PathBuf {
    fn serialize(&self) -> Vec<u8> {
        self.to_string_lossy().to_string().serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let s = String::deserialize(bytes)?;
        Ok(PathBuf::from(s))
    }

    fn serialized_size(&self) -> u64 {
        self.to_string_lossy().to_string().serialized_size()
    }
}

impl Serialize for SystemTime {
    fn serialize(&self) -> Vec<u8> {
        let (duration, before_epoch) = match self.duration_since(UNIX_EPOCH) {
            Ok(duration) => (duration, false),
            Err(duration) => (duration.duration(), true),
        };
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        let mut bytes = [0_u8; 13];
        bytes[0..8].copy_from_slice(&secs.to_le_bytes());
        bytes[8..12].copy_from_slice(&nanos.to_le_bytes());
        bytes[12] = if before_epoch { 0_u8 } else { 1_u8 };
        bytes.to_vec()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() < 13 {
            return Err(DbError::from(format!(
                "Invalid SystemTime bytes length (should be at least 13): {}",
                bytes.len()
            )));
        }
        let mut secs_bytes = [0_u8; 8];
        secs_bytes.copy_from_slice(&bytes[0..8]);
        let mut nanos_bytes = [0_u8; 4];
        nanos_bytes.copy_from_slice(&bytes[8..12]);
        let before_epoch = bytes[12] == 0_u8;
        let secs = u64::from_le_bytes(secs_bytes);
        let nanos = u32::from_le_bytes(nanos_bytes);
        let duration = Duration::new(secs, nanos);

        if before_epoch {
            Ok(UNIX_EPOCH.checked_sub(duration).ok_or_else(|| {
                DbError::from("SystemTime before UNIX_EPOCH is too far in the past")
            })?)
        } else {
            Ok(UNIX_EPOCH.checked_add(duration).ok_or_else(|| {
                DbError::from("SystemTime after UNIX_EPOCH is too far in the future")
            })?)
        }
    }

    fn serialized_size(&self) -> u64 {
        13
    }
}

impl Serialize for SocketAddr {
    fn serialize(&self) -> Vec<u8> {
        self.to_string().serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let s = String::deserialize(bytes)?;
        s.parse()
            .map_err(|e| DbError::from(format!("Cannot convert string to SocketAddr: {e}")))
    }

    fn serialized_size(&self) -> u64 {
        self.to_string().serialized_size()
    }
}

impl Serialize for IpAddr {
    fn serialize(&self) -> Vec<u8> {
        self.to_string().serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let s = String::deserialize(bytes)?;
        s.parse()
            .map_err(|e| DbError::from(format!("Cannot convert string to IpAddr: {e}")))
    }

    fn serialized_size(&self) -> u64 {
        self.to_string().serialized_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64() {
        let original = -10_i64;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = i64::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn i64_out_of_bounds() {
        assert_eq!(
            i64::deserialize(&Vec::<u8>::new()),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn u64() {
        let original = 10_u64;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = u64::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn u64_out_of_bounds() {
        assert_eq!(
            u64::deserialize(&Vec::<u8>::new()),
            Err(DbError::from("u64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn f64() {
        use std::f64::consts::PI;

        let original = -PI;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = f64::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn f64_out_of_bounds() {
        assert_eq!(
            f64::deserialize(&Vec::<u8>::new()),
            Err(DbError::from("f64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn usize() {
        let original: usize = 10;
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = usize::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn string() {
        let original = "This string has 24 bytes".to_string();
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
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

    #[test]
    fn ar_u8() {
        let original = vec![1_u8, 2_u8, 3_u8];
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = Vec::<u8>::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn ar_u8_out_of_bounds() {
        let mut bytes = vec![1_u8, 2_u8, 3_u8].serialize();
        bytes.pop();

        assert_eq!(
            Vec::<u8>::deserialize(&bytes),
            Err(DbError::from(
                "Vec<u8> deserialization error: out of bounds"
            ))
        );
    }

    #[test]
    fn vec_u64() {
        let original = vec![1_u64, 2_u64, 3_u64];
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        bytes.push(0);
        let deserialized = Vec::<u64>::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn vec_u64_out_of_bounds() {
        let mut bytes = vec![1_u64, 2_u64, 3_u64].serialize();
        bytes.pop();

        assert_eq!(
            Vec::<u64>::deserialize(&bytes),
            Err(DbError::from(
                "Vec<u64> deserialization error: out of bounds"
            ))
        );
    }

    #[test]
    fn vec_string() {
        let original = vec!["Hello".to_string(), "World".to_string()];
        let serialized_size = original.serialized_size();
        let mut bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);
        bytes.push(0);
        let deserialized = Vec::<String>::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn vec_string_out_of_bounds() {
        let mut bytes = vec!["Hello".to_string(), "World".to_string()].serialize();
        bytes.pop();

        assert_eq!(
            Vec::<String>::deserialize(&bytes),
            Err(DbError::from(format!(
                "Vec<{}> deserialization error: out of bounds",
                std::any::type_name::<String>()
            )))
        );

        let len: usize = 1;
        bytes = len.serialize();

        assert_eq!(
            Vec::<String>::deserialize(&bytes),
            Err(DbError::from(format!(
                "Vec<{}> deserialization error: out of bounds",
                std::any::type_name::<String>()
            )))
        );
    }

    #[test]
    fn path_buf() {
        let original = PathBuf::from("/some/test/path");
        let serialized_size = original.serialized_size();
        let bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        let deserialized = PathBuf::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn system_time() {
        let original = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let serialized_size = original.serialized_size();
        let bytes = original.serialize();

        assert_eq!(bytes.len() as u64, serialized_size);

        let deserialized = SystemTime::deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn socket_address() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let bytes = addr.serialize();
        let deserialized = SocketAddr::deserialize(&bytes).unwrap();

        assert_eq!(addr, deserialized);
    }

    #[test]
    fn ip_address() {
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        let bytes = addr.serialize();
        let deserialized = IpAddr::deserialize(&bytes).unwrap();

        assert_eq!(addr, deserialized);
    }
}
