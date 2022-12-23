use crate::DbValue;

use super::logic_operator::LogicOperator;

pub struct KeyValueCondition {
    pub value: DbValue,
    pub operator: LogicOperator,
}
