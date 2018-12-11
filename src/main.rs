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
pub mod parser;
pub mod scheduling;
pub mod to_minils;
pub mod to_obc;
pub mod typer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let nodes = parser::parse_file(filename);

    // Typing
    let typed_nodes = typer::annotate_types(nodes);
    if let Err(message) = typed_nodes {
        println!("Typing Error: {}", message);
        return;
    }
    let typed_nodes = typed_nodes.unwrap();

    // Causality checking
    // TODO improve message
    if causality::check_causality(&typed_nodes) {
        println!("Causality okay!");
    } else {
        println!("Causality not okay!");
        return;
    }

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

    println!("{:#?}", obc_machines);
}
