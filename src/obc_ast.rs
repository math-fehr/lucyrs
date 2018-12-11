use crate::ast::{BinOp, Type, UnOp, Value};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Machine {
    pub name: String,
    pub memory: HashMap<String, Value>,
    pub instances: HashMap<String, String>,
    pub step_inputs: Vec<(String, Type)>,
    pub step_returns: Vec<(String, Type)>,
    pub step_vars: HashMap<String, Type>,
    pub step_stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assignment(String, Expr),
    StateAssignment(String, Expr),
    Step(Vec<String>, String, Vec<Expr>),
    Reset(String),
    Control(String, Vec<Stmt>, Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(String),
    Value(Value),
    State(String),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}
