use crate::ast::{Clock, Type, Value};
use crate::ident;
use crate::minils::normalized_ast as norm;
use crate::obc::ast as obc;

use std::collections::HashMap;

pub fn to_obc(node: norm::Node) -> obc::Machine {
    let memory = get_memories(&node);
    let name = node.name;
    let step_inputs = node.in_params;
    let mut step_returns = node.out_params;
    let mut temp_instances = HashMap::new();
    let mut step_stmts = vec![];
    for eq in node.eq_list {
        eq_to_obc(
            eq,
            &memory,
            &mut temp_instances,
            &mut step_stmts,
            &node.defined_params,
        );
    }
    let mut step_vars = HashMap::new();
    for (s, (t, _)) in node.defined_params {
        step_vars.insert(s, t);
    }
    let mut instances = HashMap::new();
    for (s, i) in temp_instances {
        for j in 0..i {
            instances.insert(ident::gen_ident(s.clone(), j), s.clone());
        }
    }
    for (s, t) in &mut step_returns {
        if memory.get(s).is_some() {
            let s_result = s.clone() + "_result";
            step_vars.insert(s_result.clone(), t.clone());
            step_stmts.push(obc::Stmt::Assignment(
                s_result.clone(),
                obc::Expr::State(s.clone()),
            ));
            *s = s_result;
        }
    }
    for (s, (_, c)) in &memory {
        let stmt = obc::Stmt::StateAssignment(s.clone(), obc::Expr::Var(s.clone()));
        step_stmts.push(add_control(stmt, c.clone()));
    }
    let mut memory_without_clocks = HashMap::new();
    for (s, (t, _)) in memory {
        memory_without_clocks.insert(s, t);
    }
    obc::Machine {
        name,
        memory: memory_without_clocks,
        instances,
        step_inputs,
        step_returns,
        step_vars,
        step_stmts,
    }
}

fn get_memories(node: &norm::Node) -> HashMap<String, (Value, Clock)> {
    let mut memory = HashMap::new();
    for eq in &node.eq_list {
        match &eq.eq {
            norm::ExprEqBase::Fby(s, v, _) => {
                memory.insert(s.clone(), (v.clone(), eq.clock.clone()));
            }
            _ => (),
        }
    }
    memory
}

fn eq_to_obc(
    eq: norm::Eq,
    memory: &HashMap<String, (Value, Clock)>,
    instances: &mut HashMap<String, u32>,
    step_stmts: &mut Vec<obc::Stmt>,
    step_vars: &HashMap<String, (Type, Clock)>,
) {
    match eq.eq {
        norm::ExprEqBase::Fby(s, _, box expr) => {
            let expr = a_to_obc(expr, memory, instances, step_stmts);
            let stmt = add_control(obc::Stmt::Assignment(s, expr), eq.clock);
            step_stmts.push(stmt);
        }
        norm::ExprEqBase::FunCall(pat, fun, exprs, r) => {
            let n_fun = if let Some(n) = instances.get(&fun) {
                n + 1
            } else {
                1
            };
            instances.insert(fun.clone(), n_fun);
            let ident = ident::gen_ident(fun, n_fun - 1);
            if let Some(r) = r {
                let mut stmt = obc::Stmt::Reset(ident.clone());
                stmt = obc::Stmt::Control(r.clone(), vec![stmt], vec![]);
                let r_clock = match step_vars.get(&r) {
                    Some((_, ck)) => ck.clone(),
                    None => Clock::Const,
                };
                stmt = add_control(stmt, r_clock);
                step_stmts.push(stmt);
            }
            let exprs = exprs
                .into_iter()
                .map(|e| a_to_obc(e, memory, instances, step_stmts))
                .collect();
            let stmt = add_control(obc::Stmt::Step(pat, ident, exprs), eq.clock);
            step_stmts.push(stmt);
        }
        norm::ExprEqBase::ExprCA(s, box expr) => {
            let mut stmt = ca_to_obc(s, expr, memory, instances, step_stmts);
            stmt = add_control(stmt, eq.clock);
            step_stmts.push(stmt);
        }
    };
}

fn add_control(mut stmt: obc::Stmt, clock: Clock) -> obc::Stmt {
    match clock {
        Clock::Const => (),
        Clock::Ck(hm) => {
            for (ck, b) in hm {
                if b {
                    stmt = obc::Stmt::Control(ck, vec![stmt], vec![]);
                } else {
                    stmt = obc::Stmt::Control(ck, vec![], vec![stmt]);
                }
            }
        }
    }
    stmt
}

fn ca_to_obc(
    lhs: String,
    expr: norm::ExprCA,
    memory: &HashMap<String, (Value, Clock)>,
    instances: &mut HashMap<String, u32>,
    step_stmts: &mut Vec<obc::Stmt>,
) -> obc::Stmt {
    match expr.expr {
        norm::ExprCABase::Merge(x, box expr_true, box expr_false) => {
            let expr_true = ca_to_obc(lhs.clone(), expr_true, memory, instances, step_stmts);
            let expr_false = ca_to_obc(lhs, expr_false, memory, instances, step_stmts);
            obc::Stmt::Control(x, vec![expr_true], vec![expr_false])
        }
        norm::ExprCABase::ExprA(box expr) => {
            let expr = a_to_obc(expr, memory, instances, step_stmts);
            obc::Stmt::Assignment(lhs, expr)
        }
    }
}

fn a_to_obc(
    expr: norm::ExprA,
    memory: &HashMap<String, (Value, Clock)>,
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
        norm::ExprABase::When(box e, _, _) => a_to_obc(e, memory, instances, step_stmts),
    }
}
