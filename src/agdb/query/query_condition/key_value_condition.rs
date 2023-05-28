use crate::DbKey;

use super::comparison::Comparison;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyValueCondition {
    pub key: DbKey,
    pub comparison: Comparison,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbValue;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            KeyValueCondition {
                comparison: Comparison::Equal(DbValue::Int(0)),
                key: DbValue::Int(0)
            }
        );
    }
}
