use crate::query::query_id::QueryId;

#[derive(Debug, PartialEq)]
pub struct SelectId {
    pub id: QueryId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            SelectId {
                id: QueryId::from(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            SelectId {
                id: QueryId::from(0)
            },
            SelectId {
                id: QueryId::from(0)
            }
        );
    }
}
