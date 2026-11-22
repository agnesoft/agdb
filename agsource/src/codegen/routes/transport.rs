//! [`SourceCrate::transport`]: the requests the implementations of a
//! client's transport trait send with reqwest.

use crate::Error;
use crate::ItemId;
use crate::ModuleId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use syn::visit::Visit;

/// The request a method of a transport trait sends, the indices being those
/// of its parameters after `self`.
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    /// The method of the reqwest `Client` called: `get`, `post`.
    pub verb: String,
    /// The parameter that is the URL.
    pub uri: usize,
    /// The parameter, or the `Option` parameter, sent as the JSON body.
    pub body: Option<usize>,
    /// Where the JSON body of the response is in the value the method
    /// returns in `Ok`: the indices of the tuples it is in, none when it is
    /// not read.
    pub response: Option<Vec<usize>>,
}

/// The requests by the name of the trait and of the method.
pub type Transport = BTreeMap<(String, String), Request>;

impl SourceCrate {
    /// The requests the methods of the impls of the crate's traits send
    /// through a field that is, or derefs to, a reqwest `Client`: the
    /// `self.client.<verb>(uri)`, the `.json(body)` of the request and the
    /// `.json()` of the response, in the method or the methods of its type
    /// it calls. The impls of a trait sending different requests from a
    /// method are an error.
    pub fn transport(&self) -> (Transport, Vec<Error>) {
        let methods = methods(self);
        let mut out = Transport::new();
        let mut errors = Vec::new();

        for id in self.item_ids() {
            let syn::Item::Impl(i) = self.item(id) else {
                continue;
            };
            let (Some((path, _)), Some(ty)) = (&i.trait_, self_item(self, id, i)) else {
                continue;
            };
            let Resolved::Item(t) = self.resolve(id.module(), path, Namespace::Type) else {
                continue;
            };
            let syn::Item::Trait(t) = self.item(t) else {
                continue;
            };
            let reader = Reader {
                c: self,
                ty,
                methods: methods.get(&ty).cloned().unwrap_or_default(),
            };

            for f in i.items.iter().filter_map(|i| match i {
                syn::ImplItem::Fn(f) => Some(f),
                _ => None,
            }) {
                let Some(read) = reader.method(id.module(), f, 0) else {
                    continue;
                };
                let key = (t.ident.to_string(), f.sig.ident.to_string());
                let at = |message| Error::Lowering {
                    path: self.file(self.module(id.module()).file()).path().to_owned(),
                    line: f.sig.ident.span().start().line,
                    item: f.sig.ident.to_string(),
                    message,
                };

                match read.request() {
                    Ok(request) => match out.get(&key) {
                        Some(other) if *other != request => errors.push(at(format!(
                            "the impls of `{}::{}` send different requests",
                            key.0, key.1
                        ))),
                        _ => {
                            out.insert(key, request);
                        }
                    },
                    Err(message) => errors.push(at(message)),
                }
            }
        }

        (out, errors)
    }
}

/// The local type the impl is for.
fn self_item(c: &SourceCrate, id: ItemId, i: &syn::ItemImpl) -> Option<ItemId> {
    match &*i.self_ty {
        syn::Type::Path(p) => match c.resolve(id.module(), &p.path, Namespace::Type) {
            Resolved::Item(ty) => Some(ty),
            _ => None,
        },
        _ => None,
    }
}

/// The methods of the impls of each local type, with the module of the
/// impl.
fn methods(c: &SourceCrate) -> HashMap<ItemId, HashMap<String, (ModuleId, &syn::ImplItemFn)>> {
    let mut out: HashMap<ItemId, HashMap<String, (ModuleId, &syn::ImplItemFn)>> = HashMap::new();

    for id in c.item_ids() {
        let syn::Item::Impl(i) = c.item(id) else {
            continue;
        };
        let Some(ty) = self_item(c, id, i) else {
            continue;
        };

        for item in &i.items {
            if let syn::ImplItem::Fn(f) = item {
                out.entry(ty)
                    .or_default()
                    .insert(f.sig.ident.to_string(), (id.module(), f));
            }
        }
    }

    out
}

/// What a method sends, in the indices of its parameters.
#[derive(Default)]
struct Read {
    /// The verbs called with the parameter of the URL.
    verbs: BTreeSet<(String, usize)>,
    bodies: BTreeSet<usize>,
    response: Option<Vec<usize>>,
}

impl Read {
    fn request(self) -> Result<Request, String> {
        let [(verb, uri)] = self
            .verbs
            .into_iter()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|verbs: Vec<(String, usize)>| {
                format!(
                    "a transport sending {} requests",
                    verbs
                        .iter()
                        .map(|(v, _)| format!("`{v}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        if uri == NOT_A_PARAM {
            return Err(format!("a `{verb}` to a URL that is no parameter"));
        }

        let body = match self.bodies.into_iter().collect::<Vec<_>>().as_slice() {
            [] => None,
            [body] => Some(*body),
            _ => return Err(format!("a `{verb}` with more than one JSON body")),
        };
        Ok(Request {
            verb,
            uri,
            body,
            response: self.response,
        })
    }
}

const MAX_DEPTH: usize = 8;
const NOT_A_PARAM: usize = usize::MAX;

struct Reader<'a> {
    c: &'a SourceCrate,
    /// The type of the impl.
    ty: ItemId,
    methods: HashMap<String, (ModuleId, &'a syn::ImplItemFn)>,
}

impl Reader<'_> {
    /// What the method sends, none when it sends no request.
    fn method(&self, module: ModuleId, f: &syn::ImplItemFn, depth: usize) -> Option<Read> {
        let params: Vec<String> = f
            .sig
            .inputs
            .iter()
            .filter_map(|i| match i {
                syn::FnArg::Typed(t) => match &*t.pat {
                    syn::Pat::Ident(i) => Some(i.ident.to_string()),
                    _ => Some(String::new()),
                },
                syn::FnArg::Receiver(_) => None,
            })
            .collect();
        let mut body = Body {
            reader: self,
            module,
            params: &params,
            options: HashMap::new(),
            decoded: BTreeSet::new(),
            read: Read::default(),
            sends: false,
            depth,
        };
        body.visit_block(&f.block);

        if let Some(tail) = tail(&f.block) {
            body.read.response = body.returned(tail).or(body.read.response);
        }

        body.sends.then_some(body.read)
    }

    /// Whether the field of the type, by its type or what that derefs to, is
    /// a reqwest `Client`.
    fn is_client(&self, field: &str) -> bool {
        let syn::Item::Struct(s) = self.c.item(self.ty) else {
            return false;
        };

        s.fields
            .iter()
            .find(|f| f.ident.as_ref().is_some_and(|i| i == field))
            .is_some_and(|f| self.client_type(self.ty.module(), &f.ty, 0))
    }

    fn client_type(&self, module: ModuleId, ty: &syn::Type, depth: usize) -> bool {
        let p = match ty {
            syn::Type::Path(p) => p,
            syn::Type::Reference(r) => return self.client_type(module, &r.elem, depth),
            _ => return false,
        };

        match self.c.resolve(module, &p.path, Namespace::Type) {
            Resolved::External(path) => {
                path.first().is_some_and(|f| f == "reqwest")
                    && path.last().is_some_and(|l| l == "Client")
            }
            Resolved::Item(id) if depth < MAX_DEPTH => self.c.item_ids().any(|i| {
                let syn::Item::Impl(imp) = self.c.item(i) else {
                    return false;
                };

                imp.trait_.as_ref().is_some_and(|(t, _)| {
                    matches!(self.c.resolve(i.module(), t, Namespace::Type), Resolved::External(t) if t.last().is_some_and(|l| l == "Deref"))
                }) && self_item(self.c, i, imp) == Some(id)
                    && imp.items.iter().any(|item| match item {
                        syn::ImplItem::Type(t) if t.ident == "Target" => {
                            self.client_type(i.module(), &t.ty, depth + 1)
                        }
                        _ => false,
                    })
            }),
            _ => false,
        }
    }
}

/// The expression a block evaluates to.
fn tail(block: &syn::Block) -> Option<&syn::Expr> {
    match block.stmts.last() {
        Some(syn::Stmt::Expr(e, None)) => Some(e),
        _ => None,
    }
}

/// The expression without `.await`, `?`, parentheses and references.
fn bare(e: &syn::Expr) -> &syn::Expr {
    match e {
        syn::Expr::Await(a) => bare(&a.base),
        syn::Expr::Try(t) => bare(&t.expr),
        syn::Expr::Paren(p) => bare(&p.expr),
        syn::Expr::Reference(r) => bare(&r.expr),
        e => e,
    }
}

fn ident(e: &syn::Expr) -> Option<String> {
    match bare(e) {
        syn::Expr::Path(p) => p.path.get_ident().map(ToString::to_string),
        _ => None,
    }
}

fn is_self(e: &syn::Expr) -> bool {
    matches!(bare(e), syn::Expr::Path(p) if p.path.is_ident("self"))
}

struct Body<'a, 'p> {
    reader: &'a Reader<'a>,
    module: ModuleId,
    params: &'p [String],
    /// The names bound by `if let Some(x) = param`, with the parameter.
    options: HashMap<String, usize>,
    /// The locals bound to a decoded JSON response.
    decoded: BTreeSet<String>,
    read: Read,
    sends: bool,
    depth: usize,
}

impl Body<'_, '_> {
    fn param(&self, e: &syn::Expr) -> Option<usize> {
        let name = ident(e)?;
        self.params
            .iter()
            .position(|p| *p == name)
            .or_else(|| self.options.get(&name).copied())
    }

    /// The method of the type the expression calls on `self`, with the
    /// parameters its arguments are.
    fn self_call(&self, e: &syn::Expr) -> Option<(Read, Vec<Option<usize>>)> {
        let syn::Expr::MethodCall(m) = bare(e) else {
            return None;
        };

        if !is_self(&m.receiver) || self.depth >= MAX_DEPTH {
            return None;
        }

        let (module, f) = self.reader.methods.get(&m.method.to_string())?;
        let read = self.reader.method(*module, f, self.depth + 1)?;
        Some((read, m.args.iter().map(|a| self.param(a)).collect()))
    }

    /// Where the decoded response is in what the expression returns.
    fn returned(&self, e: &syn::Expr) -> Option<Vec<usize>> {
        if let Some((read, _)) = self.self_call(e) {
            return read.response;
        }

        let call = match bare(e) {
            syn::Expr::Call(call) => call,
            syn::Expr::If(i) => {
                return tail(&i.then_branch)
                    .and_then(|t| self.returned(t))
                    .or_else(|| i.else_branch.as_ref().and_then(|(_, e)| self.returned(e)));
            }
            syn::Expr::Block(b) => return tail(&b.block).and_then(|t| self.returned(t)),
            syn::Expr::Match(m) => return m.arms.iter().find_map(|a| self.returned(&a.body)),
            _ => return None,
        };
        let syn::Expr::Path(p) = &*call.func else {
            return None;
        };

        if !matches!(self.reader.c.resolve(self.module, &p.path, Namespace::Value), Resolved::External(v) if v.last().is_some_and(|l| l == "Ok"))
        {
            return None;
        }

        call.args.first().and_then(|value| self.position(value))
    }

    fn position(&self, e: &syn::Expr) -> Option<Vec<usize>> {
        match bare(e) {
            syn::Expr::Tuple(t) => t.elems.iter().enumerate().find_map(|(i, e)| {
                self.position(e).map(|mut at| {
                    at.insert(0, i);
                    at
                })
            }),
            e => ident(e)
                .filter(|n| self.decoded.contains(n))
                .map(|_| Vec::new()),
        }
    }
}

/// Whether the expression decodes a JSON response: a `.json()` call.
fn decodes(e: &syn::Expr) -> bool {
    struct Decodes(bool);

    impl Visit<'_> for Decodes {
        fn visit_expr_method_call(&mut self, m: &syn::ExprMethodCall) {
            self.0 |= m.method == "json" && m.args.is_empty();
            syn::visit::visit_expr_method_call(self, m);
        }
    }

    let mut v = Decodes(false);
    v.visit_expr(e);
    v.0
}

impl<'ast> Visit<'ast> for Body<'_, '_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let (syn::Pat::Ident(name), Some(init)) = (&local.pat, &local.init)
            && decodes(&init.expr)
        {
            self.decoded.insert(name.ident.to_string());
        }

        syn::visit::visit_local(self, local);
    }

    fn visit_expr_let(&mut self, e: &'ast syn::ExprLet) {
        if let syn::Pat::TupleStruct(t) = &*e.pat
            && t.path.is_ident("Some")
            && let [syn::Pat::Ident(name)] = t.elems.iter().collect::<Vec<_>>()[..]
            && let Some(param) = self.param(&e.expr)
        {
            self.options.insert(name.ident.to_string(), param);
        }

        syn::visit::visit_expr_let(self, e);
    }

    fn visit_expr_return(&mut self, r: &'ast syn::ExprReturn) {
        if self.read.response.is_none()
            && let Some(e) = &r.expr
        {
            self.read.response = self.returned(e);
        }

        syn::visit::visit_expr_return(self, r);
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let args: Vec<&syn::Expr> = m.args.iter().collect();

        if let syn::Expr::Field(field) = bare(&m.receiver)
            && is_self(&field.base)
            && let syn::Member::Named(name) = &field.member
            && self.reader.is_client(&name.to_string())
            && let [uri] = args[..]
        {
            self.sends = true;

            match self.param(uri) {
                Some(uri) => {
                    self.read.verbs.insert((m.method.to_string(), uri));
                }
                None => {
                    self.read.verbs.insert((m.method.to_string(), NOT_A_PARAM));
                }
            }
        } else if m.method == "json"
            && let [body] = args[..]
            && let Some(body) = self.param(body)
        {
            self.read.bodies.insert(body);
        } else if let Some((read, args)) = self.self_call(&syn::Expr::MethodCall(m.clone())) {
            self.sends = true;
            let at = |i: usize| args.get(i).copied().flatten().unwrap_or(NOT_A_PARAM);
            self.read
                .verbs
                .extend(read.verbs.into_iter().map(|(v, uri)| (v, at(uri))));
            self.read.bodies.extend(read.bodies.into_iter().map(at));
        }

        syn::visit::visit_expr_method_call(self, m);
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::test_utilities::transport;

    /// A client crate with the `impls`: the transport trait `Http`, the
    /// types `A` and `B` with a reqwest `Client` field `client`, the field
    /// `inner` deref-ing to one and `direct` borrowing one, and the error
    /// type `E`.
    fn source(impls: &str) -> String {
        format!(
            "pub struct Inner(pub reqwest::Client);

impl std::ops::Deref for Inner {{
    fn deref(&self) -> &Self::Target {{
        &self.0
    }}

    type Target = reqwest::Client;
}}

pub struct Cycle;

impl std::ops::Deref for Cycle {{
    type Target = Cycle;
}}

pub trait Http {{}}

pub struct E;

pub struct A<'a, T> {{
    pub client: reqwest::Client,
    pub inner: Inner,
    pub direct: &'a reqwest::Client,
    pub pair: (u8, u8),
    pub generic: T,
    pub cycle: Cycle,
}}

pub struct B {{
    pub client: reqwest::Client,
}}

pub enum C {{
    V,
}}

{impls}"
        )
    }

    mod source_crate {
        use super::*;

        mod transport {
            use super::*;
            use crate::codegen::routes::Request;

            fn request(
                verb: &str,
                uri: usize,
                body: Option<usize>,
                response: Option<&[usize]>,
            ) -> Request {
                Request {
                    verb: verb.into(),
                    uri,
                    body,
                    response: response.map(<[usize]>::to_vec),
                }
            }

            #[test]
            fn transport_should_read_the_requests_of_the_methods() {
                // GIVEN
                let source = source(
                    "impl<T> Http for A<'_, T> {
    const NAME: &'static str = \"a\";

    fn name(&self) -> &str {
        \"a\"
    }

    async fn get<T>(&self, _: u8, uri: &str) -> Result<(u16, T), E> {
        let response = (self.inner).get(uri).send().await?;
        let body = response.json().await?;
        if body.is_empty() {
            return Ok((0, body));
        }
        Ok((200, body))
    }

    async fn post<T, U>(&self, uri: &str, json: Option<T>) -> Result<(u16, U), E> {
        let mut request = self.direct.post(uri);
        if let Some(body) = json {
            request = request.json(&body);
        }
        if let Ok(x) = ready() {}
        let response = request.send().await?;
        let body = response.json().await?;
        match response.status() {
            404 => Err(E),
            _ => Ok((200, body)),
        }
    }

    async fn put<U>(&self, uri: &str) -> Result<(u16, U), E> {
        let body = self.client.put(uri).send().await?.json().await?;
        {
            Ok((200, (body)))
        }
    }

    async fn delete(&self, uri: &str) -> Result<u16, E> {
        self.log(uri);
        self.clone();
        return self.send(uri).await;
    }

    async fn head(&self, uri: &str) -> Result<u16, E> {
        if uri.is_empty() {
            Err(E)
        } else {
            Ok(self.send(uri).await?)
        }
    }

    async fn patch(&self, uri: &str) -> Option<u8> {
        self.client.patch(uri);
        if true { Some(1) } else { (make)(1) }
    }

    async fn options(&self, uri: &str) -> u8 {
        self.client.options(uri);
        uri
    }
}

impl<T> A<'_, T> {
    fn log(&self, uri: &str) {
        return;
    }

    async fn send(&self, uri: &str) -> Result<u16, E> {
        self.client.delete(uri).send().await?;
        Ok(200)
    }
}",
                );

                // WHEN
                let (transport, errors) = transport(&source);

                // THEN
                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(
                    transport.into_iter().collect::<Vec<_>>(),
                    [
                        ("delete", request("delete", 0, None, None)),
                        ("get", request("get", 1, None, Some(&[1]))),
                        ("head", request("delete", 0, None, None)),
                        ("options", request("options", 0, None, None)),
                        ("patch", request("patch", 0, None, None)),
                        ("post", request("post", 0, Some(1), Some(&[1]))),
                        ("put", request("put", 0, None, Some(&[1]))),
                    ]
                    .map(|(name, r)| (("Http".to_owned(), name.to_owned()), r))
                );
            }

            #[test]
            fn transport_of_impls_sending_different_requests_should_fail() {
                // GIVEN
                let source = source(
                    "impl Http for B {
    async fn get(&self, uri: &str) {
        self.client.get(uri);
    }
}

impl<T> Http for A<'_, T> {
    async fn get(&self, uri: &str) {
        self.client.get(uri);
    }

    async fn post(&self, uri: &str) {
        self.client.post(uri);
    }
}

impl Http for B {
    async fn post(&self, uri: &str) {
        self.client.put(uri);
    }
}",
                );

                // WHEN
                let (transport, errors) = transport(&source);

                // THEN
                assert_eq!(
                    errors,
                    [
                        "/api/src/lib.rs:55: `post`: the impls of `Http::post` send different requests"
                    ]
                );
                assert_eq!(transport.len(), 2);
            }

            #[test]
            fn transport_of_unreadable_requests_should_fail() {
                // GIVEN
                let source = source(
                    "impl Http for B {
    async fn get(&self, uri: &str) {
        self.client.get(uri);
        self.client.post(uri);
    }

    async fn put(&self, uri: &str) {
        self.client.put(\"/fixed\");
    }

    async fn post(&self, uri: &str, a: u8, b: u8) {
        self.client.post(uri).json(&a).json(&b);
    }
}",
                );

                // WHEN
                let (transport, errors) = transport(&source);

                // THEN
                assert_eq!(
                    errors,
                    [
                        "/api/src/lib.rs:39: `get`: a transport sending `get`, `post` requests",
                        "/api/src/lib.rs:44: `put`: a `put` to a URL that is no parameter",
                        "/api/src/lib.rs:48: `post`: a `post` with more than one JSON body"
                    ]
                );
                assert_eq!(transport.len(), 0);
            }

            #[test]
            fn transport_should_skip_impls_of_no_local_type_or_trait() {
                // GIVEN
                let source = source(
                    "impl Http for reqwest::Client {
    async fn get(&self, uri: &str) {
        self.get(uri);
    }
}

impl Http for &B {
    async fn get(&self, uri: &str) {
        self.client.get(uri);
    }
}

impl Inner for B {
    async fn get(&self, uri: &str) {
        self.client.get(uri);
    }
}

impl std::fmt::Debug for B {
    async fn get(&self, uri: &str) {
        self.client.get(uri);
    }
}",
                );

                // WHEN
                let (transport, errors) = transport(&source);

                // THEN
                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(transport.len(), 0);
            }

            #[test]
            fn transport_should_send_only_through_a_reqwest_client() {
                // GIVEN
                let source = source(
                    "impl<T> Http for A<'_, T> {
    async fn get(&self, uri: &str) {
        self.pair.get(uri);
        self.generic.get(uri);
        self.cycle.get(uri);
        self.missing.get(uri);
    }
}

impl Http for C {
    async fn get(&self, uri: &str) {
        self.client.get(uri);
    }
}",
                );

                // WHEN
                let (transport, errors) = transport(&source);

                // THEN
                assert_eq!(errors, Vec::<String>::new());
                assert_eq!(transport.len(), 0);
            }
        }
    }
}
