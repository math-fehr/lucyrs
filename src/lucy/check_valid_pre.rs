//! Check if the pre constructs are valid and won't result in nil value being
//! used.

use crate::lucy::clock_typed_ast::{BaseExpr, Expr, Node};

/// Check if the pre defined in the lucyrs code are correct
pub fn check_valid_pre(nodes: &Vec<Node>) -> Result<(), String> {
    for node in nodes {
        for (idents, expr) in &node.eq_list {
            if !check_valid_pre_expr(expr, 0) {
                return Err(format!("A pre construct will give an uninitialized value in the expression where {} is defined", idents[0]));
            }
        }
    }
    Ok(())
}

/// Check if the pre defined in the expression are correct.
/// depth is the depth we are in the future. It is the number of imbricated pre we can add
fn check_valid_pre_expr(expr: &Expr, depth: i32) -> bool {
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
            if depth > 0 {
                check_valid_pre_expr(&e, depth - 1)
            } else {
                false
            }
        }
        BaseExpr::Arrow(exprs) => {
            for i in 0..exprs.len() {
                if !check_valid_pre_expr(&exprs[i], depth + i as i32) {
                    return false;
                }
            }
            true
        }
    }
}
