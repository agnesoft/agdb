use std::time::Duration;
use std::time::Instant;

use crate::Result;
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
use crate::utils::Runs;

use super::ConfiguredRequest;
use super::DefaultClients;
use super::HttpRequest;
use super::Received;
use super::Request;
use super::RequestOutput;
use super::TransportError;
use super::TransportErrorKind;
#[cfg(feature = "tokio")]
use super::run_async;
use super::run_blocking;
use super::transport_failure;

/// [internal]
///
/// Sends the blocking runs by its client.
struct BlockingClient<'a>(&'a reqwest::blocking::Client);

/// [internal]
///
/// Sends the async runs by its client.
#[cfg(feature = "tokio")]
struct AsyncClient<'a>(&'a reqwest::Client);

/// The production [`Request`] implementation backed by [`reqwest`].
///
/// Normally obtained via [`platform()`](crate::platform) rather than
/// constructed directly.
pub struct RequestImpl {
    pub(super) clients: DefaultClients,
    runs: Runs<HttpRequest>,
}

impl Executor for BlockingClient<'_> {
    type Subject = HttpRequest;

    fn execute<'a>(
        &'a self,
        run: &'a Run<HttpRequest>,
        emit: Emit<'a, HttpRequest>,
    ) -> BoxFuture<'a, Result<RequestOutput>> {
        Box::pin(async move {
            check_timeout(run.subject())?;
            let start = Instant::now();
            let mut response = self
                .0
                .execute(run.subject().blocking_request())
                .map_err(|error| transport_failure(run.subject(), error.into()))?;
            let headers = std::mem::take(response.headers_mut());
            let mut received =
                Received::start(run, emit, response.status().as_u16(), headers).await?;
            let mut buffer = vec![0; READ_BYTES];

            // WHY: read in parts, as the client timeout limits each read,
            // but all of `bytes`.
            loop {
                match std::io::Read::read(&mut response, &mut buffer) {
                    Ok(0) => return Ok(received.end(start.elapsed())),
                    Ok(read) => received.push(&buffer[..read]).await?,
                    Err(error) => {
                        return Err(
                            received.interrupted(start.elapsed(), TransportError::from_io(error))
                        );
                    }
                }
            }
        })
    }

    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
        Box::pin(async move { std::thread::sleep(delay) })
    }
}

impl BlockingExecutor for BlockingClient<'_> {}

#[cfg(feature = "tokio")]
impl Executor for AsyncClient<'_> {
    type Subject = HttpRequest;

    fn execute<'a>(
        &'a self,
        run: &'a Run<HttpRequest>,
        emit: Emit<'a, HttpRequest>,
    ) -> BoxFuture<'a, Result<RequestOutput>> {
        Box::pin(async move {
            check_timeout(run.subject())?;
            let start = Instant::now();
            let mut response = self
                .0
                .execute(run.subject().async_request())
                .await
                .map_err(|error| transport_failure(run.subject(), error.into()))?;
            let headers = std::mem::take(response.headers_mut());
            let mut received =
                Received::start(run, emit, response.status().as_u16(), headers).await?;

            loop {
                match response.chunk().await {
                    Ok(Some(chunk)) => received.push(&chunk).await?,
                    Ok(None) => return Ok(received.end(start.elapsed())),
                    Err(error) => return Err(received.interrupted(start.elapsed(), error.into())),
                }
            }
        })
    }

    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(delay))
    }
}

#[cfg(feature = "tokio")]
impl AsyncExecutor for AsyncClient<'_> {}

/// [internal]
///
/// Fails `request` if its timeout is zero, which reqwest only reports once
/// its timer fires, possibly after the response.
fn check_timeout(request: &HttpRequest) -> Result {
    if request.timeout() == Some(Duration::ZERO) {
        return Err(transport_failure(
            request,
            TransportError::new(TransportErrorKind::Timeout),
        ));
    }

    Ok(())
}

impl RequestImpl {
    /// Creates a new `RequestImpl` with default clients that are built on
    /// first use.
    pub fn new() -> Self {
        Self::with_env(EnvImpl::new(Vec::new(), std::path::PathBuf::new()))
    }

    /// [internal]
    ///
    /// Creates a new `RequestImpl` taking the proxy variables from `env`,
    /// see [`Request`].
    pub(crate) fn with_env(env: EnvImpl) -> Self {
        Self {
            clients: DefaultClients::new(env),
            runs: Runs::new(Interceptors::new()),
        }
    }

    fn dispatch_blocking(
        &self,
        request: ConfiguredRequest<reqwest::blocking::RequestBuilder>,
        sink: Option<&mut dyn Sink<HttpRequest>>,
    ) -> Result<RequestOutput> {
        let (client, run) = request.into_run();
        run_blocking(&self.runs, &BlockingClient(&client), run?, sink)
    }

    #[cfg(feature = "tokio")]
    async fn dispatch_async(
        &self,
        request: ConfiguredRequest<reqwest::RequestBuilder>,
        sink: Option<&mut dyn Sink<HttpRequest>>,
    ) -> Result<RequestOutput> {
        let (client, run) = request.into_run().await;
        run_async(&self.runs, &AsyncClient(&client), run?, sink).await
    }
}

impl Default for RequestImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl Request for RequestImpl {
    fn blocking_client(&self) -> Result<reqwest::blocking::Client> {
        self.clients.blocking_client()
    }

    fn blocking_client_builder(&self) -> reqwest::blocking::ClientBuilder {
        self.clients.blocking_client_builder()
    }

    fn set_blocking_client(&self, builder: reqwest::blocking::ClientBuilder) {
        self.clients.set_blocking_client(builder);
    }

    fn send_blocking(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::blocking::RequestBuilder>>,
    ) -> Result<RequestOutput> {
        self.dispatch_blocking(request.into(), None)
    }

    fn send_blocking_with_sink(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::blocking::RequestBuilder>>,
        sink: &mut dyn Sink<HttpRequest>,
    ) -> Result<RequestOutput> {
        self.dispatch_blocking(request.into(), Some(sink))
    }

    #[cfg(feature = "tokio")]
    fn client(&self) -> Result<reqwest::Client> {
        self.clients.client()
    }

    #[cfg(feature = "tokio")]
    fn client_builder(&self) -> reqwest::ClientBuilder {
        self.clients.client_builder()
    }

    #[cfg(feature = "tokio")]
    fn set_client(&self, builder: reqwest::ClientBuilder) {
        self.clients.set_client(builder);
    }

    #[cfg(feature = "tokio")]
    fn send_async(
        &self,
        request: impl Into<ConfiguredRequest<reqwest::RequestBuilder>>,
    ) -> impl Future<Output = Result<RequestOutput>> + Send {
        self.dispatch_async(request.into(), None)
    }

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

/// The size of the reads of a blocking response body, like
/// [`std::io::copy`].
const READ_BYTES: usize = 8 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::BlockingRequestBuilderExt;
    #[cfg(feature = "tokio")]
    use crate::request::RequestBuilderExt;
    use crate::request::tests::*;

    fn unavailable_once(index: usize, _: &str) -> Vec<u8> {
        if index == 0 {
            response(503, "unavailable")
        } else {
            response(200, "body")
        }
    }

    fn lines(_: usize, _: &str) -> Vec<u8> {
        response(500, "1\n2\n3")
    }

    /// Returns the URL of a path of `server`, as named by the errors.
    fn path(server: &Server) -> String {
        format!("{}/path", server.url)
    }

    mod request_impl {
        use super::*;

        mod default {
            use super::*;

            #[test]
            fn default_should_send_requests() {
                let server = serve(ok);
                let request = RequestImpl::default();

                let output = blocking_get(&request, &server.url)
                    .send_blocking(&request)
                    .unwrap();

                assert_eq!(output.status, 200);
            }
        }

        mod with_env {
            use super::*;

            #[test]
            fn with_env_should_send_through_proxy_of_env() {
                let proxy = serve(ok);
                let env = crate::env::EnvImpl::new(Vec::new(), std::path::PathBuf::new());
                crate::Env::set_var(&env, "HTTP_PROXY", proxy.url.as_str()).unwrap();
                let request = RequestImpl::with_env(env.clone());

                blocking_get(&request, "http://example.invalid/path")
                    .send_blocking(&request)
                    .unwrap();

                assert!(proxy.received()[0].starts_with("GET http://example.invalid/path "));
            }
        }

        mod request {
            use super::*;

            mod blocking_client_builder {
                use super::*;

                #[test]
                fn blocking_client_builder_should_use_proxies_of_env() {
                    let request = RequestImpl::new();
                    crate::request::tests::request::blocking_client_builder::blocking_client_builder_should_use_proxies_of_env(&request, &request.clients.env);
                }

                #[test]
                fn blocking_client_builder_should_time_out_connecting_after_30_seconds() {
                    crate::request::tests::request::blocking_client_builder::blocking_client_builder_should_time_out_connecting_after_30_seconds(&RequestImpl::new());
                }

                #[test]
                fn blocking_client_builder_should_send_through_proxies_of_env() {
                    let proxy = serve(ok);
                    let request = RequestImpl::new();
                    request.set_var("HTTP_PROXY", &proxy.url);
                    let client = request.blocking_client_builder().build().unwrap();

                    client
                        .get("http://example.invalid/")
                        .send_blocking(&request)
                        .unwrap();

                    assert_eq!(proxy.received().len(), 1);
                }
            }

            #[cfg(feature = "tokio")]
            mod client_builder {
                use super::*;

                #[test]
                fn client_builder_should_use_proxies_of_env() {
                    let request = RequestImpl::new();
                    crate::request::tests::request::client_builder::client_builder_should_use_proxies_of_env(&request, &request.clients.env);
                }

                #[test]
                fn client_builder_should_time_out_connecting_after_30_seconds_only() {
                    crate::request::tests::request::client_builder::client_builder_should_time_out_connecting_after_30_seconds_only(&RequestImpl::new());
                }

                #[test]
                fn client_builder_should_send_through_proxies_of_env() {
                    let proxy = serve(ok);
                    let request = RequestImpl::new();
                    request.set_var("HTTP_PROXY", &proxy.url);
                    let client = request.client_builder().build().unwrap();

                    block_on(client.get("http://example.invalid/").send_async(&request)).unwrap();

                    assert_eq!(proxy.received().len(), 1);
                }
            }

            mod blocking_client {
                use super::*;

                #[test]
                fn blocking_client_invalid_builder_should_return_error_every_time() {
                    crate::request::tests::request::blocking_client::blocking_client_invalid_builder_should_return_error_every_time(&RequestImpl::new());
                }
            }

            mod set_blocking_client {
                use super::*;

                #[test]
                fn set_blocking_client_should_replace_default_client() {
                    crate::request::tests::request::set_blocking_client::set_blocking_client_should_replace_default_client(&RequestImpl::new());
                }

                #[test]
                fn set_blocking_client_should_send_by_client_of_builder() {
                    let server = serve(ok);
                    let request = RequestImpl::new();
                    request.blocking_client().unwrap();

                    request.set_blocking_client(
                        reqwest::blocking::Client::builder().user_agent("agplatform-test"),
                    );
                    blocking_get(&request, &server.url)
                        .send_blocking(&request)
                        .unwrap();

                    assert!(server.received()[0].contains("user-agent: agplatform-test\r\n"));
                }
            }

            mod send_blocking {
                use super::*;

                #[test]
                fn send_blocking_should_return_response_and_pass_run_to_interceptors() {
                    let server = serve(ok);
                    crate::request::tests::request::send_blocking::send_blocking_should_return_response_and_pass_run_to_interceptors(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_should_send_request_body_and_headers() {
                    let server = serve(ok);
                    crate::request::tests::request::send_blocking::send_blocking_should_send_request_body_and_headers(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_blocking_failed_status_should_return_error_with_output() {
                    let server = serve(not_found);
                    crate::request::tests::request::send_blocking::send_blocking_failed_status_should_return_error_with_output(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_display_lines_should_limit_displayed_body() {
                    let server = serve(lines);
                    crate::request::tests::request::send_blocking::send_blocking_display_lines_should_limit_displayed_body(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_invalid_request_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_blocking::send_blocking_invalid_request_should_return_error_bypassing_interceptors(&RequestImpl::new());
                }

                #[test]
                fn send_blocking_unreadable_body_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_blocking::send_blocking_unreadable_body_should_return_error_bypassing_interceptors(&RequestImpl::new(), &closed_url());
                }

                #[test]
                fn send_blocking_connect_failure_should_return_connect_transport_error() {
                    crate::request::tests::request::send_blocking::send_blocking_connect_failure_should_return_connect_transport_error(&RequestImpl::new(), &format!("{}/path", closed_url()));
                }

                #[test]
                fn send_blocking_timeout_should_return_timeout_transport_error() {
                    let server = serve(slow);
                    crate::request::tests::request::send_blocking::send_blocking_timeout_should_return_timeout_transport_error(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_zero_timeout_should_return_timeout_transport_error() {
                    let server = serve(ok);
                    crate::request::tests::request::send_blocking::send_blocking_zero_timeout_should_return_timeout_transport_error(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_stalled_body_should_return_timeout_transport_error_with_head() {
                    let server = serve_stalled();
                    crate::request::tests::request::send_blocking::send_blocking_stalled_body_should_return_timeout_transport_error_with_head(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_truncated_body_should_return_other_transport_error() {
                    let server = serve(truncated);
                    crate::request::tests::request::send_blocking::send_blocking_truncated_body_should_return_other_transport_error(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_streaming_should_pass_body_between_start_and_end() {
                    let server = serve(ok);
                    crate::request::tests::request::send_blocking::send_blocking_streaming_should_pass_body_between_start_and_end(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_with_retry_should_retry_until_success() {
                    let server = serve(unavailable_once);
                    crate::request::tests::request::send_blocking::send_blocking_with_retry_should_retry_until_success(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_with_retry_should_send_streamed_body_by_each_attempt() {
                    let server = serve(unavailable_once);
                    crate::request::tests::request::send_blocking::send_blocking_with_retry_should_send_streamed_body_by_each_attempt(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_blocking_with_retry_interrupted_body_should_be_retried() {
                    let server = serve(truncated);
                    crate::request::tests::request::send_blocking::send_blocking_with_retry_interrupted_body_should_be_retried(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_blocking_slow_body_should_time_out_each_read_not_whole_body() {
                    // GIVEN a client whose read timeout is shorter than the body takes
                    let server =
                        serve_chunks(Duration::from_millis(100), &["a", "b", "c", "d", "e", "f"]);
                    let request = RequestImpl::new();
                    let client = reqwest::blocking::Client::builder()
                        .timeout(Duration::from_millis(500))
                        .build()
                        .unwrap();

                    // WHEN receiving the body whole
                    let result = client.get(&server.url).send_blocking(&request);

                    // THEN it is not cut
                    assert_eq!(result.unwrap().body, b"abcdef");
                }
            }

            mod send_blocking_with_sink {
                use super::*;

                #[test]
                fn send_blocking_with_sink_failed_status_should_return_body_in_output() {
                    let server = serve(not_found);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_failed_status_should_return_body_in_output(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_with_sink_interrupted_body_should_keep_received_part() {
                    let server = serve(truncated);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_interrupted_body_should_keep_received_part(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_with_sink_interrupted_body_should_not_be_retried() {
                    let server = serve(truncated);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_interrupted_body_should_not_be_retried(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_blocking_with_sink_failure_before_body_should_be_retried() {
                    let server = serve(unavailable_once);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_failure_before_body_should_be_retried(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_blocking_with_sink_failing_sink_should_return_its_error_without_retrying() {
                    let server = serve(ok);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_failing_sink_should_return_its_error_without_retrying(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_blocking_with_sink_should_pass_head_then_body_to_sink() {
                    let server = serve(ok);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_should_pass_head_then_body_to_sink(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_blocking_with_sink_refused_head_should_return_its_error_without_retrying() {
                    let server = serve(ok);
                    crate::request::tests::request::send_blocking_with_sink::send_blocking_with_sink_refused_head_should_return_its_error_without_retrying(&RequestImpl::new(), &path(&server), &|| server.sent());
                }
            }

            #[cfg(feature = "tokio")]
            mod client {
                use super::*;

                #[test]
                fn client_invalid_builder_should_return_error_every_time() {
                    crate::request::tests::request::client::client_invalid_builder_should_return_error_every_time(&RequestImpl::new());
                }
            }

            #[cfg(feature = "tokio")]
            mod set_client {
                use super::*;

                #[test]
                fn set_client_should_replace_default_client() {
                    crate::request::tests::request::set_client::set_client_should_replace_default_client(&RequestImpl::new());
                }

                #[test]
                fn set_client_should_send_by_client_of_builder() {
                    let server = serve(ok);
                    let request = RequestImpl::new();
                    request.client().unwrap();

                    request.set_client(reqwest::Client::builder().user_agent("agplatform-test"));
                    block_on(get(&request, &server.url).send_async(&request)).unwrap();

                    assert!(server.received()[0].contains("user-agent: agplatform-test\r\n"));
                }
            }

            #[cfg(feature = "tokio")]
            mod send_async {
                use super::*;

                #[test]
                fn send_async_should_return_response_and_pass_run_to_interceptors() {
                    let server = serve(ok);
                    crate::request::tests::request::send_async::send_async_should_return_response_and_pass_run_to_interceptors(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_failed_status_should_return_error_with_output() {
                    let server = serve(not_found);
                    crate::request::tests::request::send_async::send_async_failed_status_should_return_error_with_output(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_invalid_request_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_async::send_async_invalid_request_should_return_error_bypassing_interceptors(&RequestImpl::new());
                }

                #[test]
                fn send_async_unreadable_body_should_return_error_bypassing_interceptors() {
                    crate::request::tests::request::send_async::send_async_unreadable_body_should_return_error_bypassing_interceptors(&RequestImpl::new(), &closed_url());
                }

                #[test]
                fn send_async_connect_failure_should_return_connect_transport_error() {
                    crate::request::tests::request::send_async::send_async_connect_failure_should_return_connect_transport_error(&RequestImpl::new(), &format!("{}/path", closed_url()));
                }

                #[test]
                fn send_async_timeout_should_return_timeout_transport_error() {
                    let server = serve(slow);
                    crate::request::tests::request::send_async::send_async_timeout_should_return_timeout_transport_error(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_zero_timeout_should_return_timeout_transport_error() {
                    let server = serve(ok);
                    crate::request::tests::request::send_async::send_async_zero_timeout_should_return_timeout_transport_error(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_stalled_body_should_return_timeout_transport_error_with_head() {
                    let server = serve_stalled();
                    crate::request::tests::request::send_async::send_async_stalled_body_should_return_timeout_transport_error_with_head(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_truncated_body_should_return_other_transport_error() {
                    let server = serve(truncated);
                    crate::request::tests::request::send_async::send_async_truncated_body_should_return_other_transport_error(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_with_retry_should_retry_until_success() {
                    let server = serve(unavailable_once);
                    crate::request::tests::request::send_async::send_async_with_retry_should_retry_until_success(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_async_with_retry_should_send_streamed_body_by_each_attempt() {
                    let server = serve(unavailable_once);
                    crate::request::tests::request::send_async::send_async_with_retry_should_send_streamed_body_by_each_attempt(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_async_cancelled_should_tell_interceptors() {
                    let server = serve(slow);
                    crate::request::tests::request::send_async::send_async_cancelled_should_tell_interceptors(&RequestImpl::new(), &path(&server));
                }
            }

            #[cfg(feature = "tokio")]
            mod send_async_with_sink {
                use super::*;

                #[test]
                fn send_async_with_sink_failed_status_should_return_body_in_output() {
                    let server = serve(not_found);
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_failed_status_should_return_body_in_output(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_with_sink_interrupted_body_should_keep_received_part() {
                    let server = serve(truncated);
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_interrupted_body_should_keep_received_part(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_with_sink_failure_before_body_should_be_retried() {
                    let server = serve(unavailable_once);
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_failure_before_body_should_be_retried(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_async_with_sink_failing_sink_should_return_its_error() {
                    let server = serve(ok);
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_failing_sink_should_return_its_error(&RequestImpl::new(), &path(&server), &|| server.sent());
                }

                #[test]
                fn send_async_with_sink_should_pass_head_then_body_to_sink() {
                    let server = serve(ok);
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_should_pass_head_then_body_to_sink(&RequestImpl::new(), &path(&server));
                }

                #[test]
                fn send_async_with_sink_refused_head_should_return_its_error_without_retrying() {
                    let server = serve(ok);
                    crate::request::tests::request::send_async_with_sink::send_async_with_sink_refused_head_should_return_its_error_without_retrying(&RequestImpl::new(), &path(&server), &|| server.sent());
                }
            }

            mod interceptors {
                use super::*;

                #[test]
                fn interceptors_should_be_none_by_default() {
                    crate::request::tests::request::interceptors::interceptors_should_be_none_by_default(&RequestImpl::new());
                }
            }

            mod clear_interceptors {
                use super::*;

                #[test]
                fn clear_interceptors_should_remove_interceptors_of_later_runs() {
                    let server = serve(ok);
                    crate::request::tests::request::clear_interceptors::clear_interceptors_should_remove_interceptors_of_later_runs(&RequestImpl::new(), &path(&server));
                }
            }

            mod add_interceptor {
                use super::*;

                #[test]
                fn add_interceptor_should_add_innermost_interceptor() {
                    let server = serve(ok);
                    crate::request::tests::request::add_interceptor::add_interceptor_should_add_innermost_interceptor(&RequestImpl::new(), &path(&server));
                }
            }
        }
    }
}
