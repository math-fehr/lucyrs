pub mod grammar;
pub mod ast;

use std::fs::File;
use std::io::Read;

pub fn parse_file(filename: &str) -> Vec<ast::Node> {
    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect(&("Error while loading file ".to_owned() + filename));
    grammar::FileParser::new().parse(&contents).unwrap()
}
