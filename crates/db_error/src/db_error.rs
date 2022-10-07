#[derive(Debug)]
pub struct DbError {
    pub description: String,
    pub cause: Option<Box<DbError>>,
    pub source_location: std::panic::Location<'static>,
}

impl DbError {
    pub fn caused_by(mut self, error: DbError) -> DbError {
        self.cause = Some(Box::new(error));

        self
    }
}
