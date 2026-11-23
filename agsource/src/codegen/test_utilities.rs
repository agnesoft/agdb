use super::lower::Roots;
use super::lowered::Conversion;
use super::lowered::Expr;
use super::lowered::Module;
use super::lowered::Ty;
use crate::Cfg;
use crate::ItemId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use crate::Target;
use crate::TargetKind;
use agplatform::Fs;
use agplatform::Platform;
use agplatform::test_platform::test_platform;

pub(crate) fn load(source: &str) -> SourceCrate {
    let platform = test_platform();
    platform.fs().create_dir_all("/c/src").unwrap();
    platform.fs().write("/c/src/lib.rs", source).unwrap();
    SourceCrate::load(
        &platform,
        &Target::new(TargetKind::Lib, "/c/src/lib.rs"),
        &Cfg::All,
    )
}

/// The module of `Root` of the crate `c` and of its `traits` with the tests
/// of a test crate of it, and the errors of both, with `/` separators.
pub(crate) fn tested(
    source: &str,
    traits: &[&str],
    tests: &str,
    data: &[&str],
) -> (Module, Vec<String>) {
    let platform = test_platform();
    platform.fs().create_dir_all("/c/tests").unwrap();
    platform.fs().write("/c/tests/t.rs", tests).unwrap();
    let mut target = Target::new(TargetKind::Test, "/c/tests/t.rs");
    target.externs.push("c".into());
    let t = SourceCrate::load(&platform, &target, &Cfg::All);
    let (mut m, mut errors) = lower(source, traits, data);
    errors.extend(strings(&t.lower_tests(&load(source), "c", &mut m)));
    (m, errors)
}

/// The module of `Root` of the crate `c` with the exported types of the
/// API crate `api` of it, and the errors of both, with `/` separators.
pub(crate) fn exported(source: &str, api: &str, data: &[&str]) -> (Module, Vec<String>) {
    let platform = test_platform();
    platform.fs().create_dir_all("/api/src").unwrap();
    platform.fs().write("/api/src/lib.rs", api).unwrap();
    let mut target = Target::new(TargetKind::Lib, "/api/src/lib.rs");
    target.externs.push("c".into());
    let a = SourceCrate::load(&platform, &target, &Cfg::All);
    let (mut m, mut errors) = lower(source, &[], data);
    errors.extend(strings(&a.lower_exported(&load(source), "c", &mut m)));
    (m, errors)
}

/// A client crate: a builder generic over a transport trait with a
/// supertrait, results, `async`, `format!` and `Display` impls.
pub(crate) const CLIENT: &str = r#"use std::fmt;

pub const PREFIX: &str = "/api";

pub type ApiResult<T> = Result<T, ApiError>;

pub struct ApiError {
    pub status: u16,
    pub description: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {{{}}}", self.status, self.description)
    }
}

pub enum Kind {
    File,
    Memory,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::File => f.write_str("file"),
            Kind::Memory => f.write_str("memory"),
        }
    }
}

pub trait Http {
    async fn get<T>(&self, uri: &str) -> ApiResult<(u16, T)>;

    async fn status(&self, uri: &str) -> ApiResult<u16> {
        Ok(self.get::<()>(uri).await?.0)
    }
}

pub trait Client: Http {}

pub struct Transport;

impl Http for Transport {
    async fn get<T>(&self, uri: &str) -> ApiResult<(u16, T)> {
        panic!("glue")
    }
}

impl Client for Transport {}

pub struct Unrelated;

impl Unrelated {
    pub fn get(&self) -> u8 {
        1
    }
}

pub struct Api<T: Client> {
    client: T,
    base: String,
    pub token: Option<String>,
}

impl<T: Client> Api<T> {
    pub fn new(client: T, address: &str) -> Self {
        let base = if address.starts_with("http") {
            address.to_string()
        } else {
            format!("http://{address}{}", PREFIX)
        };
        Self {
            client,
            base,
            token: None,
        }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub async fn kind(&self, kind: Kind, n: u64) -> ApiResult<u16> {
        self.client
            .get::<()>(&self.url(&format!("/{kind}/{0}", n.to_string())))
            .await
            .map(|(status, _)| status)
    }

    pub async fn login(&mut self) -> ApiResult<u16> {
        let (status, token): (u16, String) = self.client.get(&self.url("/login")).await?;
        self.token = Some(token);
        Ok(status)
    }

    pub async fn ping(&self) -> ApiResult<u16> {
        self.client.status(&self.url("/ping")).await
    }

    pub fn fail(&self) -> ApiResult<u16> {
        Err(ApiError {
            status: 1,
            description: "fail".to_string(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}
"#;

/// The [client](SourceCrate::lower_client) module of the API crate `api`
/// of an empty crate `c`, and the errors, with `/` separators.
pub(crate) fn client(api: &str) -> (Module, Vec<String>) {
    let platform = test_platform();
    platform.fs().create_dir_all("/api/src").unwrap();
    platform.fs().write("/api/src/lib.rs", api).unwrap();
    let mut target = Target::new(TargetKind::Lib, "/api/src/lib.rs");
    target.externs.push("c".into());
    let a = SourceCrate::load(&platform, &target, &Cfg::All);
    let (m, errors) = a.lower_client(&load(""), "c");
    (m, strings(&errors))
}

/// The module of `Root` of the crate `c` of `source` with the builders of
/// the [`CLIENT`] crate `api` [provided](Module::provide) and the tests of
/// the module `suite` of the test crate `tests` of both, its module `glue`
/// the glue, and the errors, with `/` separators.
pub(crate) fn suite(source: &str, tests: &str) -> (Module, Vec<String>) {
    let platform = test_platform();
    platform.fs().create_dir_all("/api/src").unwrap();
    platform.fs().write("/api/src/lib.rs", CLIENT).unwrap();
    platform.fs().create_dir_all("/t/tests").unwrap();
    platform.fs().write("/t/tests/t.rs", tests).unwrap();
    let mut api = Target::new(TargetKind::Lib, "/api/src/lib.rs");
    api.externs.push("c".into());
    let a = SourceCrate::load(&platform, &api, &Cfg::All);
    let mut target = Target::new(TargetKind::Test, "/t/tests/t.rs");
    target.externs.extend(["c".into(), "api".into()]);
    let t = SourceCrate::load(&platform, &target, &Cfg::All);
    let c = load(source);
    let (mut m, mut errors) = lower(source, &[], &[]);
    let (client, client_errors) = a.lower_client(&c, "c");
    errors.extend(strings(&client_errors));
    m.provide(client, "api");
    errors.extend(strings(&t.lower_suite(
        &[(&c, "c"), (&a, "api")],
        "suite",
        "glue",
        &mut m,
    )));
    (m, errors)
}

/// A test crate of [`suite`]: tests in the module `suite` and its modules
/// using the structs, functions and constants of the module `glue`, the
/// client of `api` and the types of `c`.
pub(crate) const SUITE: &str = r#"mod glue {
    use api::Api;
    use api::Transport;

    pub const ADMIN: &str = "admin";

    pub type Name = String;

    pub static PORT: u16 = 3000;

    pub struct Server {
        pub api: Api<Transport>,
        pub dir: String,
        port: u16,
    }

    pub struct Pair(pub u8, pub u8);

    pub struct Helpers;

    impl Helpers {
        pub fn agent() -> String {
            panic!("glue")
        }
    }

    impl Server {
        pub async fn new() -> Self {
            panic!("glue")
        }

        pub fn address(&self) -> String {
            panic!("glue")
        }

        fn private(&self) {}
    }

    pub fn next_name(prefix: &str) -> String {
        panic!("glue")
    }
}

mod suite {
    use crate::glue::ADMIN;
    use crate::glue::Server;

    struct NotATest;

    const TESTS: &[&str] = tests![status = ignore("needs a server")];

    async fn broken() {
        custom![1];
    }

    mod login {
        use crate::glue::Helpers;
        use crate::glue::Name;
        use crate::glue::Pair;
        use crate::glue::Server;
        use crate::glue::next_name;
        use c::Root;

        const TESTS: &[&str] = tests![logs_in, fails];

        async fn logs_in() {
            let mut server = Server::new().await;
            let status = server.api.login().await.unwrap();
            let name: Name = next_name("user");
            let agent = Helpers::agent();
            let pair = Pair(1, 2);
            let port = crate::glue::PORT;
            assert_ne!(status, 0);
        }

        async fn fails() {
            let server = Server::new().await;
            let status = server.api.fail().unwrap_err().status;
            let code = server.api.ping().await.unwrap_err().status;
            let root = Root::new();
            assert!(server.address().is_empty());
        }
    }

    async fn status() {
        let server = Server::new().await;
        assert_eq!(server.dir, ADMIN);
    }

    mod logout {
        use crate::glue::ADMIN;
        use crate::glue::Server;

        async fn logs_out() {
            let server = Server::new().await;
            assert_ne!(server.dir, ADMIN);
        }
    }
}

fn outside() {}
"#;

/// The client module of `api` after the passes of a TS client, with
/// neither the lowering nor the functions having errors.
pub(crate) fn client_passed(api: &str) -> Module {
    use super::pass::EraseOwnership;
    use super::pass::ExplicitReturns;
    use super::pass::Inline;
    use super::pass::Passes;
    use super::pass::Prune;
    use super::pass::Resolve;
    use super::pass::ThrowErrors;
    use super::ts::EraseCasts;
    use super::ts::RemoveDeadHelpers;

    let (mut m, mut errors) = client(api);
    let passes = Passes::new()
        .with(Resolve)
        .with(Prune)
        .with(ExplicitReturns)
        .with(Inline)
        .with(EraseOwnership)
        .with(ThrowErrors)
        .with(EraseCasts)
        .with(RemoveDeadHelpers);
    errors.extend(strings(&passes.run(&mut m, &mut |_, _| {})));
    m.for_each_func(&mut |f, _| errors.extend(f.errors.iter().cloned()));
    assert_eq!(errors, Vec::<String>::new());
    m
}

pub(crate) fn id(c: &SourceCrate, path: &str) -> ItemId {
    c.item_ids()
        .find(|id| c.resolve_str(path, Namespace::Type) == Resolved::Item(*id))
        .unwrap()
}

/// The module of `Root` and the errors, with `/` separators.
pub(crate) fn lower(source: &str, traits: &[&str], data: &[&str]) -> (Module, Vec<String>) {
    let c = load(source);
    let roots = Roots {
        root: id(&c, "Root"),
        traits: traits.iter().map(|t| id(&c, t)).collect(),
        data: data.iter().map(|d| id(&c, d)).collect(),
    };
    let (m, errors) = c.lower(&roots);
    (m, strings(&errors))
}

pub(crate) fn strings(errors: &[crate::Error]) -> Vec<String> {
    errors
        .iter()
        .map(|e| e.to_string().replace('\\', "/"))
        .collect()
}

/// The module of `Root` after [`Resolve`](super::pass::Resolve) and
/// [`ExplicitReturns`](super::pass::ExplicitReturns), with neither the
/// lowering nor the functions having errors.
pub(crate) fn resolved(source: &str, traits: &[&str], data: &[&str]) -> Module {
    use super::pass::Pass;

    let (mut m, mut errors) = lower(source, traits, data);
    super::pass::Resolve.run(&mut m, &mut Vec::new());
    super::pass::ExplicitReturns.run(&mut m, &mut Vec::new());
    m.for_each_func(&mut |f, _| errors.extend(f.errors.iter().cloned()));
    assert_eq!(errors, Vec::<String>::new());
    m
}

/// A conversion with no generics, location or errors.
pub(crate) fn conversion(to: &str, from: Ty, body: Expr) -> Conversion {
    Conversion {
        to: to.into(),
        from,
        generics: Vec::new(),
        body,
        at: None,
        errors: Vec::new(),
    }
}

/// An API crate of [`ROUTES_SERVER`]: serde types, an enum sent by its
/// `Display`, a transport trait with a provided method, its reqwest
/// implementation through a `Deref` field and a helper, and a client
/// sending requests to `{address}/api`.
pub(crate) const ROUTES_API: &str = r#"use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;

pub type ApiResult<T> = Result<T, ApiError>;

pub struct ApiError {
    pub status: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    #[serde(rename = "label")]
    pub name: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Items(pub Vec<Item>);

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    InMemory,
    File,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::InMemory => f.write_str("in_memory"),
            Kind::File => f.write_str("file"),
        }
    }
}

pub trait Http {
    async fn get<T: DeserializeOwned>(&self, uri: &str) -> ApiResult<(u16, T)>;
    async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        uri: &str,
        json: Option<T>,
    ) -> ApiResult<(u16, R)>;
    async fn delete(&self, uri: &str) -> ApiResult<u16>;

    async fn exec(&self, uri: &str, items: &[Item]) -> ApiResult<(u16, Vec<Item>)> {
        self.post(uri, Some(items)).await
    }
}

pub struct Inner(pub reqwest::Client);

impl std::ops::Deref for Inner {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Reqwest {
    pub client: Inner,
}

impl Http for Reqwest {
    async fn get<T: DeserializeOwned>(&self, uri: &str) -> ApiResult<(u16, T)> {
        let response = self.client.get(uri).send().await?;
        let body = response.json::<T>().await?;
        Ok((200, body))
    }

    async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        uri: &str,
        json: Option<T>,
    ) -> ApiResult<(u16, R)> {
        let mut request = self.client.post(uri);
        if let Some(json) = json {
            request = request.json(&json);
        }
        let response = request.send().await?;
        if response.status().is_success() {
            let body = response.json::<R>().await?;
            Ok((200, body))
        } else {
            Err(ApiError { status: 500 })
        }
    }

    async fn delete(&self, uri: &str) -> ApiResult<u16> {
        self.send_delete(uri).await
    }
}

impl Reqwest {
    async fn send_delete(&self, uri: &str) -> ApiResult<u16> {
        self.client.delete(uri).send().await?;
        Ok(200)
    }
}

pub struct Api<T: Http> {
    client: T,
    base_url: String,
}

impl<T: Http> Api<T> {
    pub fn new(client: T, address: &str) -> Self {
        Self {
            client,
            base_url: format!("{address}/api"),
        }
    }

    pub async fn login(&self, name: &str, password: &str) -> ApiResult<String> {
        Ok(self
            .client
            .post::<Login, String>(
                &self.url("/login"),
                Some(Login {
                    name: name.to_string(),
                    password: password.to_string(),
                }),
            )
            .await?
            .1)
    }

    pub async fn items(&self, owner: &str) -> ApiResult<(u16, Items)> {
        self.client
            .get(&self.url(&format!("/items/{owner}/list")))
            .await
    }

    pub async fn add(&self, owner: &str, kind: Kind) -> ApiResult<u16> {
        Ok(self
            .client
            .post::<(), ()>(
                &self.url(&format!("/items/{owner}/add?kind={kind}&force=true")),
                None,
            )
            .await?
            .0)
    }

    pub async fn remove(&self, owner: &str, id: u64) -> ApiResult<u16> {
        self.client
            .delete(&self.url(&format!("/items/{owner}/{id}")))
            .await
    }

    pub async fn exec(&self, owner: &str, items: &[Item]) -> ApiResult<(u16, Vec<Item>)> {
        self.client
            .exec(&self.url(&format!("/items/{owner}/exec")), items)
            .await
    }

    fn url(&self, uri: &str) -> String {
        format!("{}{uri}", self.base_url)
    }
}
"#;

/// A server binary of [`ROUTES_API`]: an axum router nested under a base
/// path known when it runs, handlers with `Path`, `Query`, `Json` and local
/// extractors, answering through aliases, header arrays and a local
/// `IntoResponse`, and a route no client method calls.
pub(crate) const ROUTES_SERVER: &str = r#"use axum::Json;
use axum::Router;
use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing;
use serde::Deserialize;

type Answer<T = StatusCode> = Result<T, StatusCode>;

type Header = [(&'static str, String); 1];

#[derive(Deserialize)]
struct AddParams {
    kind: api::Kind,
    #[serde(default)]
    force: bool,
}

struct User;

impl<S> FromRequestParts<S> for User {
    type Rejection = StatusCode;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, StatusCode> {
        Ok(User)
    }
}

struct Batch<T>(T);

impl<S, T> FromRequest<S> for Batch<T> {
    type Rejection = StatusCode;

    async fn from_request(request: Request, state: &S) -> Result<Self, StatusCode> {
        let Json(value) = Json::<T>::from_request(request, state).await?;
        Ok(Batch(value))
    }
}

struct Reply<T> {
    value: T,
}

impl<T> IntoResponse for Reply<T> {
    fn into_response(self) -> Response {
        Json(self.value).into_response()
    }
}

async fn login(Json(login): Json<api::Login>) -> Answer<(StatusCode, Json<String>)> {
    Ok((StatusCode::OK, Json(login.name)))
}

async fn items(_user: User, Path(owner): Path<String>) -> Answer<(StatusCode, Json<api::Items>)> {
    Err(StatusCode::NOT_FOUND)
}

async fn add(
    _user: User,
    State(_state): State<u8>,
    Path(owner): Path<String>,
    Query(params): Query<AddParams>,
) -> Answer<(StatusCode, Header)> {
    Ok((StatusCode::CREATED, [("commit-index", owner)]))
}

async fn remove(Path((owner, id)): Path<(String, u64)>) -> Answer {
    Ok(StatusCode::NO_CONTENT)
}

async fn exec(
    Path(owner): Path<String>,
    Batch(items): Batch<Vec<api::Item>>,
) -> Answer<(StatusCode, Reply<Vec<api::Item>>)> {
    Ok((StatusCode::OK, Reply { value: items }))
}

async fn internal() -> StatusCode {
    StatusCode::OK
}

fn app(basepath: String) -> Router {
    let api = Router::new()
        .route("/login", routing::post(login))
        .route("/items/{owner}/list", routing::get(items))
        .route("/items/{owner}/add", routing::post(add))
        .route("/items/{owner}/{id}", routing::delete(remove))
        .route("/items/{owner}/exec", routing::post(exec))
        .route("/internal", routing::get(internal));
    let router = Router::new().nest("/api", api).with_state(0);

    if basepath.is_empty() {
        router
    } else {
        Router::new().nest(&basepath, router)
    }
}

#[tokio::main]
async fn main() {
    let app = app(String::new());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
"#;

/// The API crate `api` of [`routes_server`], naming the crates of a
/// client.
fn routes_api(api: &str) -> SourceCrate {
    let platform = test_platform();
    platform.fs().create_dir_all("/api/src").unwrap();
    platform.fs().write("/api/src/lib.rs", api).unwrap();
    let mut target = Target::new(TargetKind::Lib, "/api/src/lib.rs");
    target
        .externs
        .extend(["c".into(), "reqwest".into(), "serde".into()]);
    SourceCrate::load(&platform, &target, &Cfg::All)
}

/// The server binary `server`, naming the API crate `api` and the crates
/// of a server.
fn routes_server(server: &str) -> SourceCrate {
    let platform = test_platform();
    platform.fs().create_dir_all("/s/src").unwrap();
    platform.fs().write("/s/src/main.rs", server).unwrap();
    let mut target = Target::new(TargetKind::Bin, "/s/src/main.rs");
    target
        .externs
        .extend(["api".into(), "axum".into(), "serde".into(), "tokio".into()]);
    SourceCrate::load(&platform, &target, &Cfg::All)
}

/// The routes of the server binary `server` of the API crate `api`, and
/// the errors, with `/` separators.
pub(crate) fn served(api: &str, server: &str) -> (super::routes::Routes, Vec<String>) {
    let a = routes_api(api);
    let (routes, errors) = routes_server(server).lower_routes(&[(&a, "api")]);
    (routes, strings(&errors))
}

/// The transport of the API crate `api`, and the errors, with `/`
/// separators.
pub(crate) fn transport(api: &str) -> (super::routes::Transport, Vec<String>) {
    let (transport, errors) = routes_api(api).transport();
    (transport, strings(&errors))
}

/// The check of the client of the API crate `api` after
/// [`Inline`](super::pass::Inline) against the server binary `server`, and
/// the errors of the lowering, the passes, the transport, the routes and
/// the check, with `/` separators.
pub(crate) fn checked(api: &str, server: &str) -> (super::routes::Checked, Vec<String>) {
    use super::pass::ExplicitReturns;
    use super::pass::Inline;
    use super::pass::Passes;
    use super::pass::Prune;
    use super::pass::Resolve;

    let a = routes_api(api);
    let c = load("");
    let (mut client, errors) = a.lower_client(&c, "c");
    let mut errors = strings(&errors);
    let passes = Passes::new()
        .with(Resolve)
        .with(Prune)
        .with(ExplicitReturns)
        .with(Inline);
    errors.extend(strings(&passes.run(&mut client, &mut |_, _| {})));
    client.for_each_func(&mut |f, _| errors.extend(f.errors.iter().cloned()));
    let mut data = Module::default();
    errors.extend(strings(&a.lower_exported(&c, "c", &mut data)));
    let (transport, transport_errors) = a.transport();
    errors.extend(strings(&transport_errors));
    let (routes, route_errors) = routes_server(server).lower_routes(&[(&a, "api")]);
    errors.extend(strings(&route_errors));
    let (checked, check_errors) = super::routes::check(&client, &[&data], &transport, &routes);
    errors.extend(strings(&check_errors));
    (checked, errors)
}

/// The source with each `(from, to)` replacing the first `from`, which must
/// be in it.
pub(crate) fn edited(source: &str, edits: &[(&str, &str)]) -> String {
    edits.iter().fold(source.to_owned(), |source, (from, to)| {
        assert!(source.contains(from), "no `{from}` in the source");
        source.replacen(from, to, 1)
    })
}
