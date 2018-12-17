//! Module containing the obc ast, and a function to translate it into rust

pub mod ast;
pub mod merge_control;
pub mod to_rust;

use crate::obc::ast::Machine;

/// Compile an obc program into Rust
pub fn to_rust(mut machines: Vec<Machine>, entry_machine: &str) -> String {
    for machine in &mut machines {
        machine.step_stmts = merge_control::merge_near_control(machine.step_stmts.clone());
    }
    to_rust::obc_to_rust(&machines, entry_machine)
}
