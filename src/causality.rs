use crate::typer::typed_ast::BaseExpr::*;
use crate::typer::typed_ast::{Expr, Node};

use petgraph::graphmap::GraphMap;
use std::collections::HashSet;

pub fn check_causality(nodes: &Vec<Node>) -> bool {
    for node in nodes {
        if !check_multiple_definition(node) {
            return false;
        }
        if !check_causality_node(node) {
            return false;
        }
    }
    true
}

fn check_multiple_definition(node: &Node) -> bool {
    let mut set = HashSet::new();
    for eq in &node.eq_list {
        for ident in &eq.0 {
            if set.get(ident).is_some() {
                return false;
            }
            set.insert(ident);
        }
    }
    true
}

fn check_causality_node(node: &Node) -> bool {
    let mut causality_graph = GraphMap::<&str, (), petgraph::Directed>::new();
    for param in &node.out_params {
        causality_graph.add_node(&param.0);
    }
    for param in &node.local_params {
        causality_graph.add_node(&param.0);
    }
    for eq in &node.eq_list {
        let deps = get_var_deps(&eq.1, node);
        for ident in &eq.0 {
            for dep in &deps {
                causality_graph.add_edge(ident, dep, ());
            }
        }
    }
    !petgraph::algo::is_cyclic_directed(&causality_graph)
}

fn get_var_deps<'a>(expr: &'a Expr, node: &'a Node) -> Vec<&'a str> {
    match &expr.expr {
        Value(_) | Pre(_) => vec![],
        UnOp(_, box e) => get_var_deps(&e, node),
        BinOp(_, box e1, box e2) => {
            let mut v = get_var_deps(&e1, node);
            v.append(&mut get_var_deps(&e2, node));
            v
        }
        Arrow(box e1, box e2) => {
            let mut v = get_var_deps(&e1, node);
            v.append(&mut get_var_deps(&e2, node));
            v
        }
        IfThenElse(box e1, box e2, box e3) => {
            let mut v = get_var_deps(&e1, node);
            v.append(&mut get_var_deps(&e2, node));
            v.append(&mut get_var_deps(&e3, node));
            v
        }
        Var(s) => {
            let t = node.in_params.iter().find(|param| *s == param.0);
            if t.is_some() {
                vec![]
            } else {
                vec![s]
            }
        }
        FunCall(_, exprs) => {
            let mut v = vec![];
            for expr in exprs {
                v.append(&mut get_var_deps(&expr, node));
            }
            v
        }
    }
}
