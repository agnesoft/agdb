use super::query_ids::QueryIds;
use crate::graph_search::SearchControl;
use crate::DbKey;
use crate::DbValue;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryConditionLogic {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryConditionModifier {
    None,
    Not,
    NotBeyond,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryCondition {
    Distance {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
        value: CountComparison,
    },
    Edge {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
    },
    EdgeCount {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
        value: CountComparison,
    },
    EdgeCountFrom {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
        value: CountComparison,
    },
    EdgeCountTo {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
        value: CountComparison,
    },
    Ids {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
        values: QueryIds,
    },
    KeyValue {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
        key: DbKey,
        value: Comparison,
    },
    Keys {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
        values: Vec<DbKey>,
    },
    Node {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
    },
    Where {
        logic: QueryConditionLogic,
        modifier: QueryConditionModifier,
    },
    EndWhere,
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
        format!("{:?}", QueryCondition::EndWhere);
        format!("{:?}", Comparison::Equal(DbValue::Int(0)));
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn derived_from_clone() {
        let left = QueryCondition::EndWhere;
        let right = left.clone();
        assert_eq!(left, right);

        let left = Comparison::Equal(DbValue::Int(0));
        let right = left.clone();
        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(QueryCondition::EndWhere, QueryCondition::EndWhere);

        assert_eq!(
            Comparison::Equal(DbValue::Int(0)),
            Comparison::Equal(DbValue::Int(0))
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
