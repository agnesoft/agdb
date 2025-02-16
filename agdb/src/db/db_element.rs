use crate::DbId;
use crate::DbKeyValue;

/// Database element used in [`QueryResult`]
/// that represents a node or an edge.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct DbElement {
    /// Element id.
    pub id: DbId,

    /// Origin node id.
    pub from: Option<DbId>,

    /// Destination node id.
    pub to: Option<DbId>,

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
                id: DbId(0),
                from: None,
                to: None,
                values: vec![]
            }
        );
    }
    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbElement {
                id: DbId(0),
                from: None,
                to: None,
                values: vec![]
            },
            DbElement {
                id: DbId(0),
                from: None,
                to: None,
                values: vec![]
            }
        );
    }

    #[test]
    fn derived_from_clone() {
        let element = DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: vec![],
        };
        let other = element.clone();
        assert_eq!(element, other);
    }

    #[test]
    fn derived_from_partial_ord() {
        let element = DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: vec![],
        };
        let other = DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![],
        };
        assert!(element < other);
    }

    #[test]
    fn derived_from_ord() {
        let element = DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: vec![],
        };
        assert_eq!(element.cmp(&element), std::cmp::Ordering::Equal);
    }
}
