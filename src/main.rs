#![feature(box_syntax)]

use std::env;

pub mod parser;
pub mod ast;
pub mod typer;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let nodes = parser::parse_file(filename);
    let typed_nodes = typer::annotate_types(nodes);

    print!("{:?}", typed_nodes)
}
