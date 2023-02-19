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

    #[test]
    fn derived_from_clone() {
        let left = Comparison::Equal(DbValue::Int(0));
        let right = left.clone();

        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            Comparison::Equal(DbValue::Int(0)),
            Comparison::Equal(DbValue::Int(0))
        );
    }
}
