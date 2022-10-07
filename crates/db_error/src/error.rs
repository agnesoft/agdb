use crate::DbError;

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(cause) = &self.cause {
            return Some(cause);
        }

        None
    }
}
