//! Contains the untyped AST for the LucyRS language

use crate::ast::{BinOp, Type, UnOp, Value, Clock};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub in_params: Vec<(String, Type)>,
    pub out_params: Vec<(String, Type)>,
    pub local_params: HashMap<String, (Type, Clock)>,
    pub eq_list: Vec<(Vec<String>, Expr)>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    When(Box<Expr>, String, bool),
    Merge(String, Box<Expr>, Box<Expr>),
    Fby(Value, Box<Expr>),
    IfThenElse(Box<Expr>, Box<Expr>, Box<Expr>),
    Var(String),
    FunCall(String, Vec<Expr>, Option<String>),
    Current(String, Value),
    Pre(Box<Expr>),
    Arrow(Vec<Expr>),
}
