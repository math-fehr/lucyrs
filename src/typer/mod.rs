use self::typed_ast::{BaseExpr, Expr, Node};
use crate::ast::{BinOp, Type, UnOp, Value};
use crate::parser::ast;
use std::collections::HashMap;

mod typed_ast;

struct Context<'a> {
    variables: &'a HashMap<String, Type>,
    functions: &'a HashMap<String, (Vec<Type>, Vec<Type>)>,
}

pub fn annotate_types(nodes: Vec<ast::Node>) -> Result<Vec<Node>, String> {
    let mut functions = HashMap::new();
    let take_types = |vec: &Vec<(String, Type)>| { vec.iter().map(|(_,t)| {t.clone()}).collect()};
    let mut add_function = |node: &ast::Node| -> Result<(), String> {
        if functions.contains_key(&node.name) {
            Err(format!("Node {} was declared twice", node.name))
        } else {
            functions.insert(node.name.clone(), (take_types(&node.in_params), take_types(&node.out_params)));
            Ok(())
        }
    };
    for node in &nodes {
        add_function(node)?;
    }
    let mut typed_nodes = vec![];
    for node in nodes {
        typed_nodes.push(type_node(node, &functions)?);
    }
    Ok(typed_nodes)
}

pub fn type_node(node: ast::Node, functions: &HashMap<String, (Vec<Type>, Vec<Type>)>) -> Result<Node, String> {
    let mut variables = HashMap::new();
    let mut add_variables = |list: &Vec<(String, Type)>| -> Result<(), String> {
        for (ident, typ) in list {
            if variables.contains_key(ident) {
                return Err(String::from(
                    "Cannot declare two variables with the same name in a node",
                ));
            } else {
                variables.insert(ident.clone(), typ.clone());
            }
        }
        Ok(())
    };
    add_variables(&node.in_params)?;
    add_variables(&node.out_params)?;
    add_variables(&node.local_params)?;

    let context = Context {
        variables: &variables,
        functions,
    };

    let mut typed_expr = vec![];
    for eq in node.eq_list {
        typed_expr.push((eq.0, type_expr(eq.1, &context)?));
    }

    let node = Node {
        name: node.name,
        in_params: node.in_params,
        out_params: node.out_params,
        local_params: node.local_params,
        eq_list: typed_expr,
    };
    Ok(node)
}

fn type_expr(expr: ast::Expr, context: &Context) -> Result<Expr, String> {
    match expr {
        ast::Expr::Value(v) => Ok(type_value(v)),
        ast::Expr::UnOp(op, expr) => type_unop(op, *expr, context),
        ast::Expr::BinOp(op, lhs, rhs) => type_binop(op, *lhs, *rhs, context),
        ast::Expr::Pre(expr) => type_pre(*expr, context),
        ast::Expr::Arrow(expr1, expr2) => type_arrow(*expr1, *expr2, context),
        ast::Expr::IfThenElse(e_cond, e_then, e_else) => {
            type_ifthenelse(*e_cond, *e_then, *e_else, context)
        }
        ast::Expr::Var(ident) => type_var(ident, context),
        ast::Expr::FunCall(ident, params) => type_funcall(ident, params, context),
    }
}

fn type_value(value: Value) -> Expr {
    let typ = match value {
        Value::Bool(_) => vec![Type::Bool],
        Value::Int(_) => vec![Type::Int],
        Value::Real(_) => vec![Type::Real],
    };
    let expr = BaseExpr::Value(value);
    Expr { expr, typ }
}

fn type_unop(op: UnOp, expr: ast::Expr, context: &Context) -> Result<Expr, String> {
    let typed_expr = type_expr(expr, context)?;
    if typed_expr.typ.len() != 1 {
        Err(String::from("Unary operator cannot be applied to a tuple"))
    } else {
        match (op, typed_expr.typ[0].clone()) {
            (UnOp::Not, t) => {
                if let Type::Bool = t {
                    Ok(Expr {
                        expr: BaseExpr::UnOp(UnOp::Not, box typed_expr),
                        typ: vec![Type::Bool],
                    })
                } else {
                    Err(String::from(
                        "The not operator can only be applied to booleans",
                    ))
                }
            }
            (UnOp::UMinus, t) => {
                if let Type::Bool = t {
                    Err(String::from(
                        "The minus unary operator can only be applied to integers or reals",
                    ))
                } else {
                    Ok(Expr {
                        expr: BaseExpr::UnOp(UnOp::UMinus, box typed_expr),
                        typ: vec![Type::Int],
                    })
                }
            }
        }
    }
}

fn type_binop(
    op: BinOp,
    lhs: ast::Expr,
    rhs: ast::Expr,
    context: &Context,
) -> Result<Expr, String> {
    let typed_lhs = type_expr(lhs, context)?;
    let typed_rhs = type_expr(rhs, context)?;
    if typed_lhs.typ.len() != 1 || typed_rhs.typ.len() != 1 {
        Err(String::from("Binary operator cannot be applied to tuples"))
    } else if typed_lhs.typ[0] != typed_rhs.typ[0] {
        Err(String::from(
            "Binary operator should be applied on equal types",
        ))
    } else {
        let typ = typed_lhs.typ[0].clone();
        match op {
            c @ BinOp::Lt | c @ BinOp::Le | c @ BinOp::Gt | c @ BinOp::Ge => match typ {
                Type::Bool => Err(String::from(
                    "Lt, Le, Gt, and Ge operators should be applied on integers or reals",
                )),
                _ => Ok(Expr {
                    expr: BaseExpr::BinOp(c, box typed_lhs, box typed_rhs),
                    typ: vec![Type::Bool],
                }),
            },
            c @ BinOp::Mul | c @ BinOp::Div | c @ BinOp::Add | c @ BinOp::Sub => match typ {
                Type::Bool => Err(String::from(
                    "Mul, Div, Add, Sub operators should be applied on integers or reals",
                )),
                t => Ok(Expr {
                    expr: BaseExpr::BinOp(c, box typed_lhs, box typed_rhs),
                    typ: vec![t],
                }),
            },
            BinOp::Mod => match typ {
                Type::Int => Ok(Expr {
                    expr: BaseExpr::BinOp(BinOp::Mod, box typed_lhs, box typed_rhs),
                    typ: vec![Type::Int],
                }),
                _ => Err(String::from("Mod operator should be applied on integers")),
            },
            c @ BinOp::Or | c @ BinOp::Xor | c @ BinOp::And | c @ BinOp::Impl => match typ {
                Type::Bool => Ok(Expr {
                    expr: BaseExpr::BinOp(c, box typed_lhs, box typed_rhs),
                    typ: vec![Type::Bool],
                }),
                _ => Err(String::from(
                    "Or, And, and Impl operators should be applied on boolean types",
                )),
            },
            c @ BinOp::Neq | c @ BinOp::Eq => Ok(Expr {
                expr: BaseExpr::BinOp(c, box typed_lhs, box typed_rhs),
                typ: vec![Type::Bool],
            }),
        }
    }
}

fn type_pre(expr: ast::Expr, context: &Context) -> Result<Expr, String> {
    let typed_expr = type_expr(expr, context)?;
    let typ = typed_expr.typ.clone();
    Ok(Expr {
        expr: BaseExpr::Pre(box typed_expr),
        typ,
    })
}

fn type_arrow(lhs: ast::Expr, rhs: ast::Expr, context: &Context) -> Result<Expr, String> {
    let typed_lhs = type_expr(lhs, context)?;
    let typed_rhs = type_expr(rhs, context)?;
    if typed_lhs.typ != typed_rhs.typ {
        Err(String::from(
            "The type of the left hand side and the right hand side of an arrow should be equal",
        ))
    } else {
        let typ = typed_lhs.typ.clone();
        Ok(Expr {
            expr: BaseExpr::Arrow(box typed_lhs, box typed_rhs),
            typ,
        })
    }
}

fn type_ifthenelse(
    expr_cond: ast::Expr,
    expr_then: ast::Expr,
    expr_else: ast::Expr,
    context: &Context,
) -> Result<Expr, String> {
    let typed_cond = type_expr(expr_cond, context)?;
    let typed_then = type_expr(expr_then, context)?;
    let typed_else = type_expr(expr_else, context)?;
    if typed_cond.typ != [Type::Bool] {
        Err(String::from(
            "The conditional in a if statement should have type bool",
        ))
    } else if typed_then.typ != typed_else.typ {
        Err(String::from(
            "The type of both branches of a conditional should be equal",
        ))
    } else {
        let typ = typed_then.typ.clone();
        Ok(Expr {
            expr: BaseExpr::IfThenElse(box typed_cond, box typed_then, box typed_else),
            typ,
        })
    }
}

fn type_var(ident: String, context: &Context) -> Result<Expr, String> {
    if let Some(t) = context.variables.get(&ident) {
        Ok(Expr {
            expr: BaseExpr::Var(ident),
            typ: vec![t.clone()],
        })
    } else {
        Err(format!("Variable {} used but not declared", &ident))
    }
}

fn type_funcall(ident: String, inputs: Vec<ast::Expr>, context: &Context) -> Result<Expr, String> {
    if let Some((in_type, out_type)) = context.functions.get(&ident) {
        if inputs.len() != in_type.len() {
            Err(format!(
                "Node {} expect {} inputs, but only {} were given",
                &ident,
                inputs.len(),
                in_type.len()
            ))
        } else {
            let mut typed_inputs = vec![];
            for input in inputs {
                typed_inputs.push(type_expr(input, context)?);
            }
            for i in 0..typed_inputs.len() {
                if typed_inputs[i].typ.len() != 1 || typed_inputs[i].typ[0] != in_type[i] {
                    return Err(format!(
                        "Input {} has not the expected type in node call.",
                        i
                    ));
                }
            }
            Ok(Expr {
                expr: BaseExpr::FunCall(ident, typed_inputs),
                typ: out_type.clone(),
            })
        }
    } else {
        Err(format!("Node {} used but not declared", &ident))
    }
}
