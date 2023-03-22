use crate::query::query_id::QueryId;

#[derive(Debug, PartialEq)]
pub struct RemoveEdge {
    pub id: QueryId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveEdge { id: QueryId::Id(0) });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveEdge { id: QueryId::Id(0) },
            RemoveEdge { id: QueryId::Id(0) }
        );
    }
}
