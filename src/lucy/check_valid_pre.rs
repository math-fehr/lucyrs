//! Check if the pre constructs are valid and won't result in nil value being
//! used.

use crate::lucy::clock_typed_ast::{BaseExpr, Expr, Node};

/// Check if the pre defined in the lucyrs code are correct
pub fn check_valid_pre(nodes: &Vec<Node>) -> Result<(), String> {
    for node in nodes {
        for (idents, expr) in &node.eq_list {
            if !check_valid_pre_expr(expr, false) {
                return Err(format!("A pre construct will give an uninitialized value in the expression where {} is defined", idents[0]));
            }
        }
    }
    Ok(())
}

/// Check if the pre deifned in the expression are correct.
/// Depth is true if a pre construct is allowed
fn check_valid_pre_expr(expr: &Expr, depth: bool) -> bool {
    match &expr.expr {
        BaseExpr::Value(_) | BaseExpr::Var(_) | BaseExpr::Current(_, _) => true,
        BaseExpr::UnOp(_, box e) => check_valid_pre_expr(&e, depth),
        BaseExpr::BinOp(_, box e_1, box e_2) => {
            check_valid_pre_expr(&e_1, depth) && check_valid_pre_expr(&e_2, depth)
        }
        BaseExpr::When(box e, _, _) => check_valid_pre_expr(&e, depth),
        BaseExpr::Merge(_, box e_1, box e_2) => {
            check_valid_pre_expr(&e_1, depth) && check_valid_pre_expr(&e_2, depth)
        }
        BaseExpr::Fby(_, box e) => check_valid_pre_expr(&e, depth),
        BaseExpr::IfThenElse(box e_1, box e_2, box e_3) => {
            check_valid_pre_expr(&e_1, depth)
                && check_valid_pre_expr(&e_2, depth)
                && check_valid_pre_expr(&e_3, depth)
        }
        BaseExpr::FunCall(_, v, _) => {
            for e in v {
                if !check_valid_pre_expr(&e, depth) {
                    return false;
                }
            }
            true
        }
        BaseExpr::Pre(box e) => {
            if depth {
                check_valid_pre_expr(&e, false)
            } else {
                false
            }
        }
        BaseExpr::Arrow(box e_1, box e_2) => {
            check_valid_pre_expr(&e_1, depth) && check_valid_pre_expr(&e_2, true)
        }
    }
}
