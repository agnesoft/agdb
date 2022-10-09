use crate::DbError;
use std::error::Error;

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(cause) = &self.cause {
            return Some(cause);
        }

        None
    }
}
