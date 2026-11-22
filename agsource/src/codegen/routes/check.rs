//! [`check`]: the requests of a client's calls against the routes of the
//! server.

use super::Route;
use super::Routes;
use super::Segment;
use super::Transport;
use super::path_text;
use crate::Error;
use crate::codegen::lowered::Callee;
use crate::codegen::lowered::Expr;
use crate::codegen::lowered::Func;
use crate::codegen::lowered::K;
use crate::codegen::lowered::Lit;
use crate::codegen::lowered::Module;
use crate::codegen::lowered::Pat;
use crate::codegen::lowered::Shape;
use crate::codegen::lowered::Ty;
use crate::codegen::lowered::Variant;
use crate::codegen::pass::subst;
use std::collections::BTreeSet;

/// The client methods matched to their routes and the routes no method
/// calls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Checked {
    pub matched: Vec<Matched>,
    pub unmatched: Vec<Route>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Matched {
    /// `Class::method`.
    pub method: String,
    pub verb: String,
    pub path: String,
    pub handler: String,
}

impl Checked {
    /// The matches, a line each, then the routes no method calls.
    pub fn dump(&self) -> String {
        let mut out = String::new();

        for m in &self.matched {
            out += &format!("{} -> {} {} ({})\n", m.method, m.verb, m.path, m.handler);
        }

        out += &format!(
            "\n// {} routes no client method calls\n",
            self.unmatched.len()
        );

        for r in &self.unmatched {
            out += &format!("{} {} ({})\n", r.verb, path_text(&r.path), r.handler);
        }

        out
    }
}

/// Checks the request of every call of a transport trait method in the
/// methods of the classes of `client`, by the [`Transport`] or the body the
/// trait provides for the method, against the route it is sent to: the
/// verb, the literal segments of the path, the values of its parameters,
/// the keys and values of its query string, the body and the response by
/// their serialized form. The types the client names are in `client` and
/// `data`, the server's in [`Routes::module`] and `data`.
///
/// The URL is the `format!` of the call, a field of `self` the one value
/// it is constructed with in the methods of the class: the values in a
/// constructor and the leading segments not known on both sides are the
/// base path. A value of the path or the query is sent by its `Display`:
/// the strings of the `to_string` of an enum must be the serialized names
/// of its variants.
pub fn check(
    client: &Module,
    data: &[&Module],
    transport: &Transport,
    routes: &Routes,
) -> (Checked, Vec<Error>) {
    let mut checker = Checker {
        client: std::iter::once(client)
            .chain(data.iter().copied())
            .collect(),
        server: std::iter::once(&routes.module)
            .chain(data.iter().copied())
            .collect(),
        transport,
        routes,
        used: BTreeSet::new(),
        checked: Checked::default(),
        errors: Vec::new(),
    };

    for (name, class) in &client.classes {
        for f in &class.methods {
            for call in calls(&f.body, &[]) {
                checker.call(name, class.methods.as_slice(), f, &call, 0);
            }
        }
    }

    let Checker {
        used,
        mut checked,
        errors,
        ..
    } = checker;
    checked.unmatched = routes
        .routes
        .iter()
        .enumerate()
        .filter(|(i, _)| !used.contains(i))
        .map(|(_, r)| r.clone())
        .collect();
    (checked, errors)
}

/// A call of a trait method.
struct Call {
    trait_: String,
    name: String,
    args: Vec<Expr>,
    ty: Ty,
}

/// The calls of trait methods in the body, with `params` replaced by their
/// values.
fn calls(body: &[Expr], params: &[(String, Expr)]) -> Vec<Call> {
    let mut out = Vec::new();

    for e in body {
        let mut e = e.clone();
        e.rewrite(&mut |e| {
            if let K::Local(n) = &e.k
                && let Some((_, value)) = params.iter().find(|(p, _)| p == n)
            {
                *e = value.clone();
            }
        });
        e.rewrite(&mut |e| {
            if let K::Call {
                callee: Callee::Trait { trait_, name, .. },
                args,
                ..
            } = &e.k
            {
                out.push(Call {
                    trait_: trait_.clone(),
                    name: name.clone(),
                    args: args.clone(),
                    ty: e.ty.clone(),
                });
            }
        });
    }

    out
}

/// A part of a URL.
#[derive(Clone, Debug)]
enum Piece {
    Text(String),
    Value(Expr),
    /// A value of a constructor: the base path.
    Var,
}

/// A segment of the path of a URL.
#[derive(Clone, Debug)]
enum UrlSegment {
    Lit(String),
    Value(Expr),
    Var,
    /// Text and values.
    Mixed,
}

const MAX_DEPTH: usize = 8;

struct Checker<'a> {
    client: Vec<&'a Module>,
    server: Vec<&'a Module>,
    transport: &'a Transport,
    routes: &'a Routes,
    /// The indices of the routes a method calls.
    used: BTreeSet<usize>,
    checked: Checked,
    errors: Vec<Error>,
}

impl Checker<'_> {
    fn error(&mut self, class: &str, f: &Func, message: String) {
        let (path, line) =
            f.at.as_ref()
                .map(|at| (at.path.clone(), at.line))
                .unwrap_or_default();
        self.errors.push(Error::Lowering {
            path,
            line,
            item: format!("{class}::{}", f.name),
            message,
        });
    }

    fn call(&mut self, class: &str, methods: &[Func], f: &Func, call: &Call, depth: usize) {
        let Call {
            trait_, name, args, ..
        } = call;

        let Some(request) = self.transport.get(&(trait_.clone(), name.clone())) else {
            return self.provided(class, methods, f, call, depth);
        };

        let Some(uri) = args.get(request.uri) else {
            return self.error(class, f, format!("no URL argument of `{trait_}::{name}`"));
        };
        let pieces = match self.pieces(class, methods, uri, false) {
            Ok(pieces) => pieces,
            Err(message) => return self.error(class, f, message),
        };
        let (path, query) = split(&pieces);
        let segments = url_segments(&path);
        let Some(index) = self.route(&segments) else {
            return self.error(
                class,
                f,
                format!("no route for the URL `{}`", url_text(&pieces)),
            );
        };
        let candidates: Vec<usize> = self
            .routes
            .routes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.path == self.routes.routes[index].path)
            .map(|(i, _)| i)
            .collect();
        let Some(&index) = candidates
            .iter()
            .find(|&&i| self.routes.routes[i].verb == request.verb)
        else {
            return self.error(
                class,
                f,
                format!(
                    "`{}` sends `{}` to `{}`, which takes {}",
                    url_text(&pieces),
                    request.verb,
                    path_text(&self.routes.routes[index].path),
                    candidates
                        .iter()
                        .map(|&i| format!("`{}`", self.routes.routes[i].verb))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        };
        let route = &self.routes.routes[index];
        self.used.insert(index);
        self.checked.matched.push(Matched {
            method: format!("{class}::{}", f.name),
            verb: route.verb.clone(),
            path: path_text(&route.path),
            handler: route.handler.clone(),
        });

        let io = match &route.io {
            Ok(io) => io,
            Err(message) => {
                return self.error(
                    class,
                    f,
                    format!(
                        "the handler `{}` of `{}`: {message}",
                        route.handler,
                        path_text(&route.path)
                    ),
                );
            }
        };
        let mut messages = Vec::new();
        let values: Vec<&Expr> = segments
            .iter()
            .filter_map(|s| match s {
                UrlSegment::Value(e) => Some(e),
                _ => None,
            })
            .collect();

        if values.len() != io.params.len() {
            messages.push(format!(
                "{} path values for the {} parameters of `{}`",
                values.len(),
                io.params.len(),
                route.handler
            ));
        } else {
            for (value, param) in values.iter().zip(&io.params) {
                if let Err(m) = self.value(value, param) {
                    messages.push(format!("a path value: {m}"));
                }
            }
        }

        messages.extend(self.query(&query, io.query.as_ref()));
        messages.extend(self.body(request.body.map(|b| &args[b]), io.body.as_ref()));

        if let Some(at) = &request.response {
            messages.extend(self.response(&call.ty, at, io.response.as_ref()));
        }

        for m in messages {
            self.error(
                class,
                f,
                format!("`{} {}`: {m}", route.verb, path_text(&route.path)),
            );
        }
    }

    /// The call of a trait method the transport does not implement, by the
    /// body the trait provides: its calls with the parameters as the
    /// arguments.
    fn provided(&mut self, class: &str, methods: &[Func], f: &Func, call: &Call, depth: usize) {
        let Call {
            trait_, name, args, ..
        } = call;
        let provided = self
            .client
            .iter()
            .find_map(|m| m.traits.get(trait_))
            .and_then(|fs| fs.iter().find(|t| t.name == *name && !t.body.is_empty()));

        let Some(provided) = provided.filter(|_| depth < MAX_DEPTH) else {
            return self.error(class, f, format!("no transport sends `{trait_}::{name}`"));
        };

        let params: Vec<(String, Expr)> = provided
            .params
            .iter()
            .map(|(n, _)| n.clone())
            .zip(args.iter().cloned())
            .collect();
        let inner = calls(&provided.body, &params);

        if inner.is_empty() {
            return self.error(class, f, format!("no transport sends `{trait_}::{name}`"));
        }

        for mut inner in inner {
            inner.ty = call.ty.clone();
            self.call(class, methods, f, &inner, depth + 1);
        }
    }

    /// The pieces of the URL; the values of a constructor (`base`) are the
    /// base path.
    fn pieces(
        &self,
        class: &str,
        methods: &[Func],
        e: &Expr,
        base: bool,
    ) -> Result<Vec<Piece>, String> {
        match &e.k {
            K::Ref(inner) | K::Deref(inner) => self.pieces(class, methods, inner, base),
            K::Lit(Lit::Str(s)) => Ok(vec![Piece::Text(s.clone())]),
            K::Format(parts) => {
                let mut out = Vec::new();

                for p in parts {
                    out.extend(match is_text(p) {
                        true => self.pieces(class, methods, p, base)?,
                        false if base => vec![Piece::Var],
                        false => vec![Piece::Value(p.clone())],
                    });
                }

                Ok(out)
            }
            K::Field { base: b, name } if matches!(b.k, K::This) => {
                self.field(class, methods, name)
            }
            _ if base => Ok(vec![Piece::Var]),
            _ => Err("a URL that is no string or `format!`".into()),
        }
    }

    /// The pieces of the field of `self`: the value it is constructed with.
    fn field(&self, class: &str, methods: &[Func], name: &str) -> Result<Vec<Piece>, String> {
        let mut values = Vec::new();
        let mut assigned = false;

        for m in methods {
            for e in &m.body {
                e.clone().rewrite(&mut |e| match &e.k {
                    K::Construct { class: c, fields } if c == class => {
                        values.extend(
                            fields
                                .iter()
                                .filter(|(n, _)| n == name)
                                .map(|(_, v)| v.clone()),
                        );
                    }
                    K::Assign { place, .. }
                        if matches!(&place.k, K::Field { base, name: n } if n == name && matches!(base.k, K::This)) =>
                    {
                        assigned = true;
                    }
                    _ => {}
                });
            }
        }

        match (values.as_slice(), assigned) {
            ([value], false) => self.pieces(class, methods, value, true),
            (_, true) => Err(format!("the field `{name}` of the URL is assigned")),
            _ => Err(format!(
                "the field `{name}` of the URL is not constructed once"
            )),
        }
    }

    /// The route the path is sent to: of those it matches, the one with
    /// literal segments before parameters.
    fn route(&self, segments: &[UrlSegment]) -> Option<usize> {
        self.routes
            .routes
            .iter()
            .enumerate()
            .filter(|(_, r)| matches(&r.path, segments))
            .min_by_key(|(_, r)| {
                r.path
                    .iter()
                    .map(|s| match s {
                        Segment::Lit(_) => 0,
                        Segment::Param(_) | Segment::Var => 1,
                        Segment::Rest(_) => 2,
                    })
                    .collect::<Vec<u8>>()
            })
            .map(|(i, _)| i)
    }

    /// Whether the value sent by its `Display` is read as the type of the
    /// server.
    fn value(&self, value: &Expr, server: &Ty) -> Result<(), String> {
        let value = match &value.k {
            K::Call {
                callee: Callee::Member { name, .. },
                recv: Some(recv),
                args,
            } if name == "to_string" && args.is_empty() => recv,
            _ => value,
        };
        let client = value.ty.strip_refs();
        let server = strip_option(&server.strip_refs());

        if let Ty::Named(name, _) = &client
            && let Some(strings) = self.display(name)
        {
            return self.display_names(name, &strings?, &server);
        }

        match self.wire_eq(&server, &client) {
            true if server.is_primitive() => Ok(()),
            true => Err(format!(
                "`{}` is sent by its `Display`, which is not known",
                ty_text(&client)
            )),
            false => Err(format!(
                "`{}` sent, the server reads `{}`",
                ty_text(&client),
                ty_text(&server)
            )),
        }
    }

    /// The strings of the `to_string` of the client's enum by variant, an
    /// error when it is no match of strings.
    fn display(&self, name: &str) -> Option<Result<Vec<(String, String)>, String>> {
        let f = self
            .client
            .iter()
            .find_map(|m| m.enums.get(name))
            .and_then(|e| e.methods.iter().find(|f| f.name == "to_string"))?;
        let unsupported = || Err(format!("the `Display` of `{name}` is no match of strings"));
        let mut body = f.body.as_slice();

        while let [e] = body {
            match &e.k {
                K::Return(Some(v)) => body = std::slice::from_ref(&**v),
                K::Match { value, arms } if matches!(value.k, K::This) => {
                    let mut out = Vec::new();

                    for (pat, arm) in arms {
                        let (Pat::Variant { var, .. }, [s]) = (pat, arm.as_slice()) else {
                            return Some(unsupported());
                        };
                        let s = match &s.k {
                            K::Return(Some(v)) => &**v,
                            _ => s,
                        };
                        let K::Lit(Lit::Str(s)) = &s.k else {
                            return Some(unsupported());
                        };
                        out.push((var.clone(), s.clone()));
                    }

                    return Some(Ok(out));
                }
                _ => break,
            }
        }

        Some(unsupported())
    }

    fn display_names(
        &self,
        name: &str,
        strings: &[(String, String)],
        server: &Ty,
    ) -> Result<(), String> {
        let Ty::Named(server_name, _) = server else {
            return Err(format!(
                "`{name}` sent, the server reads `{}`",
                ty_text(server)
            ));
        };
        let Some((m, e)) = self
            .server
            .iter()
            .find_map(|m| m.enums.get(server_name).map(|e| (*m, e)))
        else {
            return Err(format!(
                "`{name}` sent, the server reads `{server_name}`, no enum"
            ));
        };
        let names: Vec<(&str, Option<String>)> = e
            .variants
            .iter()
            .map(|v| {
                (
                    v.name(),
                    wire_name(m, server_name, v.name()).map(|(n, _)| n),
                )
            })
            .collect();
        let mut errors = Vec::new();

        for (var, s) in strings {
            if names.iter().any(|(_, w)| w.as_ref() == Some(s)) {
                continue;
            }

            errors.push(match names.iter().find(|(n, _)| n == var) {
                Some((_, Some(w))) => {
                    format!("`{name}::{var}` is sent as \"{s}\", the server reads \"{w}\"")
                }
                _ => format!(
                    "`{name}::{var}` is sent as \"{s}\", which `{server_name}` does not read"
                ),
            });
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors.join("; ")),
        }
    }

    /// Whether the literal is read as the type of the server.
    fn literal(&self, value: &str, server: &Ty) -> bool {
        match strip_option(&server.strip_refs()) {
            Ty::Bool => value == "true" || value == "false",
            Ty::Int(_) => value.parse::<i128>().is_ok(),
            Ty::Float => value.parse::<f64>().is_ok(),
            Ty::Str => true,
            Ty::Named(name, _) => self.server.iter().any(|m| {
                m.enums.get(&name).is_some_and(|e| {
                    e.variants
                        .iter()
                        .any(|v| wire_name(m, &name, v.name()).is_some_and(|(n, _)| n == value))
                })
            }),
            _ => false,
        }
    }

    fn query(&self, query: &[Piece], server: Option<&Ty>) -> Vec<String> {
        let pairs = match query_pairs(query) {
            Ok(pairs) => pairs,
            Err(m) => return vec![m],
        };

        let Some(server) = server else {
            return match pairs.is_empty() {
                true => Vec::new(),
                false => vec![format!(
                    "the query `{}` sent, the handler takes no `Query`",
                    pairs
                        .iter()
                        .map(|(k, _)| k.as_str())
                        .collect::<Vec<_>>()
                        .join("&")
                )],
            };
        };
        let fields = match self.fields(server) {
            Ok(fields) => fields,
            Err(m) => return vec![m],
        };
        let mut out = Vec::new();

        for (key, value) in &pairs {
            let Some((_, ty, _)) = fields.iter().find(|(n, _, _)| n == key) else {
                out.push(format!(
                    "the query key `{key}` is no field of `{}`",
                    ty_text(server)
                ));
                continue;
            };
            let result = match value {
                Piece::Text(text) if self.literal(text, ty) => Ok(()),
                Piece::Text(text) => Err(format!("\"{text}\" is not read as `{}`", ty_text(ty))),
                Piece::Value(e) => self.value(e, ty),
                Piece::Var => Err("a value of the base path".into()),
            };

            if let Err(m) = result {
                out.push(format!("the query key `{key}`: {m}"));
            }
        }

        for (name, _, optional) in &fields {
            if !optional && !pairs.iter().any(|(k, _)| k == name) {
                out.push(format!(
                    "the query key `{name}` of `{}` is not sent",
                    ty_text(server)
                ));
            }
        }

        out
    }

    /// The serialized fields of the server's struct: the name, the type and
    /// whether a value may leave it out.
    fn fields(&self, ty: &Ty) -> Result<Vec<(String, Ty, bool)>, String> {
        let Ty::Named(name, args) = ty.strip_ref() else {
            return Err(format!("a `Query` of `{}`, no struct", ty_text(ty)));
        };
        let Some((m, class)) = self
            .server
            .iter()
            .find_map(|m| m.classes.get(name).map(|c| (*m, c)))
        else {
            return Err(format!("a `Query` of `{name}`, no struct"));
        };
        let params = generics(&class.generics, args);

        Ok(class
            .fields
            .iter()
            .filter_map(|(field, ty)| {
                let (wire, optional) = wire_name(m, name, field)?;
                let ty = subst(ty, &Ty::SelfTy, &params);
                let optional = optional || matches!(ty, Ty::Option(_));
                Some((wire, ty, optional))
            })
            .collect())
    }

    fn body(&self, sent: Option<&Expr>, server: Option<&Ty>) -> Vec<String> {
        let sent = sent.and_then(|e| match &e.k {
            K::Variant {
                var, payload: None, ..
            } if var == "None" => None,
            K::Variant {
                var,
                payload: Some(p),
                ..
            } if var == "Some" => Some(p.ty.strip_refs()),
            _ => Some(strip_option(&e.ty.strip_refs())),
        });

        match (sent, server) {
            (None, None) => Vec::new(),
            (Some(sent), Some(server)) if self.wire_eq(server, &sent) => Vec::new(),
            (Some(sent), Some(server)) => vec![format!(
                "the body `{}` sent, the handler reads `{}`",
                ty_text(&sent),
                ty_text(server)
            )],
            (Some(sent), None) => vec![format!(
                "the body `{}` sent, the handler reads none",
                ty_text(&sent)
            )],
            (None, Some(server)) => vec![format!(
                "no body sent, the handler reads `{}`",
                ty_text(server)
            )],
        }
    }

    fn response(&self, ty: &Ty, at: &[usize], server: Option<&Ty>) -> Vec<String> {
        let mut ty = ty.strip_ref().clone();

        for step in std::iter::once(None).chain(at.iter().map(Some)) {
            ty = match (ty, step) {
                (Ty::Result(ok, _), None) => ok.strip_ref().clone(),
                (Ty::Tuple(items), Some(&i)) if i < items.len() => items[i].strip_ref().clone(),
                _ => Ty::Unknown,
            };
        }

        match (&ty, server) {
            (Ty::Unit, None) => Vec::new(),
            (ty, Some(server)) if self.wire_eq(server, ty) => Vec::new(),
            (Ty::Unknown | Ty::Param(_), _) => {
                vec!["the type of the response the client reads is not known".into()]
            }
            (ty, None) => vec![format!(
                "the response `{}` read, the handler answers none",
                ty_text(ty)
            )],
            (ty, Some(server)) => vec![format!(
                "the response `{}` read, the handler answers `{}`",
                ty_text(ty),
                ty_text(server)
            )],
        }
    }

    /// Whether the server's and the client's types are serialized the same.
    fn wire_eq(&self, server: &Ty, client: &Ty) -> bool {
        self.eq(server, client, &mut Vec::new())
    }

    fn eq(&self, server: &Ty, client: &Ty, seen: &mut Vec<(Ty, Ty)>) -> bool {
        let server = unwrap(&self.server, &server.strip_refs());
        let client = unwrap(&self.client, &client.strip_refs());

        match (&server, &client) {
            (Ty::List(s), Ty::List(c)) | (Ty::Option(s), Ty::Option(c)) => self.eq(s, c, seen),
            (Ty::Tuple(s), Ty::Tuple(c)) => {
                s.len() == c.len() && s.iter().zip(c).all(|(s, c)| self.eq(s, c, seen))
            }
            (Ty::Named(s, s_args), Ty::Named(c, c_args)) => {
                if seen.contains(&(server.clone(), client.clone())) {
                    return true;
                }

                seen.push((server.clone(), client.clone()));
                self.named_eq((s, s_args), (c, c_args), seen)
            }
            (Ty::Unknown | Ty::Param(_), _) | (_, Ty::Unknown | Ty::Param(_)) => false,
            (s, c) => s == c,
        }
    }

    fn named_eq(
        &self,
        (s, s_args): (&str, &[Ty]),
        (c, c_args): (&str, &[Ty]),
        seen: &mut Vec<(Ty, Ty)>,
    ) -> bool {
        if let (Some((sm, sc)), Some((cm, cc))) = (
            self.server
                .iter()
                .find_map(|m| m.classes.get(s).map(|x| (*m, x))),
            self.client
                .iter()
                .find_map(|m| m.classes.get(c).map(|x| (*m, x))),
        ) {
            let s_params = generics(&sc.generics, s_args);
            let c_params = generics(&cc.generics, c_args);
            let s_fields = members(sm, s, &sc.fields);
            let c_fields = members(cm, c, &cc.fields);

            return s_fields.len() == c_fields.len()
                && s_fields.iter().all(|(name, ty, optional)| {
                    c_fields.iter().any(|(n, t, o)| {
                        n == name
                            && o == optional
                            && self.eq(
                                &subst(ty, &Ty::SelfTy, &s_params),
                                &subst(t, &Ty::SelfTy, &c_params),
                                seen,
                            )
                    })
                });
        }

        if let (Some((sm, se)), Some((cm, ce))) = (
            self.server
                .iter()
                .find_map(|m| m.enums.get(s).map(|x| (*m, x))),
            self.client
                .iter()
                .find_map(|m| m.enums.get(c).map(|x| (*m, x))),
        ) {
            let tagging = |m: &Module, n: &str| m.wire.get(n).map(|w| w.tagging.clone());
            let s_vars = variants(sm, s, &se.variants);
            let c_vars = variants(cm, c, &ce.variants);

            return tagging(sm, s) == tagging(cm, c)
                && s_vars.len() == c_vars.len()
                && s_vars.iter().all(|(name, payload)| {
                    c_vars.iter().any(|(n, p)| {
                        n == name
                            && match (payload, p) {
                                (None, None) => true,
                                (Some(s), Some(c)) => self.eq(s, c, seen),
                                _ => false,
                            }
                    })
                });
        }

        false
    }
}

/// The type a newtype or transparent struct is serialized as.
fn unwrap(modules: &[&Module], ty: &Ty) -> Ty {
    let mut ty = ty.clone();

    for _ in 0..MAX_DEPTH {
        match newtype(modules, &ty) {
            Some(field) => ty = field,
            None => break,
        }
    }

    ty
}

/// The type of the one field of a newtype or transparent struct.
fn newtype(modules: &[&Module], ty: &Ty) -> Option<Ty> {
    let Ty::Named(name, args) = ty else {
        return None;
    };
    let (m, class) = modules
        .iter()
        .find_map(|m| m.classes.get(name).map(|c| (*m, c)))?;
    let wire = m.wire.get(name);
    let newtype = wire.is_some_and(|w| w.transparent)
        || (wire.is_some_and(|w| w.shape == Shape::Tuple) && class.fields.len() == 1);

    match (&class.fields[..], newtype) {
        ([(_, field)], true) => {
            Some(subst(field, &Ty::SelfTy, &generics(&class.generics, args)).strip_refs())
        }
        _ => None,
    }
}

fn generics(generics: &[crate::codegen::lowered::Generic], args: &[Ty]) -> Vec<(String, Ty)> {
    generics
        .iter()
        .map(|g| g.name.clone())
        .zip(args.iter().cloned())
        .collect()
}

/// The serialized fields of a struct: name, type, optional.
fn members(m: &Module, name: &str, fields: &[(String, Ty)]) -> Vec<(String, Ty, bool)> {
    fields
        .iter()
        .filter_map(|(field, ty)| {
            wire_name(m, name, field).map(|(wire, optional)| (wire, ty.clone(), optional))
        })
        .collect()
}

/// The serialized variants of an enum: name and payload.
fn variants(m: &Module, name: &str, vars: &[Variant]) -> Vec<(String, Option<Ty>)> {
    vars.iter()
        .filter_map(|v| {
            let (name, _) = wire_name(m, name, v.name())?;
            let payload = match v {
                Variant::Unit(_) => None,
                Variant::Tuple(_, ty) => Some(ty.clone()),
                Variant::Struct(_, fields) => {
                    Some(Ty::Tuple(fields.iter().map(|(_, t)| t.clone()).collect()))
                }
            };
            Some((name, payload))
        })
        .collect()
}

fn strip_option(ty: &Ty) -> Ty {
    match ty {
        Ty::Option(inner) => inner.strip_refs(),
        ty => ty.clone(),
    }
}

/// Whether the expression is URL text: a string, a `format!` or a field of
/// `self`, behind references.
fn is_text(e: &Expr) -> bool {
    match &e.k {
        K::Ref(inner) | K::Deref(inner) => is_text(inner),
        K::Lit(Lit::Str(_)) | K::Format(_) => true,
        K::Field { base, .. } => matches!(base.k, K::This),
        _ => false,
    }
}

/// The serialized name of the field or the variant of the type and whether
/// a value may leave it out, none when it is skipped.
fn wire_name(m: &Module, ty: &str, member: &str) -> Option<(String, bool)> {
    match m.wire.get(ty).and_then(|w| w.members.get(member)) {
        Some(member) if member.skip => None,
        Some(member) => Some((member.name.clone(), member.optional)),
        None => Some((member.to_owned(), false)),
    }
}

/// The pieces before and after the `?`.
fn split(pieces: &[Piece]) -> (Vec<Piece>, Vec<Piece>) {
    let mut path = Vec::new();
    let mut query = Vec::new();
    let mut in_query = false;

    for p in pieces {
        match p {
            Piece::Text(t) if !in_query && t.contains('?') => {
                let (before, after) = t.split_once('?').unwrap_or_default();
                path.push(Piece::Text(before.to_owned()));
                query.push(Piece::Text(after.to_owned()));
                in_query = true;
            }
            p if in_query => query.push(p.clone()),
            p => path.push(p.clone()),
        }
    }

    (path, query)
}

/// The segments of the path, without the leading ones not known.
fn url_segments(pieces: &[Piece]) -> Vec<UrlSegment> {
    let mut out = Vec::new();
    let mut current: Vec<Piece> = Vec::new();

    let flush = |current: &mut Vec<Piece>, out: &mut Vec<UrlSegment>| {
        let segment = match current.as_slice() {
            [] if out.is_empty() => None,
            [Piece::Value(e)] => Some(UrlSegment::Value(e.clone())),
            pieces if pieces.iter().any(|p| matches!(p, Piece::Var)) => Some(UrlSegment::Var),
            pieces => Some(
                pieces
                    .iter()
                    .map(|p| match p {
                        Piece::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect::<Option<String>>()
                    .map_or(UrlSegment::Mixed, UrlSegment::Lit),
            ),
        };
        out.extend(segment);
        current.clear();
    };

    for p in pieces {
        match p {
            Piece::Text(t) => {
                let mut parts = t.split('/');

                if let Some(first) = parts.next()
                    && !first.is_empty()
                {
                    current.push(Piece::Text(first.to_owned()));
                }

                for part in parts {
                    flush(&mut current, &mut out);

                    if !part.is_empty() {
                        current.push(Piece::Text(part.to_owned()));
                    }
                }
            }
            p => current.push(p.clone()),
        }
    }

    flush(&mut current, &mut out);

    out.into_iter()
        .skip_while(|s| matches!(s, UrlSegment::Var))
        .collect()
}

/// The keys and values of the query string.
fn query_pairs(pieces: &[Piece]) -> Result<Vec<(String, Piece)>, String> {
    let mut out = Vec::new();
    let mut key: Option<String> = None;
    let mut value: Vec<Piece> = Vec::new();
    let mut pending = String::new();

    let mut finish = |key: &mut Option<String>,
                      value: &mut Vec<Piece>,
                      pending: &mut String|
     -> Result<(), String> {
        if let Some(k) = key.take() {
            let v = match std::mem::take(value).as_slice() {
                [] => Piece::Text(String::new()),
                [Piece::Text(t)] => Piece::Text(t.clone()),
                [p @ (Piece::Value(_) | Piece::Var)] => p.clone(),
                _ => return Err(format!("the query value of `{k}` is not one value")),
            };
            out.push((k, v));
        } else if !pending.is_empty() {
            return Err(format!("the query part `{pending}` has no value"));
        }

        pending.clear();
        Ok(())
    };

    for p in pieces {
        match p {
            Piece::Text(t) => {
                for (i, part) in t.split('&').enumerate() {
                    if i > 0 {
                        finish(&mut key, &mut value, &mut pending)?;
                    }

                    if key.is_some() {
                        if !part.is_empty() {
                            value.push(Piece::Text(part.to_owned()));
                        }
                    } else {
                        pending.push_str(part);

                        if let Some((k, v)) = pending.clone().split_once('=') {
                            key = Some(k.to_owned());
                            pending.clear();

                            if !v.is_empty() {
                                value.push(Piece::Text(v.to_owned()));
                            }
                        }
                    }
                }
            }
            p if key.is_some() => value.push(p.clone()),
            _ => return Err("a query key that is no literal".into()),
        }
    }

    finish(&mut key, &mut value, &mut pending)?;
    Ok(out)
}

/// Whether the route's path matches the segments of the URL.
fn matches(route: &[Segment], url: &[UrlSegment]) -> bool {
    match (route.first(), url.first()) {
        (None, None) => true,
        (Some(Segment::Rest(_)), Some(_)) => true,
        (Some(Segment::Lit(r)), Some(UrlSegment::Lit(u))) if r == u => {
            matches(&route[1..], &url[1..])
        }
        (Some(Segment::Param(_)), Some(UrlSegment::Value(_))) => matches(&route[1..], &url[1..]),
        _ => false,
    }
}

fn url_text(pieces: &[Piece]) -> String {
    pieces
        .iter()
        .map(|p| match p {
            Piece::Text(t) => t.clone(),
            Piece::Value(_) => "{}".to_owned(),
            Piece::Var => "{..}".to_owned(),
        })
        .collect()
}

fn ty_text(ty: &Ty) -> String {
    match ty {
        Ty::Unit => "()".into(),
        Ty::Bool => "bool".into(),
        Ty::Int(k) => (*k).to_owned(),
        Ty::Float => "f64".into(),
        Ty::Str => "String".into(),
        Ty::List(t) => format!("Vec<{}>", ty_text(t)),
        Ty::Option(t) => format!("Option<{}>", ty_text(t)),
        Ty::Result(o, e) => format!("Result<{}, {}>", ty_text(o), ty_text(e)),
        Ty::Tuple(items) => format!(
            "({})",
            items.iter().map(ty_text).collect::<Vec<_>>().join(", ")
        ),
        Ty::Named(n, args) if args.is_empty() => n.clone(),
        Ty::Named(n, args) => format!(
            "{n}<{}>",
            args.iter().map(ty_text).collect::<Vec<_>>().join(", ")
        ),
        Ty::Param(p) => p.clone(),
        Ty::Ref(t) => format!("&{}", ty_text(t)),
        _ => "_".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::test_utilities::ROUTES_API;
    use crate::codegen::test_utilities::ROUTES_SERVER;
    use crate::codegen::test_utilities::checked;
    use crate::codegen::test_utilities::edited;
    use crate::codegen::test_utilities::lower;

    /// The errors of the check of the edited [`ROUTES_API`] against the
    /// edited [`ROUTES_SERVER`].
    fn errors(api: &[(&str, &str)], server: &[(&str, &str)]) -> Vec<String> {
        checked(&edited(ROUTES_API, api), &edited(ROUTES_SERVER, server)).1
    }

    mod check {
        use super::*;

        #[test]
        fn check_should_match_the_calls_to_their_routes() {
            // GIVEN
            let (api, server) = (ROUTES_API, ROUTES_SERVER);

            // WHEN
            let (checked, errors) = checked(api, server);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
            assert_eq!(
                checked.dump(),
                "Api::login -> post /api/login (login)
Api::items -> get /api/items/{owner}/list (items)
Api::add -> post /api/items/{owner}/add (add)
Api::remove -> delete /api/items/{owner}/{id} (remove)
Api::exec -> post /api/items/{owner}/exec (exec)

// 1 routes no client method calls
get /api/internal (internal)
"
            );
        }

        #[test]
        fn check_url_without_route_should_fail() {
            // GIVEN
            let api = [("\"/items/{owner}/list\"", "\"/items/{owner}/lists\"")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:138: `Api::items`: no route for the URL `{..}/api/items/{}/lists`"
                ]
            );
        }

        #[test]
        fn check_other_verb_should_fail() {
            // GIVEN
            let server = [("routing::delete(remove)", "routing::put(remove)")];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:155: `Api::remove`: `{..}/api/items/{}/{}` sends `delete` to `/api/items/{owner}/{id}`, which takes `put`"
                ]
            );
        }

        #[test]
        fn check_trait_method_without_transport_should_fail() {
            // GIVEN
            let api = [
                (
                    "    async fn delete(&self, uri: &str) -> ApiResult<u16>;",
                    "    async fn delete(&self, uri: &str) -> ApiResult<u16>;\n    async fn head(&self, uri: &str) -> ApiResult<u16>;",
                ),
                (
                    "            .delete(&self.url",
                    "            .head(&self.url",
                ),
            ];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                ["/api/src/lib.rs:156: `Api::remove`: no transport sends `Http::head`"]
            );
        }

        #[test]
        fn check_provided_method_calling_itself_should_fail() {
            // GIVEN
            let api = [(
                "self.post(uri, Some(items)).await",
                "self.exec(uri, items).await",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                ["/api/src/lib.rs:161: `Api::exec`: no transport sends `Http::exec`"]
            );
        }

        #[test]
        fn check_provided_method_without_calls_should_fail() {
            // GIVEN
            let api = [("self.post(uri, Some(items)).await", "panic!(\"none\")")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                ["/api/src/lib.rs:161: `Api::exec`: no transport sends `Http::exec`"]
            );
        }

        #[test]
        fn check_call_without_url_should_fail() {
            // GIVEN
            let api = [(
                ".delete(&self.url(&format!(\"/items/{owner}/{id}\")))",
                ".delete()",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                ["/api/src/lib.rs:155: `Api::remove`: no URL argument of `Http::delete`"]
            );
        }

        #[test]
        fn check_url_of_a_parameter_should_fail() {
            // GIVEN
            let api = [(
                ".delete(&self.url(&format!(\"/items/{owner}/{id}\")))",
                ".delete(owner)",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                ["/api/src/lib.rs:155: `Api::remove`: a URL that is no string or `format!`"]
            );
        }

        #[test]
        fn check_assigned_base_url_should_fail() {
            // GIVEN
            let api = [(
                "    fn url(&self",
                "    pub fn rebase(&mut self, base: String) {\n        self.base_url = base;\n    }\n\n    fn url(&self",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:124: `Api::login`: the field `base_url` of the URL is assigned",
                    "/api/src/lib.rs:138: `Api::items`: the field `base_url` of the URL is assigned",
                    "/api/src/lib.rs:144: `Api::add`: the field `base_url` of the URL is assigned",
                    "/api/src/lib.rs:155: `Api::remove`: the field `base_url` of the URL is assigned",
                    "/api/src/lib.rs:161: `Api::exec`: the field `base_url` of the URL is assigned"
                ]
            );
        }

        #[test]
        fn check_base_url_constructed_twice_should_fail() {
            // GIVEN
            let api = [(
                "    fn url(&self",
                "    pub fn local(client: T) -> Self {\n        Self {\n            client,\n            base_url: \"/api\".to_string(),\n        }\n    }\n\n    fn url(&self",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:124: `Api::login`: the field `base_url` of the URL is not constructed once",
                    "/api/src/lib.rs:138: `Api::items`: the field `base_url` of the URL is not constructed once",
                    "/api/src/lib.rs:144: `Api::add`: the field `base_url` of the URL is not constructed once",
                    "/api/src/lib.rs:155: `Api::remove`: the field `base_url` of the URL is not constructed once",
                    "/api/src/lib.rs:161: `Api::exec`: the field `base_url` of the URL is not constructed once"
                ]
            );
        }

        #[test]
        fn check_should_take_a_base_url_value_as_the_base_path() {
            // GIVEN
            let api = [(
                "base_url: format!(\"{address}/api\"),",
                "base_url: address.to_string(),",
            )];
            let server = [("Router::new().nest(\"/api\", api)", "api")];

            // WHEN
            let errors = errors(&api, &server);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
        }

        #[test]
        fn check_unsupported_handler_should_fail() {
            // GIVEN
            let server = [(
                "async fn remove(Path((owner, id)): Path<(String, u64)>)",
                "async fn remove(Path((owner, id)): Path<(String, u64)>, count: Option<u8>)",
            )];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:155: `Api::remove`: the handler `remove` of `/api/items/{owner}/{id}`: unsupported the extractor `Option`"
                ]
            );
        }

        #[test]
        fn check_path_values_of_other_parameters_should_fail() {
            // GIVEN
            let server = [(
                "Path((owner, id)): Path<(String, u64)>",
                "Path(owner): Path<String>",
            )];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:155: `Api::remove`: `delete /api/items/{owner}/{id}`: 2 path values for the 1 parameters of `remove`"
                ]
            );
        }

        #[test]
        fn check_path_value_of_another_type_should_fail() {
            // GIVEN
            let server = [(
                "Path((owner, id)): Path<(String, u64)>",
                "Path((owner, id)): Path<(String, String)>",
            )];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:155: `Api::remove`: `delete /api/items/{owner}/{id}`: a path value: `u64` sent, the server reads `String`"
                ]
            );
        }

        #[test]
        fn check_value_without_display_should_fail() {
            // GIVEN
            let api = [
                (
                    "pub async fn remove(&self, owner: &str, id: u64)",
                    "pub async fn remove(&self, owner: &str, id: Items)",
                ),
                (
                    "pub trait Http {",
                    "impl fmt::Display for Items {\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n        f.write_str(\"items\")\n    }\n}\n\npub trait Http {",
                ),
            ];
            let server = [(
                "Path((owner, id)): Path<(String, u64)>",
                "Path((owner, id)): Path<(String, api::Items)>",
            )];

            // WHEN
            let errors = errors(&api, &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:161: `Api::remove`: `delete /api/items/{owner}/{id}`: a path value: `Items` is sent by its `Display`, which is not known"
                ]
            );
        }

        #[test]
        fn check_display_of_a_format_should_fail() {
            // GIVEN
            let api = [(
                "        match self {\n            Kind::InMemory => f.write_str(\"in_memory\"),\n            Kind::File => f.write_str(\"file\"),\n        }",
                "        write!(f, \"{}\", 1)",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:141: `Api::add`: `post /api/items/{owner}/add`: the query key `kind`: the `Display` of `Kind` is no match of strings"
                ]
            );
        }

        #[test]
        fn check_display_with_a_wildcard_should_fail() {
            // GIVEN
            let api = [(
                "Kind::File => f.write_str(\"file\"),",
                "_ => f.write_str(\"file\"),",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `kind`: the `Display` of `Kind` is no match of strings"
                ]
            );
        }

        #[test]
        fn check_display_of_no_literal_should_fail() {
            // GIVEN
            let api = [(
                "Kind::File => f.write_str(\"file\"),",
                "Kind::File => f.write_str(if true { \"file\" } else { \"f\" }),",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `kind`: the `Display` of `Kind` is no match of strings"
                ]
            );
        }

        #[test]
        fn check_display_of_another_name_should_fail() {
            // GIVEN
            let api = [("f.write_str(\"in_memory\")", "f.write_str(\"memory\")")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `kind`: `Kind::InMemory` is sent as \"memory\", the server reads \"in_memory\""
                ]
            );
        }

        #[test]
        fn check_display_read_as_a_primitive_should_fail() {
            // GIVEN
            let server = [("    kind: api::Kind,", "    kind: String,")];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `kind`: `Kind` sent, the server reads `String`"
                ]
            );
        }

        #[test]
        fn check_display_read_as_a_struct_should_fail() {
            // GIVEN
            let server = [("    kind: api::Kind,", "    kind: api::Login,")];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `kind`: `Kind` sent, the server reads `Login`, no enum"
                ]
            );
        }

        #[test]
        fn check_should_allow_variants_the_client_does_not_send() {
            // GIVEN
            let server = [
                ("    kind: api::Kind,", "    kind: ServerKind,"),
                (
                    "#[derive(Deserialize)]\nstruct AddParams",
                    "#[derive(Deserialize)]\n#[serde(rename_all = \"snake_case\")]\nenum ServerKind {\n    InMemory,\n    File,\n    Remote,\n}\n\n#[derive(Deserialize)]\nstruct AddParams",
                ),
            ];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
        }

        #[test]
        fn check_should_read_literals_by_the_type() {
            // GIVEN
            let api = [("&force=true", "&force=true&n=1&f=1.5&s=x&e=file")];
            let server = [(
                "    force: bool,",
                "    force: bool,\n    n: Option<u8>,\n    f: f64,\n    s: String,\n    e: api::Kind,",
            )];

            // WHEN
            let errors = errors(&api, &server);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
        }

        #[test]
        fn check_literals_of_other_types_should_fail() {
            // GIVEN
            let api = [("&force=true", "&force=&n=x&f=y&e=other&l=1")];
            let server = [(
                "    force: bool,",
                "    force: bool,\n    n: u8,\n    f: f64,\n    e: api::Kind,\n    l: Vec<u8>,",
            )];

            // WHEN
            let errors = errors(&api, &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `force`: \"\" is not read as `bool`",
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `n`: \"x\" is not read as `u8`",
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `f`: \"y\" is not read as `f64`",
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `e`: \"other\" is not read as `Kind`",
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `l`: \"1\" is not read as `Vec<u8>`"
                ]
            );
        }

        #[test]
        fn check_query_value_of_two_values_should_fail() {
            // GIVEN
            let api = [("?kind={kind}&", "?kind={kind}x&")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query value of `kind` is not one value"
                ]
            );
        }

        #[test]
        fn check_query_part_without_value_should_fail() {
            // GIVEN
            let api = [("?kind={kind}&", "?force&kind={kind}&")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query part `force` has no value"
                ]
            );
        }

        #[test]
        fn check_query_key_of_a_value_should_fail() {
            // GIVEN
            let api = [("?kind={kind}&force=true", "?{kind}=1")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: a query key that is no literal"
                ]
            );
        }

        #[test]
        fn check_query_value_of_the_base_path_should_fail() {
            // GIVEN
            let api = [
                (
                    "    base_url: String,\n}",
                    "    base_url: String,\n    token: String,\n}",
                ),
                (
                    "            base_url: format!(\"{address}/api\"),",
                    "            base_url: format!(\"{address}/api\"),\n            token: address.to_string(),",
                ),
                (
                    "&format!(\"/items/{owner}/add?kind={kind}&force=true\")",
                    "&format!(\"/items/{owner}/add?kind={kind}&force={}\", self.token)",
                ),
            ];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:146: `Api::add`: `post /api/items/{owner}/add`: the query key `force`: a value of the base path"
                ]
            );
        }

        #[test]
        fn check_query_to_a_handler_without_query_should_fail() {
            // GIVEN
            let api = [("\"/items/{owner}/{id}\"", "\"/items/{owner}/{id}?x=1\"")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:155: `Api::remove`: `delete /api/items/{owner}/{id}`: the query `x` sent, the handler takes no `Query`"
                ]
            );
        }

        #[test]
        fn check_query_of_no_struct_should_fail() {
            // GIVEN
            let server = [("Query<AddParams>", "Query<u8>")];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: a `Query` of `u8`, no struct"
                ]
            );
        }

        #[test]
        fn check_query_of_an_enum_should_fail() {
            // GIVEN
            let server = [("Query<AddParams>", "Query<api::Kind>")];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: a `Query` of `Kind`, no struct"
                ]
            );
        }

        #[test]
        fn check_query_of_other_keys_should_fail() {
            // GIVEN
            let api = [("?kind={kind}", "?kinds={kind}")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `kinds` is no field of `AddParams`",
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query key `kind` of `AddParams` is not sent"
                ]
            );
        }

        #[test]
        fn check_should_read_the_query_of_a_generic_struct() {
            // GIVEN
            let server = [
                (
                    "struct AddParams {\n    kind: api::Kind,",
                    "struct AddParams<T> {\n    kind: T,\n    #[serde(skip)]\n    cache: u8,",
                ),
                ("Query<AddParams>", "Query<AddParams<api::Kind>>"),
            ];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
        }

        #[test]
        fn check_no_body_to_a_handler_reading_one_should_fail() {
            // GIVEN
            let api = [(
                "                Some(Login {\n                    name: name.to_string(),\n                    password: password.to_string(),\n                }),",
                "                None,",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:124: `Api::login`: `post /api/login`: no body sent, the handler reads `Login`"
                ]
            );
        }

        #[test]
        fn check_body_of_another_type_should_fail() {
            // GIVEN
            let server = [(
                "Json(login): Json<api::Login>",
                "Json(login): Json<api::Item>",
            )];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:124: `Api::login`: `post /api/login`: the body `Login` sent, the handler reads `Item`"
                ]
            );
        }

        #[test]
        fn check_body_to_a_handler_reading_none_should_fail() {
            // GIVEN
            let api = [(
                "&self.url(&format!(\"/items/{owner}/add?kind={kind}&force=true\")),\n                None,",
                "&self.url(&format!(\"/items/{owner}/add?kind={kind}&force=true\")),\n                Some(1),",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the body `i32` sent, the handler reads none"
                ]
            );
        }

        #[test]
        fn check_response_of_another_type_should_fail() {
            // GIVEN
            let server = [(
                "Answer<(StatusCode, Json<String>)>",
                "Answer<(StatusCode, Json<u64>)>",
            )];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:124: `Api::login`: `post /api/login`: the response `String` read, the handler answers `u64`"
                ]
            );
        }

        #[test]
        fn check_unit_response_of_a_body_should_fail() {
            // GIVEN
            let server = [(
                "Answer<(StatusCode, Header)>",
                "Answer<(StatusCode, Header, Json<String>)>",
            )];

            // WHEN
            let errors = errors(&[], &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the response `()` read, the handler answers `String`"
                ]
            );
        }

        #[test]
        fn check_response_without_body_should_fail() {
            // GIVEN
            let api = [(".post::<(), ()>(", ".post::<(), String>(")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the response `String` read, the handler answers none"
                ]
            );
        }

        #[test]
        fn check_response_not_known_should_fail() {
            // GIVEN
            let api = [(
                "            .get(&self.url(&format!(\"/items/{owner}/list\")))\n            .await\n",
                "            .get(&self.url(&format!(\"/items/{owner}/list\")))\n            .await?;\n        Ok((200, Items(Vec::new())))\n",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:138: `Api::items`: `get /api/items/{owner}/list`: the type of the response the client reads is not known"
                ]
            );
        }

        #[test]
        fn check_display_of_a_variant_the_server_does_not_read_should_fail() {
            // GIVEN
            let api = [
                ("    File,\n}", "    File,\n    Remote,\n}"),
                (
                    "Kind::File => f.write_str(\"file\"),",
                    "Kind::File => f.write_str(\"file\"),\n            Kind::Remote => f.write_str(\"remote\"),",
                ),
            ];

            let server = [
                ("    kind: api::Kind,", "    kind: ServerKind,"),
                (
                    "#[derive(Deserialize)]\nstruct AddParams",
                    "#[derive(Deserialize)]\n#[serde(rename_all = \"snake_case\")]\nenum ServerKind {\n    InMemory,\n    File,\n}\n\n#[derive(Deserialize)]\nstruct AddParams",
                ),
            ];

            // WHEN
            let errors = errors(&api, &server);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:146: `Api::add`: `post /api/items/{owner}/add`: the query key `kind`: `Kind::Remote` is sent as \"remote\", which `ServerKind` does not read"
                ]
            );
        }

        #[test]
        fn check_query_part_without_value_at_the_end_should_fail() {
            // GIVEN
            let api = [("&force=true", "&force=true&x")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:144: `Api::add`: `post /api/items/{owner}/add`: the query part `x` has no value"
                ]
            );
        }

        #[test]
        fn check_url_with_a_mixed_segment_should_fail() {
            // GIVEN
            let api = [("\"/items/{owner}/list\"", "\"/items/{owner}x/list\"")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:138: `Api::items`: no route for the URL `{..}/api/items/{}x/list`"
                ]
            );
        }

        #[test]
        fn check_url_with_a_trailing_slash_should_fail() {
            // GIVEN
            let api = [("\"/items/{owner}/list\"", "\"/items/{owner}/list/\"")];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:138: `Api::items`: no route for the URL `{..}/api/items/{}/list/`"
                ]
            );
        }

        #[test]
        fn check_should_match_empty_segments() {
            // GIVEN
            let api = [("\"/items/{owner}/list\"", "\"/items//{owner}/list/\"")];
            let server = [("\"/items/{owner}/list\"", "\"/items//{owner}/list/\"")];

            // WHEN
            let errors = errors(&api, &server);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
        }

        #[test]
        fn check_should_prefer_literal_segments_to_the_rest() {
            // GIVEN
            let server = [(
                ".route(\"/internal\", routing::get(internal));",
                ".route(\"/internal\", routing::get(internal))\n        .route(\"/items/{*rest}\", routing::get(internal));",
            )];

            // WHEN
            let (checked, errors) = checked(ROUTES_API, &edited(ROUTES_SERVER, &server));

            // THEN
            assert_eq!(errors, Vec::<String>::new());
            assert_eq!(checked.unmatched.len(), 2);
        }

        #[test]
        fn check_should_read_a_url_literal() {
            // GIVEN
            let api = [(
                "    fn url(&self",
                "    pub async fn ping(&self) -> ApiResult<u16> {\n        self.client.delete(\"/api/internal\").await\n    }\n\n    fn url(&self",
            )];
            let server = [("routing::get(internal)", "routing::delete(internal)")];

            // WHEN
            let (checked, errors) =
                checked(&edited(ROUTES_API, &api), &edited(ROUTES_SERVER, &server));

            // THEN
            assert_eq!(errors, Vec::<String>::new());
            assert!(checked.unmatched.is_empty());
        }

        #[test]
        fn check_should_read_a_value_by_its_to_string() {
            // GIVEN
            let api = [
                (
                    "format!(\"/items/{owner}/{id}\")",
                    "format!(\"/items/{owner}/{}\", id.to_string())",
                ),
                (
                    "format!(\"/items/{owner}/add?kind={kind}&force=true\")",
                    "format!(\"/items/{owner}/add?kind={}&force=true\", kind.to_string())",
                ),
            ];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
        }

        #[test]
        fn check_should_read_the_body_of_an_option_value() {
            // GIVEN
            let api = [
                (
                    "    pub async fn login(&self, name: &str, password: &str) -> ApiResult<String> {\n",
                    "    pub async fn login(&self, name: &str, password: &str) -> ApiResult<String> {\n        let body = Some(Login {\n            name: name.to_string(),\n            password: password.to_string(),\n        });\n",
                ),
                (
                    "                Some(Login {\n                    name: name.to_string(),\n                    password: password.to_string(),\n                }),",
                    "                body,",
                ),
            ];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(errors, Vec::<String>::new());
        }

        #[test]
        fn check_response_outside_a_tuple_should_fail() {
            // GIVEN
            let api = [(
                "let body = response.json::<T>().await?;\n        Ok((200, body))",
                "let body = response.json::<T>().await?;\n        Ok((200, (0, body)))",
            )];

            // WHEN
            let errors = errors(&api, &[]);

            // THEN
            assert_eq!(
                errors,
                [
                    "/api/src/lib.rs:138: `Api::items`: `get /api/items/{owner}/list`: the type of the response the client reads is not known"
                ]
            );
        }
    }

    mod ty_text {
        use super::*;

        #[test]
        fn ty_text_should_write_every_type() {
            // GIVEN
            let tys = [
                Ty::Option(Box::new(Ty::Str)),
                Ty::Result(Box::new(Ty::Unit), Box::new(Ty::Bool)),
                Ty::Tuple(vec![Ty::Float, Ty::Param("T".into())]),
                Ty::Named(
                    "Map".into(),
                    vec![Ty::Str, Ty::Named("V".into(), Vec::new())],
                ),
                Ty::Ref(Box::new(Ty::Str)),
                Ty::Unknown,
            ];

            // WHEN
            let text: Vec<String> = tys.iter().map(ty_text).collect();

            // THEN
            assert_eq!(
                text,
                [
                    "Option<String>",
                    "Result<(), bool>",
                    "(f64, T)",
                    "Map<String, V>",
                    "&String",
                    "_"
                ]
            );
        }
    }

    mod checker {
        use super::*;

        mod wire_eq {
            use super::*;

            /// Whether the `server` type of the server source and the
            /// `client` type of the client source are serialized the same.
            fn eq(server: (&str, &str), client: (&str, &str)) -> bool {
                let source = |s: &str| format!("use serde::Serialize;\npub struct Root;\n{s}");
                let module = |(s, name): (&str, &str)| {
                    let names: Vec<&str> = s
                        .split(['{', '(', ';', ' ', '<', '\n'])
                        .zip(s.split(['{', '(', ';', ' ', '<', '\n']).skip(1))
                        .filter(|(k, _)| *k == "struct" || *k == "enum")
                        .map(|(_, n)| n)
                        .collect();
                    let _ = name;
                    lower(&source(s), &[], &names).0
                };
                let (server_m, client_m) = (module(server), module(client));
                let routes = Routes {
                    routes: Vec::new(),
                    module: server_m,
                };
                let transport = Transport::new();
                let checker = Checker {
                    client: vec![&client_m],
                    server: vec![&routes.module],
                    transport: &transport,
                    routes: &routes,
                    used: BTreeSet::new(),
                    checked: Checked::default(),
                    errors: Vec::new(),
                };
                checker.wire_eq(&Ty::named(server.1), &Ty::named(client.1))
            }

            #[test]
            fn wire_eq_should_compare_the_fields_by_their_serialized_names() {
                // GIVEN
                let server = "#[derive(Serialize)]\npub struct A {\n    #[serde(rename = \"n\")]\n    pub name: String,\n    #[serde(skip)]\n    pub cache: u8,\n    pub list: Vec<Option<(u8, bool)>>,\n}";
                let client = "#[derive(Serialize)]\npub struct B {\n    pub n: String,\n    pub list: Vec<Option<(u8, bool)>>,\n}";

                // WHEN
                let eq = eq((server, "A"), (client, "B"));

                // THEN
                assert!(eq);
            }

            #[test]
            fn wire_eq_should_compare_the_fields() {
                // GIVEN
                let server =
                    "#[derive(Serialize)]\npub struct A {\n    pub a: u8,\n    pub b: (u8, u8),\n}";
                let other = [
                    "#[derive(Serialize)]\npub struct B {\n    pub a: u8,\n}",
                    "#[derive(Serialize)]\npub struct B {\n    pub a: u16,\n    pub b: (u8, u8),\n}",
                    "#[derive(Serialize)]\npub struct B {\n    #[serde(default)]\n    pub a: u8,\n    pub b: (u8, u8),\n}",
                    "#[derive(Serialize)]\npub struct B {\n    pub a: u8,\n    pub b: (u8,),\n}",
                    "#[derive(Serialize)]\npub enum B {\n    A,\n}",
                ];

                // WHEN
                let eq: Vec<bool> = other.iter().map(|c| eq((server, "A"), (c, "B"))).collect();

                // THEN
                assert_eq!(eq, [false; 5]);
            }

            #[test]
            fn wire_eq_should_unwrap_newtypes_and_generics() {
                // GIVEN
                let server = "#[derive(Serialize)]\npub struct A(pub Vec<W<u8>>);\n#[derive(Serialize)]\n#[serde(transparent)]\npub struct W<T> {\n    pub value: T,\n}";
                let client = "#[derive(Serialize)]\npub struct B {\n    pub list: Vec<u8>,\n}\n#[derive(Serialize)]\npub struct C(pub Vec<u8>);";

                // WHEN
                let eq = (
                    eq((server, "A"), (client, "C")),
                    eq((server, "A"), (client, "B")),
                );

                // THEN
                assert_eq!(eq, (true, false));
            }

            #[test]
            fn wire_eq_should_compare_the_variants() {
                // GIVEN
                let server = "#[derive(Serialize)]\npub enum A {\n    #[serde(rename = \"u\")]\n    Unit,\n    #[serde(skip)]\n    Skipped,\n    Tuple(u8),\n    Struct { a: u8 },\n}";
                let client = "#[derive(Serialize)]\npub enum B {\n    u,\n    Tuple(u8),\n    Struct { a: u8 },\n}";
                let other = [
                    "#[derive(Serialize)]\npub enum B {\n    u,\n    Tuple(u8),\n}",
                    "#[derive(Serialize)]\npub enum B {\n    u,\n    Tuple(u16),\n    Struct { a: u8 },\n}",
                    "#[derive(Serialize)]\npub enum B {\n    u,\n    Tuple,\n    Struct { a: u8 },\n}",
                    "#[derive(Serialize)]\n#[serde(untagged)]\npub enum B {\n    u,\n    Tuple(u8),\n    Struct { a: u8 },\n}",
                ];

                // WHEN
                let same = eq((server, "A"), (client, "B"));
                let others: Vec<bool> = other.iter().map(|c| eq((server, "A"), (c, "B"))).collect();

                // THEN
                assert!(same);
                assert_eq!(others, [false; 4]);
            }

            #[test]
            fn wire_eq_should_compare_recursive_types() {
                // GIVEN
                let server = "#[derive(Serialize)]\npub struct A {\n    pub children: Vec<A>,\n}";
                let client = "#[derive(Serialize)]\npub struct B {\n    pub children: Vec<B>,\n}";

                // WHEN
                let eq = eq((server, "A"), (client, "B"));

                // THEN
                assert!(eq);
            }

            #[test]
            fn wire_eq_should_take_the_rust_names_without_serde() {
                // GIVEN
                let server =
                    "pub struct A {\n    pub a: u8,\n    pub e: E,\n}\npub enum E {\n    V(u8),\n}";
                let client =
                    "pub struct B {\n    pub a: u8,\n    pub e: F,\n}\npub enum F {\n    V(u8),\n}";

                // WHEN
                let eq = eq((server, "A"), (client, "B"));

                // THEN
                assert!(eq);
            }

            #[test]
            fn wire_eq_should_not_match_unknown_types() {
                // GIVEN
                let server = "#[derive(Serialize)]\npub struct A {\n    pub a: Unknown,\n}";
                let client = "#[derive(Serialize)]\npub struct B {\n    pub a: Unknown,\n}";

                // WHEN
                let eq = eq((server, "A"), (client, "B"));

                // THEN
                assert!(!eq);
            }
        }
    }
}
