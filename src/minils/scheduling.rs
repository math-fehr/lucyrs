//! Schedule normalized minils nodes

use crate::minils::normalized_ast::*;

use petgraph::graphmap::GraphMap;
use std::collections::HashSet;

/// Schedule normalized minils nodes
pub fn schedule(mut node: Node) -> Node {
    // When this function is called, the program should
    // not define a variable twice
    assert!(check_multiple_definition(&node));

    schedule_node(&mut node);
    node
}

/// Check if a variable is defined multiple times
fn check_multiple_definition(node: &Node) -> bool {
    let mut set = HashSet::new();
    for eq in &node.eq_list {
        for ident in get_defined_vars(eq) {
            if set.get(ident).is_some() {
                return false;
            }
            set.insert(ident);
        }
    }
    true
}

/// Schedule a normalized minils node
fn schedule_node(node: &mut Node) {
    let mut causality_graph = GraphMap::<usize, (), petgraph::Directed>::new();
    for i in 0..node.eq_list.len() {
        causality_graph.add_node(i);
    }
    let defined_vars: Vec<Vec<&str>> = node.eq_list.iter().map(get_defined_vars).collect();
    let var_dependencies: Vec<Vec<&str>> =
        node.eq_list.iter().map(get_var_dependencies_eq).collect();
    for i in 0..node.eq_list.len() {
        for j in 0..node.eq_list.len() {
            for defined_var in &defined_vars[i] {
                for var in &var_dependencies[j] {
                    if var == defined_var {
                        causality_graph.add_edge(i, j, ());
                    }
                }
            }
        }
    }
    let topo_sort = petgraph::algo::toposort(&causality_graph, None);
    assert!(topo_sort.is_ok());
    let topo_sort = topo_sort.unwrap();
    node.eq_list = topo_sort
        .into_iter()
        .map(|i| node.eq_list[i].clone())
        .collect();
}

/// Get the defined variables in a minils normalized eq
fn get_defined_vars(eq: &Eq) -> Vec<&str> {
    match &eq.eq {
        ExprEqBase::Fby(_, _, _) => vec![],
        ExprEqBase::FunCall(v, _, _, _) => v.iter().map(|s| s.as_str()).collect(),
        ExprEqBase::ExprCA(s, _) => vec![&s],
    }
}

/// Normalize an assignment into an eq normalized minils node
fn get_var_dependencies_eq(eq: &Eq) -> Vec<&str> {
    match &eq.eq {
        ExprEqBase::Fby(_, _, box a) => get_var_dependencies_a(a),
        ExprEqBase::FunCall(_, _, params, r) => {
            let mut v = params
                .iter()
                .map(get_var_dependencies_a)
                .flatten()
                .collect::<Vec<&str>>();
            if let Some(r) = r {
                v.push(&r);
            }
            v
        }
        ExprEqBase::ExprCA(_, box ca) => get_var_dependencies_ca(&ca),
    }
}

/// Normalize an assignment into an eq normalized ca
fn get_var_dependencies_ca(ca: &ExprCA) -> Vec<&str> {
    match &ca.expr {
        ExprCABase::Merge(s, box ca_1, box ca_2) => {
            let mut vars_1 = get_var_dependencies_ca(&ca_1);
            let mut vars_2 = get_var_dependencies_ca(&ca_2);
            vars_1.append(&mut vars_2);
            vars_1.push(s);
            vars_1
        }
        ExprCABase::ExprA(box a) => get_var_dependencies_a(&a),
    }
}

/// Normalize an assignment into an eq normalized a
fn get_var_dependencies_a(a: &ExprA) -> Vec<&str> {
    match &a.expr {
        ExprABase::Value(_) => vec![],
        ExprABase::Var(s) => vec![&s],
        ExprABase::UnOp(_, box a) => get_var_dependencies_a(&a),
        ExprABase::BinOp(_, box a_1, box a_2) => {
            let mut vars_1 = get_var_dependencies_a(&a_1);
            let mut vars_2 = get_var_dependencies_a(&a_2);
            vars_1.append(&mut vars_2);
            vars_1
        }
        ExprABase::When(box a, s, _) => {
            let mut vars = get_var_dependencies_a(&a);
            vars.push(s);
            vars
        }
    }
}
