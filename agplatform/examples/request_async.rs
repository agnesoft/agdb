//! Async requests on a tokio runtime: fetching pages concurrently,
//! cancelling a request that misses its deadline, and streaming a large
//! body through a sink without holding it in memory.
//!
//! The tests share a `TestPlatform` with the spawned tasks. A response set
//! with `TestRequest::set_pending` is held until its `TestGate` is
//! released, so a test can check what happens while a request is in
//! flight: that other requests are sent meanwhile, or that it is
//! cancelled when it takes too long.
//!
//! ```bash
//! cargo run --example request_async --features request,tokio -- http://example.com http://example.org
//! ```

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use agplatform::Env;
use agplatform::HttpRequest;
use agplatform::Platform;
use agplatform::Request;
use agplatform::RequestBuilderExt;
use agplatform::interceptor::Interceptor;
use agplatform::interceptor::Run;
use tokio::task::JoinSet;

/// Fetches all `urls` concurrently, retrying their transient failures, and
/// returns their bodies in their order.
async fn fetch_all<P: Platform + 'static>(
    platform: &Arc<P>,
    urls: Vec<String>,
) -> agplatform::Result<Vec<agplatform::Result<String>>> {
    let client = platform.request().client()?;
    let mut interceptors = platform.request().interceptors();
    interceptors.insert(0, HttpRequest::retry(2, Duration::from_millis(100)));
    let mut tasks = JoinSet::new();

    for (index, url) in urls.into_iter().enumerate() {
        let (platform, client, interceptors) =
            (platform.clone(), client.clone(), interceptors.clone());
        tasks.spawn(async move {
            let output = client
                .get(url)
                .interceptors(interceptors)
                .send_async(platform.request())
                .await;
            (
                index,
                output.and_then(|output| Ok(output.text()?.to_string())),
            )
        });
    }

    let mut bodies = tasks.join_all().await;
    bodies.sort_by_key(|(index, _)| *index);
    Ok(bodies.into_iter().map(|(_, body)| body).collect())
}

/// Fetches `url` unless it takes longer than `deadline`, when the request
/// is cancelled and `None` returned.
async fn fetch_within(
    platform: &impl Platform,
    url: &str,
    deadline: Duration,
) -> agplatform::Result<Option<String>> {
    let request = platform.request();
    let sent = request.send_async(request.client()?.get(url));

    match tokio::time::timeout(deadline, sent).await {
        Ok(output) => Ok(Some(output?.text()?.to_string())),
        Err(_) => Ok(None),
    }
}

/// Counts the lines of the body of `url` as it arrives.
async fn count_lines(platform: &impl Platform, url: &str) -> agplatform::Result<usize> {
    let request = platform.request();
    let mut lines = 0;

    request
        .client()?
        .get(url)
        .send_async_with_sink(request, &mut |(), chunk: &[u8]| {
            lines += chunk.iter().filter(|&&byte| byte == b'\n').count();
            Ok(())
        })
        .await?;

    Ok(lines)
}

/// Records the requests cancelled by dropping their futures.
#[derive(Clone, Default)]
struct Cancellations(Arc<Mutex<Vec<String>>>);

impl Interceptor<HttpRequest> for Cancellations {
    fn cancelled(&self, run: &Run<HttpRequest>) {
        self.0.lock().unwrap().push(run.subject().target());
    }
}

fn main() -> agplatform::Result<()> {
    let cancellations = Cancellations::default();
    let platform = agplatform::platform();
    platform.request().add_interceptor(cancellations.clone());
    let platform = Arc::new(platform);
    let urls: Vec<String> = platform
        .env()
        .args()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime should start");

    runtime.block_on(async {
        for (url, body) in urls.iter().zip(fetch_all(&platform, urls.clone()).await?) {
            match body {
                Ok(body) => println!("{url}: {} bytes", body.len()),
                Err(error) => println!("{url}: {error}"),
            }
        }
        for url in &urls {
            println!(
                "{url}: {} lines",
                count_lines(platform.as_ref(), url).await?
            );
            let page = fetch_within(platform.as_ref(), url, Duration::from_secs(1)).await?;
            println!("{url}: fetched within a second: {}", page.is_some());
        }
        println!("cancelled: {:?}", cancellations.0.lock().unwrap());
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::RequestOutput;
    use agplatform::test_platform::test_platform;

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    fn ok(body: &str) -> RequestOutput {
        RequestOutput {
            status: 200,
            body: body.as_bytes().to_vec(),
            ..Default::default()
        }
    }

    mod fetch_all {
        use super::*;

        #[test]
        fn fetch_all_should_send_requests_concurrently() {
            // GIVEN a slow page held until released and a fast one
            let platform = Arc::new(test_platform());
            let slow = platform.request.set_pending("/slow", ok("slow"));
            platform.request.set_response("/fast", ok("fast"));
            let urls = vec![
                "https://a.com/slow".to_string(),
                "https://b.com/fast".to_string(),
            ];

            let bodies = block_on(async {
                // WHEN fetching them
                let fetching = tokio::spawn({
                    let platform = platform.clone();
                    async move { fetch_all(&platform, urls).await }
                });

                // THEN the fast one is sent while the slow one is in flight
                tokio::time::timeout(Duration::from_secs(10), async {
                    while platform.request.calls().len() < 2 {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .expect("both requests should be sent");

                // AND the bodies are in the order of the urls once the slow
                // one is answered
                slow.release();
                fetching.await.unwrap().unwrap()
            });

            assert_eq!(bodies[0].as_deref().unwrap(), "slow");
            assert_eq!(bodies[1].as_deref().unwrap(), "fast");
        }

        #[test]
        fn fetch_all_should_return_failure_of_each_url() {
            let platform = Arc::new(test_platform());
            platform.request.set_response("", ok("page"));
            platform.request.set_response(
                "/missing",
                RequestOutput {
                    status: 404,
                    ..ok("")
                },
            );
            let urls = vec![
                "https://a.com/missing".to_string(),
                "https://a.com/".to_string(),
            ];

            let bodies = block_on(fetch_all(&platform, urls)).unwrap();

            assert_eq!(
                bodies[0]
                    .as_ref()
                    .unwrap_err()
                    .request_output()
                    .unwrap()
                    .status,
                404
            );
            assert_eq!(bodies[1].as_deref().unwrap(), "page");
        }

        #[test]
        fn fetch_all_client_failure_should_fail() {
            let platform = Arc::new(test_platform());
            platform.request.set_client_failure(true);

            let result = block_on(fetch_all(&platform, vec!["https://a.com/".to_string()]));

            assert!(result.is_err());
            assert!(platform.request.calls().is_empty());
        }
    }

    mod fetch_within {
        use super::*;

        #[test]
        fn fetch_within_should_return_body_received_in_time() {
            let platform = test_platform();
            platform.request.set_response("", ok("page"));

            let body = block_on(fetch_within(
                &platform,
                "https://a.com/",
                Duration::from_secs(10),
            ));

            assert_eq!(body.unwrap().as_deref(), Some("page"));
        }

        #[test]
        fn fetch_within_missed_deadline_should_cancel_request() {
            // GIVEN a page that is never answered and an interceptor of the
            // cancellations
            let cancellations = Cancellations::default();
            let platform = test_platform();
            platform.request().add_interceptor(cancellations.clone());
            let _gate = platform.request.set_pending("", ok("page"));

            // WHEN fetching it with a deadline
            let body = block_on(fetch_within(
                &platform,
                "https://a.com/",
                Duration::from_millis(10),
            ));

            // THEN the request is cancelled when the deadline passes
            assert_eq!(body.unwrap(), None);
            assert_eq!(*cancellations.0.lock().unwrap(), ["GET https://a.com/"]);
        }
    }

    mod count_lines {
        use super::*;

        #[test]
        fn count_lines_should_count_across_chunks() {
            let platform = test_platform();
            platform.request.set_chunks(
                "/log",
                ok(""),
                [b"a\nb".to_vec(), b"\nc\n".to_vec(), b"d".to_vec()],
            );

            let lines = block_on(count_lines(&platform, "https://a.com/log")).unwrap();

            assert_eq!(lines, 3);
        }
    }

    mod cancellations {
        use super::*;

        mod interceptor {
            use super::*;

            mod handle {
                #[test]
                fn handle_invalid_run_should_fail_in_one_attempt() {
                    // WHY: an example cannot call the contract cases, which are test
                    // code of agplatform.
                }

                #[test]
                fn handle_replayed_run_should_return_replay_output_once() {
                    // WHY: an example cannot call the contract cases, which are test
                    // code of agplatform.
                }
            }

            mod chunk {
                #[test]
                fn chunk_should_pass_chunk_on_unchanged() {
                    // WHY: an example cannot call the contract cases, which are test
                    // code of agplatform.
                }
            }

            mod cancelled {
                use super::*;

                #[test]
                fn cancelled_should_not_be_called_for_requests_not_cancelled() {
                    let cancellations = Cancellations::default();
                    let platform = test_platform();
                    platform.request().add_interceptor(cancellations.clone());
                    platform.request.set_response("/ok", ok(""));
                    platform.request.set_response(
                        "/error",
                        RequestOutput {
                            status: 500,
                            ..ok("")
                        },
                    );

                    let _ = block_on(fetch_within(
                        &platform,
                        "https://a.com/ok",
                        Duration::from_secs(10),
                    ));
                    let _ = block_on(fetch_within(
                        &platform,
                        "https://a.com/error",
                        Duration::from_secs(10),
                    ));

                    assert!(cancellations.0.lock().unwrap().is_empty());
                }
            }
        }
    }
}
