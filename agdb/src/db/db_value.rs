use super::db_f64::DbF64;
use super::db_value_index::DbValueIndex;
use crate::storage::Storage;
use crate::storage::StorageData;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as DisplayResult;

/// Database value is a strongly types value.
///
/// It is an enum of limited number supported types
/// that are universal across all platforms
/// and programming languages.
///
/// The value is constructible from large number of
/// raw types or associated types (e.g. i32, &str, etc.).
/// Getting the raw value back as string can be done
/// with `to_string()` but otherwise requires a `match`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DbValue {
    /// Byte array, sometimes referred to as blob
    Bytes(Vec<u8>),

    /// 64-bit wide signed integer
    I64(i64),

    /// 64-bit wide unsigned integer
    U64(u64),

    /// 64-bit floating point number
    F64(DbF64),

    /// UTF-8 string
    String(String),

    /// List of 64-bit wide signed integers
    VecI64(Vec<i64>),

    /// List of 64-bit wide unsigned integers
    VecU64(Vec<u64>),

    /// List of 64-bit floating point numbers
    VecF64(Vec<DbF64>),

    /// List of UTF-8 strings
    VecString(Vec<String>),
}

pub(crate) const BYTES_META_VALUE: u8 = 1_u8;
pub(crate) const I64_META_VALUE: u8 = 2_u8;
pub(crate) const U64_META_VALUE: u8 = 3_u8;
pub(crate) const F64_META_VALUE: u8 = 4_u8;
pub(crate) const STRING_META_VALUE: u8 = 5_u8;
pub(crate) const VEC_I64_META_VALUE: u8 = 6_u8;
pub(crate) const VEC_U64_META_VALUE: u8 = 7_u8;
pub(crate) const VEC_F64_META_VALUE: u8 = 8_u8;
pub(crate) const VEC_STRING_META_VALUE: u8 = 9_u8;

impl DbValue {
    /// Returns `&Vec<u8>` or an error if the value is
    /// of a different type.
    pub fn bytes(&self) -> Result<&Vec<u8>, DbError> {
        match self {
            DbValue::Bytes(v) => Ok(v),
            DbValue::I64(_) => Self::type_error("i64", "bytes"),
            DbValue::U64(_) => Self::type_error("u64", "bytes"),
            DbValue::F64(_) => Self::type_error("f64", "bytes"),
            DbValue::String(_) => Self::type_error("string", "bytes"),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "bytes"),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "bytes"),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "bytes"),
            DbValue::VecString(_) => Self::type_error("vec<string>", "bytes"),
        }
    }

    /// Returns `&String` or an error if the value is
    /// of a different type.
    pub fn string(&self) -> Result<&String, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "string"),
            DbValue::I64(_) => Self::type_error("i64", "string"),
            DbValue::U64(_) => Self::type_error("u64", "string"),
            DbValue::F64(_) => Self::type_error("f64", "string"),
            DbValue::String(v) => Ok(v),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "string"),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "string"),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "string"),
            DbValue::VecString(_) => Self::type_error("vec<string>", "string"),
        }
    }

    /// Returns `DbF64` possibly converted from `i64` or `u64`
    /// or na error if the conversion failed or the value is of
    /// a different type.
    pub fn to_f64(&self) -> Result<DbF64, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "f64"),
            DbValue::I64(v) => Ok(DbF64::from(f64::from(i32::try_from(*v)?))),
            DbValue::U64(v) => Ok(DbF64::from(f64::from(u32::try_from(*v)?))),
            DbValue::F64(v) => Ok(*v),
            DbValue::String(_) => Self::type_error("string", "f64"),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "f64"),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "f64"),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "f64"),
            DbValue::VecString(_) => Self::type_error("vec<string>", "f64"),
        }
    }

    /// Returns `i64` possibly converted from `u64`
    /// or na error if the conversion failed or the value is of
    /// a different type.
    pub fn to_i64(&self) -> Result<i64, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "i64"),
            DbValue::I64(v) => Ok(*v),
            DbValue::U64(v) => Ok(i64::try_from(*v)?),
            DbValue::F64(_) => Self::type_error("f64", "i64"),
            DbValue::String(_) => Self::type_error("string", "i64"),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "i64"),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "i64"),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "i64"),
            DbValue::VecString(_) => Self::type_error("vec<string>", "i64"),
        }
    }

    /// Returns `u64` possibly converted from `i64`
    /// or na error if the conversion failed or the value is of
    /// a different type.
    pub fn to_u64(&self) -> Result<u64, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "u64"),
            DbValue::I64(v) => Ok(u64::try_from(*v)?),
            DbValue::U64(v) => Ok(*v),
            DbValue::F64(_) => Self::type_error("f64", "u64"),
            DbValue::String(_) => Self::type_error("string", "u64"),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "u64"),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "u64"),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "u64"),
            DbValue::VecString(_) => Self::type_error("vec<string>", "u64"),
        }
    }

    /// Returns `&Vec<DbF64>` or an error if the value is
    /// of a different type.
    pub fn vec_f64(&self) -> Result<&Vec<DbF64>, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "vec<f64>"),
            DbValue::I64(_) => Self::type_error("i64", "vec<f64>"),
            DbValue::U64(_) => Self::type_error("u64", "vec<f64>"),
            DbValue::F64(_) => Self::type_error("f64", "vec<f64>"),
            DbValue::String(_) => Self::type_error("string", "vec<f64>"),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "vec<f64>"),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "vec<f64>"),
            DbValue::VecF64(v) => Ok(v),
            DbValue::VecString(_) => Self::type_error("vec<string>", "vec<f64>"),
        }
    }

    /// Returns `&Vec<i64>` or an error if the value is
    /// of a different type.
    pub fn vec_i64(&self) -> Result<&Vec<i64>, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "vec<i64>"),
            DbValue::I64(_) => Self::type_error("i64", "vec<i64>"),
            DbValue::U64(_) => Self::type_error("u64", "vec<i64>"),
            DbValue::F64(_) => Self::type_error("f64", "vec<i64>"),
            DbValue::String(_) => Self::type_error("string", "vec<i64>"),
            DbValue::VecI64(v) => Ok(v),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "vec<i64>"),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "vec<i64>"),
            DbValue::VecString(_) => Self::type_error("vec<string>", "vec<i64>"),
        }
    }

    /// Returns `&Vec<u64>` or an error if the value is
    /// of a different type.
    pub fn vec_u64(&self) -> Result<&Vec<u64>, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "vec<u64>"),
            DbValue::I64(_) => Self::type_error("i64", "vec<u64>"),
            DbValue::U64(_) => Self::type_error("u64", "vec<u64>"),
            DbValue::F64(_) => Self::type_error("f64", "vec<u64>"),
            DbValue::String(_) => Self::type_error("string", "vec<u64>"),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "vec<u64>"),
            DbValue::VecU64(v) => Ok(v),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "vec<u64>"),
            DbValue::VecString(_) => Self::type_error("vec<string>", "vec<u64>"),
        }
    }

    /// Returns `&Vec<String>` or an error if the value is
    /// of a different type.
    pub fn vec_string(&self) -> Result<&Vec<String>, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "vec<string>"),
            DbValue::I64(_) => Self::type_error("i64", "vec<string>"),
            DbValue::U64(_) => Self::type_error("u64", "vec<string>"),
            DbValue::F64(_) => Self::type_error("f64", "vec<string>"),
            DbValue::String(_) => Self::type_error("string", "vec<string>"),
            DbValue::VecI64(_) => Self::type_error("vec<i64>", "vec<string>"),
            DbValue::VecU64(_) => Self::type_error("vec<u64>", "vec<string>"),
            DbValue::VecF64(_) => Self::type_error("vec<f64>", "vec<string>"),
            DbValue::VecString(v) => Ok(v),
        }
    }

    pub(crate) fn load_db_value<D: StorageData>(
        value_index: DbValueIndex,
        storage: &Storage<D>,
    ) -> Result<DbValue, DbError> {
        Ok(match value_index.get_type() {
            BYTES_META_VALUE => {
                if value_index.is_value() {
                    DbValue::Bytes(value_index.value().to_vec())
                } else {
                    DbValue::Bytes(storage.value_as_bytes(StorageIndex(value_index.index()))?)
                }
            }
            I64_META_VALUE => {
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(value_index.value());
                DbValue::I64(i64::from_le_bytes(bytes))
            }
            U64_META_VALUE => {
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(value_index.value());
                DbValue::U64(u64::from_le_bytes(bytes))
            }
            F64_META_VALUE => {
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(value_index.value());
                DbValue::F64(DbF64::from(f64::from_le_bytes(bytes)))
            }
            STRING_META_VALUE => {
                if value_index.is_value() {
                    DbValue::String(String::from_utf8_lossy(value_index.value()).to_string())
                } else {
                    DbValue::String(storage.value::<String>(StorageIndex(value_index.index()))?)
                }
            }
            VEC_I64_META_VALUE => {
                DbValue::VecI64(storage.value::<Vec<i64>>(StorageIndex(value_index.index()))?)
            }
            VEC_U64_META_VALUE => {
                DbValue::VecU64(storage.value::<Vec<u64>>(StorageIndex(value_index.index()))?)
            }
            VEC_F64_META_VALUE => {
                DbValue::VecF64(storage.value::<Vec<DbF64>>(StorageIndex(value_index.index()))?)
            }
            VEC_STRING_META_VALUE => {
                DbValue::VecString(storage.value::<Vec<String>>(StorageIndex(value_index.index()))?)
            }
            _ => panic!(),
        })
    }

    pub(crate) fn store_db_value<D: StorageData>(
        &self,
        storage: &mut Storage<D>,
    ) -> Result<DbValueIndex, DbError> {
        let mut index = DbValueIndex::new();

        match self {
            DbValue::Bytes(v) => {
                index.set_type(BYTES_META_VALUE);
                if !index.set_value(v) {
                    index.set_index(storage.insert_bytes(v)?.0);
                }
            }
            DbValue::I64(v) => {
                index.set_type(I64_META_VALUE);
                index.set_value(&v.to_le_bytes());
            }
            DbValue::U64(v) => {
                index.set_type(U64_META_VALUE);
                index.set_value(&v.to_le_bytes());
            }
            DbValue::F64(v) => {
                index.set_type(F64_META_VALUE);
                index.set_value(&v.to_f64().to_le_bytes());
            }
            DbValue::String(v) => {
                index.set_type(STRING_META_VALUE);
                let bytes = v.as_bytes();
                if !index.set_value(bytes) {
                    index.set_index(storage.insert(v)?.0);
                }
            }
            DbValue::VecI64(v) => {
                index.set_type(VEC_I64_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
            DbValue::VecU64(v) => {
                index.set_type(VEC_U64_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
            DbValue::VecF64(v) => {
                index.set_type(VEC_F64_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
            DbValue::VecString(v) => {
                index.set_type(VEC_STRING_META_VALUE);
                index.set_index(storage.insert(v)?.0);
            }
        }

        Ok(index)
    }

    fn type_error<T>(from: &str, to: &str) -> Result<T, DbError> {
        Err(DbError::from(format!(
            "Type mismatch. Cannot convert '{from}' to '{to}'."
        )))
    }
}

impl Default for DbValue {
    fn default() -> Self {
        Self::I64(0)
    }
}

impl From<f32> for DbValue {
    fn from(value: f32) -> Self {
        DbValue::F64(value.into())
    }
}

impl From<f64> for DbValue {
    fn from(value: f64) -> Self {
        DbValue::F64(value.into())
    }
}

impl From<DbF64> for DbValue {
    fn from(value: DbF64) -> Self {
        DbValue::F64(value)
    }
}

impl From<i32> for DbValue {
    fn from(value: i32) -> Self {
        DbValue::I64(value.into())
    }
}

impl From<i64> for DbValue {
    fn from(value: i64) -> Self {
        DbValue::I64(value)
    }
}

impl From<u32> for DbValue {
    fn from(value: u32) -> Self {
        DbValue::U64(value.into())
    }
}

impl From<u64> for DbValue {
    fn from(value: u64) -> Self {
        DbValue::U64(value)
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
        DbValue::VecF64(value.iter().map(|i| (*i).into()).collect())
    }
}

impl From<Vec<f64>> for DbValue {
    fn from(value: Vec<f64>) -> Self {
        DbValue::VecF64(value.iter().map(|i| (*i).into()).collect())
    }
}

impl From<Vec<DbF64>> for DbValue {
    fn from(value: Vec<DbF64>) -> Self {
        DbValue::VecF64(value)
    }
}

impl From<Vec<i32>> for DbValue {
    fn from(value: Vec<i32>) -> Self {
        DbValue::VecI64(value.iter().map(|i| *i as i64).collect())
    }
}

impl From<Vec<i64>> for DbValue {
    fn from(value: Vec<i64>) -> Self {
        DbValue::VecI64(value)
    }
}

impl From<Vec<u32>> for DbValue {
    fn from(value: Vec<u32>) -> Self {
        DbValue::VecU64(value.iter().map(|i| *i as u64).collect())
    }
}

impl From<Vec<u64>> for DbValue {
    fn from(value: Vec<u64>) -> Self {
        DbValue::VecU64(value)
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

impl TryFrom<DbValue> for Vec<u8> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.bytes()?.clone())
    }
}

impl TryFrom<DbValue> for u64 {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        value.to_u64()
    }
}

impl TryFrom<DbValue> for u32 {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_u64()?.try_into()?)
    }
}

impl TryFrom<DbValue> for i64 {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        value.to_i64()
    }
}

impl TryFrom<DbValue> for i32 {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_i64()?.try_into()?)
    }
}

impl TryFrom<DbValue> for f64 {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_f64()?.to_f64())
    }
}

impl TryFrom<DbValue> for f32 {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_f64()?.to_f64() as f32)
    }
}

impl TryFrom<DbValue> for String {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.string()?.clone())
    }
}

impl TryFrom<DbValue> for Vec<u64> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.vec_u64()?.clone())
    }
}

impl TryFrom<DbValue> for Vec<u32> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        let mut result = vec![];
        let value = value.vec_u64()?;
        result.reserve(value.len());
        value.iter().try_for_each(|u| -> Result<(), DbError> {
            result.push((*u).try_into()?);
            Ok(())
        })?;
        Ok(result)
    }
}

impl TryFrom<DbValue> for Vec<i64> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.vec_i64()?.clone())
    }
}

impl TryFrom<DbValue> for Vec<i32> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        let mut result = vec![];
        let value = value.vec_i64()?;
        result.reserve(value.len());
        value.iter().try_for_each(|u| -> Result<(), DbError> {
            result.push((*u).try_into()?);
            Ok(())
        })?;
        Ok(result)
    }
}

impl TryFrom<DbValue> for Vec<f64> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.vec_f64()?.iter().map(|f| f.to_f64()).collect())
    }
}

impl TryFrom<DbValue> for Vec<f32> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.vec_f64()?.iter().map(|f| f.to_f64() as f32).collect())
    }
}

impl TryFrom<DbValue> for Vec<String> {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.vec_string()?.clone())
    }
}

impl Display for DbValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            DbValue::Bytes(v) => write!(f, "{}", String::from_utf8_lossy(v)),
            DbValue::I64(v) => write!(f, "{}", v),
            DbValue::U64(v) => write!(f, "{}", v),
            DbValue::F64(v) => write!(f, "{}", v.to_f64()),
            DbValue::String(v) => write!(f, "{}", v),
            DbValue::VecI64(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            DbValue::VecU64(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            DbValue::VecF64(v) => write!(
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
            DbValue::I64(value) => value.stable_hash(),
            DbValue::U64(value) => value.stable_hash(),
            DbValue::F64(value) => value.stable_hash(),
            DbValue::String(value) => value.stable_hash(),
            DbValue::VecI64(value) => value.stable_hash(),
            DbValue::VecU64(value) => value.stable_hash(),
            DbValue::VecF64(value) => value.stable_hash(),
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
        assert!(matches!(DbValue::from(1_i32), DbValue::I64 { .. }));
        assert!(matches!(DbValue::from(1_i64), DbValue::I64 { .. }));
        assert!(matches!(DbValue::from(1_u32), DbValue::U64 { .. }));
        assert!(matches!(DbValue::from(1_u64), DbValue::U64 { .. }));
        assert!(matches!(DbValue::from(1.0_f32), DbValue::F64 { .. }));
        assert!(matches!(DbValue::from(1.0_f64), DbValue::F64 { .. }));
        assert!(matches!(
            DbValue::from(DbF64::from(1.0_f64)),
            DbValue::F64 { .. }
        ));
        assert!(matches!(DbValue::from(""), DbValue::String { .. }));
        assert!(matches!(
            DbValue::from(String::new()),
            DbValue::String { .. }
        ));
        assert!(matches!(DbValue::from(vec![1_i32]), DbValue::VecI64 { .. }));
        assert!(matches!(DbValue::from(vec![1_i64]), DbValue::VecI64 { .. }));
        assert!(matches!(DbValue::from(vec![1_u32]), DbValue::VecU64 { .. }));
        assert!(matches!(DbValue::from(vec![1_u64]), DbValue::VecU64 { .. }));
        assert!(matches!(
            DbValue::from(vec![1.0_f32]),
            DbValue::VecF64 { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![1.0_f64]),
            DbValue::VecF64 { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![DbF64::from(1.0_f64)]),
            DbValue::VecF64 { .. }
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
        let storage = Storage::<FileStorage>::new(&test_file.filename).unwrap();

        let _ = DbValue::load_db_value(DbValueIndex::new(), &storage);
    }

    #[test]
    fn to_u64() {
        assert_eq!(DbValue::from(1_u64).to_u64().unwrap(), 1_u64);
        assert_eq!(DbValue::from(1_i64).to_u64().unwrap(), 1_u64);
        assert_eq!(
            DbValue::from(-1_i64).to_u64(),
            Err(DbError::from(
                "out of range integral type conversion attempted"
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u8; 1]).to_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'u64'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).to_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'u64'."
            ))
        );
        assert_eq!(
            DbValue::from("").to_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'u64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u64]).to_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'u64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_i64]).to_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'u64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).to_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'u64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).to_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'u64'."
            ))
        );
    }

    #[test]
    fn to_i64() {
        assert_eq!(DbValue::from(-1_i64).to_i64().unwrap(), -1_i64);
        assert_eq!(DbValue::from(1_u64).to_i64().unwrap(), 1_i64);
        assert_eq!(
            DbValue::from(u64::MAX).to_i64(),
            Err(DbError::from(
                "out of range integral type conversion attempted"
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u8; 1]).to_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'i64'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).to_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'i64'."
            ))
        );
        assert_eq!(
            DbValue::from("").to_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'i64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u64]).to_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'i64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_i64]).to_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'i64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).to_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'i64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).to_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'i64'."
            ))
        );
    }

    #[test]
    fn to_f64() {
        assert_eq!(
            DbValue::from(1.1_f64).to_f64().unwrap(),
            DbF64::from(1.1_f64)
        );
        assert_eq!(
            DbValue::from(-1_i64).to_f64().unwrap(),
            DbF64::from(-1.0_f64)
        );
        assert_eq!(DbValue::from(1_i64).to_f64().unwrap(), DbF64::from(1.0_f64));
        assert_eq!(DbValue::from(1_u64).to_f64().unwrap(), DbF64::from(1.0_f64));
        assert_eq!(
            DbValue::from(i64::MAX).to_f64(),
            Err(DbError::from(
                "out of range integral type conversion attempted"
            ))
        );
        assert_eq!(
            DbValue::from(u64::MAX).to_f64(),
            Err(DbError::from(
                "out of range integral type conversion attempted"
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u8; 1]).to_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'f64'."
            ))
        );
        assert_eq!(
            DbValue::from("").to_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'f64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u64]).to_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'f64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_i64]).to_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'f64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).to_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'f64'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).to_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'f64'."
            ))
        );
    }

    #[test]
    fn vec_i64() {
        assert_eq!(
            DbValue::from(vec![-1_i64]).vec_i64().unwrap(),
            &vec![-1_i64]
        );
        assert_eq!(
            DbValue::from(-1_i64).vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'vec<i64>'."
            ))
        );
        assert_eq!(
            DbValue::from(1_u64).vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'vec<i64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1_u64]).vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'vec<i64>'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'vec<i64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u8; 1]).vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'vec<i64>'."
            ))
        );
        assert_eq!(
            DbValue::from("").vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'vec<i64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'vec<i64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).vec_i64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'vec<i64>'."
            ))
        );
    }

    #[test]
    fn vec_u64() {
        assert_eq!(DbValue::from(vec![1_u64]).vec_u64().unwrap(), &vec![1_u64]);
        assert_eq!(
            DbValue::from(-1_i64).vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'vec<u64>'."
            ))
        );
        assert_eq!(
            DbValue::from(1_u64).vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'vec<u64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![-1_i64]).vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'vec<u64>'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'vec<u64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u8; 1]).vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'vec<u64>'."
            ))
        );
        assert_eq!(
            DbValue::from("").vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'vec<u64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'vec<u64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).vec_u64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'vec<u64>'."
            ))
        );
    }

    #[test]
    fn vec_f64() {
        assert_eq!(
            DbValue::from(vec![1.1]).vec_f64().unwrap(),
            &vec![DbF64::from(1.1_f64)]
        );
        assert_eq!(
            DbValue::from(-1_i64).vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'vec<f64>'."
            ))
        );
        assert_eq!(
            DbValue::from(1_u64).vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'vec<f64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![-1_i64]).vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'vec<f64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1_u64]).vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'vec<f64>'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'vec<f64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u8; 1]).vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'vec<f64>'."
            ))
        );
        assert_eq!(
            DbValue::from("").vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'vec<f64>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).vec_f64(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'vec<f64>'."
            ))
        );
    }

    #[test]
    fn vec_string() {
        assert_eq!(
            DbValue::from(vec![""]).vec_string().unwrap(),
            &vec!["".to_string()]
        );
        assert_eq!(
            DbValue::from(-1_i64).vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'vec<string>'."
            ))
        );
        assert_eq!(
            DbValue::from(1_u64).vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'vec<string>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![-1_i64]).vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'vec<string>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1_u64]).vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'vec<string>'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'vec<string>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'vec<string>'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0_u8; 1]).vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'vec<string>'."
            ))
        );
        assert_eq!(
            DbValue::from("").vec_string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'vec<string>'."
            ))
        );
    }

    #[test]
    fn bytes() {
        assert_eq!(DbValue::from(vec![1_u8]).bytes().unwrap(), &vec![1_u8]);
        assert_eq!(
            DbValue::from(-1_i64).bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'bytes'."
            ))
        );
        assert_eq!(
            DbValue::from(1_u64).bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'bytes'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![-1_i64]).bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'bytes'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1_u64]).bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'bytes'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'bytes'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'bytes'."
            ))
        );
        assert_eq!(
            DbValue::from("").bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'bytes'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).bytes(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'bytes'."
            ))
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            DbValue::from("hello").string().unwrap(),
            &"hello".to_string()
        );
        assert_eq!(
            DbValue::from(vec![1_u8]).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'string'."
            ))
        );
        assert_eq!(
            DbValue::from(-1_i64).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'string'."
            ))
        );
        assert_eq!(
            DbValue::from(1_u64).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'string'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![-1_i64]).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<i64>' to 'string'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1_u64]).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<u64>' to 'string'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'string'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1.1]).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<f64>' to 'string'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![""]).string(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'vec<string>' to 'string'."
            ))
        );
    }
}
