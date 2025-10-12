use crate::DbError;
use crate::DbTypeMarker;
use crate::StorageData;
use crate::db::db_f64::DbF64;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as DisplayResult;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
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

#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct DbValues(pub Vec<DbValue>);

const BYTES_META_VALUE: u8 = 1_u8;
const I64_META_VALUE: u8 = 2_u8;
const U64_META_VALUE: u8 = 3_u8;
const F64_META_VALUE: u8 = 4_u8;
const STRING_META_VALUE: u8 = 5_u8;
const VEC_I64_META_VALUE: u8 = 6_u8;
const VEC_U64_META_VALUE: u8 = 7_u8;
const VEC_F64_META_VALUE: u8 = 8_u8;
const VEC_STRING_META_VALUE: u8 = 9_u8;

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

    /// Returns `bool` possibly converted from `i64`, `u64`, `f64` or `string`.
    /// For numerical types any non-zero value will be `true`. For `string` only
    /// "1" or "true" will be `true` and all other string values will be `false`.
    /// Conversion from bytes or vectorized types is an error.
    pub fn to_bool(&self) -> Result<bool, DbError> {
        match self {
            DbValue::Bytes(_) => Self::type_error("bytes", "bool"),
            DbValue::I64(v) => Ok(*v != 0),
            DbValue::U64(v) => Ok(*v != 0),
            DbValue::F64(v) => Ok(*v != 0.0.into()),
            DbValue::String(v) => Ok(v == "true" || v == "1"),
            DbValue::VecI64(_) => Self::type_error("Vec<i64>", "bool"),
            DbValue::VecU64(_) => Self::type_error("Vec<u64>", "bool"),
            DbValue::VecF64(_) => Self::type_error("Vec<f64>", "bool"),
            DbValue::VecString(_) => Self::type_error("Vec<string>", "bool"),
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
    /// or an error if the conversion failed or the value is of
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

    /// Returns `Vec<bool>` possibly converted from `bytes` or vectorized types.
    /// For numerical types any non-zero value will be `true`. For `string` only
    /// "1" or "true" will be `true` and all other string values will be `false`.
    /// Conversion from i64, u64, f64 and string is an error.
    pub fn vec_bool(&self) -> Result<Vec<bool>, DbError> {
        match self {
            DbValue::Bytes(v) => Ok(v.iter().map(|b| *b != 0).collect()),
            DbValue::I64(_) => Self::type_error("i64", "Vec<bool>"),
            DbValue::U64(_) => Self::type_error("u64", "Vec<bool>"),
            DbValue::F64(_) => Self::type_error("f64", "Vec<bool>"),
            DbValue::String(_) => Self::type_error("string", "Vec<bool>"),
            DbValue::VecI64(v) => Ok(v.iter().map(|i| *i != 0).collect()),
            DbValue::VecU64(v) => Ok(v.iter().map(|i| *i != 0).collect()),
            DbValue::VecF64(v) => Ok(v.iter().map(|i| *i != 0.0.into()).collect()),
            DbValue::VecString(v) => Ok(v.iter().map(|s| s == "true" || s == "1").collect()),
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
                    DbValue::Bytes(
                        storage
                            .value_as_bytes(StorageIndex(value_index.index()))?
                            .to_vec(),
                    )
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

    #[track_caller]
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

impl From<PathBuf> for DbValue {
    fn from(value: PathBuf) -> Self {
        (&value).into()
    }
}

impl From<&PathBuf> for DbValue {
    fn from(value: &PathBuf) -> Self {
        DbValue::String(value.to_string_lossy().to_string())
    }
}

impl From<SystemTime> for DbValue {
    fn from(value: SystemTime) -> Self {
        let (duration, before_epoch) = match value.duration_since(UNIX_EPOCH) {
            Ok(duration) => (duration, false),
            Err(duration) => (duration.duration(), true),
        };
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        let mut bytes = [0_u8; 13];
        bytes[0..8].copy_from_slice(&secs.to_le_bytes());
        bytes[8..12].copy_from_slice(&nanos.to_le_bytes());
        bytes[12] = if before_epoch { 0_u8 } else { 1_u8 };
        DbValue::Bytes(bytes.to_vec())
    }
}

impl From<SocketAddr> for DbValue {
    fn from(value: SocketAddr) -> Self {
        DbValue::String(value.to_string())
    }
}

impl From<IpAddr> for DbValue {
    fn from(value: IpAddr) -> Self {
        DbValue::String(value.to_string())
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

impl From<usize> for DbValue {
    fn from(value: usize) -> Self {
        DbValue::U64(value as u64)
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

impl From<bool> for DbValue {
    fn from(value: bool) -> Self {
        DbValue::U64(if value { 1 } else { 0 })
    }
}

impl From<Vec<u8>> for DbValue {
    fn from(value: Vec<u8>) -> Self {
        DbValue::Bytes(value)
    }
}

impl From<Vec<i64>> for DbValue {
    fn from(value: Vec<i64>) -> Self {
        DbValue::VecI64(value)
    }
}

impl From<Vec<i32>> for DbValue {
    fn from(value: Vec<i32>) -> Self {
        DbValue::VecI64(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Vec<u64>> for DbValue {
    fn from(value: Vec<u64>) -> Self {
        DbValue::VecU64(value)
    }
}

impl From<Vec<u32>> for DbValue {
    fn from(value: Vec<u32>) -> Self {
        DbValue::VecU64(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<Vec<usize>> for DbValue {
    fn from(value: Vec<usize>) -> Self {
        DbValue::VecU64(value.into_iter().map(|x| x as u64).collect())
    }
}

impl From<Vec<f64>> for DbValue {
    fn from(value: Vec<f64>) -> Self {
        DbValue::VecF64(value.into_iter().map(DbF64::from).collect())
    }
}

impl From<Vec<f32>> for DbValue {
    fn from(value: Vec<f32>) -> Self {
        DbValue::VecF64(value.into_iter().map(DbF64::from).collect())
    }
}

impl From<Vec<DbF64>> for DbValue {
    fn from(value: Vec<DbF64>) -> Self {
        DbValue::VecF64(value)
    }
}

impl From<Vec<String>> for DbValue {
    fn from(value: Vec<String>) -> Self {
        DbValue::VecString(value)
    }
}

impl From<Vec<&str>> for DbValue {
    fn from(value: Vec<&str>) -> Self {
        DbValue::VecString(value.into_iter().map(|s| s.to_string()).collect())
    }
}

impl From<Vec<bool>> for DbValue {
    fn from(value: Vec<bool>) -> Self {
        DbValue::VecU64(value.into_iter().map(|b| if b { 1 } else { 0 }).collect())
    }
}

impl From<&[u8]> for DbValue {
    fn from(value: &[u8]) -> Self {
        DbValue::Bytes(value.to_vec())
    }
}

impl From<&[i64]> for DbValue {
    fn from(value: &[i64]) -> Self {
        DbValue::VecI64(value.to_vec())
    }
}

impl From<&[i32]> for DbValue {
    fn from(value: &[i32]) -> Self {
        DbValue::VecI64(value.iter().map(|x| *x as i64).collect())
    }
}

impl From<&[u64]> for DbValue {
    fn from(value: &[u64]) -> Self {
        DbValue::VecU64(value.to_vec())
    }
}

impl From<&[u32]> for DbValue {
    fn from(value: &[u32]) -> Self {
        DbValue::VecU64(value.iter().map(|x| *x as u64).collect())
    }
}

impl From<&[usize]> for DbValue {
    fn from(value: &[usize]) -> Self {
        DbValue::VecU64(value.iter().map(|x| *x as u64).collect())
    }
}

impl From<&[f64]> for DbValue {
    fn from(value: &[f64]) -> Self {
        DbValue::VecF64(value.iter().map(|f| DbF64::from(*f)).collect())
    }
}

impl From<&[f32]> for DbValue {
    fn from(value: &[f32]) -> Self {
        DbValue::VecF64(value.iter().map(|f| DbF64::from(*f)).collect())
    }
}

impl From<&[DbF64]> for DbValue {
    fn from(value: &[DbF64]) -> Self {
        DbValue::VecF64(value.to_vec())
    }
}

impl From<&[String]> for DbValue {
    fn from(value: &[String]) -> Self {
        DbValue::VecString(value.to_vec())
    }
}

impl From<&[&str]> for DbValue {
    fn from(value: &[&str]) -> Self {
        DbValue::VecString(value.iter().map(|s| s.to_string()).collect())
    }
}

impl From<&[bool]> for DbValue {
    fn from(value: &[bool]) -> Self {
        DbValue::VecU64(value.iter().map(|b| if *b { 1 } else { 0 }).collect())
    }
}

impl<const N: usize> From<[u8; N]> for DbValue {
    fn from(value: [u8; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[i32; N]> for DbValue {
    fn from(value: [i32; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[i64; N]> for DbValue {
    fn from(value: [i64; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[u64; N]> for DbValue {
    fn from(value: [u64; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[u32; N]> for DbValue {
    fn from(value: [u32; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[usize; N]> for DbValue {
    fn from(value: [usize; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[f32; N]> for DbValue {
    fn from(value: [f32; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[f64; N]> for DbValue {
    fn from(value: [f64; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[DbF64; N]> for DbValue {
    fn from(value: [DbF64; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[String; N]> for DbValue {
    fn from(value: [String; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[&str; N]> for DbValue {
    fn from(value: [&str; N]) -> Self {
        value.as_slice().into()
    }
}

impl<const N: usize> From<[bool; N]> for DbValue {
    fn from(value: [bool; N]) -> Self {
        value.as_slice().into()
    }
}

impl<T: Into<DbValue> + DbTypeMarker> From<Vec<T>> for DbValue {
    fn from(value: Vec<T>) -> Self {
        let db_values = value
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<DbValue>>();
        match db_values.first() {
            Some(DbValue::I64(_)) => {
                DbValue::VecI64(db_values.into_iter().map(|v| v.to_i64().unwrap()).collect())
            }
            Some(DbValue::U64(_)) => {
                DbValue::VecU64(db_values.into_iter().map(|v| v.to_u64().unwrap()).collect())
            }
            Some(DbValue::F64(_)) => {
                DbValue::VecF64(db_values.into_iter().map(|v| v.to_f64().unwrap()).collect())
            }
            Some(DbValue::String(_)) => DbValue::VecString(
                db_values
                    .into_iter()
                    .map(|v| v.string().unwrap().to_owned())
                    .collect(),
            ),
            Some(DbValue::Bytes(_)) => DbValue::Bytes(crate::AgdbSerialize::serialize(&db_values)),
            _ => DbValue::Bytes(Vec::new()),
        }
    }
}

impl<T: Into<DbValue> + Clone + DbTypeMarker> From<&[T]> for DbValue {
    fn from(value: &[T]) -> Self {
        value.to_vec().into()
    }
}

impl<T: Into<DbValue>> From<Vec<T>> for DbValues {
    fn from(value: Vec<T>) -> Self {
        DbValues(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Into<DbValue> + Clone> From<&Vec<T>> for DbValues {
    fn from(value: &Vec<T>) -> Self {
        value.as_slice().into()
    }
}

impl<T: Into<DbValue> + Clone> From<&[T]> for DbValues {
    fn from(value: &[T]) -> Self {
        DbValues(value.iter().map(|v| v.clone().into()).collect())
    }
}

impl<T: Into<DbValue> + Clone, const N: usize> From<[T; N]> for DbValues {
    fn from(value: [T; N]) -> Self {
        value.as_slice().into()
    }
}

impl From<u64> for DbValues {
    fn from(value: u64) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<u32> for DbValues {
    fn from(value: u32) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<usize> for DbValues {
    fn from(value: usize) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<i64> for DbValues {
    fn from(value: i64) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<i32> for DbValues {
    fn from(value: i32) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<f64> for DbValues {
    fn from(value: f64) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<f32> for DbValues {
    fn from(value: f32) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<DbF64> for DbValues {
    fn from(value: DbF64) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<&str> for DbValues {
    fn from(value: &str) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<&String> for DbValues {
    fn from(value: &String) -> Self {
        DbValues(vec![value.into()])
    }
}

impl From<String> for DbValues {
    fn from(value: String) -> Self {
        DbValues(vec![value.into()])
    }
}

impl TryFrom<DbValue> for PathBuf {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(PathBuf::from(value.string()?))
    }
}

impl TryFrom<DbValue> for SystemTime {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        let bytes = value.bytes()?;
        if bytes.len() != 13 {
            return Err(DbError::from(format!(
                "Invalid SystemTime bytes length (should be 13): {}",
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
}

impl TryFrom<DbValue> for SocketAddr {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        value
            .string()?
            .parse::<SocketAddr>()
            .map_err(|e| DbError::from(format!("Cannot convert string to SocketAddr: {e}")))
    }
}

impl TryFrom<DbValue> for IpAddr {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        value
            .string()?
            .parse::<IpAddr>()
            .map_err(|e| DbError::from(format!("Cannot convert string to IpAddr: {e}")))
    }
}

impl TryFrom<DbValue> for Vec<u8> {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.bytes()?.clone())
    }
}

impl TryFrom<DbValue> for u64 {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        value.to_u64()
    }
}

impl TryFrom<DbValue> for u32 {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_u64()?.try_into()?)
    }
}

impl TryFrom<DbValue> for i64 {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        value.to_i64()
    }
}

impl TryFrom<DbValue> for i32 {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_i64()?.try_into()?)
    }
}

impl TryFrom<DbValue> for f64 {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_f64()?.to_f64())
    }
}

impl TryFrom<DbValue> for f32 {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.to_f64()?.to_f64() as f32)
    }
}

impl TryFrom<DbValue> for String {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(value.string()?.clone())
    }
}

impl TryFrom<DbValue> for bool {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        value.to_bool()
    }
}

impl<T: TryFrom<DbValue, Error = DbError>> TryFrom<DbValue> for Vec<T> {
    type Error = DbError;

    #[track_caller]
    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        let db_values: Vec<DbValue> = match value {
            DbValue::VecI64(v) => Ok(v.into_iter().map(DbValue::from).collect()),
            DbValue::VecU64(v) => Ok(v.into_iter().map(DbValue::from).collect()),
            DbValue::VecF64(v) => Ok(v.into_iter().map(DbValue::from).collect()),
            DbValue::VecString(v) => Ok(v.into_iter().map(DbValue::from).collect()),
            DbValue::Bytes(v) => {
                if v.is_empty() {
                    Ok(vec![])
                } else {
                    crate::AgdbSerialize::deserialize(&v).map_err(|mut e| {
                        e.description = format!(
                            "Cannot convert 'bytes' to 'Vec<DbValue>': {}",
                            e.description
                        );
                        e
                    })
                }
            }
            DbValue::I64(_) => DbValue::type_error("i64", "Vec<DbValue>"),
            DbValue::U64(_) => DbValue::type_error("u64", "Vec<DbValue>"),
            DbValue::F64(_) => DbValue::type_error("f64", "Vec<DbValue>"),
            DbValue::String(_) => DbValue::type_error("string", "Vec<DbValue>"),
        }?;
        db_values
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<T>, Self::Error>>()
    }
}

impl Display for DbValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            DbValue::Bytes(v) => write!(f, "{}", String::from_utf8_lossy(v)),
            DbValue::I64(v) => write!(f, "{v}"),
            DbValue::U64(v) => write!(f, "{v}"),
            DbValue::F64(v) => write!(f, "{}", v.to_f64()),
            DbValue::String(v) => write!(f, "{v}"),
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

/// Enables `PathBuf` to be stored as a vector in the database by marking it with the `DbTypeMarker` trait.
impl DbTypeMarker for PathBuf {}

/// Enables `SystemTime` to be stored as a vector in the database by marking it with the `DbTypeMarker` trait.
impl DbTypeMarker for SystemTime {}

/// Enables `SocketAddr` to be stored as a vector in the database by marking it with the `DbTypeMarker` trait.
impl DbTypeMarker for SocketAddr {}

/// Enables `IpAddr` to be stored as a vector in the database by marking it with the `DbTypeMarker` trait.
impl DbTypeMarker for IpAddr {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;
    use std::cmp::Ordering;
    use std::collections::HashSet;
    use std::net::IpAddr;
    use std::net::SocketAddr;

    #[derive(Clone)]
    enum TestEnumString {
        A,
        B,
    }

    impl DbTypeMarker for TestEnumString {}

    impl From<TestEnumString> for DbValue {
        fn from(value: TestEnumString) -> Self {
            match value {
                TestEnumString::A => DbValue::from("A"),
                TestEnumString::B => DbValue::from("B"),
            }
        }
    }

    #[test]
    fn derived_from_eq() {
        let mut map = HashSet::<DbValue>::new();
        map.insert(DbValue::from(1));
    }

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", DbValue::from(""));
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
        let storage = Storage::<FileStorage>::new(test_file.file_name()).unwrap();

        let _ = DbValue::load_db_value(DbValueIndex::new(), &storage);
    }

    #[test]
    fn to_u64() {
        assert_eq!(DbValue::from(1_u64).to_u64().unwrap(), 1_u64);
        assert_eq!(DbValue::from(1_i64).to_u64().unwrap(), 1_u64);
        assert_eq!(DbValue::from(1_usize).to_u64().unwrap(), 1_u64);
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
        assert_eq!(DbValue::from([-1_i64]).vec_i64().unwrap(), &vec![-1_i64]);
        assert_eq!(
            DbValue::from(vec![-1_i32]).vec_i64().unwrap(),
            &vec![-1_i64]
        );
        assert_eq!(DbValue::from([-1_i32]).vec_i64().unwrap(), &vec![-1_i64]);

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
        assert_eq!(DbValue::from([1_u64]).vec_u64().unwrap(), &vec![1_u64]);
        assert_eq!(
            DbValue::from(vec![1_usize]).vec_u64().unwrap(),
            &vec![1_u64]
        );
        assert_eq!(DbValue::from([1_usize]).vec_u64().unwrap(), &vec![1_u64]);
        assert_eq!(DbValue::from(vec![1_u32]).vec_u64().unwrap(), &vec![1_u64]);
        assert_eq!(DbValue::from([1_u32]).vec_u64().unwrap(), &vec![1_u64]);

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
            DbValue::from([1.1]).vec_f64().unwrap(),
            &vec![DbF64::from(1.1_f64)]
        );
        assert_eq!(
            DbValue::from(vec![1.0_f32]).vec_f64().unwrap(),
            &vec![DbF64::from(1.0_f64)]
        );
        assert_eq!(
            DbValue::from([1.0_f32]).vec_f64().unwrap(),
            &vec![DbF64::from(1.0_f64)]
        );
        assert_eq!(
            DbValue::from(vec![DbF64::from(1.1)]).vec_f64().unwrap(),
            &vec![DbF64::from(1.1_f64)]
        );
        assert_eq!(
            DbValue::from([DbF64::from(1.1)]).vec_f64().unwrap(),
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
            DbValue::from([""]).vec_string().unwrap(),
            &vec!["".to_string()]
        );
        assert_eq!(
            DbValue::from(vec![String::new()]).vec_string().unwrap(),
            &vec!["".to_string()]
        );
        assert_eq!(
            DbValue::from([String::new()]).vec_string().unwrap(),
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
        assert_eq!(DbValue::from([1_u8]).bytes().unwrap(), &vec![1_u8]);

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

    #[test]
    fn to_bool() {
        assert!(DbValue::from(true).to_bool().unwrap());
        assert!(DbValue::from(1_u64).to_bool().unwrap());
        assert!(DbValue::from(10_i64).to_bool().unwrap());
        assert!(DbValue::from(1.1).to_bool().unwrap());
        assert!(DbValue::from("true").to_bool().unwrap());
        assert!(DbValue::from("1").to_bool().unwrap());

        assert!(!DbValue::from(false).to_bool().unwrap());
        assert!(!DbValue::from(0_u64).to_bool().unwrap());
        assert!(!DbValue::from(0_i64).to_bool().unwrap());
        assert!(!DbValue::from(0.0).to_bool().unwrap());
        assert!(!DbValue::from("").to_bool().unwrap());
        assert!(!DbValue::from("2").to_bool().unwrap());

        assert_eq!(
            DbValue::from(vec![0_u8]).to_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'bytes' to 'bool'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1_i64]).to_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'Vec<i64>' to 'bool'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![1_u64]).to_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'Vec<u64>' to 'bool'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![0.0_f64]).to_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'Vec<f64>' to 'bool'."
            ))
        );
        assert_eq!(
            DbValue::from(vec!["a", ""]).to_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'Vec<string>' to 'bool'."
            ))
        );
        assert_eq!(
            DbValue::from(vec![true, false]).to_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'Vec<u64>' to 'bool'."
            ))
        );
    }

    #[test]
    fn vec_bool() {
        assert_eq!(
            DbValue::from(vec![true, false]).vec_bool().unwrap(),
            vec![true, false]
        );
        assert_eq!(
            DbValue::from(vec![1_i64, 0, -2]).vec_bool().unwrap(),
            vec![true, false, true]
        );
        assert_eq!(
            DbValue::from(vec![1_u64, 0, 2]).vec_bool().unwrap(),
            vec![true, false, true]
        );
        assert_eq!(
            DbValue::from(vec![1.1_f64, 0.0, 2.2]).vec_bool().unwrap(),
            vec![true, false, true]
        );
        assert_eq!(
            DbValue::from(vec!["true", "1", "", "0", "false", "2"])
                .vec_bool()
                .unwrap(),
            vec![true, true, false, false, false, false]
        );
        assert_eq!(
            DbValue::from(vec![0_u8, 1, 2]).vec_bool().unwrap(),
            vec![false, true, true]
        );
        assert_eq!(
            DbValue::from([true, false]).vec_bool().unwrap(),
            vec![true, false]
        );

        assert_eq!(
            DbValue::from(1_i64).vec_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'Vec<bool>'."
            ))
        );
        assert_eq!(
            DbValue::from(1_u64).vec_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'Vec<bool>'."
            ))
        );
        assert_eq!(
            DbValue::from(1.1_f64).vec_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'Vec<bool>'."
            ))
        );
        assert_eq!(
            DbValue::from("true").vec_bool(),
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'Vec<bool>'."
            ))
        );
    }

    #[test]
    fn try_from_vec() {
        let value: Result<Vec<u64>, DbError> = DbValue::from(0_u64).try_into();

        assert_eq!(
            value,
            Err(DbError::from(
                "Type mismatch. Cannot convert 'u64' to 'Vec<DbValue>'."
            ))
        );

        let value: Result<Vec<u64>, DbError> = DbValue::from(0_i64).try_into();

        assert_eq!(
            value,
            Err(DbError::from(
                "Type mismatch. Cannot convert 'i64' to 'Vec<DbValue>'."
            ))
        );

        let value: Result<Vec<u64>, DbError> = DbValue::from(0.0_f64).try_into();

        assert_eq!(
            value,
            Err(DbError::from(
                "Type mismatch. Cannot convert 'f64' to 'Vec<DbValue>'."
            ))
        );

        let value: Result<Vec<u64>, DbError> = DbValue::from(vec![0_u8]).try_into();

        assert_eq!(
            value,
            Err(DbError::from(
                "Cannot convert 'bytes' to 'Vec<DbValue>': u64 deserialization error: out of bounds"
            ))
        );

        let value: Result<Vec<u64>, DbError> = DbValue::from("").try_into();

        assert_eq!(
            value,
            Err(DbError::from(
                "Type mismatch. Cannot convert 'string' to 'Vec<DbValue>'."
            ))
        );
    }

    #[test]
    fn vec_of_user_values_as_strings() {
        let vec = vec![TestEnumString::A, TestEnumString::B];
        let db_value: DbValue = vec.into();
        let vec: Vec<String> = db_value.vec_string().unwrap().clone();
        assert_eq!(vec, vec!["A", "B"]);
    }

    #[test]
    fn vec_of_user_values_as_f64() {
        enum TestEnum {
            A,
            B,
        }

        impl DbTypeMarker for TestEnum {}

        impl From<TestEnum> for DbValue {
            fn from(value: TestEnum) -> Self {
                match value {
                    TestEnum::A => DbValue::from(1.0),
                    TestEnum::B => DbValue::from(2.0),
                }
            }
        }

        let vec = vec![TestEnum::A, TestEnum::B];
        let db_value: DbValue = vec.into();
        let vec: Vec<DbF64> = db_value.vec_f64().unwrap().clone();
        assert_eq!(vec, vec![DbF64::from(1.0), DbF64::from(2.0)]);
    }

    #[test]
    fn vec_of_user_values_as_u64() {
        enum TestEnum {
            A,
            B,
        }

        impl DbTypeMarker for TestEnum {}

        impl From<TestEnum> for DbValue {
            fn from(value: TestEnum) -> Self {
                match value {
                    TestEnum::A => DbValue::from(1_u64),
                    TestEnum::B => DbValue::from(2_u64),
                }
            }
        }

        let vec = vec![TestEnum::A, TestEnum::B];
        let db_value: DbValue = vec.into();
        let vec: Vec<u64> = db_value.vec_u64().unwrap().clone();
        assert_eq!(vec, vec![1, 2]);
    }

    #[test]
    fn vec_of_user_values_as_i64() {
        enum TestEnum {
            A,
            B,
        }

        impl DbTypeMarker for TestEnum {}

        impl From<TestEnum> for DbValue {
            fn from(value: TestEnum) -> Self {
                match value {
                    TestEnum::A => DbValue::from(1_i64),
                    TestEnum::B => DbValue::from(2_i64),
                }
            }
        }

        let vec = vec![TestEnum::A, TestEnum::B];
        let db_value: DbValue = vec.into();
        let vec: Vec<i64> = db_value.vec_i64().unwrap().clone();
        assert_eq!(vec, vec![1, 2]);
    }

    #[test]
    fn vec_of_user_values_empty() {
        let vec = Vec::<TestEnumString>::new();
        let db_value: DbValue = vec.into();
        let vec = db_value.bytes().unwrap().clone();
        assert_eq!(vec, Vec::<u8>::new());
    }

    #[test]
    fn vec_of_user_values_slice() {
        let vec = Vec::<TestEnumString>::new();
        let db_value: DbValue = vec.as_slice().into();
        let vec = db_value.bytes().unwrap().clone();
        assert_eq!(vec, Vec::<u8>::new());
    }

    #[test]
    fn to_db_values() {
        assert_eq!(DbValues::from(1_u64).0, vec![DbValue::from(1_u64)]);
        assert_eq!(
            DbValues::from(vec![1, 2, 3]).0,
            vec![DbValue::from(1), DbValue::from(2), DbValue::from(3)]
        );
        assert_eq!(
            DbValues::from(&vec![1, 2, 3]).0,
            vec![DbValue::from(1), DbValue::from(2), DbValue::from(3)]
        );
        assert_eq!(
            DbValues::from([1, 2, 3]).0,
            vec![DbValue::from(1), DbValue::from(2), DbValue::from(3)]
        );
        assert_eq!(
            DbValues::from([1, 2, 3].as_slice()).0,
            vec![DbValue::from(1), DbValue::from(2), DbValue::from(3)]
        );
        assert_eq!(DbValues::from(1_i64).0, vec![DbValue::from(1_i64)]);
        assert_eq!(DbValues::from(1_i32).0, vec![DbValue::from(1_i32)]);
        assert_eq!(DbValues::from(1_u64).0, vec![DbValue::from(1_u64)]);
        assert_eq!(DbValues::from(1_u32).0, vec![DbValue::from(1_u32)]);
        assert_eq!(DbValues::from(1_usize).0, vec![DbValue::from(1_usize)]);
        assert_eq!(DbValues::from(1.0).0, vec![DbValue::from(1.0)]);
        assert_eq!(DbValues::from(1.0_f32).0, vec![DbValue::from(1.0_f32)]);
        assert_eq!(
            DbValues::from(DbF64::from(1.0)).0,
            vec![DbValue::from(DbF64::from(1.0))]
        );
        assert_eq!(DbValues::from("Hello").0, vec![DbValue::from("Hello")]);
        assert_eq!(
            DbValues::from("Hello".to_string()).0,
            vec![DbValue::from("Hello".to_string())]
        );
        assert_eq!(
            DbValues::from(&String::new()).0,
            vec![DbValue::from(&String::new())]
        );
        assert_eq!(
            DbValues::from(vec!["Hello"]).0,
            vec![DbValue::from("Hello")]
        );
        assert_eq!(DbValues::from(["Hello"]).0, vec![DbValue::from("Hello")]);
        assert_eq!(
            DbValues::from(["Hello"].as_slice()).0,
            vec![DbValue::from("Hello")]
        );
    }

    #[test]
    fn path_buf() {
        let path = PathBuf::from("/some/path");
        let db_value: DbValue = path.clone().into();
        let path_back: PathBuf = db_value.clone().try_into().unwrap();
        assert_eq!(path, path_back);

        let string: String = db_value.clone().try_into().unwrap();
        assert_eq!(string, "/some/path".to_string());
    }

    #[test]
    fn system_time() {
        let time = SystemTime::now();
        let db_value: DbValue = time.into();
        let time_back: SystemTime = db_value.try_into().unwrap();
        assert_eq!(time, time_back);

        let invalid = DbValue::Bytes(vec![]);
        let result: Result<SystemTime, DbError> = invalid.try_into();
        assert!(result.is_err());

        let before_epoch = SystemTime::UNIX_EPOCH - Duration::from_secs(67);
        let db_value: DbValue = before_epoch.into();
        let db_time = db_value.try_into().unwrap();

        assert_eq!(before_epoch, db_time);
    }

    #[test]
    fn socket_addr() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let db_value: DbValue = addr.into();
        let addr_back: SocketAddr = db_value.clone().try_into().unwrap();
        assert_eq!(addr, addr_back);

        let string: String = db_value.try_into().unwrap();
        assert_eq!(string, "127.0.0.1:8080".to_string());

        let ipv6: SocketAddr = "[::]:8080".parse().unwrap();
        let db_value: DbValue = ipv6.into();
        let addr_back: SocketAddr = db_value.clone().try_into().unwrap();
        assert_eq!(ipv6, addr_back);

        let string: String = db_value.try_into().unwrap();
        assert_eq!(string, "[::]:8080".to_string());

        let invalid = DbValue::String("invalid".to_string());
        let result: Result<SocketAddr, DbError> = invalid.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn ip_addr() {
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        let db_value: DbValue = addr.into();
        let addr_back: IpAddr = db_value.try_into().unwrap();
        assert_eq!(addr, addr_back);

        let string: String = DbValue::from(addr).try_into().unwrap();
        assert_eq!(string, "127.0.0.1".to_string());

        let ipv6: IpAddr = "::".parse().unwrap();
        let db_value: DbValue = ipv6.into();
        let addr_back: IpAddr = db_value.try_into().unwrap();
        assert_eq!(ipv6, addr_back);

        let string: String = DbValue::from(ipv6).try_into().unwrap();
        assert_eq!(string, "::".to_string());

        let invalid = DbValue::String("invalid".to_string());
        let result: Result<IpAddr, DbError> = invalid.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn vec_pathbuf() {
        let paths = vec![PathBuf::from("/some/path"), PathBuf::from("/other/path")];
        let db_value: DbValue = paths.clone().into();
        let paths_back: Vec<PathBuf> = db_value.clone().try_into().unwrap();
        assert_eq!(paths, paths_back);

        let strings: Vec<String> = db_value.vec_string().unwrap().clone();
        assert_eq!(
            strings,
            vec!["/some/path".to_string(), "/other/path".to_string()]
        );
    }

    #[test]
    fn vec_system_time() {
        let times = vec![
            SystemTime::now(),
            SystemTime::now() + Duration::from_secs(3600),
        ];
        let db_value: DbValue = times.clone().into();
        let times_back: Vec<SystemTime> = db_value.try_into().unwrap();
        assert_eq!(times, times_back);
    }

    #[test]
    fn vec_socket_addr() {
        let addrs: Vec<SocketAddr> = vec![
            "127.0.0.1:8080".parse().unwrap(),
            "[::]:8080".parse().unwrap(),
        ];
        let db_value: DbValue = addrs.clone().into();
        let addrs_back: Vec<SocketAddr> = db_value.clone().try_into().unwrap();
        assert_eq!(addrs, addrs_back);
        let strings: Vec<String> = db_value.vec_string().unwrap().clone();
        assert_eq!(
            strings,
            vec!["127.0.0.1:8080".to_string(), "[::]:8080".to_string()]
        );
    }

    #[test]
    fn vec_ip_addr() {
        let addrs: Vec<IpAddr> = vec!["127.0.0.1".parse().unwrap(), "::".parse().unwrap()];
        let db_value: DbValue = addrs.clone().into();
        let addrs_back: Vec<IpAddr> = db_value.clone().try_into().unwrap();
        assert_eq!(addrs, addrs_back);
        let strings: Vec<String> = db_value.vec_string().unwrap().clone();
        assert_eq!(strings, vec!["127.0.0.1".to_string(), "::".to_string()]);
    }
}
