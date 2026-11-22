/// The conversion of a database of an older version done when it was
/// opened, see [`DbImpl::migration()`](crate::DbImpl::migration).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DbMigration {
    /// The version of the database before the conversion, `0` for the
    /// databases created by agdb 0.10.0 and older.
    pub from: u64,

    /// The version of the database after the conversion.
    pub to: u64,

    /// The inconsistencies of the data the conversion found and fixed.
    pub fixed: Vec<String>,
}
