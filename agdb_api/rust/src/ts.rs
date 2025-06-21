use agdb::api::API;
use agdb::api::ApiType;
use agdb::api::Enum;

#[allow(dead_code, clippy::upper_case_acronyms)]
struct TS {
    buffer: String,
}

impl TS {
    fn new() -> Self {
        let mut buffer = String::new();
        buffer.push_str("declare namespace agdb {\n");

        TS { buffer }
    }

    fn finalize(mut self) -> String {
        self.buffer.push_str("}\n");
        self.buffer
    }

    fn add_enum_def(&mut self, e: &'static Enum) {
        self.buffer
            .push_str(&format!("export interface {} {{\n", e.name));

        for v in &e.variants {
            self.buffer
                .push_str(&format!("    {}: {},\n", v.name, Self::name(&(v.ty)())));
        }

        self.buffer.push_str("}\n");
    }

    fn add_struct_def(&mut self, s: &'static agdb::api::Struct) {
        self.buffer.push_str(&format!("export type {} = ", s.name));

        if !s.fields.is_empty() && s.fields[0].name.is_empty() {
            self.buffer.push_str(&Self::name(&(s.fields[0].ty)()));
            self.buffer.push_str(";\n");
        } else {
            self.buffer.push_str("{\n");
            for f in &s.fields {
                self.buffer
                    .push_str(&format!("    {}: {},\n", f.name, Self::name(&(f.ty)())));
            }
            self.buffer.push_str("}\n");
        }
    }

    fn add_type_def(&mut self, ty: ApiType) {
        match ty.ty {
            agdb::api::Type::Enum(e) => self.add_enum_def(e),
            agdb::api::Type::Struct(s) => self.add_struct_def(s),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ts_api() {
        let api = API::def();
        let mut ts = TS::new();

        for ty in api.types {
            ts.add_type_def(ty);
        }

        std::fs::write("../../api.ts", ts.finalize()).unwrap();
    }
}
