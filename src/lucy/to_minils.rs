use crate::minils::ast as minils;
use crate::lucy::clock_typed_ast as typ;

pub fn to_minils(node: typ::Node) -> minils::Node {
    let name = node.name;
    let in_params = node.in_params;
    let out_params = node.out_params;
    let local_params = node.local_params;
    let mut new_node = minils::Node {
        name,
        in_params,
        out_params,
        local_params,
        eq_list: vec![],
    };
    for (idents, expr) in node.eq_list {
        new_node.eq_list.push((idents, to_minils_expr(expr)));
    }
    new_node
}

fn to_minils_expr(expr: typ::Expr) -> minils::Expr {
    let expr_ = match expr.expr {
        typ::BaseExpr::Value(v) => minils::BaseExpr::Value(v),
        typ::BaseExpr::UnOp(op, box e1) => {
            let e1 = to_minils_expr(e1);
            minils::BaseExpr::UnOp(op, box e1)
        }
        typ::BaseExpr::BinOp(op, box e1, box e2) => {
            let e1 = to_minils_expr(e1);
            let e2 = to_minils_expr(e2);
            minils::BaseExpr::BinOp(op, box e1, box e2)
        }
        typ::BaseExpr::Fby(e1, box e2) => {
            let e2 = to_minils_expr(e2);
            minils::BaseExpr::Fby(e1, box e2)
        }
        typ::BaseExpr::When(box e, ck, b) => {
            let e = to_minils_expr(e);
            minils::BaseExpr::When(box e, ck, b)
        }
        typ::BaseExpr::Merge(ck, box e_t, box e_f) => {
            let e_t = to_minils_expr(e_t);
            let e_f = to_minils_expr(e_f);
            minils::BaseExpr::Merge(ck, box e_t, box e_f)
        }
        typ::BaseExpr::IfThenElse(box e1, box e2, box e3) => {
            let e1 = to_minils_expr(e1);
            let e2 = to_minils_expr(e2);
            let e3 = to_minils_expr(e3);
            minils::BaseExpr::IfThenElse(box e1, box e2, box e3)
        }
        typ::BaseExpr::Var(s) => minils::BaseExpr::Var(s),
        typ::BaseExpr::FunCall(s, exprs) => {
            let exprs = exprs.into_iter().map(to_minils_expr).collect();
            minils::BaseExpr::FunCall(s, exprs)
        }
    };
    minils::Expr {
        typ: expr.typ,
        expr: expr_,
        clock: expr.clock,
    }
}
