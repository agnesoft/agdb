use std::any::Any;
use std::time::Duration;

use reqwest::header::HeaderMap;

use crate::Result;
#[cfg(feature = "tokio")]
use crate::interceptor::AsyncExecutor;
use crate::interceptor::BlockingExecutor;
use crate::interceptor::Emit;
use crate::interceptor::Interceptor;
use crate::interceptor::Interceptors;
use crate::interceptor::Run;
use crate::interceptor::Sink;
use crate::utils::Runs;
use crate::utils::conclude;

macro_rules! request_error {
    ($($arg:tt)*) => {
        $crate::Error::request(format!($($arg)*))
    };
}

mod cache;
mod clients;
mod configured_request;
mod http_request;
mod request_impl;
mod request_output;
mod transport_error;

pub use cache::RequestCache;
pub(crate) use clients::DefaultClients;
#[cfg(test)]
pub(crate) use clients::ReqwestEnv;
pub use configured_request::ConfiguredRequest;
pub use http_request::HttpRequest;
pub use http_request::RequestOptions;
pub use request_impl::RequestImpl;
pub use request_output::RequestOutput;
pub use transport_error::TransportError;
pub use transport_error::TransportErrorKind;

/// Enabled by the `request` feature flag.
///
/// Abstract network request interface.
///
/// Sends HTTP requests described by [`reqwest`] request builders and
/// receives their responses. Usually used through [`RequestBuilderExt`]
/// and [`BlockingRequestBuilderExt`] rather than directly.
///
/// Requests are sent by the [`reqwest`] client that built them: a default
/// one shared by all requests from [`client`](Self::client) (or
/// [`blocking_client`](Self::blocking_client)), configured with
/// [`set_client`](Self::set_client), or one with its own configuration
/// built from [`client_builder`](Self::client_builder), e.g. with a
/// proxy, a cookie store or extra root certificates, which reqwest only
/// supports per client. TLS needs the `tls` (rustls) or `native-tls` feature.
///
/// Proxies are configured from the platform environment by the rules of
/// hyper-util, which reqwest uses: `HTTP_PROXY`, `HTTPS_PROXY` and
/// `ALL_PROXY` (or lowercase) with the exceptions in `NO_PROXY`, none when
/// `REQUEST_METHOD` is set (CGI). Its changes, e.g. by
/// [`Env::set_var`](crate::Env::set_var) of the platform, apply to the
/// default clients and the builders from
/// [`client_builder`](Self::client_builder). A plain
/// [`reqwest::ClientBuilder`] reads the process environment instead, as do
/// the TLS backends (e.g. `SSL_CERT_FILE`) always.
///
/// Each request runs through the interceptors of the request, none by
/// default (see [`add_interceptor`](Self::add_interceptor)), or its own,
/// see [`ConfiguredRequest::interceptors`], as an [`HttpRequest`].
///
/// The two provided implementations are `RequestImpl` (delegates to
/// [`reqwest`]) and
/// [`TestRequest`](crate::test_platform::test_request::TestRequest)
/// (purely in-memory, behind the `testing` feature).
///
/// Requests are [`Sync`] so that the futures of
/// [`send_async`](Self::send_async) borrowing them are [`Send`].
pub trait Request: Send + Sync {
    /// Returns the default blocking client, built from the
    /// [`set_blocking_client`](Self::set_blocking_client) builder (the
    /// [`blocking_client_builder`](Self::blocking_client_builder) otherwise)
    /// on the first call, and again after the proxy variables change. The
    /// client is a cheap handle sharing the connection pool.
    ///
    /// The blocking client runs its own runtime thread, so it must not be
    /// built or used in an async context. Dropping it there is fine.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be built, every time.
    fn blocking_client(&self) -> Result<reqwest::blocking::Client>;

    /// Returns a blocking client builder with the proxies of the platform
    /// environment, see [`Request`]. Call
    /// [`no_proxy`](reqwest::blocking::ClientBuilder::no_proxy) on it before
    /// adding other proxies, as reqwest uses the first matching one.
    ///
    /// The default blocking client is built from it too. Connecting times
    /// out after 30 seconds, and the response after 5 minutes without data,
    /// instead of after 30 seconds in total like with a plain
    /// [`reqwest::blocking::ClientBuilder`]. The 5 minutes are its
    /// [`timeout`](reqwest::blocking::ClientBuilder::timeout), which limits
    /// sending the request until its response head arrives, and each read
    /// of the response body by [`send_blocking`](Self::send_blocking) or
    /// [`Read`](std::io::Read), but the whole body read by
    /// [`bytes`](reqwest::blocking::Response::bytes) or
    /// [`text`](reqwest::blocking::Response::text). The
    /// [`timeout`](reqwest::blocking::RequestBuilder::timeout) of a request
    /// limits all of it instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Platform, Request};
    ///
    /// let platform = agplatform::platform();
    /// let request = platform.request();
    /// let client = request.blocking_client_builder().user_agent("agent").build().unwrap();
    /// ```
    fn blocking_client_builder(&self) -> reqwest::blocking::ClientBuilder;

    /// Sets the builder of the default blocking client, replacing the
    /// previous client. The client is built by the next
    /// [`blocking_client`](Self::blocking_client) call.
    fn set_blocking_client(&self, builder: reqwest::blocking::ClientBuilder);

    /// Sends `request`, a request builder with its [`RequestOptions`], with
    /// the client that built it and returns the response. A plain builder
    /// is sent with the default options, a [`ConfiguredRequest`] (e.g. from
    /// [`BlockingRequestBuilderExt`]) with its own.
    ///
    /// A streamed request body (e.g. a reader) is read into memory first
    /// so that it can be passed to the interceptors and retried. The whole
    /// response body is read too. Each run passes through the interceptors,
    /// see [`HttpRequest`], unless the request cannot be built.
    ///
    /// # Errors
    ///
    /// The errors name only the method and the URL without its
    /// credentials, query and fragment, as they may hold secrets. An
    /// [`Interceptor`] gets the whole request, e.g. to log it redacted.
    ///
    /// Returns an error if:
    /// - The request cannot be built, e.g. its URL is invalid, or its body
    ///   cannot be read. It does not pass through the interceptors.
    /// - It cannot be sent or its response received, e.g. the connection
    ///   fails or times out. Its [`TransportError`] is returned by
    ///   [`Error::source_as`](crate::Error::source_as).
    /// - The response is not a [success](RequestOutput::success), e.g. its
    ///   status is not 2xx. Its [`RequestOutput`] is returned by
    ///   [`Error::request_output`](crate::Error::request_output).
    /// - An interceptor returns an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, ConfiguredRequest, Platform, Request, RequestOptions, RequestOutput};
    ///
    /// let platform = agplatform::platform();
    /// let request = platform.request();
    /// # let request = agplatform::TestRequest::new();
    /// # request.set_response("", RequestOutput { status: 200, body: b"pong".to_vec(), ..Default::default() });
    ///
    /// let client = request.blocking_client().unwrap();
    /// let output = request.send_blocking(client.get("https://example.com/ping")).unwrap();
    /// assert_eq!(output.text().unwrap(), "pong");
    ///
    /// request.send_blocking(client.get("https://example.com/ping").display_lines(3)).unwrap();
    /// let options = RequestOptions { keep_partial_body: true, ..Default::default() };
    /// request
    ///     .send_blocking(ConfiguredRequest::with_options(client.get("https://example.com/ping"), options))
    ///     .unwrap();
    /// ```
    fn send_blocking(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::blocking::RequestBuilder>>,
    ) -> Result<RequestOutput>;

    /// Sends `request` like [`Self::send_blocking`], passing the response
    /// of a success (2xx) to `sink` as it arrives instead of returning its
    /// body in [`RequestOutput::body`], e.g. to write a large download to a
    /// file: its head (the status and the headers, with an empty body) to
    /// [`Sink::head`] first, then the body to [`Sink::write`]. A failed
    /// response is returned as usual. A closure `|(), chunk: &[u8]|` is a
    /// sink of the body.
    ///
    /// A sink awaiting, e.g. a full channel, holds the response back. Its
    /// futures are polled on the current thread, so they must not need a
    /// runtime, see [`BlockingExecutor`](crate::interceptor::BlockingExecutor).
    ///
    /// The body passes through the interceptors before, see
    /// [`Interceptor::chunk`]. An attempt failing after a part of the body was
    /// passed to `sink` is not retried by [`Retry`](crate::interceptor::Retry),
    /// as `sink` would get the body again from its start, see
    /// [`Run::streamed`]. A caller can resume it instead, e.g. with a `Range`
    /// header and the response head from [`TransportError::output`]. Each
    /// attempt passes its own head, so an attempt retried before its body
    /// reached `sink` passes a head again.
    ///
    /// # Errors
    ///
    /// As [`Self::send_blocking`], or the error returned by `sink`, which
    /// ends the run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::Sink;
    /// use agplatform::{BoxFuture, HttpRequest, Platform, Request, RequestOutput};
    ///
    /// struct File {
    ///     status: u16,
    ///     data: Vec<u8>,
    /// }
    ///
    /// impl Sink<HttpRequest> for File {
    ///     fn head<'a>(&'a mut self, head: &'a RequestOutput) -> BoxFuture<'a, agplatform::Result> {
    ///         self.status = head.status;
    ///         Box::pin(std::future::ready(Ok(())))
    ///     }
    ///
    ///     fn write<'a>(&'a mut self, (): (), data: &'a [u8]) -> BoxFuture<'a, agplatform::Result> {
    ///         self.data.extend_from_slice(data);
    ///         Box::pin(std::future::ready(Ok(())))
    ///     }
    /// }
    ///
    /// let platform = agplatform::platform();
    /// let request = platform.request();
    /// # let request = agplatform::TestRequest::new();
    /// # request.set_response("", RequestOutput::status(200).body("data"));
    /// let client = request.blocking_client().unwrap();
    ///
    /// let mut file = File { status: 0, data: Vec::new() };
    /// let output = request.send_blocking_with_sink(client.get("https://example.com/download"), &mut file).unwrap();
    /// assert!(output.body.is_empty());
    /// assert_eq!((file.status, file.data.as_slice()), (200, b"data".as_slice()));
    ///
    /// let mut body = Vec::new();
    /// let mut sink = |(), chunk: &[u8]| {
    ///     body.extend_from_slice(chunk);
    ///     Ok(())
    /// };
    /// request.send_blocking_with_sink(client.get("https://example.com/download"), &mut sink).unwrap();
    /// assert_eq!(body, b"data");
    /// ```
    fn send_blocking_with_sink(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::blocking::RequestBuilder>>,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> Result<RequestOutput>;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Returns the default client, built from the
    /// [`set_client`](Self::set_client) builder (the
    /// [`client_builder`](Self::client_builder) otherwise) on the first
    /// call, and again after the proxy variables change. The client is a
    /// cheap handle sharing the connection pool.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be built, every time.
    #[cfg(feature = "tokio")]
    fn client(&self) -> Result<reqwest::Client>;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Returns a client builder with the proxies of the platform
    /// environment, see [`Request`]. Call
    /// [`no_proxy`](reqwest::ClientBuilder::no_proxy) on it before adding
    /// other proxies, as reqwest uses the first matching one.
    ///
    /// The default client is built from it too. Connecting times out after
    /// 30 seconds and each read of the response after 5 minutes of
    /// silence, while a slow response is never cut in total. The
    /// [`timeout`](reqwest::RequestBuilder::timeout) of a request limits
    /// all of it in addition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Platform, Request};
    /// use std::time::Duration;
    ///
    /// let platform = agplatform::platform();
    /// let request = platform.request();
    /// let client = request
    ///     .client_builder()
    ///     .timeout(Duration::from_secs(60))
    ///     .build()
    ///     .unwrap();
    /// ```
    #[cfg(feature = "tokio")]
    fn client_builder(&self) -> reqwest::ClientBuilder;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Sets the builder of the default client, replacing the previous
    /// client. The client is built by the next [`client`](Self::client)
    /// call.
    #[cfg(feature = "tokio")]
    fn set_client(&self, builder: reqwest::ClientBuilder);

    /// Enabled by the `tokio` feature flag.
    ///
    /// Sends `request` like [`Self::send_blocking`], without blocking the current
    /// thread. Must be awaited on a [`tokio`] runtime with I/O and time
    /// enabled. Dropping the future cancels the request, which the
    /// interceptors are told by [`Interceptor::cancelled`].
    ///
    /// # Errors
    ///
    /// As [`Self::send_blocking`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Platform, Request, RequestBuilderExt, RequestOutput};
    ///
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// let platform = agplatform::platform();
    /// let request = platform.request();
    /// # let request = agplatform::TestRequest::new();
    /// # request.set_response("", RequestOutput { status: 200, body: b"pong".to_vec(), ..Default::default() });
    ///
    /// let client = request.client().unwrap();
    /// let output = request.send_async(client.get("https://example.com/ping")).await.unwrap();
    /// assert_eq!(output.text().unwrap(), "pong");
    /// # });
    /// ```
    #[cfg(feature = "tokio")]
    fn send_async(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::RequestBuilder>>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Sends `request` like [`Self::send_async`], passing the response of a
    /// success to `sink` like [`Self::send_blocking_with_sink`], its futures
    /// awaited on the runtime of the request, e.g. sending the body to a
    /// bounded channel of a streamed response of a proxy.
    ///
    /// # Errors
    ///
    /// As [`Self::send_blocking_with_sink`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::Sink;
    /// use agplatform::{BoxFuture, HttpRequest, Platform, Request, RequestOutput};
    /// use std::sync::Arc;
    ///
    /// /// Forwards the status, then the body, to a bounded channel.
    /// struct Forward(tokio::sync::mpsc::Sender<Vec<u8>>);
    ///
    /// impl Forward {
    ///     async fn send(&self, data: Vec<u8>) -> agplatform::Result {
    ///         self.0.send(data).await.map_err(|_| agplatform::Error::other("receiver dropped"))
    ///     }
    /// }
    ///
    /// impl Sink<HttpRequest> for Forward {
    ///     fn head<'a>(&'a mut self, head: &'a RequestOutput) -> BoxFuture<'a, agplatform::Result> {
    ///         Box::pin(self.send(head.status.to_string().into_bytes()))
    ///     }
    ///
    ///     fn write<'a>(&'a mut self, (): (), data: &'a [u8]) -> BoxFuture<'a, agplatform::Result> {
    ///         Box::pin(self.send(data.to_vec()))
    ///     }
    /// }
    ///
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// let platform = Arc::new(agplatform::platform());
    /// # let platform = Arc::new(agplatform::test_platform::test_platform());
    /// # platform.request().set_response("", RequestOutput::status(200).body("data"));
    /// let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    /// let shared = platform.clone();
    ///
    /// let sent = tokio::spawn(async move {
    ///     let client = shared.request().client()?;
    ///     let download = client.get("https://example.com/download");
    ///     shared.request().send_async_with_sink(download, &mut Forward(sender)).await
    /// });
    ///
    /// let mut received = Vec::new();
    /// while let Some(data) = receiver.recv().await {
    ///     received.push(data);
    /// }
    /// assert_eq!(sent.await.unwrap().unwrap().status, 200);
    /// assert_eq!(received, [b"200".to_vec(), b"data".to_vec()]);
    /// # });
    /// ```
    #[cfg(feature = "tokio")]
    fn send_async_with_sink(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::RequestBuilder>>,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send;

    /// Returns the interceptors of the runs, e.g. to set them with changes
    /// on a run, see [`ConfiguredRequest::interceptors`].
    fn interceptors(&self) -> Interceptors<HttpRequest>;

    /// Removes the interceptors, none by default, for the runs starting
    /// afterwards.
    fn clear_interceptors(&self);

    /// Adds `interceptor` as the innermost one for the runs starting
    /// afterwards. Its state to be read later, e.g. what it recorded, is
    /// shared through an [`Arc`](std::sync::Arc) it holds.
    ///
    /// See [`HttpRequest`].
    fn add_interceptor(&self, interceptor: impl Interceptor<HttpRequest>);
}

/// Enabled by the `request` and `tokio` feature flags.
///
/// Extends [`reqwest::RequestBuilder`] with the ability to send it
/// through a [`Request`].
///
/// # Examples
///
/// ```rust
/// use agplatform::{Platform, Request, RequestBuilderExt, RequestOutput};
///
/// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
/// let platform = agplatform::platform();
/// let request = platform.request();
/// # let request = &agplatform::TestRequest::new();
/// # request.set_response("", RequestOutput { status: 200, ..Default::default() });
/// let client = request.client().unwrap();
///
/// client.get("https://example.com").send_async(request).await.unwrap();
/// client
///     .post("https://example.com/upload")
///     .bearer_auth("token")
///     .body("data")
///     .display_lines(3)
///     .send_async(request)
///     .await
///     .unwrap();
/// # });
/// ```
#[cfg(feature = "tokio")]
pub trait RequestBuilderExt: Sized {
    /// Sends the request asynchronously with the default
    /// [`RequestOptions`].
    ///
    /// See [`Request::send_async`].
    fn send_async<R: Request + ?Sized>(
        self,
        request: &R,
    ) -> impl Future<Output = Result<RequestOutput>> + Send;

    /// Sends the request asynchronously with the default
    /// [`RequestOptions`], passing the body of a successful response to
    /// `sink`.
    ///
    /// See [`Request::send_async_with_sink`].
    fn send_async_with_sink<R: Request + ?Sized>(
        self,
        request: &R,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send;

    /// Returns a [`ConfiguredRequest`] whose response shows its last
    /// `lines` when displayed.
    ///
    /// See [`RequestOptions::display_lines`].
    fn display_lines(self, lines: usize) -> ConfiguredRequest<Self>;

    /// Returns a [`ConfiguredRequest`] with its own `interceptors`.
    ///
    /// See [`ConfiguredRequest::interceptors`].
    fn interceptors(self, interceptors: Interceptors<HttpRequest>) -> ConfiguredRequest<Self>;

    /// Returns a [`ConfiguredRequest`] keeping the part of the response
    /// body received when it fails.
    ///
    /// See [`RequestOptions::keep_partial_body`].
    fn keep_partial_body(self) -> ConfiguredRequest<Self>;

    /// Returns a [`ConfiguredRequest`] with `value` for the interceptors.
    ///
    /// See [`ConfiguredRequest::with`].
    fn with<T: Any + Send + Sync>(self, value: T) -> ConfiguredRequest<Self>;
}

/// Enabled by the `request` feature flag.
///
/// Extends [`reqwest::blocking::RequestBuilder`] with the ability to send
/// it through a [`Request`].
///
/// # Examples
///
/// ```rust
/// use agplatform::{BlockingRequestBuilderExt, Platform, Request, RequestOutput};
///
/// let platform = agplatform::platform();
/// let request = platform.request();
/// # let request = &agplatform::TestRequest::new();
/// # request.set_response("", RequestOutput { status: 200, ..Default::default() });
/// let client = request.blocking_client().unwrap();
///
/// client.get("https://example.com").send_blocking(request).unwrap();
/// client
///     .get("https://example.com/download")
///     .keep_partial_body()
///     .send_blocking(request)
///     .unwrap();
/// ```
pub trait BlockingRequestBuilderExt: Sized {
    /// Sends the request with the default [`RequestOptions`].
    ///
    /// See [`Request::send_blocking`].
    fn send_blocking<R: Request + ?Sized>(self, request: &R) -> Result<RequestOutput>;

    /// Sends the request with the default [`RequestOptions`], passing the
    /// body of a successful response to `sink`.
    ///
    /// See [`Request::send_blocking_with_sink`].
    fn send_blocking_with_sink<R: Request + ?Sized>(
        self,
        request: &R,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> Result<RequestOutput>;

    /// Returns a [`ConfiguredRequest`] whose response shows its last
    /// `lines` when displayed.
    ///
    /// See [`RequestOptions::display_lines`].
    fn display_lines(self, lines: usize) -> ConfiguredRequest<Self>;

    /// Returns a [`ConfiguredRequest`] with its own `interceptors`.
    ///
    /// See [`ConfiguredRequest::interceptors`].
    fn interceptors(self, interceptors: Interceptors<HttpRequest>) -> ConfiguredRequest<Self>;

    /// Returns a [`ConfiguredRequest`] keeping the part of the response
    /// body received when it fails.
    ///
    /// See [`RequestOptions::keep_partial_body`].
    fn keep_partial_body(self) -> ConfiguredRequest<Self>;

    /// Returns a [`ConfiguredRequest`] with `value` for the interceptors.
    ///
    /// See [`ConfiguredRequest::with`].
    fn with<T: Any + Send + Sync>(self, value: T) -> ConfiguredRequest<Self>;
}

/// [internal]
///
/// The response of an attempt of a run as it arrives, its body passed to
/// `emit`, which passes only the head and the body of a successful
/// response to the sink and captures the rest, kept for a [`TransportError::output`], see
/// [`RequestOptions::keep_partial_body`].
pub(crate) struct Received<'a> {
    subject: &'a HttpRequest,
    emit: Emit<'a, HttpRequest>,
    output: RequestOutput,
}

#[cfg(feature = "tokio")]
impl RequestBuilderExt for reqwest::RequestBuilder {
    fn send_async<R: Request + ?Sized>(
        self,
        request: &R,
    ) -> impl Future<Output = Result<RequestOutput>> + Send {
        request.send_async(self)
    }

    fn send_async_with_sink<R: Request + ?Sized>(
        self,
        request: &R,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send {
        request.send_async_with_sink(self, sink)
    }

    fn display_lines(self, lines: usize) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).display_lines(lines)
    }

    fn interceptors(self, interceptors: Interceptors<HttpRequest>) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).interceptors(interceptors)
    }

    fn keep_partial_body(self) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).keep_partial_body()
    }

    fn with<T: Any + Send + Sync>(self, value: T) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).with(value)
    }
}

impl BlockingRequestBuilderExt for reqwest::blocking::RequestBuilder {
    fn send_blocking<R: Request + ?Sized>(self, request: &R) -> Result<RequestOutput> {
        request.send_blocking(self)
    }

    fn send_blocking_with_sink<R: Request + ?Sized>(
        self,
        request: &R,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> Result<RequestOutput> {
        request.send_blocking_with_sink(self, sink)
    }

    fn display_lines(self, lines: usize) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).display_lines(lines)
    }

    fn interceptors(self, interceptors: Interceptors<HttpRequest>) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).interceptors(interceptors)
    }

    fn keep_partial_body(self) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).keep_partial_body()
    }

    fn with<T: Any + Send + Sync>(self, value: T) -> ConfiguredRequest<Self> {
        ConfiguredRequest::new(self).with(value)
    }
}

impl<'a> Received<'a> {
    /// Starts the response to `run` with `status` and `headers`, its body
    /// passed to `emit`, after its head when it goes to the sink, see
    /// [`Request::send_blocking_with_sink`].
    ///
    /// # Errors
    ///
    /// Returns the error of the sink refusing the head, which ends the
    /// attempt.
    pub(crate) async fn start(
        run: &'a Run<HttpRequest>,
        mut emit: Emit<'a, HttpRequest>,
        status: u16,
        headers: HeaderMap,
    ) -> Result<Self> {
        let (subject, success) = (run.subject(), is_success(status));
        let output = RequestOutput {
            success,
            status,
            headers,
            display_lines: subject.options().display_lines,
            ..Default::default()
        };

        if success {
            emit.head(&output).await?;
        } else {
            emit.skip_sink();
        }

        Ok(Self {
            subject,
            emit,
            output,
        })
    }

    /// Passes the `data` of the body on.
    ///
    /// # Errors
    ///
    /// Returns the error of an interceptor or the sink, which ends the
    /// attempt.
    pub(crate) async fn push(&mut self, data: &[u8]) -> Result {
        self.emit.emit((), data).await
    }

    /// Returns the response received after `duration`, its body captured
    /// by the engine.
    pub(crate) fn end(self, duration: Duration) -> RequestOutput {
        RequestOutput {
            duration,
            ..self.output
        }
    }

    /// Returns the error of the body failing to be received with `error`
    /// after `duration`, with the part of the body captured if kept.
    pub(crate) fn interrupted(mut self, duration: Duration, error: TransportError) -> crate::Error {
        let body = if self.subject.options().keep_partial_body {
            self.emit.take_captured(())
        } else {
            Vec::new()
        };

        transport_failure(
            self.subject,
            error.with_output(RequestOutput {
                body,
                duration,
                ..self.output
            }),
        )
    }
}

const DISPLAY_LINES: usize = 10;

/// Returns whether `status` is a success (2xx).
fn is_success(status: u16) -> bool {
    (200..300).contains(&status)
}

/// [internal]
///
/// Runs `run` by `executor` through `runs`, passing the body of a
/// successful response to `sink`, if any, and returns its output, an error
/// when it is not a success, see [`Request::send_blocking`].
pub(crate) fn run_blocking<E: BlockingExecutor<Subject = HttpRequest>>(
    runs: &Runs<HttpRequest>,
    executor: &E,
    mut run: Run<HttpRequest>,
    sink: Option<&mut dyn Sink<HttpRequest>>,
) -> Result<RequestOutput> {
    let result = runs.run_blocking(executor, &mut run, sink);
    conclude(result, |output| failure(run.subject(), output))
}

/// [internal]
///
/// Runs `run` like [`run_blocking`] asynchronously, see
/// [`Request::send_async`].
#[cfg(feature = "tokio")]
pub(crate) async fn run_async<E: AsyncExecutor<Subject = HttpRequest>>(
    runs: &Runs<HttpRequest>,
    executor: &E,
    mut run: Run<HttpRequest>,
    sink: Option<&mut dyn Sink<HttpRequest>>,
) -> Result<RequestOutput> {
    let result = runs.run(executor, &mut run, sink).await;
    conclude(result, |output| failure(run.subject(), output))
}

/// Returns the error of `request` failing with `output` as its source.
fn failure(request: &HttpRequest, output: RequestOutput) -> crate::Error {
    let target = request.target();
    let error = if is_success(output.status) {
        request_error!("'{target}' was not accepted with status {}", output.status)
    } else {
        request_error!("'{target}' failed with status {}", output.status)
    };

    error.with_subject(target).with_source(output)
}

/// [internal]
///
/// Returns the error of `request` failing with the transport `error` as its
/// source.
pub(crate) fn transport_failure(request: &HttpRequest, error: TransportError) -> crate::Error {
    let target = request.target();
    request_error!("'{target}' {error}")
        .with_subject(target)
        .with_source(error)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::TestRequest;
    use crate::interceptor::BoxFuture;
    use crate::interceptor::Chunk;
    use crate::interceptor::DryRun;
    use crate::interceptor::Next;
    use crate::utils::OwnInterceptors;
    use reqwest::Method;
    use reqwest::Url;
    use reqwest::header::HeaderName;
    use std::io::BufRead as _;
    use std::io::Read as _;
    use std::io::Write as _;
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::sync::Arc;
    use std::sync::Mutex;

    pub(crate) struct Server {
        pub(crate) url: String,
        pub(super) received: Arc<Mutex<Vec<String>>>,
    }

    /// A request as received by the server of a contract case.
    pub(crate) struct Sent {
        pub(crate) method: String,
        pub(crate) headers: HeaderMap,
        pub(crate) body: Vec<u8>,
    }

    impl Server {
        pub(crate) fn received(&self) -> Vec<String> {
            self.received.lock().unwrap().clone()
        }

        /// Returns the requests received, parsed.
        pub(crate) fn sent(&self) -> Vec<Sent> {
            self.received()
                .iter()
                .map(|raw| {
                    let (head, body) = raw.split_once("\r\n\r\n").unwrap();
                    let mut lines = head.lines();
                    let method = lines.next().unwrap().split(' ').next().unwrap();
                    let headers = lines
                        .map(|line| {
                            let (name, value) = line.split_once(": ").unwrap();
                            (
                                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                                value.parse().unwrap(),
                            )
                        })
                        .collect();
                    Sent {
                        method: method.to_string(),
                        headers,
                        body: body.as_bytes().to_vec(),
                    }
                })
                .collect()
        }
    }

    /// Answers each connection with what `respond` returns for its index
    /// and request, then closes it.
    pub(crate) fn serve(respond: impl Fn(usize, &str) -> Vec<u8> + Send + 'static) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let received = Arc::new(Mutex::new(Vec::new()));
        let log = received.clone();

        std::thread::spawn(move || {
            for (index, stream) in listener.incoming().enumerate() {
                let mut stream = stream.unwrap();
                let request = read_request(&stream);
                log.lock().unwrap().push(request.clone());
                let _ = stream.write_all(&respond(index, &request));
            }
        });

        Server { url, received }
    }

    /// Answers each connection with a success whose body is the `chunks`,
    /// each sent after `pause`.
    pub(crate) fn serve_chunks(pause: Duration, chunks: &'static [&'static str]) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                read_request(&stream);
                let length: usize = chunks.iter().map(|chunk| chunk.len()).sum();
                let head = format!("HTTP/1.1 200 OK\r\ncontent-length: {length}\r\n\r\n");
                let _ = stream.write_all(head.as_bytes());

                for chunk in chunks {
                    std::thread::sleep(pause);
                    let _ = stream.write_all(chunk.as_bytes());
                }
            }
        });

        Server {
            url,
            received: Arc::default(),
        }
    }

    /// Answers one connection with a success whose body stops after
    /// `partial` for 2 seconds.
    pub(crate) fn serve_stalled() -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());

        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            read_request(&stream);
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 100\r\n\r\npartial");
            std::thread::sleep(Duration::from_secs(2));
        });

        Server {
            url,
            received: Arc::default(),
        }
    }

    pub(super) fn read_request(stream: &TcpStream) -> String {
        let mut reader = std::io::BufReader::new(stream);
        let mut request = String::new();
        while reader.read_line(&mut request).unwrap() > 2 {}
        let length = request
            .to_lowercase()
            .lines()
            .find_map(|line| line.strip_prefix("content-length: ").map(str::to_string))
            .map_or(0, |length| length.parse().unwrap());
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        request + &String::from_utf8_lossy(&body)
    }

    pub(crate) fn response(status: u16, body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 {status} Status\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    pub(crate) fn truncated(_: usize, _: &str) -> Vec<u8> {
        b"HTTP/1.1 200 OK\r\ncontent-length: 100\r\nconnection: close\r\n\r\npartial".to_vec()
    }

    pub(crate) fn slow(_: usize, _: &str) -> Vec<u8> {
        std::thread::sleep(Duration::from_secs(2));
        response(200, "late")
    }

    pub(crate) fn ok(_: usize, _: &str) -> Vec<u8> {
        response(200, "body")
    }

    pub(crate) fn not_found(_: usize, _: &str) -> Vec<u8> {
        response(404, "missing")
    }

    /// Returns the URL of a port nothing listens on. The port is below the
    /// ephemeral range, so the servers of parallel tests, bound to port 0,
    /// cannot take it over as they could a released ephemeral one.
    pub(crate) fn closed_url() -> String {
        static PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();
        let port = PORT.get_or_init(|| {
            (1..1024)
                .find(|port| TcpStream::connect(("127.0.0.1", *port)).is_err())
                .unwrap()
        });
        format!("http://127.0.0.1:{port}")
    }

    pub(crate) fn invalid_client() -> reqwest::blocking::ClientBuilder {
        reqwest::blocking::Client::builder().user_agent("\n")
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn invalid_async_client() -> reqwest::ClientBuilder {
        reqwest::Client::builder().user_agent("\n")
    }

    pub(crate) struct FailingReader;

    impl std::io::Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("read failed"))
        }
    }

    #[cfg(feature = "tokio")]
    pub(crate) struct FailingBody;

    #[cfg(feature = "tokio")]
    impl http_body::Body for FailingBody {
        type Data = &'static [u8];
        type Error = std::io::Error;

        fn poll_frame(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<std::result::Result<http_body::Frame<Self::Data>, Self::Error>>>
        {
            std::task::Poll::Ready(Some(Err(std::io::Error::other("read failed"))))
        }
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn streamed(data: &'static [u8]) -> reqwest::Body {
        reqwest::Body::wrap(http_body_util::Full::new(data))
    }

    /// Records the runs it handles, the chunks of the streamed ones passing
    /// through it and the cancelled runs.
    #[derive(Clone, Default)]
    pub(crate) struct Recorder(pub(super) Arc<Mutex<Vec<String>>>);

    impl Recorder {
        pub(crate) fn events(&self) -> Vec<String> {
            self.0.lock().unwrap().clone()
        }

        pub(crate) fn push(&self, event: String) {
            self.0.lock().unwrap().push(event);
        }
    }

    impl Interceptor<HttpRequest> for Recorder {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, HttpRequest>,
        ) -> BoxFuture<'a, Result<RequestOutput>> {
            Box::pin(async move {
                self.push(format!("started {}", next.run().subject().target()));
                let result = next.attempt().await;
                self.push(format!(
                    "finished {}",
                    result.as_ref().map_or_else(
                        |error| error.to_string(),
                        |output| output.status.to_string()
                    )
                ));
                result
            })
        }

        fn chunk(&self, chunk: Chunk<HttpRequest>) -> Result {
            if chunk.run().subject().options().keep_partial_body {
                self.push(format!("body {}", String::from_utf8_lossy(chunk.data())));
            }

            chunk.forward()
        }

        fn cancelled(&self, run: &Run<HttpRequest>) {
            self.push(format!("cancelled {}", run.subject().target()));
        }
    }

    /// Changes of the env the request shares, like by its platform.
    impl RequestImpl {
        pub(crate) fn set_var(&self, key: &str, value: &str) {
            crate::Env::set_var(&self.clients.env, key, value).unwrap();
        }

        pub(crate) fn remove_var(&self, key: &str) {
            crate::Env::remove_var(&self.clients.env, key);
        }
    }

    /// Returns a request whose only interceptor is the returned recorder.
    pub(super) fn observed() -> (RequestImpl, Recorder) {
        let recorder = Recorder::default();
        let request = RequestImpl::new();
        request.add_interceptor(recorder.clone());
        (request, recorder)
    }

    /// Returns a request with a single interceptor `interceptor`.
    pub(super) fn intercepted(interceptor: impl Interceptor<HttpRequest>) -> RequestImpl {
        let request = RequestImpl::new();
        request.add_interceptor(interceptor);
        request
    }

    /// Returns the interceptors `first` and `second`, in that order.
    pub(super) fn chain(
        first: impl Interceptor<HttpRequest>,
        second: impl Interceptor<HttpRequest>,
    ) -> Interceptors<HttpRequest> {
        let mut interceptors = Interceptors::new();
        interceptors.push(first);
        interceptors.push(second);
        interceptors
    }

    pub(super) fn output(status: u16, body: &[u8]) -> RequestOutput {
        RequestOutput {
            status,
            body: body.to_vec(),
            ..Default::default()
        }
    }

    /// Returns the request of `method` to `http://h`.
    pub(super) fn subject(method: Method) -> HttpRequest {
        let request = reqwest::blocking::Request::new(method, Url::parse("http://h").unwrap());
        HttpRequest::from_blocking(request, RequestOptions::default()).unwrap()
    }

    pub(crate) fn blocking_get(
        request: &RequestImpl,
        url: &str,
    ) -> reqwest::blocking::RequestBuilder {
        request.blocking_client().unwrap().get(url)
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn get(request: &RequestImpl, url: &str) -> reqwest::RequestBuilder {
        request.client().unwrap().get(url)
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    pub(super) fn transport_kind(result: &Result<RequestOutput>) -> Option<TransportErrorKind> {
        result
            .as_ref()
            .err()
            .and_then(|error| error.source_as::<TransportError>())
            .map(TransportError::kind)
    }

    pub(super) fn collect_into(
        received: &mut Vec<u8>,
    ) -> impl FnMut((), &[u8]) -> Result + Send + '_ {
        |(), data| {
            received.extend_from_slice(data);
            Ok(())
        }
    }

    mod blocking_request_builder_ext {
        use super::*;

        mod send_blocking {
            use super::*;

            #[test]
            fn send_blocking_should_send_request_with_default_options() {
                let server = serve(ok);
                let request = RequestImpl::new();

                let output = blocking_get(&request, &server.url)
                    .send_blocking(&request)
                    .unwrap();

                assert_eq!(output.body, b"body");
            }
        }

        mod send_blocking_with_sink {
            use super::*;

            #[test]
            fn send_blocking_with_sink_should_send_request_with_default_options_to_sink() {
                let server = serve(ok);
                let request = RequestImpl::new();
                let mut received = Vec::new();

                blocking_get(&request, &server.url)
                    .send_blocking_with_sink(&request, &mut collect_into(&mut received))
                    .unwrap();

                assert_eq!(received, b"body");
            }
        }

        mod display_lines {
            use super::*;

            #[test]
            fn display_lines_should_set_lines() {
                let request = reqwest::blocking::Client::new()
                    .get("http://localhost")
                    .display_lines(3);

                assert_eq!(request.options().display_lines, 3);
            }
        }

        mod interceptors {
            use super::*;

            #[test]
            fn interceptors_should_set_own_interceptors_of_run() {
                let mut interceptors = Interceptors::new();
                interceptors.push(DryRun);

                let (_, run) = reqwest::blocking::Client::new()
                    .get("http://localhost")
                    .interceptors(interceptors.clone())
                    .into_run();

                assert_eq!(
                    run.unwrap()
                        .get::<OwnInterceptors<HttpRequest>>()
                        .map(|own| &own.0),
                    Some(&interceptors)
                );
            }
        }

        mod keep_partial_body {
            use super::*;

            #[test]
            fn keep_partial_body_should_set_option() {
                let request = reqwest::blocking::Client::new()
                    .get("http://localhost")
                    .keep_partial_body();

                assert!(request.options().keep_partial_body);
            }
        }

        mod with {
            use super::*;

            #[test]
            fn with_should_set_value_for_interceptors() {
                let request = reqwest::blocking::Client::new()
                    .get("http://localhost")
                    .with(7_u32);

                let (_, run) = request.into_run();

                assert_eq!(run.unwrap().get::<u32>(), Some(&7));
            }
        }
    }

    #[cfg(feature = "tokio")]
    mod request_builder_ext {
        use super::*;

        mod send_async {
            use super::*;

            #[test]
            fn send_async_should_send_request_with_default_options() {
                let server = serve(ok);
                let request = RequestImpl::new();

                let output = block_on(get(&request, &server.url).send_async(&request)).unwrap();

                assert_eq!(output.body, b"body");
            }
        }

        mod send_async_with_sink {
            use super::*;

            #[test]
            fn send_async_with_sink_should_send_request_with_default_options_to_sink() {
                let server = serve(ok);
                let request = RequestImpl::new();
                let mut received = Vec::new();

                block_on(
                    get(&request, &server.url)
                        .send_async_with_sink(&request, &mut collect_into(&mut received)),
                )
                .unwrap();

                assert_eq!(received, b"body");
            }
        }

        mod display_lines {
            use super::*;

            #[test]
            fn display_lines_should_set_lines() {
                let request = reqwest::Client::new()
                    .get("http://localhost")
                    .display_lines(3);

                assert_eq!(request.options().display_lines, 3);
            }
        }

        mod interceptors {
            use super::*;

            #[test]
            fn interceptors_should_set_own_interceptors_of_run() {
                let mut interceptors = Interceptors::new();
                interceptors.push(DryRun);

                let (_, run) = block_on(
                    reqwest::Client::new()
                        .get("http://localhost")
                        .interceptors(interceptors.clone())
                        .into_run(),
                );

                assert_eq!(
                    run.unwrap()
                        .get::<OwnInterceptors<HttpRequest>>()
                        .map(|own| &own.0),
                    Some(&interceptors)
                );
            }
        }

        mod keep_partial_body {
            use super::*;

            #[test]
            fn keep_partial_body_should_set_option() {
                let request = reqwest::Client::new()
                    .get("http://localhost")
                    .keep_partial_body();

                assert!(request.options().keep_partial_body);
            }
        }

        mod with {
            use super::*;

            #[test]
            fn with_should_set_value_for_interceptors() {
                let request = reqwest::Client::new().get("http://localhost").with(7_u32);

                let (_, run) = block_on(request.into_run());

                assert_eq!(run.unwrap().get::<u32>(), Some(&7));
            }
        }
    }

    mod received {
        use super::*;

        mod interrupted {
            use super::*;

            #[test]
            fn interrupted_should_keep_partial_body_as_passed_on() {
                // GIVEN an interceptor redacting a secret of a body cut off
                struct Redact;

                impl Interceptor<HttpRequest> for Redact {
                    fn chunk(&self, mut chunk: Chunk<HttpRequest>) -> Result {
                        let data = String::from_utf8_lossy(chunk.data()).replace("secret", "***");
                        chunk.set_data(data.into_bytes());
                        chunk.forward()
                    }
                }

                let request = TestRequest::new();
                request.add_interceptor(Redact);
                request.set_interrupted(
                    "",
                    output(200, b""),
                    [b"token=secret".to_vec()],
                    TransportErrorKind::Other,
                );

                // WHEN it is received keeping the partial body
                let error = request
                    .blocking_client()
                    .unwrap()
                    .get("https://example.com")
                    .keep_partial_body()
                    .send_blocking(&request)
                    .unwrap_err();

                // THEN the body kept is the redacted one
                let body = &error
                    .source_as::<TransportError>()
                    .unwrap()
                    .output()
                    .unwrap()
                    .body;
                assert_eq!(body, b"token=***");
            }
        }
    }

    mod failure {
        use super::*;

        #[test]
        fn failure_should_return_error_with_output() {
            for (status, description) in [
                (404, "'GET http://h/' failed with status 404"),
                (200, "'GET http://h/' was not accepted with status 200"),
            ] {
                let error = failure(&subject(Method::GET), output(status, b""));

                assert_eq!(error.description(), description);
                assert_eq!(error.request_output().unwrap().status, status);
                assert_eq!(error.request_target(), Some("GET http://h/"));
            }
        }
    }

    /// The contract cases of [`Request`]. Each takes the `url` of a request
    /// answered as its documentation says, by a local server or the
    /// responses set, and when it checks what was sent the requests
    /// received by it from `sent`.
    pub(crate) mod request {
        use super::*;

        fn blocking(request: &impl Request, url: &str) -> reqwest::blocking::RequestBuilder {
            request.blocking_client().unwrap().get(url)
        }

        #[cfg(feature = "tokio")]
        fn not_blocking(request: &impl Request, url: &str) -> reqwest::RequestBuilder {
            request.client().unwrap().get(url)
        }

        fn interrupted_output(result: &Result<RequestOutput>) -> RequestOutput {
            result
                .as_ref()
                .unwrap_err()
                .source_as::<TransportError>()
                .and_then(TransportError::output)
                .unwrap()
                .clone()
        }

        fn own(retry: crate::interceptor::Retry<HttpRequest>) -> Interceptors<HttpRequest> {
            let mut interceptors = Interceptors::new();
            interceptors.push(retry);
            interceptors
        }

        fn recorded(request: &impl Request) -> Recorder {
            let recorder = Recorder::default();
            request.add_interceptor(recorder.clone());
            recorder
        }

        /// Records the head and the chunks of the body passed to it, each
        /// once it was pending once, refusing the head when `refuse`.
        #[derive(Default)]
        struct Heads {
            events: Vec<String>,
            refuse: bool,
        }

        impl Sink<HttpRequest> for Heads {
            fn head<'a>(&'a mut self, head: &'a RequestOutput) -> BoxFuture<'a, Result> {
                Box::pin(async move {
                    PendingOnce(false).await;

                    if self.refuse {
                        return Err(crate::Error::fs("head refused"));
                    }

                    self.events.push(format!("head {}", head.status));
                    Ok(())
                })
            }

            fn write<'a>(&'a mut self, (): (), data: &'a [u8]) -> BoxFuture<'a, Result> {
                Box::pin(async move {
                    PendingOnce(false).await;
                    self.events.push(String::from_utf8_lossy(data).into_owned());
                    Ok(())
                })
            }
        }

        /// Pending once, waking itself.
        struct PendingOnce(bool);

        impl Future for PendingOnce {
            type Output = ();

            fn poll(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<()> {
                if self.0 {
                    return std::task::Poll::Ready(());
                }

                self.0 = true;
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
        }

        pub(crate) mod blocking_client {
            use super::*;

            pub(crate) fn blocking_client_invalid_builder_should_return_error_every_time(
                request: &impl Request,
            ) {
                request.set_blocking_client(invalid_client());

                let first = request.blocking_client().unwrap_err();
                let second = request.blocking_client().unwrap_err();

                assert!(
                    first
                        .description()
                        .starts_with("client could not be built: ")
                );
                assert_eq!(first.description(), second.description());
            }
        }

        pub(crate) mod blocking_client_builder {
            use super::*;

            /// `env` is the env of the `request`.
            pub(crate) fn blocking_client_builder_should_use_proxies_of_env(
                request: &impl Request,
                env: &crate::env::EnvImpl,
            ) {
                crate::Env::set_var(env, "HTTP_PROXY", "http://proxy.invalid:3128").unwrap();

                let builder = request.blocking_client_builder();

                assert!(
                    format!("{builder:?}").contains("proxy.invalid"),
                    "{builder:?}"
                );
            }

            pub(crate) fn blocking_client_builder_should_time_out_connecting_after_30_seconds(
                request: &impl Request,
            ) {
                let builder = request.blocking_client_builder();

                assert!(format!("{builder:?}").contains("connect_timeout: 30s"));
            }
        }

        pub(crate) mod set_blocking_client {
            use super::*;

            pub(crate) fn set_blocking_client_should_replace_default_client(
                request: &impl Request,
            ) {
                request.blocking_client().unwrap();

                request.set_blocking_client(invalid_client());

                assert!(request.blocking_client().is_err());
            }
        }

        pub(crate) mod send_blocking {
            use super::*;

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_blocking_should_return_response_and_pass_run_to_interceptors(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);

                let output = blocking(request, &format!("{url}?token=secret"))
                    .send_blocking(request)
                    .unwrap();

                assert!(output.success);
                assert_eq!(output.status, 200);
                assert_eq!(output.body, b"body");
                assert_eq!(
                    recorder.events(),
                    [format!("started GET {url}"), "finished 200".to_string()]
                );
            }

            /// `url` answers `200`.
            pub(crate) fn send_blocking_should_send_request_body_and_headers(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                request
                    .blocking_client()
                    .unwrap()
                    .post(url)
                    .bearer_auth("token")
                    .header("x-key", "a")
                    .header("x-key", "b")
                    .body("data")
                    .send_blocking(request)
                    .unwrap();

                let sent = &sent()[0];
                assert_eq!(sent.method, "POST");
                assert_eq!(sent.headers["authorization"], "Bearer token");
                assert_eq!(
                    sent.headers.get_all("x-key").iter().collect::<Vec<_>>(),
                    ["a", "b"]
                );
                assert_eq!(sent.body, b"data");
            }

            /// `url` answers `404` with the body `missing`.
            pub(crate) fn send_blocking_failed_status_should_return_error_with_output(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);

                let error = blocking(request, &format!("{url}?token=secret"))
                    .send_blocking(request)
                    .unwrap_err();

                assert_eq!(
                    error.description(),
                    format!("'GET {url}' failed with status 404")
                );
                assert_eq!(error.request_target(), Some(format!("GET {url}").as_str()));
                let output = error.request_output().unwrap();
                assert_eq!(output.status, 404);
                assert_eq!(output.body, b"missing");
                assert_eq!(recorder.events().last().unwrap(), "finished 404");
            }

            /// `url` answers `500` with the body `1\n2\n3`.
            pub(crate) fn send_blocking_display_lines_should_limit_displayed_body(
                request: &impl Request,
                url: &str,
            ) {
                let error = blocking(request, url)
                    .display_lines(1)
                    .send_blocking(request)
                    .unwrap_err();

                let output = error.request_output().unwrap();
                assert_eq!(output.display_lines, 1);
                assert!(output.to_string().ends_with("body:\n(2 lines omitted)\n3"));
            }

            pub(crate) fn send_blocking_invalid_request_should_return_error_bypassing_interceptors(
                request: &impl Request,
            ) {
                let recorder = recorded(request);

                let error = blocking(request, "not a url")
                    .send_blocking(request)
                    .unwrap_err();

                assert_eq!(error.description(), "invalid request: builder error");
                assert!(recorder.events().is_empty());
            }

            pub(crate) fn send_blocking_unreadable_body_should_return_error_bypassing_interceptors(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);

                let error = request
                    .blocking_client()
                    .unwrap()
                    .post(url)
                    .body(reqwest::blocking::Body::new(FailingReader))
                    .send_blocking(request)
                    .unwrap_err();

                assert!(
                    error
                        .description()
                        .starts_with("request body could not be read: "),
                    "{error}"
                );
                assert!(recorder.events().is_empty());
            }

            /// `url` refuses the connection.
            pub(crate) fn send_blocking_connect_failure_should_return_connect_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);

                let result = blocking(request, url).send_blocking(request);

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Connect));
                let description = format!("'GET {url}' failed to connect");
                assert!(result.unwrap_err().description().starts_with(&description));
                assert!(recorder.events()[1].contains(&description));
            }

            /// `url` answers after more than 50 ms.
            pub(crate) fn send_blocking_timeout_should_return_timeout_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                let result = blocking(request, url)
                    .timeout(Duration::from_millis(50))
                    .send_blocking(request);

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Timeout));
                assert!(
                    result
                        .unwrap_err()
                        .description()
                        .starts_with(&format!("'GET {url}' timed out"))
                );
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_blocking_zero_timeout_should_return_timeout_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                let result = blocking(request, url)
                    .timeout(Duration::ZERO)
                    .send_blocking(request);

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Timeout));
            }

            /// `url` answers `200` with a `content-length` header but stops
            /// sending its body for more than 300 ms after `partial`.
            pub(crate) fn send_blocking_stalled_body_should_return_timeout_transport_error_with_head(
                request: &impl Request,
                url: &str,
            ) {
                let result = blocking(request, url)
                    .timeout(Duration::from_millis(300))
                    .keep_partial_body()
                    .send_blocking(request);

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Timeout));
                let output = interrupted_output(&result);
                assert_eq!(output.status, 200);
                assert!(output.headers.contains_key("content-length"));
                assert_eq!(output.body, b"partial");
            }

            /// `url` answers `200` with a `content-length` header but closes
            /// the connection after the body `partial`.
            pub(crate) fn send_blocking_truncated_body_should_return_other_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                for keep in [false, true] {
                    let builder = blocking(request, url);
                    let builder = if keep {
                        builder.keep_partial_body()
                    } else {
                        ConfiguredRequest::new(builder)
                    };

                    let result = builder.send_blocking(request);

                    assert_eq!(
                        transport_kind(&result),
                        Some(TransportErrorKind::Other),
                        "keep: {keep}"
                    );
                    let output = interrupted_output(&result);
                    let partial: &[u8] = if keep { b"partial" } else { b"" };
                    assert_eq!(output.status, 200, "keep: {keep}");
                    assert!(output.headers.contains_key("content-length"));
                    assert_eq!(output.body, partial, "keep: {keep}");
                }
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_blocking_streaming_should_pass_body_between_start_and_end(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);

                let output = blocking(request, url)
                    .keep_partial_body()
                    .send_blocking(request)
                    .unwrap();

                assert_eq!(output.body, b"body");
                assert_eq!(
                    recorder.events(),
                    [
                        format!("started GET {url}"),
                        "body body".to_string(),
                        "finished 200".to_string()
                    ]
                );
            }

            /// `url` answers `503` with the body `unavailable` once, then `200`
            /// with the body `body`.
            pub(crate) fn send_blocking_with_retry_should_retry_until_success(
                request: &impl Request,
                url: &str,
            ) {
                // GIVEN a request retried around a recorder
                let recorder = Recorder::default();
                let builder = blocking(request, url).interceptors(chain(
                    HttpRequest::retry(2, Duration::ZERO),
                    recorder.clone(),
                ));

                // WHEN sending it
                let output = builder.send_blocking(request).unwrap();

                // THEN the second attempt succeeds with its own body only
                assert_eq!(output.status, 200);
                assert_eq!(output.body, b"body");
                assert_eq!(
                    recorder.events(),
                    [
                        format!("started GET {url}"),
                        "finished 503".to_string(),
                        format!("started GET {url}"),
                        "finished 200".to_string()
                    ]
                );
            }

            /// `url` answers `503` once, then `200`.
            pub(crate) fn send_blocking_with_retry_should_send_streamed_body_by_each_attempt(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                request
                    .blocking_client()
                    .unwrap()
                    .post(url)
                    .body(reqwest::blocking::Body::new(std::io::Cursor::new(b"data")))
                    .interceptors(own(HttpRequest::retry(1, Duration::ZERO)))
                    .send_blocking(request)
                    .unwrap();

                let bodies: Vec<_> = sent().into_iter().map(|sent| sent.body).collect();
                assert_eq!(bodies, [b"data", b"data"]);
            }

            /// `url` answers `200` with a `content-length` header but closes
            /// the connection after the body `partial`.
            pub(crate) fn send_blocking_with_retry_interrupted_body_should_be_retried(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let result = blocking(request, url)
                    .interceptors(own(HttpRequest::retry(1, Duration::ZERO)))
                    .send_blocking(request);

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Other));
                assert_eq!(sent().len(), 2);
            }
        }

        pub(crate) mod send_blocking_with_sink {
            use super::*;

            /// `url` answers `404` with the body `missing`.
            pub(crate) fn send_blocking_with_sink_failed_status_should_return_body_in_output(
                request: &impl Request,
                url: &str,
            ) {
                let mut received = Vec::new();

                let error = blocking(request, url)
                    .send_blocking_with_sink(request, &mut collect_into(&mut received))
                    .unwrap_err();

                assert_eq!(error.request_output().unwrap().body, b"missing");
                assert!(received.is_empty());
            }

            /// `url` answers `200` with a `content-length` header but closes
            /// the connection after the body `partial`.
            pub(crate) fn send_blocking_with_sink_interrupted_body_should_keep_received_part(
                request: &impl Request,
                url: &str,
            ) {
                let mut received = Vec::new();

                let result = blocking(request, url)
                    .keep_partial_body()
                    .send_blocking_with_sink(request, &mut collect_into(&mut received));

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Other));
                assert_eq!(received, b"partial");
                let output = interrupted_output(&result);
                assert_eq!(output.status, 200);
                assert!(output.headers.contains_key("content-length"));
                assert!(output.body.is_empty());
            }

            /// `url` answers `200` with a `content-length` header but closes
            /// the connection after the body `partial`.
            pub(crate) fn send_blocking_with_sink_interrupted_body_should_not_be_retried(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let result = blocking(request, url)
                    .interceptors(own(crate::interceptor::Retry::new(2, Duration::ZERO)))
                    .send_blocking_with_sink(request, &mut collect_into(&mut Vec::new()));

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Other));
                assert_eq!(sent().len(), 1);
            }

            /// `url` answers `503` with the body `unavailable` once, then `200`
            /// with the body `body`.
            pub(crate) fn send_blocking_with_sink_failure_before_body_should_be_retried(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let mut received = Vec::new();

                let output = blocking(request, url)
                    .interceptors(own(HttpRequest::retry(1, Duration::ZERO)))
                    .send_blocking_with_sink(request, &mut collect_into(&mut received))
                    .unwrap();

                assert_eq!(output.status, 200);
                assert!(output.body.is_empty());
                assert_eq!(received, b"body");
                assert_eq!(sent().len(), 2);
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_blocking_with_sink_failing_sink_should_return_its_error_without_retrying(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let error = blocking(request, url)
                    .interceptors(own(HttpRequest::retry(2, Duration::ZERO).when(|_, _| true)))
                    .send_blocking_with_sink(request, &mut |(), _: &[u8]| {
                        Err(crate::Error::fs("disk full"))
                    })
                    .unwrap_err();

                assert_eq!(error.description(), "disk full");
                assert_eq!(sent().len(), 1);
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_blocking_with_sink_should_pass_head_then_body_to_sink(
                request: &impl Request,
                url: &str,
            ) {
                let mut sink = Heads::default();

                let output = blocking(request, url)
                    .send_blocking_with_sink(request, &mut sink)
                    .unwrap();

                assert_eq!(output.status, 200);
                assert!(output.body.is_empty());
                assert_eq!(sink.events.concat(), "head 200body");
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_blocking_with_sink_refused_head_should_return_its_error_without_retrying(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let mut sink = Heads {
                    refuse: true,
                    ..Default::default()
                };

                let error = blocking(request, url)
                    .interceptors(own(HttpRequest::retry(2, Duration::ZERO).when(|_, _| true)))
                    .send_blocking_with_sink(request, &mut sink)
                    .unwrap_err();

                assert_eq!(error.description(), "head refused");
                assert_eq!(sent().len(), 1);
            }
        }

        #[cfg(feature = "tokio")]
        pub(crate) mod client {
            use super::*;

            pub(crate) fn client_invalid_builder_should_return_error_every_time(
                request: &impl Request,
            ) {
                request.set_client(invalid_async_client());

                let first = request.client().unwrap_err();
                let second = request.client().unwrap_err();

                assert!(
                    first
                        .description()
                        .starts_with("client could not be built: ")
                );
                assert_eq!(first.description(), second.description());
            }
        }

        #[cfg(feature = "tokio")]
        pub(crate) mod client_builder {
            use super::*;

            /// `env` is the env of the `request`.
            pub(crate) fn client_builder_should_use_proxies_of_env(
                request: &impl Request,
                env: &crate::env::EnvImpl,
            ) {
                crate::Env::set_var(env, "HTTP_PROXY", "http://proxy.invalid:3128").unwrap();

                let builder = request.client_builder();

                assert!(
                    format!("{builder:?}").contains("proxy.invalid"),
                    "{builder:?}"
                );
            }

            pub(crate) fn client_builder_should_time_out_connecting_after_30_seconds_only(
                request: &impl Request,
            ) {
                let debug = format!("{:?}", request.client_builder());

                assert!(debug.contains("connect_timeout: 30s"), "{debug}");
                assert!(!debug.contains(", timeout: "), "{debug}");
            }
        }

        #[cfg(feature = "tokio")]
        pub(crate) mod set_client {
            use super::*;

            pub(crate) fn set_client_should_replace_default_client(request: &impl Request) {
                request.client().unwrap();

                request.set_client(invalid_async_client());

                assert!(request.client().is_err());
            }
        }

        #[cfg(feature = "tokio")]
        pub(crate) mod send_async {
            use super::*;

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_async_should_return_response_and_pass_run_to_interceptors(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);
                let builder = not_blocking(request, &format!("{url}?token=secret"));

                let output = block_on(builder.send_async(request)).unwrap();

                assert!(output.success);
                assert_eq!(output.status, 200);
                assert_eq!(output.body, b"body");
                assert_eq!(
                    recorder.events(),
                    [format!("started GET {url}"), "finished 200".to_string()]
                );
            }

            /// `url` answers `404` with the body `missing`.
            pub(crate) fn send_async_failed_status_should_return_error_with_output(
                request: &impl Request,
                url: &str,
            ) {
                let error = block_on(not_blocking(request, url).send_async(request)).unwrap_err();

                assert_eq!(
                    error.description(),
                    format!("'GET {url}' failed with status 404")
                );
                let output = error.request_output().unwrap();
                assert_eq!(output.status, 404);
                assert_eq!(output.body, b"missing");
            }

            pub(crate) fn send_async_invalid_request_should_return_error_bypassing_interceptors(
                request: &impl Request,
            ) {
                let recorder = recorded(request);

                let error =
                    block_on(not_blocking(request, "not a url").send_async(request)).unwrap_err();

                assert_eq!(error.description(), "invalid request: builder error");
                assert!(recorder.events().is_empty());
            }

            pub(crate) fn send_async_unreadable_body_should_return_error_bypassing_interceptors(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);
                let builder = request
                    .client()
                    .unwrap()
                    .post(url)
                    .body(reqwest::Body::wrap(FailingBody));

                let error = block_on(builder.send_async(request)).unwrap_err();

                assert!(
                    error
                        .description()
                        .starts_with("request body could not be read: "),
                    "{error}"
                );
                assert!(recorder.events().is_empty());
            }

            /// `url` refuses the connection.
            pub(crate) fn send_async_connect_failure_should_return_connect_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                let result = block_on(not_blocking(request, url).send_async(request));

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Connect));
                assert!(
                    result
                        .unwrap_err()
                        .description()
                        .starts_with(&format!("'GET {url}' failed to connect"))
                );
            }

            /// `url` answers after more than 50 ms.
            pub(crate) fn send_async_timeout_should_return_timeout_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                let builder = not_blocking(request, url).timeout(Duration::from_millis(50));

                let result = block_on(builder.send_async(request));

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Timeout));
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_async_zero_timeout_should_return_timeout_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                let builder = not_blocking(request, url).timeout(Duration::ZERO);

                let result = block_on(builder.send_async(request));

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Timeout));
            }

            /// `url` answers `200` with a `content-length` header but stops
            /// sending its body for more than 300 ms after `partial`.
            pub(crate) fn send_async_stalled_body_should_return_timeout_transport_error_with_head(
                request: &impl Request,
                url: &str,
            ) {
                let builder = not_blocking(request, url)
                    .timeout(Duration::from_millis(300))
                    .keep_partial_body();

                let result = block_on(builder.send_async(request));

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Timeout));
                let output = interrupted_output(&result);
                assert_eq!(output.status, 200);
                assert!(output.headers.contains_key("content-length"));
                assert_eq!(output.body, b"partial");
            }

            /// `url` answers `200` with a `content-length` header but closes
            /// the connection after the body `partial`.
            pub(crate) fn send_async_truncated_body_should_return_other_transport_error(
                request: &impl Request,
                url: &str,
            ) {
                for keep in [false, true] {
                    let builder = not_blocking(request, url);
                    let builder = if keep {
                        builder.keep_partial_body()
                    } else {
                        ConfiguredRequest::new(builder)
                    };

                    let result = block_on(builder.send_async(request));

                    assert_eq!(
                        transport_kind(&result),
                        Some(TransportErrorKind::Other),
                        "keep: {keep}"
                    );
                    let partial: &[u8] = if keep { b"partial" } else { b"" };
                    assert_eq!(interrupted_output(&result).body, partial, "keep: {keep}");
                }
            }

            /// `url` answers `503` with the body `unavailable` once, then `200`
            /// with the body `body`.
            pub(crate) fn send_async_with_retry_should_retry_until_success(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let builder = not_blocking(request, url)
                    .interceptors(own(HttpRequest::retry(1, Duration::ZERO)));

                let output = block_on(builder.send_async(request)).unwrap();

                assert_eq!(output.status, 200);
                assert_eq!(output.body, b"body");
                assert_eq!(sent().len(), 2);
            }

            /// `url` answers `503` once, then `200`.
            pub(crate) fn send_async_with_retry_should_send_streamed_body_by_each_attempt(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let builder = request
                    .client()
                    .unwrap()
                    .post(url)
                    .body(streamed(b"data"))
                    .interceptors(own(HttpRequest::retry(1, Duration::ZERO)));

                block_on(builder.send_async(request)).unwrap();

                let bodies: Vec<_> = sent().into_iter().map(|sent| sent.body).collect();
                assert_eq!(bodies, [b"data", b"data"]);
            }

            /// `url` answers after more than 100 ms.
            pub(crate) fn send_async_cancelled_should_tell_interceptors(
                request: &impl Request,
                url: &str,
            ) {
                // GIVEN a request answered late
                let recorder = recorded(request);
                let builder = not_blocking(request, url);

                // WHEN it is cancelled by a timeout
                let result = block_on(async {
                    tokio::time::timeout(Duration::from_millis(100), builder.send_async(request))
                        .await
                });

                // THEN the interceptors are told
                assert!(result.is_err());
                assert_eq!(
                    recorder.events(),
                    [format!("started GET {url}"), format!("cancelled GET {url}")]
                );
            }
        }

        #[cfg(feature = "tokio")]
        pub(crate) mod send_async_with_sink {
            use super::*;

            /// `url` answers `404` with the body `missing`.
            pub(crate) fn send_async_with_sink_failed_status_should_return_body_in_output(
                request: &impl Request,
                url: &str,
            ) {
                let mut received = Vec::new();
                let builder = not_blocking(request, url);

                let error = block_on(
                    builder.send_async_with_sink(request, &mut collect_into(&mut received)),
                )
                .unwrap_err();

                assert_eq!(error.request_output().unwrap().body, b"missing");
                assert!(received.is_empty());
            }

            /// `url` answers `200` with a `content-length` header but closes
            /// the connection after the body `partial`.
            pub(crate) fn send_async_with_sink_interrupted_body_should_keep_received_part(
                request: &impl Request,
                url: &str,
            ) {
                let mut received = Vec::new();
                let builder = not_blocking(request, url).keep_partial_body();

                let result = block_on(
                    builder.send_async_with_sink(request, &mut collect_into(&mut received)),
                );

                assert_eq!(transport_kind(&result), Some(TransportErrorKind::Other));
                assert_eq!(received, b"partial");
                let output = interrupted_output(&result);
                assert_eq!(output.status, 200);
                assert!(output.headers.contains_key("content-length"));
                assert!(output.body.is_empty());
            }

            /// `url` answers `503` with the body `unavailable` once, then `200`
            /// with the body `body`.
            pub(crate) fn send_async_with_sink_failure_before_body_should_be_retried(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let mut received = Vec::new();
                let builder = not_blocking(request, url)
                    .interceptors(own(HttpRequest::retry(1, Duration::ZERO)));

                let output = block_on(
                    builder.send_async_with_sink(request, &mut collect_into(&mut received)),
                )
                .unwrap();

                assert_eq!(output.status, 200);
                assert!(output.body.is_empty());
                assert_eq!(received, b"body");
                assert_eq!(sent().len(), 2);
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_async_with_sink_failing_sink_should_return_its_error(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let builder = not_blocking(request, url)
                    .interceptors(own(HttpRequest::retry(2, Duration::ZERO)));

                let error = block_on(builder.send_async_with_sink(request, &mut |(), _: &[u8]| {
                    Err(crate::Error::fs("disk full"))
                }))
                .unwrap_err();

                assert_eq!(error.description(), "disk full");
                assert_eq!(sent().len(), 1);
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_async_with_sink_should_pass_head_then_body_to_sink(
                request: &impl Request,
                url: &str,
            ) {
                let mut sink = Heads::default();
                let builder = not_blocking(request, url);

                let output = block_on(builder.send_async_with_sink(request, &mut sink)).unwrap();

                assert_eq!(output.status, 200);
                assert!(output.body.is_empty());
                assert_eq!(sink.events.concat(), "head 200body");
            }

            /// `url` answers `200` with the body `body`.
            pub(crate) fn send_async_with_sink_refused_head_should_return_its_error_without_retrying(
                request: &impl Request,
                url: &str,
                sent: &dyn Fn() -> Vec<Sent>,
            ) {
                let mut sink = Heads {
                    refuse: true,
                    ..Default::default()
                };
                let builder = not_blocking(request, url).interceptors(own(HttpRequest::retry(
                    2,
                    Duration::ZERO,
                )
                .when(|_, _| true)));

                let error = block_on(builder.send_async_with_sink(request, &mut sink)).unwrap_err();

                assert_eq!(error.description(), "head refused");
                assert_eq!(sent().len(), 1);
            }
        }

        pub(crate) mod interceptors {
            use super::*;

            pub(crate) fn interceptors_should_be_none_by_default(request: &impl Request) {
                assert!(request.interceptors().is_empty());
            }
        }

        pub(crate) mod clear_interceptors {
            use super::*;

            /// `url` answers `200`.
            pub(crate) fn clear_interceptors_should_remove_interceptors_of_later_runs(
                request: &impl Request,
                url: &str,
            ) {
                let recorder = recorded(request);

                request.clear_interceptors();
                blocking(request, url).send_blocking(request).unwrap();

                assert!(request.interceptors().is_empty());
                assert!(recorder.events().is_empty());
            }
        }

        pub(crate) mod add_interceptor {
            use super::*;

            /// `url` answers `200`.
            pub(crate) fn add_interceptor_should_add_innermost_interceptor(
                request: &impl Request,
                url: &str,
            ) {
                // GIVEN an interceptor
                let outer = recorded(request);

                // WHEN adding one changing the path and another inside it
                request.add_interceptor(ChangePath);
                let inner = recorded(request);
                blocking(request, url).send_blocking(request).unwrap();

                // THEN the run passes the outer one first
                assert_eq!(request.interceptors().len(), 3);
                assert_eq!(outer.events()[0], format!("started GET {url}"));
                assert_eq!(inner.events()[0], format!("started GET {url}/inner"));
            }

            /// Appends `/inner` to the path of the runs.
            struct ChangePath;

            impl Interceptor<HttpRequest> for ChangePath {
                fn handle<'a>(
                    &'a self,
                    mut next: Next<'a, HttpRequest>,
                ) -> BoxFuture<'a, Result<RequestOutput>> {
                    let url = next.run_mut().subject_mut().url_mut();
                    let path = format!("{}/inner", url.path());
                    url.set_path(&path);
                    next.forward()
                }
            }
        }
    }
}
