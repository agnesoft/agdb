use crate::DbValue;

use super::comparison_operator::ComparisonOperator;

pub struct KeyValueCondition {
    pub value: DbValue,
    pub operator: ComparisonOperator,
}
