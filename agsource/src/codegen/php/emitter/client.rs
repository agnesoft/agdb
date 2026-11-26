//! [`Emitter::client`]: the PHP client of an API crate.

use super::Emitter;
use super::Php;
use crate::codegen::lowered::*;

impl Emitter<'_> {
    /// The [client](crate::SourceCrate::lower_client) module: the consts,
    /// the traits as the types the target implements (an abstract class
    /// when they provide methods), the methods of the data types as
    /// functions and the builders as exported classes.
    pub fn client(mut self) -> Php {
        let m = self.m;
        self.bounded = true;

        // File header
        self.line(0, "<?php");
        self.line(0, &self.config.header.clone());
        if !self.config.namespace.is_empty() {
            self.line(0, "");
            self.line(0, &format!("namespace {};", self.config.namespace));
        }
        if !self.config.types_require.is_empty() {
            self.line(0, "");
            self.line(
                0,
                &format!(
                    "require_once __DIR__ . '/{}.php';",
                    self.config.types_require
                ),
            );
        }
        self.line(0, "");

        // Constants
        for (name, value) in &m.consts {
            self.current = name.clone();
            let value = self.expr(value);
            self.line(0, &format!("const {name} = {value};\n"));
        }

        // Traits as abstract classes or interfaces
        for (name, sigs) in &m.traits {
            self.trait_type(name, sigs);
        }

        // Data type methods as free functions
        self.data_methods();

        // Builder/client classes
        for b in &m.builders {
            self.class(&m.classes[b], true);
        }

        let head = String::new();
        let mut code = head + &self.out;
        // Trim trailing whitespace
        while code.ends_with("\n\n\n") {
            code.pop();
        }
        code.push('\n');

        Php {
            code,
            notes: self.notes,
            unsupported: self.unsupported,
            values: Vec::new(),
        }
    }

    fn trait_type(&mut self, name: &str, sigs: &[Func]) {
        let supers = self.m.supertraits.get(name).cloned().unwrap_or_default();

        if sigs.is_empty() {
            // Empty trait — just a marker, skip in PHP.
            return;
        }

        if sigs.iter().all(|f| f.body.is_empty()) {
            // All abstract methods → interface
            let extends = if supers.is_empty() {
                String::new()
            } else {
                format!(" extends {}", supers.join(", "))
            };
            self.line(0, &format!("interface {name}{extends} {{"));

            for f in sigs {
                let sig = self.abstract_signature(f);
                self.line(1, &format!("{sig};"));
            }

            self.line(0, "}\n");
            return;
        }

        // Has provided methods → abstract class
        if !supers.is_empty() {
            self.current = name.to_owned();
            self.unsupported("a trait with provided methods and supertraits");
        }

        self.line(0, &format!("abstract class {name} {{"));

        for f in sigs {
            if f.body.is_empty() {
                let sig = self.abstract_signature(f);
                self.line(1, &format!("{sig};"));
            } else {
                self.func(f, 1, "public ");
            }
        }

        self.line(0, "}\n");
    }

    /// The signature of an abstract method.
    fn abstract_signature(&mut self, f: &Func) -> String {
        self.current = f.name.clone();
        let params: Vec<String> = f
            .params
            .iter()
            .map(|(n, t)| format!("{} ${n}", self.ty(t)))
            .collect();
        let ret = self.return_ty(&f.ret);
        format!(
            "abstract public function {}({}): {ret}",
            f.name,
            params.join(", ")
        )
    }
}
