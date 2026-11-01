use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;

use reqwest::header::HeaderMap;

use crate::ConfiguredRequest;
use crate::HttpRequest;
use crate::Request;
use crate::RequestOptions;
use crate::RequestOutput;
use crate::Result;
use crate::TransportError;
use crate::TransportErrorKind;
use crate::env::EnvImpl;
#[cfg(feature = "tokio")]
use crate::interceptor::AsyncExecutor;
use crate::interceptor::BlockingExecutor;
use crate::interceptor::BoxFuture;
use crate::interceptor::Emit;
use crate::interceptor::Executor;
use crate::interceptor::Interceptor;
use crate::interceptor::Interceptors;
use crate::interceptor::Run;
use crate::interceptor::Sink;
use crate::request::DefaultClients;
use crate::request::Received;
use crate::request::run_blocking;
use crate::request::transport_failure;
use crate::test_platform::gate::Gate;
use crate::test_platform::gate::HeldGate;
#[cfg(feature = "tokio")]
use crate::test_platform::gate::YieldOnce;
use crate::utils::Runs;

/// Enabled by the `testing` and `request` feature flags.
///
/// A memory-only [`crate::Request`] implementation for tests. No network
/// is ever accessed. Requests are matched by their method and URL separated by a space (e.g.
/// `GET https://example.com/path?query`) containing the patterns set with
/// [`TestRequest::set_response`] and [`TestRequest::set_transport_error`],
/// or by the predicates set with [`TestRequest::set_response_if`].
///
/// The requests pass through the interceptors like with any [`Request`], and
/// are recorded when they would be sent, i.e. not when an interceptor sets
/// their output, e.g. [`DryRun`](crate::interceptor::DryRun). Delays,
/// e.g. of [`HttpRequest::retry`], are not waited for but
/// recorded, see [`TestRequest::sleeps`]. An async one is pending once, so
/// that a run can be cancelled while it waits.
///
/// The clients are real [`reqwest`] clients, only built and never used to
/// send anything. Their proxies come from the test environment (see
/// [`crate::Request`]), not from the process environment.
///
/// # Examples
///
/// ```rust
/// use agplatform::{BlockingRequestBuilderExt, Request, RequestOutput, TestRequest};
///
/// let request = TestRequest::new();
/// request.set_response("/status", RequestOutput { status: 200, body: b"ok".to_vec(), ..Default::default() });
///
/// let output = request.blocking_client().unwrap().get("https://example.com/status").send_blocking(&request).unwrap();
///
/// assert_eq!(output.text().unwrap(), "ok");
/// assert_eq!(request.calls(), ["GET https://example.com/status"]);
/// ```
pub struct TestRequest {
    data: Mutex<TestRequestData>,
    pub(crate) clients: DefaultClients,
    runs: Runs<HttpRequest>,
}

/// Enabled by the `testing` and `request` feature flags.
///
/// A request sent through [`TestRequest`], see
/// [`TestRequest::call_details`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRequestCall {
    /// The method, e.g. `GET`.
    pub method: String,

    /// The whole URL.
    pub url: String,

    /// The request headers.
    pub headers: HeaderMap,

    /// The request body, read into memory, empty when it has none.
    pub body: Vec<u8>,

    /// The options of the run.
    pub options: RequestOptions,
}

/// Enabled by the `testing` and `request` feature flags.
///
/// Holds the requests answered by [`TestRequest::set_pending`] until it is
/// released, e.g. to send other requests while one is in flight. Clones
/// share the gate, so it can be released from another thread or task.
///
/// Dropping all clones releases it, so that a test failing before it
/// releases the gate does not hang waiting for the held requests, e.g. at
/// the end of [`std::thread::scope`] if it is created in the scope.
#[derive(Clone)]
pub struct TestGate(HeldGate<TransportErrorKind>);

#[derive(Clone)]
struct TestResponse {
    output: RequestOutput,
    chunks: Vec<Vec<u8>>,
    error: Option<TransportErrorKind>,
    gate: Option<Gate<TransportErrorKind>>,
}

type CallPredicate = dyn Fn(&TestRequestCall) -> bool + Send + Sync;

/// Evaluated without holding the lock of [`TestRequestData`], as a
/// predicate may use the [`TestRequest`].
#[derive(Clone)]
enum Matcher {
    Pattern(Arc<str>),
    Predicate(Arc<CallPredicate>),
}

#[derive(Default)]
struct TestRequestData {
    responses: Vec<(Matcher, VecDeque<TestResponse>)>,
    calls: Vec<TestRequestCall>,
    sleeps: Vec<Duration>,
    client_failure: bool,
}

impl TestRequest {
    /// Creates a new test request interface.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::TestRequest;
    ///
    /// let request = TestRequest::new();
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            data: Mutex::new(TestRequestData::default()),
            clients: DefaultClients::new(EnvImpl::new(
                Vec::new(),
                std::path::PathBuf::from(crate::test_platform::ROOT),
            )),
            runs: Runs::new(Interceptors::new()),
        }
    }

    /// Returns the method and the URL of all requests sent so far, oldest
    /// first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// let _ = request.blocking_client().unwrap().delete("https://example.com/item?id=1").send_blocking(&request);
    ///
    /// assert_eq!(request.calls(), ["DELETE https://example.com/item?id=1"]);
    /// ```
    pub fn calls(&self) -> Vec<String> {
        self.data()
            .calls
            .iter()
            .map(TestRequestCall::line)
            .collect()
    }

    /// Returns all requests sent so far with their headers, body and
    /// options, oldest first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// let client = request.blocking_client().unwrap();
    /// let _ = client.post("https://example.com").bearer_auth("token").body("data").send_blocking(&request);
    ///
    /// let call = &request.call_details()[0];
    /// assert_eq!(call.headers["authorization"], "Bearer token");
    /// assert_eq!(call.body, b"data");
    /// ```
    pub fn call_details(&self) -> Vec<TestRequestCall> {
        self.data().calls.clone()
    }

    /// Forgets the requests sent so far.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// let _ = request.blocking_client().unwrap().get("https://example.com").send_blocking(&request);
    /// request.clear_calls();
    ///
    /// assert!(request.calls().is_empty());
    /// ```
    pub fn clear_calls(&self) {
        self.data().calls.clear();
    }

    /// Returns the delays the interceptors waited for so far, oldest first,
    /// e.g. to check the backoff of [`HttpRequest::retry`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, HttpRequest, Request, RequestOutput, TestRequest};
    /// use std::time::Duration;
    ///
    /// let request = TestRequest::new();
    /// request.add_interceptor(HttpRequest::retry(2, Duration::from_secs(1)).backoff(2));
    /// request.set_response("", RequestOutput { status: 503, ..Default::default() });
    ///
    /// let _ = request.blocking_client().unwrap().get("https://example.com").send_blocking(&request);
    ///
    /// assert_eq!(request.sleeps(), [Duration::from_secs(1), Duration::from_secs(2)]);
    /// ```
    pub fn sleeps(&self) -> Vec<Duration> {
        self.data().sleeps.clone()
    }

    /// Makes building the default clients fail while `fail` is `true`, see
    /// [`crate::Request::client`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Request, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// request.set_client_failure(true);
    ///
    /// assert!(request.blocking_client().is_err());
    /// ```
    pub fn set_client_failure(&self, fail: bool) {
        self.data().client_failure = fail;
    }

    /// Sets the `output` of the requests whose method and URL contain
    /// `pattern`. When more patterns or predicates match, the most
    /// recently set one wins, so an empty pattern set first acts as a
    /// default.
    ///
    /// A [`RequestOutput::duration`] reaching the timeout of a request
    /// makes it time out, without waiting. An output whose status is not a
    /// success makes the request fail with the output available from
    /// [`Error::request_output`](crate::Error::request_output).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, RequestOutput, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// request.set_response("", RequestOutput { status: 200, ..Default::default() });
    /// request.set_response("POST https://example.com/", RequestOutput { status: 500, ..Default::default() });
    ///
    /// let client = request.blocking_client().unwrap();
    /// assert!(client.get("https://example.com").send_blocking(&request).is_ok());
    /// assert!(client.post("https://example.com").send_blocking(&request).is_err());
    /// ```
    pub fn set_response<T: Into<String>>(&self, pattern: T, output: RequestOutput) {
        self.respond_with(
            Matcher::Pattern(pattern.into().into()),
            VecDeque::from([TestResponse::new(output)]),
        );
    }

    /// Sets the `outputs` of the requests whose method and URL contain
    /// `pattern`, returned one per request in order with the last one
    /// repeated, e.g. to make a request fail before it succeeds on a
    /// retry. Patterns match as in [`Self::set_response`]. No `outputs`
    /// set nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, HttpRequest, Request, RequestOutput, TestRequest};
    /// use std::time::Duration;
    ///
    /// let request = TestRequest::new();
    /// let unavailable = RequestOutput { status: 503, ..Default::default() };
    /// let ok = RequestOutput { status: 200, ..Default::default() };
    /// request.set_responses("example.com", [unavailable, ok]);
    ///
    /// let client = request.blocking_client().unwrap();
    /// request.add_interceptor(HttpRequest::retry(3, Duration::from_secs(1)));
    /// let output = client.get("https://example.com").send_blocking(&request).unwrap();
    ///
    /// assert_eq!(output.status, 200);
    /// assert_eq!(request.calls().len(), 2);
    /// ```
    pub fn set_responses<T: Into<String>, I: IntoIterator<Item = RequestOutput>>(
        &self,
        pattern: T,
        outputs: I,
    ) {
        self.respond_with(
            Matcher::Pattern(pattern.into().into()),
            outputs.into_iter().map(TestResponse::new).collect(),
        );
    }

    /// Sets the `output` of the requests for which `predicate` returns
    /// `true`, e.g. to match their body. Matches as in
    /// [`Self::set_response`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, RequestOutput, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// request.set_response_if(
    ///     |call| call.body.starts_with(b"query"),
    ///     RequestOutput { status: 200, body: b"result".to_vec(), ..Default::default() },
    /// );
    ///
    /// let client = request.blocking_client().unwrap();
    /// let output = client.post("https://example.com/graphql").body("query").send_blocking(&request).unwrap();
    ///
    /// assert_eq!(output.text().unwrap(), "result");
    /// ```
    pub fn set_response_if<F>(&self, predicate: F, output: RequestOutput)
    where
        F: Fn(&TestRequestCall) -> bool + Send + Sync + 'static,
    {
        self.respond_with(
            Matcher::Predicate(Arc::new(predicate)),
            VecDeque::from([TestResponse::new(output)]),
        );
    }

    /// Sets the `output` of the requests whose method and URL contain
    /// `pattern` with its body arriving in `chunks`, e.g. to test a sink
    /// (see [`crate::Request::send_blocking_with_sink`]) or an interceptor of the
    /// chunks. The body of the `output` is replaced by the chunks.
    /// Patterns match as in [`Self::set_response`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, RequestOutput, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// request.set_chunks("", RequestOutput { status: 200, ..Default::default() }, [b"ab".to_vec(), b"c".to_vec()]);
    /// let mut chunks = Vec::new();
    ///
    /// request
    ///     .blocking_client()
    ///     .unwrap()
    ///     .get("https://example.com/download")
    ///     .send_blocking_with_sink(&request, &mut |(), chunk: &[u8]| {
    ///         chunks.push(chunk.to_vec());
    ///         Ok(())
    ///     })
    ///     .unwrap();
    ///
    /// assert_eq!(chunks, [b"ab".to_vec(), b"c".to_vec()]);
    /// ```
    pub fn set_chunks<T: Into<String>, I: IntoIterator<Item = Vec<u8>>>(
        &self,
        pattern: T,
        output: RequestOutput,
        chunks: I,
    ) {
        self.respond_with(
            Matcher::Pattern(pattern.into().into()),
            VecDeque::from([TestResponse {
                chunks: chunks.into_iter().collect(),
                ..TestResponse::new(output)
            }]),
        );
    }

    /// Makes the requests whose method and URL contain `pattern` receive
    /// the `chunks` of the body of `output` and then fail with a
    /// [`TransportError`] of `kind`, e.g. for a connection lost in the
    /// middle of a download. The `output` is available from
    /// [`TransportError::output`] with the part of the body received like
    /// [`crate::Request`] does. Patterns match as in [`Self::set_response`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, RequestOutput, TestRequest, TransportErrorKind};
    ///
    /// let request = TestRequest::new();
    /// let output = RequestOutput { status: 200, ..Default::default() };
    /// request.set_interrupted("", output, [b"first half".to_vec()], TransportErrorKind::Other);
    /// let mut file = Vec::new();
    ///
    /// let result = request
    ///     .blocking_client()
    ///     .unwrap()
    ///     .get("https://example.com/download")
    ///     .send_blocking_with_sink(&request, &mut |(), chunk: &[u8]| {
    ///         file.extend_from_slice(chunk);
    ///         Ok(())
    ///     });
    ///
    /// assert!(result.is_err());
    /// assert_eq!(file, b"first half");
    /// ```
    pub fn set_interrupted<T: Into<String>, I: IntoIterator<Item = Vec<u8>>>(
        &self,
        pattern: T,
        output: RequestOutput,
        chunks: I,
        kind: TransportErrorKind,
    ) {
        self.respond_with(
            Matcher::Pattern(pattern.into().into()),
            VecDeque::from([TestResponse {
                chunks: chunks.into_iter().collect(),
                error: Some(kind),
                ..TestResponse::new(output)
            }]),
        );
    }

    /// Sets the `output` of the requests whose method and URL contain
    /// `pattern` like [`Self::set_response`], held until the returned gate
    /// is released or failed. The requests are recorded when they are
    /// sent, before being held.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, RequestOutput, TestRequest};
    ///
    /// let request = TestRequest::new();
    /// let client = request.blocking_client().unwrap();
    ///
    /// std::thread::scope(|scope| {
    ///     let gate = request.set_pending("", RequestOutput { status: 200, ..Default::default() });
    ///     let pending = scope.spawn(|| client.get("https://example.com").send_blocking(&request));
    ///
    ///     while request.calls().is_empty() {
    ///         std::thread::yield_now();
    ///     }
    ///
    ///     assert!(!pending.is_finished());
    ///     gate.release();
    ///     assert!(pending.join().unwrap().is_ok());
    /// });
    /// ```
    pub fn set_pending<T: Into<String>>(&self, pattern: T, output: RequestOutput) -> TestGate {
        let gate = TestGate::new();
        self.respond_with(
            Matcher::Pattern(pattern.into().into()),
            VecDeque::from([TestResponse {
                gate: Some(gate.0.gate().clone()),
                ..TestResponse::new(output)
            }]),
        );
        gate
    }

    /// Makes the requests whose method and URL contain `pattern` fail with
    /// a [`TransportError`] of `kind`, e.g.
    /// [`Connect`](TransportErrorKind::Connect) for a server that is down.
    /// Patterns match as in [`Self::set_response`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, TestRequest, TransportError, TransportErrorKind};
    ///
    /// let request = TestRequest::new();
    /// request.set_transport_error("example.com", TransportErrorKind::Timeout);
    ///
    /// let error = request.blocking_client().unwrap().get("https://example.com").send_blocking(&request).unwrap_err();
    ///
    /// assert_eq!(error.source_as::<TransportError>().unwrap().kind(), TransportErrorKind::Timeout);
    /// assert!(error.request_output().is_none());
    /// ```
    pub fn set_transport_error<T: Into<String>>(&self, pattern: T, kind: TransportErrorKind) {
        self.respond_with(
            Matcher::Pattern(pattern.into().into()),
            VecDeque::from([TestResponse {
                gate: Some(failed_gate(kind)),
                ..TestResponse::new(RequestOutput::default())
            }]),
        );
    }

    fn respond_with(&self, matcher: Matcher, responses: VecDeque<TestResponse>) {
        if !responses.is_empty() {
            self.data().responses.push((matcher, responses));
        }
    }

    fn dispatch_blocking(
        &self,
        request: ConfiguredRequest<reqwest::blocking::RequestBuilder>,
        sink: Option<&mut dyn Sink<HttpRequest>>,
    ) -> Result<RequestOutput> {
        #[cfg(feature = "tokio")]
        assert!(
            tokio::runtime::Handle::try_current().is_err(),
            "TestRequest: a blocking request must not be sent in an async context, \
             as it would block its thread, use send_async instead"
        );

        let (_, run) = request.into_run();
        run_blocking(&self.runs, self, run?, sink)
    }

    #[cfg(feature = "tokio")]
    async fn dispatch_async(
        &self,
        request: ConfiguredRequest<reqwest::RequestBuilder>,
        sink: Option<&mut dyn Sink<HttpRequest>>,
    ) -> Result<RequestOutput> {
        let (_, run) = request.into_run().await;
        crate::request::run_async(&self.runs, self, run?, sink).await
    }

    /// Records the call of `request` and returns the response set for it,
    /// the most recent one matching.
    fn select(&self, request: &HttpRequest) -> Result<TestResponse> {
        let call = TestRequestCall {
            method: request.method().to_string(),
            url: request.url().to_string(),
            headers: request.headers().clone(),
            body: request.body().unwrap_or_default().to_vec(),
            options: request.options().clone(),
        };
        let line = call.line();
        let matchers: Vec<Matcher> = self
            .data()
            .responses
            .iter()
            .map(|(matcher, _)| matcher.clone())
            .collect();
        let matched = matchers.iter().rposition(|matcher| match matcher {
            Matcher::Pattern(pattern) => line.contains(&**pattern),
            Matcher::Predicate(predicate) => predicate(&call),
        });
        let mut data = self.data();
        let response = matched.and_then(|index| {
            let responses = &mut data.responses[index].1;

            if responses.len() > 1 {
                responses.pop_front()
            } else {
                responses.front().cloned()
            }
        });
        data.calls.push(call);
        drop(data);

        response.ok_or_else(|| request_error!("'{}': no response set", request.target()))
    }

    /// [internal]
    ///
    /// Takes the variables reqwest reads from `env`, e.g. of a
    /// [`TestPlatform`](crate::test_platform::TestPlatform), replacing the
    /// default clients built.
    pub(crate) fn set_env(&mut self, env: EnvImpl) {
        self.clients.set_env(env);
    }

    fn fail_clients(&self) -> Result<()> {
        if self.data().client_failure {
            Err(request_error!(
                "client could not be built: simulated failure"
            ))
        } else {
            Ok(())
        }
    }

    fn data(&self) -> std::sync::MutexGuard<'_, TestRequestData> {
        self.data.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl TestRequestCall {
    fn line(&self) -> String {
        format!("{} {}", self.method, self.url)
    }
}

/// Returns a gate failing with `kind`, e.g. for a request failing before
/// its response arrives.
fn failed_gate(kind: TransportErrorKind) -> Gate<TransportErrorKind> {
    let gate = Gate::new();
    gate.open(Some(kind));
    gate
}

/// Waits until `gate` is opened, returning its failure, without blocking an
/// async run.
#[cfg(feature = "tokio")]
async fn wait(gate: &Gate<TransportErrorKind>) -> Option<TransportErrorKind> {
    gate.wait_async().await
}

/// Blocks until `gate` is opened, returning its failure.
#[cfg(not(feature = "tokio"))]
async fn wait(gate: &Gate<TransportErrorKind>) -> Option<TransportErrorKind> {
    gate.wait()
}

impl TestGate {
    fn new() -> Self {
        Self(HeldGate::new())
    }

    /// Lets the held requests receive their response, and the later ones
    /// at once. Does nothing if the gate was released or failed already.
    pub fn release(&self) {
        self.0.gate().open(None);
    }

    /// Makes the held requests fail with a [`TransportError`] of `kind`,
    /// e.g. for a connection that hangs and then drops, and the later ones
    /// at once. Does nothing if the gate was released or failed already.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{BlockingRequestBuilderExt, Request, RequestOutput, TestRequest, TransportErrorKind};
    ///
    /// let request = TestRequest::new();
    /// let gate = request.set_pending("", RequestOutput { status: 200, ..Default::default() });
    /// gate.fail(TransportErrorKind::Timeout);
    ///
    /// let result = request.blocking_client().unwrap().get("https://example.com").send_blocking(&request);
    ///
    /// assert!(result.is_err());
    /// ```
    pub fn fail(&self, kind: TransportErrorKind) {
        self.0.gate().open(Some(kind));
    }
}

/// Answers the runs of blocking and async requests alike, see
/// [`TestRequest`].
impl Executor for TestRequest {
    type Subject = HttpRequest;

    /// Returns the response set for the request, its body passed on in its
    /// chunks, unless its gate fails it first or its duration reaches the
    /// timeout of the request.
    fn execute<'a>(
        &'a self,
        run: &'a Run<HttpRequest>,
        emit: Emit<'a, HttpRequest>,
    ) -> BoxFuture<'a, Result<RequestOutput>> {
        Box::pin(async move {
            let request = run.subject();
            let response = self.select(request)?;
            let failure = match &response.gate {
                Some(gate) => wait(gate).await,
                None => None,
            };
            let simulated = |kind| transport_failure(request, TransportError::new(kind));

            if let Some(kind) = failure {
                return Err(simulated(kind));
            }

            let output = response.output;

            if request
                .timeout()
                .is_some_and(|timeout| output.duration >= timeout)
            {
                return Err(simulated(TransportErrorKind::Timeout));
            }

            let mut received = Received::start(run, emit, output.status, output.headers).await?;

            for chunk in &response.chunks {
                received.push(chunk).await?;
            }

            match response.error {
                Some(kind) => Err(received.interrupted(output.duration, TransportError::new(kind))),
                None => Ok(received.end(output.duration)),
            }
        })
    }

    /// Delays are recorded instead of waited for, an async one is pending
    /// once.
    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
        self.data().sleeps.push(delay);

        #[cfg(feature = "tokio")]
        return Box::pin(YieldOnce(false));

        #[cfg(not(feature = "tokio"))]
        Box::pin(std::future::ready(()))
    }
}

impl BlockingExecutor for TestRequest {}

#[cfg(feature = "tokio")]
impl AsyncExecutor for TestRequest {}

impl TestResponse {
    /// Returns the response of `output` with its body in one chunk, none
    /// when it is empty.
    fn new(mut output: RequestOutput) -> Self {
        let body = std::mem::take(&mut output.body);

        Self {
            output,
            chunks: if body.is_empty() { vec![] } else { vec![body] },
            error: None,
            gate: None,
        }
    }
}

impl Request for TestRequest {
    /// Returns the default blocking client like
    /// [`crate::Request::blocking_client`], built only.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be built or
    /// [`TestRequest::set_client_failure`] is set.
    fn blocking_client(&self) -> Result<reqwest::blocking::Client> {
        self.fail_clients()?;
        self.clients.blocking_client()
    }

    /// Returns a blocking client builder with the proxies of the test
    /// environment like [`crate::Request::blocking_client_builder`].
    fn blocking_client_builder(&self) -> reqwest::blocking::ClientBuilder {
        self.clients.blocking_client_builder()
    }

    /// Sets the builder of the default blocking client.
    ///
    /// See [`crate::Request::set_blocking_client`].
    fn set_blocking_client(&self, builder: reqwest::blocking::ClientBuilder) {
        self.clients.set_blocking_client(builder);
    }

    /// Records the request and returns the response set for it, its body
    /// passed to the interceptors. Retries are recorded instead of waited
    /// for, see [`TestRequest::sleeps`], the gates of
    /// [`TestRequest::set_pending`] are waited for.
    ///
    /// See [`crate::Request::send_blocking`].
    ///
    /// # Errors
    ///
    /// Returns an error if no response was set for the request, a
    /// transport error was set for it, or as [`crate::Request::send_blocking`] for
    /// the request and the response set.
    ///
    /// # Panics
    ///
    /// With the `tokio` feature, panics when called in an async context,
    /// where the blocking client cannot be used (see
    /// [`crate::Request::blocking_client`]), as does
    /// [`send_blocking_with_sink`](Self::send_blocking_with_sink).
    fn send_blocking(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::blocking::RequestBuilder>>,
    ) -> Result<RequestOutput> {
        self.dispatch_blocking(request.into(), None)
    }

    /// Sends the request like [`Self::send_blocking`], passing the head
    /// and the chunks of the body set to `sink` when its status is a
    /// success.
    ///
    /// See [`crate::Request::send_blocking_with_sink`].
    ///
    /// # Errors
    ///
    /// As [`Self::send_blocking`], or the error returned by `sink`.
    fn send_blocking_with_sink(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::blocking::RequestBuilder>>,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> Result<RequestOutput> {
        self.dispatch_blocking(request.into(), Some(sink))
    }

    /// Returns the default client like [`crate::Request::client`], built
    /// only.
    ///
    /// # Errors
    ///
    /// As [`Self::blocking_client`].
    #[cfg(feature = "tokio")]
    fn client(&self) -> Result<reqwest::Client> {
        self.fail_clients()?;
        self.clients.client()
    }

    /// Returns a client builder with the proxies of the test environment
    /// like [`crate::Request::client_builder`].
    #[cfg(feature = "tokio")]
    fn client_builder(&self) -> reqwest::ClientBuilder {
        self.clients.client_builder()
    }

    /// Sets the builder of the default client.
    ///
    /// See [`crate::Request::set_client`].
    #[cfg(feature = "tokio")]
    fn set_client(&self, builder: reqwest::ClientBuilder) {
        self.clients.set_client(builder);
    }

    /// Sends the request like [`Self::send_blocking`] once awaited,
    /// without blocking on the gates.
    ///
    /// See [`crate::Request::send_async`].
    ///
    /// # Errors
    ///
    /// As [`Self::send_blocking`].
    #[cfg(feature = "tokio")]
    fn send_async(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::RequestBuilder>>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send {
        self.dispatch_async(request.into(), None)
    }

    /// Sends the request like [`Self::send_async`], passing the response
    /// to `sink` like [`Self::send_blocking_with_sink`].
    ///
    /// See [`crate::Request::send_async_with_sink`].
    ///
    /// # Errors
    ///
    /// As [`Self::send_blocking_with_sink`].
    #[cfg(feature = "tokio")]
    fn send_async_with_sink(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::RequestBuilder>>,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send {
        self.dispatch_async(request.into(), Some(sink))
    }

    fn interceptors(&self) -> Interceptors<HttpRequest> {
        self.runs.interceptors()
    }

    fn clear_interceptors(&self) {
        self.runs.clear_interceptors();
    }

    fn add_interceptor(&self, interceptor: impl Interceptor<HttpRequest>) {
        self.runs.add_interceptor(interceptor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Env;
    use crate::request::ReqwestEnv;

    /// Changes of the env the request shares, like by its platform.
    impl TestRequest {
        fn set_var(&self, key: String, value: String) {
            self.clients.env.set_var(key, value).unwrap();
        }

        fn remove_var(&self, key: &str) {
            self.clients.env.remove_var(key);
        }
    }
    use crate::BlockingRequestBuilderExt;
    use crate::ConfiguredRequest;
    #[cfg(feature = "tokio")]
    use crate::RequestBuilderExt;
    use crate::interceptor::Chunk;
    use crate::interceptor::DryRun;
    use crate::interceptor::Next;
    use std::time::Duration;

    const URL: &str = "https://example.com/path";

    fn status(status: u16) -> RequestOutput {
        RequestOutput {
            status,
            ..Default::default()
        }
    }

    fn get(request: &TestRequest, url: &str) -> reqwest::blocking::RequestBuilder {
        request.blocking_client().unwrap().get(url)
    }

    fn statuses(results: impl IntoIterator<Item = Result<RequestOutput>>) -> Vec<u16> {
        results
            .into_iter()
            .map(|result| match result {
                Ok(output) => output.status,
                Err(error) => error.request_output().map_or(0, |output| output.status),
            })
            .collect()
    }

    fn collect_into(received: &mut Vec<Vec<u8>>) -> impl FnMut((), &[u8]) -> Result + Send + '_ {
        |(), data| {
            received.push(data.to_vec());
            Ok(())
        }
    }

    fn chunks() -> [Vec<u8>; 2] {
        [b"ab".to_vec(), b"c".to_vec()]
    }

    #[cfg(feature = "tokio")]
    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(future)
    }

    /// Records the blocking runs it handles, the chunks of the streamed
    /// runs and the cancelled ones.
    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<String>>>);

    impl Recorder {
        fn events(&self) -> Vec<String> {
            self.0.lock().unwrap().clone()
        }

        fn push(&self, event: String) {
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
                    result.as_ref().is_ok_and(|output| output.success)
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

    /// Accepts the responses with the status.
    struct Accept(u16);

    impl Interceptor<HttpRequest> for Accept {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, HttpRequest>,
        ) -> BoxFuture<'a, Result<RequestOutput>> {
            Box::pin(async move {
                next.attempt().await.map(|output| RequestOutput {
                    success: output.success || output.status == self.0,
                    ..output
                })
            })
        }
    }

    /// Returns a test request answering every request with `status` and
    /// `body`.
    fn answering(status: u16, body: &str) -> TestRequest {
        let request = TestRequest::new();
        request.set_response(
            "",
            RequestOutput {
                status,
                body: body.as_bytes().to_vec(),
                ..Default::default()
            },
        );
        request
    }

    /// Returns a test request answering every request with `503` and
    /// `unavailable` once, then with `200` and `body`.
    fn unavailable_once() -> TestRequest {
        let request = TestRequest::new();
        request.set_responses(
            "",
            [
                RequestOutput {
                    body: b"unavailable".to_vec(),
                    ..status(503)
                },
                RequestOutput {
                    body: b"body".to_vec(),
                    ..status(200)
                },
            ],
        );
        request
    }

    /// Returns a test request answering every request with `200`, failing
    /// with `kind` after the body `partial`.
    fn interrupted(kind: TransportErrorKind) -> TestRequest {
        let request = TestRequest::new();
        let mut output = status(200);
        output
            .headers
            .insert("content-length", "100".parse().unwrap());
        request.set_interrupted("", output, [b"partial".to_vec()], kind);
        request
    }

    /// Returns the requests sent by `request`.
    fn sent(request: &TestRequest) -> Vec<crate::request::tests::Sent> {
        request
            .call_details()
            .into_iter()
            .map(|call| crate::request::tests::Sent {
                method: call.method,
                headers: call.headers,
                body: call.body,
            })
            .collect()
    }

    /// Returns a test request whose only interceptor is the returned
    /// recorder.
    fn observed() -> (TestRequest, Recorder) {
        let recorder = Recorder::default();
        let request = TestRequest::new();
        request.add_interceptor(recorder.clone());
        (request, recorder)
    }

    mod test_request {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_have_no_calls() {
                assert!(TestRequest::new().calls().is_empty());
            }
        }

        mod calls {
            use super::*;

            #[test]
            fn calls_should_return_methods_and_urls_in_order() {
                let request = TestRequest::new();
                request.set_response("", status(200));
                let client = request.blocking_client().unwrap();

                client.get(URL).send_blocking(&request).unwrap();
                client
                    .post(format!("{URL}?q=1"))
                    .send_blocking(&request)
                    .unwrap();

                assert_eq!(
                    request.calls(),
                    [format!("GET {URL}"), format!("POST {URL}?q=1")]
                );
            }
        }

        mod call_details {
            use super::*;

            #[test]
            fn call_details_should_return_headers_body_and_options() {
                let request = TestRequest::new();
                request.set_response("", status(200));

                request
                    .blocking_client()
                    .unwrap()
                    .put(URL)
                    .header("x-key", "value")
                    .body("data")
                    .keep_partial_body()
                    .send_blocking(&request)
                    .unwrap();

                let mut headers = HeaderMap::new();
                headers.insert("x-key", "value".parse().unwrap());
                assert_eq!(
                    request.call_details(),
                    [TestRequestCall {
                        method: "PUT".to_string(),
                        url: URL.to_string(),
                        headers,
                        body: b"data".to_vec(),
                        options: RequestOptions {
                            keep_partial_body: true,
                            ..Default::default()
                        },
                    }]
                );
            }
        }

        mod clear_calls {
            use super::*;

            #[test]
            fn clear_calls_should_forget_calls() {
                let request = TestRequest::new();
                let _ = get(&request, URL).send_blocking(&request);

                request.clear_calls();

                assert!(request.calls().is_empty());
            }
        }

        mod sleeps {
            use super::*;

            /// Returns the interceptors retrying twice after 1 s doubled
            /// before `recorder`.
            fn retried(recorder: &Recorder) -> Interceptors<HttpRequest> {
                let mut interceptors = Interceptors::new();
                interceptors.push(HttpRequest::retry(2, Duration::from_secs(1)).backoff(2));
                interceptors.push(recorder.clone());
                interceptors
            }

            #[test]
            fn sleeps_should_return_delays_in_order() {
                let (request, recorder) = observed();
                request.set_response("", status(503));

                let _ = get(&request, URL)
                    .interceptors(retried(&recorder))
                    .send_blocking(&request);

                assert_eq!(
                    request.sleeps(),
                    [Duration::from_secs(1), Duration::from_secs(2)]
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn sleeps_should_return_async_delays_in_order() {
                let (request, recorder) = observed();
                request.set_response("", status(503));
                let builder = request.client().unwrap().get(URL);

                let _ = block_on(
                    builder
                        .interceptors(retried(&recorder))
                        .send_async(&request),
                );

                assert_eq!(
                    request.sleeps(),
                    [Duration::from_secs(1), Duration::from_secs(2)]
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn sleeps_cancelled_should_tell_interceptors_up_to_sleeping_one() {
                // GIVEN an async request waiting to be retried within an outer
                // recorder
                let (request, inner) = observed();
                request.set_response("", status(503));
                let outer = Recorder::default();
                let mut interceptors = retried(&inner);
                interceptors.insert(0, outer.clone());
                let builder = request.client().unwrap().get(URL);
                let mut future = Box::pin(builder.interceptors(interceptors).send_async(&request));
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                assert!(future.as_mut().poll(&mut context).is_pending());

                // WHEN cancelling it
                drop(future);

                // THEN the outer one is told after the delay started
                assert_eq!(request.sleeps(), [Duration::from_secs(1)]);
                assert_eq!(
                    outer.events(),
                    [format!("started GET {URL}"), format!("cancelled GET {URL}")]
                );
                assert_eq!(*inner.events().last().unwrap(), "finished false");
            }
        }

        mod set_client_failure {
            use super::*;

            #[test]
            fn set_client_failure_should_fail_client_building_until_unset() {
                // GIVEN simulated client failures
                let request = TestRequest::new();
                request.set_client_failure(true);

                // WHEN getting the default clients
                let errors = [
                    request.blocking_client().map(|_| ()),
                    #[cfg(feature = "tokio")]
                    request.client().map(|_| ()),
                ];

                // THEN they fail until the failures are unset
                for error in errors {
                    assert_eq!(
                        error.unwrap_err().description(),
                        "client could not be built: simulated failure"
                    );
                }
                request.set_client_failure(false);
                assert!(request.blocking_client().is_ok());
            }
        }

        mod set_response {
            use super::*;

            #[test]
            fn set_response_should_prefer_most_recent_matching_pattern() {
                // GIVEN a specific pattern followed by a more general one
                let request = TestRequest::new();
                request.set_response("POST https://example.com/", status(500));
                request.set_response("example.com", status(200));

                // WHEN sending a request matching both
                let result = request
                    .blocking_client()
                    .unwrap()
                    .post(URL)
                    .send_blocking(&request);

                // THEN the more recent one wins
                assert_eq!(result.unwrap().status, 200);
            }

            #[test]
            fn set_response_duration_exceeding_timeout_should_time_out() {
                let request = TestRequest::new();
                request.set_response(
                    "",
                    RequestOutput {
                        duration: Duration::from_secs(60),
                        ..status(200)
                    },
                );

                let error = get(&request, URL)
                    .timeout(Duration::from_secs(1))
                    .send_blocking(&request)
                    .unwrap_err();

                assert_eq!(error.description(), format!("'GET {URL}' timed out"));
                assert_eq!(error.request_target(), Some(&*format!("GET {URL}")));
                assert_eq!(
                    error.source_as::<TransportError>().unwrap().kind(),
                    TransportErrorKind::Timeout
                );
            }

            #[test]
            fn set_response_duration_within_timeout_should_return_output() {
                let request = TestRequest::new();
                let output = RequestOutput {
                    duration: Duration::from_secs(1),
                    ..status(200)
                };
                request.set_response("", output.clone());

                let result = get(&request, URL)
                    .timeout(Duration::from_secs(60))
                    .send_blocking(&request);

                assert_eq!(result.unwrap(), output);
            }
        }

        mod set_responses {
            use super::*;

            #[test]
            fn set_responses_should_return_outputs_in_order_repeating_last() {
                let request = TestRequest::new();
                request.set_responses("", [status(500), status(404), status(200)]);

                let results = (0..4).map(|_| get(&request, URL).send_blocking(&request));

                assert_eq!(statuses(results), [500, 404, 200, 200]);
            }

            #[test]
            fn set_responses_empty_should_set_nothing() {
                let request = TestRequest::new();
                request.set_response("", status(200));

                request.set_responses("", []);

                assert!(get(&request, URL).send_blocking(&request).is_ok());
            }

            #[test]
            fn set_responses_with_retries_should_succeed_without_waiting() {
                // GIVEN a request failing transiently twice before it succeeds
                let (request, recorder) = observed();
                request.set_responses("", [status(503), status(503), status(200)]);
                let start = std::time::Instant::now();

                // WHEN retrying it with a long delay
                request.add_interceptor(HttpRequest::retry(2, Duration::from_secs(3600)));
                let output = get(&request, URL).send_blocking(&request).unwrap();

                // THEN the last attempt of the one run succeeds at once
                assert_eq!(output.status, 200);
                assert_eq!(request.calls().len(), 3);
                assert!(start.elapsed() < Duration::from_secs(60));
                assert_eq!(
                    recorder.events(),
                    [format!("started GET {URL}"), "finished true".to_string()]
                );
            }
        }

        mod set_response_if {
            use super::*;

            #[test]
            fn set_response_if_should_match_by_predicate() {
                // GIVEN a default and a response for bodies with a query
                let request = TestRequest::new();
                request.set_response("", status(404));
                request.set_response_if(|call| call.body.starts_with(b"query"), status(200));
                let client = request.blocking_client().unwrap();

                // WHEN sending requests with and without it
                let matching = client.post(URL).body("query").send_blocking(&request);
                let other = client.post(URL).body("mutation").send_blocking(&request);

                // THEN only the one with the query gets the response
                assert_eq!(statuses([matching, other]), [200, 404]);
            }

            #[test]
            fn set_response_if_predicate_using_request_should_not_deadlock() {
                // GIVEN a predicate matching only the first call
                let request = Arc::new(TestRequest::new());
                let calls = request.clone();
                request.set_response("", status(404));
                request.set_response_if(move |_| calls.calls().is_empty(), status(200));

                // WHEN sending two requests
                let first = get(&request, URL).send_blocking(&*request);
                let second = get(&request, URL).send_blocking(&*request);

                // THEN the predicate sees the calls recorded
                assert_eq!(statuses([first, second]), [200, 404]);
            }
        }

        mod set_chunks {
            use super::*;

            #[test]
            fn set_chunks_should_return_chunks_as_body() {
                let request = TestRequest::new();
                request.set_chunks(
                    "",
                    RequestOutput {
                        body: b"replaced".to_vec(),
                        ..status(200)
                    },
                    chunks(),
                );

                let output = get(&request, URL).send_blocking(&request).unwrap();

                assert_eq!(output.body, b"abc");
            }

            #[test]
            fn set_chunks_streaming_should_report_each_chunk() {
                let (request, recorder) = observed();
                request.set_chunks("", status(200), chunks());

                get(&request, URL)
                    .keep_partial_body()
                    .send_blocking(&request)
                    .unwrap();

                assert_eq!(
                    recorder.events(),
                    [
                        format!("started GET {URL}"),
                        "body ab".to_string(),
                        "body c".to_string(),
                        "finished true".to_string(),
                    ]
                );
            }

            #[test]
            fn set_chunks_duration_exceeding_timeout_should_time_out_without_chunks() {
                let request = TestRequest::new();
                let mut received = Vec::new();
                request.set_chunks(
                    "",
                    RequestOutput {
                        duration: Duration::from_secs(60),
                        ..status(200)
                    },
                    chunks(),
                );

                let error = get(&request, URL)
                    .timeout(Duration::from_secs(1))
                    .send_blocking_with_sink(&request, &mut collect_into(&mut received))
                    .unwrap_err();

                assert_eq!(
                    error.source_as::<TransportError>().unwrap().kind(),
                    TransportErrorKind::Timeout
                );
                assert!(received.is_empty());
            }
        }

        mod set_interrupted {
            use super::*;

            #[test]
            fn set_interrupted_should_pass_chunks_to_sink_before_failing() {
                // GIVEN a download interrupted after its chunks
                let request = TestRequest::new();
                request.set_interrupted("", status(200), chunks(), TransportErrorKind::Other);
                let mut received = Vec::new();

                // WHEN sending it to a sink
                let error = get(&request, URL)
                    .send_blocking_with_sink(&request, &mut collect_into(&mut received))
                    .unwrap_err();

                // THEN the sink gets the chunks before the transport error
                assert_eq!(received, chunks());
                assert_eq!(error.description(), format!("'GET {URL}' failed"));
                let transport = error.source_as::<TransportError>().unwrap();
                assert_eq!(transport.kind(), TransportErrorKind::Other);
                assert_eq!(transport.output(), Some(&status(200)));
            }

            #[test]
            fn set_interrupted_streaming_should_return_part_received_in_error() {
                let request = TestRequest::new();
                request.set_interrupted("", status(200), chunks(), TransportErrorKind::Other);

                let streamed = get(&request, URL)
                    .keep_partial_body()
                    .send_blocking(&request);
                let buffered = get(&request, URL).send_blocking(&request);

                let body = |result: &Result<RequestOutput>| {
                    result
                        .as_ref()
                        .unwrap_err()
                        .source_as::<TransportError>()
                        .and_then(TransportError::output)
                        .unwrap()
                        .body
                        .clone()
                };
                assert_eq!(body(&streamed), b"abc");
                assert!(body(&buffered).is_empty());
            }

            #[test]
            fn set_interrupted_with_sink_should_not_be_retried() {
                // GIVEN an interrupted download
                let request = TestRequest::new();
                request.set_interrupted("", status(200), chunks(), TransportErrorKind::Other);
                let mut received = Vec::new();

                // WHEN sending it to a sink retrying any failure
                request.add_interceptor(HttpRequest::retry(2, Duration::ZERO).when(|_, _| true));
                let result = get(&request, URL)
                    .send_blocking_with_sink(&request, &mut collect_into(&mut received));

                // THEN the sink gets the body once
                assert!(result.is_err());
                assert_eq!(request.calls().len(), 1);
                assert_eq!(received, chunks());
            }

            #[test]
            fn set_interrupted_without_sink_should_be_retried_as_transient() {
                let request = TestRequest::new();
                request.set_interrupted("", status(200), chunks(), TransportErrorKind::Other);

                request.add_interceptor(HttpRequest::retry(1, Duration::from_secs(3600)));
                let result = get(&request, URL).send_blocking(&request);

                assert!(result.is_err());
                assert_eq!(request.calls().len(), 2);
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn set_interrupted_with_async_sink_should_not_be_retried() {
                let request = TestRequest::new();
                request.set_interrupted("", status(200), chunks(), TransportErrorKind::Other);
                let mut received = Vec::new();
                request.add_interceptor(HttpRequest::retry(2, Duration::ZERO));
                let builder = request.client().unwrap().get(URL);

                let result = block_on(
                    builder.send_async_with_sink(&request, &mut collect_into(&mut received)),
                );

                assert!(result.is_err());
                assert_eq!(request.calls().len(), 1);
                assert_eq!(received, chunks());
            }
        }

        mod set_pending {
            use super::*;

            #[test]
            fn set_pending_gate_should_release_once_all_clones_are_dropped() {
                // GIVEN a request held by a gate with two clones
                let request = TestRequest::new();

                std::thread::scope(|scope| {
                    let gate = request.set_pending("", status(200));
                    let clone = gate.clone();
                    let pending = scope.spawn(|| get(&request, URL).send_blocking(&request));
                    let _ = std::iter::repeat_with(std::thread::yield_now)
                        .find(|()| !request.calls().is_empty());

                    // WHEN dropping one clone and then the other
                    drop(gate);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    let held = !pending.is_finished();
                    drop(clone);

                    // THEN the request is held until both are dropped
                    assert!(held);
                    assert!(pending.join().unwrap().is_ok());
                });
            }

            #[test]
            fn set_pending_should_hold_request_until_released() {
                // GIVEN a pending response
                let request = TestRequest::new();
                let gate = request.set_pending(
                    "",
                    RequestOutput {
                        body: b"body".to_vec(),
                        ..status(200)
                    },
                );
                let builder = get(&request, URL);

                std::thread::scope(|scope| {
                    // WHEN sending a request matching it
                    let pending = scope.spawn(|| builder.send_blocking(&request));

                    // THEN it is recorded and held until the gate is released
                    let _ = std::iter::repeat_with(std::thread::yield_now)
                        .find(|()| !request.calls().is_empty());
                    assert!(!pending.is_finished());
                    gate.release();
                    assert_eq!(pending.join().unwrap().unwrap().body, b"body");
                });
            }

            #[test]
            fn set_pending_failed_should_fail_request_with_kind() {
                let request = TestRequest::new();
                let gate = request.set_pending("", status(200));
                gate.fail(TransportErrorKind::Connect);

                let error = get(&request, URL).send_blocking(&request).unwrap_err();

                assert_eq!(
                    error.source_as::<TransportError>().unwrap().kind(),
                    TransportErrorKind::Connect
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn set_pending_should_hold_async_request_until_released() {
                // GIVEN an async request matching a pending response
                let request = TestRequest::new();
                let gate = request.set_pending("", status(200));
                let builder = request.client().unwrap().get(URL);
                let mut future = std::pin::pin!(builder.send_async(&request));
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());

                // WHEN polling it before and after releasing the gate
                let held = future.as_mut().poll(&mut context);
                gate.release();
                let result = block_on(future);

                // THEN it is recorded and held until released
                assert!(held.is_pending());
                assert_eq!(request.calls(), [format!("GET {URL}")]);
                assert_eq!(result.unwrap(), status(200));
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn set_pending_cancelled_should_tell_interceptors() {
                // GIVEN an async request held by a pending response
                let (request, recorder) = observed();
                let _gate = request.set_pending("", status(200));
                let builder = request.client().unwrap().get(URL);
                let mut future = Box::pin(builder.send_async(&request));
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                assert!(future.as_mut().poll(&mut context).is_pending());

                // WHEN cancelling it
                drop(future);

                // THEN the interceptors are told
                assert_eq!(
                    recorder.events(),
                    [format!("started GET {URL}"), format!("cancelled GET {URL}")]
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn set_pending_failed_should_fail_async_request_at_once() {
                let request = TestRequest::new();
                let gate = request.set_pending("", status(200));
                gate.fail(TransportErrorKind::Timeout);
                let builder = request.client().unwrap().get(URL);

                let error = block_on(builder.send_async(&request)).unwrap_err();

                assert_eq!(
                    error.source_as::<TransportError>().unwrap().kind(),
                    TransportErrorKind::Timeout
                );
            }
        }

        mod set_transport_error {
            use super::*;

            #[test]
            fn set_transport_error_should_fail_request_with_kind() {
                let request = TestRequest::new();
                request.set_transport_error("example.com", TransportErrorKind::Connect);

                let error = get(&request, &format!("{URL}?token=secret"))
                    .send_blocking(&request)
                    .unwrap_err();

                assert_eq!(
                    error.description(),
                    format!("'GET {URL}' failed to connect")
                );
                assert_eq!(
                    error.source_as::<TransportError>().unwrap().kind(),
                    TransportErrorKind::Connect
                );
                assert!(error.request_output().is_none());
                assert!(
                    error
                        .source_as::<TransportError>()
                        .unwrap()
                        .output()
                        .is_none()
                );
            }
        }

        mod set_env {
            use super::*;

            #[test]
            fn set_env_should_take_proxy_variables() {
                let mut request = TestRequest::new();
                request.blocking_client().unwrap();
                let env = crate::TestEnv::new()
                    .with_vars(vec![("HTTP_PROXY", "http://proxy"), ("PATH", "/bin")]);

                request.set_env(env.share());

                assert_eq!(
                    request.clients.reqwest_env(),
                    ReqwestEnv::new(&[("HTTP_PROXY".into(), "http://proxy".into())])
                );
                assert!(request.blocking_client().is_ok());
            }

            #[test]
            fn set_env_vars_changed_should_take_proxy_variables_only() {
                let request = TestRequest::new();
                request.blocking_client().unwrap();

                request.set_var("HTTPS_PROXY".to_string(), "http://proxy".to_string());
                request.set_var("PATH".to_string(), "/bin".to_string());

                assert_eq!(
                    request.clients.reqwest_env(),
                    ReqwestEnv::new(&[("HTTPS_PROXY".into(), "http://proxy".into())])
                );
                assert!(request.blocking_client().is_ok());
            }

            #[test]
            fn set_env_proxy_removed_should_be_forgotten() {
                let request = TestRequest::new();
                request.set_var("HTTPS_PROXY".to_string(), "http://proxy".to_string());

                request.remove_var("HTTPS_PROXY");

                assert_eq!(request.clients.reqwest_env(), ReqwestEnv::default());
            }
        }

        mod request {
            use super::*;

            mod blocking_client_builder {
                use super::*;

                #[test]
                fn blocking_client_builder_should_use_proxies_of_env() {
                    let request = TestRequest::new();
                    crate::request::tests::request::blocking_client_builder::blocking_client_builder_should_use_proxies_of_env(&request, &request.clients.env);
                }

                #[test]
                fn blocking_client_builder_should_time_out_connecting_after_30_seconds() {
                    crate::request::tests::request::blocking_client_builder::blocking_client_builder_should_time_out_connecting_after_30_seconds(&TestRequest::new());
                }
            }

            #[cfg(feature = "tokio")]
            mod client_builder {
                use super::*;

                #[test]
                fn client_builder_should_use_proxies_of_env() {
                    let request = TestRequest::new();
                    crate::request::tests::request::client_builder::client_builder_should_use_proxies_of_env(&request, &request.clients.env);
                }

                #[test]
                fn client_builder_should_time_out_connecting_after_30_seconds_only() {
                    crate::request::tests::request::client_builder::client_builder_should_time_out_connecting_after_30_seconds_only(&TestRequest::new());
                }
            }

            mod blocking_client {
                use super::*;

                #[test]
                fn blocking_client_should_build_default_client() {
                    assert!(TestRequest::new().blocking_client().is_ok());
                }

                #[test]
                fn blocking_client_invalid_builder_should_return_error_every_time() {
                    crate::request::tests::request::blocking_client::blocking_client_invalid_builder_should_return_error_every_time(&TestRequest::new());
                }
            }

            mod set_blocking_client {
                use super::*;

                #[test]
                fn set_blocking_client_should_replace_default_client() {
                    crate::request::tests::request::set_blocking_client::set_blocking_client_should_replace_default_client(&TestRequest::new());
                }
            }

            mod send_blocking {
                use super::*;

                #[test]
                fn send_blocking_should_return_response_and_pass_run_to_interceptors() {
                    crate::request::tests::request::send_blocking::send_blocking_should_return_response_and_pass_run_to_interceptors(&answering(200, "body"), URL);
                }

                #[test]
                fn send_blocking_should_send_request_body_and_headers() {
                    let request = answering(200, "");
                    crate::request::tests::request::send_blocking::send_blocking_should_send_request_body_and_headers(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_blocking_failed_status_should_return_error_with_output() {
                    crate::request::tests::request::send_blocking::send_blocking_failed_status_should_return_error_with_output(&answering(404, "missing"), URL);
                }

                #[test]
                fn send_blocking_display_lines_should_limit_displayed_body() {
                    crate::request::tests::request::send_blocking::send_blocking_display_lines_should_limit_displayed_body(&answering(500, "1\n2\n3"), URL);
                }

                #[test]
                fn send_blocking_invalid_request_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_blocking::send_blocking_invalid_request_should_return_error_bypassing_interceptors(&TestRequest::new());
                }

                #[test]
                fn send_blocking_unreadable_body_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_blocking::send_blocking_unreadable_body_should_return_error_bypassing_interceptors(&answering(200, ""), URL);
                }

                #[test]
                fn send_blocking_connect_failure_should_return_connect_transport_error() {
                    let request = TestRequest::new();
                    request.set_transport_error("", TransportErrorKind::Connect);
                    crate::request::tests::request::send_blocking::send_blocking_connect_failure_should_return_connect_transport_error(&request, URL);
                }

                #[test]
                fn send_blocking_timeout_should_return_timeout_transport_error() {
                    let request = TestRequest::new();
                    request.set_response(
                        "",
                        RequestOutput {
                            duration: Duration::from_secs(60),
                            ..status(200)
                        },
                    );
                    crate::request::tests::request::send_blocking::send_blocking_timeout_should_return_timeout_transport_error(&request, URL);
                }

                #[test]
                fn send_blocking_zero_timeout_should_return_timeout_transport_error() {
                    crate::request::tests::request::send_blocking::send_blocking_zero_timeout_should_return_timeout_transport_error(&answering(200, "body"), URL);
                }

                #[test]
                fn send_blocking_stalled_body_should_return_timeout_transport_error_with_head() {
                    crate::request::tests::request::send_blocking::send_blocking_stalled_body_should_return_timeout_transport_error_with_head(&interrupted(TransportErrorKind::Timeout), URL);
                }

                #[test]
                fn send_blocking_truncated_body_should_return_other_transport_error() {
                    crate::request::tests::request::send_blocking::send_blocking_truncated_body_should_return_other_transport_error(&interrupted(TransportErrorKind::Other), URL);
                }

                #[test]
                fn send_blocking_streaming_should_pass_body_between_start_and_end() {
                    crate::request::tests::request::send_blocking::send_blocking_streaming_should_pass_body_between_start_and_end(&answering(200, "body"), URL);
                }

                #[test]
                fn send_blocking_with_retry_should_retry_until_success() {
                    crate::request::tests::request::send_blocking::send_blocking_with_retry_should_retry_until_success(&unavailable_once(), URL);
                }

                #[test]
                fn send_blocking_with_retry_should_send_streamed_body_by_each_attempt() {
                    let request = unavailable_once();
                    crate::request::tests::request::send_blocking::send_blocking_with_retry_should_send_streamed_body_by_each_attempt(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_blocking_with_retry_interrupted_body_should_be_retried() {
                    let request = interrupted(TransportErrorKind::Other);
                    crate::request::tests::request::send_blocking::send_blocking_with_retry_interrupted_body_should_be_retried(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_blocking_should_return_output_and_record_call() {
                    let request = TestRequest::new();
                    let output = RequestOutput {
                        body: b"body".to_vec(),
                        ..status(200)
                    };
                    request.set_response("", output.clone());

                    let result = get(&request, URL).send_blocking(&request);

                    assert_eq!(result.unwrap(), output);
                    assert_eq!(request.calls(), [format!("GET {URL}")]);
                }

                #[test]
                fn send_blocking_display_lines_should_return_output_equal_to_set() {
                    let request = TestRequest::new();
                    let output = RequestOutput {
                        body: b"body".to_vec(),
                        ..status(200)
                    };
                    request.set_response("", output.clone());

                    let result = get(&request, URL)
                        .display_lines(3)
                        .send_blocking(&request)
                        .unwrap();

                    assert_eq!(result, output);
                    assert_eq!(result.display_lines, 3);
                }

                #[test]
                fn send_blocking_unmatched_should_return_error_and_record_call() {
                    let request = TestRequest::new();
                    request.set_response("other.com", status(200));

                    let error = get(&request, &format!("{URL}?q=1"))
                        .send_blocking(&request)
                        .unwrap_err();

                    assert_eq!(error.description(), format!("'GET {URL}': no response set"));
                    assert_eq!(request.calls(), [format!("GET {URL}?q=1")]);
                }

                #[test]
                fn send_blocking_accepted_status_should_return_output_without_retry() {
                    let (request, recorder) = observed();
                    request.set_response(
                        "",
                        RequestOutput {
                            body: b"missing".to_vec(),
                            ..status(404)
                        },
                    );

                    request
                        .add_interceptor(HttpRequest::retry(2, Duration::ZERO).when(|_, _| true));
                    request.add_interceptor(Accept(404));

                    let output = get(&request, URL).send_blocking(&request).unwrap();

                    assert_eq!(output.status, 404);
                    assert_eq!(request.calls().len(), 1);
                    assert_eq!(
                        recorder.events(),
                        [format!("started GET {URL}"), "finished true".to_string()]
                    );
                }

                #[test]
                fn send_blocking_status_not_accepted_should_fail() {
                    let request = TestRequest::new();
                    request.add_interceptor(Accept(404));
                    request.set_response("", status(500));

                    let error = get(&request, URL).send_blocking(&request).unwrap_err();

                    assert_eq!(error.request_output(), Some(&status(500)));
                }

                #[test]
                fn send_blocking_invalid_request_should_not_record_call() {
                    let request = TestRequest::new();

                    let _ = get(&request, "not a url").send_blocking(&request);

                    assert!(request.calls().is_empty());
                }

                #[test]
                fn send_blocking_streamed_body_should_be_recorded() {
                    let request = TestRequest::new();
                    request.set_response("", status(200));

                    request
                        .blocking_client()
                        .unwrap()
                        .post(URL)
                        .body(reqwest::blocking::Body::new(std::io::Cursor::new(b"data")))
                        .send_blocking(&request)
                        .unwrap();

                    assert_eq!(request.call_details()[0].body, b"data");
                }

                #[test]
                fn send_blocking_streaming_should_pass_non_empty_body_to_interceptors() {
                    // GIVEN a recorder and responses with and without a body
                    let (request, recorder) = observed();
                    request.set_responses(
                        "",
                        [
                            RequestOutput {
                                body: b"body".to_vec(),
                                ..status(200)
                            },
                            status(200),
                        ],
                    );

                    // WHEN streaming both
                    for _ in 0..2 {
                        get(&request, URL)
                            .keep_partial_body()
                            .send_blocking(&request)
                            .unwrap();
                    }

                    // THEN only the body is passed between start and end
                    assert_eq!(
                        recorder.events(),
                        [
                            format!("started GET {URL}"),
                            "body body".to_string(),
                            "finished true".to_string(),
                            format!("started GET {URL}"),
                            "finished true".to_string(),
                        ]
                    );
                }

                #[test]
                fn send_blocking_with_options_should_record_options() {
                    let request = TestRequest::new();
                    request.set_response("", status(200));

                    ConfiguredRequest::new(get(&request, URL))
                        .keep_partial_body()
                        .send_blocking(&request)
                        .unwrap();

                    assert!(request.call_details()[0].options.keep_partial_body);
                }

                #[test]
                fn send_blocking_with_retry_should_retry_only_failures_safe_for_method() {
                    fn calls(method: reqwest::Method, failure: &dyn Fn(&TestRequest)) -> usize {
                        let request = TestRequest::new();
                        request.add_interceptor(HttpRequest::retry(1, Duration::ZERO));
                        failure(&request);

                        let _ = request
                            .blocking_client()
                            .unwrap()
                            .request(method, URL)
                            .send_blocking(&request);

                        request.calls().len()
                    }
                    let server_error =
                        |request: &TestRequest| request.set_response("", status(500));
                    let unavailable = |request: &TestRequest| request.set_response("", status(503));
                    let timeout = |request: &TestRequest| {
                        request.set_transport_error("", TransportErrorKind::Timeout)
                    };
                    let connect = |request: &TestRequest| {
                        request.set_transport_error("", TransportErrorKind::Connect)
                    };

                    for (method, failure, expected) in [
                        (
                            reqwest::Method::GET,
                            &server_error as &dyn Fn(&TestRequest),
                            2,
                        ),
                        (reqwest::Method::GET, &timeout, 2),
                        (reqwest::Method::POST, &server_error, 1),
                        (reqwest::Method::POST, &timeout, 1),
                        (reqwest::Method::POST, &unavailable, 2),
                        (reqwest::Method::POST, &connect, 2),
                    ] {
                        assert_eq!(calls(method.clone(), failure), expected, "method: {method}");
                    }
                }

                #[cfg(feature = "tokio")]
                #[test]
                #[should_panic(expected = "must not be sent in an async context")]
                fn send_blocking_in_async_context_should_panic() {
                    let request = TestRequest::new();
                    request.set_response("", status(200));
                    let builder = get(&request, URL);

                    let _ = block_on(async { builder.send_blocking(&request) });
                }
            }

            mod send_blocking_with_sink {
                use super::*;

                #[test]
                fn send_blocking_with_sink_failed_status_should_return_body_in_output() {
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_failed_status_should_return_body_in_output(&answering(404, "missing"), URL);
                }

                #[test]
                fn send_blocking_with_sink_interrupted_body_should_keep_received_part() {
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_interrupted_body_should_keep_received_part(&interrupted(TransportErrorKind::Other), URL);
                }

                #[test]
                fn send_blocking_with_sink_interrupted_body_should_not_be_retried() {
                    let request = interrupted(TransportErrorKind::Other);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_interrupted_body_should_not_be_retried(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_blocking_with_sink_failure_before_body_should_be_retried() {
                    let request = unavailable_once();
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_failure_before_body_should_be_retried(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_blocking_with_sink_failing_sink_should_return_its_error_without_retrying() {
                    let request = answering(200, "body");
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_failing_sink_should_return_its_error_without_retrying(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_blocking_with_sink_should_pass_each_chunk_to_sink() {
                    let request = TestRequest::new();
                    request.set_chunks("", status(200), chunks());
                    let mut received = Vec::new();

                    get(&request, URL)
                        .send_blocking_with_sink(&request, &mut collect_into(&mut received))
                        .unwrap();

                    assert_eq!(received, chunks());
                }

                #[test]
                fn send_blocking_with_sink_accepted_status_should_return_body_in_output() {
                    let request = TestRequest::new();
                    request.add_interceptor(Accept(404));
                    request.set_response(
                        "",
                        RequestOutput {
                            body: b"missing".to_vec(),
                            ..status(404)
                        },
                    );
                    let mut received = Vec::new();

                    let output = get(&request, URL)
                        .send_blocking_with_sink(&request, &mut collect_into(&mut received))
                        .unwrap();

                    assert_eq!(output.body, b"missing");
                    assert!(received.is_empty());
                }

                #[cfg(feature = "tokio")]
                #[test]
                #[should_panic(expected = "must not be sent in an async context")]
                fn send_blocking_with_sink_in_async_context_should_panic() {
                    let request = TestRequest::new();
                    request.set_response("", status(200));
                    let builder = get(&request, URL);

                    let _ = block_on(async {
                        builder
                            .send_blocking_with_sink(&request, &mut collect_into(&mut Vec::new()))
                    });
                }

                #[test]
                fn send_blocking_with_sink_should_pass_head_then_body_to_sink() {
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_should_pass_head_then_body_to_sink(&answering(200, "body"), URL);
                }

                #[test]
                fn send_blocking_with_sink_refused_head_should_return_its_error_without_retrying() {
                    let request = answering(200, "body");
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_refused_head_should_return_its_error_without_retrying(&request, URL, &|| sent(&request));
                }
            }

            #[cfg(feature = "tokio")]
            mod client {
                use super::*;

                #[test]
                fn client_should_build_default_client() {
                    assert!(TestRequest::new().client().is_ok());
                }

                #[test]
                fn client_invalid_builder_should_return_error_every_time() {
                    crate::request::tests::request::client::client_invalid_builder_should_return_error_every_time(&TestRequest::new());
                }
            }

            #[cfg(feature = "tokio")]
            mod set_client {
                use super::*;

                #[test]
                fn set_client_should_replace_default_client() {
                    crate::request::tests::request::set_client::set_client_should_replace_default_client(&TestRequest::new());
                }
            }

            #[cfg(feature = "tokio")]
            mod send_async {
                use super::*;

                #[test]
                fn send_async_should_return_response_and_pass_run_to_interceptors() {
                    crate::request::tests::request::send_async::send_async_should_return_response_and_pass_run_to_interceptors(&answering(200, "body"), URL);
                }

                #[test]
                fn send_async_failed_status_should_return_error_with_output() {
                    crate::request::tests::request::send_async::send_async_failed_status_should_return_error_with_output(&answering(404, "missing"), URL);
                }

                #[test]
                fn send_async_invalid_request_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_async::send_async_invalid_request_should_return_error_bypassing_interceptors(&TestRequest::new());
                }

                #[test]
                fn send_async_unreadable_body_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_async::send_async_unreadable_body_should_return_error_bypassing_interceptors(&answering(200, ""), URL);
                }

                #[test]
                fn send_async_connect_failure_should_return_connect_transport_error() {
                    let request = TestRequest::new();
                    request.set_transport_error("", TransportErrorKind::Connect);
                    crate::request::tests::request::send_async::send_async_connect_failure_should_return_connect_transport_error(&request, URL);
                }

                #[test]
                fn send_async_timeout_should_return_timeout_transport_error() {
                    let request = TestRequest::new();
                    request.set_response(
                        "",
                        RequestOutput {
                            duration: Duration::from_secs(60),
                            ..status(200)
                        },
                    );
                    crate::request::tests::request::send_async::send_async_timeout_should_return_timeout_transport_error(&request, URL);
                }

                #[test]
                fn send_async_zero_timeout_should_return_timeout_transport_error() {
                    crate::request::tests::request::send_async::send_async_zero_timeout_should_return_timeout_transport_error(&answering(200, "body"), URL);
                }

                #[test]
                fn send_async_stalled_body_should_return_timeout_transport_error_with_head() {
                    crate::request::tests::request::send_async::send_async_stalled_body_should_return_timeout_transport_error_with_head(&interrupted(TransportErrorKind::Timeout), URL);
                }

                #[test]
                fn send_async_truncated_body_should_return_other_transport_error() {
                    crate::request::tests::request::send_async::send_async_truncated_body_should_return_other_transport_error(&interrupted(TransportErrorKind::Other), URL);
                }

                #[test]
                fn send_async_with_retry_should_retry_until_success() {
                    let request = unavailable_once();
                    crate::request::tests::request::send_async::send_async_with_retry_should_retry_until_success(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_async_with_retry_should_send_streamed_body_by_each_attempt() {
                    let request = unavailable_once();
                    crate::request::tests::request::send_async::send_async_with_retry_should_send_streamed_body_by_each_attempt(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_async_cancelled_should_tell_interceptors() {
                    let request = TestRequest::new();
                    let _gate = request.set_pending("", status(200));
                    crate::request::tests::request::send_async::send_async_cancelled_should_tell_interceptors(&request, URL);
                }

                #[test]
                fn send_async_should_return_output_and_record_call() {
                    let request = TestRequest::new();
                    request.set_response("", status(200));
                    let builder = request.client().unwrap().get(URL);

                    let result = block_on(builder.send_async(&request));

                    assert_eq!(result.unwrap(), status(200));
                    assert_eq!(request.calls(), [format!("GET {URL}")]);
                }

                #[test]
                fn send_async_with_retries_should_succeed_without_waiting() {
                    let request = TestRequest::new();
                    request.add_interceptor(HttpRequest::retry(1, Duration::from_secs(3600)));
                    request.set_responses("", [status(503), status(200)]);
                    let builder = request.client().unwrap().get(URL);

                    let output = block_on(builder.send_async(&request)).unwrap();

                    assert_eq!(output.status, 200);
                    assert_eq!(request.calls().len(), 2);
                }

                #[test]
                fn send_async_non_idempotent_server_error_should_not_retry() {
                    let request = TestRequest::new();
                    request.add_interceptor(HttpRequest::retry(1, Duration::ZERO));
                    request.set_responses("", [status(500), status(200)]);
                    let builder = request.client().unwrap().post(URL);

                    let error = block_on(builder.send_async(&request)).unwrap_err();

                    assert_eq!(error.request_output().unwrap().status, 500);
                    assert_eq!(request.calls().len(), 1);
                }
            }

            #[cfg(feature = "tokio")]
            mod send_async_with_sink {
                use super::*;

                #[test]
                fn send_async_with_sink_failed_status_should_return_body_in_output() {
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_failed_status_should_return_body_in_output(&answering(404, "missing"), URL);
                }

                #[test]
                fn send_async_with_sink_interrupted_body_should_keep_received_part() {
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_interrupted_body_should_keep_received_part(&interrupted(TransportErrorKind::Other), URL);
                }

                #[test]
                fn send_async_with_sink_failure_before_body_should_be_retried() {
                    let request = unavailable_once();
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_failure_before_body_should_be_retried(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_async_with_sink_failing_sink_should_return_its_error() {
                    let request = answering(200, "body");
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_failing_sink_should_return_its_error(&request, URL, &|| sent(&request));
                }

                #[test]
                fn send_async_with_sink_panicking_sink_should_not_tell_cancelled() {
                    // GIVEN an async request whose sink panics
                    let (request, recorder) = observed();
                    request.set_chunks("", status(200), chunks());
                    let builder = request.client().unwrap().get(URL);

                    // WHEN sending it
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        block_on(
                            builder
                                .send_async_with_sink(&request, &mut |(), _: &[u8]| panic!("sink")),
                        )
                    }));

                    // THEN the run is not told as cancelled
                    assert!(result.is_err());
                    assert_eq!(recorder.events(), [format!("started GET {URL}")]);
                }

                #[test]
                fn send_async_with_sink_unmatched_should_return_error() {
                    let request = TestRequest::new();
                    let mut received = Vec::new();
                    let builder = request.client().unwrap().get(URL);

                    let error = block_on(
                        builder.send_async_with_sink(&request, &mut collect_into(&mut received)),
                    )
                    .unwrap_err();

                    assert_eq!(error.description(), format!("'GET {URL}': no response set"));
                }

                #[test]
                fn send_async_with_sink_should_pass_head_then_body_to_sink() {
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_should_pass_head_then_body_to_sink(&answering(200, "body"), URL);
                }

                #[test]
                fn send_async_with_sink_refused_head_should_return_its_error_without_retrying() {
                    let request = answering(200, "body");
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_refused_head_should_return_its_error_without_retrying(&request, URL, &|| sent(&request));
                }
            }

            mod interceptors {
                use super::*;

                #[test]
                fn interceptors_should_be_none_by_default() {
                    crate::request::tests::request::interceptors::interceptors_should_be_none_by_default(&TestRequest::new());
                }
            }

            mod clear_interceptors {
                use super::*;

                #[test]
                fn clear_interceptors_should_remove_interceptors_of_later_runs() {
                    crate::request::tests::request::clear_interceptors::clear_interceptors_should_remove_interceptors_of_later_runs(&answering(200, ""), URL);
                }
            }

            mod add_interceptor {
                use super::*;

                #[test]
                fn add_interceptor_should_add_innermost_interceptor() {
                    crate::request::tests::request::add_interceptor::add_interceptor_should_add_innermost_interceptor(&answering(200, ""), URL);
                }

                #[test]
                fn add_interceptor_setting_output_should_not_record_call() {
                    let request = TestRequest::new();
                    request.add_interceptor(DryRun);

                    let output = get(&request, URL).send_blocking(&request).unwrap();

                    assert_eq!(output, status(200));
                    assert!(request.calls().is_empty());
                }
            }
        }
    }

    mod test_gate {
        use super::*;

        mod release {
            use super::*;

            #[test]
            fn release_after_fail_should_keep_failure() {
                let request = TestRequest::new();
                let gate = request.set_pending("", status(200));

                gate.fail(TransportErrorKind::Other);
                gate.release();

                assert!(get(&request, URL).send_blocking(&request).is_err());
            }
        }

        mod fail {
            use super::*;

            #[test]
            fn fail_after_release_should_keep_release() {
                let request = TestRequest::new();
                let gate = request.set_pending("", status(200));

                gate.release();
                gate.fail(TransportErrorKind::Other);

                assert!(get(&request, URL).send_blocking(&request).is_ok());
            }
        }
    }
}
