//! Translate typed LucyRS AST into minils AST

use crate::ast::{Clock, Type, Value, BinOp};
use crate::ident::IdentGenerator;
use crate::lucy::clock_typed_ast as typ;
use crate::minils::ast as minils;

/// Translate a typed LucyRS AST into minils AST
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
        let expr = to_minils_expr(
            &IdentGenerator::new(idents[0].clone() + "_cond"),
            expr,
            &mut new_node,
        );
        new_node.eq_list.push((idents, expr));
    }
    new_node
}

/// Translate a typed LucyRS expression into minils expression
/// This function remove some syntaxic sugar from LucyRS,
/// like if_then_else construct, or pre, or arrow
fn to_minils_expr(
    ident: &IdentGenerator,
    expr: typ::Expr,
    node: &mut minils::Node,
) -> minils::Expr {
    let expr_ = match expr.expr {
        typ::BaseExpr::Value(v) => minils::BaseExpr::Value(v),
        typ::BaseExpr::UnOp(op, box e1) => {
            let e1 = to_minils_expr(ident, e1, node);
            minils::BaseExpr::UnOp(op, box e1)
        }
        typ::BaseExpr::BinOp(op, box e1, box e2) => {
            let e1 = to_minils_expr(ident, e1, node);
            let e2 = to_minils_expr(ident, e2, node);
            minils::BaseExpr::BinOp(op, box e1, box e2)
        }
        typ::BaseExpr::Fby(e1, box e2) => {
            let e2 = to_minils_expr(ident, e2, node);
            minils::BaseExpr::Fby(e1, box e2)
        }
        typ::BaseExpr::When(box e, ck, b) => {
            let e = to_minils_expr(ident, e, node);
            minils::BaseExpr::When(box e, ck, b)
        }
        typ::BaseExpr::Merge(ck, box e_t, box e_f) => {
            let e_t = to_minils_expr(ident, e_t, node);
            let e_f = to_minils_expr(ident, e_f, node);
            minils::BaseExpr::Merge(ck, box e_t, box e_f)
        }
        typ::BaseExpr::IfThenElse(box e_cond, box e_t, box e_f) => {
            let e_cond = to_minils_expr(ident, e_cond, node);
            let name_cond = ident.new_ident().get_ident();
            node.eq_list.push((vec![name_cond.clone()], e_cond));
            let e_t = to_minils_expr(ident, e_t, node);
            let e_f = to_minils_expr(ident, e_f, node);
            minils::BaseExpr::Merge(name_cond, box e_t, box e_f)
        }
        typ::BaseExpr::Var(s) => minils::BaseExpr::Var(s),
        typ::BaseExpr::FunCall(s, exprs, r) => {
            let exprs = exprs
                .into_iter()
                .map(|e| to_minils_expr(ident, e, node))
                .collect();
            minils::BaseExpr::FunCall(s, exprs, r)
        }
        typ::BaseExpr::Current(s, v) => {
            let clock = &node.local_params.get(&s).unwrap().1;
            to_minils_current(s, v, clock.clone(), expr.typ[0].clone(), node)
        }
        typ::BaseExpr::Pre(box e) => {
            let e = to_minils_expr(ident, e, node);
            let value = match &e.typ[0] {
                Type::Int => Value::Int(-12341234),
                Type::Real => Value::Real(std::f32::NAN),
                Type::Bool => Value::Bool(false),
            };
            minils::BaseExpr::Fby(value, box e)
        }
        typ::BaseExpr::Arrow(exprs) => {
            return to_minils_arrow(ident, exprs, expr.clock.clone(), expr.typ[0].clone(), node);
        }
    };
    minils::Expr {
        typ: expr.typ,
        expr: expr_,
        clock: expr.clock,
    }
}

fn to_minils_arrow(
    ident: &IdentGenerator,
    exprs: Vec<typ::Expr>,
    clock: Clock,
    typ: Type,
    node: &mut minils::Node,
) -> minils::Expr {
    let counter = ident.new_ident().get_ident();
    let var_counter = minils::Expr {
        expr: minils::BaseExpr::Var(counter.clone()),
        typ: vec![Type::Int],
        clock: clock.clone(),
    };
    let value_1 = minils::Expr {
        expr: minils::BaseExpr::Value(Value::Int(1)),
        typ: vec![Type::Int],
        clock: clock.clone(),
    };
    let incr_counter = minils::Expr {
        expr: minils::BaseExpr::BinOp(BinOp::Add, box var_counter, box value_1),
        typ: vec![Type::Int],
        clock: clock.clone(),
    };
    let counter_expr = minils::Expr {
        expr: minils::BaseExpr::Fby(Value::Int(0), box incr_counter),
        typ: vec![Type::Int],
        clock: clock.clone(),
    };
    node.eq_list.push((vec![counter.clone()], counter_expr));

    let var_counter = typ::Expr {
        expr: typ::BaseExpr::Var(counter),
        typ: vec![Type::Int],
        clock: clock.clone(),
    };
    let counter_equal_i = |i| {
        let value_i = typ::Expr {
            expr: typ::BaseExpr::Value(Value::Int(i)),
            typ: vec![Type::Int],
            clock: clock.clone(),
        };
        typ::Expr {
            expr: typ::BaseExpr::BinOp(BinOp::Eq, box var_counter.clone(), box value_i),
            typ: vec![Type::Bool],
            clock: clock.clone(),
        }
    };
    let mut expr = typ::Expr {
        expr: typ::BaseExpr::IfThenElse(box counter_equal_i((exprs.len()-2) as i32), box exprs[exprs.len()-2].clone(), box exprs[exprs.len()-1].clone()),
        typ: vec![typ.clone()],
        clock: clock.clone(),
    };
    for i in (0..exprs.len()-2).rev() {
        expr = typ::Expr {
            expr: typ::BaseExpr::IfThenElse(box counter_equal_i(i as i32), box exprs[i].clone(), box expr),
            typ: vec![typ.clone()],
            clock: clock.clone(),
        }
    }
    to_minils_expr(ident, expr, node)
}

/// Translate a LucyRS current expression into a minils expression
fn to_minils_current(
    ident: String,
    value: Value,
    clock: Clock,
    typ: Type,
    node: &mut minils::Node,
) -> minils::BaseExpr {
    let ident_current = ident.clone() + "_current";
    let ident_pre = ident.clone() + "_pre";
    let pre_var_expr = minils::Expr {
        expr: minils::BaseExpr::Var(ident_pre.clone()),
        typ: vec![typ.clone()],
        clock: Clock::Ck(vec![]),
    };
    match clock.clone() {
        Clock::Const => minils::BaseExpr::Var(ident),
        Clock::Ck(mut v) => {
            let mut expr = minils::Expr {
                typ: vec![typ.clone()],
                expr: minils::BaseExpr::Var(ident),
                clock,
            };
            while v.len() > 0 {
                let (ck, b) = v.pop().unwrap();
                let mut clock_true = v.clone();
                clock_true.push((ck.clone(), true));
                let clock_true = Clock::Ck(clock_true);
                let mut clock_false = v.clone();
                clock_false.push((ck.clone(), false));
                let clock_false = Clock::Ck(clock_false);
                let clock = Clock::Ck(v.clone());
                let base_expr = match b {
                    true => minils::BaseExpr::Merge(
                        ck.clone(),
                        box expr,
                        box nested_when(pre_var_expr.clone(), clock_false),
                    ),
                    false => minils::BaseExpr::Merge(
                        ck.clone(),
                        box nested_when(pre_var_expr.clone(), clock_true),
                        box expr,
                    ),
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

/// Introduce nested when in an expression that has clock base, to match the given clock
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
            for (ck, b) in v {
                let clock = match expr.clock.clone() {
                    Clock::Const => Clock::Ck(vec![(ck.clone(), b)]),
                    Clock::Ck(mut v) => {
                        v.push((ck.clone(), b));
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
