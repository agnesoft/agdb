use super::db_error::DbError;
use super::db_float::DbFloat;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DbValue {
    Bytes(Vec<u8>),
    Int(i64),
    Uint(u64),
    Float(DbFloat),
    String(String),
    VecInt(Vec<i64>),
    VecUint(Vec<u64>),
    VecFloat(Vec<DbFloat>),
    VecString(Vec<String>),
}

const BYTES_META_VALUE: u8 = 0_u8;
const INT_META_VALUE: u8 = 1_u8;
const UINT_META_VALUE: u8 = 2_u8;
const FLOAT_META_VALUE: u8 = 3_u8;
const STRING_META_VALUE: u8 = 4_u8;
const VEC_INT_META_VALUE: u8 = 5_u8;
const VEC_UINT_META_VALUE: u8 = 6_u8;
const VEC_FLOAT_META_VALUE: u8 = 7_u8;
const VEC_STRING_META_VALUE: u8 = 8_u8;

impl From<f32> for DbValue {
    fn from(value: f32) -> Self {
        DbValue::Float(value.into())
    }
}

impl From<f64> for DbValue {
    fn from(value: f64) -> Self {
        DbValue::Float(value.into())
    }
}

impl From<DbFloat> for DbValue {
    fn from(value: DbFloat) -> Self {
        DbValue::Float(value)
    }
}

impl From<i32> for DbValue {
    fn from(value: i32) -> Self {
        DbValue::Int(value.into())
    }
}

impl From<i64> for DbValue {
    fn from(value: i64) -> Self {
        DbValue::Int(value)
    }
}

impl From<u32> for DbValue {
    fn from(value: u32) -> Self {
        DbValue::Uint(value.into())
    }
}

impl From<u64> for DbValue {
    fn from(value: u64) -> Self {
        DbValue::Uint(value)
    }
}

impl From<String> for DbValue {
    fn from(value: String) -> Self {
        DbValue::String(value)
    }
}

impl From<&str> for DbValue {
    fn from(value: &str) -> Self {
        DbValue::String(value.to_string())
    }
}

impl From<Vec<f32>> for DbValue {
    fn from(value: Vec<f32>) -> Self {
        DbValue::VecFloat(value.iter().map(|i| (*i).into()).collect())
    }
}

impl From<Vec<f64>> for DbValue {
    fn from(value: Vec<f64>) -> Self {
        DbValue::VecFloat(value.iter().map(|i| (*i).into()).collect())
    }
}

impl From<Vec<DbFloat>> for DbValue {
    fn from(value: Vec<DbFloat>) -> Self {
        DbValue::VecFloat(value)
    }
}

impl From<Vec<i32>> for DbValue {
    fn from(value: Vec<i32>) -> Self {
        DbValue::VecInt(value.iter().map(|i| *i as i64).collect())
    }
}

impl From<Vec<i64>> for DbValue {
    fn from(value: Vec<i64>) -> Self {
        DbValue::VecInt(value)
    }
}

impl From<Vec<u32>> for DbValue {
    fn from(value: Vec<u32>) -> Self {
        DbValue::VecUint(value.iter().map(|i| *i as u64).collect())
    }
}

impl From<Vec<u64>> for DbValue {
    fn from(value: Vec<u64>) -> Self {
        DbValue::VecUint(value)
    }
}

impl From<Vec<String>> for DbValue {
    fn from(value: Vec<String>) -> Self {
        DbValue::VecString(value)
    }
}

impl From<Vec<&str>> for DbValue {
    fn from(value: Vec<&str>) -> Self {
        DbValue::VecString(value.iter().map(|s| s.to_string()).collect())
    }
}

impl From<Vec<u8>> for DbValue {
    fn from(value: Vec<u8>) -> Self {
        DbValue::Bytes(value)
    }
}

impl Serialize for DbValue {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.is_empty() {
            return Err(DbError::from("DbValue deserialization error: no data"));
        }

        match bytes[0] {
            BYTES_META_VALUE => Ok(DbValue::from(Vec::<u8>::deserialize(&bytes[1..])?)),
            INT_META_VALUE => Ok(DbValue::from(i64::deserialize(&bytes[8..])?)),
            UINT_META_VALUE => Ok(DbValue::from(u64::deserialize(&bytes[8..])?)),
            FLOAT_META_VALUE => Ok(DbValue::from(DbFloat::deserialize(&bytes[8..])?)),
            STRING_META_VALUE => Ok(DbValue::from(String::deserialize(&bytes[1..])?)),
            VEC_INT_META_VALUE => Ok(DbValue::from(Vec::<i64>::deserialize(&bytes[1..])?)),
            VEC_UINT_META_VALUE => Ok(DbValue::from(Vec::<u64>::deserialize(&bytes[1..])?)),
            VEC_FLOAT_META_VALUE => Ok(DbValue::from(Vec::<DbFloat>::deserialize(&bytes[1..])?)),
            VEC_STRING_META_VALUE => Ok(DbValue::from(Vec::<String>::deserialize(&bytes[1..])?)),
            _ => Err(DbError::from("DbValue deserialization error: invalid data")),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        match self {
            DbValue::Bytes(value) => {
                bytes.reserve(1 + value.serialized_size() as usize);
                bytes.push(BYTES_META_VALUE);
                bytes.extend(value.serialize());
            }
            DbValue::Int(value) => {
                bytes.resize(16, 0_u8);
                bytes[0] = INT_META_VALUE;
                bytes[8..].copy_from_slice(&value.serialize());
            }
            DbValue::Uint(value) => {
                bytes.resize(16, 0_u8);
                bytes[0] = UINT_META_VALUE;
                bytes[8..].copy_from_slice(&value.serialize());
            }
            DbValue::Float(value) => {
                bytes.resize(16, 0_u8);
                bytes[0] = FLOAT_META_VALUE;
                bytes[8..].copy_from_slice(&value.serialize());
            }
            DbValue::String(value) => {
                bytes.reserve(1 + value.serialized_size() as usize);
                bytes.push(STRING_META_VALUE);
                bytes.extend(&value.serialize());
            }
            DbValue::VecInt(value) => {
                bytes.reserve(1 + value.serialized_size() as usize);
                bytes.push(VEC_INT_META_VALUE);
                bytes.extend(&value.serialize());
            }
            DbValue::VecUint(value) => {
                bytes.reserve(1 + value.serialized_size() as usize);
                bytes.push(VEC_UINT_META_VALUE);
                bytes.extend(&value.serialize());
            }
            DbValue::VecFloat(value) => {
                bytes.reserve(1 + value.serialized_size() as usize);
                bytes.push(VEC_FLOAT_META_VALUE);
                bytes.extend(&value.serialize());
            }
            DbValue::VecString(value) => {
                bytes.push(VEC_STRING_META_VALUE);
                bytes.extend(&value.serialize());
            }
        }

        bytes
    }

    fn serialized_size(&self) -> u64 {
        match self {
            DbValue::Bytes(value) => 1 + value.serialized_size(),
            DbValue::Int(value) => 8 + value.serialized_size(),
            DbValue::Uint(value) => 8 + value.serialized_size(),
            DbValue::Float(value) => 8 + value.serialized_size(),
            DbValue::String(value) => 1 + value.serialized_size(),
            DbValue::VecInt(value) => 1 + value.serialized_size(),
            DbValue::VecUint(value) => 1 + value.serialized_size(),
            DbValue::VecFloat(value) => 1 + value.serialized_size(),
            DbValue::VecString(value) => 1 + value.serialized_size(),
        }
    }
}

impl StableHash for DbValue {
    fn stable_hash(&self) -> u64 {
        match self {
            DbValue::Bytes(value) => value.stable_hash(),
            DbValue::Int(value) => value.stable_hash(),
            DbValue::Uint(value) => value.stable_hash(),
            DbValue::Float(value) => value.stable_hash(),
            DbValue::String(value) => value.stable_hash(),
            DbValue::VecInt(value) => value.stable_hash(),
            DbValue::VecUint(value) => value.stable_hash(),
            DbValue::VecFloat(value) => value.stable_hash(),
            DbValue::VecString(value) => value.stable_hash(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    use std::collections::HashSet;

    #[test]
    fn derived_from_eq() {
        let mut map = HashSet::<DbValue>::new();
        map.insert(DbValue::from(1));
    }

    #[test]
    fn derived_from_debug() {
        format!("{:?}", DbValue::from(""));
    }

    #[test]
    fn derived_from_hash() {
        let mut map = HashSet::<DbValue>::new();
        map.insert(DbValue::from(1.0_f64));
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DbValue::from("").cmp(&DbValue::from("")), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut vec = vec![
            DbValue::from(1.1_f64),
            DbValue::from(1.3_f64),
            DbValue::from(-3.333_f64),
        ];
        vec.sort();

        assert_eq!(
            vec,
            vec![
                DbValue::from(-3.333_f64),
                DbValue::from(1.1_f64),
                DbValue::from(1.3_f64)
            ]
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(DbValue::from(vec![1_u8]), DbValue::from(vec![1_u8]));
        assert_eq!(DbValue::from(1.0_f64), DbValue::from(1.0_f64));
        assert_eq!(DbValue::from(1_i64), DbValue::from(1_i64));
        assert_eq!(DbValue::from(1_u64), DbValue::from(1_u64));
        assert_eq!(
            DbValue::from("Hello".to_string()),
            DbValue::from("Hello".to_string())
        );
        assert_eq!(DbValue::from(vec![1.0_f64]), DbValue::from(vec![1.0_f64]));
        assert_eq!(DbValue::from(vec![1_i64]), DbValue::from(vec![1_i64]));
        assert_eq!(DbValue::from(vec![1_u64]), DbValue::from(vec![1_u64]));
        assert_eq!(
            DbValue::from(vec!["Hello".to_string()]),
            DbValue::from(vec!["Hello".to_string()])
        );
    }

    #[test]
    fn from() {
        assert!(matches!(
            DbValue::from(Vec::<u8>::new()),
            DbValue::Bytes { .. }
        ));
        assert!(matches!(DbValue::from(1_i32), DbValue::Int { .. }));
        assert!(matches!(DbValue::from(1_i64), DbValue::Int { .. }));
        assert!(matches!(DbValue::from(1_u32), DbValue::Uint { .. }));
        assert!(matches!(DbValue::from(1_u64), DbValue::Uint { .. }));
        assert!(matches!(DbValue::from(1.0_f32), DbValue::Float { .. }));
        assert!(matches!(DbValue::from(1.0_f64), DbValue::Float { .. }));
        assert!(matches!(
            DbValue::from(DbFloat::from(1.0_f64)),
            DbValue::Float { .. }
        ));
        assert!(matches!(DbValue::from(""), DbValue::String { .. }));
        assert!(matches!(
            DbValue::from(String::new()),
            DbValue::String { .. }
        ));
        assert!(matches!(DbValue::from(vec![1_i32]), DbValue::VecInt { .. }));
        assert!(matches!(DbValue::from(vec![1_i64]), DbValue::VecInt { .. }));
        assert!(matches!(
            DbValue::from(vec![1_u32]),
            DbValue::VecUint { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![1_u64]),
            DbValue::VecUint { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![1.0_f32]),
            DbValue::VecFloat { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![1.0_f64]),
            DbValue::VecFloat { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![DbFloat::from(1.0_f64)]),
            DbValue::VecFloat { .. }
        ));
        assert!(matches!(DbValue::from(vec![""]), DbValue::VecString { .. }));
        assert!(matches!(
            DbValue::from(Vec::<String>::new()),
            DbValue::VecString { .. }
        ));
    }

    #[test]
    fn serialize_bytes() {
        let value = DbValue::from(vec![0_u8, 1_u8, 2_u8, 3_u8, 4_u8]);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_i64() {
        let value = DbValue::from(1_i64);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_i64_max() {
        let value = DbValue::from(i64::MAX);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_i64_min() {
        let value = DbValue::from(i64::MIN);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_i64_negative() {
        let value = DbValue::from(-1_i64);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_u64() {
        let value = DbValue::from(u64::MAX / 2);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_db_float() {
        let value = DbValue::from(0.1_f64 + 0.2_f64);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_string() {
        let value = DbValue::from("Hello, World!");
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn serialize_vec_int() {
        let value = DbValue::from(vec![i64::MAX, 0_i64, i64::MIN]);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let actual = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, actual);
    }

    #[test]
    fn serialize_vec_uint() {
        let value = DbValue::from(vec![u64::MAX, 0_u64, u64::MIN]);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let actual = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, actual);
    }

    #[test]
    fn serialize_vec_float() {
        let value = DbValue::from(vec![0.1_f64 + 0.2_f64, -0.0_f64, std::f64::consts::PI]);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let actual = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, actual);
    }

    #[test]
    fn serialize_vec_string() {
        let value = DbValue::from(vec!["Hello", ", ", "World", "!"]);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let actual = DbValue::deserialize(&bytes).unwrap();

        assert_eq!(value, actual);
    }

    #[test]
    fn serialize_no_data() {
        let bytes = Vec::<u8>::new();

        assert_eq!(
            DbValue::deserialize(&bytes),
            Err(DbError::from("DbValue deserialization error: no data"))
        );
    }

    #[test]
    fn serialize_invalid_data() {
        let bytes = vec![255_u8];

        assert_eq!(
            DbValue::deserialize(&bytes),
            Err(DbError::from("DbValue deserialization error: invalid data"))
        );
    }

    #[test]
    fn stable_hash() {
        assert_ne!(DbValue::from(vec![1_u8]).stable_hash(), 0);
        assert_ne!(DbValue::from(1.0_f64).stable_hash(), 0);
        assert_ne!(DbValue::from(1_i64).stable_hash(), 0);
        assert_ne!(DbValue::from(1_u64).stable_hash(), 0);
        assert_ne!(DbValue::from(" ").stable_hash(), 0);
        assert_ne!(DbValue::from(vec![1_i64]).stable_hash(), 0);
        assert_ne!(DbValue::from(vec![1_u64]).stable_hash(), 0);
        assert_ne!(DbValue::from(vec![1.0_f64]).stable_hash(), 0);
        assert_ne!(DbValue::from(vec![""]).stable_hash(), 0);
    }
}
