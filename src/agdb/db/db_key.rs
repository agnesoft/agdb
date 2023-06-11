use super::db_value::DbValue;

pub type DbKey = DbValue;

#[derive(Debug, Clone, PartialEq)]
pub enum DbKeyOrder {
    Asc(DbKey),
    Desc(DbKey),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", DbKeyOrder::Asc(DbKey::default()));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbKeyOrder::Asc(DbKey::default()),
            DbKeyOrder::Asc(DbKey::default())
        );
    }
}
