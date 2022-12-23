use super::direction::Direction;
use super::logic_operator::LogicOperator;

pub struct EdgeCountCondition {
    pub count: u64,
    pub operator: LogicOperator,
    pub direction: Direction,
}
