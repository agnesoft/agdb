/// Amend operation for insert/remove value queries.
/// Controls how values are applied to existing properties.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub enum Amend {
    /// Default behavior: overwrite the existing value.
    #[default]
    None,

    /// Add to / increment / append / concatenate the existing value.
    /// For numerics: adds the new value to the existing one.
    /// For strings: concatenates the new string to the existing one.
    /// For bytes: appends the new bytes to the existing byte array.
    /// For vec types: extends the existing list with the new elements.
    /// A scalar value can also be pushed onto the matching vec type.
    /// If the key does not exist, falls back to a regular insert.
    Add,

    /// Remove from / decrement the existing value.
    /// For numerics: subtracts the new value from the existing one.
    /// For strings: removes all occurrences of the substring.
    /// For bytes: error is returned as removing from a byte array is not semantically clear operation.
    /// For vec types: removes first occurrence of each element.
    /// A scalar value can also be removed from the matching vec type.
    /// If the key does not exist, this is a no-op.
    Remove,
}

impl Amend {
    /// Returns true if this is `Amend::None` (the default).
    /// Used by serde `skip_serializing_if` to omit the field
    /// when it's the default value.
    pub fn is_none(&self) -> bool {
        matches!(self, Amend::None)
    }
}
