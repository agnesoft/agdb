use super::Pass;
use crate::Error;
use crate::codegen::lowered::*;

/// References, dereferences, `mem::take`/`mem::swap` and assignments
/// through pattern bindings erased for targets that share every object by
/// reference: the borrow rules of Rust guarantee that no alias observes the
/// difference. The default of `unwrap_or_default()` becomes its argument.
pub struct EraseOwnership;

impl Pass for EraseOwnership {
    fn name(&self) -> &str {
        "erase_ownership"
    }

    fn run(&self, m: &mut Module, _errors: &mut Vec<Error>) {
        let defaults = Module {
            defaults: m.defaults.clone(),
            ..Module::default()
        };
        let mut tmp = 0;

        m.for_each_func(&mut |f, _| {
            f.recv = f.recv.map(|_| Recv::Value);
            f.body = stmts(std::mem::take(&mut f.body), &mut tmp, &defaults);
            rewrite_body(&mut f.body, &mut |e| match &mut e.k {
                K::Ref(v) | K::Deref(v) => *e = std::mem::replace(v.as_mut(), Expr::new(K::This)),
                K::Call {
                    callee: Callee::Intrinsic(Intrinsic::UnwrapOrDefault),
                    args,
                    ..
                } if args.is_empty() => args.push(defaults.default_of(&e.ty)),
                _ => {}
            });
        });
        m.map_types(&|ty| match ty {
            Ty::Ref(inner) => (**inner).clone(),
            ty => ty.clone(),
        });
    }
}

fn stmt(k: K) -> Expr {
    Expr::typed(k, Ty::Unit)
}

fn stmts(body: Vec<Expr>, tmp: &mut usize, m: &Module) -> Vec<Expr> {
    let mut out = Vec::new();

    for mut s in body {
        match &mut s.k {
            K::If { then, els, .. } | K::IfLet { then, els, .. } => {
                *then = stmts(std::mem::take(then), tmp, m);

                if let Some(els) = els {
                    *els = stmts(std::mem::take(els), tmp, m);
                }
            }
            K::While { body, .. } => *body = stmts(std::mem::take(body), tmp, m),
            _ => {}
        }

        if let K::Let {
            name,
            value: Some(value),
        } = &s.k
            && let K::Call {
                callee: Callee::Intrinsic(Intrinsic::MemTake),
                args,
                ..
            } = &value.k
            && let [place] = args.as_slice()
        {
            let place = place.clone();
            let default = m.default_of(&place.ty);
            out.push(stmt(K::Let {
                name: name.clone(),
                value: Some(place.clone().boxed()),
            }));
            out.push(stmt(K::Assign {
                place: place.boxed(),
                value: default.boxed(),
            }));
            continue;
        }

        if let K::Call {
            callee: Callee::Intrinsic(Intrinsic::MemSwap),
            args,
            ..
        } = &s.k
            && let [a, b] = args.as_slice()
        {
            let (a, b) = (a.clone(), b.clone());
            *tmp += 1;
            let t = format!("tmp{tmp}");
            let ty = a.ty.clone();
            out.push(stmt(K::Let {
                name: t.clone(),
                value: Some(a.clone().boxed()),
            }));
            out.push(stmt(K::Assign {
                place: a.boxed(),
                value: b.clone().boxed(),
            }));
            out.push(stmt(K::Assign {
                place: b.boxed(),
                value: Expr::local(&t, ty).boxed(),
            }));
            continue;
        }

        match s.k {
            K::IfLet {
                pat,
                value,
                mut then,
                els,
            } => {
                let value = if is_place(&value) {
                    value
                } else {
                    *tmp += 1;
                    let name = format!("m{tmp}");
                    let ty = value.ty.clone();
                    out.push(stmt(K::Let {
                        name: name.clone(),
                        value: Some(value),
                    }));
                    Expr::local(&name, ty).boxed()
                };
                let places = pat.projections(&value).1;
                rewrite_body(&mut then, &mut |e| {
                    if let K::Assign { place, .. } = &mut e.k
                        && let K::Deref(inner) = &place.k
                        && let K::Local(n) = &inner.k
                        && let Some((_, p)) = places.iter().find(|(b, _)| b == n)
                    {
                        **place = p.clone();
                    }
                });
                out.push(stmt(K::IfLet {
                    pat,
                    value,
                    then,
                    els,
                }));
            }
            k => out.push(Expr { k, ty: s.ty }),
        }
    }

    out
}

/// A place the pattern bindings can be assigned through, a temporary
/// otherwise: a copy of a value type would not update the place.
fn is_place(e: &Expr) -> bool {
    match &e.k {
        K::Local(_) | K::This => true,
        K::Field { base, .. } | K::Ref(base) | K::Deref(base) => is_place(base),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::print;
    use crate::codegen::test_utilities::resolved;

    mod erase_ownership {
        use super::*;

        mod pass {
            use super::*;

            mod name {
                use super::*;

                #[test]
                fn name_should_be_erase_ownership() {
                    assert_eq!(EraseOwnership.name(), "erase_ownership");
                }
            }

            mod run {
                use super::*;

                #[test]
                fn run_should_erase_the_references_and_the_moves() {
                    let mut m = resolved(
                        r#"
                        pub struct Root {
                            pub v: Vec<u8>,
                            pub w: Vec<u8>,
                            pub o: Option<u8>,
                            pub d: D,
                        }
                        pub struct D {
                            pub n: u8,
                        }
                        impl Root {
                            pub fn take(&mut self) -> Vec<u8> {
                                let v = std::mem::take(&mut self.v);
                                v
                            }
                            pub fn swap(&mut self, c: bool) {
                                if c {
                                    std::mem::swap(&mut self.v, &mut self.w);
                                } else {
                                    while c {
                                        std::mem::swap(&mut self.v, &mut self.w);
                                    }
                                }
                            }
                            pub fn place(&mut self, c: bool) {
                                if c {
                                    if let Some(n) = &mut self.o {
                                        *n = 1;
                                        self.v = vec![];
                                    } else {
                                        let d = std::mem::take(&mut self.d);
                                    }
                                }
                                if let Some(n) = self.v.last_mut() {
                                    *n = 2;
                                }
                            }
                            pub fn local(&self, o: &Option<u8>) -> u8 {
                                if let Some(n) = o {
                                    *n
                                } else {
                                    o.unwrap_or_default()
                                }
                            }
                        }
                        "#,
                        &[],
                        &[],
                    );
                    m.defaults
                        .insert("D".into(), Expr::local("D_default", Ty::named("D")));

                    EraseOwnership.run(&mut m, &mut Vec::new());

                    assert_eq!(
                        print::module(&m, "erase_ownership"),
                        "// ===== after erase_ownership\n// Root\nfn take(self) -> Vec<u8> {\n    let v: Vec<u8> = self.v;\n    self.v = vec![];\n    return v;\n}\n\n// Root\nfn swap(self, c: bool) -> () {\n    if c {\n        let tmp1: Vec<u8> = self.v;\n        self.v = self.w;\n        self.w = tmp1;\n    } else {\n        while c {\n            let tmp2: Vec<u8> = self.v;\n            self.v = self.w;\n            self.w = tmp2;\n        };\n    };\n}\n\n// Root\nfn place(self, c: bool) -> () {\n    if c {\n        if let Option::Some(n) = self.o {\n            self.o@Some = 1;\n            self.v = vec![];\n        } else {\n            let d: D = self.d;\n            self.d = D_default;\n        };\n    };\n    let m3: Option<u8> = self.v.LastMut();\n    if let Option::Some(n) = m3 {\n        m3@Some = 2;\n    };\n}\n\n// Root\nfn local(self, o: Option<u8>) -> u8 {\n    if let Option::Some(n) = o {\n        return n;\n    } else {\n        return o.UnwrapOrDefault(0);\n    };\n}\n\n"
                    );
                }
            }
        }
    }
}
