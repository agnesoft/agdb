use crate::DbValue;

/// Ordering for search queries
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDefImpl))]
pub enum DbKeyOrder {
    /// Ascending order (from smallest)
    Asc(DbValue),

    /// Descending order (from largest)
    Desc(DbValue),
}

#[cfg_attr(feature = "api", derive(agdb::TypeDefImpl))]
pub struct DbKeyOrders(pub Vec<DbKeyOrder>);

impl From<Vec<DbKeyOrder>> for DbKeyOrders {
    fn from(orders: Vec<DbKeyOrder>) -> Self {
        Self(orders)
    }
}

impl From<&[DbKeyOrder]> for DbKeyOrders {
    fn from(orders: &[DbKeyOrder]) -> Self {
        Self(orders.to_vec())
    }
}

impl<const N: usize> From<[DbKeyOrder; N]> for DbKeyOrders {
    fn from(orders: [DbKeyOrder; N]) -> Self {
        Self(orders.to_vec())
    }
}

impl From<DbKeyOrder> for DbKeyOrders {
    fn from(value: DbKeyOrder) -> Self {
        Self(vec![value])
    }
}

impl From<&DbKeyOrder> for DbKeyOrders {
    fn from(value: &DbKeyOrder) -> Self {
        Self(vec![value.clone()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgdbSerialize;

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

    #[test]
    fn db_key_orders() {
        let _orders = DbKeyOrders::from(vec![DbKeyOrder::Asc(1.into())]);
        let _orders = DbKeyOrders::from([DbKeyOrder::Asc(1.into())].as_slice());
        let _orders = DbKeyOrders::from([DbKeyOrder::Asc(1.into())]);
        let _orders = DbKeyOrders::from(DbKeyOrder::Asc(1.into()));
        let _orders = DbKeyOrders::from(&DbKeyOrder::Asc(1.into()));
    }

    #[test]
    fn derive_serialization() {
        let order = DbKeyOrder::Asc(1.into());
        let serialized = order.serialize();
        let deserialized = DbKeyOrder::deserialize(&serialized).unwrap();
        assert_eq!(order, deserialized);
    }
}
