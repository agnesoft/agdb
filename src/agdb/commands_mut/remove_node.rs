use crate::query::query_id::QueryId;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {
    pub id: QueryId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveNode { id: QueryId::Id(0) });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveNode { id: QueryId::Id(0) },
            RemoveNode { id: QueryId::Id(0) }
        );
    }
}
