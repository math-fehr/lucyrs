#![feature(box_syntax)]

use std::env;

pub mod parser;
pub mod ast;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let nodes = parser::parse_file(filename);
    print!("{:?}", nodes)
}
