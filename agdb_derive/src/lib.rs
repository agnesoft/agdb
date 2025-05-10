mod agdb_de_serialize;
mod api_def;
mod impl_def;
mod user_value;

use proc_macro::TokenStream;

/// The derive macro to add `agdb` compatibility
/// to user defined types. It implements [`agdb::DbUserValue`]
/// for the type automatically to allow your type to be read and
/// stored from/to the database.
///
/// If your type contains a field `db_id: Option<agdb::DbId>`
/// it will be treated specially and will additionally allow
/// shorthand inserts/updates of the elements directly.
///
/// All database types are supported. User (custom) types must
/// `impl From<T> for agdb::DbValue` and `impl TryFrom<agdb::DbValue> for T { type Error = agdb::DbError }`.
///
/// `Option` can be used on on any supported type. If a value type is
/// an `Option` and runtime value is `None` the value will NOT be stored
/// in the database. When reading elements if a type contains
/// an `Option` all keys are always retrieved and searched for individual
/// keys as opposed to standard index based read for non-optional types.
///
/// # Examples
///
/// ## Standard
/// ```ignore
/// #[derive(agdb::UserValue)]
/// struct MyValue {
///     num_value: i64,
///     string_value: String,
///     vec_value: Vec<u64>,
/// }
/// ```
///
/// ## With db_id
/// ```ignore
/// #[derive(agdb::UserValue)]
/// struct MyValue {
///     db_id: Option<agdb::DbId>, //this field is useful but not mandatory
///     num_value: i64,
///     string_value: String,
///     vec_value: Vec<u64>,
/// }
/// ```
///
/// ## With optional
/// ```ignore
/// #[derive(agdb::UserValue)]
/// struct MyValue {
///     db_id: Option<agdb::DbId>, //this field is useful but not mandatory
///     num_value: i64,
///     string_value: Option<String>, // Optional value will not be stored if None
///     vec_value: Vec<u64>,
/// }
/// ```
#[proc_macro_derive(UserValue)]
pub fn db_user_value_derive(item: TokenStream) -> TokenStream {
    user_value::db_user_value_derive(item)
}

/// The helper derive macro to add `agdb` compatibility to
/// user defined types. This type provides blank implementation
/// of the `agdb::DbUserValueMarker` trait. This is needed for the
/// vectorized custom values to be compatible with the database
/// as the `From` trait implementation witohut it conflicts
/// with the blanket implementations.
///
/// # Examples
///
/// ```ignore
/// #[derive(agdb::UserValueMarker, Default, Copy, Clone, Debug)]
/// enum MyEnum {
///    #[default]
///    A,
///    B,
/// }
/// ```
#[proc_macro_derive(UserValueMarker)]
pub fn db_user_value_marker_derive(item: TokenStream) -> TokenStream {
    user_value::db_user_value_marker_derive(item)
}

/// The derive macro to add `agdb` platform agnostic serialization
/// support. This is only needed if you want to serialize custom
/// complex data structures and do not want or cannot use serde.
/// It is primarily used internally to serialize the `agdb` data
/// structures.
#[proc_macro_derive(AgdbDeSerialize)]
pub fn agdb_de_serialize(item: TokenStream) -> TokenStream {
    agdb_de_serialize::agdb_de_serialize(item)
}

#[proc_macro_derive(ApiDef)]
pub fn api_def(item: TokenStream) -> TokenStream {
    api_def::api_def(item)
}

#[proc_macro_attribute]
pub fn impl_def(attr: TokenStream, item: TokenStream) -> TokenStream {
    impl_def::impl_def(attr, item)
}
