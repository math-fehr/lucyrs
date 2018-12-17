//! Module containing the obc ast, and a function to translate it into rust

pub mod ast;
pub mod to_rust;

use crate::obc::ast::Machine;

/// Compile an obc program into Rust
pub fn to_rust(machines: &Vec<Machine>, entry_machine: &str) -> String {
    to_rust::obc_to_rust(machines, entry_machine)
}
