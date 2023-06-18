use super::query_id::QueryId;
use crate::graph_search::SearchControl;
use crate::DbKey;
use crate::DbValue;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryConditionLogic {
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryConditionModifier {
    None,
    Not,
    NotBeyond,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryConditionData {
    Distance { value: CountComparison },
    Edge,
    EdgeCount { value: CountComparison },
    EdgeCountFrom { value: CountComparison },
    EdgeCountTo { value: CountComparison },
    Ids { values: Vec<QueryId> },
    KeyValue { key: DbKey, value: Comparison },
    Keys { values: Vec<DbKey> },
    Node,
    Where { conditions: Vec<QueryCondition> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryCondition {
    pub logic: QueryConditionLogic,
    pub modifier: QueryConditionModifier,
    pub data: QueryConditionData,
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

impl Comparison {
    pub(crate) fn compare(&self, left: &DbValue) -> bool {
        match self {
            Comparison::Equal(right) => left == right,
            Comparison::GreaterThan(right) => left > right,
            Comparison::GreaterThanOrEqual(right) => left >= right,
            Comparison::LessThan(right) => left < right,
            Comparison::LessThanOrEqual(right) => left <= right,
            Comparison::NotEqual(right) => left != right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Edge,
            }
        );

        format!("{:?}", Comparison::Equal(DbValue::Int(0)));

        format!("{:?}", CountComparison::Equal(0));
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn derived_from_clone() {
        let left = QueryCondition {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            data: QueryConditionData::Edge,
        };
        let right = left.clone();
        assert_eq!(left, right);

        let left = Comparison::Equal(DbValue::Int(0));
        let right = left.clone();
        assert_eq!(left, right);

        let left = CountComparison::Equal(0);
        let right = left.clone();
        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Edge,
            },
            QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::Edge,
            }
        );

        assert_eq!(
            Comparison::Equal(DbValue::Int(0)),
            Comparison::Equal(DbValue::Int(0))
        );

        assert_eq!(CountComparison::Equal(0), CountComparison::Equal(0));
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
