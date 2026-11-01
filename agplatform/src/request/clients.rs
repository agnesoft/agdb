use std::ffi::OsString;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;

use crate::Result;
use crate::env::EnvImpl;

/// [internal]
///
/// A client built on the first use from the builder set, or from the
/// default one of the [`ReqwestEnv`], which is built again when the env
/// differs from the one it was built with.
pub(crate) struct DefaultClient<B, C>(Mutex<ClientState<B, C>>);

/// [internal]
struct ClientState<B, C> {
    builder: Option<B>,
    client: Option<Result<C>>,
    /// The env the default client was built with.
    env: ReqwestEnv,
    custom: bool,
}

/// [internal]
///
/// The default clients of a request with the proxies of the
/// [`REQWEST_ENV_VARS`] of its env, built again when they change.
pub(crate) struct DefaultClients {
    pub(crate) env: EnvImpl,
    /// The variables taken from `env` with the generation of its variables
    /// they were taken at.
    reqwest_env: Mutex<Option<(u64, ReqwestEnv)>>,
    blocking: DefaultClient<reqwest::blocking::ClientBuilder, reqwest::blocking::Client>,
    #[cfg(feature = "tokio")]
    client: DefaultClient<reqwest::ClientBuilder, reqwest::Client>,
}

/// [internal]
///
/// The variables of the platform environment that reqwest reads, see
/// [`REQWEST_ENV_VARS`]. Their names ignore the ASCII case on Windows like
/// its environment, e.g. `https_proxy` is `HTTPS_PROXY` there.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReqwestEnv(Vec<(String, String)>);

/// The timeout of connecting of the clients from
/// [`Request::client_builder`](crate::Request::client_builder) and [`Request::blocking_client_builder`](crate::Request::blocking_client_builder).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// The timeout of each read of a response of the clients from
/// [`Request::client_builder`](crate::Request::client_builder) and [`Request::blocking_client_builder`](crate::Request::blocking_client_builder).
const READ_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// The environment variables reqwest reads, see [`Request`](crate::Request).
const REQWEST_ENV_VARS: [&str; 9] = [
    "ALL_PROXY",
    "all_proxy",
    "HTTP_PROXY",
    "http_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "NO_PROXY",
    "no_proxy",
    "REQUEST_METHOD",
];

/// Returns whether `key` is one of the [`REQWEST_ENV_VARS`].
fn is_reqwest_var(key: &str) -> bool {
    REQWEST_ENV_VARS.iter().any(|name| same_var(name, key))
}

/// Returns whether the variable names `a` and `b` are the same, ignoring
/// the ASCII case like the environment of Windows.
#[cfg(windows)]
fn same_var(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Returns whether the variable names `a` and `b` are the same.
#[cfg(not(windows))]
fn same_var(a: &str, b: &str) -> bool {
    a == b
}

/// Returns whether hyper-util's `Matcher::from_env` accepts the proxy
/// `url` of a variable, see its `parse_env_uri`.
///
/// WHY: reqwest parses a proxy given to it more leniently, e.g. trimming
/// spaces, and passes it to hyper-util normalized.
fn is_env_proxy(url: &str) -> bool {
    let Ok(uri) = url.parse::<http::Uri>() else {
        return false;
    };

    matches!(
        uri.scheme_str(),
        None | Some("http" | "https" | "socks4" | "socks4a" | "socks5" | "socks5h")
    ) && uri.authority().is_some_and(|authority| {
        let host_port = authority
            .as_str()
            .split_once('@')
            .map_or(authority.as_str(), |(_, host_port)| host_port);
        http::uri::Authority::try_from(host_port).is_ok()
    })
}

/// Returns the error of a client failing to be built.
fn client_error(error: reqwest::Error) -> crate::Error {
    request_error!("client could not be built: {error}").with_source(error)
}

fn build_blocking_client(
    builder: reqwest::blocking::ClientBuilder,
) -> Result<reqwest::blocking::Client> {
    builder.build().map_err(client_error)
}

#[cfg(feature = "tokio")]
fn build_client(builder: reqwest::ClientBuilder) -> Result<reqwest::Client> {
    builder.build().map_err(client_error)
}

impl<B, C: Clone> DefaultClient<B, C> {
    pub(crate) fn new() -> Self {
        Self(Mutex::new(ClientState {
            builder: None,
            client: None,
            env: ReqwestEnv::default(),
            custom: false,
        }))
    }

    /// Returns the client, built with `build` from the builder set on the
    /// first call, or from the `default` builder of `env` when not built
    /// with `env` yet. Fails when the builder set was lost by a build that
    /// panicked rather than building the default one.
    ///
    /// WHY: compared with `env` on each call, rather than reset when it
    /// changes, so that a client built with an env taken before a change
    /// is not kept after it.
    pub(crate) fn get(
        &self,
        env: &ReqwestEnv,
        default: &dyn Fn(&ReqwestEnv) -> B,
        build: fn(B) -> Result<C>,
    ) -> Result<C> {
        let mut state = self.state();

        if let Some(client) = &state.client
            && (state.custom || state.env == *env)
        {
            return client.clone();
        }

        let client = match state.builder.take() {
            Some(builder) => build(builder),
            None if state.custom => Err(request_error!(
                "client could not be built: its builder was lost by a panic"
            )),
            None => {
                state.env = env.clone();
                build(default(env))
            }
        };
        state.client = Some(client.clone());
        client
    }

    /// Replaces the client by one to be built from `builder`.
    pub(crate) fn set(&self, builder: B) {
        *self.state() = ClientState {
            builder: Some(builder),
            client: None,
            env: ReqwestEnv::default(),
            custom: true,
        };
    }

    fn state(&self) -> std::sync::MutexGuard<'_, ClientState<B, C>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl DefaultClients {
    pub(crate) fn new(env: EnvImpl) -> Self {
        Self {
            env,
            reqwest_env: Mutex::new(None),
            blocking: DefaultClient::new(),
            #[cfg(feature = "tokio")]
            client: DefaultClient::new(),
        }
    }

    /// Takes the variables from `env` instead, e.g. of a
    /// [`TestPlatform`](crate::test_platform::TestPlatform), for the
    /// default clients to be built with.
    #[cfg(feature = "testing")]
    pub(crate) fn set_env(&mut self, env: EnvImpl) {
        self.env = env;
        *self
            .reqwest_env
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner) = None;
    }

    /// Returns the [`REQWEST_ENV_VARS`] of the env, taken again when its
    /// variables changed since last taken.
    pub(crate) fn reqwest_env(&self) -> ReqwestEnv {
        self.env.with_vars(|generation, vars| {
            let mut taken = self
                .reqwest_env
                .lock()
                .unwrap_or_else(PoisonError::into_inner);

            match &*taken {
                Some((taken_generation, reqwest_env)) if *taken_generation == generation => {
                    reqwest_env.clone()
                }
                _ => {
                    let reqwest_env = ReqwestEnv::new(vars);
                    *taken = Some((generation, reqwest_env.clone()));
                    reqwest_env
                }
            }
        })
    }

    pub(crate) fn blocking_client(&self) -> Result<reqwest::blocking::Client> {
        self.blocking.get(
            &self.reqwest_env(),
            &ReqwestEnv::blocking_client_builder,
            build_blocking_client,
        )
    }

    pub(crate) fn blocking_client_builder(&self) -> reqwest::blocking::ClientBuilder {
        self.reqwest_env().blocking_client_builder()
    }

    pub(crate) fn set_blocking_client(&self, builder: reqwest::blocking::ClientBuilder) {
        self.blocking.set(builder);
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn client(&self) -> Result<reqwest::Client> {
        self.client.get(
            &self.reqwest_env(),
            &ReqwestEnv::client_builder,
            build_client,
        )
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn client_builder(&self) -> reqwest::ClientBuilder {
        self.reqwest_env().client_builder()
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn set_client(&self, builder: reqwest::ClientBuilder) {
        self.client.set(builder);
    }
}

impl ReqwestEnv {
    /// Returns the [`REQWEST_ENV_VARS`] of `vars`, those not valid UTF-8
    /// ignored like reqwest does, except `REQUEST_METHOD`, which only needs
    /// to be set.
    pub(crate) fn new<'a>(vars: impl IntoIterator<Item = &'a (OsString, OsString)>) -> Self {
        Self(
            vars.into_iter()
                .filter_map(|(key, value)| {
                    let key = key.to_str().filter(|key| is_reqwest_var(key))?;
                    let value = match value.to_str() {
                        Some(value) => value.to_string(),
                        None => same_var(key, "REQUEST_METHOD")
                            .then(|| value.to_string_lossy().into_owned())?,
                    };
                    Some((key.to_string(), value))
                })
                .collect(),
        )
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn client_builder(&self) -> reqwest::ClientBuilder {
        self.client_builder_with(CONNECT_TIMEOUT, READ_TIMEOUT)
    }

    pub(crate) fn blocking_client_builder(&self) -> reqwest::blocking::ClientBuilder {
        self.blocking_client_builder_with(CONNECT_TIMEOUT, READ_TIMEOUT)
    }

    /// Returns a client builder with the proxies and the `connect` and
    /// `read` timeouts.
    #[cfg(feature = "tokio")]
    fn client_builder_with(&self, connect: Duration, read: Duration) -> reqwest::ClientBuilder {
        self.proxies().into_iter().fold(
            reqwest::Client::builder()
                .connect_timeout(connect)
                .read_timeout(read)
                .no_proxy(),
            reqwest::ClientBuilder::proxy,
        )
    }

    /// Returns a blocking client builder with the proxies and the `connect`
    /// timeout. Its `read` timeout replaces the total one of 30 seconds
    /// reqwest sets by default. It limits each read of the body, and the
    /// wait for the response head, since the body of a blocking response
    /// is read without the runtime the read timeout of reqwest needs.
    fn blocking_client_builder_with(
        &self,
        connect: Duration,
        read: Duration,
    ) -> reqwest::blocking::ClientBuilder {
        self.proxies().into_iter().fold(
            reqwest::blocking::Client::builder()
                .connect_timeout(connect)
                .timeout(read)
                .no_proxy(),
            reqwest::blocking::ClientBuilder::proxy,
        )
    }

    /// Returns the proxies resolved like hyper-util's `Matcher::from_env`,
    /// which reqwest uses for the process environment: the first set of
    /// the uppercase and lowercase variable, a scheme specific proxy that
    /// is not valid falling back to `ALL_PROXY`, invalid ones ignored. Valid
    /// are those both hyper-util and reqwest accept, see [`is_env_proxy`].
    fn proxies(&self) -> Vec<reqwest::Proxy> {
        if self.var(&["REQUEST_METHOD"]).is_some() {
            return Vec::new();
        }

        let all = self.var(&["ALL_PROXY", "all_proxy"]);
        let no_proxy = self
            .var(&["NO_PROXY", "no_proxy"])
            .and_then(reqwest::NoProxy::from_string);
        let resolve = |names: &[&str], proxy: fn(&str) -> reqwest::Result<reqwest::Proxy>| {
            let parse = |url: &str| is_env_proxy(url).then(|| proxy(url).ok()).flatten();

            self.var(names)
                .and_then(parse)
                .or_else(|| all.and_then(parse))
        };

        [
            resolve(&["HTTP_PROXY", "http_proxy"], |url| {
                reqwest::Proxy::http(url)
            }),
            resolve(&["HTTPS_PROXY", "https_proxy"], |url| {
                reqwest::Proxy::https(url)
            }),
        ]
        .into_iter()
        .flatten()
        .map(|proxy| proxy.no_proxy(no_proxy.clone()))
        .collect()
    }

    fn var(&self, names: &[&str]) -> Option<&str> {
        names.iter().find_map(|name| {
            self.0
                .iter()
                .find(|(key, _)| same_var(key, name))
                .map(|(_, value)| value.as_str())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockingRequestBuilderExt;
    use crate::Request;
    #[cfg(feature = "tokio")]
    use crate::RequestBuilderExt;
    use crate::TransportError;
    use crate::TransportErrorKind;
    use crate::request::RequestImpl;
    use crate::request::tests::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    mod default_clients {
        use super::*;

        mod reqwest_env {
            use super::*;

            #[test]
            fn reqwest_env_changed_proxy_should_rebuild_default_clients() {
                // GIVEN built default clients
                let proxy = serve(ok);
                let request = RequestImpl::new();
                request.blocking_client().unwrap();
                #[cfg(feature = "tokio")]
                request.client().unwrap();

                // WHEN setting a proxy and an unrelated variable
                request.set_var("HTTP_PROXY", &proxy.url);
                request.set_var("PATH", "/bin");

                // THEN the requests of both go through the proxy
                blocking_get(&request, "http://example.invalid/")
                    .send_blocking(&request)
                    .unwrap();
                #[cfg(feature = "tokio")]
                block_on(get(&request, "http://example.invalid/").send_async(&request)).unwrap();
                #[cfg(feature = "tokio")]
                assert_eq!(proxy.received().len(), 2);
            }

            #[test]
            fn reqwest_env_changed_proxy_should_keep_clients_set() {
                let request = RequestImpl::new();
                request.set_blocking_client(invalid_client());
                #[cfg(feature = "tokio")]
                request.set_client(invalid_async_client());

                request.set_var("HTTP_PROXY", "http://proxy");

                assert!(request.blocking_client().is_err());
                #[cfg(feature = "tokio")]
                assert!(request.client().is_err());
            }

            #[test]
            fn reqwest_env_removed_proxy_should_send_directly() {
                // GIVEN a default client using a proxy
                let proxy = serve(ok);
                let server = serve(ok);
                let env = crate::env::EnvImpl::new(Vec::new(), std::path::PathBuf::new());
                crate::Env::set_var(&env, "HTTP_PROXY", proxy.url.as_str()).unwrap();
                let request = RequestImpl::with_env(env.clone());
                request.blocking_client().unwrap();

                // WHEN removing the proxy
                request.remove_var("HTTP_PROXY");

                // THEN requests go directly
                blocking_get(&request, &server.url)
                    .send_blocking(&request)
                    .unwrap();
                assert!(proxy.received().is_empty());
                assert_eq!(server.received().len(), 1);
            }
        }
    }

    mod default_client {
        use super::*;

        fn default_builder(_: &ReqwestEnv) -> reqwest::blocking::ClientBuilder {
            reqwest::blocking::Client::builder()
        }

        fn invalid_default(_: &ReqwestEnv) -> reqwest::blocking::ClientBuilder {
            invalid_client()
        }

        fn proxied() -> ReqwestEnv {
            ReqwestEnv::new(&[(OsString::from("HTTP_PROXY"), OsString::from("http://proxy"))])
        }

        mod get {
            use super::*;

            #[test]
            fn get_should_build_client_once() {
                static BUILDS: AtomicUsize = AtomicUsize::new(0);
                fn build(
                    builder: reqwest::blocking::ClientBuilder,
                ) -> Result<reqwest::blocking::Client> {
                    BUILDS.fetch_add(1, Ordering::Relaxed);
                    build_blocking_client(builder)
                }
                let client = DefaultClient::new();

                client
                    .get(&ReqwestEnv::default(), &default_builder, build)
                    .unwrap();
                client
                    .get(&ReqwestEnv::default(), &default_builder, build)
                    .unwrap();

                assert_eq!(BUILDS.load(Ordering::Relaxed), 1);
            }

            #[test]
            fn get_should_build_default_builder() {
                let client = DefaultClient::new();

                let result = client.get(
                    &ReqwestEnv::default(),
                    &invalid_default,
                    build_blocking_client,
                );

                assert!(result.is_err());
            }

            #[test]
            fn get_failed_build_should_return_error_every_time() {
                let client = DefaultClient::new();
                client.set(invalid_client());

                let first = client
                    .get(
                        &ReqwestEnv::default(),
                        &default_builder,
                        build_blocking_client,
                    )
                    .unwrap_err();
                let second = client
                    .get(
                        &ReqwestEnv::default(),
                        &default_builder,
                        build_blocking_client,
                    )
                    .unwrap_err();

                assert!(
                    first
                        .description()
                        .starts_with("client could not be built: ")
                );
                assert_eq!(first.description(), second.description());
            }

            #[test]
            fn get_after_panicking_build_of_set_builder_should_return_error() {
                // GIVEN a builder set whose build panicked
                fn panicking(
                    _: reqwest::blocking::ClientBuilder,
                ) -> Result<reqwest::blocking::Client> {
                    panic!("build failed")
                }
                let client = DefaultClient::new();
                client.set(reqwest::blocking::Client::builder());
                let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    client.get(&ReqwestEnv::default(), &default_builder, panicking)
                }));

                // WHEN getting the client again
                let result = client.get(
                    &ReqwestEnv::default(),
                    &default_builder,
                    build_blocking_client,
                );

                // THEN it fails instead of building the default builder
                assert!(panicked.is_err());
                assert_eq!(
                    result.unwrap_err().description(),
                    "client could not be built: its builder was lost by a panic"
                );
            }

            #[test]
            fn get_other_env_should_build_default_builder_again() {
                // GIVEN a client built from the default builder
                let client = DefaultClient::new();
                client
                    .get(
                        &ReqwestEnv::default(),
                        &default_builder,
                        build_blocking_client,
                    )
                    .unwrap();

                // WHEN getting it with another env
                let result = client.get(&proxied(), &invalid_default, build_blocking_client);

                // THEN the default builder is built again
                assert!(result.is_err());
            }

            #[test]
            fn get_earlier_env_after_later_should_build_later_again() {
                // GIVEN a client built with an env taken before a change
                let client = DefaultClient::new();
                client
                    .get(&proxied(), &default_builder, build_blocking_client)
                    .unwrap();
                client
                    .get(
                        &ReqwestEnv::default(),
                        &default_builder,
                        build_blocking_client,
                    )
                    .unwrap();

                // WHEN getting it with the env after the change
                let result = client.get(&proxied(), &invalid_default, build_blocking_client);

                // THEN it is not kept
                assert!(result.is_err());
            }

            #[test]
            fn get_set_builder_other_env_should_keep_client() {
                // GIVEN a client built from a builder set
                let client = DefaultClient::new();
                client.set(reqwest::blocking::Client::builder());
                client
                    .get(
                        &ReqwestEnv::default(),
                        &default_builder,
                        build_blocking_client,
                    )
                    .unwrap();

                // WHEN getting it with another env
                let result = client.get(&proxied(), &invalid_default, build_blocking_client);

                // THEN it is kept
                assert!(result.is_ok());
            }
        }

        mod set {
            use super::*;

            #[test]
            fn set_should_replace_built_client() {
                // GIVEN a built default client
                let client = DefaultClient::new();
                client
                    .get(
                        &ReqwestEnv::default(),
                        &default_builder,
                        build_blocking_client,
                    )
                    .unwrap();

                // WHEN setting an invalid builder
                client.set(invalid_client());

                // THEN the client is built again from it
                assert!(
                    client
                        .get(
                            &ReqwestEnv::default(),
                            &default_builder,
                            build_blocking_client
                        )
                        .is_err()
                );
            }
        }
    }

    mod reqwest_env {
        use super::*;

        fn strings(vars: &[(&str, &str)]) -> Vec<(String, String)> {
            vars.iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect()
        }

        fn vars(vars: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
            vars.iter()
                .map(|(key, value)| (OsString::from(key), OsString::from(value)))
                .collect()
        }

        fn not_utf8() -> OsString {
            #[cfg(unix)]
            let invalid = std::os::unix::ffi::OsStringExt::from_vec(vec![0xff]);
            // WHY: an unpaired surrogate.
            #[cfg(windows)]
            let invalid = std::os::windows::ffi::OsStringExt::from_wide(&[0xd800]);

            invalid
        }

        /// Returns which of the proxies (`a` or `b`, or `direct`) a GET of
        /// `url` through a client from `env` reaches.
        fn route(env: &ReqwestEnv, url: &str, a: &Server, b: &Server) -> &'static str {
            let client = env.blocking_client_builder().build().unwrap();
            let result = client.get(url).send();
            match (a.received().len(), b.received().len()) {
                (1, 0) => "a",
                (0, 1) => "b",
                _ => {
                    assert!(result.is_err(), "{result:?}");
                    "direct"
                }
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_keep_proxy_variables_only() {
                let all = vars(&[
                    ("PATH", "/bin"),
                    ("HTTPS_PROXY", "http://p"),
                    ("no_proxy", "h"),
                ]);

                let env = ReqwestEnv::new(&all);

                assert_eq!(
                    env,
                    ReqwestEnv(strings(&[("HTTPS_PROXY", "http://p"), ("no_proxy", "h")]))
                );
            }

            #[cfg(windows)]
            #[test]
            fn new_should_keep_proxy_variables_of_any_case() {
                let all = vars(&[("Path", "/bin"), ("Https_Proxy", "http://p")]);

                let env = ReqwestEnv::new(&all);

                assert_eq!(env, ReqwestEnv(strings(&[("Https_Proxy", "http://p")])));
            }

            #[test]
            fn new_should_ignore_variables_not_valid_utf8() {
                let invalid = not_utf8();
                let all = [
                    (OsString::from("HTTP_PROXY"), invalid.clone()),
                    (invalid, OsString::from("http://p")),
                ];

                let env = ReqwestEnv::new(&all);

                assert_eq!(env, ReqwestEnv::default());
            }

            #[cfg(not(windows))]
            #[test]
            fn new_should_keep_proxy_variables_of_exact_case_only() {
                let all = vars(&[("Https_Proxy", "http://p")]);

                let env = ReqwestEnv::new(&all);

                assert_eq!(env, ReqwestEnv::default());
            }
        }

        mod var {
            use super::*;

            #[cfg(windows)]
            #[test]
            fn var_should_ignore_case() {
                let env = ReqwestEnv::new(&vars(&[("Http_Proxy", "http://a")]));

                assert_eq!(env.var(&["HTTP_PROXY"]), Some("http://a"));
            }

            #[cfg(not(windows))]
            #[test]
            fn var_should_match_exact_case_only() {
                let env = ReqwestEnv::new(&vars(&[("http_proxy", "http://a")]));

                assert_eq!(env.var(&["HTTP_PROXY"]), None);
                assert_eq!(env.var(&["HTTP_PROXY", "http_proxy"]), Some("http://a"));
            }
        }

        mod blocking_client_builder {
            use super::*;

            #[test]
            fn blocking_client_builder_should_resolve_proxies_like_hyper_util() {
                // GIVEN two proxies a and b and a target that is not reachable
                let target = closed_url();

                for (env, expected) in [
                    (vec![], "direct"),
                    (vec![("HTTP_PROXY", "A")], "a"),
                    (vec![("http_proxy", "A")], "a"),
                    (vec![("ALL_PROXY", "A")], "a"),
                    (vec![("all_proxy", "A")], "a"),
                    (vec![("HTTP_PROXY", "A"), ("http_proxy", "B")], "a"),
                    (vec![("HTTP_PROXY", "B"), ("ALL_PROXY", "A")], "b"),
                    (vec![("HTTP_PROXY", "::invalid"), ("ALL_PROXY", "A")], "a"),
                    (vec![("HTTPS_PROXY", "A")], "direct"),
                    (
                        vec![("HTTP_PROXY", "A"), ("NO_PROXY", "127.0.0.1")],
                        "direct",
                    ),
                    (
                        vec![("HTTP_PROXY", "A"), ("no_proxy", "127.0.0.1")],
                        "direct",
                    ),
                    (vec![("HTTP_PROXY", "A"), ("NO_PROXY", "other.com")], "a"),
                    (
                        vec![("HTTP_PROXY", "A"), ("REQUEST_METHOD", "GET")],
                        "direct",
                    ),
                ] {
                    let a = serve(ok);
                    let b = serve(ok);
                    let env = ReqwestEnv::new(&vars(
                        &env.iter()
                            .map(|(key, value)| {
                                let value = match *value {
                                    "A" => a.url.as_str(),
                                    "B" => b.url.as_str(),
                                    other => other,
                                };
                                (*key, value)
                            })
                            .collect::<Vec<_>>(),
                    ));

                    // WHEN sending a request through a client from the variables
                    let route = route(&env, &target, &a, &b);

                    // THEN it takes the expected proxy
                    assert_eq!(route, expected, "env: {env:?}");
                }
            }

            #[test]
            fn blocking_client_builder_proxy_not_valid_for_hyper_util_should_fall_back_to_all_proxy()
             {
                // GIVEN proxies hyper-util rejects (an unsupported scheme or a
                // space) though reqwest parses them, and a valid ALL_PROXY
                let target = closed_url();

                let invalid: [fn(&str) -> String; 2] = [
                    |url| url.replacen("http", "ftp", 1),
                    |url| format!(" {url}"),
                ];

                for invalid in invalid {
                    let a = serve(ok);
                    let b = serve(ok);
                    let http_proxy = invalid(&b.url);
                    let env = ReqwestEnv::new(&vars(&[
                        ("HTTP_PROXY", &http_proxy),
                        ("ALL_PROXY", &a.url),
                    ]));

                    // WHEN sending a request through a client from the variables
                    let route = route(&env, &target, &a, &b);

                    // THEN it takes ALL_PROXY like hyper-util's `Matcher::from_env`
                    assert_eq!(route, "a", "HTTP_PROXY: {http_proxy:?}");
                }
            }

            #[test]
            fn blocking_client_builder_request_method_not_valid_utf8_should_send_directly() {
                // GIVEN a proxy and a CGI request method that is not valid UTF-8
                let target = closed_url();
                let a = serve(ok);
                let b = serve(ok);
                let env = ReqwestEnv::new(&[
                    (OsString::from("HTTP_PROXY"), OsString::from(&a.url)),
                    (OsString::from("REQUEST_METHOD"), not_utf8()),
                ]);

                // WHEN sending a request through a client from the variables
                let route = route(&env, &target, &a, &b);

                // THEN it is direct like hyper-util, which checks only that
                // REQUEST_METHOD is set
                assert_eq!(route, "direct");
            }

            #[test]
            fn blocking_client_builder_should_proxy_https_by_connect() {
                let proxy = serve(not_found);
                let env = ReqwestEnv::new(&vars(&[("HTTPS_PROXY", &proxy.url)]));
                let client = env.blocking_client_builder().build().unwrap();

                let _ = client.get("https://example.invalid/path").send();

                assert!(proxy.received()[0].starts_with("CONNECT example.invalid:443 "));
            }

            #[test]
            fn blocking_client_builder_should_time_out_connecting_after_30_seconds() {
                let builder = ReqwestEnv::default().blocking_client_builder();

                assert!(format!("{builder:?}").contains("connect_timeout: 30s"));
            }
        }

        mod blocking_client_builder_with {
            use super::*;

            #[test]
            fn blocking_client_builder_with_should_time_out_each_read_not_whole_body() {
                // GIVEN a body arriving in pauses shorter than the read timeout but
                // longer than it in total
                let server =
                    serve_chunks(Duration::from_millis(100), &["a", "b", "c", "d", "e", "f"]);
                let client = ReqwestEnv::default()
                    .blocking_client_builder_with(CONNECT_TIMEOUT, Duration::from_millis(500))
                    .build()
                    .unwrap();
                let mut body = Vec::new();

                // WHEN reading it in parts
                let result = client.get(&server.url).send().unwrap().copy_to(&mut body);

                // THEN it is not cut
                assert!(result.is_ok(), "{result:?}");
                assert_eq!(body, b"abcdef");
            }

            #[test]
            fn blocking_client_builder_with_should_time_out_stalled_read() {
                let server = serve_chunks(Duration::from_secs(10), &["late"]);
                let client = ReqwestEnv::default()
                    .blocking_client_builder_with(CONNECT_TIMEOUT, Duration::from_secs(1))
                    .build()
                    .unwrap();

                let error = client
                    .get(&server.url)
                    .send()
                    .unwrap()
                    .copy_to(&mut Vec::new())
                    .unwrap_err();

                assert_eq!(
                    TransportError::from(error).kind(),
                    TransportErrorKind::Timeout
                );
            }
        }

        #[cfg(feature = "tokio")]
        mod client_builder_with {
            use super::*;

            #[test]
            fn client_builder_with_should_not_time_out_slow_response_in_total() {
                // GIVEN a body arriving in pauses shorter than the read timeout but
                // longer than it in total
                let server =
                    serve_chunks(Duration::from_millis(100), &["a", "b", "c", "d", "e", "f"]);
                let client = ReqwestEnv::default()
                    .client_builder_with(CONNECT_TIMEOUT, Duration::from_millis(500))
                    .build()
                    .unwrap();

                // WHEN receiving it whole
                let body =
                    block_on(async { client.get(&server.url).send().await.unwrap().bytes().await });

                // THEN it is not cut
                assert_eq!(body.unwrap(), "abcdef");
            }

            #[test]
            fn client_builder_with_should_time_out_stalled_read() {
                let server = serve_chunks(Duration::from_secs(10), &["late"]);
                let client = ReqwestEnv::default()
                    .client_builder_with(CONNECT_TIMEOUT, Duration::from_secs(1))
                    .build()
                    .unwrap();

                let error =
                    block_on(async { client.get(&server.url).send().await.unwrap().bytes().await })
                        .unwrap_err();

                assert_eq!(
                    TransportError::from(error).kind(),
                    TransportErrorKind::Timeout
                );
            }
        }

        #[cfg(feature = "tokio")]
        mod client_builder {
            use super::*;

            #[test]
            fn client_builder_should_use_proxies() {
                let proxy = serve(ok);
                let env = ReqwestEnv::new(&vars(&[("HTTP_PROXY", &proxy.url)]));
                let client = env.client_builder().build().unwrap();

                let output = block_on(async {
                    client
                        .get("http://example.invalid/path")
                        .send()
                        .await
                        .unwrap()
                        .status()
                });

                assert_eq!(output, 200);
                assert!(
                    proxy.received()[0].starts_with("GET http://example.invalid/path HTTP/1.1")
                );
            }

            #[test]
            fn client_builder_should_time_out_connecting_after_30_seconds_only() {
                let debug = format!("{:?}", ReqwestEnv::default().client_builder());

                assert!(debug.contains("connect_timeout: 30s"), "{debug}");
                assert!(!debug.contains(", timeout: "), "{debug}");
            }
        }
    }
}
