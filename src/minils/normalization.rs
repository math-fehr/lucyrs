//! Normalize a minils program. This is simplifying the minils translation
//! into obc. In this module, many identifiers are created. The IdentGenerator
//! struct is used to generate new identifiers that will not collide with existing
//! identifiers.

use crate::ast::Clock;
use crate::ident;
use crate::ident::IdentGenerator;
use crate::minils::ast as minils;
use crate::minils::normalized_ast as norm;

use std::collections::HashMap;

/// Normalize a minils node
pub fn normalize(node: minils::Node) -> norm::Node {
    let name = ident::gen_ident(node.name, 0);
    let add_suffix_fst = |(ident, t)| (ident::gen_ident(ident, 0), t);
    let in_params = node.in_params.into_iter().map(add_suffix_fst).collect();
    let out_params = node.out_params.into_iter().map(add_suffix_fst).collect();
    let mut normalized_node = norm::Node {
        name,
        in_params,
        out_params,
        defined_params: HashMap::new(),
        eq_list: vec![],
    };
    for (idents, expr) in node.eq_list {
        let idents = idents.into_iter().map(|s| IdentGenerator::new(s)).collect();
        normalize_eq(&idents, expr, &mut normalized_node);
    }
    normalized_node
}

/// Normalize an assignment into an eq normalized minils node
fn normalize_eq(idents: &Vec<IdentGenerator>, expr: minils::Expr, node: &mut norm::Node) {
    let typ_ = expr.typ.clone();
    let clock = gen_clock_ident(expr.clock.clone());
    let mut defined_params;
    let expr_ = match expr.expr {
        minils::BaseExpr::FunCall(fun, params, mut reset) => {
            if let Some(s) = &mut reset {
                *s = ident::gen_ident(s.clone(), 0);
            }
            let params = params
                .into_iter()
                .map(|param| normalize_a(&idents[0], param, node))
                .collect();
            let defined_params_names = idents.into_iter().map(|i| i.get_ident());
            defined_params = defined_params_names.clone().zip(typ_.clone()).collect();
            norm::ExprEqBase::FunCall(
                defined_params_names.collect(),
                ident::gen_ident(fun, 0),
                params,
                reset,
            )
        }
        minils::BaseExpr::Fby(v, box expr) => {
            assert!(idents.len() == 1);
            defined_params = vec![(idents[0].get_ident(), typ_[0].clone())];
            norm::ExprEqBase::Fby(
                defined_params[0].0.clone(),
                v,
                box normalize_a(&idents[0], expr, node),
            )
        }
        _ => {
            assert!(idents.len() == 1);
            defined_params = vec![(idents[0].get_ident(), typ_[0].clone())];
            let expr_ca = normalize_ca(&idents[0], expr, node);
            norm::ExprEqBase::ExprCA(defined_params[0].0.clone(), box expr_ca)
        }
    };
    for (new_param, param_type) in defined_params {
        node.defined_params
            .insert(new_param, (param_type, clock.clone()));
    }
    node.eq_list.push(norm::Eq {
        typ: typ_,
        clock,
        eq: expr_,
    });
}

/// Normalize an expression into an ca normalized minils node
fn normalize_ca(ident: &IdentGenerator, expr: minils::Expr, node: &mut norm::Node) -> norm::ExprCA {
    assert!(expr.typ.len() == 1);
    let typ_ = expr.typ[0].clone();
    let clock = gen_clock_ident(expr.clock.clone());
    let expr_ = match expr.expr {
        minils::BaseExpr::FunCall(_, _, _) | minils::BaseExpr::Fby(_, _) => {
            let new_ident = ident.new_ident();
            normalize_eq(&vec![new_ident.clone()], expr.clone(), node);
            return norm::ExprCA::new_var(new_ident.get_ident(), typ_, clock.clone());
        }
        minils::BaseExpr::Merge(ck, box e_t, box e_f) => {
            let e_t = normalize_ca(ident, e_t, node);
            let e_f = normalize_ca(ident, e_f, node);
            norm::ExprCABase::Merge(ident::gen_ident(ck, 0), box e_t, box e_f)
        }
        _ => {
            let expr_a = normalize_a(ident, expr, node);
            norm::ExprCABase::ExprA(box expr_a)
        }
    };
    norm::ExprCA {
        typ: typ_,
        clock,
        expr: expr_,
    }
}

/// Normalize an assignment into an a normalized minils node
fn normalize_a(ident: &IdentGenerator, expr: minils::Expr, node: &mut norm::Node) -> norm::ExprA {
    assert!(expr.typ.len() == 1);
    let typ_ = expr.typ[0].clone();
    let clock = gen_clock_ident(expr.clock.clone());
    let expr_ = match expr.expr {
        minils::BaseExpr::FunCall(_, _, _)
        | minils::BaseExpr::Fby(_, _)
        | minils::BaseExpr::Merge(_, _, _) => {
            let new_ident = ident.new_ident();
            normalize_eq(&vec![new_ident.clone()], expr.clone(), node);
            norm::ExprABase::Var(new_ident.get_ident())
        }
        minils::BaseExpr::Value(v) => norm::ExprABase::Value(v),
        minils::BaseExpr::Var(s) => norm::ExprABase::Var(ident::gen_ident(s, 0)),
        minils::BaseExpr::UnOp(op, box expr) => {
            let expr = normalize_a(ident, expr, node);
            norm::ExprABase::UnOp(op, box expr)
        }
        minils::BaseExpr::BinOp(op, box lhs, box rhs) => {
            let lhs = normalize_a(ident, lhs, node);
            let rhs = normalize_a(ident, rhs, node);
            norm::ExprABase::BinOp(op, box lhs, box rhs)
        }
        minils::BaseExpr::When(box e, ck, b) => {
            let e = normalize_a(ident, e, node);
            norm::ExprABase::When(box e, ck, b)
        }
    };
    norm::ExprA {
        typ: typ_,
        clock,
        expr: expr_,
    }
}

/// Generate identifiers for clocks. This is required because in this module,
/// all identifiers change.
fn gen_clock_ident(ck: Clock) -> Clock {
    match ck {
        Clock::Const => Clock::Const,
        Clock::Ck(v) => {
            let v = v
                .into_iter()
                .map(|(ident, b)| (ident::gen_ident(ident, 0), b))
                .collect();
            Clock::Ck(v)
        }
    }
}
