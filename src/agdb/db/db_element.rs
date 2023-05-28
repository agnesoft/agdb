use super::db_key_value::DbKeyValue;
use crate::DbId;

#[derive(Debug, PartialEq)]
pub struct DbElement {
    pub index: DbId,
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
                index: DbId(0),
                values: vec![]
            }
        );
    }
    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbElement {
                index: DbId(0),
                values: vec![]
            },
            DbElement {
                index: DbId(0),
                values: vec![]
            }
        );
    }
}
