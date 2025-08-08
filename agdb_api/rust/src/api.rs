use crate::AgdbApi;
use crate::ReqwestClient;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb::api::ApiDefinition;
use agdb::api::Expression;
use agdb::api::Function;
use agdb::api::NamedType;
use agdb::api::Type;
use std::collections::HashSet;

#[allow(dead_code, clippy::upper_case_acronyms)]
pub struct API {
    main_types: Vec<Type>,
}

#[allow(dead_code)]
impl API {
    pub fn def() -> Self {
        Self {
            main_types: vec![
                AgdbApi::<ReqwestClient>::def(),
                QueryBuilder::def(),
                QueryResult::def(),
            ],
        }
    }

    pub fn types(&self) -> Vec<Type> {
        let mut types = HashSet::new();

        for ty in &self.main_types {
            get_types(ty, &mut types);
        }

        let mut r: Vec<Type> = types.into_iter().collect();
        r.sort_by(|a, b| a.name().cmp(b.name()));
        r
    }
}

fn get_types(ty: &Type, types: &mut HashSet<Type>) {
    if types.contains(ty) {
        return;
    }

    types.insert(ty.clone());

    match ty {
        Type::None | Type::U8 | Type::I64 | Type::U64 | Type::F64 | Type::String | Type::User => {}
        Type::Enum(e) => get_named_types(&e.variants, types),
        Type::Struct(s) => {
            get_named_types(&s.fields, types);
            (s.functions)().iter().for_each(|f| {
                get_function_types(f, types);
            });
        }
        Type::List(l) => get_types(&(l.ty)(), types),
    }
}

fn get_function_types(func: &Function, types: &mut HashSet<Type>) {
    get_named_types(&func.args, types);

    func.expressions.iter().for_each(|e| {
        get_expression_types(e, types);
    });

    if let Some(ret) = func.ret {
        get_types(&ret(), types);
    }
}

fn get_expression_types(e: &Expression, types: &mut HashSet<Type>) {
    match e {
        Expression::Closure { ret, body } => {
            if let Some(ret) = ret {
                get_types(&ret(), types);
            }
            body.iter().for_each(|e| get_expression_types(e, types));
        }
        Expression::Let {
            name: _,
            ty: Some(ty),
            value: _,
        } => {
            get_types(&ty(), types);
        }
        _ => {}
    }
}

fn get_named_types(tys: &[NamedType], types: &mut HashSet<Type>) {
    tys.iter().for_each(|v| {
        get_types(&(v.ty)(), types);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Uncomment for debugging the types in the API
    // #[test]
    // fn write_api() {
    //     let mut buf = String::new();
    //     let api = API::def();
    //     let types = api.types();

    //     for ty in types {
    //         buf.push_str(ty.name());
    //         buf.push('\n');
    //     }

    //     std::fs::write("API", buf).unwrap();
    // }

    #[test]
    fn no_unknown_expressions() {
        let api = API::def();
        let types = api.types();

        for ty in types {
            for f in ty.functions() {
                for e in f.expressions {
                    if let Expression::Unknown(e) = e {
                        panic!("Unknown expression: {e:?}");
                    }
                }
            }
        }
    }
}
