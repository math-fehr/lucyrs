#![feature(box_syntax)]
#![feature(box_patterns)]

use std::env;

pub mod ast;
pub mod lucy;
pub mod ident;
pub mod minils;
pub mod obc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let node_name = &args[2];
    let mut nodes = lucy::parse_file(filename);

    // Causality checking 
    // TODO improve message
    if !lucy::scheduling::schedule(&mut nodes) {
        panic!("Causality not okay!");
    }

    // Typing
    let typed_nodes = lucy::typing::annotate_types(nodes);
    if let Err(message) = typed_nodes {
        panic!("Typing Error: {}", message);
    }
    let typed_nodes = typed_nodes.unwrap();

    let clock_nodes = lucy::type_clock::annotate_clocks(typed_nodes);
    if let Err(message) = clock_nodes {
        panic!("Clock typing error: {}", message);
    }
    let clock_nodes = clock_nodes.unwrap();

    let minils_nodes: Vec<minils::ast::Node> =
        clock_nodes.into_iter().map(lucy::to_minils::to_minils).collect();

    let obc_machines = minils::to_obc(minils_nodes);

    let rust_code = obc::to_rust::obc_to_rust(&obc_machines, node_name);

    println!("{}", rust_code);
}
