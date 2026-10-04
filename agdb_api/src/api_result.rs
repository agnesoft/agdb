use crate::api_error::AgdbApiError;

/// Convenience result type for API operations.
pub type AgdbApiResult<T> = Result<T, AgdbApiError>;
