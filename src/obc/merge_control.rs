//! Contains functions to merge control structure of an obc program

use crate::obc::ast::Stmt;

/// Merge control structures that are next to each others
pub fn merge_near_control(stmts: Vec<Stmt>) -> Vec<Stmt> {
    if stmts.len() < 2 {
        return stmts;
    }
    let mut new_stmts = vec![];
    let mut stmts_iter = stmts.into_iter();
    let mut old_stmt = stmts_iter.next().unwrap();
    for stmt in stmts_iter {
        if can_merge_stmts(&old_stmt, &stmt) {
            old_stmt = merge_stmts(old_stmt, stmt);
        } else {
            new_stmts.push(old_stmt);
            old_stmt = stmt;
        }
    }
    new_stmts.push(old_stmt);
    new_stmts
}

/// Check if two stmt can be merged
fn can_merge_stmts(stmt_1: &Stmt, stmt_2: &Stmt) -> bool {
    match (stmt_1, stmt_2) {
        (Stmt::Control(ck1, _, _), Stmt::Control(ck2, _, _)) => ck1 == ck2,
        (_, _) => false,
    }
}

/// Merge stmts knowing that they can be merged
fn merge_stmts(stmt_1: Stmt, stmt_2: Stmt) -> Stmt {
    match (stmt_1, stmt_2) {
        (Stmt::Control(ck, mut v1_t, mut v1_f), Stmt::Control(_, mut v2_t, mut v2_f)) => {
            v1_t.append(&mut v2_t);
            let v1 = merge_near_control(v1_t);
            v1_f.append(&mut v2_f);
            let v2 = merge_near_control(v2_t);
            Stmt::Control(ck, v1, v2)
        }
        (_, _) => unreachable!(),
    }
}
