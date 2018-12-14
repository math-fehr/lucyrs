#![feature(box_syntax)]
#![feature(box_patterns)]

use std::env;

pub mod ast;
pub mod causality;
pub mod ident;
pub mod minils_ast;
pub mod normalization;
pub mod normalized_ast;
pub mod obc_ast;
pub mod obc_to_rust;
pub mod parser;
pub mod scheduling;
pub mod to_minils;
pub mod to_obc;
pub mod typer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let node_name = &args[2];
    let mut nodes = parser::parse_file(filename);

    // Causality checking
    // TODO improve message
    if !causality::schedule(&mut nodes) {
        panic!("Causality not okay!");
    }

    // Typing
    let typed_nodes = typer::annotate_types(nodes);
    if let Err(message) = typed_nodes {
        panic!("Typing Error: {}", message);
    }
    let typed_nodes = typed_nodes.unwrap();

    let clock_nodes = typer::type_clock::annotate_clocks(typed_nodes.clone());
    if let Err(message) = clock_nodes {
        panic!("Clock typing error: {}", message);
    }
    let clock_nodes = clock_nodes.unwrap();

    let minils_nodes: Vec<minils_ast::Node> =
        typed_nodes.into_iter().map(to_minils::to_minils).collect();

    let normalized_nodes: Vec<normalized_ast::Node> = minils_nodes
        .into_iter()
        .map(normalization::normalize)
        .collect();

    let scheduled_nodes: Vec<normalized_ast::Node> = normalized_nodes
        .into_iter()
        .map(scheduling::schedule)
        .collect();

    let obc_machines: Vec<obc_ast::Machine> =
        scheduled_nodes.into_iter().map(to_obc::to_obc).collect();

    let rust_code = obc_to_rust::obc_to_rust(&obc_machines, node_name);

    println!("{}", rust_code);
}
