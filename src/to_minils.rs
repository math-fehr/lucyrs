use crate::ast::{Type, Value};
use crate::minils_ast as minils;
use crate::typer::typed_ast as typ;

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
        typ::BaseExpr::Pre(box e) => to_minils_pre(e),
        typ::BaseExpr::Arrow(box e1, box e2) => to_minils_arrow(e1, e2),
        typ::BaseExpr::Fby(box e1, box e2) => {
            let e1 = to_minils_expr(e1);
            let e2 = to_minils_expr(e2);
            minils::BaseExpr::Fby(box e1, box e2)
        },
        typ::BaseExpr::IfThenElse(box e1, box e2, box e3) => {
            let e1 = to_minils_expr(e1);
            let e2 = to_minils_expr(e2);
            let e3 = to_minils_expr(e3);
            minils::BaseExpr::IfThenElse(box e1, box e2, box e3)
        },
        typ::BaseExpr::Var(s) => minils::BaseExpr::Var(s),
        typ::BaseExpr::FunCall(s, exprs) => {
            let exprs = exprs.into_iter().map(to_minils_expr).collect();
            minils::BaseExpr::FunCall(s, exprs)
        }
    };
    minils::Expr {
        typ: expr.typ,
        expr: expr_,
    }
}

fn to_minils_pre(expr: typ::Expr) -> minils::BaseExpr {
    let expr2 = to_minils_expr(expr);
    // Here, e is not a tuple
    let value = match expr2.typ[0] {
        Type::Int => Value::Int(std::i32::MAX),
        Type::Bool => Value::Bool(false),
        Type::Real => Value::Real(std::f32::NAN),
    };
    let expr1 = minils::Expr {
        typ: expr2.typ.clone(),
        expr: minils::BaseExpr::Value(value),
    };
    minils::BaseExpr::Fby(box expr1, box expr2)
}

fn to_minils_arrow(expr1: typ::Expr, expr2: typ::Expr) -> minils::BaseExpr {
    let expr1 = to_minils_expr(expr1);
    let expr2 = to_minils_expr(expr2);
    let true_expr = minils::Expr{
        typ: vec![Type::Bool],
        expr: minils::BaseExpr::Value(Value::Bool(true))
    };
    let false_expr = minils::Expr{
        typ: vec![Type::Bool],
        expr: minils::BaseExpr::Value(Value::Bool(true))
    };
    let cond = minils::Expr {
        typ: vec![Type::Bool],
        expr: minils::BaseExpr::Fby(box true_expr, box false_expr)
    };
    minils::BaseExpr::IfThenElse(box cond, box expr1, box expr2)
}
