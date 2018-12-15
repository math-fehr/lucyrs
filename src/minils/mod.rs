pub mod ast;
pub mod normalization;
pub mod normalized_ast;
pub mod scheduling;
pub mod to_obc;

use crate::obc::ast as obc;

pub fn to_obc(nodes: Vec<ast::Node>) -> Vec<obc::Machine> {
    nodes
        .into_iter()
        .map(normalization::normalize)
        .map(scheduling::schedule)
        .map(to_obc::to_obc)
        .collect()
}
