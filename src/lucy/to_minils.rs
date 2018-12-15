use crate::ast::{Value, Clock, Type};
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
        let expr = to_minils_expr(expr, &mut new_node);
        new_node.eq_list.push((idents, expr));
    }
    new_node
}

fn to_minils_expr(expr: typ::Expr, node: &mut minils::Node) -> minils::Expr {
    let expr_ = match expr.expr {
        typ::BaseExpr::Value(v) => minils::BaseExpr::Value(v),
        typ::BaseExpr::UnOp(op, box e1) => {
            let e1 = to_minils_expr(e1, node);
            minils::BaseExpr::UnOp(op, box e1)
        }
        typ::BaseExpr::BinOp(op, box e1, box e2) => {
            let e1 = to_minils_expr(e1, node);
            let e2 = to_minils_expr(e2, node);
            minils::BaseExpr::BinOp(op, box e1, box e2)
        }
        typ::BaseExpr::Fby(e1, box e2) => {
            let e2 = to_minils_expr(e2, node);
            minils::BaseExpr::Fby(e1, box e2)
        }
        typ::BaseExpr::When(box e, ck, b) => {
            let e = to_minils_expr(e, node);
            minils::BaseExpr::When(box e, ck, b)
        }
        typ::BaseExpr::Merge(ck, box e_t, box e_f) => {
            let e_t = to_minils_expr(e_t, node);
            let e_f = to_minils_expr(e_f, node);
            minils::BaseExpr::Merge(ck, box e_t, box e_f)
        }
        typ::BaseExpr::IfThenElse(box e1, box e2, box e3) => {
            let e1 = to_minils_expr(e1, node);
            let e2 = to_minils_expr(e2, node);
            let e3 = to_minils_expr(e3, node);
            minils::BaseExpr::IfThenElse(box e1, box e2, box e3)
        }
        typ::BaseExpr::Var(s) => minils::BaseExpr::Var(s),
        typ::BaseExpr::FunCall(s, exprs) => {
            let exprs = exprs.into_iter().map(|e| to_minils_expr(e,node)).collect();
            minils::BaseExpr::FunCall(s, exprs)
        },
        typ::BaseExpr::Current(s, v) => {
            let clock = node.local_params.iter().find(|(s_,_,_)| s_ == &s).unwrap();
            to_minils_current(s, v, clock.2.clone(), expr.typ[0].clone(), node)
        }
    };
    minils::Expr {
        typ: expr.typ,
        expr: expr_,
        clock: expr.clock,
    }
}


fn to_minils_current(ident: String, value: Value, clock: Clock, typ: Type, node: &mut minils::Node) -> minils::BaseExpr {
    let ident_current = ident.clone() + "_current";
    let ident_pre = ident.clone() + "_pre";
    let pre_var_expr = minils::Expr {
        expr: minils::BaseExpr::Var(ident_pre.clone()),
        typ: vec![typ.clone()],
        clock: Clock::Ck(vec![]),
    };
    match clock.clone() {
        Clock::Const =>
            minils::BaseExpr::Var(ident)
            ,
        Clock::Ck(mut v) => {
            let mut expr = minils::Expr {
                typ: vec![typ.clone()],
                expr: minils::BaseExpr::Var(ident),
                clock,
            };
            while v.len() > 0 {
                let (ck,b) = v.pop().unwrap();
                let mut clock_true = v.clone();
                clock_true.push((ck.clone(),true));
                let clock_true = Clock::Ck(clock_true);
                let mut clock_false = v.clone();
                clock_false.push((ck.clone(),false));
                let clock_false = Clock::Ck(clock_false);
                let clock = Clock::Ck(v.clone());
                let base_expr = match b {
                    true => minils::BaseExpr::Merge(ck.clone(), box expr, box nested_when(pre_var_expr.clone(), clock_false)),
                    false => minils::BaseExpr::Merge(ck.clone(), box nested_when(pre_var_expr.clone(), clock_true), box expr),
                };
                expr = minils::Expr {
                    typ: vec![typ.clone()],
                    expr: base_expr,
                    clock,
                }
            }
            let current_var_expr = minils::Expr {
                expr: minils::BaseExpr::Var(ident_current.clone()),
                typ: vec![typ.clone()],
                clock: Clock::Ck(vec![]),
            };
            let pre_expr = minils::Expr {
                typ: vec![typ],
                expr: minils::BaseExpr::Fby(value, box current_var_expr),
                clock: Clock::Ck(vec![]),
            };
            node.eq_list.push((vec![ident_pre], pre_expr));
            node.eq_list.push((vec![ident_current.clone()], expr));
            minils::BaseExpr::Var(ident_current.clone())
        }
    }
}

fn nested_when(expr: minils::Expr, clock: Clock) -> minils::Expr {
    if let Clock::Ck(v) = &expr.clock {
        if v.len() != 0 {
            assert!(false);
        }
    }
    match clock {
        Clock::Const => expr,
        Clock::Ck(v) => {
            let mut expr = expr;
            for (ck,b) in v {
                let clock = match expr.clock.clone() {
                    Clock::Const => Clock::Ck(vec![(ck.clone(),b)]),
                    Clock::Ck(mut v) => {
                        v.push((ck.clone(),b));
                        Clock::Ck(v)
                    }
                };
                expr = minils::Expr {
                    expr: minils::BaseExpr::When(box expr.clone(), ck, b),
                    typ: expr.typ,
                    clock: clock,
                }
            }
            expr
        }
    }
}
