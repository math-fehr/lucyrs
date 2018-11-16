use crate::ast::{Type,Value,UnOp,BinOp};

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub in_params: Vec<(String, Type)>,
    pub out_params: Vec<(String, Type)>,
    pub local_params: Vec<(String, Type)>,
    pub eq_list: Vec<(Vec<String>, Expr)>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Pre(Box<Expr>),
    Arrow(Box<Expr>, Box<Expr>),
    IfThenElse(Box<Expr>, Box<Expr>, Box<Expr>),
    Var(String),
    FunCall(String, Vec<Expr>),
}
