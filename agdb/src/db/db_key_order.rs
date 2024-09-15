use crate::DbValue;

/// Ordering for search queries
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum DbKeyOrder {
    /// Ascending order (from smallest)
    Asc(DbValue),

    /// Descending order (from largest)
    Desc(DbValue),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", DbKeyOrder::Asc(DbValue::default()));
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
            DbKeyOrder::Asc(DbValue::default()),
            DbKeyOrder::Asc(DbValue::default())
        );
    }
}
