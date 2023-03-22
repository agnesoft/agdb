use crate::query::query_id::QueryId;

#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    pub id: QueryId,
    pub alias: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertAlias {
                id: QueryId::Id(0),
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias {
                id: QueryId::Id(0),
                alias: String::new()
            },
            InsertAlias {
                id: QueryId::Id(0),
                alias: String::new()
            }
        );
    }
}
