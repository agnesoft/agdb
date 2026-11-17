//! The declarations of a crate as written: items, their attributes,
//! visibility, generics and types, every path with what it names. Bodies
//! and expressions are their source text.

use crate::ItemId;
use crate::Resolved;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: ItemId,
    pub attrs: Vec<Attr>,
    pub vis: Vis,
    pub kind: ItemKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    Fn(Fn),
    Const(Const),
    Static(Static),
    TypeAlias(TypeAlias),
}

/// An outer attribute after `cfg_attr`s are applied.
#[derive(Debug, Clone, PartialEq)]
pub enum Attr {
    /// A doc comment or `#[doc = "..."]`: the string.
    Doc(String),
    /// Any other attribute: the source text inside `#[..]`.
    Meta(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vis {
    Private,
    Public,
    Crate,
    Super,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub predicates: Vec<Predicate>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericParam {
    /// The name with its `'`.
    Lifetime(String),
    Type {
        name: String,
        bounds: Vec<Bound>,
    },
    Const {
        name: String,
        ty: Type,
    },
}

/// `ty: bounds` of a `where` clause.
#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    pub ty: Type,
    pub bounds: Vec<Bound>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Bound {
    Trait(Path),
    Lifetime(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Path(Path),
    Ref {
        lifetime: Option<String>,
        mutable: bool,
        ty: Box<Type>,
    },
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, Expr),
    ImplTrait(Vec<Bound>),
    DynTrait(Vec<Bound>),
    Paren(Box<Type>),
    FnPtr(FnTypes),
}

/// The parameter and return types of a function pointer or of a `Fn(..)`
/// bound.
#[derive(Debug, Clone, PartialEq)]
pub struct FnTypes {
    pub inputs: Vec<Type>,
    pub output: Option<Box<Type>>,
}

/// A path as written and what it names.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub leading_colon: bool,
    pub segments: Vec<Segment>,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub name: String,
    pub args: Args,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Args {
    None,
    Angle(Vec<GenericArg>),
    /// `Fn(A) -> B`.
    Fn(FnTypes),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericArg {
    Type(Type),
    Lifetime(String),
    Const(Expr),
    AssocType { name: String, ty: Type },
}

/// What the first segments of a [`Path`] name.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// A generic parameter in scope.
    Generic,
    SelfType,
    Resolved(Resolved),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Verbatim(String),
}

/// A function body, braces included.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Verbatim(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub name: String,
    pub generics: Generics,
    pub fields: Fields,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fields {
    Named(Vec<Field>),
    Unnamed(Vec<Field>),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub attrs: Vec<Attr>,
    pub vis: Vis,
    /// None in [`Fields::Unnamed`].
    pub name: Option<String>,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub generics: Generics,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub attrs: Vec<Attr>,
    pub name: String,
    pub fields: Fields,
    pub discriminant: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub name: String,
    pub generics: Generics,
    pub supertraits: Vec<Bound>,
    pub items: Vec<TraitItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitItem {
    pub attrs: Vec<Attr>,
    pub kind: TraitItemKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitItemKind {
    /// The body is the default.
    Fn {
        sig: Signature,
        body: Option<Block>,
    },
    Type {
        name: String,
        bounds: Vec<Bound>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Impl {
    pub generics: Generics,
    pub trait_: Option<Path>,
    pub self_ty: Type,
    pub items: Vec<ImplItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplItem {
    pub attrs: Vec<Attr>,
    pub vis: Vis,
    pub kind: ImplItemKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplItemKind {
    Fn(Fn),
    Type { name: String, ty: Type },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub sig: Signature,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub constness: bool,
    pub asyncness: bool,
    pub name: String,
    pub generics: Generics,
    pub receiver: Option<Receiver>,
    pub params: Vec<Param>,
    pub output: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Receiver {
    /// `self`, `mut self`.
    Value { mutable: bool },
    /// `&self`, `&'a mut self`, ...
    Ref {
        lifetime: Option<String>,
        mutable: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Static {
    pub name: String,
    pub ty: Type,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub generics: Generics,
    pub ty: Type,
}
