use super::db_float::DbFloat;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DbValue {
    Bytes(Vec<u8>),
    Float(DbFloat),
    Int(i64),
    Uint(u64),
    String(String),
    VecFloat(Vec<DbFloat>),
    VecInt(Vec<i64>),
    VecUint(Vec<u64>),
    VecString(Vec<String>),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn derived_from_eq() {
        let mut map = HashSet::<DbValue>::new();
        map.insert(DbValue::from(1));
    }

    #[test]
    fn derived_from_ord() {
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
        assert!(matches!(DbValue::from(1.0_f32), DbValue::Float { .. }));
        assert!(matches!(DbValue::from(1.0_f64), DbValue::Float { .. }));
        assert!(matches!(DbValue::from(1_i32), DbValue::Int { .. }));
        assert!(matches!(DbValue::from(1_i64), DbValue::Int { .. }));
        assert!(matches!(DbValue::from(1_u32), DbValue::Uint { .. }));
        assert!(matches!(DbValue::from(1_u64), DbValue::Uint { .. }));
        assert!(matches!(DbValue::from(""), DbValue::String { .. }));
        assert!(matches!(
            DbValue::from(String::new()),
            DbValue::String { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![1.0_f32]),
            DbValue::VecFloat { .. }
        ));
        assert!(matches!(
            DbValue::from(vec![1.0_f64]),
            DbValue::VecFloat { .. }
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
        assert!(matches!(DbValue::from(vec![""]), DbValue::VecString { .. }));
        assert!(matches!(
            DbValue::from(Vec::<String>::new()),
            DbValue::VecString { .. }
        ));
    }
}
