use crate::api::API;

fn generate() {
    let mut buffer = String::new();

    for ty in API::def().types() {
        let name = ty.name();

        match &ty {
            agdb::api::Type::None => {}
            agdb::api::Type::Enum(e) => buffer.push_str(&write_enum(name, e)),
            agdb::api::Type::Struct(s) => buffer.push_str(&write_struct(name, s)),
            agdb::api::Type::List(list) => buffer.push_str(&write_list(name, list)),
            agdb::api::Type::U8
            | agdb::api::Type::I64
            | agdb::api::Type::U64
            | agdb::api::Type::F64 => buffer.push_str(&format!("type {name} = number;\n")),
            agdb::api::Type::String => buffer.push_str(&format!("type {name} = string;\n")),
            agdb::api::Type::User => buffer.push_str(&format!("type {name} = any;\n")),
        }
    }

    std::fs::write("agdb_api.ts", buffer).unwrap();
}

fn write_enum(name: &str, e: &agdb::api::Enum) -> String {
    let mut buffer = String::new();
    buffer.push_str(&format!("type {name} = "));
    let variants = e
        .variants
        .iter()
        .map(|v| {
            if (v.ty)().name() == "None" {
                format!("\"{}\"", v.name)
            } else {
                format!("{{ {}: {} }}", v.name, (v.ty)().name())
            }
        })
        .collect::<Vec<_>>()
        .join(" | ");
    buffer.push_str(&format!("{variants};\n"));
    buffer
}

fn write_list(name: &str, list: &agdb::api::List) -> String {
    let item_ty = (list.ty)().name().to_string();
    format!("type {name} = {item_ty}[];\n")
}

fn write_struct(name: &str, s: &agdb::api::Struct) -> String {
    if s.fields.len() == 1 && (s.functions)().is_empty() && s.fields[0].name.is_empty() {
        return write_alias(name, s);
    }

    if (s.functions)().is_empty() {
        return write_pod(name, s);
    }

    write_class(name, s)
}

fn write_class(name: &str, s: &agdb::api::Struct) -> String {
    let mut buffer = String::new();
    buffer.push_str(&format!("class {name} {{\n"));
    let mut constructor_params = Vec::new();
    let mut field_inits = Vec::new();

    for (i, field) in s.fields.iter().enumerate() {
        let field_name = if field.name.is_empty() {
            format!("__value{i}")
        } else {
            format!("__{}", field.name)
        };
        let field_ty = (field.ty)().name().to_string();

        buffer.push_str(&format!("  private {field_name}: {field_ty};\n"));
        constructor_params.push(format!("{}: {}", field_name, field_ty));
        field_inits.push(format!("this.{field_name} = {field_name};"));
    }

    buffer.push_str(&format!(
        "  constructor({}) {{\n    {}\n  }}\n",
        constructor_params.join(", "),
        field_inits.join("\n    ")
    ));

    for f in (s.functions)() {
        let ret_ty = if let Some(ty) = &f.ret {
            format!(": {}", ty().name())
        } else {
            String::new()
        };
        let params = f
            .args
            .iter()
            .map(|p| format!("{}: {}", p.name, (p.ty)().name()))
            .collect::<Vec<_>>()
            .join(", ");

        buffer.push_str(&format!("  {}({}){} {{\n", f.name, params, ret_ty));

        for e in &f.expressions {
            buffer.push_str(&format!("    {};\n", write_expression(e, s)));
        }

        buffer.push_str("  }\n");
    }

    buffer.push_str("}\n");
    buffer
}

fn write_alias(name: &str, s: &agdb::api::Struct) -> String {
    let field_ty = (s.fields[0].ty)().name().to_string();
    format!("type {name} = {field_ty};\n")
}

fn write_pod(name: &str, s: &agdb::api::Struct) -> String {
    let mut buffer = String::new();
    buffer.push_str(&format!("type {name} {{\n"));
    for (i, field) in s.fields.iter().enumerate() {
        let field_name = if field.name.is_empty() {
            &format!("value{i}")
        } else {
            field.name
        };
        let field_ty = (field.ty)().name().to_string();
        buffer.push_str(&format!("    {}: {};\n", field_name, field_ty));
    }
    buffer.push_str("}\n");
    buffer
}

fn write_expression(expr: &agdb::api::Expression, s: &agdb::api::Struct) -> String {
    match expr {
        agdb::api::Expression::Array { elements } => {
            let elems = elements
                .iter()
                .map(|e| write_expression(e, s))
                .collect::<Vec<_>>()
                .join(", ");
            format!("    [{}]", elems)
        }
        agdb::api::Expression::Assign { target, value } => {
            let left = write_expression(target, s);
            let right = write_expression(value, s);
            format!("{left} = {right}")
        }
        agdb::api::Expression::Binary { op, left, right } => {
            let left_str = write_expression(left, s);
            let right_str = write_expression(right, s);
            format!("{left_str} {op} {right_str}")
        }
        agdb::api::Expression::Block(expressions) => {
            let exprs = expressions
                .iter()
                .map(|e| write_expression(e, s))
                .collect::<Vec<_>>()
                .join(";\n    ");
            format!("{{\n    {}\n  }}", exprs)
        }
        agdb::api::Expression::Call {
            recipient,
            function,
            args,
        } => {
            let recipient_str = if let Some(rec) = recipient {
                write_expression(rec, s)
            } else {
                String::new()
            };
            let dot = if !recipient_str.is_empty() { "." } else { "" };
            let args_str = args
                .iter()
                .map(|e| write_expression(e, s))
                .collect::<Vec<_>>()
                .join(", ");
            let func = match *function {
                "starts_with" => "startsWith",
                "ends_with" => "endsWith",
                _ => function,
            };
            match *function {
                "to_string" | "clone" | "into" => recipient_str,
                _ => format!("{recipient_str}{dot}{func}({args_str})"),
            }
        }
        agdb::api::Expression::Closure { ret, body } => {
            let ret_str = if ret.is_some() {
                String::from(" => ")
            } else {
                String::new()
            };
            let body_str = body
                .iter()
                .map(|e| write_expression(e, s))
                .collect::<Vec<_>>()
                .join(";\n    ");
            format!("(){}{{\n    {}\n  }}", ret_str, body_str)
        }
        agdb::api::Expression::FieldAccess { base: _, field } => {
            format!("this.__{}", field)
        }
        agdb::api::Expression::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_str = write_expression(condition, s);
            let then_str = write_expression(then_branch, s);
            let else_str = if let Some(else_b) = else_branch {
                format!(" else {}", write_expression(else_b, s))
            } else {
                String::new()
            };
            format!("if ({}) {}{}", cond_str, then_str, else_str)
        }
        agdb::api::Expression::Index { base, index } => {
            let base_str = write_expression(base, s);
            let index_str = write_expression(index, s);
            format!("{}[{}]", base_str, index_str)
        }
        agdb::api::Expression::Let { name, ty, value } => {
            let ty_str = if let Some(ty_fn) = ty {
                format!(": {}", ty_fn().name())
            } else {
                String::new()
            };
            let value_str = if let Some(val) = value {
                format!(" = {}", write_expression(val, s))
            } else {
                String::new()
            };
            format!("let {}{}{}", name, ty_str, value_str)
        }
        agdb::api::Expression::Literal(literal_value) => match literal_value {
            agdb::api::LiteralValue::I64(v) => v.to_string(),
            agdb::api::LiteralValue::F64(v) => v.to_string(),
            agdb::api::LiteralValue::String(v) => format!("\"{v}\""),
            agdb::api::LiteralValue::Bool(v) => v.to_string(),
        },
        agdb::api::Expression::Return(expression) => {
            if let Some(expr) = expression {
                let expr_str = write_expression(expr, s);
                format!("return {}", expr_str)
            } else {
                String::from("return")
            }
        }
        agdb::api::Expression::Struct { name, fields } => {
            let field_strs = fields
                .iter()
                .map(|(_field_name, expr)| write_expression(expr, s))
                .collect::<Vec<_>>()
                .join(", ");
            let name = if *name == "Self" {
                s.name.as_str()
            } else {
                name
            };
            format!("new {name}({field_strs})")
        }
        agdb::api::Expression::Unary { op, expr } => {
            let expr_str = write_expression(expr, s);
            format!("{op}{expr_str}")
        }
        agdb::api::Expression::Variable(name) => {
            match *name {
                "self" => "this",
                "None" => "\"None\"",
                _ => name,
            }
        }
        .to_string(),
        agdb::api::Expression::While { condition, body } => {
            let cond_str = write_expression(condition, s);
            let body_str = write_expression(body, s);
            format!("while ({}) {}", cond_str, body_str)
        }
        agdb::api::Expression::Unknown(expr) => {
            panic!("Unknown expression: {expr}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        generate();
    }
}
