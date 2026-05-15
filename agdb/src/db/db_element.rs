use crate::DbId;
use crate::DbKeyValue;

/// Database element used in [`QueryResult`]
/// that represents a node or an edge.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct DbElement {
    /// Element id.
    pub id: DbId,

    /// Origin node id or first outgoing edge id.
    pub from: DbId,

    /// Destination node id or first incoming edge id.
    pub to: DbId,

    /// List of key-value pairs associated with the element.
    pub values: Vec<DbKeyValue>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derived_from_debug() {
        let _ = format!(
            "{:?}",
            DbElement {
                id: DbId::default(),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![]
            }
        );
    }
    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbElement {
                id: DbId::default(),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![]
            },
            DbElement {
                id: DbId::default(),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![]
            }
        );
    }

    #[test]
    fn derived_from_clone() {
        let element = DbElement {
            id: DbId::default(),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![],
        };
        let other = element.clone();
        assert_eq!(element, other);
    }

    #[test]
    fn derived_from_partial_ord() {
        let element = DbElement {
            id: DbId::default(),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![],
        };
        let other = DbElement {
            id: DbId(1),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![],
        };
        assert!(element < other);
    }

    #[test]
    fn derived_from_ord() {
        let element = DbElement {
            id: DbId::default(),
            from: DbId::default(),
            to: DbId::default(),
            values: vec![],
        };
        assert_eq!(element.cmp(&element), std::cmp::Ordering::Equal);
    }
}
