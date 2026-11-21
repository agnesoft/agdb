use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;

/// Types the expressions and resolves what [`lower`](crate::SourceCrate::lower)
/// left by name: method calls into member, associated, trait and std
/// (intrinsic) calls, paths into variants and constants, `.into()` and
/// `From` into conversions and calls of tuple structs into constructions.
/// The type a Rust expression gets from its context (an `.into()`, a
/// literal, `None`, an empty `vec![Some("bool"), Some("bool"), Some("str"), None]`) comes from the expected type.
///
/// What it cannot resolve is a [`K::Unsupported`] and among the
/// [`Func::errors`]; in the bodies of the conversions an error.
pub struct Resolve;

impl Pass for Resolve {
    fn name(&self) -> &str {
        "resolve"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let snapshot = m.clone();

        for (name, default) in m.defaults.iter_mut() {
            let ty = Ty::named(name);
            let f = Func::new("default", Vec::new(), ty.clone(), Vec::new());
            let mut c = Ctx::new(&snapshot, name, ty.clone(), &[], &f);
            c.expr(default, Some(&ty));
        }

        let snapshot = m.clone();

        for (name, class) in m.classes.iter_mut() {
            let self_ty = Ty::Named(
                name.clone(),
                class
                    .generics
                    .iter()
                    .map(|g| Ty::Param(g.name.clone()))
                    .collect(),
            );

            for f in class
                .methods
                .iter_mut()
                .chain(class.trait_impls.values_mut().flatten())
            {
                Ctx::new(&snapshot, name, self_ty.clone(), &class.generics, f).func(f);
            }
        }

        for (name, e) in m.enums.iter_mut() {
            for f in e.methods.iter_mut() {
                Ctx::new(&snapshot, name, Ty::named(name), &[], f).func(f);
            }
        }

        for (name, funcs) in m.traits.iter_mut() {
            let this = [Generic {
                name: "Self".into(),
                bounds: vec![(name.clone(), None)],
            }];

            for f in funcs.iter_mut().filter(|f| !f.body.is_empty()) {
                Ctx::new(&snapshot, "", Ty::Param("Self".into()), &this, f).func(f);
            }
        }

        for f in m.functions.iter_mut().chain(m.tests.iter_mut()) {
            Ctx::new(&snapshot, "", Ty::Unknown, &[], f).func(f);
        }

        for conv in m.conversions.iter_mut() {
            let name = format!("from_{}", conv.to);
            let f = Func {
                generics: conv.generics.clone(),
                ..Func::new(
                    &name,
                    vec![("v".into(), conv.from.clone())],
                    Ty::named(&conv.to),
                    Vec::new(),
                )
            };
            let mut c = Ctx::new(&snapshot, &conv.to, Ty::named(&conv.to), &[], &f);
            c.expr(&mut conv.body, Some(&Ty::named(&conv.to)));
            conv.errors.append(&mut c.errors);
        }
    }
}

/// The methods of iterators and copies, the value itself outside of Rust.
const IDENTITY: &[&str] = &[
    "iter",
    "into_iter",
    "collect",
    "clone",
    "to_owned",
    "to_string",
    "as_str",
    "to_vec",
    "as_slice",
];

struct Ctx<'a> {
    m: &'a Module,
    class: String,
    self_ty: Ty,
    generics: Vec<Generic>,
    locals: Vec<(String, Ty)>,
    ret: Ty,
    errors: Vec<String>,
}

impl<'a> Ctx<'a> {
    fn new(m: &'a Module, class: &str, self_ty: Ty, class_generics: &[Generic], f: &Func) -> Self {
        let mut generics = class_generics.to_vec();
        generics.extend(f.generics.iter().cloned());

        Self {
            m,
            class: class.to_owned(),
            self_ty,
            generics,
            locals: f.params.clone(),
            ret: f.ret.clone(),
            errors: Vec::new(),
        }
    }

    fn func(mut self, f: &mut Func) {
        self.ret = self.self_subst(&f.ret);

        for s in f.body.iter_mut() {
            self.stmt(s);
        }

        f.ret = self.ret.clone();
        f.params = f
            .params
            .iter()
            .map(|(n, t)| (n.clone(), self.self_subst(t)))
            .collect();
        f.errors.append(&mut self.errors);
    }

    fn self_subst(&self, t: &Ty) -> Ty {
        subst(t, &self.self_ty, &[])
    }

    fn bound(&self, param: &str, trait_: &str) -> Option<Option<Ty>> {
        self.generics
            .iter()
            .find(|g| g.name == param)
            .and_then(|g| g.bounds.iter().find(|(t, _)| t == trait_))
            .map(|(_, a)| a.clone())
    }

    fn trait_with(&self, param: &str, method: &str) -> Option<(String, &'a Func)> {
        let g = self.generics.iter().find(|g| g.name == param)?;
        let mut traits: Vec<String> = g.bounds.iter().map(|(t, _)| t.clone()).collect();
        let mut i = 0;

        while i < traits.len() {
            let t = traits[i].clone();

            if let Some(f) = self
                .m
                .traits
                .get(&t)
                .and_then(|fs| fs.iter().find(|f| f.name == method))
            {
                return Some((t, f));
            }

            traits.extend(self.m.supertraits.get(&t).into_iter().flatten().cloned());
            i += 1;
        }

        None
    }

    /// The list the non-generic conversions into the `Into` target of `f`'s
    /// generic `t` take, when they all agree on it: what Rust infers the
    /// elements of an array argument from.
    fn list_source(&self, f: &Func, t: &Ty) -> Option<Ty> {
        let Ty::Param(p) = t else {
            return None;
        };
        let Some((_, Some(Ty::Named(to, _)))) = f
            .generics
            .iter()
            .filter(|g| g.name == *p)
            .flat_map(|g| &g.bounds)
            .find(|(t, _)| t == "Into")
        else {
            return None;
        };
        let mut lists = self
            .m
            .conversions
            .iter()
            .filter(|c| c.to == *to && c.generics.is_empty())
            .map(|c| c.from.strip_refs())
            .filter(|t| matches!(t, Ty::List(_)));
        let first = lists.next()?;

        lists.all(|l| l == first).then_some(first)
    }

    fn local(&self, n: &str) -> Option<Ty> {
        self.locals
            .iter()
            .rev()
            .find(|(l, _)| l == n)
            .map(|(_, t)| t.clone())
    }

    fn scoped(&mut self, body: &mut [Expr], binds: Vec<(String, Ty)>) {
        let len = self.locals.len();
        self.locals.extend(binds);

        for s in body.iter_mut() {
            self.stmt(s);
        }

        self.locals.truncate(len);
    }

    fn stmt(&mut self, e: &mut Expr) {
        self.expr(e, None);
    }

    fn unsupported(&mut self, e: &mut Expr, what: String) {
        self.errors.push(what.clone());
        e.k = K::Unsupported(what);
    }

    fn expr(&mut self, e: &mut Expr, expected: Option<&Ty>) {
        match &mut e.k {
            K::This => e.ty = self.self_ty.clone(),
            K::Local(n) => {
                if let Some(t) = self.local(n) {
                    e.ty = t;
                }
            }
            K::Path(segs) => {
                let segs = segs.clone();

                if let Some(v) = self.variant_path(&segs, None, expected) {
                    *e = v;
                } else if let Some(c) = segs.last().and_then(|l| self.m.consts.get(l)) {
                    *e = c.clone();
                } else {
                    self.unsupported(e, format!("unknown path `{}`", segs.join("::")));
                }
            }
            K::Call {
                callee: Callee::Path(segs, gens),
                args,
                ..
            } => {
                let segs = segs.clone();
                let gens = gens.clone();
                let args = std::mem::take(args);
                self.call_path(e, segs, gens, args, expected);
            }
            K::Call {
                callee: Callee::Method(m, types),
                recv: Some(r),
                args,
            } => {
                let m = m.clone();
                let types = types.clone();
                let mut r = std::mem::replace(r.as_mut(), Expr::new(K::This));
                let args = std::mem::take(args);
                let through = IDENTITY.contains(&m.as_str()) || m == "map";
                self.expr(&mut r, if through { expected } else { None });
                self.method_call(e, (m, types), r, args, expected);
            }
            K::Field { base, name } => {
                self.expr(base, None);
                let name = name.clone();
                e.ty = self.field_ty(&base.ty, &name);
            }
            K::Construct { class, fields } => {
                let class = if class == "Self" {
                    self.class.clone()
                } else {
                    class.clone()
                };
                let mut fields = std::mem::take(fields);

                for (n, f) in fields.iter_mut() {
                    let t = self.m.field_ty(&class, n);
                    self.expr(f, Some(&t));
                }

                let ty = self.infer_args(&class, &fields);
                *e = Expr::typed(K::Construct { class, fields }, ty);
            }
            K::Assign { place, value } => {
                self.expr(place, None);
                let t = place.ty.strip_ref().clone();
                self.expr(value, Some(&t));
                e.ty = Ty::Unit;
            }
            K::Let { name, value } => {
                let name = name.clone();
                let t = match value {
                    Some(v) => {
                        let annotated = std::mem::replace(&mut v.ty, Ty::Unknown);
                        let expected = (annotated != Ty::Unknown).then_some(annotated);
                        self.expr(v, expected.as_ref());

                        if let Some(t) = expected {
                            v.ty = t;
                        }

                        v.ty.clone()
                    }
                    None => Ty::Unknown,
                };
                self.locals.push((name, t));
                e.ty = Ty::Unit;
            }
            K::If { cond, then, els } => {
                self.expr(cond, Some(&Ty::Bool));
                self.scoped(then, Vec::new());

                if let Some(els) = els {
                    self.scoped(els, Vec::new());
                }

                e.ty = Ty::Unit;
            }
            K::IfLet {
                pat,
                value,
                then,
                els,
            } => {
                self.expr(value, None);
                let binds = self.bind(pat, value.ty.strip_ref());
                self.scoped(then, binds);

                if let Some(els) = els {
                    self.scoped(els, Vec::new());
                }

                e.ty = Ty::Unit;
            }
            K::Match { value, arms } => {
                self.expr(value, None);
                let vt = value.ty.strip_ref().clone();

                for (p, body) in arms.iter_mut() {
                    let binds = self.bind(p, &vt);
                    self.scoped(body, binds);
                }

                e.ty = Ty::Unit;
            }
            K::While { cond, body } => {
                self.expr(cond, Some(&Ty::Bool));
                self.scoped(body, Vec::new());
                e.ty = Ty::Unit;
            }
            K::Return(v) => {
                let ret = self.ret.clone();

                if let Some(v) = v {
                    self.expr(v, Some(&ret));
                }

                e.ty = Ty::Unit;
            }
            K::Binary { op, l, r } => {
                self.expr(l, None);
                let lt = l.ty.clone();
                self.expr(r, Some(&lt));
                e.ty = match *op {
                    "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" => Ty::Bool,
                    _ => lt,
                };
            }
            K::Not(v) => {
                self.expr(v, Some(&Ty::Bool));
                e.ty = Ty::Bool;
            }
            K::Cast(v) | K::Fields(v) => self.expr(v, None),
            K::Ref(v) => {
                self.expr(v, expected.map(|t| t.strip_ref()));
                e.ty = Ty::Ref(Box::new(v.ty.clone()));
            }
            K::Deref(v) => {
                self.expr(v, None);
                e.ty = v.ty.strip_ref().clone();
            }
            K::Index { base, index } => {
                self.expr(base, None);
                self.expr(index, Some(&Ty::Int("usize")));
                e.ty = match base.ty.strip_ref() {
                    Ty::List(t) => (**t).clone(),
                    _ => Ty::Unknown,
                };
            }
            K::Array(items) => {
                let el = match expected {
                    Some(Ty::List(t)) => Some((**t).clone()),
                    _ => None,
                };

                for i in items.iter_mut() {
                    self.expr(i, el.as_ref());
                }

                let el = el
                    .or_else(|| items.first().map(|i| i.ty.clone()))
                    .unwrap_or(Ty::Unknown);
                e.ty = Ty::List(Box::new(el));
            }
            K::Tuple(items) => {
                let exp = match expected {
                    Some(Ty::Tuple(ts)) => ts.clone(),
                    _ => Vec::new(),
                };

                for (i, it) in items.iter_mut().enumerate() {
                    self.expr(it, exp.get(i));
                }

                e.ty = Ty::Tuple(items.iter().map(|i| i.ty.clone()).collect());
            }
            K::Lit(l) => {
                e.ty = match (l, expected.map(|t| t.strip_ref())) {
                    (Lit::Int(_), Some(t @ (Ty::Int(_) | Ty::Float))) => t.clone(),
                    (Lit::Int(_), _) => Ty::Int("i32"),
                    (Lit::Float(_), _) => Ty::Float,
                    (Lit::Bool(_), _) => Ty::Bool,
                    (Lit::Str(_), _) => Ty::Str,
                    (Lit::Unit, _) => Ty::Unit,
                };
            }
            K::Panic(_) => e.ty = expected.cloned().unwrap_or(Ty::Unit),
            K::Convert { value, to } => {
                self.expr(value, None);
                e.ty = Ty::named(to);
            }
            K::Variant { en, var, payload } => {
                let pt = self.payload_ty(en, var);

                if let Some(p) = payload {
                    self.expr(p, pt.as_ref());
                }

                e.ty = Ty::named(en);
            }
            K::Call {
                callee: Callee::Intrinsic(i),
                recv,
                args,
            } => {
                let i = *i;

                if let Some(r) = recv {
                    self.expr(r, None);
                }

                if let (Intrinsic::AssertEq | Intrinsic::AssertNe, [left, right]) =
                    (i, args.as_mut_slice())
                {
                    self.expr(left, None);
                    let ty = left.ty.strip_ref().clone();
                    self.expr(right, Some(&ty));
                    return;
                }

                let el = recv.as_ref().and_then(|r| match r.ty.strip_ref() {
                    Ty::List(t) => Some((**t).clone()),
                    _ => None,
                });

                for a in args.iter_mut() {
                    self.closure_or_expr(a, el.as_ref(), None);
                }

                if i == Intrinsic::Map
                    && let Some(K::Closure { body, .. }) = args.first().map(|a| &a.k)
                {
                    e.ty = Ty::List(Box::new(returned_ty(body).unwrap_or(Ty::Unknown)));
                }
            }
            K::Call { args, .. } => {
                for a in args.iter_mut() {
                    self.expr(a, None);
                }
            }
            K::Await(v) => {
                self.expr(v, expected);
                e.ty = v.ty.clone();
            }
            K::Try(v) => {
                self.expr(v, None);
                e.ty = match &v.ty {
                    Ty::Result(ok, _) => (**ok).clone(),
                    _ => Ty::Unknown,
                };
            }
            K::Format(pieces) => {
                for p in pieces.iter_mut() {
                    self.expr(p, None);
                    self.display(p);
                }

                e.ty = Ty::Str;
            }
            K::Ident(_)
            | K::Closure { .. }
            | K::Throw(_)
            | K::Unsupported(_)
            | K::Payload { .. }
            | K::Unwrap(_)
            | K::TypeCase { .. } => {}
        }
    }

    /// A piece of a `format!` of a named type as the call of its `Display`,
    /// lowered as its `to_string`.
    fn display(&mut self, p: &mut Expr) {
        let Ty::Named(class, _) = p.ty.strip_ref().clone() else {
            return;
        };

        if self.m.method(&class, "to_string").is_none() {
            self.unsupported(p, format!("no `Display` of `{class}`"));
            return;
        }

        let recv = std::mem::replace(p, Expr::new(K::This));
        *p = Expr::typed(
            K::Call {
                callee: Callee::Member {
                    class,
                    name: "to_string".into(),
                },
                recv: Some(recv.boxed()),
                args: Vec::new(),
            },
            Ty::Str,
        );
    }

    fn payload_ty(&self, en: &str, var: &str) -> Option<Ty> {
        match self.m.variant(en, var) {
            Some(Variant::Tuple(_, t)) => Some(t.clone()),
            _ => None,
        }
    }

    /// An argument of an intrinsic; a function path is the closure calling
    /// it.
    fn closure_or_expr(&mut self, a: &mut Expr, param_ty: Option<&Ty>, ret: Option<&Ty>) {
        if let K::Path(segs) = &a.k {
            let call = Expr::new(K::Call {
                callee: Callee::Path(segs.clone(), Vec::new()),
                recv: None,
                args: vec![Expr::new(K::Local("x".into()))],
            });
            a.k = K::Closure {
                params: vec!["x".into()],
                body: vec![call],
            };
        }

        if let K::Closure { params, body } = &mut a.k {
            let binds: Vec<(String, Ty)> = params
                .iter()
                .map(|p| (p.clone(), param_ty.cloned().unwrap_or(Ty::Unknown)))
                .collect();
            let len = self.locals.len();
            self.locals.extend(binds);
            let tail = body.len().saturating_sub(1);

            for (i, s) in body.iter_mut().enumerate() {
                self.expr(s, if i == tail { ret } else { None });
            }

            self.locals.truncate(len);
            a.ty = Ty::Fn;
        } else {
            self.expr(a, param_ty);
        }
    }

    fn variant_path(
        &self,
        segs: &[String],
        payload: Option<Expr>,
        expected: Option<&Ty>,
    ) -> Option<Expr> {
        if let [var] = segs
            && (var == "Ok" || var == "Err")
        {
            let ty = match expected {
                Some(t @ Ty::Result(..)) => t.clone(),
                _ => Ty::Result(Box::new(Ty::Unknown), Box::new(Ty::Unknown)),
            };
            let mut v = Expr::variant("Result", var, payload);
            v.ty = ty;
            return Some(v);
        }

        if let [var] = segs
            && (var == "None" || var == "Some")
        {
            let ty = match expected {
                Some(t @ Ty::Option(_)) => t.clone(),
                _ => Ty::Option(Box::new(
                    payload
                        .as_ref()
                        .map(|p| p.ty.clone())
                        .unwrap_or(Ty::Unknown),
                )),
            };
            let mut v = Expr::variant("Option", var, payload);
            v.ty = ty;
            return Some(v);
        }

        let [.., en, var] = segs else {
            return None;
        };
        let en = if en == "Self" { &self.class } else { en };
        self.m.variant(en, var)?;
        Some(Expr::variant(en, var, payload))
    }

    fn call_path(
        &mut self,
        e: &mut Expr,
        segs: Vec<String>,
        gens: Vec<Ty>,
        mut args: Vec<Expr>,
        expected: Option<&Ty>,
    ) {
        let segs: Vec<String> = segs
            .into_iter()
            .map(|s| if s == "Self" { self.class.clone() } else { s })
            .collect();
        let last = segs.last().cloned().unwrap_or_default();
        let prev = if segs.len() >= 2 {
            segs[segs.len() - 2].clone()
        } else {
            String::new()
        };

        if segs.len() == 1 && ["Some", "Ok", "Err"].contains(&last.as_str()) && !args.is_empty() {
            let inner = match (expected, last.as_str()) {
                (Some(Ty::Option(t)), "Some") | (Some(Ty::Result(t, _)), "Ok") => {
                    Some((**t).clone())
                }
                (Some(Ty::Result(_, t)), "Err") => Some((**t).clone()),
                _ => None,
            };
            let mut p = args.remove(0);
            self.expr(&mut p, inner.as_ref());
            *e = self
                .variant_path(&segs, Some(p), expected)
                .expect("a variant of `Option` or `Result`");
            return;
        }

        if segs.len() >= 2 {
            let en = prev.clone();

            if self.m.variant(&en, &last).is_some() && !args.is_empty() {
                let pt = self.payload_ty(&en, &last);
                let mut p = args.remove(0);
                self.expr(&mut p, pt.as_ref());
                *e = Expr::variant(&en, &last, Some(p));
                return;
            }
        }

        if ((prev == "Into" && last == "into")
            || (last == "from" && self.m.conversions.iter().any(|c| c.to == prev)))
            && !args.is_empty()
        {
            let to = if last == "into" {
                gens.first().and_then(|t| t.name()).map(str::to_owned)
            } else {
                Some(prev.clone())
            };
            let mut v = args.remove(0);
            self.expr(&mut v, None);

            if let Some(p) = gens.first().filter(|t| t.is_primitive()) {
                *e = Expr::typed(K::Cast(v.boxed()), p.clone());
                return;
            }

            match to {
                Some(to) => {
                    *e = Expr::typed(
                        K::Convert {
                            value: v.boxed(),
                            to: to.clone(),
                        },
                        Ty::named(&to),
                    )
                }
                None => self.unsupported(e, "`into` without a target".into()),
            }

            return;
        }

        if last == "default" && args.is_empty() {
            let ty = if prev == "Default" {
                expected.map(|t| t.strip_ref().clone())
            } else {
                self.m
                    .defaults
                    .contains_key(&prev)
                    .then(|| Ty::named(&prev))
            };

            if let Some(ty) = ty {
                *e = self.m.default_of(&ty);
                self.expr(e, Some(&ty));
                return;
            }
        }

        if prev == "Vec" && last == "new" && args.is_empty() {
            *e = Expr::new(K::Array(Vec::new()));
            self.expr(e, expected);
            return;
        }

        if prev == "mem" && (last == "take" || last == "swap") && !args.is_empty() {
            for a in args.iter_mut() {
                self.expr(a, None);
            }

            let (i, ty) = if last == "take" {
                (Intrinsic::MemTake, args[0].ty.strip_ref().clone())
            } else {
                (Intrinsic::MemSwap, Ty::Unit)
            };
            *e = Expr::typed(
                K::Call {
                    callee: Callee::Intrinsic(i),
                    recv: None,
                    args,
                },
                ty,
            );
            return;
        }

        if segs.len() == 1 && self.m.classes.contains_key(&last) {
            let mut fields: Vec<(String, Expr)> = args
                .into_iter()
                .enumerate()
                .map(|(i, a)| (i.to_string(), a))
                .collect();

            for (n, f) in fields.iter_mut() {
                let t = self.m.field_ty(&last, n);
                self.expr(f, Some(&t));
            }

            let ty = self.infer_args(&last, &fields);
            *e = Expr::typed(
                K::Construct {
                    class: last,
                    fields,
                },
                ty,
            );
            return;
        }

        if let Some((t, f)) = self.trait_with(&prev, &last) {
            for a in args.iter_mut() {
                self.expr(a, None);
            }

            let ty = subst(&f.ret, &Ty::Param(prev.clone()), &[]);
            *e = Expr::typed(
                K::Call {
                    callee: Callee::Trait {
                        trait_: t,
                        name: last,
                        types: Vec::new(),
                    },
                    recv: Some(Expr::typed(K::Path(vec![prev.clone()]), Ty::Param(prev)).boxed()),
                    args,
                },
                ty,
            );
            return;
        }

        let class = prev.clone();

        if let Some(f) = self.m.method(&class, &last).cloned() {
            for (a, (_, t)) in args.iter_mut().zip(f.params.iter()) {
                self.expr(a, Some(t));
            }

            let ty = subst(&f.ret, &Ty::named(&class), &[]);
            let ty = match (&ty, expected) {
                (Ty::Named(c, a), Some(Ty::Named(ec, ea))) if a.is_empty() && c == ec => {
                    Ty::Named(c.clone(), ea.clone())
                }
                _ => ty,
            };
            *e = Expr::typed(
                K::Call {
                    callee: Callee::Static { class, name: last },
                    recv: None,
                    args,
                },
                ty,
            );
            return;
        }

        if let [name] = segs.as_slice()
            && let Some(f) = self.m.externs.iter().find(|f| f.name == *name).cloned()
        {
            for (a, (_, t)) in args.iter_mut().zip(f.params.iter()) {
                self.expr(a, Some(t));
            }

            *e = Expr::typed(
                K::Call {
                    callee: Callee::Extern(f.name),
                    recv: None,
                    args,
                },
                f.ret,
            );
            return;
        }

        self.unsupported(e, format!("unknown function `{}`", segs.join("::")));
    }

    fn method_call(
        &mut self,
        e: &mut Expr,
        method: (String, Vec<Ty>),
        r: Expr,
        mut args: Vec<Expr>,
        expected: Option<&Ty>,
    ) {
        let (m, types) = method;
        let rt = r.ty.strip_ref().clone();

        if m == "into" {
            let to = match &rt {
                Ty::Param(p) => self.bound(p, "Into").flatten(),
                _ => None,
            }
            .or_else(|| expected.cloned());

            if let Some(p) = to.as_ref().filter(|t| t.is_primitive()) {
                *e = Expr::typed(K::Cast(r.boxed()), p.clone());
                return;
            }

            match to.as_ref().and_then(|t| t.name()) {
                Some(to) => {
                    *e = Expr::typed(
                        K::Convert {
                            value: r.boxed(),
                            to: to.to_owned(),
                        },
                        Ty::named(to),
                    )
                }
                None => self.unsupported(e, format!("no target of `.into()` on {rt:?}")),
            }

            return;
        }

        if let Ty::Named(c, cargs) = &rt
            && let Some(f) = self.m.method(c, &m).cloned()
        {
            let params = self.class_params(c, cargs);

            for (a, (_, t)) in args.iter_mut().zip(f.params.iter()) {
                let t = matches!(a.k, K::Array(_))
                    .then(|| self.list_source(&f, t))
                    .flatten()
                    .unwrap_or_else(|| subst(t, &rt, &params));
                self.expr(a, Some(&t));
            }

            let ty = subst(&f.ret, &rt, &params);
            *e = Expr::typed(
                K::Call {
                    callee: Callee::Member {
                        class: c.clone(),
                        name: m,
                    },
                    recv: Some(r.boxed()),
                    args,
                },
                ty,
            );
            return;
        }

        if let Ty::Param(p) = &rt
            && let Some((t, f)) = self.trait_with(p, &m)
        {
            let params: Vec<(String, Ty)> = f
                .generics
                .iter()
                .map(|g| g.name.clone())
                .zip(types.iter().cloned())
                .collect();

            for (a, (_, t)) in args.iter_mut().zip(f.params.iter()) {
                let t = subst(t, &rt, &params);
                self.expr(a, Some(&t));
            }

            let ty = subst(&f.ret, &rt, &params);
            *e = Expr::typed(
                K::Call {
                    callee: Callee::Trait {
                        trait_: t,
                        name: m,
                        types,
                    },
                    recv: Some(r.boxed()),
                    args,
                },
                ty,
            );
            return;
        }

        let intrinsic = match (&rt, m.as_str()) {
            (Ty::List(t), "push") => Some((Intrinsic::Push, Ty::Unit, Some((**t).clone()))),
            (Ty::List(t), "pop") => Some((Intrinsic::Pop, Ty::Option(t.clone()), None)),
            (Ty::List(t), "last_mut") => Some((
                Intrinsic::LastMut,
                Ty::Option(Box::new(Ty::Ref(t.clone()))),
                None,
            )),
            (Ty::List(_), "len") => Some((Intrinsic::Len, Ty::Int("usize"), None)),
            (Ty::List(_) | Ty::Str, "is_empty") => Some((Intrinsic::IsEmpty, Ty::Bool, None)),
            (Ty::Option(_), "is_some") => Some((Intrinsic::IsSome, Ty::Bool, None)),
            (Ty::Option(_), "is_none") => Some((Intrinsic::IsNone, Ty::Bool, None)),
            (Ty::Option(t), "map") => Some((Intrinsic::MapSome, Ty::Unknown, Some((**t).clone()))),
            (Ty::List(_), "extend" | "append") => {
                Some((Intrinsic::Extend, Ty::Unit, Some(rt.clone())))
            }
            (Ty::List(_), "reserve") => Some((Intrinsic::Reserve, Ty::Unit, None)),
            (Ty::List(t), "for_each") => {
                Some((Intrinsic::IterForEach, Ty::Unit, Some((**t).clone())))
            }
            (Ty::List(t), "map") => Some((Intrinsic::Map, Ty::Unknown, Some((**t).clone()))),
            (Ty::Result(t, _), "map") => Some((Intrinsic::MapOk, Ty::Unknown, Some((**t).clone()))),
            (Ty::Str, "starts_with") => Some((Intrinsic::StartsWith, Ty::Bool, Some(Ty::Str))),
            (Ty::Option(t), "unwrap_or_default") => {
                Some((Intrinsic::UnwrapOrDefault, (**t).clone(), None))
            }
            (Ty::Option(t), "expect") => Some((Intrinsic::Expect, (**t).clone(), None)),
            (Ty::Option(t) | Ty::Result(t, _), "unwrap") => {
                Some((Intrinsic::Unwrap, (**t).clone(), None))
            }
            (Ty::Result(t, _), "expect") => Some((Intrinsic::Unwrap, (**t).clone(), None)),
            (Ty::Result(_, e), "unwrap_err") => Some((Intrinsic::UnwrapErr, (**e).clone(), None)),
            (Ty::List(t), "find") => {
                Some((Intrinsic::Find, Ty::Option(t.clone()), Some((**t).clone())))
            }
            (Ty::List(t), "any") => Some((Intrinsic::Any, Ty::Bool, Some((**t).clone()))),
            (Ty::List(t), "all") => Some((Intrinsic::All, Ty::Bool, Some((**t).clone()))),
            (Ty::List(t), "sort_by_key") => {
                Some((Intrinsic::SortByKey, Ty::Unit, Some((**t).clone())))
            }
            (Ty::List(t), "remove") => {
                Some((Intrinsic::Remove, (**t).clone(), Some(Ty::Int("usize"))))
            }
            (Ty::Str, "repeat") => Some((Intrinsic::Repeat, Ty::Str, Some(Ty::Int("usize")))),
            _ => None,
        };

        if let Some((i, ty, arg_ty)) = intrinsic {
            let ret = match (i, expected) {
                (Intrinsic::Map, Some(Ty::List(t))) => Some((**t).clone()),
                _ => None,
            };

            for a in args.iter_mut() {
                self.closure_or_expr(a, arg_ty.as_ref(), ret.as_ref());
            }

            let returned = match args.first().map(|a| &a.k) {
                Some(K::Closure { body, .. }) => returned_ty(body).unwrap_or(Ty::Unknown),
                _ => Ty::Unknown,
            };
            let ty = match (i, &rt) {
                (Intrinsic::Map, _) => Ty::List(Box::new(returned)),
                (Intrinsic::MapOk, Ty::Result(_, err)) => {
                    Ty::Result(Box::new(returned), err.clone())
                }
                (Intrinsic::MapSome, _) => Ty::Option(Box::new(returned)),
                _ => ty,
            };
            *e = Expr::typed(
                K::Call {
                    callee: Callee::Intrinsic(i),
                    recv: Some(r.boxed()),
                    args,
                },
                ty,
            );
            return;
        }

        if m == "to_string" && rt.is_primitive() && rt != Ty::Str {
            *e = Expr::typed(K::Format(vec![r]), Ty::Str);
            return;
        }

        if IDENTITY.contains(&m.as_str()) {
            *e = r;
            return;
        }

        self.unsupported(e, format!("unknown method `{m}` on {rt:?}"));
    }

    fn class_params(&self, class: &str, args: &[Ty]) -> Vec<(String, Ty)> {
        self.m
            .generics_of(class)
            .iter()
            .zip(args)
            .map(|(g, a)| (g.name.clone(), a.clone()))
            .collect()
    }

    fn field_ty(&self, base: &Ty, name: &str) -> Ty {
        match base.strip_ref() {
            Ty::Named(c, args) => subst(
                &self.m.field_ty(c, name),
                base.strip_ref(),
                &self.class_params(c, args),
            ),
            Ty::Tuple(ts) => name
                .parse::<usize>()
                .ok()
                .and_then(|i| ts.get(i).cloned())
                .unwrap_or(Ty::Unknown),
            _ => Ty::Unknown,
        }
    }

    /// The type of a generic class with the type arguments from the values
    /// of its fields.
    fn infer_args(&self, class: &str, fields: &[(String, Expr)]) -> Ty {
        let args = self
            .m
            .generics_of(class)
            .iter()
            .map(|g| {
                fields
                    .iter()
                    .find_map(|(n, v)| match self.m.field_ty(class, n) {
                        Ty::Param(p) if p == g.name => Some(v.ty.strip_ref().clone()),
                        Ty::Named(_, a) if a.contains(&Ty::Param(g.name.clone())) => {
                            match v.ty.strip_ref() {
                                Ty::Named(_, va) => va.first().cloned(),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .unwrap_or(Ty::Param(g.name.clone()))
            })
            .collect();

        Ty::Named(class.to_owned(), args)
    }

    fn bind(&mut self, p: &mut Pat, t: &Ty) -> Vec<(String, Ty)> {
        match p {
            Pat::Wild | Pat::Lit(_) => Vec::new(),
            Pat::Bind(n) => vec![(n.clone(), t.clone())],
            Pat::Variant { en, var, inner } => {
                let t = t.strip_ref();

                if let Ty::Option(it) = t {
                    *en = "Option".into();

                    return if var == "Some" {
                        inner.iter_mut().flat_map(|i| self.bind(i, it)).collect()
                    } else {
                        Vec::new()
                    };
                }

                if let Ty::Named(n, _) = t {
                    *en = n.clone();
                }

                match self.m.variant(en, var) {
                    Some(Variant::Tuple(_, Ty::Tuple(ts))) if inner.len() > 1 => {
                        let ts = ts.clone();
                        inner
                            .iter_mut()
                            .zip(ts)
                            .flat_map(|(i, t)| self.bind(i, &t))
                            .collect()
                    }
                    Some(Variant::Tuple(_, pt)) => {
                        let pt = pt.clone();
                        inner.iter_mut().flat_map(|i| self.bind(i, &pt)).collect()
                    }
                    _ => Vec::new(),
                }
            }
            Pat::Struct { name, fields } => {
                let decl = self
                    .m
                    .classes
                    .get(name)
                    .map(|c| c.fields.clone())
                    .unwrap_or_default();

                fields
                    .iter_mut()
                    .flat_map(|(n, fp)| {
                        let ft = decl
                            .iter()
                            .find(|(d, _)| d == n)
                            .map(|(_, t)| t.clone())
                            .unwrap_or(Ty::Unknown);
                        self.bind(fp, &ft)
                    })
                    .collect()
            }
        }
    }
}

/// The type with `Self` as `self_ty` and the generic parameters of
/// `params` as their types.
pub fn subst(t: &Ty, self_ty: &Ty, params: &[(String, Ty)]) -> Ty {
    match t {
        Ty::SelfTy => self_ty.clone(),
        Ty::Param(p) => params
            .iter()
            .find(|(n, _)| n == p)
            .map(|(_, t)| t.clone())
            .unwrap_or_else(|| t.clone()),
        Ty::List(i) => Ty::List(Box::new(subst(i, self_ty, params))),
        Ty::Option(i) => Ty::Option(Box::new(subst(i, self_ty, params))),
        Ty::Result(ok, err) => Ty::Result(
            Box::new(subst(ok, self_ty, params)),
            Box::new(subst(err, self_ty, params)),
        ),
        Ty::Ref(i) => Ty::Ref(Box::new(subst(i, self_ty, params))),
        Ty::Tuple(v) => Ty::Tuple(v.iter().map(|t| subst(t, self_ty, params)).collect()),
        Ty::Named(n, a) => Ty::Named(
            n.clone(),
            a.iter().map(|t| subst(t, self_ty, params)).collect(),
        ),
        t => t.clone(),
    }
}

/// The type of the first `return` of the body, also in `if` branches.
pub fn returned_ty(body: &[Expr]) -> Option<Ty> {
    body.iter().find_map(|s| match &s.k {
        K::Return(Some(v)) => Some(v.ty.clone()),
        K::If { then, els, .. } | K::IfLet { then, els, .. } => {
            returned_ty(then).or_else(|| els.as_deref().and_then(returned_ty))
        }
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::CLIENT;
    use crate::codegen::test_utilities::SUITE;
    use crate::codegen::test_utilities::client;
    use crate::codegen::test_utilities::conversion;
    use crate::codegen::test_utilities::lower;
    use crate::codegen::test_utilities::strings;
    use crate::codegen::test_utilities::suite;

    const SOURCE: &str = r#"
        pub struct Root {
            pub ids: Vec<u64>,
            pub o: Option<u64>,
            pub q: Q<u64>,
            pub t: (u8, bool),
            pub w: Wrap<Q<u64>>,
            pub w2: Wrap2<u64>,
            pub e: E,
            pub v: V,
        }
        pub struct Q<T> {
            pub v: T,
        }
        pub struct Wrap<T> {
            pub inner: T,
        }
        pub struct Wrap2<T> {
            pub n: u8,
            pub q: Q<T>,
        }
        pub struct V(pub u64);
        pub enum E {
            A,
            B(u64),
            C(u8, bool),
        }
        pub trait Tr {
            fn keys() -> Vec<u64>;
            fn id(&self) -> u64;
            fn with(v: u64) -> u64;
            fn add(&self, v: u64) -> u64;
        }
        pub const K: u64 = 1;
        impl Default for V {
            fn default() -> Self {
                V(0)
            }
        }
        impl Tr for V {
            fn keys() -> Vec<u64> {
                vec![]
            }
            fn id(&self) -> u64 {
                self.0
            }
            fn with(v: u64) -> u64 {
                v
            }
            fn add(&self, v: u64) -> u64 {
                v
            }
        }
        impl<T> Q<T> {
            pub fn get(&self, i: u64) -> T {
                self.v
            }
            pub fn new(v: T) -> Self {
                Q { v }
            }
        }
        impl E {
            pub fn f(&self) -> u8 {
                match self {
                    E::A => 1,
                    E::B(v) => 2,
                    E::C(a, b) => 3,
                }
            }
            pub fn b() -> E {
                Self::B(1)
            }
            pub fn new() -> E {
                Self::b()
            }
            pub fn a() -> E {
                Self::A
            }
        }
        impl Root {
            pub fn std<T: Tr, I: Into<V>>(mut self, t: T, i: I) -> Self {
                let a = self.ids.len();
                let b = self.ids.is_empty();
                self.ids.push(1);
                let c = self.ids.pop();
                let l = self.ids.last_mut();
                self.ids.extend(vec![]);
                self.ids.append(&mut vec![2]);
                self.ids.reserve(1);
                self.ids.iter().for_each(|x| {
                    let y = x;
                });
                let m = self.ids.iter().map(|x| {
                    return x;
                }).collect();
                let m2 = self.ids.iter().map(|x| x).collect();
                let n = self.o.unwrap_or_default();
                let e = self.o.expect("e");
                let s = "s".to_string();
                let mt = std::mem::take(&mut self.ids);
                std::mem::swap(&mut self.ids, &mut vec![]);
                self
            }

            pub fn conversions<T: Tr, I: Into<V>>(mut self, t: T, i: I) -> Self {
                let v1 = i.into();
                let v2 = Into::<V>::into(1);
                let v3 = V::from(1);
                self.v = 1.into();
                let v4 = Into::into(1);
                let v5 = t.into();
                self
            }

            pub fn rest(mut self) -> Self {
                let p = Into::<u64>::into(1);
                self.v = Default::default();
                let dv = V::default();
                let un = Default::default();
                let g = self.q.get(self.t.0.into());
                self.ids = self.ids.iter().map(|x| *x).collect();
                let qs = self.ids.iter().map(Q::new).collect();
                let ms = self.ids.iter().map(|x| {
                    let y = *x;
                    y
                }).collect();
                self
            }

            pub fn calls<T: Tr, I: Into<V>>(mut self, t: T, i: I) -> Self {
                let tk = T::keys();
                let ti = t.id();
                let tw = T::with(1);
                let ta = t.add(1);
                let g = self.q.get(0);
                self.q = Q::new(1);
                let h = Q::new(1);
                let x = self.e.f();
                let vv = V(1);
                unknown();
                self.nothing();
                i.nothing();
                self
            }

            pub fn values(mut self) -> Root {
                let k = K;
                let none = None;
                let some = Some(1);
                self.o = Some(2);
                self.o = None;
                let eb = E::B(1);
                let ea = E::A;
                let tup = (1, true);
                self.t = (1, true);
                self.ids = vec![1];
                let arr = vec![true];
                let f0 = self.t.0;
                let f9 = self.t.9;
                let idx = self.ids[0];
                let nidx = self.o[0];
                let neg = !true;
                let r = &self.ids;
                let d = *r;
                let cmp = 1 == 2;
                let add = 1 + 2;
                let fl = 1.5;
                let st = "s";
                let w = Wrap { inner: Q { v: 1 } };
                let w2 = Wrap2 { n: 1, q: Q { v: 1 } };
                let w3 = Wrap2 { q: nothing };
                let sf = Self { o: None };
                let p = nothing;
                let pp = nothing::path;
                let fp = nothing.field;
                let u;
                panic!("p");
                return self;
            }

            pub fn control(self) -> Root {
                if let Some(x) = self.o {
                    let y = x;
                } else {
                    let z = 1;
                }
                if true {
                } else {
                }
                if true {}
                match self.e {
                    E::A => {}
                    E::B(x) => {
                        let y = x;
                    }
                    E::C(p, q) => {}
                }
                match self.o {
                    Some(x) => {}
                    None => {}
                }
                match self.ids.len() {
                    1 => {}
                    _ => {}
                }
                if let Q { v } = self.q {
                    let y = v;
                }
                match self.missing {
                    E::A => {}
                }
                while false {}
                self
            }
        }
    "#;

    fn resolve(extra: impl FnOnce(&mut Module)) -> (Module, Vec<String>) {
        let (mut m, errors) = lower(SOURCE, &["Tr"], &[]);
        assert_eq!(errors, Vec::<String>::new());
        extra(&mut m);
        let mut errors = Vec::new();
        Resolve.run(&mut m, &mut errors);
        (m, strings(&errors))
    }

    fn typed(f: &Func) -> String {
        let mut out = print::func(f);

        for s in &f.body {
            if let K::Let {
                name,
                value: Some(v),
            } = &s.k
            {
                out += &format!("\n{name}: {}", print::ty(&v.ty));
            }
        }

        out
    }

    fn method(m: &Module, class: &str, name: &str) -> String {
        typed(m.method(class, name).unwrap())
    }

    fn errors(m: &Module, class: &str, name: &str) -> Vec<String> {
        m.method(class, name).unwrap().errors.clone()
    }

    mod resolve {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_resolve() {
                    assert_eq!(Resolve.name(), "resolve");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_resolve_the_std_methods_into_intrinsics() {
                    let (m, _) = resolve(|_| {});

                    assert_eq!(
                        method(&m, "Root", "std"),
                        "fn std<T: Tr, I: Into<V>>(mut self, t: T, i: I) -> Root {\n    let a: usize = self.ids.Len();\n    let b: bool = self.ids.IsEmpty();\n    self.ids.Push(1);\n    let c: Option<u64> = self.ids.Pop();\n    let l: Option<&u64> = self.ids.LastMut();\n    self.ids.Extend(vec![]);\n    self.ids.Extend(&vec![2]);\n    self.ids.Reserve(1);\n    self.ids.IterForEach(|x| {\n    let y: u64 = x;\n});\n    let m: Vec<u64> = self.ids.Map(|x| {\n    return x;\n});\n    let m2: Vec<?> = self.ids.Map(|x| {\n    x;\n});\n    let n: u64 = self.o.UnwrapOrDefault();\n    let e: u64 = self.o.Expect(\"e\");\n    let s: str = \"s\";\n    let mt: Vec<u64> = .MemTake(&self.ids);\n    .MemSwap(&self.ids, &vec![]);\n    self;\n}\na: usize\nb: bool\nc: Option<u64>\nl: Option<&u64>\nm: Vec<u64>\nm2: Vec<?>\nn: u64\ne: u64\ns: str\nmt: Vec<u64>"
                    );
                    assert_eq!(errors(&m, "Root", "std"), Vec::<String>::new());
                }

                #[test]
                fn run_should_type_the_elements_of_an_array_argument_by_the_lists_the_target_converts_from()
                 {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root {
                            pub w: W,
                        }
                        pub struct V;
                        pub struct W(u8);
                        pub struct S;
                        impl From<u8> for W {
                            fn from(v: u8) -> Self {
                                W(v)
                            }
                        }
                        impl From<Vec<W>> for V {
                            fn from(v: Vec<W>) -> Self {
                                V
                            }
                        }
                        impl From<&[W]> for V {
                            fn from(v: &[W]) -> Self {
                                V
                            }
                        }
                        impl<X: Clone> From<Vec<X>> for V {
                            fn from(v: Vec<X>) -> Self {
                                V
                            }
                        }
                        impl From<u8> for V {
                            fn from(v: u8) -> Self {
                                V
                            }
                        }
                        impl From<Vec<u8>> for S {
                            fn from(v: Vec<u8>) -> Self {
                                S
                            }
                        }
                        impl From<Vec<Vec<u8>>> for S {
                            fn from(v: Vec<Vec<u8>>) -> Self {
                                S
                            }
                        }
                        impl Root {
                            pub fn take<T: Into<V>>(self, t: T) -> Root {
                                self
                            }
                            pub fn other<T: Into<S>>(self, t: T) -> Root {
                                self
                            }
                            pub fn plain(self, t: Vec<u8>) -> Root {
                                self
                            }
                            pub fn bound<T: Clone>(self, t: T) -> Root {
                                self
                            }
                            pub fn list<T: Into<Vec<u8>>>(self, t: T) -> Root {
                                self
                            }
                            pub fn single<T: Into<W>>(self, t: T) -> Root {
                                self
                            }
                            pub fn calls(self) -> Root {
                                self.take([1.into()]).other([1]).plain(vec![1]).bound([1]).list([1]).single([1]).take(1)
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );

                    Resolve.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        method(&m, "Root", "calls"),
                        "fn calls(self) -> Root {\n    self.take(vec![1.into::<W>()]).other(vec![1]).plain(vec![1]).bound(vec![1]).list(vec![1]).single(vec![1]).take(1);\n}"
                    );
                    assert_eq!(errors(&m, "Root", "calls"), Vec::<String>::new());
                }

                #[test]
                fn run_should_resolve_the_conversions() {
                    let (m, _) = resolve(|_| {});

                    assert_eq!(
                        method(&m, "Root", "conversions"),
                        "fn conversions<T: Tr, I: Into<V>>(mut self, t: T, i: I) -> Root {\n    let v1: V = i.into::<V>();\n    let v2: V = 1.into::<V>();\n    let v3: ? = /* unsupported: unknown function `V::from` */;\n    self.v = 1.into::<V>();\n    let v4: ? = /* unsupported: `into` without a target */;\n    let v5: ? = /* unsupported: no target of `.into()` on Param(\"T\") */;\n    self;\n}\nv1: V\nv2: V\nv3: ?\nv4: ?\nv5: ?"
                    );
                    assert_eq!(
                        errors(&m, "Root", "conversions"),
                        vec![
                            "unknown function `V::from`",
                            "`into` without a target",
                            "no target of `.into()` on Param(\"T\")"
                        ]
                    );
                }

                #[test]
                fn run_should_resolve_the_primitive_conversions_the_defaults_and_the_function_paths()
                 {
                    let (m, _) = resolve(|_| {});

                    assert_eq!(
                        method(&m, "Root", "rest"),
                        "fn rest(mut self) -> Root {\n    let p: u64 = (1 as u64);\n    self.v = V(0);\n    let dv: V = V(0);\n    let un: ? = /* unsupported: unknown function `default::Default::default` */;\n    let g: u64 = self.q.get((self.t.0 as u64));\n    self.ids = self.ids.Map(|x| {\n    *x;\n});\n    let qs: Vec<?> = self.ids.Map(|x| {\n    Q::new(x);\n});\n    let ms: Vec<?> = self.ids.Map(|x| {\n    let y: u64 = *x;\n    y;\n});\n    self;\n}\np: u64\ndv: V\nun: ?\ng: u64\nqs: Vec<?>\nms: Vec<?>"
                    );
                    assert_eq!(
                        errors(&m, "Root", "rest"),
                        ["unknown function `default::Default::default`"]
                    );
                    assert_eq!(
                        m.defaults
                            .iter()
                            .map(|(n, d)| format!("{n}: {} {}", print::expr(d), print::ty(&d.ty)))
                            .collect::<Vec<_>>(),
                        ["V: V(0) V"]
                    );
                }

                #[test]
                fn run_should_resolve_the_calls() {
                    let (m, _) = resolve(|_| {});

                    assert_eq!(
                        method(&m, "Root", "calls"),
                        "fn calls<T: Tr, I: Into<V>>(mut self, t: T, i: I) -> Root {\n    let tk: Vec<u64> = <Tr>::keys(T, );\n    let ti: u64 = <Tr>::id(t, );\n    let tw: u64 = <Tr>::with(T, 1);\n    let ta: u64 = <Tr>::add(t, 1);\n    let g: u64 = self.q.get(0);\n    self.q = Q::new(1);\n    let h: Q = Q::new(1);\n    let x: u8 = self.e.f();\n    let vv: V = V(1);\n    /* unsupported: unknown function `unknown` */;\n    /* unsupported: unknown method `nothing` on Named(\"Root\", []) */;\n    /* unsupported: unknown method `nothing` on Param(\"I\") */;\n    self;\n}\ntk: Vec<u64>\nti: u64\ntw: u64\nta: u64\ng: u64\nh: Q\nx: u8\nvv: V"
                    );
                    assert_eq!(
                        errors(&m, "Root", "calls"),
                        vec![
                            "unknown function `unknown`",
                            "unknown method `nothing` on Named(\"Root\", [])",
                            "unknown method `nothing` on Param(\"I\")"
                        ]
                    );
                }

                #[test]
                fn run_should_type_the_values() {
                    let (m, _) = resolve(|_| {});

                    assert_eq!(
                        method(&m, "Root", "values"),
                        "fn values(mut self) -> Root {\n    let k: u64 = 1;\n    let none: Option<?> = Option::None;\n    let some: Option<i32> = Option::Some(1);\n    self.o = Option::Some(2);\n    self.o = Option::None;\n    let eb: E = E::B(1);\n    let ea: E = E::A;\n    let tup: (i32, bool) = (1, true);\n    self.t = (1, true);\n    self.ids = vec![1];\n    let arr: Vec<bool> = vec![true];\n    let f0: u8 = self.t.0;\n    let f9: ? = self.t.9;\n    let idx: u64 = self.ids[0];\n    let nidx: ? = self.o[0];\n    let neg: bool = !true;\n    let r: &Vec<u64> = &self.ids;\n    let d: Vec<u64> = *r;\n    let cmp: bool = (1 == 2);\n    let add: i32 = (1 + 2);\n    let fl: f64 = 1.5;\n    let st: str = \"s\";\n    let w: Wrap<Q<i32>> = Wrap { inner: Q { v: 1 } };\n    let w2: Wrap2<i32> = Wrap2 { n: 1, q: Q { v: 1 } };\n    let w3: Wrap2<T> = Wrap2 { q: /* unsupported: unknown path `nothing` */ };\n    let sf: Root = Root { o: Option::None };\n    let p: ? = /* unsupported: unknown path `nothing` */;\n    let pp: ? = /* unsupported: unknown path `nothing::path` */;\n    let fp: ? = /* unsupported: unknown path `nothing` */.field;\n    let u;\n    panic!(\"p\");\n    return self;\n}\nk: u64\nnone: Option<?>\nsome: Option<i32>\neb: E\nea: E\ntup: (i32, bool)\narr: Vec<bool>\nf0: u8\nf9: ?\nidx: u64\nnidx: ?\nneg: bool\nr: &Vec<u64>\nd: Vec<u64>\ncmp: bool\nadd: i32\nfl: f64\nst: str\nw: Wrap<Q<i32>>\nw2: Wrap2<i32>\nw3: Wrap2<T>\nsf: Root\np: ?\npp: ?\nfp: ?"
                    );
                    assert_eq!(
                        errors(&m, "Root", "values"),
                        vec![
                            "unknown path `nothing`",
                            "unknown path `nothing`",
                            "unknown path `nothing::path`",
                            "unknown path `nothing`"
                        ]
                    );
                }

                #[test]
                fn run_should_bind_the_patterns() {
                    let (m, _) = resolve(|_| {});

                    assert_eq!(
                        method(&m, "Root", "control"),
                        "fn control(self) -> Root {\n    if let Option::Some(x) = self.o {\n        let y: u64 = x;\n    } else {\n        let z: i32 = 1;\n    };\n    if true {\n    } else {\n    };\n    if true {\n    };\n    match self.e {\n        E::A => {\n        },\n        E::B(x) => {\n            let y: u64 = x;\n        },\n        E::C(p, q) => {\n        },\n    };\n    match self.o {\n        Option::Some(x) => {\n        },\n        Option::None => {\n        },\n    };\n    match self.ids.Len() {\n        1 => {\n        },\n        _ => {\n        },\n    };\n    if let Q { v: v } = self.q {\n        let y: T = v;\n    };\n    match self.missing {\n        E::A => {\n        },\n    };\n    while false {\n    };\n    self;\n}"
                    );
                    assert_eq!(errors(&m, "Root", "control"), Vec::<String>::new());
                }

                #[test]
                fn run_should_resolve_from_into_a_conversion_and_keep_its_errors() {
                    let (m, errors) = resolve(|m| {
                        m.conversions = vec![
                            conversion(
                                "V",
                                Ty::Int("u64"),
                                Expr::new(K::Construct {
                                    class: "V".into(),
                                    fields: vec![("0".into(), Expr::new(K::Local("v".into())))],
                                }),
                            ),
                            conversion(
                                "E",
                                Ty::Int("u64"),
                                Expr::new(K::Path(vec!["nothing".into()])),
                            ),
                        ];
                    });

                    assert_eq!(
                        method(&m, "Root", "conversions"),
                        "fn conversions<T: Tr, I: Into<V>>(mut self, t: T, i: I) -> Root {\n    let v1: V = i.into::<V>();\n    let v2: V = 1.into::<V>();\n    let v3: V = 1.into::<V>();\n    self.v = 1.into::<V>();\n    let v4: ? = /* unsupported: `into` without a target */;\n    let v5: ? = /* unsupported: no target of `.into()` on Param(\"T\") */;\n    self;\n}\nv1: V\nv2: V\nv3: V\nv4: ?\nv5: ?"
                    );
                    assert_eq!(
                        m.conversions
                            .iter()
                            .map(|c| print::expr(&c.body))
                            .collect::<Vec<_>>(),
                        vec!["V(v)", "/* unsupported: unknown path `nothing` */"]
                    );
                    assert_eq!(errors, Vec::<String>::new());
                    assert_eq!(
                        m.conversions
                            .iter()
                            .map(|c| c.errors.clone())
                            .collect::<Vec<_>>(),
                        vec![Vec::<String>::new(), vec!["unknown path `nothing`".into()]]
                    );
                }

                #[test]
                fn run_should_type_the_resolved_forms_of_the_functions_of_the_module() {
                    let local = |name: &str| Expr::new(K::Local(name.into()));
                    let body = vec![
                        Expr::new(K::Let {
                            name: "variant".into(),
                            value: Some(
                                Expr::variant("E", "B", Some(Expr::new(K::Lit(Lit::Int(1)))))
                                    .boxed(),
                            ),
                        }),
                        Expr::new(K::Let {
                            name: "unit".into(),
                            value: Some(Expr::variant("E", "A", None).boxed()),
                        }),
                        Expr::new(K::Let {
                            name: "convert".into(),
                            value: Some(
                                Expr::new(K::Convert {
                                    value: local("x").boxed(),
                                    to: "V".into(),
                                })
                                .boxed(),
                            ),
                        }),
                        Expr::new(K::Let {
                            name: "take".into(),
                            value: Some(
                                Expr::new(K::Call {
                                    callee: Callee::Intrinsic(Intrinsic::MemTake),
                                    recv: None,
                                    args: vec![local("x")],
                                })
                                .boxed(),
                            ),
                        }),
                        Expr::new(K::Let {
                            name: "map".into(),
                            value: Some(
                                Expr::new(K::Call {
                                    callee: Callee::Intrinsic(Intrinsic::Map),
                                    recv: Some(local("ids").boxed()),
                                    args: vec![Expr::new(K::Closure {
                                        params: vec!["i".into()],
                                        body: vec![Expr::new(K::Return(Some(
                                            Expr::new(K::Lit(Lit::Bool(true))).boxed(),
                                        )))],
                                    })],
                                })
                                .boxed(),
                            ),
                        }),
                        Expr::new(K::Call {
                            callee: Callee::Intrinsic(Intrinsic::Map),
                            recv: Some(local("x").boxed()),
                            args: vec![local("x")],
                        }),
                        Expr::new(K::Call {
                            callee: Callee::Extern("ext".into()),
                            recv: None,
                            args: vec![Expr::new(K::Lit(Lit::Unit))],
                        }),
                        Expr::new(K::Ident("ident".into())),
                        Expr::new(K::Local("undeclared".into())),
                        Expr::new(K::Closure {
                            params: Vec::new(),
                            body: Vec::new(),
                        }),
                        Expr::new(K::Unwrap(local("x").boxed())),
                        Expr::new(K::Return(None)),
                    ];
                    let (m, _) = resolve(|m| {
                        m.functions.push(Func::new(
                            "f",
                            vec![
                                ("x".into(), Ty::Int("u64")),
                                ("ids".into(), Ty::List(Box::new(Ty::Int("u64")))),
                            ],
                            Ty::Unit,
                            body,
                        ))
                    });

                    assert_eq!(
                        typed(&m.functions[0]),
                        "fn f(x: u64, ids: Vec<u64>) -> () {\n    let variant: E = E::B(1);\n    let unit: E = E::A;\n    let convert: V = x.into::<V>();\n    let take: ? = .MemTake(x);\n    let map: Vec<bool> = ids.Map(|i| {\n    return true;\n});\n    x.Map(x);\n    extern ext(());\n    ident/*unresolved*/;\n    undeclared;\n    || {\n    };\n    x@Some;\n    return ;\n}\nvariant: E\nunit: E\nconvert: V\ntake: ?\nmap: Vec<bool>"
                    );
                }

                #[test]
                fn run_should_resolve_the_methods_of_data_enums_and_trait_impls() {
                    let (m, _) = resolve(|_| {});

                    assert_eq!(
                        method(&m, "Q", "get"),
                        "fn get(&self, i: u64) -> T {\n    self.v;\n}"
                    );
                    assert_eq!(
                        method(&m, "Q", "new"),
                        "fn new(v: T) -> Q<T> {\n    Q { v: v };\n}"
                    );
                    assert_eq!(
                        method(&m, "E", "f"),
                        "fn f(&self) -> u8 {\n    match self {\n        E::A => {\n            1;\n        },\n        E::B(v) => {\n            2;\n        },\n        E::C(a, b) => {\n            3;\n        },\n    };\n}"
                    );
                    assert_eq!(method(&m, "E", "b"), "fn b() -> E {\n    E::B(1);\n}");
                    assert_eq!(method(&m, "E", "new"), "fn new() -> E {\n    E::b();\n}");
                    assert_eq!(method(&m, "E", "a"), "fn a() -> E {\n    E::A;\n}");
                    assert_eq!(
                        method(&m, "V", "id"),
                        "fn id(&self) -> u64 {\n    self.0;\n}"
                    );
                }

                #[test]
                fn run_should_resolve_the_client_calls_through_the_supertraits_and_type_the_results()
                 {
                    let (mut m, _) = client(CLIENT);
                    Resolve.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        [
                            method(&m, "Api", "kind"),
                            method(&m, "Api", "login"),
                            method(&m, "Api", "fail"),
                            typed(&m.traits["Http"][1]),
                        ],
                        [
                            "async fn kind(&self, kind: Kind, n: u64) -> Result<u16, ApiError> {\n    <Http>::get::<()>(self.client, &self.url(&format!(\"/\", kind.to_string(), \"/\", format!(n)))).await.MapOk(|tuple1| {\n    let status: u16 = tuple1.0;\n    status;\n});\n}",
                            "async fn login(&mut self) -> Result<u16, ApiError> {\n    let tuple1: (u16, str) = <Http>::get(self.client, &self.url(\"/login\")).await?;\n    let status: u16 = tuple1.0;\n    let token: str = tuple1.1;\n    self.token = Option::Some(token);\n    Result::Ok(status);\n}\ntuple1: (u16, str)\nstatus: u16\ntoken: str",
                            "fn fail(&self) -> Result<u16, ApiError> {\n    Result::Err(ApiError { status: 1, description: \"fail\" });\n}",
                            "async fn status(&self, uri: &str) -> Result<u16, ApiError> {\n    Result::Ok(<Http>::get::<()>(self, uri).await?.0);\n}",
                        ]
                    );
                }

                #[test]
                fn run_should_format_the_primitives_and_the_types_with_a_display_impl() {
                    let (mut m, _) = client(CLIENT);
                    Resolve.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        [
                            method(&m, "Api", "new"),
                            method(&m, "ApiError", "to_string")
                        ],
                        [
                            "fn new(client: T, address: &str) -> Api<T> {\n    let base: () = if address.StartsWith(\"http\") {\n    address;\n} else {\n    format!(\"http://\", address, \"/api\");\n};\n    Api { client: client, base: base, token: Option::None };\n}\nbase: ()",
                            "fn to_string(&self) -> str {\n    format!(self.status, \": {\", self.description, \"}\");\n}",
                        ]
                    );
                }

                #[test]
                fn run_should_resolve_the_option_result_list_and_string_methods_of_tests() {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root { pub ids: Vec<u64>, pub o: Option<u64> }
                        impl Root {
                            pub fn checks(&mut self, r: Result<u64, bool>, s: String) -> u64 {
                                let a = self.o.unwrap();
                                let b = self.o.is_some();
                                let c = self.o.is_none();
                                let d = self.o.map(|x| x + 1);
                                let e = r.unwrap();
                                let f = r.expect("e");
                                let g = r.unwrap_err();
                                let h = self.ids.iter().find(|x| **x == 1);
                                let i = self.ids.iter().any(|x| *x == 1);
                                let j = self.ids.iter().all(|x| *x == 1);
                                self.ids.sort_by_key(|x| *x);
                                let k = self.ids.remove(0);
                                let l = s.repeat(2);
                                let n = s.is_empty();
                                assert_ne!(a, 1);
                                a
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    Resolve.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        method(&m, "Root", "checks"),
                        "fn checks(&mut self, r: Result<u64, bool>, s: str) -> u64 {\n    let a: u64 = self.o.Unwrap();\n    let b: bool = self.o.IsSome();\n    let c: bool = self.o.IsNone();\n    let d: Option<?> = self.o.MapSome(|x| {\n    (x + 1);\n});\n    let e: u64 = r.Unwrap();\n    let f: u64 = r.Unwrap(\"e\");\n    let g: bool = r.UnwrapErr();\n    let h: Option<u64> = self.ids.Find(|x| {\n    (**x == 1);\n});\n    let i: bool = self.ids.Any(|x| {\n    (*x == 1);\n});\n    let j: bool = self.ids.All(|x| {\n    (*x == 1);\n});\n    self.ids.SortByKey(|x| {\n    *x;\n});\n    let k: u64 = self.ids.Remove(0);\n    let l: str = s.Repeat(2);\n    let n: bool = s.IsEmpty();\n    .AssertNe(a, 1);\n    a;\n}\na: u64\nb: bool\nc: bool\nd: Option<?>\ne: u64\nf: u64\ng: bool\nh: Option<u64>\ni: bool\nj: bool\nk: u64\nl: str\nn: bool"
                    );
                    assert_eq!(errors(&m, "Root", "checks"), Vec::<String>::new());
                }

                #[test]
                fn run_should_type_the_calls_of_the_externs_and_the_methods_of_the_opaque_classes()
                {
                    let (mut m, _) = suite(
                        "pub struct Root;\nimpl Root { pub fn new() -> Root { Root } }",
                        SUITE,
                    );

                    Resolve.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        typed(&m.tests[1]),
                        "async fn login::logs_in() -> () {\n    let server: Server = Server::new().await;\n    let status: u16 = server.api.login().await.Unwrap();\n    let name: str = extern next_name(\"user\");\n    let agent: str = Helpers::agent();\n    let pair: ? = /* unsupported: unknown function `Pair` */;\n    let port: ? = /* unsupported: unknown path `PORT` */;\n    .AssertNe(status, 0);\n}\nserver: Server\nstatus: u16\nname: str\nagent: str\npair: ?\nport: ?"
                    );
                }

                #[test]
                fn run_should_type_the_results_by_the_expected_type() {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root;
                        impl Root {
                            pub fn f(&self, o: Option<u8>) -> Option<u8> {
                                let any = Ok(1);
                                let err: Result<u8, bool> = Err(true);
                                let v = o?;
                                o
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    Resolve.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        method(&m, "Root", "f"),
                        "fn f(&self, o: Option<u8>) -> Option<u8> {\n    let any: Result<?, ?> = Result::Ok(1);\n    let err: Result<u8, bool> = Result::Err(true);\n    let v: ? = o?;\n    o;\n}\nany: Result<?, ?>\nerr: Result<u8, bool>\nv: ?"
                    );
                }

                #[test]
                fn run_should_report_a_formatted_type_without_a_display_impl() {
                    let (mut m, _) = lower(
                        r#"
                        pub struct Root;
                        pub struct S;
                        impl Root {
                            pub fn f(&self, s: S) -> String {
                                format!("{}", s)
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    Resolve.run(&mut m, &mut Vec::new());

                    assert_eq!(errors(&m, "Root", "f"), ["no `Display` of `S`"]);
                }
            }
        }
    }

    mod subst {
        use super::*;

        #[test]
        fn subst_should_replace_self_and_the_parameters_within_the_type() {
            let b = |t| Box::new(t);
            let t = Ty::Tuple(vec![
                Ty::SelfTy,
                Ty::List(b(Ty::Param("T".into()))),
                Ty::Option(b(Ty::Param("U".into()))),
                Ty::Ref(b(Ty::Named("A".into(), vec![Ty::Param("T".into())]))),
                Ty::Result(b(Ty::Param("T".into())), b(Ty::Param("U".into()))),
                Ty::Bool,
            ]);

            assert_eq!(
                print::ty(&subst(&t, &Ty::named("S"), &[("T".into(), Ty::Str)])),
                "(S, Vec<str>, Option<U>, &A<str>, Result<str, U>, bool)"
            );
        }
    }

    mod returned_ty {
        use super::*;

        #[test]
        fn returned_ty_should_be_the_type_of_the_first_return_also_in_branches() {
            let ret = |ty| Expr::new(K::Return(Some(Expr::typed(K::This, ty).boxed())));
            let if_ = |then, els| {
                Expr::new(K::If {
                    cond: Expr::new(K::This).boxed(),
                    then,
                    els,
                })
            };
            let if_let = |then, els| {
                Expr::new(K::IfLet {
                    pat: Pat::Wild,
                    value: Expr::new(K::This).boxed(),
                    then,
                    els,
                })
            };

            assert_eq!(
                [
                    vec![Expr::new(K::This), ret(Ty::Bool), ret(Ty::Str)],
                    vec![if_(vec![], None), if_(vec![ret(Ty::Bool)], None)],
                    vec![if_let(vec![], Some(vec![ret(Ty::Str)]))],
                    vec![Expr::new(K::Return(None))],
                ]
                .iter()
                .map(|body| returned_ty(body).map(|t| print::ty(&t)))
                .collect::<Vec<_>>(),
                [Some("bool"), Some("bool"), Some("str"), None].map(|t| t.map(str::to_owned))
            );
        }
    }
}
