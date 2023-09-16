use crate::DbElement;
use crate::DbError;
use crate::DbId;
use crate::DbKey;
use crate::DbKeyValue;

/// Trait that allows use of user defined values
/// directly by facilitating translation from/to
/// database primitive types. The names of fields
/// becomes keys of type `String`. Values must be
/// of types that are convertible to/from database
/// primitive types.
///
/// The special type `db_id` of type [`Option<DbId>`](DbId)
/// can be used to allow direct insertion and select
/// of a user value tied with a particular database
/// element. The field `db_id` is skipped in the derive
/// macro and used only for the `db_id()` method.
///
/// The trait is derivable using `agdb::UserValue`
/// derive macro. When implementing it by hand you
/// can apply additional logic, use different keys
/// and/or their type, skip fields or change their
/// types etc.
///
/// Examples:
///
/// ```
/// use agdb::{DbId, UserValue};
///
/// #[derive(UserValue)]
/// struct MyValueNoId { key: String, another_key: i32 } // "key": "value", "another_key": 10_i64
///
/// #[derive(UserValue)]
/// struct MyValue { db_id: Option<DbId>, key: String } // "key": "value"
/// ```
pub trait DbUserValue: Sized {
    /// Returns the database id if present.
    fn db_id(&self) -> Option<DbId>;

    /// Returns the list of database keys of
    /// this type.
    fn db_keys() -> Vec<DbKey>;

    /// Constructs the user value from the `element`
    /// extracting the values from element `values`
    /// and the `db_id` if that field is present.
    fn from_db_element(element: &DbElement) -> Result<Self, DbError>;

    /// Converts the fields (skipping `db_id` if present)
    /// to the database key-values.
    fn to_db_values(&self) -> Vec<DbKeyValue>;
}
