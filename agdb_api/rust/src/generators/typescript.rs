use agdb::api::Type;

use crate::api::API;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Default)]
struct Typescript {
    buffer: String,
    names: HashMap<String, String>,
    types: HashMap<String, Type>,
}

#[allow(dead_code)]
impl Typescript {
    fn generate(&mut self) {
        for ty in API::def().types() {
            let name = ty.name();
            self.names
                .insert(ty.name().to_string(), self.ts_name(name).to_string());
            self.types.insert(name.to_string(), ty.clone());
        }

        for ty in API::def().types() {
            let name = ty.name();

            match &ty {
                agdb::api::Type::None => {}
                agdb::api::Type::Enum(e) => {
                    if e.name.starts_with("Result_") || e.name.starts_with("Option_") {
                        continue;
                    }

                    self.buffer
                        .push_str(&self.write_enum(&self.ts_name(name), e));
                }
                agdb::api::Type::Struct(s) => {
                    if s.name.starts_with("Tuple_") {
                        continue;
                    }

                    self.buffer
                        .push_str(&self.write_struct(&self.ts_name(name), s));
                }
                agdb::api::Type::U8
                | agdb::api::Type::I64
                | agdb::api::Type::U64
                | agdb::api::Type::F64
                | agdb::api::Type::String
                | agdb::api::Type::List(_) => { /* ignored */ }
                agdb::api::Type::User => self.buffer.push_str(&format!("type {name} = any;\n")),
            }
        }

        std::fs::write("agdb_api.ts", &self.buffer).unwrap();
    }

    fn ts_name(&self, name: &str) -> String {
        if let Some(name) = self.names.get(name) {
            return name.to_string();
        }

        match name {
            "String" => "string".to_string(),
            "u8" | "i64" | "u64" | "f64" => "number".to_string(),
            _ => {
                if let Some(name) = name.strip_prefix("List_") {
                    return format!("{}[]", self.ts_name(name));
                }

                if let Some(name) = name.strip_prefix("Result_") {
                    let (ok, err) = name.rsplit_once("_").expect("Result should have two parts");
                    let ok_type = self.ts_name(ok);
                    let err_type = self.ts_name(err);
                    return format!("{ok_type} | {err_type}");
                }

                if let Some(name) = name.strip_prefix("Option_") {
                    return format!("{} | undefined", self.ts_name(name));
                }

                if let Some(name) = name.strip_prefix("Tuple_") {
                    let mut rest = name;
                    let mut types = Vec::new();

                    while !rest.is_empty() {
                        let (elem_ts, new_rest) = self.parse_one_type(rest);
                        types.push(elem_ts);
                        rest = new_rest;
                    }

                    return format!("[{}]", types.join(", "));
                }

                name.to_string()
            }
        }
    }

    fn parse_one_type<'a>(&self, s: &'a str) -> (String, &'a str) {
        if s.is_empty() {
            return (String::new(), s);
        }

        if let Some(inner) = s.strip_prefix("List_") {
            let (elem_ts, rest) = self.parse_one_type(inner);
            return (format!("{elem_ts}[]"), rest);
        }

        if let Some(inner) = s.strip_prefix("Option_") {
            let (elem_ts, rest) = self.parse_one_type(inner);
            return (format!("{elem_ts} | undefined"), rest);
        }

        if let Some(inner) = s.strip_prefix("Result_") {
            let (ok_ts, mut rest) = self.parse_one_type(inner);
            if let Some(r) = rest.strip_prefix('_') {
                rest = r;
            }
            let (err_ts, rest) = self.parse_one_type(rest);
            return (format!("{ok_ts} | {err_ts}"), rest);
        }

        if let Some(inner) = s.strip_prefix("Tuple_") {
            let mut rest = inner;
            let mut elems = Vec::new();
            while !rest.is_empty() {
                let (elem_ts, r) = self.parse_one_type(rest);
                elems.push(elem_ts);
                rest = r;
            }
            return (format!("[{}]", elems.join(", ")), rest);
        }

        if let Some(pos) = s.find('_') {
            let token = &s[..pos];
            let rest = &s[pos + 1..];
            (self.ts_name(token), rest)
        } else {
            (self.ts_name(s), "")
        }
    }

    fn write_enum(&self, name: &str, e: &agdb::api::Enum) -> String {
        let mut buffer = String::new();
        buffer.push_str(&format!("type {name} = "));
        let variants = e
            .variants
            .iter()
            .map(|v| {
                if (v.ty)().name() == "None" {
                    format!("\"{}\"", v.name)
                } else {
                    format!("{{ {}: {} }}", v.name, self.ts_name((v.ty)().name()))
                }
            })
            .collect::<Vec<_>>()
            .join(" | ");
        buffer.push_str(&format!("{variants};\n"));
        buffer
    }

    fn write_struct(&self, name: &str, s: &agdb::api::Struct) -> String {
        if name == "ReqwestClient" {
            return Self::write_reqwest_client();
        }

        if s.fields.len() == 1 && (s.functions)().is_empty() && s.fields[0].name.is_empty() {
            return self.write_alias(name, s);
        }

        if (s.functions)().is_empty() {
            return self.write_pod(name, s);
        }

        self.write_class(name, s)
    }

    fn write_reqwest_client() -> String {
        let mut buffer = String::new();
        buffer.push_str("class ReqwestClient {\n");
        buffer.push_str(
            r#"
  get(url: string, token: string | undefined): [number, string] {
    return [200, ""]
  }
  delete(url: string, token: string | undefined): [number, string] {
    return [200, ""]
  }
  post(url: string, data: string | undefined, token: string | undefined): [number, string] {
    return [200, ""]
  }
  put(url: string, data: string | undefined, token: string | undefined): [number, string] {
    return [200, ""]
  }
"#,
        );
        buffer.push_str("}\n");

        buffer
    }

    fn write_class(&self, name: &str, s: &agdb::api::Struct) -> String {
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
            let field_ty = self.ts_name((field.ty)().name());

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
                format!(": {}", self.ts_name(ty().name()))
            } else {
                String::new()
            };
            let params = f
                .args
                .iter()
                .map(|p| format!("{}: {}", p.name, self.ts_name((p.ty)().name())))
                .collect::<Vec<_>>()
                .join(", ");

            buffer.push_str(&format!("  {}({}){} {{\n", f.name, params, ret_ty));

            for e in &f.expressions {
                buffer.push_str(&format!("    {};\n", self.write_expression(e, s)));
            }

            buffer.push_str("  }\n");
        }

        buffer.push_str("}\n");
        buffer
    }

    fn write_alias(&self, name: &str, s: &agdb::api::Struct) -> String {
        let field_ty = self.ts_name((s.fields[0].ty)().name());
        format!("type {name} = {field_ty};\n")
    }

    fn write_pod(&self, name: &str, s: &agdb::api::Struct) -> String {
        let mut buffer = String::new();
        let mut constructor_params = Vec::new();
        let mut field_inits = Vec::new();

        buffer.push_str(&format!("class {name} {{\n"));
        for (i, field) in s.fields.iter().enumerate() {
            let field_name = if field.name.is_empty() {
                &format!("value{i}")
            } else {
                field.name
            };
            let field_ty = self.ts_name((field.ty)().name());
            buffer.push_str(&format!("   public {}: {};\n", field_name, field_ty));
            constructor_params.push(format!("{}: {}", field_name, field_ty));
            field_inits.push(format!("this.{field_name} = {field_name};"));
        }
        buffer.push_str(&format!(
            "  constructor({}) {{\n    {}\n  }}\n",
            constructor_params.join(", "),
            field_inits.join("\n    ")
        ));
        buffer.push_str("}\n");
        buffer
    }

    fn write_expression(&self, expr: &agdb::api::Expression, s: &agdb::api::Struct) -> String {
        match expr {
            agdb::api::Expression::Array { elements } => {
                let elems = elements
                    .iter()
                    .map(|e| self.write_expression(e, s))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("    [{}]", elems)
            }
            agdb::api::Expression::Assign { target, value } => {
                let left = self.write_expression(target, s);
                let right = self.write_expression(value, s);
                format!("{left} = {right}")
            }
            agdb::api::Expression::Binary { op, left, right } => {
                let left_str = self.write_expression(left, s);
                let right_str = self.write_expression(right, s);
                format!("{left_str} {op} {right_str}")
            }
            agdb::api::Expression::Block(expressions) => {
                let exprs = expressions
                    .iter()
                    .map(|e| self.write_expression(e, s))
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
                    self.write_expression(rec, s)
                } else {
                    String::new()
                };
                let dot = if !recipient_str.is_empty() { "." } else { "" };
                let args_str = args
                    .iter()
                    .map(|e| self.write_expression(e, s))
                    .collect::<Vec<_>>()
                    .join(", ");
                let func = match *function {
                    "starts_with" => "startsWith",
                    "ends_with" => "endsWith",
                    _ => function,
                };
                match *function {
                    "to_string" | "clone" | "into" => recipient_str,
                    "Ok" => args_str,
                    "format" => self.write_format_string(args, s),
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
                    .map(|e| self.write_expression(e, s))
                    .collect::<Vec<_>>()
                    .join(";\n    ");
                format!("(){}{{\n    {}\n  }}", ret_str, body_str)
            }
            agdb::api::Expression::FieldAccess { base, field } => {
                if let agdb::api::Expression::Variable(name) = base.as_ref()
                    && *name == "self"
                {
                    if *field == "0" {
                        return "this.__value0".to_string();
                    } else {
                        return format!("this.__{}", field);
                    }
                }

                if field.chars().all(|c| c.is_ascii_digit()) {
                    let base_str = self.write_expression(base, s);
                    return format!("({})[{}]", base_str, field);
                }

                let base_str = self.write_expression(base, s);
                format!("{base_str}.{}", field)
            }
            agdb::api::Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_str = self.write_expression(condition, s);
                let then_str = self.write_expression(then_branch, s);
                let else_str = if let Some(else_b) = else_branch {
                    format!(" else {}", self.write_expression(else_b, s))
                } else {
                    String::new()
                };
                format!("if ({}) {}{}", cond_str, then_str, else_str)
            }
            agdb::api::Expression::Index { base, index } => {
                let base_str = self.write_expression(base, s);
                let index_str = self.write_expression(index, s);
                format!("{}[{}]", base_str, index_str)
            }
            agdb::api::Expression::Let { name, ty, value } => self.write_let(s, name, ty, value),
            agdb::api::Expression::Literal(literal_value) => match literal_value {
                agdb::api::LiteralValue::I64(v) => v.to_string(),
                agdb::api::LiteralValue::F64(v) => v.to_string(),
                agdb::api::LiteralValue::String(v) => format!("\"{v}\""),
                agdb::api::LiteralValue::Bool(v) => v.to_string(),
            },
            agdb::api::Expression::Return(expression) => {
                if let Some(expr) = expression {
                    let expr_str = self.write_expression(expr, s);
                    let new = if !expr_str.starts_with("new")
                        && expr_str
                            .chars()
                            .next()
                            .expect("return expression should not be empty")
                            .is_uppercase()
                    {
                        "new "
                    } else {
                        ""
                    };
                    format!("return {}{}", new, expr_str)
                } else {
                    String::from("return")
                }
            }
            agdb::api::Expression::Struct { name, fields } => {
                let field_strs = fields
                    .iter()
                    .map(|(_field_name, expr)| self.write_expression(expr, s))
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
                let expr_str = self.write_expression(expr, s);
                format!("{op}{expr_str}")
            }
            agdb::api::Expression::Variable(name) => {
                match *name {
                    "self" => "this",
                    "None" => "undefined",
                    _ => name,
                }
            }
            .to_string(),
            agdb::api::Expression::While { condition, body } => {
                let cond_str = self.write_expression(condition, s);
                let body_str = self.write_expression(body, s);
                format!("while ({}) {}", cond_str, body_str)
            }
            agdb::api::Expression::Unknown(expr) => {
                panic!("Unknown expression: {expr}")
            }
        }
    }

    fn write_let(
        &self,
        s: &agdb::api::Struct,
        name: &&'static str,
        ty: &Option<fn() -> agdb::api::Type>,
        value: &Option<Box<agdb::api::Expression>>,
    ) -> String {
        let ty_str = if let Some(ty_fn) = ty {
            format!(": {}", ty_fn().name())
        } else {
            String::new()
        };
        let value_str = if let Some(val) = value {
            let value = if let agdb::api::Expression::If {
                condition,
                then_branch,
                else_branch,
            } = (*val).as_ref()
            {
                let then = if let agdb::api::Expression::Block(e) = (*then_branch).as_ref() {
                    self.write_expression(
                        e.first()
                            .expect("then condition should not be an empty block"),
                        s,
                    )
                } else {
                    self.write_expression(then_branch, s)
                };
                let else_branch = else_branch
                    .as_ref()
                    .expect("else condition should not be an empty block");
                let else_branch = if let agdb::api::Expression::Block(e) = (*else_branch).as_ref() {
                    self.write_expression(
                        e.first()
                            .expect("else condition should not be an empty block"),
                        s,
                    )
                } else {
                    self.write_expression(then_branch, s)
                };

                format!(
                    "{} ? {then} : {else_branch}",
                    self.write_expression(condition, s),
                )
            } else {
                self.write_expression(val, s)
            };
            format!(" = {value}")
        } else {
            String::new()
        };
        format!("let {}{}{}", name, ty_str, value_str)
    }

    fn write_format_string(&self, args: &[agdb::api::Expression], s: &agdb::api::Struct) -> String {
        if let agdb::api::Expression::Literal(agdb::api::LiteralValue::String(format_str)) = args[0]
        {
            let mut result = format!("`{format_str}`");
            result = result.replace("{", "${");

            for arg in &args[1..] {
                result = result.replacen(
                    "${}",
                    &format!("${{{}}}", &self.write_expression(arg, s)),
                    1,
                );
            }

            result
        } else {
            panic!("Invalid format string expression");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        Typescript::default().generate();
    }
}
