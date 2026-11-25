//! The items of a crate as written: their attributes, visibility, generics,
//! types and bodies, every path with what it names.

use crate::Resolved;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
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
    Use(Use),
    ExternCrate(ExternCrate),
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
    Qualified(Box<Qualified>),
    Ref {
        lifetime: Option<String>,
        mutable: bool,
        ty: Box<Type>,
    },
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, Box<Expr>),
    ImplTrait(Vec<Bound>),
    DynTrait(Vec<Bound>),
    Paren(Box<Type>),
    FnPtr(FnTypes),
    /// `_`.
    Infer,
}

/// `<ty as Trait>::rest` or `<ty>::rest`.
#[derive(Debug, Clone, PartialEq)]
pub struct Qualified {
    pub ty: Type,
    pub trait_: Option<Path>,
    pub rest: Vec<Segment>,
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
    /// `<A, B>`, `::<A, B>` in expressions and patterns.
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
    /// A binding in scope: a parameter, `self` or a binding of a pattern.
    Local,
    /// An item of an enclosing block.
    LocalItem,
    Resolved(Resolved),
}

/// `use tree;`, `use ::tree;`.
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub leading_colon: bool,
    pub tree: UseTree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseTree {
    /// `name::tree`.
    Path { name: String, tree: Box<UseTree> },
    /// `name`, `name as rename` (`self` for the path before it), with what
    /// it names in the first of the type, value and macro namespaces it
    /// names something in.
    Name {
        name: String,
        rename: Option<String>,
        target: Target,
    },
    /// `*`, with what the path before it names.
    Glob(Target),
    /// `{a, b::c}`.
    Group(Vec<UseTree>),
}

/// `extern crate name;`, `extern crate name as rename;`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternCrate {
    pub name: String,
    pub rename: Option<String>,
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
    pub unsafety: bool,
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
    pub unsafety: bool,
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
    /// `self: Box<Self>`, `mut self: Rc<Self>`, ...
    Typed { mutable: bool, ty: Box<Type> },
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

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        attrs: Vec<Attr>,
        pat: Pat,
        init: Option<Expr>,
        /// The block of `let pat = init else { .. };`.
        diverge: Option<Expr>,
    },
    Item(Item),
    /// An expression with `;`, or without: the value of the block when it
    /// is the last statement.
    Expr {
        attrs: Vec<Attr>,
        expr: Expr,
        semi: bool,
    },
}

/// An expression with its parentheses as written.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Array(Vec<Expr>),
    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `async {}`, `async move {}`.
    Async {
        capture: bool,
        block: Block,
    },
    Await(Box<Expr>),
    /// `#[attr] expr`, an expression with attributes.
    Attributed {
        attrs: Vec<Attr>,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Block(Block),
    Break,
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Cast {
        expr: Box<Expr>,
        ty: Box<Type>,
    },
    Closure(Box<Closure>),
    /// `const { .. }`.
    Const(Block),
    Continue,
    Field {
        expr: Box<Expr>,
        member: Member,
    },
    ForLoop {
        pat: Box<Pat>,
        expr: Box<Expr>,
        body: Block,
    },
    If {
        cond: Box<Expr>,
        then_branch: Block,
        /// A block or another `if`.
        else_branch: Option<Box<Expr>>,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },
    /// `let pat = expr` in a condition.
    Let {
        pat: Box<Pat>,
        expr: Box<Expr>,
    },
    Lit(Lit),
    Loop(Block),
    Macro(Box<Macro>),
    Match {
        expr: Box<Expr>,
        arms: Vec<Arm>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        turbofish: Vec<GenericArg>,
        args: Vec<Expr>,
    },
    Paren(Box<Expr>),
    Path(Path),
    Qualified(Box<Qualified>),
    Range {
        start: Option<Box<Expr>>,
        /// `..=`.
        closed: bool,
        end: Option<Box<Expr>>,
    },
    Reference {
        mutable: bool,
        expr: Box<Expr>,
    },
    /// `[expr; len]`.
    Repeat {
        expr: Box<Expr>,
        len: Box<Expr>,
    },
    Return(Option<Box<Expr>>),
    Struct {
        path: Path,
        fields: Vec<FieldValue>,
    },
    /// `expr?`.
    Try(Box<Expr>),
    Tuple(Vec<Expr>),
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    /// `unsafe { ... }`.
    Unsafe(Block),
    While {
        cond: Box<Expr>,
        body: Block,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitXorAssign,
    BitAndAssign,
    BitOrAssign,
    ShlAssign,
    ShrAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Deref,
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Str(String),
    Char(char),
    /// The digits in base 10.
    Int {
        digits: String,
        suffix: String,
    },
    Float {
        digits: String,
        suffix: String,
    },
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Named(String),
    Index(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub member: Member,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    /// `move`.
    pub capture: bool,
    pub inputs: Vec<Pat>,
    pub output: Option<Type>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub attrs: Vec<Attr>,
    pub pat: Pat,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// A macro call by what the macro does with its arguments, see
/// [`crate::MacroCall`].
#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub path: Path,
    pub call: MacroCall,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroCall {
    Format {
        exprs: Vec<Expr>,
        format: Option<FormatArgs>,
    },
    Vec(Vec<Expr>),
    VecRepeat(Box<Expr>, Box<Expr>),
    Matches {
        expr: Box<Expr>,
        pat: Box<Pat>,
        guard: Option<Box<Expr>>,
    },
    /// The statics of `thread_local!`.
    ThreadLocal(Vec<Item>),
    /// `file!()`, `line!()`, `column!()`, `module_path!()`.
    NoArgs,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatArgs {
    pub string: String,
    pub args: Vec<FormatArg>,
    /// The names the string uses that no argument gives.
    pub captures: Vec<Path>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatArg {
    pub name: Option<String>,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pat {
    /// `name`, `ref mut name @ subpat`, ...: an identifier that names no
    /// constant, unit struct or unit variant.
    Binding {
        by_ref: bool,
        mutable: bool,
        name: String,
        subpat: Option<Box<Pat>>,
    },
    Lit(Lit),
    Or(Vec<Pat>),
    Paren(Box<Pat>),
    Path(Path),
    /// `a..=b`, `a..`, `..=b`: the bounds are literals or paths.
    Range {
        start: Option<Box<Expr>>,
        closed: bool,
        end: Option<Box<Expr>>,
    },
    Ref {
        mutable: bool,
        pat: Box<Pat>,
    },
    Struct {
        path: Path,
        fields: Vec<FieldPat>,
        /// `..`.
        rest: bool,
    },
    Tuple(Vec<Pat>),
    TupleStruct {
        path: Path,
        elems: Vec<Pat>,
    },
    /// `pat: ty` of a `let` or a closure parameter.
    Type {
        pat: Box<Pat>,
        ty: Type,
    },
    Wild,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldPat {
    pub member: Member,
    pub pat: Pat,
}

impl BinOp {
    pub const ALL: [BinOp; 28] = [
        BinOp::Add,
        BinOp::Sub,
        BinOp::Mul,
        BinOp::Div,
        BinOp::Rem,
        BinOp::And,
        BinOp::Or,
        BinOp::BitXor,
        BinOp::BitAnd,
        BinOp::BitOr,
        BinOp::Shl,
        BinOp::Shr,
        BinOp::Eq,
        BinOp::Lt,
        BinOp::Le,
        BinOp::Ne,
        BinOp::Ge,
        BinOp::Gt,
        BinOp::AddAssign,
        BinOp::SubAssign,
        BinOp::MulAssign,
        BinOp::DivAssign,
        BinOp::RemAssign,
        BinOp::BitXorAssign,
        BinOp::BitAndAssign,
        BinOp::BitOrAssign,
        BinOp::ShlAssign,
        BinOp::ShrAssign,
    ];

    pub fn token(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::BitXor => "^",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Ne => "!=",
            BinOp::Ge => ">=",
            BinOp::Gt => ">",
            BinOp::AddAssign => "+=",
            BinOp::SubAssign => "-=",
            BinOp::MulAssign => "*=",
            BinOp::DivAssign => "/=",
            BinOp::RemAssign => "%=",
            BinOp::BitXorAssign => "^=",
            BinOp::BitAndAssign => "&=",
            BinOp::BitOrAssign => "|=",
            BinOp::ShlAssign => "<<=",
            BinOp::ShrAssign => ">>=",
        }
    }
}

impl UnOp {
    pub const ALL: [UnOp; 3] = [UnOp::Deref, UnOp::Not, UnOp::Neg];

    pub fn token(self) -> &'static str {
        match self {
            UnOp::Deref => "*",
            UnOp::Not => "!",
            UnOp::Neg => "-",
        }
    }
}
