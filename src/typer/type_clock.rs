use crate::ast::{BinOp, Clock, Type};
use crate::typer::clock_typed_ast as ck;
use crate::typer::typed_ast as typ;

use std::collections::HashMap;

pub fn annotate_clocks(nodes: Vec<typ::Node>) -> Result<Vec<ck::Node>, String> {
    let mut clock_nodes = vec![];
    for node in nodes {
        clock_nodes.push(annotate_clocks_node(node)?);
    }
    Ok(clock_nodes)
}

fn annotate_clocks_node(node: typ::Node) -> Result<ck::Node, String> {
    let mut variables = HashMap::new();
    for (var, typ) in &node.in_params {
        variables.insert(var.clone(), (typ.clone(), Clock::Ck(HashMap::new())));
    }
    let mut defined_params = node.local_params.clone();
    defined_params.append(&mut node.out_params.clone());
    let mut eq_list = vec![];
    for (vars, expr) in node.eq_list {
        let expr = annotate_expr(expr, &variables)?;
        for var in &vars {
            let typ = defined_params
                .iter()
                .find(|(s, _)| s == var)
                .unwrap()
                .1
                .clone();
            variables.insert(var.clone(), (typ, expr.clock.clone()));
        }
        eq_list.push((vars, expr));
    }
    Ok(ck::Node {
        name: node.name,
        in_params: node.in_params,
        out_params: node.out_params,
        local_params: node.local_params,
        eq_list,
    })
}

fn annotate_expr(
    expr: typ::Expr,
    vars: &HashMap<String, (Type, Clock)>,
) -> Result<ck::Expr, String> {
    let typ = expr.typ;
    let (expr, clock) = match expr.expr {
        typ::BaseExpr::Value(v) => (ck::BaseExpr::Value(v), Clock::Const),
        typ::BaseExpr::UnOp(op, box e) => {
            let e = annotate_expr(e, vars)?;
            let clock = e.clock.clone();
            (ck::BaseExpr::UnOp(op, box e), clock)
        }
        typ::BaseExpr::BinOp(op, box e1, box e2) => annotate_binop(op, e1, e2, vars)?,
        typ::BaseExpr::When(box e, s, b) => annotate_when(e, s, b, vars)?,
        typ::BaseExpr::Merge(ck, box e_t, box e_f) => annotate_merge(ck, e_t, e_f, vars)?,
        typ::BaseExpr::Fby(v, box e) => {
            let e = annotate_expr(e, vars)?;
            let clock = e.clock.clone();
            (ck::BaseExpr::Fby(v, box e), clock)
        }
        typ::BaseExpr::IfThenElse(box cond, box e_t, box e_f) => {
            annotate_ifthenelse(cond, e_t, e_f, vars)?
        }
        typ::BaseExpr::Var(s) => annotate_var(s, vars),
        typ::BaseExpr::FunCall(s, exprs) => annotate_funcall(s, exprs, vars)?,
    };
    Ok(ck::Expr { expr, typ, clock })
}

fn annotate_binop(
    op: BinOp,
    e1: typ::Expr,
    e2: typ::Expr,
    vars: &HashMap<String, (Type, Clock)>,
) -> Result<(ck::BaseExpr, Clock), String> {
    let mut e1 = annotate_expr(e1, vars)?;
    let mut e2 = annotate_expr(e2, vars)?;
    let clock1 = e1.clock.clone();
    let clock2 = e2.clock.clone();
    if !Clock::is_compatible(&e1.clock, &e2.clock) {
        return Err(String::from(
            "The two expressions have incompatible clocks in the binary operation",
        ));
    }
    lower_clock(&mut e1, &clock1);
    lower_clock(&mut e2, &clock2);
    let clock = e1.clock.clone();
    Ok((ck::BaseExpr::BinOp(op, box e1, box e2), clock))
}

fn annotate_when(
    e: typ::Expr,
    s: String,
    b: bool,
    vars: &HashMap<String, (Type, Clock)>,
) -> Result<(ck::BaseExpr, Clock), String> {
    let mut e = annotate_expr(e, vars)?;
    if e.clock == Clock::Const {
        lower_clock(&mut e, &Clock::Ck(HashMap::new()));
    }
    let clock = match e.clock.clone() {
        Clock::Const => unreachable!(),
        Clock::Ck(mut hm) => {
            hm.insert(s.clone(), b);
            Clock::Ck(hm)
        }
    };
    Ok((ck::BaseExpr::When(box e, s, b), clock))
}

fn annotate_merge(
    ck: String,
    e_t: typ::Expr,
    e_f: typ::Expr,
    vars: &HashMap<String, (Type, Clock)>,
) -> Result<(ck::BaseExpr, Clock), String> {
    let e_t = annotate_expr(e_t, vars)?;
    let e_f = annotate_expr(e_f, vars)?;
    let mut hm_t = match &e_t.clock {
        Clock::Const => {
            return Err(String::from(
                "Left expression in a merge should have a clock on true(merge clock)",
            ));
        }
        Clock::Ck(hm) => {
            if let Some(true) = hm.get(&ck) {
                hm.clone()
            } else {
                return Err(String::from(
                    "Left expression in a merge should have a clock on true(merge clock)",
                ));
            }
        }
    };
    let mut hm_f = match &e_f.clock {
        Clock::Const => {
            return Err(String::from(
                "Right expression in a merge should have a clock on false(merge clock)",
            ));
        }
        Clock::Ck(hm) => {
            if let Some(false) = hm.get(&ck) {
                hm.clone()
            } else {
                return Err(String::from(
                    "Right expression in a merge should have a clock on false(merge clock)",
                ));
            }
        }
    };
    hm_t.remove(&ck);
    hm_f.remove(&ck);
    if hm_t != hm_f {
        return Err(String::from("Both expressions in a merge construct should have the same clock (modulo the merge clock)"));
    }
    let ck_clock = &vars.get(&ck).unwrap().1;
    if !Clock::is_compatible(ck_clock, &Clock::Ck(hm_f)) {
        return Err(String::from(
            "Expressions in merge construct should have clock compatible with the merge clock.",
        ));
    }
    Ok((ck::BaseExpr::Merge(ck, box e_t, box e_f), Clock::Ck(hm_t)))
}

fn annotate_ifthenelse(
    cond: typ::Expr,
    e_t: typ::Expr,
    e_f: typ::Expr,
    vars: &HashMap<String, (Type, Clock)>,
) -> Result<(ck::BaseExpr, Clock), String> {
    let mut cond = annotate_expr(cond, vars)?;
    let mut e_t = annotate_expr(e_t, vars)?;
    let mut e_f = annotate_expr(e_f, vars)?;
    if !Clock::is_compatible(&e_t.clock, &cond.clock)
        || !Clock::is_compatible(&e_f.clock, &cond.clock)
    {
        return Err(String::from(
            "Expressions in a if construct should have compatible clocks",
        ));
    }
    lower_clock(&mut cond, &e_t.clock);
    lower_clock(&mut cond, &e_f.clock);
    lower_clock(&mut e_t, &cond.clock);
    lower_clock(&mut e_t, &e_f.clock);
    lower_clock(&mut e_f, &cond.clock);
    lower_clock(&mut e_t, &e_f.clock);
    let clock = cond.clock.clone();
    Ok((ck::BaseExpr::IfThenElse(box cond, box e_t, box e_f), clock))
}

fn annotate_var(var: String, vars: &HashMap<String, (Type, Clock)>) -> (ck::BaseExpr, Clock) {
    match vars.get(&var) {
        None => (ck::BaseExpr::Var(var), Clock::Const),
        Some(ck) => (ck::BaseExpr::Var(var), ck.1.clone()),
    }
}

fn annotate_funcall(
    fun: String,
    exprs: Vec<typ::Expr>,
    vars: &HashMap<String, (Type, Clock)>,
) -> Result<(ck::BaseExpr, Clock), String> {
    let mut exprs_ = vec![];
    for expr in exprs {
        exprs_.push(annotate_expr(expr, vars)?);
    }
    for i in 0..exprs_.len() {
        for j in 0..exprs_.len() {
            if !Clock::is_compatible(&exprs_[i].clock, &exprs_[j].clock) {
                return Err(String::from(
                    "Parameters of node call should have the same clock",
                ));
            }
            let clock_j = exprs_[j].clock.clone();
            let clock_i = exprs_[i].clock.clone();
            lower_clock(&mut exprs_[i], &clock_j);
            lower_clock(&mut exprs_[j], &clock_i);
        }
    }
    let clock = exprs_[0].clock.clone();
    Ok((ck::BaseExpr::FunCall(fun, exprs_), clock))
}

fn lower_clock(expr: &mut ck::Expr, clock: &Clock) {
    if let Clock::Const = clock {
        return;
    }
    if let Clock::Ck(_) = &expr.clock {
        assert!(&expr.clock == clock);
    }
    expr.clock = clock.clone();
    match &mut expr.expr {
        ck::BaseExpr::UnOp(_, box e) => lower_clock(e, clock),
        ck::BaseExpr::BinOp(_, box e1, box e2) => {
            lower_clock(e1, clock);
            lower_clock(e2, clock);
        }
        ck::BaseExpr::Merge(_, _, _) => unreachable!(),
        ck::BaseExpr::Fby(_, box e) => lower_clock(e, clock),
        ck::BaseExpr::IfThenElse(box e1, box e2, box e3) => {
            lower_clock(e1, clock);
            lower_clock(e2, clock);
            lower_clock(e3, clock);
        }
        ck::BaseExpr::FunCall(_, v) => {
            v.iter_mut().for_each(|e| lower_clock(e, clock));
        }
        _ => (),
    }
}
