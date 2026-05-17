use crate::api_error::AgdbApiError;

pub type AgdbApiResult<T> = Result<T, AgdbApiError>;
