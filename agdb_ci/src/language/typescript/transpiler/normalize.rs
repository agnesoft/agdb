use agdb::type_def::GenericKind;
use agdb::type_def::Literal;
use agdb::type_def::PointerKind;
use agdb::type_def::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum NormalizedType {
    Primitive(Primitive),
    Array(Box<NormalizedType>),
    Tuple(Vec<NormalizedType>),
    Nullable(Box<NormalizedType>),
    Named(String),
    NamedGeneric {
        name: String,
        args: Vec<NormalizedType>,
    },
    Generic(String),
    Function {
        args: Vec<NormalizedType>,
        ret: Box<NormalizedType>,
        is_async: bool,
    },
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Number,
    String,
    Boolean,
}

pub fn normalize_type(ty: &Type) -> NormalizedType {
    match ty {
        Type::Literal(lit) => normalize_literal(lit),
        Type::Vec(inner) | Type::Slice(inner) => {
            NormalizedType::Array(Box::new(normalize_type(&inner())))
        }
        Type::Option(inner) => NormalizedType::Nullable(Box::new(normalize_type(&inner()))),
        Type::Result { ok, .. } => normalize_type(&ok()),
        Type::Reference(reference) => normalize_type(&(reference.ty)()),
        Type::Pointer(pointer) => normalize_pointer(pointer),
        Type::Generic(generic) => NormalizedType::Generic(generic.name.to_string()),
        Type::SelfType(_) => NormalizedType::Named("this".to_string()),
        Type::Tuple(elements) => {
            let types: Vec<NormalizedType> =
                elements.iter().map(|f| normalize_type(&f())).collect();
            NormalizedType::Tuple(types)
        }
        Type::Struct(s) => {
            let generics: Vec<NormalizedType> = s
                .generics
                .iter()
                .filter(|g| !matches!(g.kind, GenericKind::Lifetime))
                .map(|g| NormalizedType::Generic(g.name.to_string()))
                .collect();
            if generics.is_empty() {
                NormalizedType::Named(s.name.to_string())
            } else {
                NormalizedType::NamedGeneric {
                    name: s.name.to_string(),
                    args: generics,
                }
            }
        }
        Type::Enum(e) => {
            let generics: Vec<NormalizedType> = e
                .generics
                .iter()
                .filter(|g| !matches!(g.kind, GenericKind::Lifetime))
                .map(|g| NormalizedType::Generic(g.name.to_string()))
                .collect();
            if generics.is_empty() {
                NormalizedType::Named(e.name.to_string())
            } else {
                NormalizedType::NamedGeneric {
                    name: e.name.to_string(),
                    args: generics,
                }
            }
        }
        Type::Trait(t) => NormalizedType::Named(t.name.to_string()),
        Type::Function(f) => {
            let args: Vec<NormalizedType> = f
                .args
                .iter()
                .filter_map(|arg| arg.ty.map(|ty_fn| normalize_type(&ty_fn())))
                .collect();
            let ret = normalize_type(&(f.ret)());
            NormalizedType::Function {
                args,
                ret: Box::new(ret),
                is_async: f.async_fn,
            }
        }
        Type::Test(f) => {
            let args: Vec<NormalizedType> = f
                .args
                .iter()
                .filter_map(|arg| arg.ty.map(|ty_fn| normalize_type(&ty_fn())))
                .collect();
            let ret = normalize_type(&(f.ret)());
            NormalizedType::Function {
                args,
                ret: Box::new(ret),
                is_async: f.async_fn,
            }
        }
        Type::Impl(_) => NormalizedType::Void,
        Type::Static(s) => normalize_type(&(s.ty)()),
    }
}

fn normalize_literal(lit: &Literal) -> NormalizedType {
    match lit {
        Literal::Bool => NormalizedType::Primitive(Primitive::Boolean),
        Literal::Str | Literal::String => NormalizedType::Primitive(Primitive::String),
        Literal::Unit => NormalizedType::Void,
        Literal::F32
        | Literal::F64
        | Literal::I8
        | Literal::I16
        | Literal::I32
        | Literal::I64
        | Literal::I128
        | Literal::Isize
        | Literal::U8
        | Literal::U16
        | Literal::U32
        | Literal::U64
        | Literal::U128
        | Literal::Usize => NormalizedType::Primitive(Primitive::Number),
    }
}

fn normalize_pointer(pointer: &agdb::type_def::Pointer) -> NormalizedType {
    let inner = normalize_type(&(pointer.ty)());
    match pointer.kind {
        PointerKind::Box
        | PointerKind::Arc
        | PointerKind::Rc
        | PointerKind::Mutex
        | PointerKind::RwLock
        | PointerKind::Cell
        | PointerKind::RefCell
        | PointerKind::UnsafeCell
        | PointerKind::Pin
        | PointerKind::Cow
        | PointerKind::Raw => inner,
        PointerKind::OnceLock
        | PointerKind::OnceCell
        | PointerKind::LazyLock
        | PointerKind::LazyCell
        | PointerKind::ArcWeak
        | PointerKind::RcWeak => NormalizedType::Nullable(Box::new(inner)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::type_def::TypeDefinition;

    #[test]
    fn literal_bool() {
        let ty = bool::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Primitive(Primitive::Boolean)
        );
    }

    #[test]
    fn literal_i32() {
        let ty = i32::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Primitive(Primitive::Number)
        );
    }

    #[test]
    fn literal_string() {
        let ty = String::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Primitive(Primitive::String)
        );
    }

    #[test]
    fn literal_unit() {
        let ty = <()>::type_def();
        assert_eq!(normalize_type(&ty), NormalizedType::Void);
    }

    #[test]
    fn vec_of_i32() {
        let ty = Vec::<i32>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Array(Box::new(NormalizedType::Primitive(Primitive::Number)))
        );
    }

    #[test]
    fn option_of_string() {
        let ty = Option::<String>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Nullable(Box::new(NormalizedType::Primitive(Primitive::String)))
        );
    }

    #[test]
    fn result_resolves_to_ok_type() {
        let ty = Result::<i32, String>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Primitive(Primitive::Number)
        );
    }

    #[test]
    fn reference_stripped() {
        let ty = <&i32>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Primitive(Primitive::Number)
        );
    }

    #[test]
    fn box_stripped() {
        let ty = Box::<String>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Primitive(Primitive::String)
        );
    }

    #[test]
    fn arc_stripped() {
        let ty = std::sync::Arc::<i32>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Primitive(Primitive::Number)
        );
    }

    #[test]
    fn tuple_types() {
        let ty = <(i32, String)>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Tuple(vec![
                NormalizedType::Primitive(Primitive::Number),
                NormalizedType::Primitive(Primitive::String),
            ])
        );
    }

    #[test]
    fn struct_named() {
        #[derive(agdb::TypeDef)]
        struct Point {
            _x: i32,
            _y: i32,
        }
        let ty = Point::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Named("Point".to_string())
        );
    }

    #[test]
    fn generic_struct() {
        #[derive(agdb::TypeDef)]
        struct Container<T> {
            _value: T,
        }
        let ty = Container::<i32>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::NamedGeneric {
                name: "Container".to_string(),
                args: vec![NormalizedType::Generic("T".to_string())],
            }
        );
    }

    #[test]
    fn once_lock_nullable() {
        let ty = std::sync::OnceLock::<i32>::type_def();
        assert_eq!(
            normalize_type(&ty),
            NormalizedType::Nullable(Box::new(NormalizedType::Primitive(Primitive::Number)))
        );
    }
}
