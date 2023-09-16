use crate::graph_search::SearchControl;
use crate::DbKey;
use crate::DbValue;
use crate::QueryId;

/// Logical operator for query conditions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryConditionLogic {
    /// Logical AND (&&)
    And,

    /// Logical Or (||)
    Or,
}

/// Query condition modifier
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QueryConditionModifier {
    /// No modifier
    None,

    /// Continues the search beyond the current element
    /// if the condition being modified passes.
    Beyond,

    /// Reversal of the result (equivalent to `!`).
    Not,

    /// Stops the search beyond the current element
    /// if the condition being modified passes.
    NotBeyond,
}

/// Query condition data
#[derive(Debug, Clone, PartialEq)]
pub enum QueryConditionData {
    /// Distance from the search origin. Takes count comparison
    /// (e.g. Equal, GreaterThan).
    Distance(CountComparison),

    /// Is the current element an edge? I.e. `id < 0`.
    Edge,

    /// Tests number of edges (from+to) of the current element.
    /// Only nodes will pass. Self-referential edges are
    /// counted twice. Takes count comparison
    /// (e.g. Equal, GreaterThan).
    EdgeCount(CountComparison),

    /// Tests the number of outgoing edges (from) of the
    /// current element. Takes count comparison
    /// (e.g. Equal, GreaterThan).
    EdgeCountFrom(CountComparison),

    /// Tests the number of incoming edges (to) of the
    /// current element. Takes count comparison
    /// (e.g. Equal, GreaterThan).
    EdgeCountTo(CountComparison),

    /// Tests if the current id is in the list of ids.
    Ids(Vec<QueryId>),

    /// Tests if the current element has a property `key`
    /// with a value that evaluates true against `comparison`.
    KeyValue {
        /// Property key
        key: DbKey,

        /// Comparison operator (e.g. Equal, GreaterThan etc.)
        value: Comparison,
    },

    /// Test if the current element has **all** of the keys listed.
    Keys(Vec<DbKey>),

    /// Is the current element a node? I.e. `0 < id`.
    Node,

    /// Nested list of conditions (equivalent to brackets).
    Where(Vec<QueryCondition>),
}

/// Query condition. The condition consists of
/// `data`, logic operator and a modifier.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryCondition {
    /// Logic operator (e.g. And, Or)
    pub logic: QueryConditionLogic,

    /// Condition modifier (e.g. None, Beyond, Not, NotBeyond)
    pub modifier: QueryConditionModifier,

    /// Condition data (or type) defining what type
    /// of validation is to be performed.
    pub data: QueryConditionData,
}

/// Comparison of unsigned integers (`u64`) used
/// by `distance()` and `edge_count*()` conditions. Supports
/// the usual set of named comparisons: `==, !=, <, <=, >, =>`.
#[derive(Debug, Clone, PartialEq)]
pub enum CountComparison {
    /// property == this
    Equal(u64),

    /// property > this
    GreaterThan(u64),

    /// property >= this
    GreaterThanOrEqual(u64),

    /// property < this
    LessThan(u64),

    /// property <= this
    LessThanOrEqual(u64),

    /// property != this
    NotEqual(u64),
}

/// Comparison of database values ([`DbValue`]) used
/// by `key()` condition. Supports
/// the usual set of named comparisons: `==, !=, <, <=, >, =>`
/// plus `contains()`. The comparisons are type
/// strict except for the `contains` comparison
/// which allows vectorized version of the base type. Notably
/// however it does not support the `bytes` and integral types
/// where the "contains" makes little sense (i.e. does 3 contain 1?).
#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    /// property == this
    Equal(DbValue),

    /// property > this
    GreaterThan(DbValue),

    /// property >= this
    GreaterThanOrEqual(DbValue),

    /// property < this
    LessThan(DbValue),

    /// property <= this
    LessThanOrEqual(DbValue),

    /// property != this
    NotEqual(DbValue),

    /// property.contains(this)
    Contains(DbValue),
}

impl CountComparison {
    pub(crate) fn compare_distance(&self, right: u64) -> SearchControl {
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

    pub(crate) fn compare(&self, left: u64) -> bool {
        match self {
            CountComparison::Equal(right) => left == *right,
            CountComparison::GreaterThan(right) => left > *right,
            CountComparison::GreaterThanOrEqual(right) => left >= *right,
            CountComparison::LessThan(right) => left < *right,
            CountComparison::LessThanOrEqual(right) => left <= *right,
            CountComparison::NotEqual(right) => left != *right,
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
            Comparison::Contains(right) => match (left, right) {
                (DbValue::String(left), DbValue::String(right)) => left.contains(right),
                (DbValue::String(left), DbValue::VecString(right)) => {
                    right.iter().all(|x| left.contains(x))
                }
                (DbValue::VecI64(left), DbValue::I64(right)) => left.contains(right),
                (DbValue::VecI64(left), DbValue::VecI64(right)) => {
                    right.iter().all(|x| left.contains(x))
                }
                (DbValue::VecU64(left), DbValue::U64(right)) => left.contains(right),
                (DbValue::VecU64(left), DbValue::VecU64(right)) => {
                    right.iter().all(|x| left.contains(x))
                }
                (DbValue::VecF64(left), DbValue::F64(right)) => left.contains(right),
                (DbValue::VecF64(left), DbValue::VecF64(right)) => {
                    right.iter().all(|x| left.contains(x))
                }
                (DbValue::VecString(left), DbValue::String(right)) => left.contains(right),
                (DbValue::VecString(left), DbValue::VecString(right)) => {
                    right.iter().all(|x| left.contains(x))
                }
                _ => false,
            },
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

        format!("{:?}", Comparison::Equal(DbValue::I64(0)));

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

        let left = Comparison::Equal(DbValue::I64(0));
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
            Comparison::Equal(DbValue::I64(0)),
            Comparison::Equal(DbValue::I64(0))
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

        assert_eq!(Equal(2).compare_distance(3), Stop(false));
        assert_eq!(Equal(2).compare_distance(2), Stop(true));
        assert_eq!(Equal(2).compare_distance(1), Continue(false));
        assert_eq!(NotEqual(2).compare_distance(3), Continue(true));
        assert_eq!(NotEqual(2).compare_distance(2), Continue(false));
        assert_eq!(NotEqual(2).compare_distance(1), Continue(true));
        assert_eq!(GreaterThan(2).compare_distance(3), Continue(true));
        assert_eq!(GreaterThan(2).compare_distance(2), Continue(false));
        assert_eq!(GreaterThan(2).compare_distance(1), Continue(false));
        assert_eq!(GreaterThanOrEqual(2).compare_distance(3), Continue(true));
        assert_eq!(GreaterThanOrEqual(2).compare_distance(2), Continue(true));
        assert_eq!(GreaterThanOrEqual(2).compare_distance(1), Continue(false));
        assert_eq!(LessThan(2).compare_distance(3), Stop(false));
        assert_eq!(LessThan(2).compare_distance(2), Stop(false));
        assert_eq!(LessThan(2).compare_distance(1), Continue(true));
        assert_eq!(LessThanOrEqual(2).compare_distance(3), Stop(false));
        assert_eq!(LessThanOrEqual(2).compare_distance(2), Stop(true));
        assert_eq!(LessThanOrEqual(2).compare_distance(1), Continue(true));
    }

    #[test]
    fn contains() {
        assert!(Comparison::Contains("abc".into()).compare(&"0abc123".into()));
        assert!(!Comparison::Contains("abcd".into()).compare(&"0abc123".into()));

        assert!(
            Comparison::Contains(vec!["ab".to_string(), "23".to_string()].into())
                .compare(&"0abc123".into())
        );
        assert!(
            !Comparison::Contains(vec!["abcd".to_string(), "23".to_string()].into())
                .compare(&"0abc123".into())
        );

        assert!(Comparison::Contains(1.into()).compare(&vec![2, 1, 3].into()));
        assert!(!Comparison::Contains(4.into()).compare(&vec![2, 1, 3].into()));

        assert!(Comparison::Contains(vec![2, 3].into()).compare(&vec![2, 1, 3].into()));
        assert!(!Comparison::Contains(vec![2, 4].into()).compare(&vec![2, 1, 3].into()));

        assert!(Comparison::Contains(1_u64.into()).compare(&vec![2_u64, 1_u64, 3_u64].into()));
        assert!(!Comparison::Contains(4_u64.into()).compare(&vec![2_u64, 1_u64, 3_u64].into()));

        assert!(Comparison::Contains(vec![2_u64, 3_u64].into())
            .compare(&vec![2_u64, 1_u64, 3_u64].into()));
        assert!(!Comparison::Contains(vec![2_u64, 4_u64].into())
            .compare(&vec![2_u64, 1_u64, 3_u64].into()));

        assert!(Comparison::Contains(1.1.into()).compare(&vec![2.1, 1.1, 3.3].into()));
        assert!(!Comparison::Contains(4.2.into()).compare(&vec![2.1, 1.1, 3.3].into()));

        assert!(Comparison::Contains(vec![2.2, 3.3].into()).compare(&vec![2.2, 1.1, 3.3].into()));
        assert!(!Comparison::Contains(vec![2.2, 4.4].into()).compare(&vec![2.2, 1.1, 3.3].into()));

        assert!(Comparison::Contains("abc".into())
            .compare(&vec!["0".to_string(), "abc".to_string(), "123".to_string()].into()));
        assert!(!Comparison::Contains("abcd".into())
            .compare(&vec!["0".to_string(), "abc".to_string(), "123".to_string()].into()));

        assert!(
            Comparison::Contains(vec!["abc".to_string(), "123".to_string()].into())
                .compare(&vec!["0".to_string(), "abc".to_string(), "123".to_string()].into())
        );
        assert!(
            !Comparison::Contains(vec!["abcd".to_string(), "123".to_string()].into())
                .compare(&vec!["0".to_string(), "abc".to_string(), "123".to_string()].into())
        );

        assert!(!Comparison::Contains("abc".into()).compare(&1.into()));
    }
}
