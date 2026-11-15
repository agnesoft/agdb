//! A blocking client of a release service: reading releases with retries,
//! publishing them and following the progress of a large response.
//!
//! The client takes `&impl Request`, so its tests run against a
//! `TestRequest`. It returns the responses set per method and URL,
//! including failing statuses, broken connections and bodies arriving in
//! chunks, and then reports the requests sent with their headers and
//! bodies. Retries are recorded instead of waited for, so tests of a
//! backoff are instant.
//!
//! ```bash
//! cargo run --example request --features request -- http://example.com
//! ```

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use agplatform::BlockingRequestBuilderExt;
use agplatform::Env;
use agplatform::HttpRequest;
use agplatform::Platform;
use agplatform::Request;
use agplatform::TransportError;
use agplatform::TransportErrorKind;
use agplatform::interceptor::Chunk;
use agplatform::interceptor::Interceptor;
use agplatform::interceptor::Interceptors;
use agplatform::interceptor::Retry;

/// Returns the interceptors of `request` retried by `retry` around them.
fn retried(request: &impl Request, retry: Retry<HttpRequest>) -> Interceptors<HttpRequest> {
    let mut interceptors = request.interceptors();
    interceptors.insert(0, retry);
    interceptors
}

/// Returns the latest version, retrying transient failures (connection
/// errors, timeouts, 408, 429 and 5xx) with an exponential backoff.
fn latest_version(request: &impl Request, service: &str) -> agplatform::Result<String> {
    let output = request
        .blocking_client()?
        .get(format!("{service}/releases/latest"))
        .interceptors(retried(
            request,
            HttpRequest::retry(4, Duration::from_millis(100)).backoff(2),
        ))
        .send_blocking(request)?;
    Ok(output.text()?.trim().to_string())
}

/// Returns the notes of the release `version`, `None` when there is none.
/// Calls the request interface directly, which is equivalent to sending
/// the builder with `BlockingRequestBuilderExt` like the other functions.
fn release_notes(
    request: &impl Request,
    service: &str,
    version: &str,
) -> agplatform::Result<Option<String>> {
    let client = request.blocking_client()?;

    match request.send_blocking(client.get(format!("{service}/releases/{version}"))) {
        Ok(output) => Ok(Some(output.text()?.to_string())),
        Err(error)
            if error
                .request_output()
                .is_some_and(|output| output.status == 404) =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

/// Publishes a release. Only the attempts that could not connect are
/// retried, as the service may have received any other one.
fn publish(
    request: &impl Request,
    service: &str,
    token: &str,
    notes: &str,
) -> agplatform::Result<()> {
    request
        .blocking_client()?
        .post(format!("{service}/releases"))
        .bearer_auth(token)
        .body(notes.to_string())
        .interceptors(retried(
            request,
            HttpRequest::retry(3, Duration::from_secs(1)).when(|_, result| {
                result
                    .as_ref()
                    .err()
                    .and_then(|error| error.source_as::<TransportError>())
                    .is_some_and(|error| error.kind() == TransportErrorKind::Connect)
            }),
        ))
        .send_blocking(request)?;
    Ok(())
}

/// Returns the changelog of all releases, showing the progress of its body
/// by the interceptors of `request` as it arrives.
fn changelog(request: &impl Request, service: &str) -> agplatform::Result<String> {
    let output = request
        .blocking_client()?
        .get(format!("{service}/changelog"))
        .with(ShowProgress)
        .send_blocking(request)?;
    Ok(output.text_lossy().into_owned())
}

/// Marks the requests whose progress [`Progress`] shows.
struct ShowProgress;

/// Counts the bytes of the response bodies of the requests marked with
/// [`ShowProgress`].
#[derive(Clone, Default)]
struct Progress(Arc<AtomicUsize>);

impl Interceptor<HttpRequest> for Progress {
    fn chunk(&self, chunk: Chunk<HttpRequest>) -> agplatform::Result {
        let request = chunk.run().subject();

        if chunk.run().get::<ShowProgress>().is_some() {
            let data = chunk.data().len();
            let received = self.0.fetch_add(data, Ordering::Relaxed) + data;
            eprintln!("{}: {received} bytes", request.target());
        }

        chunk.forward()
    }
}

fn main() -> agplatform::Result<()> {
    let platform = agplatform::platform();
    platform.request().add_interceptor(Progress::default());
    let service = platform
        .env()
        .args()
        .nth(1)
        .map_or("http://localhost:8080".to_string(), |arg| {
            arg.to_string_lossy().into_owned()
        });
    let request = platform.request();

    let version = latest_version(request, &service)?;
    println!("latest: {version}");
    println!("notes: {:?}", release_notes(request, &service, &version)?);
    println!("changelog: {}", changelog(request, &service)?);
    if let Some(token) = platform.env().var("RELEASE_TOKEN") {
        let token = token
            .into_string()
            .map_err(|_| agplatform::Error::env("RELEASE_TOKEN is not valid UTF-8"))?;
        publish(request, &service, &token, "Released from the example")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::RequestOutput;
    use agplatform::TestRequest;

    const SERVICE: &str = "https://releases.example.com";

    fn status(status: u16, body: &str) -> RequestOutput {
        RequestOutput {
            status,
            body: body.as_bytes().to_vec(),
            ..Default::default()
        }
    }

    mod latest_version {
        use super::*;

        #[test]
        fn latest_version_should_retry_transient_failures() {
            // GIVEN a service that is unavailable and then overloaded
            let request = TestRequest::new();
            request.set_responses(
                "/releases/latest",
                [status(503, ""), status(429, ""), status(200, "1.2.0\n")],
            );

            // WHEN asking for the latest version
            let version = latest_version(&request, SERVICE).unwrap();

            // THEN it is returned by the third attempt after the backoff
            assert_eq!(version, "1.2.0");
            assert_eq!(request.calls().len(), 3);
            assert_eq!(
                request.sleeps(),
                [Duration::from_millis(100), Duration::from_millis(200)]
            );
        }

        #[test]
        fn latest_version_unreachable_should_fail_with_transport_error() {
            let request = TestRequest::new();
            request.set_transport_error("", TransportErrorKind::Connect);

            let error = latest_version(&request, SERVICE).unwrap_err();

            assert_eq!(
                error.source_as::<TransportError>().unwrap().kind(),
                TransportErrorKind::Connect
            );
            assert_eq!(request.calls().len(), 5);
        }
    }

    mod release_notes {
        use super::*;

        #[test]
        fn release_notes_should_return_body() {
            let request = TestRequest::new();
            request.set_response(
                "GET https://releases.example.com/releases/1.2.0",
                status(200, "Faster"),
            );

            let notes = release_notes(&request, SERVICE, "1.2.0").unwrap();

            assert_eq!(notes.as_deref(), Some("Faster"));
        }

        #[test]
        fn release_notes_unknown_version_should_return_none() {
            let request = TestRequest::new();
            request.set_response("/releases/", status(404, "no such release"));

            let notes = release_notes(&request, SERVICE, "9.9.9").unwrap();

            assert_eq!(notes, None);
        }

        #[test]
        fn release_notes_server_error_should_fail_with_response() {
            let request = TestRequest::new();
            request.set_response("/releases/", status(500, "database down"));

            let error = release_notes(&request, SERVICE, "1.2.0").unwrap_err();

            assert_eq!(
                error.request_output().unwrap().text().unwrap(),
                "database down"
            );
        }
    }

    mod publish {
        use super::*;

        #[test]
        fn publish_should_post_notes_with_token() {
            let request = TestRequest::new();
            request.set_response("POST", status(201, ""));

            publish(&request, SERVICE, "secret", "Faster").unwrap();

            let call = &request.call_details()[0];
            assert_eq!(call.url, "https://releases.example.com/releases");
            assert_eq!(call.headers["authorization"], "Bearer secret");
            assert_eq!(call.body, b"Faster");
        }

        #[test]
        fn publish_should_retry_only_failed_connections() {
            for (kind, attempts) in [
                (TransportErrorKind::Connect, 4),
                (TransportErrorKind::Timeout, 1),
                (TransportErrorKind::Other, 1),
            ] {
                let request = TestRequest::new();
                request.set_transport_error("POST", kind);

                let result = publish(&request, SERVICE, "secret", "Faster");

                assert!(result.is_err(), "kind: {kind}");
                assert_eq!(request.calls().len(), attempts, "kind: {kind}");
            }
        }
    }

    mod changelog {
        use super::*;

        #[test]
        fn changelog_should_show_progress_of_body_by_interceptors() {
            // GIVEN a changelog arriving in two chunks and an interceptor
            // counting the progress
            let progress = Progress::default();
            let request = TestRequest::new();
            request.add_interceptor(progress.clone());
            request.set_chunks(
                "/changelog",
                status(200, ""),
                [b"1.1.0\n".to_vec(), b"1.2.0\n".to_vec()],
            );

            // WHEN reading the changelog
            let changelog = changelog(&request, SERVICE).unwrap();

            // THEN the interceptor saw the whole body as it arrived
            assert_eq!(changelog, "1.1.0\n1.2.0\n");
            assert_eq!(progress.0.load(Ordering::Relaxed), 12);
        }
    }

    mod progress {
        mod interceptor {
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
        }
    }
}
