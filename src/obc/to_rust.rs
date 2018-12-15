use crate::ast::{BinOp, Type, UnOp, Value};
use crate::ident;
use crate::obc::ast::{Expr, Machine, Stmt};

pub fn obc_to_rust(machines: &Vec<Machine>, entry_machine: &str) -> String {
    let entry_machine = machines
        .iter()
        .find(|m| m.name == ident::gen_ident(entry_machine.to_string(), 0))
        .unwrap();
    let prog = get_rust_main(entry_machine) + "\n\n";
    prog + &machines.into_iter().fold(String::new(), |s, machine| {
        s + &machine_to_rust(machine) + "\n\n"
    })
}

fn get_rust_main(machine: &Machine) -> String {
    let mut main = String::from("use std::io::{self, Read};\n");
    main += "fn main() {\n";
    main += &format!(
        "    let mut entry_machine: {} = Default::default();\n",
        machine.name
    );
    main += "    entry_machine.reset();\n";
    main += "    let mut buffer =  String::new();\n";
    main += "    loop {\n";
    for (input, typ) in &machine.step_inputs {
        main += &format!("        buffer = String::new();\n");
        main += &format!(
            "        println!(\"Value of {} ({}): \");\n",
            input,
            type_to_rust(typ)
        );
        main += "        io::stdin().read_line(&mut buffer).unwrap();\n";
        main += &format!(
            "        let {}: {} = buffer.trim().parse().unwrap();\n",
            input,
            type_to_rust(typ)
        );
    }
    let inputs = machine
        .step_inputs
        .iter()
        .map(|(s, _)| s.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let outputs = machine
        .step_returns
        .iter()
        .map(|(s, _)| s.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    main += &format!(
        "        let ({}) = entry_machine.step({});\n",
        outputs.clone(),
        inputs
    );
    main += &format!("        println!(\"Results: {{:?}}\", ({}));\n", outputs);
    main += "        println!(\"{:#?}\", entry_machine);\n";
    main += "    }\n";
    main + "}\n"
}

fn machine_to_rust(machine: &Machine) -> String {
    let mut machine_str = get_struct_definition(machine);
    machine_str += "\n";
    machine_str += &get_functions_definition(machine);
    machine_str
}

fn get_struct_definition(machine: &Machine) -> String {
    let mut def = format!("#[derive(Default, Debug)]\n");
    def += &format!("struct {} {{\n", machine.name);
    for (memory, val) in &machine.memory {
        def += &format!("    pub {}: {},\n", memory, type_to_rust(&val.get_type()));
    }
    for (instance, typ) in &machine.instances {
        def += &format!("    pub {}: {},\n", instance, typ);
    }
    def += "}\n";
    def
}

fn get_functions_definition(machine: &Machine) -> String {
    let mut def = format!("impl {} {{\n", machine.name);
    def += &get_reset_definition(machine);
    def += "\n";
    def += &get_step_definition(machine);
    def += "}\n";
    def
}

fn get_reset_definition(machine: &Machine) -> String {
    let mut def = format!("    pub fn reset(&mut self) {{\n");
    for (memory, value) in &machine.memory {
        def += &format!("        self.{} = {};\n", memory, value_to_rust(value));
    }
    for (instance, _) in &machine.instances {
        def += &format!("        self.{}.reset();\n", instance);
    }
    def += "    }\n";
    def
}

fn get_step_definition(machine: &Machine) -> String {
    let inputs = machine
        .step_inputs
        .iter()
        .map(|(name, typ)| format!("{}: {}", name, type_to_rust(typ)))
        .collect::<Vec<String>>()
        .join(", ");
    let outputs = machine
        .step_returns
        .iter()
        .map(|(_, typ)| type_to_rust(typ))
        .collect::<Vec<String>>()
        .join(", ");

    let mut def = format!(
        "    pub fn step(&mut self, {}) -> ({}) {{\n",
        inputs, outputs
    );
    for (var, typ) in &machine.step_vars {
        def += &format!(
            "        let mut {}: {} = Default::default();\n",
            var,
            type_to_rust(typ)
        );
    }
    for stmt in &machine.step_stmts {
        def += &stmt_to_rust(machine, stmt, 2);
    }
    let returns = machine
        .step_returns
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    def += &format!("        ({})\n", returns);
    def += "    }\n";
    def
}

fn stmt_to_rust(machine: &Machine, stmt: &Stmt, n_indent: i32) -> String {
    let indent = " ".repeat((n_indent as usize) * 4);
    match stmt {
        Stmt::Assignment(s, expr) => format!("{}{} = {};\n", indent, s, expr_to_rust(expr)),
        Stmt::StateAssignment(s, expr) => {
            format!("{}self.{} = {};\n", indent, s, expr_to_rust(expr))
        },
        Stmt::Step(results, fun, params) => {
            let params = params
                .iter()
                .map(expr_to_rust)
                .collect::<Vec<String>>()
                .join(", ");
            let results_temp = results
                .iter()
                .map(|s| s.clone() + "_result")
                .collect::<Vec<String>>();
            let results_str = results_temp.clone().join(", ");
            let mut step = format!(
                "{}let ({}) = self.{}.step({});\n",
                indent, results_str, fun, params
            );
            for (l,r) in results.iter().zip(results_temp) {
                step += &format!("{}{} = {};\n", indent, l, r);
            }
            step
        }
        Stmt::Reset(s) => format!("{}self.{}.reset();\n", indent, s),
        Stmt::Control(x, stmts_true, stmts_false) => {
            let cond = if machine.memory.contains_key(x) {
                format!("self.{}", x)
            } else {
                x.clone()
            };
            let mut string = format!("{}if {} {{\n", indent.clone(), cond);
            for stmt in stmts_true {
                string += &stmt_to_rust(machine, stmt, n_indent + 1);
            }
            string += &format!("{}}} else {{\n", indent.clone());
            for stmt in stmts_false {
                string += &stmt_to_rust(machine, stmt, n_indent + 1);
            }
            string += &format!("{}}}\n", indent);
            string
        }
    }
}

fn expr_to_rust(expr: &Expr) -> String {
    match expr {
        Expr::Var(s) => s.clone(),
        Expr::Value(v) => value_to_rust(v),
        Expr::State(s) => format!("self.{}", s),
        Expr::UnOp(op, box expr) => format!("({}{})", unop_to_rust(op), expr_to_rust(expr)),
        Expr::BinOp(op, box lhs, box rhs) => {
            if let BinOp::Impl = op {
                format!("(!{} || {})", expr_to_rust(lhs), expr_to_rust(rhs))
            } else {
                format!(
                    "({} {} {})",
                    expr_to_rust(lhs),
                    binop_to_rust(op),
                    expr_to_rust(rhs)
                )
            }
        }
    }
}

fn unop_to_rust(op: &UnOp) -> String {
    match op {
        UnOp::Not => String::from("!"),
        UnOp::UMinus => String::from("-"),
    }
}

fn binop_to_rust(op: &BinOp) -> String {
    match op {
        BinOp::Lt => String::from("<"),
        BinOp::Le => String::from("<="),
        BinOp::Gt => String::from(">"),
        BinOp::Ge => String::from(">="),
        BinOp::Mul => String::from("*"),
        BinOp::Div => String::from("/"),
        BinOp::Add => String::from("+"),
        BinOp::Sub => String::from("-"),
        BinOp::Mod => String::from("%"),
        BinOp::Or => String::from("||"),
        BinOp::Xor => String::from("^"),
        BinOp::And => String::from("&&"),
        BinOp::Impl => unreachable!(),
        BinOp::Neq => String::from("!="),
        BinOp::Eq => String::from("=="),
    }
}

fn type_to_rust(typ: &Type) -> String {
    match typ {
        Type::Int => String::from("i32"),
        Type::Real => String::from("f32"),
        Type::Bool => String::from("bool"),
    }
}

fn value_to_rust(val: &Value) -> String {
    match val {
        Value::Int(i) => i.to_string(),
        Value::Real(r) => r.to_string() + "f32",
        Value::Bool(b) => b.to_string(),
    }
}
