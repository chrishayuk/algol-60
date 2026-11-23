//! The syntax tree, as the parser leaves it: names are still names.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Integer,
    Real,
    Boolean,
}

impl Type {
    pub fn name(self) -> &'static str {
        match self {
            Type::Integer => "integer",
            Type::Real => "real",
            Type::Boolean => "Boolean",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Pow,
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
    Ne,
    And,
    Or,
    Impl,
    Eqv,
}

#[derive(Clone, Debug)]
pub enum E {
    Int(i64),
    Real(f64),
    Bool(bool),
    Str(String),
    Id(String, u32),
    /// `a[i, j]`: a subscripted variable or a switch designator.
    Sub(String, Vec<E>, u32),
    /// `f(x, y)`: a function designator.
    Call(String, Vec<E>, u32),
    Neg(Box<E>),
    Pos(Box<E>),
    Not(Box<E>),
    Bin(BOp, Box<E>, Box<E>),
    If(Box<E>, Box<E>, Box<E>),
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub labels: Vec<(String, u32)>,
    pub kind: SK,
    pub line: u32,
}

#[derive(Clone, Debug)]
pub enum SK {
    Dummy,
    Assign(Vec<E>, E),
    Call(String, Vec<E>, u32),
    Goto(E),
    If(E, Box<Stmt>, Option<Box<Stmt>>),
    For(E, Vec<ForElem>, Box<Stmt>),
    Compound(Vec<Stmt>),
    Block(Block),
}

#[derive(Clone, Debug)]
pub enum ForElem {
    Expr(E),
    Step(E, E, E),
    While(E, E),
}

#[derive(Clone, Debug)]
pub struct Block {
    pub decls: Vec<Decl>,
    pub body: Vec<Stmt>,
}

pub type Name = (String, u32);

#[derive(Clone, Debug)]
pub enum Decl {
    Simple { own: bool, ty: Type, names: Vec<Name> },
    Array { own: bool, ty: Type, segs: Vec<(Vec<Name>, Vec<(E, E)>)> },
    Switch { name: Name, elems: Vec<E> },
    Proc(ProcDecl),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spec {
    Simple(Type),
    Array(Type),
    Proc(Option<Type>),
    Label,
    Switch,
    Str,
}

impl Spec {
    pub fn name(self) -> String {
        match self {
            Spec::Simple(t) => t.name().into(),
            Spec::Array(t) => format!("{} array", t.name()),
            Spec::Proc(Some(t)) => format!("{} procedure", t.name()),
            Spec::Proc(None) => "procedure".into(),
            Spec::Label => "label".into(),
            Spec::Switch => "switch".into(),
            Spec::Str => "string".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcDecl {
    pub name: Name,
    pub ty: Option<Type>,
    pub params: Vec<Name>,
    pub values: Vec<Name>,
    pub specs: Vec<(Spec, Vec<Name>)>,
    pub body: Box<Stmt>,
}
