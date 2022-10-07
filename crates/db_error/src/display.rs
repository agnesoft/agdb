use crate::DbError;

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self.source_location.to_string().replace('\\', "/");
        write!(f, "{} (at {})", self.description, location)
    }
}
