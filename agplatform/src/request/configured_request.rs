use std::any::Any;

use crate::Result;
use crate::interceptor::Interceptors;
use crate::interceptor::Run;
use crate::interceptor::Sink;
use crate::utils::OwnInterceptors;

use super::HttpRequest;
use super::Request;
use super::RequestOptions;
use super::RequestOutput;

/// Enabled by the `request` feature flag.
///
/// A [`reqwest`] request builder `B` to be sent with [`RequestOptions`].
///
/// Usually obtained from [`RequestBuilderExt`](crate::RequestBuilderExt) or
/// [`BlockingRequestBuilderExt`](crate::BlockingRequestBuilderExt).
pub struct ConfiguredRequest<B> {
    builder: B,
    options: RequestOptions,
    /// The values for the interceptors, set on the run once it is built.
    values: Vec<SetValue>,
}

/// [internal]
///
/// Sets a value of [`ConfiguredRequest::with`] on the run.
type SetValue = Box<dyn FnOnce(&mut Run<HttpRequest>) + Send>;

impl<B> ConfiguredRequest<B> {
    /// Creates a request of `builder` with the default [`RequestOptions`].
    pub fn new(builder: B) -> Self {
        Self::with_options(builder, RequestOptions::default())
    }

    /// Creates a request of `builder` with `options`, e.g. read from a
    /// configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{ConfiguredRequest, RequestOptions};
    ///
    /// let options = RequestOptions { keep_partial_body: true, ..Default::default() };
    /// let builder = agplatform::reqwest::blocking::Client::new().get("https://example.com");
    /// let request = ConfiguredRequest::with_options(builder, options);
    ///
    /// assert!(request.options().keep_partial_body);
    /// ```
    pub fn with_options(builder: B, options: RequestOptions) -> Self {
        Self {
            builder,
            options,
            values: Vec::new(),
        }
    }

    /// Returns the options.
    pub fn options(&self) -> &RequestOptions {
        &self.options
    }

    /// Shows the last `lines` of the response body when displayed.
    ///
    /// See [`RequestOptions::display_lines`].
    pub fn display_lines(mut self, lines: usize) -> Self {
        self.options.display_lines = lines;
        self
    }

    /// Sends the request with its own `interceptors` instead of those of
    /// the request, see [`Request::interceptors`]. They are taken once as
    /// the run starts, so the interceptors cannot change them.
    pub fn interceptors(self, interceptors: Interceptors<HttpRequest>) -> Self {
        self.with(OwnInterceptors(interceptors))
    }

    /// Keeps the part of the response body received when it fails.
    ///
    /// See [`RequestOptions::keep_partial_body`].
    pub fn keep_partial_body(mut self) -> Self {
        self.options.keep_partial_body = true;
        self
    }

    /// Sets `value` of type `T` on the run for the interceptors, replacing
    /// the previous one, e.g. a
    /// [`DryRunOutput`](crate::interceptor::DryRunOutput) for
    /// [`DryRun`](crate::interceptor::DryRun). Read with [`Run::get`].
    pub fn with<T: Any + Send + Sync>(mut self, value: T) -> Self {
        self.values.push(Box::new(move |run| run.insert(value)));
        self
    }

    /// Returns the run of `subject` with the values of the request.
    fn run(values: Vec<SetValue>, subject: HttpRequest) -> Run<HttpRequest> {
        let mut run = Run::new(subject);
        values.into_iter().for_each(|set| set(&mut run));
        run
    }
}

impl ConfiguredRequest<reqwest::blocking::RequestBuilder> {
    /// Sends the request with the options.
    ///
    /// See [`Request::send_blocking`].
    pub fn send_blocking<R: Request + ?Sized>(self, request: &R) -> Result<RequestOutput> {
        request.send_blocking(self)
    }

    /// Sends the request with the options, passing the head and the body
    /// of a successful response to `sink`.
    ///
    /// See [`Request::send_blocking_with_sink`].
    pub fn send_blocking_with_sink<R: Request + ?Sized>(
        self,
        request: &R,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> Result<RequestOutput> {
        request.send_blocking_with_sink(self, sink)
    }

    /// [internal]
    ///
    /// Returns the client of the request and its run, failing when the
    /// request cannot be built or its body cannot be read.
    pub(crate) fn into_run(self) -> (reqwest::blocking::Client, Result<Run<HttpRequest>>) {
        let (client, request) = self.builder.build_split();
        let run = request
            .map_err(invalid_request)
            .and_then(|request| HttpRequest::from_blocking(request, self.options))
            .map(|subject| Self::run(self.values, subject));
        (client, run)
    }
}

impl From<reqwest::blocking::RequestBuilder>
    for ConfiguredRequest<reqwest::blocking::RequestBuilder>
{
    fn from(builder: reqwest::blocking::RequestBuilder) -> Self {
        Self::new(builder)
    }
}

#[cfg(feature = "tokio")]
impl From<reqwest::RequestBuilder> for ConfiguredRequest<reqwest::RequestBuilder> {
    fn from(builder: reqwest::RequestBuilder) -> Self {
        Self::new(builder)
    }
}

#[cfg(feature = "tokio")]
impl ConfiguredRequest<reqwest::RequestBuilder> {
    /// Enabled by the `tokio` feature flag.
    ///
    /// Sends the request asynchronously with the options.
    ///
    /// See [`Request::send_async`].
    pub fn send_async<R: Request + ?Sized>(
        self,
        request: &R,
    ) -> impl Future<Output = Result<RequestOutput>> + Send {
        request.send_async(self)
    }

    /// Enabled by the `tokio` feature flag.
    ///
    /// Sends the request asynchronously with the options, passing the head
    /// and the body of a successful response to `sink`.
    ///
    /// See [`Request::send_async_with_sink`].
    pub fn send_async_with_sink<R: Request + ?Sized>(
        self,
        request: &R,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send {
        request.send_async_with_sink(self, sink)
    }

    /// [internal]
    ///
    /// Returns the client of the request and its run like
    /// [`ConfiguredRequest::into_run`] of a blocking one.
    pub(crate) async fn into_run(self) -> (reqwest::Client, Result<Run<HttpRequest>>) {
        let (client, request) = self.builder.build_split();
        let run = match request.map_err(invalid_request) {
            Ok(request) => HttpRequest::from_async(request, self.options)
                .await
                .map(|subject| Self::run(self.values, subject)),
            Err(error) => Err(error),
        };
        (client, run)
    }
}

/// Returns the error of a request failing to be built.
fn invalid_request(error: reqwest::Error) -> crate::Error {
    let error = error.without_url();
    request_error!("invalid request: {error}").with_source(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::BlockingRequestBuilderExt;
    #[cfg(feature = "tokio")]
    use crate::request::RequestBuilderExt;
    use crate::request::tests::*;
    use reqwest::Method;
    use reqwest::Version;
    use std::time::Duration;

    mod configured_request {
        use super::*;

        mod display_lines {
            use super::*;

            #[test]
            fn display_lines_should_set_lines() {
                let request = ConfiguredRequest::new(
                    reqwest::blocking::Client::new().get("http://localhost"),
                )
                .display_lines(4);

                assert_eq!(request.options().display_lines, 4);
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_have_default_options() {
                let blocking = ConfiguredRequest::new(
                    reqwest::blocking::Client::new().get("http://localhost"),
                );

                assert_eq!(blocking.options(), &RequestOptions::default());
            }
        }

        mod with_options {
            use super::*;

            #[test]
            fn with_options_should_keep_options() {
                let options = RequestOptions {
                    keep_partial_body: true,
                    ..Default::default()
                };

                let request = ConfiguredRequest::with_options(
                    reqwest::blocking::Client::new().get("http://localhost"),
                    options.clone(),
                );

                assert_eq!(request.options(), &options);
            }
        }

        mod with {
            use super::*;

            #[test]
            fn with_should_replace_value_of_same_type() {
                let request = ConfiguredRequest::new(
                    reqwest::blocking::Client::new().get("http://localhost"),
                )
                .with(1_u32)
                .with("other")
                .with(2_u32);

                let run = request.into_run().1.unwrap();

                assert_eq!(run.get::<u32>(), Some(&2));
                assert_eq!(run.get::<&str>(), Some(&"other"));
            }
        }

        mod from {
            use super::*;

            #[allow(clippy::module_inception)]
            mod from {
                use super::*;

                #[test]
                fn from_builder_should_have_default_options() {
                    let blocking = ConfiguredRequest::from(
                        reqwest::blocking::Client::new().get("http://localhost"),
                    );

                    assert_eq!(blocking.options(), &RequestOptions::default());
                }

                #[cfg(feature = "tokio")]
                #[test]
                fn from_async_builder_should_have_default_options() {
                    let request =
                        ConfiguredRequest::from(reqwest::Client::new().get("http://localhost"));

                    assert_eq!(request.options(), &RequestOptions::default());
                }
            }
        }

        mod send_blocking {
            use super::*;

            #[test]
            fn send_blocking_should_send_request_with_options() {
                let server = serve(ok);
                let (request, recorder) = observed();

                blocking_get(&request, &server.url)
                    .keep_partial_body()
                    .send_blocking(&request)
                    .unwrap();

                assert!(recorder.events().contains(&"body body".to_string()));
            }
        }

        mod send_blocking_with_sink {
            use super::*;

            #[test]
            fn send_blocking_with_sink_should_send_request_with_options_to_sink() {
                let server = serve(ok);
                let (request, recorder) = observed();
                let mut received = Vec::new();

                blocking_get(&request, &server.url)
                    .keep_partial_body()
                    .send_blocking_with_sink(&request, &mut collect_into(&mut received))
                    .unwrap();

                assert!(recorder.events().contains(&"body body".to_string()));
                assert_eq!(received, b"body");
            }
        }

        #[cfg(feature = "tokio")]
        mod send_async {
            use super::*;

            #[test]
            fn send_async_should_send_request_with_options() {
                let server = serve(ok);
                let (request, recorder) = observed();

                block_on(
                    get(&request, &server.url)
                        .keep_partial_body()
                        .send_async(&request),
                )
                .unwrap();

                assert!(recorder.events().contains(&"body body".to_string()));
            }
        }

        #[cfg(feature = "tokio")]
        mod send_async_with_sink {
            use super::*;

            #[test]
            fn send_async_with_sink_should_send_request_with_options_to_sink() {
                let server = serve(ok);
                let (request, recorder) = observed();
                let mut received = Vec::new();

                block_on(
                    get(&request, &server.url)
                        .keep_partial_body()
                        .send_async_with_sink(&request, &mut collect_into(&mut received)),
                )
                .unwrap();

                assert!(recorder.events().contains(&"body body".to_string()));
                assert_eq!(received, b"body");
            }
        }

        mod into_run {
            use super::*;

            #[test]
            fn into_run_should_build_request_with_options() {
                let client = reqwest::blocking::Client::new();
                let request = client
                    .put("http://h/path")
                    .header("x-key", "value")
                    .timeout(Duration::from_secs(5))
                    .version(Version::HTTP_2)
                    .body("data")
                    .display_lines(3);

                let run = request.into_run().1.unwrap();

                let subject = run.subject();
                assert_eq!(subject.method(), Method::PUT);
                assert_eq!(subject.url().as_str(), "http://h/path");
                assert_eq!(subject.headers()["x-key"], "value");
                assert_eq!(subject.timeout(), Some(Duration::from_secs(5)));
                assert_eq!(subject.version, Version::HTTP_2);
                assert_eq!(subject.body(), Some(b"data".as_slice()));
                assert_eq!(subject.options().display_lines, 3);
            }

            #[test]
            fn into_run_invalid_request_should_fail() {
                let request = ConfiguredRequest::new(reqwest::blocking::Client::new().get("::"));

                let error = request.into_run().1.err().unwrap();

                assert_eq!(error.description(), "invalid request: builder error");
                assert!(error.source_as::<reqwest::Error>().unwrap().is_builder());
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn into_run_async_should_build_request_with_options() {
                let request = reqwest::Client::new()
                    .post("http://h/path")
                    .body(streamed(b"data"))
                    .keep_partial_body();

                let run = block_on(request.into_run()).1.unwrap();

                assert_eq!(run.subject().method(), Method::POST);
                assert_eq!(run.subject().body(), Some(b"data".as_slice()));
                assert!(run.subject().options().keep_partial_body);
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn into_run_async_invalid_request_should_fail() {
                let request = ConfiguredRequest::new(reqwest::Client::new().get("::"));

                let error = block_on(request.into_run()).1.err().unwrap();

                assert_eq!(error.description(), "invalid request: builder error");
            }
        }
    }
}
