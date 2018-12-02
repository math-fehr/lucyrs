#![feature(box_syntax)]
#![feature(box_patterns)]

use std::env;

pub mod parser;
pub mod ast;
pub mod typer;
pub mod causality;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let nodes = parser::parse_file(filename);

    // Typing
    let typed_nodes = typer::annotate_types(nodes);
    if let Err(message) = typed_nodes {
        print!("Typing Error: {}", message);
        return;
    }
    let typed_nodes = typed_nodes.unwrap();

    // Causality checking
    // TODO improve message
    if causality::check_causality(&typed_nodes) {
        print!("Causality okay!");
    } else {
        print!("Causality not okay!");
    }
}
