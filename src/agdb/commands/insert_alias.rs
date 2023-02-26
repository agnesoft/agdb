use crate::query::query_id::QueryId;

#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    pub id: Option<QueryId>,
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
                id: None,
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias {
                id: None,
                alias: String::new()
            },
            InsertAlias {
                id: None,
                alias: String::new()
            }
        );
    }
}
