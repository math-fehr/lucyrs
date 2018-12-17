//! This module contains the functions to parse a LucyRS file, and to
//! translate it into typed LucyRS AST, then into minils AST.

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

use crate::minils::ast as mls;
use self::clock_typed_ast as typ;

/// Parse a LucyRS file and return the node list
pub fn parse_file(filename: &str) -> Vec<ast::Node> {
    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect(&("Error while loading file ".to_owned() + filename));
    grammar::FileParser::new().parse(&contents).unwrap()
}

/// Type the LucyRS nodes
pub fn type_nodes(mut nodes: Vec<ast::Node>) -> Vec<typ::Node> {
    if !scheduling::schedule(&mut nodes) {
        panic!("The program is not causal");
    }

    let typed_nodes = typing::annotate_types(nodes);
    if let Err(message) = typed_nodes {
        panic!("Typing Error: {}", message);
    }
    let typed_nodes = typed_nodes.unwrap();

    let clock_nodes = type_clock::annotate_clocks(typed_nodes);
    if let Err(message) = clock_nodes {
        panic!("Clock typing error: {}", message);
    }
    clock_nodes.unwrap()
}

/// Translate typed LucyRS nodes into minils
pub fn to_minils(nodes: Vec<typ::Node>) -> Vec<mls::Node> {
    nodes.into_iter().map(to_minils::to_minils).collect()
}
