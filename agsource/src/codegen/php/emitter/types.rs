//! [`Emitter::types`]: the PHP declarations of the serde types.

use super::Emitter;
use super::Php;
use crate::codegen::lowered::*;

impl Emitter<'_> {
    /// The declarations of the [wire](Module::wire) types: structs as classes
    /// with typed properties and `\JsonSerializable`, unit-only enums as PHP
    /// 8.1 backed enums, and payload enums as classes with static factory
    /// methods.
    pub fn types(mut self) -> Php {
        let m = self.m;

        self.line(0, "<?php");
        self.line(0, &self.config.header.clone());
        if !self.config.namespace.is_empty() {
            self.line(0, "");
            self.line(0, &format!("namespace {};", self.config.namespace));
        }
        self.line(0, "");

        for (name, wire) in &m.wire {
            self.current = name.clone();

            match m.classes.get(name) {
                Some(c) => self.struct_declaration(c, wire),
                None => self.enum_declaration(&m.enums[name], wire),
            }
        }

        Php {
            code: self.out,
            notes: self.notes,
            unsupported: self.unsupported,
            values: Vec::new(),
        }
    }

    fn struct_declaration(&mut self, c: &Class, wire: &Wire) {
        self.php_doc(0, wire.doc.as_deref());
        let fields: Vec<(&String, &Ty, &Member)> = c
            .fields
            .iter()
            .filter_map(|(f, ty)| wire.members.get(f).filter(|m| !m.skip).map(|m| (f, ty, m)))
            .collect();

        match (wire.shape, fields.as_slice()) {
            (_, [(_, ty, _)]) if wire.transparent => {
                // A transparent newtype: emit a type alias comment and a class
                // wrapping the inner type.
                let inner = self.wire_ty(ty);
                self.line(0, &format!("/** Transparent wrapper for {inner}. */"));
                self.line(
                    0,
                    &format!("class {} implements \\JsonSerializable {{", c.name),
                );
                self.line(
                    1,
                    &format!("public function __construct(public {inner} $value) {{}}"),
                );
                self.line(0, "");
                self.line(1, "public function jsonSerialize(): mixed {");
                self.line(2, "return $this->value instanceof \\JsonSerializable ? $this->value->jsonSerialize() : $this->value;");
                self.line(1, "}");
                self.line(0, "}");
                self.line(0, "");
            }
            (_, _) if wire.transparent => {
                self.unsupported("a transparent struct without one field");
            }
            (Shape::Unit, _) => {
                self.line(0, "/** Unit type, serializes as null. */");
                self.line(
                    0,
                    &format!("class {} implements \\JsonSerializable {{", c.name),
                );
                self.line(1, "public function jsonSerialize(): mixed { return null; }");
                self.line(0, "}");
                self.line(0, "");
            }
            (Shape::Tuple, fields) => {
                self.line(
                    0,
                    &format!("class {} implements \\JsonSerializable {{", c.name),
                );
                // Store as positional array
                let params: Vec<String> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, (_, ty, _))| format!("public {} $_{i}", self.wire_ty(ty)))
                    .collect();
                self.line(
                    1,
                    &format!("public function __construct({}) {{}}", params.join(", ")),
                );
                self.line(0, "");
                self.line(1, "public function jsonSerialize(): mixed {");
                let items: Vec<String> = (0..fields.len())
                    .map(|i| format!("$this->_{i} instanceof \\JsonSerializable ? $this->_{i}->jsonSerialize() : $this->_{i}"))
                    .collect();
                self.line(2, &format!("return [{}];", items.join(", ")));
                self.line(1, "}");
                self.line(0, "}");
                self.line(0, "");
            }
            (Shape::Named, fields) if !fields.is_empty() => {
                self.line(
                    0,
                    &format!("class {} implements \\JsonSerializable {{", c.name),
                );

                // Constructor with promoted properties
                if fields.len() == 1 {
                    let (_, ty, m) = fields[0];
                    let php_ty = self.constructor_ty(ty, m);
                    let name = &m.name;
                    let default = self.default_value(ty, m);
                    self.line(
                        1,
                        &format!(
                            "public function __construct(public {php_ty} ${name}{default}) {{}}"
                        ),
                    );
                } else {
                    self.line(1, "public function __construct(");
                    let last = fields.len() - 1;
                    for (i, (_, ty, m)) in fields.iter().enumerate() {
                        let php_ty = self.constructor_ty(ty, m);
                        let name = &m.name;
                        let default = self.default_value(ty, m);
                        let comma = if i == last { "" } else { "," };
                        self.line(2, &format!("public {php_ty} ${name}{default}{comma}"));
                    }
                    self.line(1, ") {}");
                }

                self.line(0, "");

                // jsonSerialize
                self.line(1, "public function jsonSerialize(): mixed {");
                if fields.len() == 1 {
                    let (_, ty, m) = fields[0];
                    let name = &m.name;
                    self.line(2, "return [");
                    self.line(
                        3,
                        &format!("'{}' => {},", name, self.serialize_field(name, ty)),
                    );
                    self.line(2, "];");
                } else {
                    self.line(2, "return [");
                    for (_, ty, m) in fields {
                        let name = &m.name;
                        self.line(
                            3,
                            &format!("'{}' => {},", name, self.serialize_field(name, ty)),
                        );
                    }
                    self.line(2, "];");
                }
                self.line(1, "}");

                self.line(0, "}");
                self.line(0, "");
            }
            (Shape::Named, _) => {
                // Empty struct
                self.line(
                    0,
                    &format!("class {} implements \\JsonSerializable {{", c.name),
                );
                self.line(
                    1,
                    "public function jsonSerialize(): mixed { return new \\stdClass(); }",
                );
                self.line(0, "}");
                self.line(0, "");
            }
        }
    }

    fn enum_declaration(&mut self, e: &Enum, wire: &Wire) {
        self.php_doc(0, wire.doc.as_deref());
        let variants: Vec<(&Variant, &Member)> = e
            .variants
            .iter()
            .filter_map(|v| {
                wire.members
                    .get(v.name())
                    .filter(|m| !m.skip)
                    .map(|m| (v, m))
            })
            .collect();

        if variants.is_empty() {
            // No variants — emit nothing.
            return;
        }

        // Check if all variants are unit (no payloads) → backed enum.
        let all_unit = variants.iter().all(|(v, _)| matches!(v, Variant::Unit(_)));

        if all_unit {
            self.backed_enum(&e.name, &variants, wire);
        } else {
            self.factory_enum(&e.name, &variants, wire);
        }
    }

    /// A unit-only enum as a PHP 8.1 backed string enum.
    fn backed_enum(&mut self, name: &str, variants: &[(&Variant, &Member)], _wire: &Wire) {
        self.line(0, &format!("enum {name}: string {{"));

        for (v, m) in variants {
            self.php_doc(1, m.doc.as_deref());
            self.line(1, &format!("case {} = '{}';", v.name(), m.name));
        }

        self.line(0, "}");
        self.line(0, "");
    }

    /// A payload enum as a class with static factory methods, a private
    /// constructor, and `\JsonSerializable` using serde's tagging.
    fn factory_enum(&mut self, name: &str, variants: &[(&Variant, &Member)], wire: &Wire) {
        self.line(0, &format!("class {name} implements \\JsonSerializable {{"));
        self.line(
            1,
            "private function __construct(private string $tag, private mixed $value) {}",
        );
        self.line(0, "");

        // Static factory methods
        for (v, m) in variants {
            self.php_doc(1, m.doc.as_deref());
            match v {
                Variant::Unit(var) => {
                    self.line(1, &format!(
                        "public static function {var}(): self {{ return new self('{}', null); }}",
                        m.name
                    ));
                }
                Variant::Tuple(var, ty) => {
                    let php_ty = self.wire_ty(ty);
                    self.line(1, &format!(
                        "public static function {var}({php_ty} $value): self {{ return new self('{}', $value); }}",
                        m.name
                    ));
                }
                Variant::Struct(var, fields) => {
                    let params: Vec<String> = fields
                        .iter()
                        .filter_map(|(f, ty)| {
                            m.fields
                                .get(f)
                                .filter(|fm| !fm.skip)
                                .map(|fm| format!("{} ${}", self.wire_ty(ty), fm.name))
                        })
                        .collect();
                    let keys: Vec<String> = fields
                        .iter()
                        .filter_map(|(f, _)| {
                            m.fields
                                .get(f)
                                .filter(|fm| !fm.skip)
                                .map(|fm| format!("'{}' => ${}", fm.name, fm.name))
                        })
                        .collect();
                    self.line(1, &format!(
                        "public static function {var}({}): self {{ return new self('{}', [{}]); }}",
                        params.join(", "),
                        m.name,
                        keys.join(", ")
                    ));
                }
            }
        }

        self.line(0, "");

        // Getter for the tag
        self.line(1, "public function getTag(): string { return $this->tag; }");
        self.line(0, "");

        // Getter for the value
        self.line(
            1,
            "public function getValue(): mixed { return $this->value; }",
        );
        self.line(0, "");

        // jsonSerialize based on tagging
        self.line(1, "public function jsonSerialize(): mixed {");
        let serialize_value = "$this->value instanceof \\JsonSerializable ? $this->value->jsonSerialize() : $this->value";
        match &wire.tagging {
            Tagging::External => {
                self.line(2, "if ($this->value === null) {");
                self.line(3, "return $this->tag;");
                self.line(2, "}");
                self.line(2, &format!("return [$this->tag => {serialize_value}];"));
            }
            Tagging::Internal(tag) => {
                self.line(2, "if (is_array($this->value)) {");
                self.line(
                    3,
                    &format!("return array_merge(['{tag}' => $this->tag], $this->value);"),
                );
                self.line(2, "}");
                self.line(2, &format!("return ['{tag}' => $this->tag];"));
            }
            Tagging::Adjacent(tag, content) => {
                self.line(2, "if ($this->value === null) {");
                self.line(3, &format!("return ['{tag}' => $this->tag];"));
                self.line(2, "}");
                self.line(
                    2,
                    &format!("return ['{tag}' => $this->tag, '{content}' => {serialize_value}];"),
                );
            }
            Tagging::Untagged => {
                self.line(2, "if ($this->value === null) {");
                self.line(3, "return null;");
                self.line(2, "}");
                self.line(2, &format!("return {serialize_value};"));
            }
        }
        self.line(1, "}");
        self.line(0, "}");
        self.line(0, "");
    }

    /// The PHP type for a wire type.
    pub(crate) fn wire_ty(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "null".into(),
            Ty::Bool => "bool".into(),
            Ty::Int(_) => "int".into(),
            Ty::Float => "float".into(),
            Ty::Str => "string".into(),
            Ty::List(_) => "array".into(),
            Ty::Option(inner) => format!("{}|null", self.wire_ty(inner)),
            Ty::Tuple(_) => "array".into(),
            Ty::Named(name, _) if self.m.wire.contains_key(name) => name.clone(),
            Ty::Param(p) => format!("mixed /* {p} */"),
            ty => self.unsupported(&format!(
                "{} is no serde type",
                crate::codegen::print::ty(ty)
            )),
        }
    }

    /// The PHP constructor parameter type for a field, handling optionality.
    fn constructor_ty(&mut self, ty: &Ty, m: &Member) -> String {
        if m.optional || matches!(ty, Ty::Option(_)) {
            let inner = match ty {
                Ty::Option(inner) => self.wire_ty(inner),
                _ => self.wire_ty(ty),
            };
            format!("{inner}|null")
        } else {
            self.wire_ty(ty)
        }
    }

    /// The default value for a constructor parameter.
    fn default_value(&self, ty: &Ty, m: &Member) -> String {
        if m.optional || matches!(ty, Ty::Option(_)) {
            " = null".into()
        } else {
            String::new()
        }
    }

    /// The PHP expression to serialize a field value.
    fn serialize_field(&self, name: &str, ty: &Ty) -> String {
        match ty {
            Ty::Named(..) | Ty::Option(_) => {
                format!(
                    "$this->{name} instanceof \\JsonSerializable ? $this->{name}->jsonSerialize() : $this->{name}"
                )
            }
            Ty::List(_) => {
                format!(
                    "array_map(fn($v) => $v instanceof \\JsonSerializable ? $v->jsonSerialize() : $v, $this->{name})"
                )
            }
            _ => format!("$this->{name}"),
        }
    }

    /// A PHP doc comment.
    fn php_doc(&mut self, indent: usize, doc: Option<&str>) {
        let Some(doc) = doc else {
            return;
        };
        let doc = doc.replace("*/", "*\\/");
        let lines: Vec<&str> = doc.lines().collect();

        if let [line] = lines.as_slice() {
            self.line(indent, &format!("/** {line} */"));
            return;
        }

        self.line(indent, "/**");

        for line in lines {
            self.line(
                indent,
                &format!(" *{}{line}", if line.is_empty() { "" } else { " " }),
            );
        }

        self.line(indent, " */");
    }
}
