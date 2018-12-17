#![feature(box_syntax)]
#![feature(box_patterns)]

use std::env;

pub mod ast;
pub mod ident;
pub mod lucy;
pub mod minils;
pub mod obc;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("The first argument of the program should be the file path of the lucyrs file, and the second argument should be the entry node.");
    }
    let filename = &args[1];
    let node_name = &args[2];

    // Parse the lucyrs file
    let nodes = lucy::parse_file(filename);

    // Type the lucy nodes
    let typed_nodes = lucy::type_nodes(nodes);

    // Compile it into minils AST
    let minils_nodes = lucy::to_minils(typed_nodes);

    // Compile it into obc AST
    let obc_machines = minils::to_obc(minils_nodes);

    // Compile it into rust file
    let rust_code = obc::to_rust(obc_machines, node_name);

    // Output the file
    println!("{}", rust_code);
}
