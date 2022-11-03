pub enum DbValue {
    Bytes(Vec<u8>),
    Double(f64),
    Int(i64),
    Uint(u64),
    String(String),
    VecDouble(Vec<f64>),
    VecInt(Vec<i64>),
    VecUint(Vec<u64>),
    VecString(Vec<String>),
}

impl From<f32> for DbValue {
    fn from(value: f32) -> Self {
        DbValue::Double(value.into())
    }
}

impl From<f64> for DbValue {
    fn from(value: f64) -> Self {
        DbValue::Double(value)
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
        DbValue::VecDouble(value.iter().map(|i| *i as f64).collect())
    }
}

impl From<Vec<f64>> for DbValue {
    fn from(value: Vec<f64>) -> Self {
        DbValue::VecDouble(value)
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

    #[test]
    fn from() {
        assert!(matches!(
            DbValue::from(Vec::<u8>::new()),
            DbValue::Bytes { .. }
        ));
        assert!(matches!(DbValue::from(1.0_f32), DbValue::Double { .. }));
        assert!(matches!(DbValue::from(1.0_f64), DbValue::Double { .. }));
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
            DbValue::from(Vec::<f32>::new()),
            DbValue::VecDouble { .. }
        ));
        assert!(matches!(
            DbValue::from(Vec::<f64>::new()),
            DbValue::VecDouble { .. }
        ));
        assert!(matches!(
            DbValue::from(Vec::<i32>::new()),
            DbValue::VecInt { .. }
        ));
        assert!(matches!(
            DbValue::from(Vec::<i64>::new()),
            DbValue::VecInt { .. }
        ));
        assert!(matches!(
            DbValue::from(Vec::<u32>::new()),
            DbValue::VecUint { .. }
        ));
        assert!(matches!(
            DbValue::from(Vec::<u64>::new()),
            DbValue::VecUint { .. }
        ));
        assert!(matches!(
            DbValue::from(Vec::<&str>::new()),
            DbValue::VecString { .. }
        ));
        assert!(matches!(
            DbValue::from(Vec::<String>::new()),
            DbValue::VecString { .. }
        ));
    }
}
