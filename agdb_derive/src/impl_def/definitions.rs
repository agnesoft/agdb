use std::fmt::Display;

pub struct NamedType {
    pub name: String,
    pub ty: String,
}

pub struct NamedTypes(pub Vec<NamedType>);

pub struct Function {
    pub name: String,
    pub generics: NamedTypes,
    pub args: NamedTypes,
    pub ret: String,
    pub ret_generics: NamedTypes,
}

pub struct Functions {
    pub ty: String,
    pub generics: NamedTypes,
    pub functions: Vec<Function>,
}

impl Display for NamedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ty.is_empty() {
            return f.write_str(&self.name);
        } else {
            return f.write_fmt(format_args!("{}: {}", self.name, self.ty));
        }
    }
}

impl Display for NamedTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self
            .0
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        if !args.is_empty() {
            return f.write_str(&args);
        }

        Ok(())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!(
            "fn {name}<{generics}>({args}) -> {ret}<{ret_generics}>;",
            name = self.name,
            generics = self.generics.to_string(),
            args = self.args.to_string(),
            ret = self.ret,
            ret_generics = self.ret_generics.to_string()
        ));
    }
}

impl Display for Functions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let functions = self
            .functions
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        return f.write_fmt(format_args!(
            "impl<{generics}> {ty}<{generics}> {{\n{functions}\n}}",
            generics = self.generics.to_string(),
            ty = self.ty,
            functions = functions
        ));
    }
}
