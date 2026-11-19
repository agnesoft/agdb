use crate::DbElement;
use crate::DbError;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryId;

const DB_ID: &str = "db_id";

/// Trait that allows use of user defined values
/// directly by facilitating translation from/to
/// database primitive types. The names of fields
/// becomes keys of type `String`. Values must be
/// of types that are convertible to/from database
/// primitive types.
///
/// The special name `db_id` can be used to allow
/// direct insertion and select of a user value tied
/// with a particular database element. The field `db_id`
/// should be of a type `Option<T>` where [`T: Into<QueryId>`](QueryId).
/// Typically either `Option<QueryId>` or `Option<DbId>`.
/// The former allows usage of aliases for insertions. Note
/// that when retrieving elements from the database the
/// alias is never returned, only the numerical id.
///
/// The field `db_id` is skipped in the derive macro
/// and used only for the `db_id()` method implementation.
///
/// The trait is derivable using `agdb::DbType`
/// derive macro. When implementing it by hand you
/// can apply additional logic, use different keys
/// and/or their type, skip fields or change their
/// types etc.
///
/// Examples:
///
/// ```
/// use agdb::{DbId, DbType};
///
/// #[derive(DbType)]
/// struct MyValueNoId { key: String, another_key: i32 } // "key": "value", "another_key": 10_i64
///
/// #[derive(DbType)]
/// struct MyValue { db_id: Option<DbId>, key: String } // "key": "value"
/// ```
pub trait DbType: Sized {
    type ValueType;

    /// Returns the database id if present.
    fn db_id(&self) -> Option<QueryId>;

    /// Returns the list of database keys of
    /// this type.
    fn db_keys() -> Vec<DbValue>;

    /// Constructs the user value from the `element`
    /// extracting the values from element `values`
    /// and the `db_id` if that field is present.
    fn from_db_element(element: &DbElement) -> Result<Self::ValueType, DbError>;

    /// Converts the fields (skipping `db_id` if present)
    /// to the database key-values.
    fn to_db_values(&self) -> Vec<DbKeyValue>;

    /// Returns an optional element id. Typically this
    /// would be a name of the type for which this
    /// trait is being implemented. By default returns `None`.
    fn db_element_id() -> Option<DbValue> {
        None
    }
}

impl<T: DbType> DbType for &T {
    type ValueType = T::ValueType;

    fn db_id(&self) -> Option<QueryId> {
        (*self).db_id()
    }

    fn db_keys() -> Vec<DbValue> {
        T::db_keys()
    }

    fn from_db_element(element: &DbElement) -> Result<Self::ValueType, DbError> {
        T::from_db_element(element)
    }

    fn to_db_values(&self) -> Vec<DbKeyValue> {
        (*self).to_db_values()
    }

    fn db_element_id() -> Option<DbValue> {
        T::db_element_id()
    }
}

/// The fields of a value as (key, value) with `None` for an `Option` field
/// without a value and the key `db_id` for the id: what the `DbType` derive
/// makes of the fields of a struct and the clients in other languages of
/// their objects.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq)]
pub struct DbFields(pub Vec<(DbValue, Option<DbValue>)>);

impl DbFields {
    /// The fields with a value.
    pub fn into_db_values(self) -> Vec<DbKeyValue> {
        self.0
            .into_iter()
            .filter_map(|(key, value)| value.map(|value| DbKeyValue { key, value }))
            .collect()
    }

    fn is_db_id(key: &DbValue) -> bool {
        if let DbValue::String(name) = key {
            return name == DB_ID;
        }

        false
    }

    fn query_id(value: &DbValue) -> Option<QueryId> {
        match value {
            DbValue::I64(id) => Some(QueryId::Id(DbId(*id))),
            DbValue::String(alias) => Some(QueryId::Alias(alias.clone())),
            _ => None,
        }
    }
}

impl DbType for DbFields {
    type ValueType = DbFields;

    fn db_id(&self) -> Option<QueryId> {
        let mut id: Option<QueryId> = None;
        self.0.iter().for_each(|field| {
            if let Some(value) = &field.1
                && Self::is_db_id(&field.0)
            {
                id = Self::query_id(value);
            }
        });
        id
    }

    fn db_keys() -> Vec<DbValue> {
        Vec::new()
    }

    fn from_db_element(element: &DbElement) -> Result<Self::ValueType, DbError> {
        let mut fields = vec![(DB_ID.into(), Some(element.id.0.into()))];
        element.values.iter().for_each(|kv| {
            fields.push((kv.key.clone(), Some(kv.value.clone())));
        });
        Ok(DbFields(fields))
    }

    /// The fields with a value other than the id.
    fn to_db_values(&self) -> Vec<DbKeyValue> {
        let mut values = Vec::new();
        self.0.iter().for_each(|field| {
            if let Some(value) = &field.1
                && !Self::is_db_id(&field.0)
            {
                values.push(DbKeyValue {
                    key: field.0.clone(),
                    value: value.clone(),
                });
            }
        });
        values
    }

    fn db_element_id() -> Option<DbValue> {
        None
    }
}

/// Marker trait for user values to get around
/// conflicting trait implementations between database
/// and blanket `std` implementations. Implement it
/// or use the derive macro `agdb::DbTypeMarker`
/// for custom types that are to be used with the database.
///
/// # Examples
///
/// ```rust
/// #[derive(Default, Clone, Copy, Debug)]
/// enum MyEnum {
///   #[default]
///   A,
///   B,
/// }
///
/// impl agdb::DbTypeMarker for MyEnum {}
/// ```
pub trait DbTypeMarker {}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields() -> DbFields {
        DbFields(vec![
            ("key".into(), Some("value".into())),
            ("none".into(), None),
            (DB_ID.into(), Some("alias".into())),
            (1.into(), Some(2.into())),
        ])
    }

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", fields());
    }

    #[test]
    fn derived_from_clone_and_partial_eq() {
        assert_eq!(fields().clone(), fields());
    }

    #[test]
    fn db_fields_into_db_values() {
        assert_eq!(
            fields().into_db_values(),
            vec![
                ("key", "value").into(),
                (DB_ID, "alias").into(),
                (1, 2).into()
            ]
        );
    }

    #[test]
    fn db_fields_to_db_values() {
        assert_eq!(
            fields().to_db_values(),
            vec![("key", "value").into(), (1, 2).into()]
        );
    }

    #[test]
    fn db_fields_db_id() {
        let id = |value: DbValue| DbFields(vec![(DB_ID.into(), Some(value))]).db_id();

        assert_eq!(fields().db_id(), Some(QueryId::Alias("alias".into())));
        assert_eq!(id(DbValue::I64(1)), Some(QueryId::Id(DbId(1))));
        assert_eq!(id(DbValue::U64(2)), None);
        assert_eq!(id(DbValue::F64(1.0.into())), None);
        assert_eq!(DbFields(vec![(DB_ID.into(), None)]).db_id(), None);
    }

    #[test]
    fn db_fields_keys_and_element_id() {
        assert_eq!(DbFields::db_keys(), Vec::<DbValue>::new());
        assert_eq!(DbFields::db_element_id(), None);
    }

    #[test]
    fn db_fields_from_db_element() {
        let element = DbElement {
            id: DbId(3),
            from: DbId(0),
            to: DbId(0),
            values: vec![("key", "value").into()],
        };

        let fields = DbFields::from_db_element(&element).unwrap();

        assert_eq!(fields.db_id(), Some(QueryId::Id(DbId(3))));
        assert_eq!(fields.to_db_values(), element.values);
    }
}
