use std::time::Duration;

use bytes::Bytes;
use reqwest::Method;
use reqwest::Url;
use reqwest::Version;
use reqwest::header::HeaderMap;

use crate::Result;
use crate::interceptor::BoxFuture;
use crate::interceptor::Captured;
use crate::interceptor::Emit;
use crate::interceptor::Retry;
use crate::interceptor::Run;
use crate::interceptor::Subject;

use super::DISPLAY_LINES;
use super::RequestOutput;
use super::TransportError;
use super::TransportErrorKind;
use super::is_success;

/// Enabled by the `request` feature flag.
///
/// A request sent by a [`Request`](crate::Request), the [`Subject`] of its runs
/// through the interceptors, see [`Run::subject`]. Its body is read into
/// memory.
///
/// The [`Interceptor`](crate::interceptor::Interceptor)s of a request get its
/// [`Run`] and may change it, e.g. add a header, before passing it on, or retry
/// it, e.g. with [`Self::retry`], or replace its response, e.g. with
/// [`DryRun`](crate::interceptor::DryRun) or
/// [`RequestCache`](crate::RequestCache). The chunks of the response body pass
/// through [`Interceptor::chunk`](crate::interceptor::Interceptor::chunk) as
/// they arrive, as received rather than split into lines, before they are
/// passed to the sink of
/// [`Request::send_blocking_with_sink`](crate::Request::send_blocking_with_sink) when the
/// response is a success.
///
/// The output of a run is a [success](RequestOutput::success) when its
/// status is (2xx), which an interceptor may change, e.g. to accept the
/// status `404` of a lookup of something that may not exist. Inside the
/// chain an output that is not a success is returned as `Ok`, the methods
/// of [`Request`](crate::Request) return it as an error.
///
/// Each attempt passes the run through the rest of the chain again with the
/// changes made to it before, so an interceptor should replace what it
/// sets, e.g. [`HeaderMap::insert`] a header rather than
/// [`append`](HeaderMap::append) it. The headers added are seen by all
/// interceptors, mark the values of secrets
/// [sensitive](reqwest::header::HeaderValue::set_sensitive) so that their
/// [`Debug`] hides them.
///
/// # Examples
///
/// ```rust
/// use agplatform::interceptor::{BoxFuture, Interceptor, Next};
/// use agplatform::{BlockingRequestBuilderExt, HttpRequest, Request, RequestOutput, Result};
/// use std::sync::{Arc, Mutex};
///
/// #[derive(Clone, Default)]
/// struct Failures(Arc<Mutex<Vec<String>>>);
///
/// impl Interceptor<HttpRequest> for Failures {
///     fn handle<'a>(&'a self, mut next: Next<'a, HttpRequest>) -> BoxFuture<'a, Result<RequestOutput>> {
///         Box::pin(async move {
///             let result = next.attempt().await;
///
///             if let Ok(output) = &result
///                 && !output.success
///             {
///                 let target = next.run().subject().target();
///                 self.0.lock().unwrap().push(format!("{target} {}", output.status));
///             }
///
///             result
///         })
///     }
/// }
///
/// let failures = Failures::default();
/// let request = agplatform::TestRequest::new();
/// request.add_interceptor(failures.clone());
/// request.set_response("", RequestOutput { status: 404, ..Default::default() });
///
/// let client = request.blocking_client().unwrap();
/// let _ = client.get("https://example.com/missing?token=secret").send_blocking(&request);
///
/// assert_eq!(*failures.0.lock().unwrap(), ["GET https://example.com/missing 404"]);
/// ```
pub struct HttpRequest {
    method: Method,
    url: Url,
    headers: HeaderMap,
    pub(super) version: Version,
    timeout: Option<Duration>,
    body: Option<Bytes>,
    options: RequestOptions,
}

/// Enabled by the `request` feature flag.
///
/// Options for [`Request::send_blocking`](crate::Request::send_blocking).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestOptions {
    /// The number of the last lines of the response body shown when
    /// displaying the [`RequestOutput`], e.g. as the source of an error, at
    /// most as many KiB of them. 10 by default.
    pub display_lines: usize,

    /// Keeps the part of the response body received, as the interceptors passed
    /// it on, in the [`TransportError::output`] of a response that fails to be
    /// received, e.g. to resume a download. Its body is not kept when it goes
    /// to a sink, see
    /// [`Request::send_blocking_with_sink`](crate::Request::send_blocking_with_sink).
    pub keep_partial_body: bool,
}

impl HttpRequest {
    /// Returns the request body, `None` when it has none.
    pub fn body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }

    /// Returns the key of the request for
    /// [`RequestCache`](crate::RequestCache): the method, the whole URL and the
    /// body escaped, but not the headers, as they may hold secrets that change.
    /// `None` for the methods other than `GET` and `HEAD`, which are not
    /// cached.
    pub fn cache_key(&self) -> Option<String> {
        matches!(self.method, Method::GET | Method::HEAD).then(|| {
            let key = format!("{} {}", self.method, self.url);

            match &self.body {
                Some(body) if !body.is_empty() => format!("{key}\n{}", body.escape_ascii()),
                _ => key,
            }
        })
    }

    /// Returns the request headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns the request headers to be changed, e.g. by an interceptor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::{BoxFuture, Interceptor, Next};
    /// use agplatform::reqwest::header::{AUTHORIZATION, HeaderValue};
    /// use agplatform::{HttpRequest, RequestOutput, Result};
    ///
    /// struct Token(String);
    ///
    /// impl Interceptor<HttpRequest> for Token {
    ///     fn handle<'a>(&'a self, mut next: Next<'a, HttpRequest>) -> BoxFuture<'a, Result<RequestOutput>> {
    ///         let request = next.run_mut().subject_mut();
    ///
    ///         if request.url().host_str() == Some("api.example.com")
    ///             && let Ok(mut value) = HeaderValue::from_str(&format!("Bearer {}", self.0))
    ///         {
    ///             value.set_sensitive(true);
    ///             request.headers_mut().insert(AUTHORIZATION, value);
    ///         }
    ///
    ///         next.forward()
    ///     }
    /// }
    /// ```
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Returns the method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns the options.
    pub fn options(&self) -> &RequestOptions {
        &self.options
    }

    /// Returns the options to be changed, e.g. by an interceptor.
    pub fn options_mut(&mut self) -> &mut RequestOptions {
        &mut self.options
    }

    /// Returns the [`Retry`] of the failed attempts of the runs [safe to
    /// retry](Self::safe_to_retry), up to `retries` times after `delay`,
    /// waiting at least as long as a failed response asks with its
    /// `Retry-After` header (as seconds or an HTTP date, e.g. of a 429 or
    /// 503), see [`RequestOutput::retry_after`]. Its
    /// [`when`](Retry::when) decides instead of the safety, e.g. to retry
    /// also another status. Each attempt is limited by the timeouts of the
    /// request and its client on its own.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, HttpRequest, Request, RequestOutput, TestRequest};
    /// use std::time::Duration;
    ///
    /// let request = TestRequest::new();
    /// request.add_interceptor(
    ///     HttpRequest::retry(3, Duration::from_millis(100))
    ///         .backoff(2)
    ///         .when(|run, result| {
    ///             run.subject().safe_to_retry(result) || result.as_ref().is_ok_and(|output| output.status == 412)
    ///         }),
    /// );
    /// request.set_responses("", [RequestOutput { status: 412, ..Default::default() }, RequestOutput { status: 200, ..Default::default() }]);
    ///
    /// let client = request.blocking_client().unwrap();
    /// client.put("https://example.com/item").body("data").send_blocking(&request).unwrap();
    ///
    /// assert_eq!(request.calls().len(), 2);
    /// ```
    pub fn retry(retries: u32, delay: Duration) -> Retry<HttpRequest> {
        Retry::<Self>::new(retries, delay)
            .when(|run, result| run.subject().safe_to_retry(result))
            .retry_after(RequestOutput::retry_after)
    }

    /// Returns `true` if the failed `result` of an attempt of the request
    /// may pass when retried without the risk of processing the request
    /// twice: of an idempotent method (`GET`, `HEAD`, `PUT`, `DELETE`,
    /// `OPTIONS` and `TRACE`, see RFC 9110) a [`TransportError`] or the
    /// status `408` (Request Timeout), `429` (Too Many Requests) or `5xx`,
    /// and of the others (e.g. `POST` and `PATCH`) only a
    /// [`TransportErrorKind::Connect`] error or the status `408`, `429` or
    /// `503` (Service Unavailable). Used by [`Self::retry`].
    pub fn safe_to_retry(&self, result: &Result<RequestOutput>) -> bool {
        let idempotent = self.method.is_idempotent();

        match result {
            Ok(output) if output.success => false,
            Ok(output) if idempotent => matches!(output.status, 408 | 429 | 500..=599),
            Ok(output) => matches!(output.status, 408 | 429 | 503),
            Err(error) => error
                .source_as::<TransportError>()
                .is_some_and(|error| idempotent || error.kind() == TransportErrorKind::Connect),
        }
    }

    /// Sets the request body, `None` for none, e.g. in an interceptor.
    pub fn set_body(&mut self, body: Option<Vec<u8>>) {
        self.body = body.map(Bytes::from);
    }

    /// Sets the method, e.g. in an interceptor.
    pub fn set_method(&mut self, method: Method) {
        self.method = method;
    }

    /// Sets the timeout of the request (not of its client), `None` for
    /// none, e.g. in an interceptor.
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    /// Returns the method and the URL without its credentials, query and
    /// fragment, e.g. `GET https://example.com/path`, as named by the
    /// errors.
    pub fn target(&self) -> String {
        let mut url = self.url.clone();
        url.set_query(None);
        url.set_fragment(None);
        let _ = url.set_username("");
        let _ = url.set_password(None);
        format!("{} {url}", self.method)
    }

    /// Returns the timeout of the request (not of its client).
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Returns the URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the URL to be changed, e.g. by an interceptor.
    pub fn url_mut(&mut self) -> &mut Url {
        &mut self.url
    }

    /// [internal]
    ///
    /// Returns the blocking request sent by an attempt.
    pub(crate) fn blocking_request(&self) -> reqwest::blocking::Request {
        let mut request = reqwest::blocking::Request::new(self.method.clone(), self.url.clone());
        *request.headers_mut() = self.headers.clone();
        *request.version_mut() = self.version;
        *request.timeout_mut() = self.timeout;
        *request.body_mut() = self.body.clone().map(Into::into);
        request
    }

    /// [internal]
    ///
    /// Returns the async request sent by an attempt.
    #[cfg(feature = "tokio")]
    pub(crate) fn async_request(&self) -> reqwest::Request {
        let mut request = reqwest::Request::new(self.method.clone(), self.url.clone());
        *request.headers_mut() = self.headers.clone();
        *request.version_mut() = self.version;
        *request.timeout_mut() = self.timeout;
        *request.body_mut() = self.body.clone().map(Into::into);
        request
    }

    /// [internal]
    ///
    /// Returns the request of `request` with `options`, its body read into
    /// memory.
    pub(crate) fn from_blocking(
        mut request: reqwest::blocking::Request,
        options: RequestOptions,
    ) -> Result<Self> {
        let body = match request.body_mut().take() {
            Some(mut body) => {
                body.buffer().map_err(unreadable_body)?;
                // WHY: a blocking body gives its bytes only by reference,
                // copying them would cost as much as sending them.
                Some(Bytes::from_owner(BufferedBody(body)))
            }
            None => None,
        };

        Ok(Self {
            method: request.method().clone(),
            url: request.url().clone(),
            headers: std::mem::take(request.headers_mut()),
            version: request.version(),
            timeout: request.timeout().copied(),
            body,
            options,
        })
    }

    /// [internal]
    ///
    /// Returns the request of `request` with `options`, its body read into
    /// memory.
    #[cfg(feature = "tokio")]
    pub(crate) async fn from_async(
        mut request: reqwest::Request,
        options: RequestOptions,
    ) -> Result<Self> {
        let body = match request.body_mut().take() {
            Some(body) => Some(
                http_body_util::BodyExt::collect(body)
                    .await
                    .map_err(unreadable_body)?
                    .to_bytes(),
            ),
            None => None,
        };

        Ok(Self {
            method: request.method().clone(),
            url: request.url().clone(),
            headers: std::mem::take(request.headers_mut()),
            version: request.version(),
            timeout: request.timeout().copied(),
            body,
            options,
        })
    }
}

impl Subject for HttpRequest {
    type Output = RequestOutput;
    type Stream = ();

    fn validate(_run: &Run<Self>) -> Result {
        Ok(())
    }

    /// Passes the head and the body to the sink only when the status is a
    /// success, like a response.
    fn replay<'a>(
        run: &'a Run<Self>,
        output: &'a RequestOutput,
        emit: &'a mut Emit<'_, Self>,
    ) -> BoxFuture<'a, Result> {
        Box::pin(async move {
            if !is_success(output.status) {
                emit.skip_sink();
            } else if run.has_sink() {
                emit.head(&head(output)).await?;
            }

            if output.body.is_empty() {
                Ok(())
            } else {
                emit.emit((), &output.body).await
            }
        })
    }

    /// Sets the success from the status, the body captured and the lines
    /// displayed of the options.
    fn finish(
        run: &Run<Self>,
        output: RequestOutput,
        mut captured: Captured<Self>,
    ) -> RequestOutput {
        RequestOutput {
            success: is_success(output.status),
            body: captured.take(()),
            display_lines: run.subject().options.display_lines,
            ..output
        }
    }
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            display_lines: DISPLAY_LINES,
            keep_partial_body: false,
        }
    }
}

/// [internal]
///
/// A blocking body read into memory, see [`reqwest::blocking::Body::buffer`].
struct BufferedBody(reqwest::blocking::Body);

impl AsRef<[u8]> for BufferedBody {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes().unwrap_or_default()
    }
}

/// Returns the head of `output`, without its body.
fn head(output: &RequestOutput) -> RequestOutput {
    RequestOutput {
        success: output.success,
        status: output.status,
        headers: output.headers.clone(),
        body: Vec::new(),
        duration: output.duration,
        display_lines: output.display_lines,
    }
}

/// Returns the error of a request body failing to be read.
fn unreadable_body(error: reqwest::Error) -> crate::Error {
    let error = error.without_url();
    request_error!("request body could not be read: {error}").with_source(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestRequest;
    use crate::interceptor::DryRun;
    use crate::interceptor::DryRunOutput;
    use crate::interceptor::Interceptor;
    use crate::interceptor::Interceptors;
    use crate::interceptor::Next;
    use crate::request::BlockingRequestBuilderExt;
    use crate::request::Request;
    use crate::request::RequestImpl;
    use crate::request::tests::*;
    use std::sync::Arc;

    /// Changes the request before passing it on.
    struct Change(fn(&mut HttpRequest));

    impl Interceptor<HttpRequest> for Change {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, HttpRequest>,
        ) -> BoxFuture<'a, Result<RequestOutput>> {
            (self.0)(next.run_mut().subject_mut());
            next.forward()
        }
    }

    mod http_request {
        use super::*;

        mod body {
            use super::*;

            #[test]
            fn body_should_return_body_or_none() {
                let mut request = subject(Method::POST);
                assert_eq!(request.body(), None);

                request.set_body(Some(b"data".to_vec()));

                assert_eq!(request.body(), Some(b"data".as_slice()));
            }
        }

        mod cache_key {
            use super::*;

            #[test]
            fn cache_key_should_name_method_url_and_body_of_get_and_head() {
                let mut get = subject(Method::GET);
                assert_eq!(get.cache_key().as_deref(), Some("GET http://h/"));
                get.set_body(Some(b"a\nb\xff".to_vec()));
                assert_eq!(
                    get.cache_key().as_deref(),
                    Some("GET http://h/\na\\nb\\xff")
                );

                assert_eq!(
                    subject(Method::HEAD).cache_key().as_deref(),
                    Some("HEAD http://h/")
                );
                for method in [Method::POST, Method::PUT, Method::DELETE] {
                    assert_eq!(subject(method).cache_key(), None);
                }
            }

            #[test]
            fn cache_key_empty_body_should_equal_no_body() {
                let mut empty = subject(Method::GET);
                empty.set_body(Some(Vec::new()));

                assert_eq!(empty.cache_key(), subject(Method::GET).cache_key());
            }
        }

        mod headers {
            use super::*;

            #[test]
            fn headers_should_return_headers() {
                assert!(subject(Method::GET).headers().is_empty());
            }
        }

        mod headers_mut {
            use super::*;

            #[test]
            fn headers_mut_should_change_headers_sent() {
                let server = serve(ok);
                let request = intercepted(Change(|request| {
                    request
                        .headers_mut()
                        .insert("x-key", "value".parse().unwrap());
                }));

                blocking_get(&request, &server.url)
                    .send_blocking(&request)
                    .unwrap();

                assert!(server.received()[0].contains("x-key: value\r\n"));
            }
        }

        mod method {
            use super::*;

            #[test]
            fn method_should_return_method() {
                assert_eq!(subject(Method::PATCH).method(), Method::PATCH);
            }
        }

        mod options {
            use super::*;

            #[test]
            fn options_should_return_options() {
                assert_eq!(subject(Method::GET).options(), &RequestOptions::default());
            }
        }

        mod options_mut {
            use super::*;

            #[test]
            fn options_mut_should_change_options_of_run() {
                let server = serve(not_found);
                let request = intercepted(Change(|request| {
                    request.options_mut().display_lines = 1;
                }));

                let error = blocking_get(&request, &server.url)
                    .send_blocking(&request)
                    .unwrap_err();

                assert_eq!(error.request_output().unwrap().display_lines, 1);
            }
        }

        mod retry {
            use super::*;

            /// Returns a failed response asking to wait `retry_after`.
            fn asking(retry_after: &str) -> RequestOutput {
                let mut output = output(503, b"");
                output
                    .headers
                    .insert(reqwest::header::RETRY_AFTER, retry_after.parse().unwrap());
                output
            }

            fn test_request() -> TestRequest {
                let request = TestRequest::new();
                request.add_interceptor(HttpRequest::retry(2, Duration::from_secs(1)));
                request
            }

            #[test]
            fn retry_should_retry_failures_safe_to_retry_until_success() {
                let request = test_request();
                request.set_responses("", [output(503, b""), output(200, b"ok")]);

                let output = request
                    .blocking_client()
                    .unwrap()
                    .get("http://h")
                    .send_blocking(&request)
                    .unwrap();

                assert_eq!(output.body, b"ok");
                assert_eq!(request.sleeps(), [Duration::from_secs(1)]);
            }

            #[test]
            fn retry_should_not_retry_failures_unsafe_to_retry() {
                let request = test_request();
                request.set_response("", output(500, b""));

                let result = request
                    .blocking_client()
                    .unwrap()
                    .post("http://h")
                    .send_blocking(&request);

                assert!(result.is_err());
                assert_eq!(request.calls().len(), 1);
            }

            #[test]
            fn retry_should_wait_at_least_retry_after() {
                let request = test_request();
                request.set_responses("", [asking("5"), asking(" 30 "), output(200, b"")]);

                request
                    .blocking_client()
                    .unwrap()
                    .get("http://h")
                    .send_blocking(&request)
                    .unwrap();

                assert_eq!(
                    request.sleeps(),
                    [Duration::from_secs(5), Duration::from_secs(30)]
                );
            }

            #[test]
            fn retry_when_should_decide_instead_of_safety() {
                let request = TestRequest::new();
                request.add_interceptor(
                    HttpRequest::retry(1, Duration::ZERO).when(|_, result| result.is_ok()),
                );
                request.set_response("", output(404, b""));

                let _ = request
                    .blocking_client()
                    .unwrap()
                    .post("http://h")
                    .send_blocking(&request);

                assert_eq!(request.calls().len(), 2);
            }
        }

        mod safe_to_retry {
            use super::*;

            fn transport(kind: TransportErrorKind) -> Result<RequestOutput> {
                Err(crate::Error::request("transport").with_source(TransportError::new(kind)))
            }

            /// Returns whether each failure of `method` is safe to retry.
            fn safe(method: Method) -> Vec<bool> {
                let request = subject(method);

                [
                    Ok(RequestOutput {
                        success: true,
                        ..output(500, b"")
                    }),
                    Err(crate::Error::request("other")),
                    transport(TransportErrorKind::Connect),
                    transport(TransportErrorKind::Timeout),
                    transport(TransportErrorKind::Other),
                    Ok(output(404, b"")),
                    Ok(output(408, b"")),
                    Ok(output(429, b"")),
                    Ok(output(500, b"")),
                    Ok(output(502, b"")),
                    Ok(output(503, b"")),
                ]
                .iter()
                .map(|result| request.safe_to_retry(result))
                .collect()
            }

            #[test]
            fn safe_to_retry_idempotent_method_should_accept_transient_failures() {
                for method in [
                    Method::GET,
                    Method::HEAD,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::TRACE,
                ] {
                    assert_eq!(
                        safe(method.clone()),
                        [
                            false, false, true, true, true, false, true, true, true, true, true
                        ],
                        "method: {method}"
                    );
                }
            }

            #[test]
            fn safe_to_retry_non_idempotent_method_should_accept_failures_before_processing_only() {
                for method in [Method::POST, Method::PATCH, Method::CONNECT] {
                    assert_eq!(
                        safe(method.clone()),
                        [
                            false, false, true, false, false, false, true, true, false, false, true
                        ],
                        "method: {method}"
                    );
                }
            }
        }

        mod set_body {
            use super::*;

            #[test]
            fn set_body_should_replace_or_remove_body_sent() {
                for (body, expected) in [(Some(b"new".to_vec()), "\r\n\r\nnew"), (None, "\r\n\r\n")]
                {
                    let server = serve(ok);
                    let request = RequestImpl::new();
                    let mut interceptors = Interceptors::new();
                    let body = Arc::new(body);
                    interceptors.push(SetBody(body.clone()));

                    request
                        .blocking_client()
                        .unwrap()
                        .post(&server.url)
                        .body("old")
                        .interceptors(interceptors)
                        .send_blocking(&request)
                        .unwrap();

                    assert!(server.received()[0].ends_with(expected), "{body:?}");
                }
            }

            /// Sets the body before passing the run on.
            struct SetBody(Arc<Option<Vec<u8>>>);

            impl Interceptor<HttpRequest> for SetBody {
                fn handle<'a>(
                    &'a self,
                    mut next: Next<'a, HttpRequest>,
                ) -> BoxFuture<'a, Result<RequestOutput>> {
                    next.run_mut().subject_mut().set_body((*self.0).clone());
                    next.forward()
                }
            }
        }

        mod set_method {
            use super::*;

            #[test]
            fn set_method_should_change_method_sent() {
                let server = serve(ok);
                let request = intercepted(Change(|request| request.set_method(Method::PUT)));

                blocking_get(&request, &server.url)
                    .send_blocking(&request)
                    .unwrap();

                assert!(server.received()[0].starts_with("PUT / "));
            }
        }

        mod set_timeout {
            use super::*;

            #[test]
            fn set_timeout_should_change_timeout_of_request() {
                let server = serve(slow);
                let request = intercepted(Change(|request| {
                    request.set_timeout(Some(Duration::from_millis(50)));
                }));

                let result = blocking_get(&request, &server.url).send_blocking(&request);

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Timeout));
            }
        }

        mod target {
            use super::*;

            #[test]
            fn target_should_strip_credentials_query_and_fragment() {
                let url = Url::parse("https://user:secret@example.com:8080/path?token=secret#part")
                    .unwrap();
                let request = reqwest::blocking::Request::new(Method::GET, url);

                let target = HttpRequest::from_blocking(request, RequestOptions::default())
                    .unwrap()
                    .target();

                assert_eq!(target, "GET https://example.com:8080/path");
            }
        }

        mod timeout {
            use super::*;

            #[test]
            fn timeout_should_return_timeout_of_request() {
                let mut request = subject(Method::GET);
                assert_eq!(request.timeout(), None);

                request.set_timeout(Some(Duration::from_secs(1)));

                assert_eq!(request.timeout(), Some(Duration::from_secs(1)));
            }
        }

        mod url {
            use super::*;

            #[test]
            fn url_should_return_url() {
                assert_eq!(subject(Method::GET).url().as_str(), "http://h/");
            }
        }

        mod url_mut {
            use super::*;

            #[test]
            fn url_mut_should_change_url_sent() {
                let server = serve(ok);
                let request = intercepted(Change(|request| request.url_mut().set_path("/other")));

                blocking_get(&request, &server.url)
                    .send_blocking(&request)
                    .unwrap();

                assert!(server.received()[0].starts_with("GET /other "));
            }
        }

        mod blocking_request {
            use super::*;

            #[test]
            fn blocking_request_should_copy_request() {
                let mut request = subject(Method::POST);
                request.headers_mut().insert("x-key", "v".parse().unwrap());
                request.set_timeout(Some(Duration::from_secs(1)));
                request.set_body(Some(b"data".to_vec()));
                request.version = Version::HTTP_2;

                let sent = request.blocking_request();

                assert_eq!(sent.method(), Method::POST);
                assert_eq!(sent.url().as_str(), "http://h/");
                assert_eq!(sent.headers()["x-key"], "v");
                assert_eq!(sent.timeout(), Some(&Duration::from_secs(1)));
                assert_eq!(sent.version(), Version::HTTP_2);
                assert_eq!(
                    sent.body().and_then(reqwest::blocking::Body::as_bytes),
                    Some(b"data".as_slice())
                );
            }
        }

        #[cfg(feature = "tokio")]
        mod async_request {
            use super::*;

            #[test]
            fn async_request_should_copy_request() {
                let mut request = subject(Method::POST);
                request.headers_mut().insert("x-key", "v".parse().unwrap());
                request.set_timeout(Some(Duration::from_secs(1)));
                request.set_body(Some(b"data".to_vec()));
                request.version = Version::HTTP_2;

                let sent = request.async_request();

                assert_eq!(sent.method(), Method::POST);
                assert_eq!(sent.url().as_str(), "http://h/");
                assert_eq!(sent.headers()["x-key"], "v");
                assert_eq!(sent.timeout(), Some(&Duration::from_secs(1)));
                assert_eq!(sent.version(), Version::HTTP_2);
                assert_eq!(
                    sent.body().and_then(reqwest::Body::as_bytes),
                    Some(b"data".as_slice())
                );
            }
        }

        mod from_blocking {
            use super::*;

            #[test]
            fn from_blocking_should_read_body_into_memory() {
                for body in [
                    None,
                    Some(reqwest::blocking::Body::from("data")),
                    Some(reqwest::blocking::Body::new(std::io::Cursor::new(b"data"))),
                ] {
                    let expected = body.as_ref().map(|_| b"data".as_slice());
                    let mut request = reqwest::blocking::Request::new(
                        Method::POST,
                        Url::parse("http://h").unwrap(),
                    );
                    *request.body_mut() = body;

                    let request =
                        HttpRequest::from_blocking(request, RequestOptions::default()).unwrap();

                    assert_eq!(request.body(), expected);
                }
            }

            #[test]
            fn from_blocking_unreadable_body_should_fail() {
                let mut request =
                    reqwest::blocking::Request::new(Method::POST, Url::parse("http://h").unwrap());
                *request.body_mut() = Some(reqwest::blocking::Body::new(FailingReader));

                let error = HttpRequest::from_blocking(request, RequestOptions::default())
                    .err()
                    .unwrap();

                assert!(
                    error
                        .description()
                        .starts_with("request body could not be read: "),
                    "{error}"
                );
            }
        }

        #[cfg(feature = "tokio")]
        mod from_async {
            use super::*;

            #[test]
            fn from_async_should_read_body_into_memory() {
                for body in [
                    None,
                    Some(reqwest::Body::from("data")),
                    Some(streamed(b"data")),
                ] {
                    let expected = body.as_ref().map(|_| b"data".as_slice());
                    let mut request =
                        reqwest::Request::new(Method::POST, Url::parse("http://h").unwrap());
                    *request.body_mut() = body;

                    let request =
                        block_on(HttpRequest::from_async(request, RequestOptions::default()))
                            .unwrap();

                    assert_eq!(request.body(), expected);
                }
            }

            #[test]
            fn from_async_unreadable_body_should_fail() {
                let mut request =
                    reqwest::Request::new(Method::POST, Url::parse("http://h").unwrap());
                *request.body_mut() = Some(reqwest::Body::wrap(FailingBody));

                let error = block_on(HttpRequest::from_async(request, RequestOptions::default()))
                    .err()
                    .unwrap();

                assert!(
                    error
                        .description()
                        .starts_with("request body could not be read: "),
                    "{error}"
                );
            }
        }

        mod subject {
            use super::*;

            mod validate {
                use super::*;

                #[test]
                fn validate_should_accept_every_request() {
                    assert!(HttpRequest::validate(&Run::new(subject(Method::GET))).is_ok());
                }
            }

            mod replay {
                use super::*;

                #[test]
                fn replay_finished_output_should_finish_to_same_output() {
                    for (display_lines, status) in [(10, 200), (10, 404), (2, 200)] {
                        let mut run = Run::new(subject(Method::GET));
                        run.subject_mut().options_mut().display_lines = display_lines;

                        crate::interceptor::tests::subject::replay::replay_finished_output_should_finish_to_same_output(
                            &run,
                            &output(status, b"body"),
                        );
                    }
                }

                #[test]
                fn replay_should_pass_body_of_success_only_to_sink() {
                    for (status, sunk, body) in
                        [(200, b"set".as_slice(), b"".as_slice()), (404, b"", b"set")]
                    {
                        let request = intercepted(DryRun);
                        let mut received = Vec::new();

                        let result = blocking_get(&request, "http://example.invalid/")
                            .with(DryRunOutput(output(status, b"set")))
                            .send_blocking_with_sink(&request, &mut collect_into(&mut received));

                        let output =
                            result.unwrap_or_else(|error| error.request_output().unwrap().clone());
                        assert_eq!(received, sunk, "status: {status}");
                        assert_eq!(output.body, body, "status: {status}");
                    }
                }

                /// Records the heads without their body and the chunks, refusing
                /// the heads when `refuse`.
                struct Heads {
                    events: Vec<String>,
                    refuse: bool,
                }

                impl crate::interceptor::Sink<HttpRequest> for Heads {
                    fn head<'a>(&'a mut self, head: &'a RequestOutput) -> BoxFuture<'a, Result> {
                        if self.refuse {
                            return Box::pin(std::future::ready(Err(crate::Error::fs(
                                "head refused",
                            ))));
                        }

                        self.events
                            .push(format!("head {} {:?}", head.status, head.body));
                        Box::pin(std::future::ready(Ok(())))
                    }

                    fn write<'a>(&'a mut self, (): (), data: &'a [u8]) -> BoxFuture<'a, Result> {
                        self.events.push(String::from_utf8_lossy(data).into_owned());
                        Box::pin(std::future::ready(Ok(())))
                    }
                }

                #[test]
                fn replay_should_pass_head_without_body_before_body_to_sink() {
                    let request = intercepted(DryRun);
                    let mut sink = Heads {
                        events: Vec::new(),
                        refuse: false,
                    };

                    blocking_get(&request, "http://example.invalid/")
                        .with(DryRunOutput(output(200, b"set")))
                        .send_blocking_with_sink(&request, &mut sink)
                        .unwrap();

                    assert_eq!(sink.events, ["head 200 []", "set"]);
                }

                #[test]
                fn replay_refused_head_should_fail() {
                    let request = intercepted(DryRun);
                    let mut sink = Heads {
                        events: Vec::new(),
                        refuse: true,
                    };

                    let error = blocking_get(&request, "http://example.invalid/")
                        .with(DryRunOutput(output(200, b"set")))
                        .send_blocking_with_sink(&request, &mut sink)
                        .unwrap_err();

                    assert_eq!(error.description(), "head refused");
                }

                #[test]
                fn replay_empty_body_should_pass_nothing() {
                    let (request, recorder) = observed();
                    request.add_interceptor(DryRun);

                    let output = blocking_get(&request, "http://example.invalid/")
                        .keep_partial_body()
                        .send_blocking(&request)
                        .unwrap();

                    assert_eq!(output, output_of(200));
                    assert_eq!(
                        recorder.events(),
                        [
                            "started GET http://example.invalid/".to_string(),
                            "finished 200".to_string()
                        ]
                    );
                }

                fn output_of(status: u16) -> RequestOutput {
                    output(status, b"")
                }
            }

            mod finish {
                use super::*;

                #[test]
                fn finish_should_set_success_from_status_and_display_lines() {
                    let request = intercepted(DryRun);

                    let ok = blocking_get(&request, "http://example.invalid/")
                        .display_lines(2)
                        .send_blocking(&request)
                        .unwrap();
                    let accepted = blocking_get(&request, "http://example.invalid/")
                        .with(DryRunOutput(RequestOutput {
                            success: true,
                            ..output(404, b"")
                        }))
                        .send_blocking(&request)
                        .unwrap_err();

                    assert!(ok.success);
                    assert_eq!(ok.display_lines, 2);
                    assert!(!accepted.request_output().unwrap().success);
                }
            }
        }
    }

    mod request_options {
        use super::*;

        mod default {
            use super::*;

            #[test]
            fn default_should_show_ten_lines_without_keeping_partial_body() {
                let options = RequestOptions::default();

                assert_eq!(options.display_lines, 10);
                assert!(!options.keep_partial_body);
            }
        }
    }
}
