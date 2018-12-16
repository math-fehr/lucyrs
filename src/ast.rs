use std::collections::HashMap;

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

impl Value {
    pub fn get_type(&self) -> Type {
        match &self {
            Value::Bool(_) => Type::Bool,
            Value::Int(_) => Type::Int,
            Value::Real(_) => Type::Real,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Clock {
    Const,
    Ck(Vec<(String, bool)>),
}

impl Clock {
    pub fn is_compatible(clock1: &Clock, clock2: &Clock) -> bool {
        match clock1 {
            Clock::Const => true,
            Clock::Ck(hm1) => match clock2 {
                Clock::Const => true,
                Clock::Ck(hm2) => hm1 == hm2,
            },
        }
    }

    pub fn is_faster_or_equal_than(&self, clock: &Clock) -> bool {
        match &self {
            Clock::Const => true,
            Clock::Ck(v_1) => match clock {
                Clock::Const => v_1.len() == 0,
                Clock::Ck(v_2) => {
                    if v_1.len() > v_2.len() {
                        false
                    } else {
                        v_1.iter()
                            .zip(v_2)
                            .map(|((c_1, _), (c_2, _))| c_1 == c_2)
                            .fold(true, |acc, b| b && acc)
                    }
                }
            },
        }
    }
}
