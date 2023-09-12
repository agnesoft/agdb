use crate::DbValue;

/// Alias to `DbValue`
pub type DbKey = DbValue;

/// Ordering for search queries
#[derive(Debug, Clone, PartialEq)]
pub enum DbKeyOrder {
    /// Ascending order (from smallest)
    Asc(DbKey),

    /// Descending order (from largest)
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
    #[allow(clippy::redundant_clone)]
    fn derived_from_clone() {
        let order = DbKeyOrder::Asc(1.into());
        let other = order.clone();

        assert_eq!(order, other);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbKeyOrder::Asc(DbKey::default()),
            DbKeyOrder::Asc(DbKey::default())
        );
    }
}
