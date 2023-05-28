use super::comparison::Comparison;
use super::direction::Direction;

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeCountCondition {
    pub comparison: Comparison,
    pub direction: Direction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbValue;
    #[test]

    fn derived_from_debug() {
        format!(
            "{:?}",
            EdgeCountCondition {
                comparison: Comparison::Equal(DbValue::Int(0)),
                direction: Direction::From
            }
        );
    }
}
