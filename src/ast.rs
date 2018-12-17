//! Contains some part of the AST that are common in the different parts of the compiler

/// Binary operation type
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

/// Unary operation type
#[derive(Debug, Clone)]
pub enum UnOp {
    Not,
    UMinus,
}

/// Different types of the synchronous language
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Real,
    Bool,
}

/// Constant values
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Real(f32),
}

impl Value {
    /// Get the type of a constant
    pub fn get_type(&self) -> Type {
        match &self {
            Value::Bool(_) => Type::Bool,
            Value::Int(_) => Type::Int,
            Value::Real(_) => Type::Real,
        }
    }
}

/// A clock used in the synchronous language
/// Const means that it refer to a statically computable expression
#[derive(Debug, Clone, PartialEq)]
pub enum Clock {
    Const,
    Ck(Vec<(String, bool)>),
}

impl Clock {
    /// Check if two clocks are compatible
    /// Two clocks are compatible is they are equals, or if one of them is const
    pub fn is_compatible(clock1: &Clock, clock2: &Clock) -> bool {
        match clock1 {
            Clock::Const => true,
            Clock::Ck(hm1) => match clock2 {
                Clock::Const => true,
                Clock::Ck(hm2) => hm1 == hm2,
            },
        }
    }

    /// Check is the clock is faster or equal than the given clock
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
