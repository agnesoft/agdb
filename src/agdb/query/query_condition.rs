use super::query_ids::QueryIds;
use crate::graph_search::SearchControl;
use crate::DbKey;
use crate::DbValue;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryCondition {
    And,
    Distance(CountComparison),
    Edge,
    EdgeCount(EdgeCountCondition),
    EndWhere,
    Ids(QueryIds),
    KeyValue(KeyValueCondition),
    Keys(Vec<DbKey>),
    Node,
    Not,
    NotBeyond,
    Or,
    Where,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CountComparison {
    Equal(u64),
    GreaterThan(u64),
    GreaterThanOrEqual(u64),
    LessThan(u64),
    LessThanOrEqual(u64),
    NotEqual(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Equal(DbValue),
    GreaterThan(DbValue),
    GreaterThanOrEqual(DbValue),
    LessThan(DbValue),
    LessThanOrEqual(DbValue),
    NotEqual(DbValue),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeCountCondition {
    pub comparison: CountComparison,
    pub direction: Direction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Both,
    From,
    To,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyValueCondition {
    pub key: DbKey,
    pub comparison: Comparison,
}

impl CountComparison {
    pub(crate) fn compare(&self, right: u64) -> SearchControl {
        match self {
            CountComparison::Equal(left) => match right.cmp(left) {
                std::cmp::Ordering::Less => SearchControl::Continue(false),
                std::cmp::Ordering::Equal => SearchControl::Stop(true),
                std::cmp::Ordering::Greater => SearchControl::Stop(false),
            },
            CountComparison::GreaterThan(left) => match right.cmp(left) {
                std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                    SearchControl::Continue(false)
                }
                std::cmp::Ordering::Greater => SearchControl::Continue(true),
            },
            CountComparison::GreaterThanOrEqual(left) => match right.cmp(left) {
                std::cmp::Ordering::Less => SearchControl::Continue(false),
                std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => {
                    SearchControl::Continue(true)
                }
            },
            CountComparison::LessThan(left) => match right.cmp(left) {
                std::cmp::Ordering::Less => SearchControl::Continue(true),
                std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => {
                    SearchControl::Stop(false)
                }
            },
            CountComparison::LessThanOrEqual(left) => match right.cmp(left) {
                std::cmp::Ordering::Less => SearchControl::Continue(true),
                std::cmp::Ordering::Equal => SearchControl::Stop(true),
                std::cmp::Ordering::Greater => SearchControl::Stop(false),
            },
            CountComparison::NotEqual(left) => match right.cmp(left) {
                std::cmp::Ordering::Less | std::cmp::Ordering::Greater => {
                    SearchControl::Continue(true)
                }
                std::cmp::Ordering::Equal => SearchControl::Continue(false),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryCondition::Where);
        format!("{:?}", Comparison::Equal(DbValue::Int(0)));
        format!("{:?}", Direction::From);
        format!(
            "{:?}",
            EdgeCountCondition {
                comparison: CountComparison::Equal(0),
                direction: Direction::From
            }
        );
        format!(
            "{:?}",
            KeyValueCondition {
                comparison: Comparison::Equal(DbValue::Int(0)),
                key: DbValue::Int(0)
            }
        );
    }

    #[test]
    fn derived_from_clone() {
        let left = QueryCondition::Where;
        let right = left.clone();
        assert_eq!(left, right);

        let left = Comparison::Equal(DbValue::Int(0));
        let right = left.clone();
        assert_eq!(left, right);

        let left = Direction::From;
        let right = left.clone();
        assert_eq!(left, right);

        let left = EdgeCountCondition {
            comparison: CountComparison::Equal(0),
            direction: Direction::From,
        };
        let right = left.clone();
        assert_eq!(left, right);

        let left = KeyValueCondition {
            comparison: Comparison::Equal(DbValue::Int(0)),
            key: DbValue::Int(0),
        };
        let right = left.clone();
        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(QueryCondition::Where, QueryCondition::Where);

        assert_eq!(
            Comparison::Equal(DbValue::Int(0)),
            Comparison::Equal(DbValue::Int(0))
        );

        assert_eq!(Direction::From, Direction::From);

        assert_eq!(
            EdgeCountCondition {
                comparison: CountComparison::Equal(0),
                direction: Direction::From
            },
            EdgeCountCondition {
                comparison: CountComparison::Equal(0),
                direction: Direction::From
            }
        );

        assert_eq!(
            KeyValueCondition {
                comparison: Comparison::Equal(DbValue::Int(0)),
                key: DbValue::Int(0)
            },
            KeyValueCondition {
                comparison: Comparison::Equal(DbValue::Int(0)),
                key: DbValue::Int(0)
            }
        );
    }

    #[test]
    fn count_comparison() {
        use CountComparison::Equal;
        use CountComparison::GreaterThan;
        use CountComparison::GreaterThanOrEqual;
        use CountComparison::LessThan;
        use CountComparison::LessThanOrEqual;
        use CountComparison::NotEqual;
        use SearchControl::Continue;
        use SearchControl::Stop;

        assert_eq!(Equal(2).compare(3), Stop(false));
        assert_eq!(Equal(2).compare(2), Stop(true));
        assert_eq!(Equal(2).compare(1), Continue(false));
        assert_eq!(NotEqual(2).compare(3), Continue(true));
        assert_eq!(NotEqual(2).compare(2), Continue(false));
        assert_eq!(NotEqual(2).compare(1), Continue(true));
        assert_eq!(GreaterThan(2).compare(3), Continue(true));
        assert_eq!(GreaterThan(2).compare(2), Continue(false));
        assert_eq!(GreaterThan(2).compare(1), Continue(false));
        assert_eq!(GreaterThanOrEqual(2).compare(3), Continue(true));
        assert_eq!(GreaterThanOrEqual(2).compare(2), Continue(true));
        assert_eq!(GreaterThanOrEqual(2).compare(1), Continue(false));
        assert_eq!(LessThan(2).compare(3), Stop(false));
        assert_eq!(LessThan(2).compare(2), Stop(false));
        assert_eq!(LessThan(2).compare(1), Continue(true));
        assert_eq!(LessThanOrEqual(2).compare(3), Stop(false));
        assert_eq!(LessThanOrEqual(2).compare(2), Stop(true));
        assert_eq!(LessThanOrEqual(2).compare(1), Continue(true));
    }
}
