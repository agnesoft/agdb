use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::AgdbSerialize;
use agdb_api::BINARY_CONTENT_TYPE;
use axum::Json;
use axum::body::Bytes;
use axum::extract::FromRequest;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::de::DeserializeOwned;

// Cluster traffic uses agdb's binary serialization instead of JSON, which has
// no byte type and sent binary payloads as arrays of numbers (~3.5x larger).
// The version is part of the content type so a future format change is
// negotiated the same way as binary vs JSON.
pub(crate) const CONTENT_TYPE: &str = "application/vnd.agdb.cluster.v1";

pub(crate) fn encode<T: AgdbSerialize>(value: &T) -> Vec<u8> {
    value.serialize()
}

pub(crate) fn decode<T: AgdbSerialize>(bytes: &[u8]) -> ServerResult<T> {
    let value = T::deserialize(bytes)?;
    let size = value.serialized_size();

    if size != bytes.len() as u64 {
        return Err(ServerError::from(format!(
            "message has {} bytes, expected {size}",
            bytes.len()
        )));
    }

    Ok(value)
}

/// Request body accepted as JSON or as agdb's binary serialization
/// ([`BINARY_CONTENT_TYPE`]). The reply should use the same encoding.
pub(crate) struct Negotiated<T> {
    pub(crate) value: T,
    pub(crate) binary: bool,
}

impl<S: Send + Sync, T: DeserializeOwned + AgdbSerialize> FromRequest<S> for Negotiated<T> {
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let binary = request
            .headers()
            .get(header::CONTENT_TYPE)
            .is_some_and(|value| value == BINARY_CONTENT_TYPE);

        let value = if binary {
            let bytes = Bytes::from_request(request, state)
                .await
                .map_err(IntoResponse::into_response)?;
            decode(&bytes).map_err(|e| (StatusCode::BAD_REQUEST, e.description).into_response())?
        } else {
            Json::<T>::from_request(request, state)
                .await
                .map_err(IntoResponse::into_response)?
                .0
        };

        Ok(Self { value, binary })
    }
}

pub(crate) fn reply<T: serde::Serialize + AgdbSerialize>(value: T, binary: bool) -> Response {
    if binary {
        (
            [(header::CONTENT_TYPE, BINARY_CONTENT_TYPE)],
            encode(&value),
        )
            .into_response()
    } else {
        Json(value).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() -> ServerResult<()> {
        let value = vec![("owner".to_string(), Some(7_u64))];
        assert_eq!(
            decode::<Vec<(String, Option<u64>)>>(&encode(&value))?,
            value
        );
        Ok(())
    }

    #[test]
    fn rejects_malformed_input() {
        let mut bytes = encode(&7_u64);
        bytes.push(0);

        assert!(decode::<u64>(&bytes).is_err());
        assert!(decode::<u64>(&bytes[..4]).is_err());
        assert!(decode::<Vec<u64>>(&encode(&u64::MAX)).is_err());
    }
}
