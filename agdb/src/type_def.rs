pub mod enum_def;
pub mod expression_def;
pub mod function_def;
pub mod impl_def;
pub mod struct_def;
pub mod trait_def;

pub use enum_def::Enum;
pub use expression_def::Expression;
pub use expression_def::LiteralValue;
pub use expression_def::Op;
pub use function_def::Function;
pub use impl_def::Impl;
pub use struct_def::Struct;
pub use trait_def::Trait;

pub type Tuple = &'static [fn() -> Type];

pub trait TypeDefinition {
    fn type_def() -> Type;

    fn generic_type_names() -> Vec<&'static str> {
        match Self::type_def() {
            Type::Enum(e) => e.generics.iter().map(|g| g.name).collect(),
            Type::Function(f) => f.generics.iter().map(|g| g.name).collect(),
            Type::Test(f) => f.generics.iter().map(|g| g.name).collect(),
            Type::Struct(s) => s.generics.iter().map(|g| g.name).collect(),
            Type::Trait(t) => t.generics.iter().map(|g| g.name).collect(),
            Type::Impl(i) => i.generics.iter().map(|g| g.name).collect(),
            _ => vec![],
        }
    }

    fn impl_defs() -> Vec<Impl> {
        vec![]
    }
}

#[derive(Debug, agdb::TypeDef)]
pub enum Type {
    Enum(Enum),
    Function(Function),
    Test(Function),
    Generic(Generic),
    Impl(Impl),
    Literal(Literal),
    Option(fn() -> Type),
    Pointer(Pointer),
    Reference(Reference),
    Result { ok: fn() -> Type, err: fn() -> Type },
    SelfType(bool),
    Slice(fn() -> Type),
    Static(Static),
    Struct(Struct),
    Trait(Trait),
    Tuple(&'static [fn() -> Type]),
    Vec(fn() -> Type),
}

#[derive(Debug, agdb::TypeDef)]
pub struct Static {
    pub name: &'static str,
    pub ty: fn() -> Type,
    pub value: &'static [Expression],
}

impl Type {
    pub fn name(&self) -> &'static str {
        match self {
            Type::Enum(e) => e.name,
            Type::Function(_) => "fn",
            Type::Test(_) => "test",
            Type::Generic(g) => g.name,
            Type::Impl(i) => i.name,
            Type::Literal(l) => l.name(),
            Type::Option(_) => "Option",
            Type::Pointer(_) => "Pointer",
            Type::Reference(_) => "Reference",
            Type::Result { .. } => "Result",
            Type::SelfType(_) => "Self",
            Type::Slice(_) => "Slice",
            Type::Static(s) => s.name,
            Type::Struct(s) => s.name,
            Type::Trait(t) => t.name,
            Type::Tuple(_) => "Tuple",
            Type::Vec(_) => "Vec",
        }
    }

    #[allow(dead_code)]
    pub fn functions(&self) -> &'static [Function] {
        match self {
            Type::Impl(i) => i.functions,
            Type::Trait(t) => t.functions,
            _ => &[],
        }
    }
}

#[derive(Debug, agdb::TypeDef)]
pub struct Variable {
    pub name: &'static str,
    pub ty: Option<fn() -> Type>,
}

#[derive(Debug, agdb::TypeDef)]
pub struct Generic {
    pub kind: GenericKind,
    pub name: &'static str,
    pub bounds: &'static [fn() -> Type],
}

#[derive(Debug, agdb::TypeDef)]
pub enum GenericKind {
    Type,
    Lifetime,
    Const,
}

#[derive(Debug, agdb::TypeDef)]
pub struct Reference {
    pub mutable: bool,
    pub lifetime: Option<&'static str>,
    pub ty: fn() -> Type,
}

#[derive(Debug, agdb::TypeDef)]
pub struct Pointer {
    pub kind: PointerKind,
    pub ty: fn() -> Type,
}

#[derive(Debug, agdb::TypeDef)]
pub enum PointerKind {
    Arc,
    ArcWeak,
    Box,
    Cell,
    Cow,
    LazyCell,
    LazyLock,
    Mutex,
    OnceCell,
    OnceLock,
    Pin,
    Raw,
    Rc,
    RcWeak,
    RefCell,
    RwLock,
    UnsafeCell,
}

#[derive(Debug, agdb::TypeDef)]
pub enum Literal {
    Bool,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    Str,
    String,
    U8,
    U16,
    U32,
    U64,
    U128,
    Unit,
    Usize,
}

impl Literal {
    pub const fn name(&self) -> &'static str {
        match self {
            Literal::Bool => "bool",
            Literal::F32 => "f32",
            Literal::F64 => "f64",
            Literal::I8 => "i8",
            Literal::I16 => "i16",
            Literal::I32 => "i32",
            Literal::I64 => "i64",
            Literal::I128 => "i128",
            Literal::Isize => "isize",
            Literal::Str => "str",
            Literal::String => "String",
            Literal::U8 => "u8",
            Literal::U16 => "u16",
            Literal::U32 => "u32",
            Literal::U64 => "u64",
            Literal::U128 => "u128",
            Literal::Unit => "()",
            Literal::Usize => "usize",
        }
    }
}

macro_rules! impl_type_def_literal {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl TypeDefinition for $ty {
                fn type_def() -> Type {
                    Type::Literal(Literal::$variant)
                }
            }
        )*
    };
}

impl_type_def_literal! {
    bool   => Bool,
    i8     => I8,
    i16    => I16,
    i32    => I32,
    i64    => I64,
    i128   => I128,
    isize  => Isize,
    f32    => F32,
    f64    => F64,
    &str   => Str,
    String => String,
    u8     => U8,
    u16    => U16,
    u32    => U32,
    u64    => U64,
    u128   => U128,
    usize  => Usize,
    ()     => Unit,
}

impl<T: TypeDefinition> TypeDefinition for &[T] {
    fn type_def() -> Type {
        Type::Slice(T::type_def)
    }
}

impl<T: TypeDefinition> TypeDefinition for Vec<T> {
    fn type_def() -> Type {
        Type::Vec(T::type_def)
    }
}

impl<T: TypeDefinition, E: TypeDefinition> TypeDefinition for Result<T, E> {
    fn type_def() -> Type {
        Type::Result {
            ok: T::type_def,
            err: E::type_def,
        }
    }
}

impl<T: TypeDefinition> TypeDefinition for Option<T> {
    fn type_def() -> Type {
        Type::Option(T::type_def)
    }
}

impl<T: TypeDefinition> TypeDefinition for Box<T> {
    fn type_def() -> Type {
        Type::Pointer(Pointer {
            kind: PointerKind::Box,
            ty: T::type_def,
        })
    }
}

impl<T> TypeDefinition for &T
where
    T: TypeDefinition,
{
    fn type_def() -> Type {
        Type::Reference(Reference {
            mutable: false,
            lifetime: None,
            ty: T::type_def,
        })
    }
}

impl<T: TypeDefinition, V: TypeDefinition> TypeDefinition for (T, V) {
    fn type_def() -> Type {
        Type::Tuple(&[T::type_def, V::type_def])
    }
}

impl<T: TypeDefinition> TypeDefinition for std::sync::Arc<T> {
    fn type_def() -> Type {
        Type::Pointer(Pointer {
            kind: PointerKind::Arc,
            ty: T::type_def,
        })
    }
}

impl<const N: usize, T: TypeDefinition> TypeDefinition for [T; N] {
    fn type_def() -> Type {
        Type::Slice(T::type_def)
    }
}

impl TypeDefinition for std::path::PathBuf {
    fn type_def() -> Type {
        Type::Struct(Struct {
            name: "PathBuf",
            generics: &[],
            fields: &[Variable {
                name: "inner",
                ty: Some(|| Type::Vec(u8::type_def)),
            }],
            impl_defs: Vec::new,
        })
    }
}

impl TypeDefinition for std::time::Duration {
    fn type_def() -> Type {
        Type::Struct(Struct {
            name: "Duration",
            generics: &[],
            fields: &[
                Variable {
                    name: "secs",
                    ty: Some(|| Type::Literal(Literal::U64)),
                },
                Variable {
                    name: "nanos",
                    ty: Some(|| Type::Literal(Literal::U32)),
                },
            ],
            impl_defs: Vec::new,
        })
    }
}

impl TypeDefinition for std::sync::atomic::AtomicU16 {
    fn type_def() -> Type {
        Type::Struct(Struct {
            name: "AtomicU16",
            generics: &[],
            fields: &[Variable {
                name: "value",
                ty: Some(|| Type::Literal(Literal::U16)),
            }],
            impl_defs: Vec::new,
        })
    }
}

impl<T: TypeDefinition> TypeDefinition for std::sync::OnceLock<T> {
    fn type_def() -> Type {
        Type::Pointer(Pointer {
            kind: PointerKind::OnceLock,
            ty: T::type_def,
        })
    }
}

impl<T: TypeDefinition> TypeDefinition for std::sync::Weak<T> {
    fn type_def() -> Type {
        Type::Pointer(Pointer {
            kind: PointerKind::ArcWeak,
            ty: T::type_def,
        })
    }
}

impl<T: TypeDefinition> TypeDefinition for tokio::sync::RwLock<T> {
    fn type_def() -> Type {
        Type::Pointer(Pointer {
            kind: PointerKind::RwLock,
            ty: T::type_def,
        })
    }
}

macro_rules! impl_type_def_fn_ptr {
    ($(($($arg:ident),*)),* $(,)?) => {
        $(
            impl<R: TypeDefinition, $($arg: TypeDefinition),*> TypeDefinition for fn($($arg),*) -> R {
                fn type_def() -> Type {
                    Type::Function(Function {
                        name: "",
                        generics: &[],
                        args: &[
                            $(
                                Variable {
                                    name: "",
                                    ty: Some(<$arg as TypeDefinition>::type_def),
                                }
                            ),*
                        ],
                        ret: <R as TypeDefinition>::type_def,
                        async_fn: false,
                        body: &[],
                    })
                }
            }
        )*
    };
}

impl_type_def_fn_ptr! {
    (),
    (A0),
    (A0, A1),
    (A0, A1, A2),
    (A0, A1, A2, A3),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_def_literals() {
        assert!(matches!(bool::type_def(), Type::Literal(Literal::Bool)));
        assert!(matches!(i32::type_def(), Type::Literal(Literal::I32)));
        assert!(matches!(f64::type_def(), Type::Literal(Literal::F64)));
        assert!(matches!(String::type_def(), Type::Literal(Literal::String)));
    }

    #[test]
    fn options() {
        let Type::Option(inner) = Option::<i32>::type_def() else {
            panic!("Expected an option type definition");
        };

        let Type::Literal(Literal::I32) = inner() else {
            panic!("Expected a literal type definition");
        };
    }

    #[test]
    fn results() {
        let Type::Result { ok, err } = Result::<i32, String>::type_def() else {
            panic!("Expected a result type definition");
        };

        let Type::Literal(Literal::I32) = ok() else {
            panic!("Expected a literal type definition for ok");
        };

        let Type::Literal(Literal::String) = err() else {
            panic!("Expected a literal type definition for err");
        };
    }

    #[test]
    fn derive_type_enum_itself() {
        let Type::Enum(def) = Type::type_def() else {
            panic!("Expected enum type definition for Type");
        };

        assert_eq!(def.name, "Type");
        assert!(def.variants.iter().any(|v| v.name == "Function"));
        assert!(def.variants.iter().any(|v| v.name == "Test"));
        assert!(def.variants.iter().any(|v| v.name == "Trait"));
    }

    #[test]
    fn function_pointer_type_def() {
        let Type::Function(def) = <fn(i32) -> String as TypeDefinition>::type_def() else {
            panic!("Expected function type definition");
        };

        assert_eq!(def.args.len(), 1);
        assert_eq!(def.args[0].name, "");

        let Some(arg_ty) = def.args[0].ty else {
            panic!("Expected i32 argument type");
        };
        let Type::Literal(Literal::I32) = arg_ty() else {
            panic!("Expected i32 argument type");
        };

        let Type::Literal(Literal::String) = (def.ret)() else {
            panic!("Expected String return type");
        };
    }
}
