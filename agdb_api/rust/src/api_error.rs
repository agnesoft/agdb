use reqwest::StatusCode;

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

impl From<reqwest::Error> for AgdbApiError {
    fn from(error: reqwest::Error) -> Self {
        Self {
            status: error
                .status()
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                .as_u16(),
            description: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for AgdbApiError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            description: value.to_string(),
        }
    }
}
