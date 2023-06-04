use super::db_error::DbError;
use super::db_float::DbFloat;
use super::db_value_index::DbValueIndex;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as DisplayResult;

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

pub(crate) const BYTES_META_VALUE: u8 = 1_u8;
pub(crate) const INT_META_VALUE: u8 = 2_u8;
pub(crate) const UINT_META_VALUE: u8 = 3_u8;
pub(crate) const FLOAT_META_VALUE: u8 = 4_u8;
pub(crate) const STRING_META_VALUE: u8 = 5_u8;
pub(crate) const VEC_INT_META_VALUE: u8 = 6_u8;
pub(crate) const VEC_UINT_META_VALUE: u8 = 7_u8;
pub(crate) const VEC_FLOAT_META_VALUE: u8 = 8_u8;
pub(crate) const VEC_STRING_META_VALUE: u8 = 9_u8;

impl DbValue {
    pub(crate) fn load_db_value<S: Storage>(
        value_index: DbValueIndex,
        storage: &S,
    ) -> Result<DbValue, DbError> {
        Ok(match value_index.get_type() {
            BYTES_META_VALUE => {
                if value_index.is_value() {
                    DbValue::Bytes(value_index.value().to_vec())
                } else {
                    DbValue::Bytes(storage.value_as_bytes(StorageIndex(value_index.index()))?)
                }
            }
            INT_META_VALUE => {
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(value_index.value());
                DbValue::Int(i64::from_le_bytes(bytes))
            }
            UINT_META_VALUE => {
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(value_index.value());
                DbValue::Uint(u64::from_le_bytes(bytes))
            }
            FLOAT_META_VALUE => {
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(value_index.value());
                DbValue::Float(DbFloat::from(f64::from_le_bytes(bytes)))
            }
            STRING_META_VALUE => {
                if value_index.is_value() {
                    DbValue::String(String::from_utf8_lossy(value_index.value()).to_string())
                } else {
                    DbValue::String(storage.value::<String>(StorageIndex(value_index.index()))?)
                }
            }
            VEC_INT_META_VALUE => {
                DbValue::VecInt(storage.value::<Vec<i64>>(StorageIndex(value_index.index()))?)
            }
            VEC_UINT_META_VALUE => {
                DbValue::VecUint(storage.value::<Vec<u64>>(StorageIndex(value_index.index()))?)
            }
            VEC_FLOAT_META_VALUE => {
                DbValue::VecFloat(storage.value::<Vec<DbFloat>>(StorageIndex(value_index.index()))?)
            }
            VEC_STRING_META_VALUE => {
                DbValue::VecString(storage.value::<Vec<String>>(StorageIndex(value_index.index()))?)
            }
            _ => panic!(),
        })
    }

    pub(crate) fn store_db_value<S: Storage>(
        &self,
        storage: &mut S,
    ) -> Result<DbValueIndex, DbError> {
        let mut index = DbValueIndex::new();

        match self {
            DbValue::Bytes(v) => {
                index.set_type(BYTES_META_VALUE);
                if !index.set_value(v) {
                    index.set_index(storage.insert_bytes(v)?.0);
                }
            }
            DbValue::Int(v) => {
                index.set_type(INT_META_VALUE);
                index.set_value(&v.to_le_bytes());
            }
            DbValue::Uint(v) => {
                index.set_type(UINT_META_VALUE);
                index.set_value(&v.to_le_bytes());
            }
            DbValue::Float(v) => {
                index.set_type(FLOAT_META_VALUE);
                index.set_value(&v.to_f64().to_le_bytes());
            }
            DbValue::String(v) => {
                index.set_type(STRING_META_VALUE);
                let bytes = v.as_bytes();
                if !index.set_value(bytes) {
                    index.set_index(storage.insert(v)?.0);
                }
            }
            DbValue::VecInt(v) => {
                index.set_type(VEC_INT_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
            DbValue::VecUint(v) => {
                index.set_type(VEC_UINT_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
            DbValue::VecFloat(v) => {
                index.set_type(VEC_FLOAT_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
            DbValue::VecString(v) => {
                index.set_type(VEC_STRING_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
        }

        Ok(index)
    }
}

impl Default for DbValue {
    fn default() -> Self {
        Self::Int(0)
    }
}

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

impl From<&String> for DbValue {
    fn from(value: &String) -> Self {
        DbValue::String(value.clone())
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

impl Display for DbValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            DbValue::Bytes(v) => write!(f, "{}", String::from_utf8_lossy(v)),
            DbValue::Int(v) => write!(f, "{}", v),
            DbValue::Uint(v) => write!(f, "{}", v),
            DbValue::Float(v) => write!(f, "{}", v.to_f64()),
            DbValue::String(v) => write!(f, "{}", v),
            DbValue::VecInt(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            DbValue::VecUint(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            DbValue::VecFloat(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|x| x.to_f64().to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            DbValue::VecString(v) => write!(f, "[{}]", v.join(", ")),
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
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;
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
    fn derived_from_display() {
        assert_eq!(format!("{}", DbValue::from(vec![65_u8])), "A");
        assert_eq!(format!("{}", DbValue::from(1.1_f64)), "1.1");
        assert_eq!(format!("{}", DbValue::from(-1_i64)), "-1");
        assert_eq!(format!("{}", DbValue::from(1_u64)), "1");
        assert_eq!(format!("{}", DbValue::from("Hello".to_string())), "Hello");
        assert_eq!(
            format!("{}", DbValue::from(vec![1.1_f64, -0.9_f64])),
            "[1.1, -0.9]"
        );
        assert_eq!(format!("{}", DbValue::from(vec![-1_i64])), "[-1]");
        assert_eq!(format!("{}", DbValue::from(vec![1_u64, 3_u64])), "[1, 3]");
        assert_eq!(
            format!(
                "{}",
                DbValue::from(vec!["Hello".to_string(), "World".to_string()])
            ),
            "[Hello, World]"
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

    #[test]
    #[should_panic]
    fn bad_deserialization() {
        let test_file = TestFile::new();
        let storage = FileStorage::new(&test_file.filename).unwrap();

        let _ = DbValue::load_db_value(DbValueIndex::new(), &storage);
    }
}
