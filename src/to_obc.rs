use crate::ast::Value;
use crate::ident;
use crate::normalized_ast as norm;
use crate::obc_ast as obc;

use std::collections::HashMap;

pub fn to_obc(node: norm::Node) -> obc::Machine {
    let memory = get_memories(&node);
    let name = node.name;
    let step_inputs = node.in_params;
    let step_returns = node.out_params;
    let step_vars = node.defined_params;
    let mut temp_instances = HashMap::new();
    let mut step_stmts = vec![];
    for eq in node.eq_list {
        eq_to_obc(
            eq,
            &memory,
            &mut temp_instances,
            &mut step_stmts,
        );
    }
    let mut instances = HashMap::new();
    for (s, i) in temp_instances {
        for j in 0..i {
            instances.insert(ident::gen_ident(s.clone(), j), s.clone());
        }
    }
    for (s, _) in &memory {
        step_stmts.push(obc::Stmt::StateAssignment(s.clone(), obc::Expr::Var(s.clone())));
    }
    obc::Machine {
        name,
        memory,
        instances,
        step_inputs,
        step_returns,
        step_vars,
        step_stmts,
    }
}

pub fn get_memories(node: &norm::Node) -> HashMap<String, Value> {
    let mut memory = HashMap::new();
    for eq in &node.eq_list {
        match &eq.eq {
            norm::ExprEqBase::Fby(s, v, _) => {
                memory.insert(s.clone(), v.clone());
            },
            _ => ()
        }
    }
    memory
}

pub fn eq_to_obc(
    eq: norm::Eq,
    memory: &HashMap<String, Value>,
    instances: &mut HashMap<String, u32>,
    step_stmts: &mut Vec<obc::Stmt>,
) {
    match eq.eq {
        norm::ExprEqBase::Fby(s, _, box expr) => {
            let expr = a_to_obc(expr, memory, instances, step_stmts);
            step_stmts.push(obc::Stmt::Assignment(s, expr));
        }
        norm::ExprEqBase::FunCall(pat, fun, exprs) => {
            let n_fun = if let Some(n) = instances.get(&fun) {
                n + 1
            } else {
                1
            };
            instances.insert(fun.clone(), n_fun);
            let ident = ident::gen_ident(fun, n_fun - 1);
            let exprs = exprs
                .into_iter()
                .map(|e| a_to_obc(e, memory, instances, step_stmts))
                .collect();
            step_stmts.push(obc::Stmt::Step(pat, ident, exprs));
        }
        norm::ExprEqBase::ExprCA(s, box expr) => {
            let stmt = ca_to_obc(s, expr, memory, instances, step_stmts);
            step_stmts.push(stmt);
        }
    }
}

pub fn ca_to_obc(
    lhs: String,
    expr: norm::ExprCA,
    memory: &HashMap<String, Value>,
    instances: &mut HashMap<String, u32>,
    step_stmts: &mut Vec<obc::Stmt>,
) -> obc::Stmt {
    match expr.expr {
        norm::ExprCABase::Merge(x, box expr_true, box expr_false) => {
            let expr_true = ca_to_obc(
                lhs.clone(),
                expr_true,
                memory,
                instances,
                step_stmts,
            );
            let expr_false = ca_to_obc(
                lhs,
                expr_false,
                memory,
                instances,
                step_stmts,
            );
            obc::Stmt::Control(x, vec![expr_true], vec![expr_false])
        }
        norm::ExprCABase::ExprA(box expr) => {
            let expr = a_to_obc(expr, memory, instances, step_stmts);
            obc::Stmt::Assignment(lhs, expr)
        }
    }
}

pub fn a_to_obc(
    expr: norm::ExprA,
    memory: &HashMap<String, Value>,
    instances: &mut HashMap<String, u32>,
    step_stmts: &mut Vec<obc::Stmt>,
) -> obc::Expr {
    match expr.expr {
        norm::ExprABase::Value(v) => obc::Expr::Value(v),
        norm::ExprABase::Var(s) => {
            if memory.get(&s).is_some() {
                obc::Expr::State(s)
            } else {
                obc::Expr::Var(s)
            }
        }
        norm::ExprABase::UnOp(op, box expr) => {
            let expr = a_to_obc(expr, memory, instances, step_stmts);
            obc::Expr::UnOp(op, box expr)
        }
        norm::ExprABase::BinOp(op, box lhs, box rhs) => {
            let lhs = a_to_obc(lhs, memory, instances, step_stmts);
            let rhs = a_to_obc(rhs, memory, instances, step_stmts);
            obc::Expr::BinOp(op, box lhs, box rhs)
        }
    }
}
