use crate::lucy::ast::Expr::*;
use crate::lucy::ast::{Expr, Node};

use petgraph::graphmap::GraphMap;
use std::collections::HashSet;

pub fn schedule(nodes: &mut Vec<Node>) -> bool {
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

fn check_causality_node(node: &mut Node) -> bool {
    let mut causality_graph = GraphMap::<usize, (), petgraph::Directed>::new();
    for i in 0..node.eq_list.len() {
        causality_graph.add_node(i);
    }
    let var_dependencies: Vec<Vec<&str>> = node
        .eq_list
        .iter()
        .map(|(_, e)| get_var_deps(e, node))
        .collect();
    let defined_vars: Vec<&Vec<String>> = node.eq_list.iter().map(|(v, _)| v).collect();
    for i in 0..node.eq_list.len() {
        for j in 0..node.eq_list.len() {
            for defined_var in defined_vars[i] {
                for var in &var_dependencies[j] {
                    if var == defined_var {
                        causality_graph.add_edge(i, j, ());
                    }
                }
            }
        }
    }
    let topo_sort = petgraph::algo::toposort(&causality_graph, None);
    if let Ok(topo_sort) = topo_sort {
        node.eq_list = topo_sort
            .into_iter()
            .map(|i| node.eq_list[i].clone())
            .collect();
        true
    } else {
        false
    }
}

fn get_var_deps<'a>(expr: &'a Expr, node: &'a Node) -> Vec<&'a str> {
    match expr {
        Value(_) | Fby(_, _) => vec![],
        UnOp(_, box e) => get_var_deps(&e, node),
        BinOp(_, box e1, box e2) => {
            let mut v = get_var_deps(&e1, node);
            v.append(&mut get_var_deps(&e2, node));
            v
        }
        When(box expr, ck, _) => {
            let mut v = get_var_deps(&expr, node);
            v.push(ck);
            v
        }
        Merge(ck, box e1, box e2) => {
            let mut v = get_var_deps(&e1, node);
            v.append(&mut get_var_deps(&e2, node));
            v.push(ck);
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
