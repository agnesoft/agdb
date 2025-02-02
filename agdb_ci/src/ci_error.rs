use std::fmt::Display;

#[derive(Debug)]
pub(crate) struct CIError {
    pub(crate) description: String,
}

impl<E: Display> From<E> for CIError {
    fn from(error: E) -> Self {
        Self {
            description: error.to_string(),
        }
    }
}
