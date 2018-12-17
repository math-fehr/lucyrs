//! This module contains functions to normalize and schedule minils, as
//! well as translating it into obc

pub mod ast;
pub mod normalization;
pub mod normalized_ast;
pub mod scheduling;
pub mod to_obc;

use crate::obc::ast as obc;

/// Transform minils into obc
pub fn to_obc(nodes: Vec<ast::Node>) -> Vec<obc::Machine> {
    nodes
        .into_iter()
        .map(normalization::normalize)
        .map(scheduling::schedule)
        .map(to_obc::to_obc)
        .collect()
}
