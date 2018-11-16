#[derive(Debug, Clone)]
pub enum BinOp {
    Lt,
    Le,
    Gt,
    Ge,
    Neq,
    Eq,
    Or,
    And,
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Impl,
    Arrow,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Not,
    UMinus,
}

#[derive(Debug, Clone)]
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
