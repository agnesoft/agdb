use super::db_key_value::DbKeyValue;
use crate::DbId;

#[derive(Debug, PartialEq)]
pub struct DbElement {
    pub id: DbId,
    pub values: Vec<DbKeyValue>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            DbElement {
                id: DbId(0),
                values: vec![]
            }
        );
    }
    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbElement {
                id: DbId(0),
                values: vec![]
            },
            DbElement {
                id: DbId(0),
                values: vec![]
            }
        );
    }
}
