use crate::ident;
use crate::ident::IdentGenerator;
use crate::minils_ast as minils;
use crate::normalized_ast as norm;

use std::collections::HashMap;

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

fn normalize_eq(idents: &Vec<IdentGenerator>, expr: minils::Expr, node: &mut norm::Node) {
    let typ_ = expr.typ.clone();
    let mut defined_params = vec![];
    let expr_ = match expr.expr {
        minils::BaseExpr::FunCall(fun, params) => {
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
        node.defined_params.insert(new_param, param_type);
    }
    node.eq_list.push(norm::Eq {
        typ: typ_,
        eq: expr_,
    });
}

fn normalize_ca(ident: &IdentGenerator, expr: minils::Expr, node: &mut norm::Node) -> norm::ExprCA {
    assert!(expr.typ.len() == 1);
    let typ_ = expr.typ[0].clone();
    let expr_ = match expr.expr {
        minils::BaseExpr::FunCall(_, _) | minils::BaseExpr::Fby(_, _) => {
            let new_ident = ident.new_ident();
            normalize_eq(&vec![new_ident.clone()], expr.clone(), node);
            return norm::ExprCA::new_var(new_ident.get_ident(), typ_);
        }
        minils::BaseExpr::IfThenElse(box cond, box expr_true, box expr_false) => {
            let new_ident = ident.new_ident();
            normalize_eq(&vec![new_ident.clone()], cond, node);
            let expr_true = normalize_ca(ident, expr_true, node);
            let expr_false = normalize_ca(ident, expr_false, node);
            norm::ExprCABase::Merge(new_ident.get_ident(), box expr_true, box expr_false)
        }
        _ => {
            let expr_a = normalize_a(ident, expr, node);
            norm::ExprCABase::ExprA(box expr_a)
        }
    };
    norm::ExprCA {
        typ: typ_,
        expr: expr_,
    }
}

fn normalize_a(ident: &IdentGenerator, expr: minils::Expr, node: &mut norm::Node) -> norm::ExprA {
    assert!(expr.typ.len() == 1);
    let typ_ = expr.typ[0].clone();
    let expr_ = match expr.expr {
        minils::BaseExpr::FunCall(_, _)
        | minils::BaseExpr::Fby(_, _)
        | minils::BaseExpr::IfThenElse(_, _, _) => {
            let new_ident = ident.new_ident();
            normalize_eq(&vec![new_ident.clone()], expr.clone(), node);
            norm::ExprABase::Var(new_ident.get_ident())
        }
        minils::BaseExpr::Value(v) => norm::ExprABase::Value(v),
        minils::BaseExpr::Var(s) => norm::ExprABase::Var(s),
        minils::BaseExpr::UnOp(op, box expr) => {
            let expr = normalize_a(ident, expr, node);
            norm::ExprABase::UnOp(op, box expr)
        }
        minils::BaseExpr::BinOp(op, box lhs, box rhs) => {
            let lhs = normalize_a(ident, lhs, node);
            let rhs = normalize_a(ident, rhs, node);
            norm::ExprABase::BinOp(op, box lhs, box rhs)
        }
    };
    norm::ExprA {
        typ: typ_,
        expr: expr_,
    }
}
