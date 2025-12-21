mod api_def;
mod db_serialize;
mod db_type;
mod db_value;

use proc_macro::TokenStream;

/// The derive macro to add `agdb` compatibility
/// to user defined types. It implements [`agdb::UserDbType`]
/// for the type automatically to allow your type to be read and
/// stored from/to the database.
///
/// If your type contains a field `db_id: Option<agdb::DbId>`
/// it will be treated specially and will additionally allow
/// shorthand inserts/updates of the elements directly.
///
/// All database types are supported. User (custom) types must either
/// `impl From<T> for agdb::DbValue` and `impl TryFrom<agdb::DbValue> for T { type Error = agdb::DbError }`
/// or you must use `#[agdb(flatten)]` attribute to merge the fields in a flat list (transitively). Flattened
/// types must themselves be derived from `agdb::DbType` (or implement `agdb::UserDbType`)
///
/// NOTE: if the nested struct(s) have keys of the same name the value will
/// be overwritten by the last encountered field of the same name (transitively).
/// Use `#[agdb(rename = "new_name")]` to disambiguate.
///
/// `Option` can be used on on any supported type. If a value type is
/// an `Option` and runtime value is `None` the value will NOT be stored
/// in the database. When reading elements if a type contains
/// an `Option` all keys are always retrieved and searched for individual
/// keys.
///
/// # Examples
///
/// ## Standard
/// ```ignore
/// #[derive(DbType)]
/// struct MyValue {
///     num_value: i64,
///     string_value: String,
///     vec_value: Vec<u64>,
/// }
/// ```
///
/// ## With db_id
/// ```ignore
/// #[derive(DbType)]
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
/// #[derive(DbType)]
/// struct MyValue {
///     db_id: Option<agdb::DbId>, //this field is useful but not mandatory
///     num_value: i64,
///     string_value: Option<String>, // Optional value will not be stored if None
///     vec_value: Vec<u64>,
/// }
/// ```
///
/// ## Flatten
/// ```ignore
/// #[derive(DbType)]
/// struct MyValue {
///     num_value: i64,
///     #[agdb(flatten)]
///     nested: NestedStruct,
/// }
/// ```
#[proc_macro_derive(DbType, attributes(agdb))]
pub fn user_db_type_derive(item: TokenStream) -> TokenStream {
    db_type::db_type_derive(item)
}

/// The same as DbType but additionally implements `DbType::db_element_id()` allowing
/// more streamlined selection and search of strongly typed elements. This derive adds
/// additional element property to the element representation in the database of type
/// `(String, String)`, i.e. `("db_element_id", <usertypename>.to_string())`.
#[proc_macro_derive(DbElement, attributes(agdb))]
pub fn user_db_element_derive(item: TokenStream) -> TokenStream {
    db_type::db_element_derive(item)
}

/// The helper derive macro to add `agdb` compatibility to
/// user defined types. This type provides blank implementation
/// of the `agdb::DbTypeMarker` trait. This is needed for the
/// vectorized custom values to be compatible with the database
/// as the `From` trait implementation without it conflicts
/// with the blanket implementations.
///
/// # Examples
///
/// ```ignore
/// #[derive(agdb::DbTypeMarker, Default, Copy, Clone, Debug)]
/// enum MyEnum {
///    #[default]
///    A,
///    B,
/// }
/// ```
#[proc_macro_derive(DbTypeMarker)]
pub fn user_db_type_marker_derive(item: TokenStream) -> TokenStream {
    db_type::db_type_marker_derive(item)
}

/// The derive macro to add `agdb` platform agnostic serialization
/// support. This is only needed if you want to serialize custom
/// complex data structures and do not want or cannot use serde.
/// It is primarily used internally to serialize the `agdb` data
/// structures.
#[proc_macro_derive(DbSerialize)]
pub fn agdb_de_serialize(item: TokenStream) -> TokenStream {
    db_serialize::db_serialize(item)
}

/// The derive macro to allow automatically serializing user types
/// into `DbValue::Bytes`. Useful when deriving `DbType` for
/// a custom type with nested custom types or enums as it avoids
/// the need to manually implement From/TryFrom for such nested types.
/// It does additionally support enums and vectorized types (`Vec<T>`).
///
/// NOTE: It requires both `DbTypeMarker` and `AgdbSerialize` traits
/// to be implemented. You can derive them with `#[derive(DbTypeMarker, AgdbSerialize)]`.
#[proc_macro_derive(DbValue)]
pub fn user_db_value_derive(item: TokenStream) -> TokenStream {
    db_value::user_db_value_derive(item)
}

#[proc_macro_derive(TypeDef)]
pub fn type_def(item: TokenStream) -> TokenStream {
    api_def::type_def_impl(item)
}

#[proc_macro_derive(TypeDefImpl)]
pub fn type_def_impl(item: TokenStream) -> TokenStream {
    api_def::type_def_impl_impl(item)
}

#[proc_macro_attribute]
pub fn impl_def(_attr: TokenStream, item: TokenStream) -> TokenStream {
    api_def::impl_def_impl(item)
}
