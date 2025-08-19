use crate::DbElement;
use crate::DbError;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryId;

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
/// use agdb::{DbId, agdb::DbType};
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
