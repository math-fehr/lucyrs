use crate::ast::{BinOp, Type, UnOp, Value};
use crate::lucy::ast;
use crate::lucy::typed_ast::{BaseExpr, Expr, Node};
use std::collections::HashMap;

struct Context<'a> {
    variables: &'a HashMap<String, Type>,
    functions: &'a HashMap<String, (Vec<Type>, Vec<Type>)>,
}

pub fn annotate_types(nodes: Vec<ast::Node>) -> Result<Vec<Node>, String> {
    let mut functions = HashMap::new();
    let take_types = |vec: &Vec<(String, Type)>| vec.iter().map(|(_, t)| t.clone()).collect();
    let mut add_function = |node: &ast::Node| -> Result<(), String> {
        if functions.contains_key(&node.name) {
            Err(format!("Node {} was declared twice", node.name))
        } else {
            functions.insert(
                node.name.clone(),
                (take_types(&node.in_params), take_types(&node.out_params)),
            );
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

pub fn type_node(
    node: ast::Node,
    functions: &HashMap<String, (Vec<Type>, Vec<Type>)>,
) -> Result<Node, String> {
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
        ast::Expr::When(box expr, ck, b) => type_when(expr, ck, b, context),
        ast::Expr::Merge(s, box e_true, box e_false) => type_merge(s, e_true, e_false, context),
        ast::Expr::Fby(v, expr2) => type_fby(v, *expr2, context),
        ast::Expr::IfThenElse(e_cond, e_then, e_else) => {
            type_ifthenelse(*e_cond, *e_then, *e_else, context)
        }
        ast::Expr::Var(ident) => type_var(ident, context),
        ast::Expr::FunCall(ident, params, ck) => type_funcall(ident, params, ck, context),
        ast::Expr::Current(ident, v) => type_current(ident, v, context),
        ast::Expr::Pre(box e) => type_pre(e, context),
        ast::Expr::Arrow(box e1, box e2) => type_arrow(e1, e2, context),
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
                        expr: BaseExpr::UnOp(UnOp::UMinus, box typed_expr.clone()),
                        typ: typed_expr.typ.clone(),
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

fn type_when(expr: ast::Expr, ck: String, b: bool, context: &Context) -> Result<Expr, String> {
    let typed_expr = type_expr(expr, context)?;
    let typ = typed_expr.typ.clone();
    if context.variables.get(&ck).is_none() {
        return Err(String::from(
            "The clock in a when construct should be a boolean",
        ));
    }
    Ok(Expr {
        expr: BaseExpr::When(box typed_expr, ck, b),
        typ,
    })
}

fn type_merge(
    ck: String,
    e_true: ast::Expr,
    e_false: ast::Expr,
    context: &Context,
) -> Result<Expr, String> {
    let typed_e_true = type_expr(e_true, context)?;
    let typed_e_false = type_expr(e_false, context)?;
    if typed_e_false.typ != typed_e_true.typ {
        return Err(String::from(
            "The type of the two expressions in a merge construct should have the same type",
        ));
    }
    let typ = typed_e_false.typ.clone();
    if context.variables.get(&ck).is_none() {
        return Err(String::from(
            "The clock in a merge construct should be a boolean",
        ));
    }
    Ok(Expr {
        expr: BaseExpr::Merge(ck, box typed_e_true, box typed_e_false),
        typ,
    })
}

fn type_fby(init: Value, rhs: ast::Expr, context: &Context) -> Result<Expr, String> {
    let typed_rhs = type_expr(rhs, context)?;
    let typed_init = type_value(init.clone());
    if typed_init.typ != typed_rhs.typ {
        Err(String::from(
            "The type of the left hand side and the right hand side of an arrow or a fby should be equal",
        ))
    } else {
        let typ = typed_init.typ.clone();
        Ok(Expr {
            expr: BaseExpr::Fby(init, box typed_rhs),
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

fn type_funcall(
    ident: String,
    inputs: Vec<ast::Expr>,
    ck: Option<String>,
    context: &Context,
) -> Result<Expr, String> {
    if let Some((in_type, out_type)) = context.functions.get(&ident) {
        if inputs.len() != in_type.len() {
            return Err(format!(
                "Node {} expect {} inputs, but only {} were given",
                &ident,
                inputs.len(),
                in_type.len()
            ));
        }
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
        if let Some(ck) = ck.clone() {
            if let Some(t) = context.variables.get(&ck) {
                if t != &Type::Bool {
                    return Err(format!(
                        "The variable {} was used as reset but is of type {:?}",
                        ck, t
                    ));
                }
            } else {
                return Err(format!("Variable {} used but not declared", &ident));
            }
        }
        Ok(Expr {
            expr: BaseExpr::FunCall(ident, typed_inputs, ck),
            typ: out_type.clone(),
        })
    } else {
        Err(format!("Node {} used but not declared", &ident))
    }
}

fn type_current(ident: String, value: Value, context: &Context) -> Result<Expr, String> {
    if let Some(t) = context.variables.get(&ident) {
        if t != &value.get_type() {
            Err(String::from("In a current construct, the initial value and the variable should have the same type."))
        } else {
            Ok(Expr {
                expr: BaseExpr::Current(ident, value),
                typ: vec![t.clone()],
            })
        }
    } else {
        Err(format!("Variable {} used but not declared", &ident))
    }
}

fn type_pre(expr: ast::Expr, context: &Context) -> Result<Expr, String> {
    let typed_expr = type_expr(expr, context)?;
    if typed_expr.typ.len() != 1 {
        return Err(String::from("pre operator cannot be applied to a tuple"));
    }
    let typ = typed_expr.typ.clone();
    Ok(Expr {
        expr: BaseExpr::Pre(box typed_expr),
        typ,
    })
}

fn type_arrow(expr_1: ast::Expr, expr_2: ast::Expr, context: &Context) -> Result<Expr, String> {
    let expr_1 = type_expr(expr_1, context)?;
    let expr_2 = type_expr(expr_2, context)?;
    if expr_1.typ.len() != 1 || expr_2.typ.len() != 1 {
        return Err(String::from(
            "In an arrow construct, the two expressions hsould not be tuples",
        ));
    }
    if expr_1.typ[0] != expr_2.typ[0] {
        return Err(String::from(
            "In an arrow construct, both expressions should have same size",
        ));
    }
    let typ = expr_1.typ.clone();
    Ok(Expr {
        expr: BaseExpr::Arrow(box expr_1, box expr_2),
        typ,
    })
}
