#[derive(Debug)]
pub struct AgdbApiError {
    pub status: u16,
    pub description: String,
}

impl std::error::Error for AgdbApiError {}

impl std::fmt::Display for AgdbApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.description)
    }
}
