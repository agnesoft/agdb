use crate::interceptor::Cache;

use super::HttpRequest;

/// Enabled by the `request` feature flag.
///
/// The [`Cache`] of the successful responses of the `GET` and `HEAD`
/// requests by their [`HttpRequest::cache_key`], the
/// [default](Self::default) one, or by another key with [`Cache::new`].
/// The key has no headers, so a response is returned whatever the
/// credentials or the content negotiation (e.g. `Accept`) of the request,
/// use a cache per credentials, or a key with the headers the responses
/// depend on, when they do.
///
/// # Examples
///
/// ```rust
/// use agplatform::{BlockingRequestBuilderExt, Request, RequestCache, RequestOutput, TestRequest};
///
/// let cache = RequestCache::default();
/// let request = TestRequest::new();
/// request.add_interceptor(cache.clone());
/// request.set_response("", RequestOutput { status: 200, body: b"manifest".to_vec(), ..Default::default() });
/// let client = request.blocking_client().unwrap();
///
/// client.get("https://registry.example.com/v2/app/manifests/latest").send_blocking(&request).unwrap();
/// let output = client.get("https://registry.example.com/v2/app/manifests/latest").send_blocking(&request).unwrap();
///
/// assert_eq!(output.body, b"manifest");
/// assert_eq!(request.calls().len(), 1);
/// assert_eq!(cache.entries().len(), 1);
/// ```
pub type RequestCache = Cache<HttpRequest>;

impl Default for Cache<HttpRequest> {
    /// Returns the cache of the requests by their
    /// [`HttpRequest::cache_key`].
    fn default() -> Self {
        Self::new(|run| run.subject().cache_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestRequest;
    use crate::request::BlockingRequestBuilderExt;
    use crate::request::Request;
    use crate::request::tests::*;

    mod default {
        use super::*;

        #[test]
        fn default_should_cache_get_and_head_by_cache_key() {
            let cache = RequestCache::default();
            let request = TestRequest::new();
            request.add_interceptor(cache.clone());
            request.set_response("", output(200, b"body"));
            let client = request.blocking_client().unwrap();

            for _ in 0..2 {
                client.get("http://h/a").send_blocking(&request).unwrap();
                client.post("http://h/a").send_blocking(&request).unwrap();
            }

            assert_eq!(request.calls().len(), 3);
            assert_eq!(cache.entries()[0].0, "GET http://h/a");
        }

        #[test]
        fn default_hit_with_sink_should_pass_cached_body_to_sink() {
            // GIVEN a response cached
            let cache = RequestCache::default();
            let request = TestRequest::new();
            request.add_interceptor(cache.clone());
            request.set_response("", output(200, b"body"));
            let client = request.blocking_client().unwrap();
            client.get("http://h/a").send_blocking(&request).unwrap();
            let mut received = Vec::new();

            // WHEN sending the request again to a sink
            let output = client
                .get("http://h/a")
                .send_blocking_with_sink(&request, &mut collect_into(&mut received))
                .unwrap();

            // THEN the sink gets the cached body without a call
            assert_eq!(received, b"body");
            assert!(output.body.is_empty());
            assert_eq!(request.calls().len(), 1);
        }
    }
}
