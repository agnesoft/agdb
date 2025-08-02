#[derive(Debug)]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
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
                .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR)
                .as_u16(),
            description: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for AgdbApiError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            status: 0,
            description: value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _ = format!(
            "{:?}",
            AgdbApiError {
                status: 0,
                description: "test".to_string(),
            }
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            format!(
                "{}",
                AgdbApiError {
                    status: 0,
                    description: "test".to_string(),
                }
            ),
            "0: test"
        );
    }

    #[test]
    fn from_reqwest_error() {
        let error = reqwest::ClientBuilder::new()
            .user_agent("\0")
            .build()
            .unwrap_err();
        assert_eq!(AgdbApiError::from(error).description, "builder error");
    }

    #[test]
    fn from_serde_json_eror() {
        let error = serde_json::from_str::<()>("").unwrap_err();
        assert_eq!(
            AgdbApiError::from(error).description,
            "EOF while parsing a value at line 1 column 0"
        );
    }
}
