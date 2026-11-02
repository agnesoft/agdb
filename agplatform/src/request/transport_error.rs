use super::RequestOutput;

/// Enabled by the `request` feature flag.
///
/// The failure to send a request or to receive its response, the source of the
/// error returned from
/// [`Request::send_blocking`](crate::Request::send_blocking).
///
/// # Examples
///
/// ```rust
/// use agplatform::{BlockingRequestBuilderExt, Request, TestRequest, TransportError, TransportErrorKind};
///
/// let request = TestRequest::new();
/// request.set_transport_error("", TransportErrorKind::Connect);
///
/// let error = request.blocking_client().unwrap().get("https://example.com").send_blocking(&request).unwrap_err();
///
/// assert_eq!(error.source_as::<TransportError>().unwrap().kind(), TransportErrorKind::Connect);
/// ```
#[derive(Debug)]
pub struct TransportError {
    kind: TransportErrorKind,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    output: Option<RequestOutput>,
}

/// Enabled by the `request` feature flag.
///
/// Classifies [`TransportError`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportErrorKind {
    /// The connection could not be established (including its timeout), so
    /// the request was not sent and retrying it is always safe.
    Connect,
    /// The request or the response timed out.
    Timeout,
    /// Any other failure, e.g. the connection was closed before the whole
    /// response was received.
    Other,
}

impl TransportError {
    /// [internal]
    ///
    /// Creates an error of `kind` without a source, e.g. a simulated one.
    pub(crate) fn new(kind: TransportErrorKind) -> Self {
        Self {
            kind,
            source: None,
            output: None,
        }
    }

    /// Returns the kind of the error.
    pub fn kind(&self) -> TransportErrorKind {
        self.kind
    }

    /// Returns the response whose body failed to be received, e.g. to resume a
    /// download with its headers, `None` when the request failed before the
    /// response arrived. Its body is the part received, empty when it was
    /// passed to a sink (see
    /// [`Request::send_blocking_with_sink`](crate::Request::send_blocking_with_sink)) or not
    /// streamed (see
    /// [`RequestOptions::keep_partial_body`](crate::RequestOptions::keep_partial_body)).
    pub fn output(&self) -> Option<&RequestOutput> {
        self.output.as_ref()
    }

    /// [internal]
    ///
    /// Records the response whose body failed to be received.
    pub(crate) fn with_output(mut self, output: RequestOutput) -> Self {
        self.output = Some(output);
        self
    }

    /// [internal]
    ///
    /// Classifies the `error` of reading a blocking response body, which
    /// holds the [`reqwest::Error`], if any.
    pub(super) fn from_io(error: std::io::Error) -> Self {
        match error.downcast::<reqwest::Error>() {
            Ok(error) => error.into(),
            Err(error) => Self {
                kind: TransportErrorKind::Other,
                source: Some(Box::new(error)),
                output: None,
            },
        }
    }
}

impl From<reqwest::Error> for TransportError {
    /// Classifies `error`, dropping its URL as it may hold secrets.
    fn from(error: reqwest::Error) -> Self {
        let kind = if error.is_connect() {
            TransportErrorKind::Connect
        } else if error.is_timeout() {
            TransportErrorKind::Timeout
        } else {
            TransportErrorKind::Other
        };

        Self {
            kind,
            source: Some(Box::new(error.without_url())),
            output: None,
        }
    }
}

impl std::fmt::Display for TransportError {
    /// Shows the kind followed by the chain of the sources, as the
    /// messages of the [`reqwest`] errors alone are not specific.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let sources: String =
            std::iter::successors(std::error::Error::source(self), |&error| error.source())
                .map(|error| format!(": {error}"))
                .collect();
        write!(f, "{}{sources}", self.kind)
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn std::error::Error + 'static))
    }
}

impl std::fmt::Display for TransportErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransportErrorKind::Connect => write!(f, "failed to connect"),
            TransportErrorKind::Timeout => write!(f, "timed out"),
            TransportErrorKind::Other => write!(f, "failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::tests::*;
    use std::error::Error as _;
    use std::time::Duration;

    mod transport_error {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_create_error_without_source() {
                let error = TransportError::new(TransportErrorKind::Timeout);

                assert_eq!(error.kind(), TransportErrorKind::Timeout);
                assert!(error.source().is_none());
                assert!(error.output().is_none());
            }
        }

        mod kind {
            use super::*;

            #[test]
            fn kind_should_return_kind() {
                assert_eq!(
                    TransportError::new(TransportErrorKind::Connect).kind(),
                    TransportErrorKind::Connect
                );
            }
        }

        mod output {
            use super::*;

            #[test]
            fn output_should_return_response_recorded() {
                let output = output(200, b"");

                let error =
                    TransportError::new(TransportErrorKind::Other).with_output(output.clone());

                assert_eq!(error.output(), Some(&output));
            }
        }

        mod with_output {
            use super::*;

            #[test]
            fn with_output_should_keep_kind() {
                let error = TransportError::new(TransportErrorKind::Other)
                    .with_output(RequestOutput::default());

                assert_eq!(error.kind(), TransportErrorKind::Other);
            }
        }

        mod from_io {
            use super::*;

            #[test]
            fn from_io_should_classify_reqwest_error_it_holds() {
                let failure = reqwest::blocking::Client::new()
                    .get(closed_url())
                    .send()
                    .unwrap_err();

                let error = TransportError::from_io(std::io::Error::other(failure));

                assert_eq!(error.kind(), TransportErrorKind::Connect);
                assert!(error.source().unwrap().is::<reqwest::Error>());
            }

            #[test]
            fn from_io_other_error_should_keep_it_as_source() {
                let error = TransportError::from_io(std::io::Error::other("reset"));

                assert_eq!(error.kind(), TransportErrorKind::Other);
                assert_eq!(error.to_string(), "failed: reset");
            }
        }

        mod from {
            use super::*;

            #[test]
            fn from_should_classify_reqwest_errors_without_url() {
                // GIVEN a connect, a timeout and a body failure
                let client = reqwest::blocking::Client::new();
                let slow_server = serve(slow);
                let truncated_server = serve(truncated);
                let failures = [
                    (
                        client.get(closed_url()).send().unwrap_err(),
                        TransportErrorKind::Connect,
                    ),
                    (
                        client
                            .get(&slow_server.url)
                            .timeout(Duration::from_millis(50))
                            .send()
                            .unwrap_err(),
                        TransportErrorKind::Timeout,
                    ),
                    (
                        client
                            .get(&truncated_server.url)
                            .send()
                            .unwrap()
                            .bytes()
                            .unwrap_err(),
                        TransportErrorKind::Other,
                    ),
                ];

                for (failure, expected) in failures {
                    // WHEN converting them
                    let error = TransportError::from(failure);

                    // THEN they are classified and the URL is dropped
                    assert_eq!(error.kind(), expected);
                    assert!(!error.to_string().contains("127.0.0.1"), "{error}");
                }
            }
        }

        mod std_fmt_display {
            use super::*;

            #[test]
            fn fmt_should_show_kind_and_source_chain() {
                let failure = reqwest::blocking::Client::new()
                    .get(closed_url())
                    .send()
                    .unwrap_err();
                let chain: String =
                    std::iter::successors(failure.source(), |&error| error.source())
                        .map(|error| format!(": {error}"))
                        .collect();
                let error = TransportError::from(failure);

                assert_eq!(
                    error.to_string(),
                    format!("failed to connect: error sending request{chain}")
                );
                assert_eq!(
                    TransportError::new(TransportErrorKind::Timeout).to_string(),
                    "timed out"
                );
            }
        }

        mod std_error_error {
            use super::*;

            mod source {
                use super::*;

                #[test]
                fn source_should_return_reqwest_error() {
                    let failure = reqwest::blocking::Client::new()
                        .get(closed_url())
                        .send()
                        .unwrap_err();

                    let error = TransportError::from(failure);

                    assert!(error.source().unwrap().is::<reqwest::Error>());
                }
            }
        }
    }

    mod transport_error_kind {
        use super::*;

        mod std_fmt_display {
            use super::*;

            #[test]
            fn fmt_should_describe_kind() {
                for (kind, displayed) in [
                    (TransportErrorKind::Connect, "failed to connect"),
                    (TransportErrorKind::Timeout, "timed out"),
                    (TransportErrorKind::Other, "failed"),
                ] {
                    assert_eq!(kind.to_string(), displayed);
                }
            }
        }
    }
}
