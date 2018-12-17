//! Functions to schedule untyped LucyRS nodes, and check if there
//! is multiple definitions of variables

use crate::lucy::ast::Expr::*;
use crate::lucy::ast::{Expr, Node};

use petgraph::graphmap::GraphMap;
use std::collections::HashMap;
use std::collections::HashSet;

/// Schedule the untyped LucyRS nodes
/// Also, check if there is multiple definitions of variables in nodes
pub fn schedule(nodes: Vec<Node>) -> Result<Vec<Node>, String> {
    let mut nodes = schedule_nodes(nodes)?;
    for node in &mut nodes {
        if !check_multiple_definition(node) {
            return Err(String::from(
                "A variable was defined multiple times in a node",
            ));
        }
        if !check_causality_node(node) {
            return Err(String::from("A node is not causal"));
        }
    }
    Ok(nodes)
}

/// Check if there is multiple definitions of variables
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

fn schedule_nodes(nodes: Vec<Node>) -> Result<Vec<Node>, String> {
    let mut causality_graph = GraphMap::<&str, (), petgraph::Directed>::new();
    let mut nodes_hm = HashMap::<&str, &Node>::new();
    for i in 0..nodes.len() {
        causality_graph.add_node(&nodes[i].name);
        nodes_hm.insert(&nodes[i].name, &nodes[i]);
    }
    for node in &*nodes {
        for (_, expr) in &node.eq_list {
            for called_node in get_node_deps(expr) {
                causality_graph.add_edge(&node.name, called_node, ());
            }
        }
    }
    let topo_sort = petgraph::algo::toposort(&causality_graph, None);
    if let Ok(topo_sort) = topo_sort {
        let nodes = topo_sort
            .into_iter()
            .map(|node| (*nodes_hm.get(&node).unwrap()).clone())
            .collect();
        Ok(nodes)
    } else {
        Err(String::from("There is a cyclic call between the nodes"))
    }
}

fn get_node_deps<'a>(expr: &'a Expr) -> Vec<&'a str> {
    match expr {
        Value(_) | Var(_) | Current(_, _) => vec![],
        Pre(box e) => get_node_deps(&e),
        Fby(_, box e) => get_node_deps(&e),
        UnOp(_, box e) => get_node_deps(&e),
        BinOp(_, box e1, box e2) => {
            let mut v = get_node_deps(&e1);
            v.append(&mut get_node_deps(&e2));
            v
        }
        When(box expr, _, _) => get_node_deps(&expr),
        Merge(ck, box e1, box e2) => {
            let mut v = get_node_deps(&e1);
            v.append(&mut get_node_deps(&e2));
            v.push(ck);
            v
        }
        IfThenElse(box e1, box e2, box e3) => {
            let mut v = get_node_deps(&e1);
            v.append(&mut get_node_deps(&e2));
            v.append(&mut get_node_deps(&e3));
            v
        }
        FunCall(fun, exprs, _) => {
            let mut v = vec![];
            for expr in exprs {
                v.append(&mut get_node_deps(&expr));
            }
            v.push(&fun);
            v
        }
        Arrow(box e1, box e2) => {
            let mut v = get_node_deps(&e1);
            v.append(&mut get_node_deps(&e2));
            v
        }
    }
}

/// Check if the node is causal, and schedule it if it is
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

/// Get the var dependencies of an expression
fn get_var_deps<'a>(expr: &'a Expr, node: &'a Node) -> Vec<&'a str> {
    match expr {
        Value(_) | Fby(_, _) | Pre(_) => vec![],
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
        FunCall(_, exprs, ck) => {
            let mut v = vec![];
            for expr in exprs {
                v.append(&mut get_var_deps(&expr, node));
            }
            if let Some(ck) = ck {
                v.push(ck);
            }
            v
        }
        Current(s, _) => vec![s],
        Arrow(box e1, box e2) => {
            let mut v = get_var_deps(&e1, node);
            v.append(&mut get_var_deps(&e2, node));
            v
        }
    }
}
