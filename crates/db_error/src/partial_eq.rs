use crate::DbError;

impl PartialEq for DbError {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description && self.cause == other.cause
    }
}
