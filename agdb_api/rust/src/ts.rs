use agdb::api::API;
use agdb::api::ApiType;
use agdb::api::Enum;

#[allow(dead_code, clippy::upper_case_acronyms)]
struct TS {
    buffer: String,
    indent: usize,
}

impl TS {
    fn new() -> Self {
        let mut buffer = String::new();
        buffer.push_str("declare namespace agdb {\n");

        TS { buffer, indent: 0 }
    }

    fn finalize(mut self) -> String {
        self.print("}\n");
        self.buffer
    }

    fn add_body(&mut self, expressions: &[agdb::api::Expression], has_return: bool) {
        if has_return {
            for expr in expressions[..expressions.len() - 1].iter() {
                self.add_expression(expr);
            }
            if let Some(expr) = expressions.last() {
                self.print_indent("return ");
                self.add_expression(expr);
            }
        } else {
            for expr in expressions {
                self.add_expression(expr);
            }
        }
    }

    fn add_enum_def(&mut self, e: &'static Enum) {
        let variants = e
            .variants
            .iter()
            .map(|v| Self::name(&(v.ty)()))
            .collect::<Vec<_>>();
        self.print(&format!(
            "export type {} = {};\n",
            e.name,
            variants.join(" | ")
        ));
    }

    fn add_sub_expression(&mut self, e: &agdb::api::Expression) {
        match e {
            agdb::api::Expression::Array { elements } => {
                self.print("[");
                for e in elements {
                    self.add_sub_expression(e);
                }
                self.print("]");
            }
            agdb::api::Expression::Assign { target, value } => {
                self.add_sub_expression(target);
                self.print(" = ");
                self.add_sub_expression(value);
            }
            agdb::api::Expression::Binary { op, left, right } => {
                self.add_sub_expression(left);
                self.print(&format!(" {op} "));
                self.add_sub_expression(right);
            }
            agdb::api::Expression::Block(expressions) => {
                self.print("{\n");
                self.add_body(expressions, false);
                self.print("}\n");
            }
            agdb::api::Expression::Call {
                recipient,
                function,
                args,
            } => {
                if let Some(r) = recipient {
                    self.add_sub_expression(r);
                    self.print(".");
                }
                self.print(function);
                self.print("(");
                for (i, arg) in args.iter().enumerate() {
                    self.add_sub_expression(arg);
                    if i < args.len() - 1 {
                        self.print(", ");
                    }
                }
                self.print(")");
            }
            agdb::api::Expression::Closure { ret, body } => {
                let ret = if let Some(r) = ret {
                    format!(": {}", Self::name(&(r)()))
                } else {
                    "()".to_string()
                };
                self.print(&format!("(): {ret} => {{\n"));
                self.indent += 1;
                self.add_body(body, false);
                self.indent -= 1;
                self.print("}\n");
            }
            agdb::api::Expression::FieldAccess { base, field } => {
                self.add_sub_expression(base);
                self.print(".");
                self.print(field);
            }
            agdb::api::Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.print("if (");
                self.add_sub_expression(condition);
                self.print(") {\n");
                self.indent += 1;
                self.add_expression(then_branch);
                self.indent -= 1;
                self.print("}");

                if let Some(else_branch) = else_branch {
                    self.print(" else {\n");
                    self.indent += 1;
                    self.add_expression(else_branch);
                    self.indent -= 1;
                    self.print("}");
                }
            }
            agdb::api::Expression::Index { base, index } => {
                self.add_sub_expression(base);
                self.print("[");
                self.add_sub_expression(index);
                self.print("]");
            }
            agdb::api::Expression::Let { name, ty, value } => {
                let type_annotation = if let Some(t) = ty {
                    format!(": {}", Self::name(&(t)()))
                } else {
                    String::new()
                };
                self.print(&format!("const {name}{type_annotation} = "));
                self.add_sub_expression(value);
            }
            agdb::api::Expression::Literal(ty) => match ty {
                agdb::api::LiteralValue::I64(v) => self.print(v),
                agdb::api::LiteralValue::F64(v) => self.print(v),
                agdb::api::LiteralValue::String(v) => self.print(v),
                agdb::api::LiteralValue::Bool(v) => self.print(if *v { "true" } else { "false" }),
            },
            agdb::api::Expression::Return(expression) => {
                self.print("return ");
                if let Some(expr) = expression {
                    self.add_sub_expression(expr);
                }
            }
            agdb::api::Expression::Struct { name, fields } => {
                self.print(&format!("new {name}"));
                if fields.is_empty() {
                    self.print(";\n");
                } else {
                    self.print(" {\n");
                    for field in fields {
                        self.print(&format!("    {}: ", field.0));
                        self.add_sub_expression(&field.1);
                        self.print(",\n");
                    }
                    self.print("}\n");
                }
            }
            agdb::api::Expression::Unary { op, expr } => {
                self.print(&format!("{op}"));
                self.add_sub_expression(expr);
            }
            agdb::api::Expression::Variable(v) => self.print(v),
            agdb::api::Expression::While { condition, body } => {
                self.print("while (");
                self.add_sub_expression(condition);
                self.print(") {\n");
                self.add_expression(body);
                self.print("}\n");
            }
            agdb::api::Expression::Unknown(e) => panic!("Unhandled expression: {e}"),
        }
    }

    fn add_expression(&mut self, expr: &agdb::api::Expression) {
        self.add_sub_expression(expr);
    }

    fn add_function(&mut self, f: &agdb::api::Function) {
        let ret_val = if let Some(ret) = &f.ret {
            let ret_val_name = Self::name(&(ret)());
            format!(": {ret_val_name}")
        } else {
            String::new()
        };

        self.print_indent(&format!(
            "{}({}){} {{\n",
            f.name,
            f.args
                .iter()
                .map(|a| format!("{}: {}", a.name, Self::name(&(a.ty)())))
                .collect::<Vec<_>>()
                .join(", "),
            ret_val
        ));
        self.indent += 1;
        self.add_body(&f.expressions, f.ret.is_some());
        self.indent -= 1;
        self.print("    }\n");
    }

    fn add_struct_def(&mut self, s: &'static agdb::api::Struct, functions: &[agdb::api::Function]) {
        if !functions.is_empty() {
            self.add_class_def(s, functions);
        } else {
            self.print(&format!("export type {} = ", s.name));

            if let Some(ty) = s.fields.first()
                && ty.name.is_empty()
            {
                self.print(&Self::name(&(ty.ty)()));
                self.print(";\n");
            } else {
                self.print("{\n");
                for f in &s.fields {
                    self.print_indent(&format!("{}: {};\n", f.name, Self::name(&(f.ty)())));
                }
                self.print("}\n");
            }
        }
    }

    fn add_class_def(&mut self, s: &'static agdb::api::Struct, functions: &[agdb::api::Function]) {
        self.print(&format!("export class {} {{\n", s.name));

        for f in &s.fields {
            let name = if f.name.is_empty() { "data" } else { f.name };
            self.print_indent(&format!("{name}: {};\n", Self::name(&(f.ty)())));
        }

        self.print("\n");

        for f in functions {
            self.add_function(f);
        }

        self.print("}\n");
    }

    fn add_type_def(&mut self, ty: &ApiType, functions: &[agdb::api::Function]) {
        match ty.ty {
            agdb::api::Type::Enum(e) => self.add_enum_def(e),
            agdb::api::Type::Struct(s) => self.add_struct_def(s, functions),
            _ => {}
        }
    }

    fn name(ty: &agdb::api::Type) -> String {
        match ty {
            agdb::api::Type::None => "undefined".to_string(),
            agdb::api::Type::U8
            | agdb::api::Type::I64
            | agdb::api::Type::U64
            | agdb::api::Type::F64 => "number".to_string(),
            agdb::api::Type::String => "string".to_string(),
            agdb::api::Type::User => "any".to_string(),
            agdb::api::Type::Enum(e) => e.name.to_string(),
            agdb::api::Type::Struct(s) => s.name.to_string(),
            agdb::api::Type::List(l) => format!("{}[]", Self::name(&l)),
            agdb::api::Type::Option(o) => format!("{} | null", Self::name(&o)),
        }
    }

    fn print(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    fn print_indent(&mut self, s: &str) {
        let indent = (self.indent + 1) * 4;
        self.buffer
            .push_str(&format!("{:indent$}{s}", "", indent = indent));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ts_api() {
        let api = API::def();
        let mut ts = TS::new();

        for ty in &api.types {
            ts.add_type_def(ty, &ty.functions);
        }

        std::fs::write("../../api.ts", ts.finalize()).unwrap();
    }
}
