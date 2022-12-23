use super::comparison_operator::ComparisonOperator;
use super::direction::Direction;

pub struct EdgeCountCondition {
    pub count: u64,
    pub operator: ComparisonOperator,
    pub direction: Direction,
}
