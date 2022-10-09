use crate::DbError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FMTResult;

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        let location = self.source_location.to_string().replace('\\', "/");
        write!(f, "{} (at {})", self.description, location)
    }
}
