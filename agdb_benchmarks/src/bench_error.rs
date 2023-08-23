#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct BenchError {
    pub(crate) description: String,
}

impl<T: std::error::Error> From<T> for BenchError {
    fn from(value: T) -> Self {
        Self {
            description: value.to_string(),
        }
    }
}
