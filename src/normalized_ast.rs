use crate::ast::{BinOp, Type, UnOp, Value};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub in_params: Vec<(String, Type)>,
    pub out_params: Vec<(String, Type)>,
    pub defined_params: HashMap<String, Type>,
    pub eq_list: Vec<Eq>,
}

#[derive(Debug, Clone)]
pub struct Eq {
    pub typ: Vec<Type>,
    pub eq: ExprEqBase,
}

#[derive(Debug, Clone)]
pub enum ExprEqBase {
    Fby(String, Value, Box<ExprA>),
    FunCall(Vec<String>, String, Vec<ExprA>),
    ExprCA(String, Box<ExprCA>),
}

#[derive(Debug, Clone)]
pub struct ExprCA {
    pub typ: Type,
    pub expr: ExprCABase,
}

impl ExprCA {
    pub fn new_var(var: String, typ: Type) -> ExprCA {
        let expr_a = ExprA {
            typ: typ.clone(),
            expr: ExprABase::Var(var),
        };
        ExprCA {
            typ: typ,
            expr: ExprCABase::ExprA(box expr_a),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprCABase {
    Merge(String, Box<ExprCA>, Box<ExprCA>),
    ExprA(Box<ExprA>),
}

#[derive(Debug, Clone)]
pub struct ExprA {
    pub typ: Type,
    pub expr: ExprABase,
}

#[derive(Debug, Clone)]
pub enum ExprABase {
    Value(Value),
    Var(String),
    UnOp(UnOp, Box<ExprA>),
    BinOp(BinOp, Box<ExprA>, Box<ExprA>),
}
