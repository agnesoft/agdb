use super::db_key_value::DbKeyValue;

#[derive(Debug, PartialEq)]
pub struct DbElement {
    pub index: i64,
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
                index: 0,
                values: vec![]
            }
        );
    }
}
