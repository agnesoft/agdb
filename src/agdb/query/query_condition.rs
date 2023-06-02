use super::query_ids::QueryIds;
use crate::DbKey;
use crate::DbValue;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum QueryCondition {
    And,
    Distance(Comparison),
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

#[allow(dead_code)]
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
    pub comparison: Comparison,
    pub direction: Direction,
}

#[allow(dead_code)]
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
                comparison: Comparison::Equal(DbValue::Int(0)),
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
            comparison: Comparison::Equal(DbValue::Int(0)),
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
                comparison: Comparison::Equal(DbValue::Int(0)),
                direction: Direction::From
            },
            EdgeCountCondition {
                comparison: Comparison::Equal(DbValue::Int(0)),
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
}
