pub mod scheduling;
pub mod to_minils;
pub mod grammar;
pub mod ast;
pub mod typed_ast;
pub mod clock_typed_ast;
pub mod type_clock;
pub mod typing;

use std::fs::File;
use std::io::Read;

pub fn parse_file(filename: &str) -> Vec<ast::Node> {
    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect(&("Error while loading file ".to_owned() + filename));
    grammar::FileParser::new().parse(&contents).unwrap()
}
