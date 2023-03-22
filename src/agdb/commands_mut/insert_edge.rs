use crate::query::query_id::QueryId;

#[derive(Debug, PartialEq)]
pub struct InsertEdge {
    pub from: QueryId,
    pub to: QueryId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertEdge {
                from: QueryId::Id(0),
                to: QueryId::Id(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertEdge {
                from: QueryId::Id(0),
                to: QueryId::Id(0)
            },
            InsertEdge {
                from: QueryId::Id(0),
                to: QueryId::Id(0)
            }
        );
    }
}
