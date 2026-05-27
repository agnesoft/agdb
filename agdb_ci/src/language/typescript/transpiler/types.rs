use agdb::type_def::GenericKind;
use agdb::type_def::Type;

use super::normalize::NormalizedType;
use super::normalize::Primitive;
use super::normalize::normalize_type;

pub fn type_annotation(ty: &Type) -> String {
    emit_normalized(&normalize_type(ty))
}

pub fn emit_normalized(normalized: &NormalizedType) -> String {
    match normalized {
        NormalizedType::Primitive(p) => match p {
            Primitive::Number => "number".to_string(),
            Primitive::String => "string".to_string(),
            Primitive::Boolean => "boolean".to_string(),
        },
        NormalizedType::Array(inner) => {
            let inner_str = emit_normalized(inner);
            if inner_str.contains('|') {
                format!("({inner_str})[]")
            } else {
                format!("{inner_str}[]")
            }
        }
        NormalizedType::Tuple(elements) => {
            let parts: Vec<String> = elements.iter().map(emit_normalized).collect();
            format!("[{}]", parts.join(", "))
        }
        NormalizedType::Nullable(inner) => {
            let inner_str = emit_normalized(inner);
            format!("{inner_str} | null")
        }
        NormalizedType::Named(name) => name.clone(),
        NormalizedType::NamedGeneric { name, args } => {
            let args_str: Vec<String> = args.iter().map(emit_normalized).collect();
            format!("{name}<{}>", args_str.join(", "))
        }
        NormalizedType::Generic(name) => name.clone(),
        NormalizedType::Function {
            args,
            ret,
            is_async,
        } => {
            let args_str: Vec<String> = args
                .iter()
                .enumerate()
                .map(|(i, a)| format!("arg{i}: {}", emit_normalized(a)))
                .collect();
            let ret_str = emit_normalized(ret);
            let ret_str = if *is_async {
                format!("Promise<{ret_str}>")
            } else {
                ret_str
            };
            format!("({}) => {ret_str}", args_str.join(", "))
        }
        NormalizedType::Void => "void".to_string(),
    }
}

pub fn generic_params(ty: &Type) -> String {
    let generics = match ty {
        Type::Struct(s) => s.generics,
        Type::Enum(e) => e.generics,
        Type::Trait(t) => t.generics,
        Type::Function(f) | Type::Test(f) => f.generics,
        Type::Impl(i) => i.generics,
        _ => return String::new(),
    };

    let type_params: Vec<&str> = generics
        .iter()
        .filter(|g| !matches!(g.kind, GenericKind::Lifetime))
        .map(|g| g.name)
        .collect();

    if type_params.is_empty() {
        String::new()
    } else {
        format!("<{}>", type_params.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::type_def::TypeDefinition;

    #[test]
    fn bool_annotation() {
        assert_eq!(type_annotation(&bool::type_def()), "boolean");
    }

    #[test]
    fn i32_annotation() {
        assert_eq!(type_annotation(&i32::type_def()), "number");
    }

    #[test]
    fn f64_annotation() {
        assert_eq!(type_annotation(&f64::type_def()), "number");
    }

    #[test]
    fn string_annotation() {
        assert_eq!(type_annotation(&String::type_def()), "string");
    }

    #[test]
    fn str_annotation() {
        assert_eq!(type_annotation(&<&str>::type_def()), "string");
    }

    #[test]
    fn unit_annotation() {
        assert_eq!(type_annotation(&<()>::type_def()), "void");
    }

    #[test]
    fn vec_annotation() {
        assert_eq!(type_annotation(&Vec::<i32>::type_def()), "number[]");
    }

    #[test]
    fn option_annotation() {
        assert_eq!(
            type_annotation(&Option::<String>::type_def()),
            "string | null"
        );
    }

    #[test]
    fn vec_of_option_annotation() {
        assert_eq!(
            type_annotation(&Vec::<Option<i32>>::type_def()),
            "(number | null)[]"
        );
    }

    #[test]
    fn result_annotation() {
        assert_eq!(
            type_annotation(&Result::<i32, String>::type_def()),
            "number"
        );
    }

    #[test]
    fn tuple_annotation() {
        assert_eq!(
            type_annotation(&<(i32, String)>::type_def()),
            "[number, string]"
        );
    }

    #[test]
    fn reference_stripped() {
        assert_eq!(type_annotation(&<&i32>::type_def()), "number");
    }

    #[test]
    fn box_stripped() {
        assert_eq!(type_annotation(&Box::<String>::type_def()), "string");
    }

    #[test]
    fn struct_name() {
        #[derive(agdb::TypeDef)]
        struct MyStruct {
            _field: i32,
        }
        assert_eq!(type_annotation(&MyStruct::type_def()), "MyStruct");
    }

    #[test]
    fn generic_struct_annotation() {
        #[derive(agdb::TypeDef)]
        struct Container<T> {
            _value: T,
        }
        assert_eq!(
            type_annotation(&Container::<i32>::type_def()),
            "Container<T>"
        );
    }

    #[test]
    fn generic_params_for_struct() {
        #[derive(agdb::TypeDef)]
        struct S<T, U> {
            _a: T,
            _b: U,
        }
        assert_eq!(generic_params(&S::<i32, String>::type_def()), "<T, U>");
    }

    #[test]
    fn generic_params_skips_lifetimes() {
        #[derive(agdb::TypeDef)]
        struct S<'a> {
            _a: &'a str,
        }
        assert_eq!(generic_params(&S::type_def()), "");
    }

    #[test]
    fn function_type_annotation() {
        let ty = <fn(i32, String) -> bool as TypeDefinition>::type_def();
        assert_eq!(
            type_annotation(&ty),
            "(arg0: number, arg1: string) => boolean"
        );
    }
}
