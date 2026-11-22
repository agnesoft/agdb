//! [`SourceCrate::lower_routes`]: the routes of the axum routers a server
//! serves, read from the expressions that build them.

use super::Io;
use super::Route;
use super::Routes;
use super::Segment;
use crate::Error;
use crate::ItemId;
use crate::MacroCall;
use crate::ModuleId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use crate::codegen::lower::TypeLowering;
use crate::codegen::lowered::Location;
use crate::codegen::lowered::Ty;
use crate::codegen::pass::subst;
use std::collections::HashMap;
use std::rc::Rc;
use syn::spanned::Spanned;
use syn::visit::Visit;

impl SourceCrate {
    /// The routes of the routers this crate, a server, passes to
    /// `axum::serve`, with the types of their handlers lowered from the
    /// `sources` (each with its name in the paths); the server's own data
    /// types they name are in [`Routes::module`].
    ///
    /// A router is `Router::new()` with `route`, `nest` and `merge`, the
    /// methods leaving its routes as they are (`layer`, `with_state`, ..),
    /// `if` (the routes of both branches), blocks and locals, and the calls
    /// of the functions returning one. What else makes a value, such as a
    /// router of another crate, adds no routes: a request to them is not
    /// known to be served. A path is a literal or a `format!`, whose
    /// arguments and any other expression are a [`Segment::Var`].
    ///
    /// A handler takes `State`, `Path`, `Query`, `Json` and the extractors of
    /// the crate: those implementing `FromRequestParts` are no part of the
    /// request, those implementing `FromRequest` take the body of their
    /// `Json::<T>::from_request`. It answers a `StatusCode`, a `Json`, a type
    /// of the crate implementing `IntoResponse` whose body is the
    /// `Json(self.field)` of it, or a tuple of a `StatusCode`, header arrays
    /// and one of these last, in `Result`s and type aliases.
    pub fn lower_routes(&self, sources: &[(&SourceCrate, &str)]) -> (Routes, Vec<Error>) {
        let mut reader = Reader {
            c: self,
            types: TypeLowering::new(self, sources),
            impls: impls(self),
            errors: Vec::new(),
            depth: 0,
        };
        let mut raw = Vec::new();

        for (id, f) in fns(self) {
            let mut calls = Vec::new();
            Serves {
                c: self,
                module: id.module(),
                calls: &mut calls,
            }
            .visit_block(&f.block);

            for arg in calls {
                let mut env = Env::default();
                let scope = Scope {
                    item: id,
                    module: id.module(),
                    name: f.sig.ident.to_string(),
                };
                reader.locals(&scope, &f.block, arg, &mut env);

                match reader.expr(&scope, arg, &mut env, false) {
                    Val::Routes(routes) => raw.extend(routes),
                    Val::Unknown => {
                        reader.error(&scope, arg, "the router served is not read".into())
                    }
                }
            }
        }

        if raw.is_empty() && reader.errors.is_empty() {
            reader.errors.push(Error::Lowering {
                path: self.file(self.module(self.root()).file()).path().to_owned(),
                line: 0,
                item: String::new(),
                message: "no router passed to `axum::serve`".into(),
            });
        }

        let mut routes: Vec<Route> = Vec::new();

        for r in raw {
            let mut path: Vec<Segment> = r
                .path
                .iter()
                .skip_while(|s| **s == Segment::Var)
                .cloned()
                .collect();

            if path.is_empty() && !r.path.is_empty() {
                path.push(Segment::Var);
            }

            if routes.iter().any(|o| o.verb == r.verb && o.path == path) {
                continue;
            }

            let (handler, at, io) = match r.handler {
                Handler::Fn(id, f, text) => (text, reader.location(id), reader.io(id, f)),
                Handler::Other(text, at) => {
                    (text, at, Err("the handler is not a function".to_owned()))
                }
            };
            routes.push(Route {
                verb: r.verb,
                path,
                handler,
                at,
                io,
            });
        }

        let Reader {
            types, mut errors, ..
        } = reader;
        let (module, mut lowering) = types.finish();
        errors.append(&mut lowering);
        (Routes { routes, module }, errors)
    }
}

/// The functions of the crate with their ids, those of impls too.
fn fns(c: &SourceCrate) -> Vec<(ItemId, &syn::ItemFn)> {
    c.item_ids()
        .filter_map(|id| match c.item(id) {
            syn::Item::Fn(f) => Some((id, f)),
            _ => None,
        })
        .collect()
}

/// An impl with its module and the path of the type it is for.
type Impl<'a> = (ModuleId, &'a syn::ItemImpl, &'a syn::Path);

/// The impls of the traits of other crates, by the type and the last
/// segment of the trait.
fn impls(c: &SourceCrate) -> HashMap<(ItemId, String), Impl<'_>> {
    let mut out = HashMap::new();

    for id in c.item_ids() {
        let syn::Item::Impl(i) = c.item(id) else {
            continue;
        };
        let (Some((path, _)), syn::Type::Path(ty)) = (&i.trait_, &*i.self_ty) else {
            continue;
        };

        if let (Resolved::Item(ty_id), Resolved::External(t)) = (
            c.resolve(id.module(), &ty.path, Namespace::Type),
            c.resolve(id.module(), path, Namespace::Type),
        ) && let Some(last) = t.last()
        {
            out.insert((ty_id, last.clone()), (id.module(), i, &ty.path));
        }
    }

    out
}

/// Whether `r` is an item of the crate `krate` whose path ends with `last`.
fn external(r: &Resolved, krate: &str, last: &str) -> bool {
    matches!(r, Resolved::External(p) if p.first().is_some_and(|f| f == krate) && p.last().is_some_and(|l| l == last))
}

/// The second argument of the calls of `axum::serve`.
struct Serves<'a, 'e> {
    c: &'a SourceCrate,
    module: ModuleId,
    calls: &'a mut Vec<&'e syn::Expr>,
}

impl<'e> Visit<'e> for Serves<'_, 'e> {
    fn visit_expr_call(&mut self, call: &'e syn::ExprCall) {
        if let syn::Expr::Path(p) = &*call.func
            && external(
                &self.c.resolve(self.module, &p.path, Namespace::Value),
                "axum",
                "serve",
            )
            && let Some(router) = call.args.iter().nth(1)
        {
            self.calls.push(router);
        }

        syn::visit::visit_expr_call(self, call);
    }
}

#[derive(Clone)]
struct Raw<'a> {
    verb: String,
    path: Vec<Segment>,
    handler: Handler<'a>,
}

#[derive(Clone)]
enum Handler<'a> {
    /// The function, with its path as written.
    Fn(ItemId, &'a syn::ItemFn, String),
    /// An expression other than a path of a function.
    Other(String, Location),
}

#[derive(Clone)]
enum Val<'a> {
    Routes(Vec<Raw<'a>>),
    /// Not known to be a router.
    Unknown,
}

impl<'a> Val<'a> {
    fn union(self, other: Val<'a>) -> Val<'a> {
        match (self, other) {
            (Val::Routes(mut a), Val::Routes(b)) => {
                a.extend(b);
                Val::Routes(a)
            }
            (Val::Routes(a), Val::Unknown) | (Val::Unknown, Val::Routes(a)) => Val::Routes(a),
            (Val::Unknown, Val::Unknown) => Val::Unknown,
        }
    }
}

/// The values of the locals in scope.
#[derive(Clone, Default)]
struct Env<'a>(HashMap<String, Val<'a>>);

struct Scope {
    /// The function the expressions are in.
    item: ItemId,
    module: ModuleId,
    name: String,
}

/// A type written in `module`, the generic parameters of `bindings` being
/// their types.
#[derive(Clone)]
struct TypeRef {
    module: ModuleId,
    ty: syn::Type,
    bindings: Rc<Vec<(String, TypeRef)>>,
}

struct Reader<'a> {
    c: &'a SourceCrate,
    types: TypeLowering<'a>,
    impls: HashMap<(ItemId, String), Impl<'a>>,
    errors: Vec<Error>,
    /// The functions being read for the routers they return.
    depth: usize,
}

const MAX_DEPTH: usize = 16;

impl<'a> Reader<'a> {
    fn location(&self, id: ItemId) -> Location {
        Location {
            path: self
                .c
                .file(self.c.module(id.module()).file())
                .path()
                .to_owned(),
            line: self.c.item(id).span().start().line,
        }
    }

    fn error(&mut self, scope: &Scope, node: &impl Spanned, message: String) {
        self.errors.push(Error::Lowering {
            path: self.location(scope.item).path,
            line: node.span().start().line,
            item: scope.name.clone(),
            message,
        });
    }

    /// Binds the locals of the statements of `block` before the one
    /// containing `until`.
    fn locals(&mut self, scope: &Scope, block: &syn::Block, until: &syn::Expr, env: &mut Env<'a>) {
        let until = until.span().start();

        for s in &block.stmts {
            if s.span().end() >= until {
                break;
            }

            self.stmt(scope, s, env, false);
        }
    }

    /// The value of the block, its locals bound in a copy of `env` and its
    /// assignments of the locals of `env` applied to it.
    fn block(
        &mut self,
        scope: &Scope,
        block: &syn::Block,
        env: &mut Env<'a>,
        maybe: bool,
    ) -> Val<'a> {
        let mut inner = env.clone();
        let mut value = Val::Unknown;
        let mut shadowed = Vec::new();

        for s in &block.stmts {
            if let syn::Stmt::Local(local) = s {
                shadowed.extend(bound(&local.pat));
            }

            value = self.stmt(scope, s, &mut inner, maybe);
        }

        for (name, v) in env.0.iter_mut() {
            if !shadowed.contains(name)
                && let Some(changed) = inner.0.remove(name)
            {
                *v = changed;
            }
        }

        value
    }

    /// The value of the statement, an expression without `;`, its locals
    /// bound in `env`.
    fn stmt(&mut self, scope: &Scope, s: &syn::Stmt, env: &mut Env<'a>, maybe: bool) -> Val<'a> {
        match s {
            syn::Stmt::Local(local) => {
                let value = match &local.init {
                    Some(init) => self.expr(scope, &init.expr, env, maybe),
                    None => Val::Unknown,
                };

                if let Some(name) = bound(&local.pat) {
                    env.0.insert(name, value);
                }

                Val::Unknown
            }
            syn::Stmt::Expr(e, None) => self.expr(scope, e, env, maybe),
            syn::Stmt::Expr(e, Some(_)) => {
                self.expr(scope, e, env, maybe);
                Val::Unknown
            }
            syn::Stmt::Item(_) | syn::Stmt::Macro(_) => Val::Unknown,
        }
    }

    /// The value of the expression, its assignments of the locals of `env`
    /// applied to it, those that may not happen (`maybe`, in loops and
    /// `if`s) adding to the routes of the local.
    fn expr(&mut self, scope: &Scope, e: &syn::Expr, env: &mut Env<'a>, maybe: bool) -> Val<'a> {
        match e {
            syn::Expr::Paren(p) => self.expr(scope, &p.expr, env, maybe),
            syn::Expr::Reference(r) => self.expr(scope, &r.expr, env, maybe),
            syn::Expr::Try(t) => self.expr(scope, &t.expr, env, maybe),
            syn::Expr::Await(a) => self.expr(scope, &a.base, env, maybe),
            syn::Expr::Block(b) => self.block(scope, &b.block, env, maybe),
            syn::Expr::If(i) => {
                let then = self.block(scope, &i.then_branch, env, true);
                let els = match &i.else_branch {
                    Some((_, els)) => self.expr(scope, els, env, true),
                    None => Val::Unknown,
                };
                then.union(els)
            }
            syn::Expr::ForLoop(syn::ExprForLoop { body, .. })
            | syn::Expr::While(syn::ExprWhile { body, .. })
            | syn::Expr::Loop(syn::ExprLoop { body, .. }) => {
                self.block(scope, body, env, true);
                Val::Unknown
            }
            syn::Expr::Assign(a) => {
                if let syn::Expr::Path(p) = &*a.left
                    && let Some(name) = p.path.get_ident().map(ToString::to_string)
                    && let Some(old) = env.0.get(&name).cloned()
                {
                    let value = self.expr(scope, &a.right, env, maybe);
                    let value = match maybe {
                        true => old.union(value),
                        false => value,
                    };
                    env.0.insert(name, value);
                }

                Val::Unknown
            }
            syn::Expr::Path(p) => p
                .path
                .get_ident()
                .and_then(|name| env.0.get(&name.to_string()).cloned())
                .unwrap_or(Val::Unknown),
            syn::Expr::Call(call) => self.call(scope, call, env, maybe),
            syn::Expr::MethodCall(m) => self.method_call(scope, m, env, maybe),
            _ => Val::Unknown,
        }
    }

    fn call(
        &mut self,
        scope: &Scope,
        call: &syn::ExprCall,
        env: &mut Env<'a>,
        maybe: bool,
    ) -> Val<'a> {
        let syn::Expr::Path(p) = &*call.func else {
            return Val::Unknown;
        };

        match self.c.resolve(scope.module, &p.path, Namespace::Value) {
            Resolved::External(v)
                if v.first().is_some_and(|f| f == "axum")
                    && v.ends_with(&["Router".into(), "new".into()]) =>
            {
                Val::Routes(Vec::new())
            }
            Resolved::External(v)
                if v.ends_with(&["Result".into(), "Ok".into()]) && call.args.len() == 1 =>
            {
                self.expr(scope, &call.args[0], env, maybe)
            }
            Resolved::Item(id) => self.returned(id),
            _ => Val::Unknown,
        }
    }

    /// The routers the function returns, if its return type names an axum
    /// `Router`.
    fn returned(&mut self, id: ItemId) -> Val<'a> {
        let syn::Item::Fn(f) = self.c.item(id) else {
            return Val::Unknown;
        };
        let mut names = Names {
            c: self.c,
            module: id.module(),
            found: false,
        };
        names.visit_return_type(&f.sig.output);

        if !names.found || self.depth >= MAX_DEPTH {
            return Val::Unknown;
        }

        self.depth += 1;
        let scope = Scope {
            item: id,
            module: id.module(),
            name: f.sig.ident.to_string(),
        };
        let value = self.block(&scope, &f.block, &mut Env::default(), false);
        self.depth -= 1;
        value
    }

    fn method_call(
        &mut self,
        scope: &Scope,
        m: &syn::ExprMethodCall,
        env: &mut Env<'a>,
        maybe: bool,
    ) -> Val<'a> {
        let Val::Routes(mut routes) = self.expr(scope, &m.receiver, env, maybe) else {
            return Val::Unknown;
        };
        let args: Vec<&syn::Expr> = m.args.iter().collect();

        match (m.method.to_string().as_str(), args.as_slice()) {
            ("route", [path, method_router]) => {
                let path = self.path(scope, path);

                for (verb, handler) in self.method_router(scope, method_router) {
                    routes.push(Raw {
                        verb,
                        path: path.clone(),
                        handler,
                    });
                }
            }
            ("nest", [prefix, router]) => {
                let prefix = self.path(scope, prefix);

                if let Val::Routes(nested) = self.expr(scope, router, env, maybe) {
                    routes.extend(nested.into_iter().map(|mut r| {
                        r.path = prefix.iter().cloned().chain(r.path).collect();
                        r
                    }));
                }
            }
            ("merge", [router]) => {
                if let Val::Routes(merged) = self.expr(scope, router, env, maybe) {
                    routes.extend(merged);
                }
            }
            (
                "layer" | "route_layer" | "with_state" | "fallback" | "fallback_service"
                | "nest_service" | "route_service",
                _,
            ) => {}
            (method, _) => {
                self.error(
                    scope,
                    &m.method,
                    format!("unsupported the router method `{method}`"),
                );
                return Val::Unknown;
            }
        }

        Val::Routes(routes)
    }

    /// The methods and handlers of a `routing::get(handler).post(..)`.
    fn method_router(&mut self, scope: &Scope, e: &syn::Expr) -> Vec<(String, Handler<'a>)> {
        let (verb, handler, rest) = match e {
            syn::Expr::Call(call) if call.args.len() == 1 => {
                let syn::Expr::Path(p) = &*call.func else {
                    return self.unsupported_method_router(scope, e);
                };

                match self.c.resolve(scope.module, &p.path, Namespace::Value) {
                    Resolved::External(path)
                        if path.first().is_some_and(|f| f == "axum")
                            && path.len() >= 2
                            && path[path.len() - 2] == "routing" =>
                    {
                        (path[path.len() - 1].clone(), &call.args[0], Vec::new())
                    }
                    _ => return self.unsupported_method_router(scope, e),
                }
            }
            syn::Expr::MethodCall(m) if m.args.len() == 1 => (
                m.method.to_string(),
                &m.args[0],
                self.method_router(scope, &m.receiver),
            ),
            e => return self.unsupported_method_router(scope, e),
        };
        let handler = match handler {
            syn::Expr::Path(p) => match self.c.resolve(scope.module, &p.path, Namespace::Value) {
                Resolved::Item(id) => match self.c.item(id) {
                    syn::Item::Fn(f) => Handler::Fn(id, f, path_name(&p.path)),
                    _ => self.other(scope, handler),
                },
                _ => self.other(scope, handler),
            },
            handler => self.other(scope, handler),
        };
        let mut out = rest;
        out.push((verb, handler));
        out
    }

    fn other(&self, scope: &Scope, handler: &syn::Expr) -> Handler<'a> {
        let mut at = self.location(scope.item);
        at.line = handler.span().start().line;
        let text = match handler {
            syn::Expr::Path(p) => path_name(&p.path),
            _ => "an expression".to_owned(),
        };
        Handler::Other(text, at)
    }

    fn unsupported_method_router(
        &mut self,
        scope: &Scope,
        e: &syn::Expr,
    ) -> Vec<(String, Handler<'a>)> {
        self.error(scope, e, "unsupported a method router".into());
        Vec::new()
    }

    /// The segments of a route path: a literal or a `format!`.
    fn path(&mut self, scope: &Scope, e: &syn::Expr) -> Vec<Segment> {
        match e {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => segments::<()>(&[Piece::Text(s.value())]),
            syn::Expr::Reference(r) => self.path(scope, &r.expr),
            syn::Expr::Macro(m) => match self.c.macro_call(scope.module, &m.mac) {
                Ok(MacroCall::Format {
                    format: Some(format),
                    ..
                }) if m.mac.path.is_ident("format") => {
                    segments(&format_pieces(&format.string.value()))
                }
                _ => vec![Segment::Var],
            },
            _ => vec![Segment::Var],
        }
    }

    /// The request and response of the handler.
    fn io(&mut self, id: ItemId, f: &syn::ItemFn) -> Result<Io, String> {
        let mut io = Io::default();

        for input in &f.sig.inputs {
            let syn::FnArg::Typed(arg) = input else {
                return Err("a handler with a receiver".into());
            };
            let ty = TypeRef {
                module: id.module(),
                ty: (*arg.ty).clone(),
                bindings: Rc::default(),
            };
            self.extractor(&ty, &mut io)?;
        }

        io.response = match &f.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => self.response(&TypeRef {
                module: id.module(),
                ty: (**ty).clone(),
                bindings: Rc::default(),
            })?,
        };
        Ok(io)
    }

    fn extractor(&mut self, ty: &TypeRef, io: &mut Io) -> Result<(), String> {
        let syn::Type::Path(p) = &ty.ty else {
            return Err(format!("unsupported the extractor `{}`", text(&ty.ty)));
        };
        let args = args(ty, &p.path);

        match self.c.resolve(ty.module, &p.path, Namespace::Type) {
            Resolved::External(path) if path.first().is_some_and(|f| f == "axum") => {
                match (path.last().map(String::as_str), args.as_slice()) {
                    (
                        Some(
                            "State" | "HeaderMap" | "Extension" | "ConnectInfo" | "Method" | "Uri"
                            | "OriginalUri",
                        ),
                        _,
                    ) => {}
                    (Some("Path"), [arg]) => {
                        io.params = match self.lower(arg) {
                            Ty::Tuple(params) => params,
                            param => vec![param],
                        }
                    }
                    (Some("Query"), [arg]) => io.query = Some(self.lower(arg)),
                    (Some("Json"), [arg]) => io.body = Some(self.lower(arg)),
                    _ => return Err(format!("unsupported the extractor `{}`", text(&ty.ty))),
                }
            }
            Resolved::Item(s) if self.impls.contains_key(&(s, "FromRequestParts".into())) => {}
            Resolved::Item(s)
                if let Some(&(module, i, path)) = self.impls.get(&(s, "FromRequest".into())) =>
            {
                let body = self.request_body(module, i, path, &args)?;
                io.body = Some(self.lower(&body));
            }
            _ => return Err(format!("unsupported the extractor `{}`", text(&ty.ty))),
        }

        Ok(())
    }

    /// The `T` of the `Json::<T>::from_request` of the impl for the type
    /// with the `args`.
    fn request_body(
        &mut self,
        module: ModuleId,
        i: &syn::ItemImpl,
        path: &syn::Path,
        args: &[TypeRef],
    ) -> Result<TypeRef, String> {
        let bindings = impl_bindings(i, path, module, args);
        let mut found = FromRequests {
            c: self.c,
            module,
            found: Vec::new(),
        };
        found.visit_item_impl(i);

        match found.found.as_slice() {
            [ty] => Ok(TypeRef {
                module,
                ty: ty.clone(),
                bindings: Rc::new(bindings),
            }),
            _ => Err("an extractor without one `Json::<T>::from_request`".into()),
        }
    }

    /// The body of a response of the type, none for a status or headers.
    fn response(&mut self, ty: &TypeRef) -> Result<Option<Ty>, String> {
        match &ty.ty {
            syn::Type::Paren(p) => self.response(&TypeRef {
                ty: (*p.elem).clone(),
                ..ty.clone()
            }),
            syn::Type::Array(_) => Ok(None),
            syn::Type::Tuple(t) if !t.elems.is_empty() => {
                let at = |elem: &syn::Type| TypeRef {
                    ty: elem.clone(),
                    ..ty.clone()
                };
                let elems: Vec<&syn::Type> = t.elems.iter().collect();

                if !self.is_status(&at(elems[0])) {
                    return Err(format!(
                        "a response tuple `{}` not starting with a `StatusCode`",
                        text(&ty.ty)
                    ));
                }

                let rest = &elems[1..];
                let mut body = None;

                for (i, elem) in rest.iter().enumerate() {
                    let Some(answered) = self.response(&at(elem))? else {
                        continue;
                    };

                    if i + 1 != rest.len() {
                        return Err(format!(
                            "a response tuple `{}` with a body before its end",
                            text(&ty.ty)
                        ));
                    }

                    body = Some(answered);
                }

                Ok(body)
            }
            syn::Type::Path(p) => {
                if let Some(bound) = p
                    .path
                    .get_ident()
                    .and_then(|i| ty.bindings.iter().find(|(n, _)| i == n))
                {
                    return self.response(&bound.1.clone());
                }

                let args = args(ty, &p.path);

                match self.c.resolve(ty.module, &p.path, Namespace::Type) {
                    Resolved::External(path) => {
                        match (path.last().map(String::as_str), args.as_slice()) {
                            (Some("StatusCode"), []) => Ok(None),
                            (Some("Result"), [ok, ..]) => self.response(ok),
                            (Some("Json"), [body]) if path.first().is_some_and(|f| f == "axum") => {
                                Ok(Some(self.lower(body)))
                            }
                            _ => Err(format!("unsupported the response `{}`", text(&ty.ty))),
                        }
                    }
                    Resolved::Item(id) => match self.c.item(id) {
                        syn::Item::Type(alias) => {
                            let bindings = alias_bindings(alias, id.module(), &args);
                            self.alias_response(id, alias, bindings)
                        }
                        syn::Item::Struct(s)
                            if let Some(&(module, i, _)) =
                                self.impls.get(&(id, "IntoResponse".into())) =>
                        {
                            self.response_body(id, s, module, i, &args).map(Some)
                        }
                        _ => Err(format!("unsupported the response `{}`", text(&ty.ty))),
                    },
                    _ => Err(format!("unsupported the response `{}`", text(&ty.ty))),
                }
            }
            _ => Err(format!("unsupported the response `{}`", text(&ty.ty))),
        }
    }

    fn alias_response(
        &mut self,
        id: ItemId,
        alias: &syn::ItemType,
        bindings: Vec<(String, TypeRef)>,
    ) -> Result<Option<Ty>, String> {
        self.response(&TypeRef {
            module: id.module(),
            ty: (*alias.ty).clone(),
            bindings: Rc::new(bindings),
        })
    }

    fn is_status(&self, ty: &TypeRef) -> bool {
        match &ty.ty {
            syn::Type::Path(p) => {
                matches!(self.c.resolve(ty.module, &p.path, Namespace::Type), Resolved::External(path) if path.last().is_some_and(|l| l == "StatusCode"))
            }
            _ => false,
        }
    }

    /// The type of the `Json(self.field)` of the `IntoResponse` impl of the
    /// struct with the `args`.
    fn response_body(
        &mut self,
        id: ItemId,
        s: &syn::ItemStruct,
        module: ModuleId,
        i: &syn::ItemImpl,
        args: &[TypeRef],
    ) -> Result<Ty, String> {
        let mut found = JsonFields {
            c: self.c,
            module,
            found: Vec::new(),
        };
        found.visit_item_impl(i);
        let [field] = found.found.as_slice() else {
            return Err(format!(
                "an `IntoResponse` of `{}` without one `Json(self.field)`",
                s.ident
            ));
        };
        let Some(field) = s
            .fields
            .iter()
            .find(|f| f.ident.as_ref().is_some_and(|i| i == field))
        else {
            return Err(format!("no field `{field}` in `{}`", s.ident));
        };
        let bindings = s
            .generics
            .type_params()
            .map(|p| p.ident.to_string())
            .zip(args.iter().cloned())
            .collect();
        Ok(self.lower(&TypeRef {
            module: id.module(),
            ty: field.ty.clone(),
            bindings: Rc::new(bindings),
        }))
    }

    /// The lowered type, the generic parameters bound replaced by their
    /// types.
    fn lower(&mut self, ty: &TypeRef) -> Ty {
        if let syn::Type::Path(p) = &ty.ty
            && let Some((_, bound)) = p
                .path
                .get_ident()
                .and_then(|i| ty.bindings.iter().find(|(n, _)| i == n))
        {
            return self.lower(&bound.clone());
        }

        let names: Vec<String> = ty.bindings.iter().map(|(n, _)| n.clone()).collect();
        let lowered = self.types.ty(ty.module, &names, &ty.ty);
        let params: Vec<(String, Ty)> = ty
            .bindings
            .iter()
            .map(|(n, b)| (n.clone(), self.lower(b)))
            .collect();
        subst(&lowered, &Ty::SelfTy, &params)
    }
}

/// The type arguments of the last segment of the path, written where `ty`
/// is.
fn args(ty: &TypeRef, path: &syn::Path) -> Vec<TypeRef> {
    match path.segments.last().map(|s| &s.arguments) {
        Some(syn::PathArguments::AngleBracketed(a)) => a
            .args
            .iter()
            .filter_map(|a| match a {
                syn::GenericArgument::Type(t) => Some(TypeRef {
                    ty: t.clone(),
                    ..ty.clone()
                }),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The types of the type parameters of an alias: the arguments, else the
/// defaults.
fn alias_bindings(
    alias: &syn::ItemType,
    module: ModuleId,
    args: &[TypeRef],
) -> Vec<(String, TypeRef)> {
    let mut out: Vec<(String, TypeRef)> = Vec::new();

    for (i, p) in alias.generics.type_params().enumerate() {
        let ty = match (args.get(i), &p.default) {
            (Some(arg), _) => arg.clone(),
            (None, Some(default)) => TypeRef {
                module,
                ty: default.1.clone(),
                bindings: Rc::new(out.clone()),
            },
            (None, None) => continue,
        };
        out.push((p.ident.to_string(), ty));
    }

    out
}

/// The types of the type parameters of an impl for a type with the `args`:
/// those the type is written with in the impl.
fn impl_bindings(
    i: &syn::ItemImpl,
    path: &syn::Path,
    module: ModuleId,
    bound: &[TypeRef],
) -> Vec<(String, TypeRef)> {
    let written = args(
        &TypeRef {
            module,
            ty: (*i.self_ty).clone(),
            bindings: Rc::default(),
        },
        path,
    );
    let params: Vec<String> = i
        .generics
        .type_params()
        .map(|p| p.ident.to_string())
        .collect();

    written
        .iter()
        .zip(bound)
        .filter_map(|(w, a)| match &w.ty {
            syn::Type::Path(w) => w
                .path
                .get_ident()
                .map(ToString::to_string)
                .filter(|n| params.contains(n))
                .map(|n| (n, a.clone())),
            _ => None,
        })
        .collect()
}

/// Whether a path of the return type names an axum `Router`.
struct Names<'a> {
    c: &'a SourceCrate,
    module: ModuleId,
    found: bool,
}

impl Visit<'_> for Names<'_> {
    fn visit_path(&mut self, path: &syn::Path) {
        self.found |= external(
            &self.c.resolve(self.module, path, Namespace::Type),
            "axum",
            "Router",
        );
        syn::visit::visit_path(self, path);
    }
}

/// The `T`s of the `Json::<T>::from_request` calls.
struct FromRequests<'a> {
    c: &'a SourceCrate,
    module: ModuleId,
    found: Vec<syn::Type>,
}

impl Visit<'_> for FromRequests<'_> {
    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        if let syn::Expr::Path(p) = &*call.func
            && let [json, from] = p.path.segments.iter().collect::<Vec<_>>().as_slice()
            && from.ident == "from_request"
            && let syn::PathArguments::AngleBracketed(a) = &json.arguments
            && let [syn::GenericArgument::Type(t)] = a.args.iter().collect::<Vec<_>>().as_slice()
            && external(
                &self
                    .c
                    .resolve(self.module, &json.ident.clone().into(), Namespace::Type),
                "axum",
                "Json",
            )
        {
            self.found.push(t.clone());
        }

        syn::visit::visit_expr_call(self, call);
    }
}

/// The fields of the `Json(self.field)` calls.
struct JsonFields<'a> {
    c: &'a SourceCrate,
    module: ModuleId,
    found: Vec<String>,
}

impl Visit<'_> for JsonFields<'_> {
    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        if let syn::Expr::Path(p) = &*call.func
            && external(
                &self.c.resolve(self.module, &p.path, Namespace::Value),
                "axum",
                "Json",
            )
            && let [syn::Expr::Field(f)] = call.args.iter().collect::<Vec<_>>().as_slice()
            && let syn::Expr::Path(base) = &*f.base
            && base.path.is_ident("self")
            && let syn::Member::Named(name) = &f.member
        {
            self.found.push(name.to_string());
        }

        syn::visit::visit_expr_call(self, call);
    }
}

/// The name a `let` pattern binds.
fn bound(pat: &syn::Pat) -> Option<String> {
    match pat {
        syn::Pat::Ident(i) if i.subpat.is_none() => Some(i.ident.to_string()),
        syn::Pat::Type(t) => bound(&t.pat),
        _ => None,
    }
}

fn path_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn text(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => path_name(&p.path),
        syn::Type::Reference(r) => format!("&{}", text(&r.elem)),
        syn::Type::Paren(p) => text(&p.elem),
        syn::Type::Tuple(t) => format!(
            "({})",
            t.elems.iter().map(text).collect::<Vec<_>>().join(", ")
        ),
        syn::Type::Array(a) => format!("[{}; _]", text(&a.elem)),
        syn::Type::ImplTrait(_) => "impl ..".to_owned(),
        _ => "_".to_owned(),
    }
}

/// A part of a path: text, or a value written into it.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Piece<T> {
    Text(String),
    Value(T),
}

/// The pieces of a `format!` string: its text and, for each `{..}`, a
/// value.
fn format_pieces(string: &str) -> Vec<Piece<()>> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut chars = string.chars().peekable();

    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('{', Some('{')) | ('}', Some('}')) => {
                chars.next();
                text.push(c);
            }
            ('{', _) => {
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                }

                out.push(Piece::Text(std::mem::take(&mut text)));
                out.push(Piece::Value(()));
            }
            (c, _) => text.push(c),
        }
    }

    out.push(Piece::Text(text));
    out
}

/// The segments of a route path made of the pieces: `{name}` in the text is
/// a parameter, `{*name}` the rest, a value a [`Segment::Var`] as is a
/// segment with one.
fn segments<T>(pieces: &[Piece<T>]) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut current: Vec<&Piece<T>> = Vec::new();
    let mut texts: Vec<String> = Vec::new();

    let flush = |current: &mut Vec<&Piece<T>>, texts: &mut Vec<String>, out: &mut Vec<Segment>| {
        let text = texts.concat();
        let has_value = current.iter().any(|p| matches!(p, Piece::Value(_)));

        if has_value {
            out.push(Segment::Var);
        } else if !text.is_empty() || !out.is_empty() {
            out.push(
                match text.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
                    Some(name) => match name.strip_prefix('*') {
                        Some(rest) => Segment::Rest(rest.to_owned()),
                        None => Segment::Param(name.to_owned()),
                    },
                    None => Segment::Lit(text),
                },
            );
        }

        current.clear();
        texts.clear();
    };

    for piece in pieces {
        match piece {
            Piece::Text(t) => {
                let mut parts = t.split('/');
                let first = parts.next().unwrap_or_default();
                texts.push(first.to_owned());
                current.push(piece);

                for part in parts {
                    flush(&mut current, &mut texts, &mut out);
                    texts.push(part.to_owned());
                    current.push(piece);
                }
            }
            Piece::Value(_) => current.push(piece),
        }
    }

    flush(&mut current, &mut texts, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::routes::path_text;
    use crate::codegen::test_utilities::ROUTES_API;
    use crate::codegen::test_utilities::ROUTES_SERVER;
    use crate::codegen::test_utilities::served;

    const IMPORTS: &str = "use axum::Json;
use axum::Router;
use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::Request;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing;
";

    /// The routes the server serves, each as `verb path handler: io`, then
    /// its errors.
    fn routes(server: &str) -> Vec<String> {
        let (routes, errors) = served(ROUTES_API, &format!("{IMPORTS}\n{server}"));
        let ty = |t: &Option<Ty>| t.as_ref().map_or("-".to_owned(), print::ty);

        routes
            .routes
            .iter()
            .map(|r| {
                let io = match &r.io {
                    Ok(io) => format!(
                        "({}) ?{} {} -> {}",
                        io.params
                            .iter()
                            .map(print::ty)
                            .collect::<Vec<_>>()
                            .join(", "),
                        ty(&io.query),
                        ty(&io.body),
                        ty(&io.response)
                    ),
                    Err(m) => m.clone(),
                };
                format!("{} {} {}: {io}", r.verb, path_text(&r.path), r.handler)
            })
            .chain(errors)
            .collect()
    }

    mod source_crate {
        use super::*;

        mod lower_routes {
            use super::*;

            #[test]
            fn lower_routes_should_read_the_routes_of_the_served_router() {
                // GIVEN
                let server = ROUTES_SERVER;

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    [
                        "post /api/login login: () ?- Login -> str",
                        "get /api/items/{owner}/list items: (str) ?- - -> Items",
                        "post /api/items/{owner}/add add: (str) ?AddParams - -> -",
                        "delete /api/items/{owner}/{id} remove: (str, u64) ?- - -> -",
                        "post /api/items/{owner}/exec exec: (str) ?- Vec<Item> -> Vec<Item>",
                        "get /api/internal internal: () ?- - -> -"
                    ]
                );
            }

            #[test]
            fn lower_routes_should_follow_the_locals_and_their_assignments() {
                // GIVEN
                let server = "#[tokio::main]
async fn main() {
    let late;
    let (a, b) = (1, 2);
    let app: Router = routes();
    axum::serve(listener, app).await;
}

fn routes() -> Router {
    let mut router = Router::new().route(\"/a\", routing::get(h));
    let kept = Router::new().route(\"/kept\", routing::get(h));
    unknown = 1;
    router.field = 1;
    fn inner() {}
    println!(\"x\");
    a + b;
    for i in 0..2 {
        router = router.route(\"/for\", routing::get(h));
    }
    while false {
        router = router.route(\"/while\", routing::get(h));
    }
    loop {
        router = router.route(\"/loop\", routing::get(h));
    }
    if true {
        router = router.route(\"/if\", routing::get(h))
    } else if false {
        router = router.route(\"/else_if\", routing::get(h));
    } else {
        router = Router::new();
    }
    {
        let kept = Router::new();
        router = router.route(\"/block\", routing::get(h));
    }
    router = router.merge(kept).route(\"/after\", routing::get(h));
    router
}

async fn h() -> StatusCode {
    StatusCode::OK
}";

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    [
                        "get /a h: () ?- - -> -",
                        "get /for h: () ?- - -> -",
                        "get /while h: () ?- - -> -",
                        "get /loop h: () ?- - -> -",
                        "get /if h: () ?- - -> -",
                        "get /else_if h: () ?- - -> -",
                        "get /block h: () ?- - -> -",
                        "get /kept h: () ?- - -> -",
                        "get /after h: () ?- - -> -"
                    ]
                );
            }

            #[test]
            fn lower_routes_should_read_the_router_expressions() {
                // GIVEN
                let server = "#[tokio::main]
async fn main() {
    axum::serve(listener, (&made()?).await).await;
}

const PATH: &str = \"/const\";

struct Wrapper(u8);

fn made() -> Result<Router, E> {
    let base = Router::new().route(\"/m\", routing::get(h).post(h));
    Ok(Router::new()
        .nest(&format!(\"/{}/n\", 1), base)
        .nest(\"/unknown\", unknown())
        .merge(if true { listed() } else { 1 })
        .merge(if true { 1 } else { 2 })
        .merge(if true { Router::new() })
        .merge(Wrapper(1))
        .merge(not_router())
        .merge(recursive())
        .merge(a::b)
        .merge(1)
        .merge((f)())
        .merge(Ok())
        .route(concat!(\"/c\", \"d\"), routing::get(h))
        .route(PATH, routing::get(h))
        .layer(x)
        .route_layer(x)
        .with_state(1)
        .fallback(h)
        .fallback_service(x)
        .nest_service(\"/s\", x)
        .route_service(\"/r\", x))
}

fn listed() -> Router {
    Router::new().route(\"/l\", routing::get(h))
}

fn not_router() -> u8 {
    1
}

fn recursive() -> Router {
    recursive()
}

async fn h() -> StatusCode {
    StatusCode::OK
}";

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    [
                        "get /n/m h: () ?- - -> -",
                        "post /n/m h: () ?- - -> -",
                        "get /l h: () ?- - -> -",
                        "get /{..} h: () ?- - -> -"
                    ]
                );
            }

            #[test]
            fn lower_routes_unsupported_method_routers_should_fail() {
                // GIVEN
                let server = "#[tokio::main]
async fn main() {
    axum::serve(listener, app()).await;
}

struct Wrapper;

fn app() -> Router {
    Router::new()
        .route(\"/a\", (routing::get)(h))
        .route(\"/b\", other::get(h))
        .route(\"/c\", routing::get(h).post(h, 1))
        .route(\"/d\", make_router)
        .route(\"/e\", routing::get(Wrapper))
        .route(\"/f\", routing::get(axum::handler))
        .route(\"/g\", routing::get(|| async {}))
}

async fn h() -> StatusCode {
    StatusCode::OK
}";

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    [
                        "get /e Wrapper: the handler is not a function",
                        "get /f axum::handler: the handler is not a function",
                        "get /g an expression: the handler is not a function",
                        "/s/src/main.rs:24: `app`: unsupported a method router",
                        "/s/src/main.rs:25: `app`: unsupported a method router",
                        "/s/src/main.rs:26: `app`: unsupported a method router",
                        "/s/src/main.rs:27: `app`: unsupported a method router"
                    ]
                );
            }

            #[test]
            fn lower_routes_unsupported_router_method_should_fail() {
                // GIVEN
                let server = "#[tokio::main]
async fn main() {
    axum::serve(listener, Router::new().into_make_service()).await;
}";

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    [
                        "/s/src/main.rs:17: `main`: unsupported the router method `into_make_service`",
                        "/s/src/main.rs:17: `main`: the router served is not read"
                    ]
                );
            }

            #[test]
            fn lower_routes_without_served_router_should_fail() {
                // GIVEN
                let server = "fn main() {
    axum::serve(listener);
    other::serve(listener, Router::new());
}";

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    ["/s/src/main.rs:0: ``: no router passed to `axum::serve`"]
                );
            }

            #[test]
            fn lower_routes_should_read_the_requests_and_responses_of_the_handlers() {
                // GIVEN
                let server = "#[tokio::main]
async fn main() {
    axum::serve(listener, app()).await;
}

fn app() -> Router {
    Router::new()
        .route(\"/none\", routing::get(none))
        .route(\"/paren\", routing::get(paren))
        .route(\"/generic\", routing::get(generic))
        .route(\"/alias\", routing::get(alias))
        .route(\"/lifetime\", routing::get(lifetime))
        .route(\"/batch\", routing::post(batch))
        .route(\"/pairs\", routing::post(pairs))
        .route(\"/parts\", routing::get(parts))
}

trait Local {}

impl Local for List<u8> {}

impl<T> List<T> {}

impl<T> IntoResponse for &List<T> {}

struct List<T> {
    items: Vec<T>,
}

impl<T> IntoResponse for List<T> {
    fn into_response(self) -> Response {
        Json(1);
        Json(self.items).into_response()
    }
}

struct Batch<T>(T);

impl<S, T> FromRequest<S> for Batch<Vec<T>> {
    async fn from_request(request: Request, state: &S) -> Result<Self, StatusCode> {
        Json::<T>::from_request(request, state).await
    }
}

struct Pairs<T>(T);

impl<S, T> FromRequest<S> for Pairs<(T, u8)> {
    async fn from_request(request: Request, state: &S) -> Result<Self, StatusCode> {
        Json::<T>::from_request(request, state).await
    }
}

type Pair<A, B> = Json<(A, B)>;

type Life<'a, T> = Json<T>;

async fn none(headers: HeaderMap) {}

async fn paren() -> (StatusCode) {
    StatusCode::OK
}

async fn generic() -> List<u8> {
    List { items: Vec::new() }
}

async fn alias() -> Pair<u8> {
    Json((1, 2))
}

async fn lifetime() -> Life<'static, u8> {
    Json(1)
}

async fn batch(Batch(items): Batch<Vec<u8>>) -> StatusCode {
    StatusCode::OK
}

async fn pairs(p: Pairs<(u8, u8)>) -> StatusCode {
    StatusCode::OK
}

async fn parts(
    State(_s): State<u8>,
    axum::Extension(_e): axum::Extension<u8>,
    axum::extract::ConnectInfo(_c): axum::extract::ConnectInfo<u8>,
    _m: axum::http::Method,
    _u: axum::http::Uri,
    axum::extract::OriginalUri(_o): axum::extract::OriginalUri,
) -> StatusCode {
    StatusCode::OK
}";

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    [
                        "get /none none: () ?- - -> -",
                        "get /paren paren: () ?- - -> -",
                        "get /generic generic: () ?- - -> Vec<u8>",
                        "get /alias alias: () ?- - -> (u8, ?)",
                        "get /lifetime lifetime: () ?- - -> u8",
                        "post /batch batch: () ?- ? -> -",
                        "post /pairs pairs: () ?- ? -> -",
                        "get /parts parts: () ?- - -> -"
                    ]
                );
            }

            #[test]
            fn lower_routes_unsupported_handlers_should_fail() {
                // GIVEN
                let server = "#[tokio::main]
async fn main() {
    axum::serve(listener, app()).await;
}

fn app() -> Router {
    Router::new()
        .route(\"/receiver\", routing::get(receiver))
        .route(\"/tuple\", routing::get(tuple))
        .route(\"/paren\", routing::get(paren))
        .route(\"/array\", routing::get(array))
        .route(\"/bare\", routing::get(bare))
        .route(\"/form\", routing::get(form))
        .route(\"/plain\", routing::get(plain))
        .route(\"/no_json\", routing::get(no_json))
        .route(\"/two_json\", routing::get(two_json))
        .route(\"/first\", routing::get(first))
        .route(\"/elem\", routing::get(elem))
        .route(\"/early\", routing::get(early))
        .route(\"/string\", routing::get(string))
        .route(\"/local\", routing::get(local))
        .route(\"/unknown\", routing::get(unknown))
        .route(\"/opaque\", routing::get(opaque))
        .route(\"/status_ref\", routing::get(status_ref))
        .route(\"/no_field\", routing::get(no_field))
        .route(\"/missing\", routing::get(missing))
}

struct Plain;

struct NoJson;

impl<S> FromRequest<S> for NoJson {}

struct TwoJson;

impl<S> FromRequest<S> for TwoJson {
    async fn from_request(request: Request, state: &S) -> Result<Self, StatusCode> {
        Json::<u8>::from_request(request, state).await;
        Json::<u16>::from_request(request, state).await
    }
}

struct NoField;

impl IntoResponse for NoField {}

struct Missing {
    value: u8,
}

impl IntoResponse for Missing {
    fn into_response(self) -> Response {
        Json(self.other).into_response()
    }
}

async fn receiver(self) {}

async fn tuple((a, b): (u8, u8)) {}

async fn paren(a: (u8)) {}

async fn array(a: [u8; 2]) {}

async fn bare(f: fn()) {}

async fn form(axum::Form(f): axum::Form<u8>) {}

async fn plain(p: Plain) {}

async fn no_json(n: NoJson) {}

async fn two_json(t: TwoJson) {}

async fn first() -> (Json<u8>, StatusCode) {}

async fn elem() -> (StatusCode, String) {}

async fn early() -> (StatusCode, Json<u8>, Json<u8>) {}

async fn string() -> String {}

async fn local() -> Plain {}

async fn unknown() -> Unknown {}

async fn opaque() -> impl IntoResponse {}

async fn status_ref() -> (&StatusCode, u8) {}

async fn no_field() -> NoField {}

async fn missing() -> Missing {}";

                // WHEN
                let routes = routes(server);

                // THEN
                assert_eq!(
                    routes,
                    [
                        "get /receiver receiver: a handler with a receiver",
                        "get /tuple tuple: unsupported the extractor `(u8, u8)`",
                        "get /paren paren: unsupported the extractor `u8`",
                        "get /array array: unsupported the extractor `[u8; _]`",
                        "get /bare bare: unsupported the extractor `_`",
                        "get /form form: unsupported the extractor `axum::Form`",
                        "get /plain plain: unsupported the extractor `Plain`",
                        "get /no_json no_json: an extractor without one `Json::<T>::from_request`",
                        "get /two_json two_json: an extractor without one `Json::<T>::from_request`",
                        "get /first first: a response tuple `(Json, StatusCode)` not starting with a `StatusCode`",
                        "get /elem elem: unsupported the response `String`",
                        "get /early early: a response tuple `(StatusCode, Json, Json)` with a body before its end",
                        "get /string string: unsupported the response `String`",
                        "get /local local: unsupported the response `Plain`",
                        "get /unknown unknown: unsupported the response `Unknown`",
                        "get /opaque opaque: unsupported the response `impl ..`",
                        "get /status_ref status_ref: a response tuple `(&StatusCode, u8)` not starting with a `StatusCode`",
                        "get /no_field no_field: an `IntoResponse` of `NoField` without one `Json(self.field)`",
                        "get /missing missing: no field `other` in `Missing`"
                    ]
                );
            }
        }
    }

    mod segments {
        use super::*;

        #[test]
        fn segments_should_read_the_literals_parameters_and_values() {
            // GIVEN
            let pieces = format_pieces("/{{a}}/{}/x{}y/{owner}/{*rest}//end/");

            // WHEN
            let segments = segments(&pieces);

            // THEN
            assert_eq!(path_text(&segments), "/{a}/{..}/{..}/{..}/{..}//end/");
        }

        #[test]
        fn segments_of_the_root_should_be_empty() {
            // GIVEN
            let pieces = [Piece::<()>::Text("/".into())];

            // WHEN
            let segments = segments(&pieces);

            // THEN
            assert_eq!(segments, []);
        }
    }
}
