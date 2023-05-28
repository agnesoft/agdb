use crate::DbValue;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Equal(DbValue),
    GreaterThan(DbValue),
    GreaterThanOrEqual(DbValue),
    LessThan(DbValue),
    LessThanOrEqual(DbValue),
    NotEqual(DbValue),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", Comparison::Equal(DbValue::Int(0)));
    }
}
