#[derive(Debug, Clone)]
pub enum BinOp {
    Lt,
    Le,
    Gt,
    Ge,
    Mul,
    Div,
    Add,
    Sub,
    Mod,
    Or,
    Xor,
    And,
    Impl,
    Neq,
    Eq,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Not,
    UMinus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Real,
    Bool,
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Real(f32),
}
