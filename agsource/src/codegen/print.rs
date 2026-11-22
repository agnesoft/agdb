//! The lowered IR as Rust-like text, for the dumps after each pass.

use super::lowered::*;

pub fn ty(t: &Ty) -> String {
    match t {
        Ty::Unit => "()".into(),
        Ty::Bool => "bool".into(),
        Ty::Int(name) => (*name).into(),
        Ty::Float => "f64".into(),
        Ty::Str => "str".into(),
        Ty::List(t) => format!("Vec<{}>", ty(t)),
        Ty::Option(t) => format!("Option<{}>", ty(t)),
        Ty::Result(ok, err) => format!("Result<{}, {}>", ty(ok), ty(err)),
        Ty::Tuple(v) => format!("({})", join(v.iter().map(ty))),
        Ty::Named(name, args) if args.is_empty() => name.clone(),
        Ty::Named(name, args) => format!("{name}<{}>", join(args.iter().map(ty))),
        Ty::Param(name) => name.clone(),
        Ty::SelfTy => "Self".into(),
        Ty::Ref(t) => format!("&{}", ty(t)),
        Ty::Into(name) => format!("impl Into<{name}>"),
        Ty::Dict(name, of) => format!("dyn {name}<{}>", ty(of)),
        Ty::Fn => "fn".into(),
        Ty::Unknown => "?".into(),
    }
}

pub fn pat(p: &Pat) -> String {
    match p {
        Pat::Wild => "_".into(),
        Pat::Bind(name) => name.clone(),
        Pat::Lit(l) => lit(l),
        Pat::Variant { en, var, inner } => {
            let path = if en.is_empty() {
                var.clone()
            } else {
                format!("{en}::{var}")
            };

            if inner.is_empty() {
                path
            } else {
                format!("{path}({})", join(inner.iter().map(pat)))
            }
        }
        Pat::Struct { name, fields } => format!(
            "{name} {{ {} }}",
            join(fields.iter().map(|(n, p)| format!("{n}: {}", pat(p))))
        ),
    }
}

fn lit(l: &Lit) -> String {
    match l {
        Lit::Bool(b) => b.to_string(),
        Lit::Int(i) => i.to_string(),
        Lit::Float(f) => format!("{f:?}"),
        Lit::Str(s) => format!("{s:?}"),
        Lit::Unit => "()".into(),
    }
}

fn join(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

fn list(v: &[Expr]) -> String {
    join(v.iter().map(expr))
}

fn block(v: &[Expr], indent: usize) -> String {
    let pad = "    ".repeat(indent + 1);
    let mut s = "{\n".to_string();

    for e in v {
        s += &format!("{pad}{};\n", stmt(e, indent + 1));
    }

    s + &"    ".repeat(indent) + "}"
}

fn els(e: &Option<Vec<Expr>>, indent: usize) -> String {
    e.as_ref()
        .map(|e| format!(" else {}", block(e, indent)))
        .unwrap_or_default()
}

fn stmt(e: &Expr, indent: usize) -> String {
    match &e.k {
        K::If { cond, then, els: e } => format!(
            "if {} {}{}",
            expr(cond),
            block(then, indent),
            els(e, indent)
        ),
        K::IfLet {
            pat: p,
            value,
            then,
            els: e,
        } => format!(
            "if let {} = {} {}{}",
            pat(p),
            expr(value),
            block(then, indent),
            els(e, indent)
        ),
        K::Match { value, arms } => {
            let pad = "    ".repeat(indent + 1);
            let mut s = format!("match {} {{\n", expr(value));

            for (p, body) in arms {
                s += &format!("{pad}{} => {},\n", pat(p), block(body, indent + 1));
            }

            s + &"    ".repeat(indent) + "}"
        }
        K::While { cond, body } => format!("while {} {}", expr(cond), block(body, indent)),
        K::Closure { params, body } => format!("|{}| {}", params.join(", "), block(body, indent)),
        _ => expr(e),
    }
}

pub fn expr(e: &Expr) -> String {
    match &e.k {
        K::Ident(name) => format!("{name}/*unresolved*/"),
        K::Local(name) => name.clone(),
        K::This => "self".into(),
        K::Lit(l) => lit(l),
        K::Array(v) => format!("vec![{}]", list(v)),
        K::Tuple(v) => format!("({})", list(v)),
        K::Construct { class, fields }
            if !fields.is_empty() && fields.iter().all(|(n, _)| n.parse::<usize>().is_ok()) =>
        {
            format!("{class}({})", join(fields.iter().map(|(_, e)| expr(e))))
        }
        K::Construct { class, fields } => format!(
            "{class} {{ {} }}",
            join(fields.iter().map(|(n, e)| format!("{n}: {}", expr(e))))
        ),
        K::Variant {
            en,
            var,
            payload: Some(p),
        } => format!("{en}::{var}({})", expr(p)),
        K::Variant { en, var, .. } => format!("{en}::{var}"),
        K::Path(segments) => segments.join("::"),
        K::Field { base, name } => format!("{}.{name}", expr(base)),
        K::Payload { base, var, .. } => format!("{}@{var}", expr(base)),
        K::Unwrap(base) => format!("{}@Some", expr(base)),
        K::Index { base, index } => format!("{}[{}]", expr(base), expr(index)),
        K::Call { callee, recv, args } => call(callee, recv.as_deref(), args),
        K::Convert { value, to } => format!("{}.into::<{to}>()", expr(value)),
        K::TypeCase { value, arms } => format!(
            "typecase {} {{ {} }}",
            expr(value),
            join(
                arms.iter()
                    .map(|(t, p, e)| format!("{}@{} => {}", ty(t), pat(p), expr(e)))
            )
        ),
        K::Assign { place, value } => format!("{} = {}", expr(place), expr(value)),
        K::Let { name, value } => format!(
            "let {name}{}",
            value
                .as_ref()
                .map(|v| format!(": {} = {}", ty(&v.ty), expr(v)))
                .unwrap_or_default()
        ),
        K::Return(value) => format!(
            "return {}",
            value.as_ref().map(|v| expr(v)).unwrap_or_default()
        ),
        K::Binary { op, l, r } => format!("({} {op} {})", expr(l), expr(r)),
        K::Not(e) => format!("!{}", expr(e)),
        K::Ref(e) => format!("&{}", expr(e)),
        K::Deref(e) => format!("*{}", expr(e)),
        K::Cast(v) => format!("({} as {})", expr(v), ty(&e.ty)),
        K::Fields(v) => format!("fields::<{}>({})", ty(&e.ty), expr(v)),
        K::Await(v) => format!("{}.await", expr(v)),
        K::Try(v) => format!("{}?", expr(v)),
        K::Format(pieces) => format!("format!({})", list(pieces)),
        K::Throw(v) => format!("throw {}", expr(v)),
        K::Panic(message) => format!("panic!({message:?})"),
        K::Unsupported(construct) => format!("/* unsupported: {construct} */"),
        K::Match { .. } | K::If { .. } | K::IfLet { .. } | K::While { .. } | K::Closure { .. } => {
            stmt(e, 0)
        }
    }
}

fn call(callee: &Callee, recv: Option<&Expr>, args: &[Expr]) -> String {
    let a = list(args);
    let r = recv.map(expr).unwrap_or_default();

    match callee {
        Callee::Path(path, _) => format!("{}({a})/*unresolved*/", path.join("::")),
        Callee::Method(name, types) => format!("{r}.{name}{}({a})/*unresolved*/", turbofish(types)),
        Callee::Member { name, .. } => format!("{r}.{name}({a})"),
        Callee::Static { class, name } => format!("{class}::{name}({a})"),
        Callee::Trait {
            trait_,
            name,
            types,
        } => {
            let r = recv.map(|_| format!("{r}, ")).unwrap_or_default();
            format!("<{trait_}>::{name}{}({r}{a})", turbofish(types))
        }
        Callee::Intrinsic(i) => format!("{r}.{i:?}({a})"),
        Callee::Extern(name) => format!("extern {name}({a})"),
    }
}

fn turbofish(types: &[Ty]) -> String {
    if types.is_empty() {
        String::new()
    } else {
        format!("::<{}>", join(types.iter().map(ty)))
    }
}

pub fn func(f: &Func) -> String {
    let recv = f.recv.map(|r| match r {
        Recv::Value => "self".to_string(),
        Recv::ValueMut => "mut self".to_string(),
        Recv::Ref => "&self".to_string(),
        Recv::RefMut => "&mut self".to_string(),
    });
    let params = join(
        recv.into_iter()
            .chain(f.params.iter().map(|(n, t)| format!("{n}: {}", ty(t)))),
    );
    let generics = if f.generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", join(f.generics.iter().map(generic)))
    };

    format!(
        "{}fn {}{generics}({params}) -> {} {}",
        if f.is_async { "async " } else { "" },
        f.name,
        ty(&f.ret),
        block(&f.body, 0)
    )
}

fn generic(g: &Generic) -> String {
    let bounds: Vec<String> = g
        .bounds
        .iter()
        .map(|(t, arg)| {
            arg.as_ref()
                .map(|a| format!("{t}<{}>", ty(a)))
                .unwrap_or(t.clone())
        })
        .collect();

    if bounds.is_empty() {
        g.name.clone()
    } else {
        format!("{}: {}", g.name, bounds.join(" + "))
    }
}

/// The dump of the module after `title`: the methods of the builders, the
/// trait impls of the classes, the functions of the module and the tests.
pub fn module(m: &Module, title: &str) -> String {
    let mut out = format!("// ===== after {title}\n");

    for b in &m.builders {
        for f in &m.classes[b].methods {
            out += &format!("// {b}\n{}\n\n", func(f));
        }
    }

    for (name, c) in &m.classes {
        for (trait_, funcs) in &c.trait_impls {
            for f in funcs {
                out += &format!("// impl {trait_} for {name}\n{}\n\n", func(f));
            }
        }
    }

    for f in &m.functions {
        out += &format!("{}\n\n", func(f));
    }

    for f in &m.tests {
        let ignore = f.ignore.as_ref().map(|r| format!(" ignored {r:?}"));
        out += &format!("// test{}\n{}\n\n", ignore.unwrap_or_default(), func(f));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ty {
        use super::*;

        #[test]
        fn ty_should_print_every_type() {
            let b = |t| Box::new(t);
            let types = [
                Ty::Unit,
                Ty::Bool,
                Ty::Int("u8"),
                Ty::Float,
                Ty::Str,
                Ty::List(b(Ty::Bool)),
                Ty::Option(b(Ty::Bool)),
                Ty::Tuple(vec![Ty::Bool, Ty::Unit]),
                Ty::named("A"),
                Ty::Named("A".into(), vec![Ty::Bool]),
                Ty::Param("T".into()),
                Ty::SelfTy,
                Ty::Ref(b(Ty::Bool)),
                Ty::Into("A".into()),
                Ty::Dict("Tr".into(), b(Ty::Bool)),
                Ty::Fn,
                Ty::Unknown,
            ];

            assert_eq!(
                types.iter().map(ty).collect::<Vec<_>>(),
                vec![
                    "()",
                    "bool",
                    "u8",
                    "f64",
                    "str",
                    "Vec<bool>",
                    "Option<bool>",
                    "(bool, ())",
                    "A",
                    "A<bool>",
                    "T",
                    "Self",
                    "&bool",
                    "impl Into<A>",
                    "dyn Tr<bool>",
                    "fn",
                    "?"
                ]
            );
        }
    }

    mod expr {
        use super::*;

        #[test]
        fn expr_should_print_the_forms_the_passes_make() {
            let this = || Expr::new(K::This).boxed();
            let exprs = [
                K::Payload {
                    base: this(),
                    en: "E".into(),
                    var: "A".into(),
                },
                K::TypeCase {
                    value: this(),
                    arms: vec![
                        (
                            Ty::Bool,
                            Pat::Bind("b".into()),
                            Expr::new(K::Lit(Lit::Bool(true))),
                        ),
                        (Ty::Str, Pat::Wild, Expr::new(K::Lit(Lit::Str("s".into())))),
                    ],
                },
                K::Call {
                    callee: Callee::Trait {
                        trait_: "Tr".into(),
                        name: "f".into(),
                        types: Vec::new(),
                    },
                    recv: None,
                    args: Vec::new(),
                },
                K::Call {
                    callee: Callee::Path(vec!["a".into(), "f".into()], Vec::new()),
                    recv: None,
                    args: Vec::new(),
                },
                K::Cast(this()),
                K::Fields(this()),
            ];

            assert_eq!(
                exprs
                    .into_iter()
                    .map(|k| expr(&Expr::new(k)))
                    .collect::<Vec<_>>(),
                vec![
                    "self@A",
                    "typecase self { bool@b => true, str@_ => \"s\" }",
                    "<Tr>::f()",
                    "a::f()/*unresolved*/",
                    "(self as ?)",
                    "fields::<?>(self)"
                ]
            );
        }
    }

    mod module {
        use super::*;

        #[test]
        fn module_should_dump_the_functions_of_the_module() {
            let mut f = Func::new("f", Vec::new(), Ty::Unit, Vec::new());
            f.generics.push(Generic {
                name: "T".into(),
                bounds: Vec::new(),
            });
            let m = Module {
                functions: vec![f],
                ..Default::default()
            };

            assert_eq!(
                module(&m, "pass"),
                "// ===== after pass\nfn f<T>() -> () {\n}\n\n"
            );
        }
    }
}
