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

        for (name, wire) in &m.wire {
            self.current = name.clone();

            match m.classes.get(name) {
                Some(c) => self.struct_declaration(c, wire),
                None => self.enum_declaration(&m.enums[name], wire),
            }
        }

        Php {
            code: self.php_header() + &self.out,
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
            Ty::Option(inner) => {
                let inner_ty = self.wire_ty(inner);
                if inner_ty.starts_with("mixed") {
                    inner_ty
                } else {
                    format!("{inner_ty}|null")
                }
            }
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
            if inner.starts_with("mixed") {
                inner
            } else {
                format!("{inner}|null")
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::php::PhpConfig;
    use crate::codegen::test_utilities::lower;

    /// The declarations of the wire types `data` and what they reach,
    /// declared after `Root`.
    fn declared(source: &str, data: &[&str]) -> Php {
        let (m, errors) = lower(&format!("pub struct Root;\n{source}"), &[], data);
        assert_eq!(errors, Vec::<String>::new());
        let config = PhpConfig {
            header: "// header".into(),
            namespace: "Test".into(),
            types_require: "".into(),
        };
        Emitter::new(&m, &config).types()
    }

    mod emitter {
        use super::*;

        mod types {
            use super::*;

            #[test]
            fn types_should_declare_the_structs_as_serde_serializes_them() {
                let php = declared(
                    r#"/// A
///
/// */ b
#[derive(Serialize)]
pub struct Named<T> {
    /// a
    pub a: u8,
    pub b: Option<String>,
    #[serde(default)]
    pub c: Vec<Option<bool>>,
    #[serde(skip)]
    pub d: u8,
    #[serde(rename = "e-f")]
    pub e: (u8, f64),
    pub g: T,
    pub h: Unit,
    pub i: (),
    pub _j: Vec<u8>,
    #[serde(rename = "$k")]
    pub k: u8,
}
#[derive(Serialize)]
pub struct Unit;
#[derive(Serialize)]
pub struct Newtype(pub u8);
#[derive(Serialize)]
pub struct Tuple(pub u8, pub Named<u8>);
#[derive(Serialize)]
pub struct Skipped(pub u8, #[serde(skip)] pub u8);
#[derive(Serialize)]
#[serde(transparent)]
pub struct Transparent { pub a: Newtype }
#[derive(Serialize)]
pub struct Empty {}
#[derive(Serialize)]
pub struct Flattened { pub a: u8, #[serde(flatten)] pub b: Empty }
#[derive(Serialize)]
pub struct OnlyFlattened { #[serde(flatten)] pub a: Newtype, #[serde(flatten)] pub b: Tuple }"#,
                    &[
                        "Named",
                        "Skipped",
                        "Transparent",
                        "Flattened",
                        "OnlyFlattened",
                    ],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nclass Empty implements \\JsonSerializable {\n    public function jsonSerialize(): mixed { return new \\stdClass(); }\n}\n\nclass Flattened implements \\JsonSerializable {\n    public function __construct(\n        public int $a,\n        public Empty $b\n    ) {}\n\n    public function jsonSerialize(): mixed {\n        return [\n            'a' => $this->a,\n            'b' => $this->b instanceof \\JsonSerializable ? $this->b->jsonSerialize() : $this->b,\n        ];\n    }\n}\n\n/**\n * A\n *\n * *\\/ b\n */\nclass Named implements \\JsonSerializable {\n    public function __construct(\n        public int $a,\n        public string|null $b = null,\n        public array|null $c = null,\n        public array $e-f,\n        public mixed /* T */ $g,\n        public Unit $h,\n        public null $i,\n        public array $_j,\n        public int $$k\n    ) {}\n\n    public function jsonSerialize(): mixed {\n        return [\n            'a' => $this->a,\n            'b' => $this->b instanceof \\JsonSerializable ? $this->b->jsonSerialize() : $this->b,\n            'c' => array_map(fn($v) => $v instanceof \\JsonSerializable ? $v->jsonSerialize() : $v, $this->c),\n            'e-f' => $this->e-f,\n            'g' => $this->g,\n            'h' => $this->h instanceof \\JsonSerializable ? $this->h->jsonSerialize() : $this->h,\n            'i' => $this->i,\n            '_j' => array_map(fn($v) => $v instanceof \\JsonSerializable ? $v->jsonSerialize() : $v, $this->_j),\n            '$k' => $this->$k,\n        ];\n    }\n}\n\nclass Newtype implements \\JsonSerializable {\n    public function __construct(public int $_0) {}\n\n    public function jsonSerialize(): mixed {\n        return [$this->_0 instanceof \\JsonSerializable ? $this->_0->jsonSerialize() : $this->_0];\n    }\n}\n\nclass OnlyFlattened implements \\JsonSerializable {\n    public function __construct(\n        public Newtype $a,\n        public Tuple $b\n    ) {}\n\n    public function jsonSerialize(): mixed {\n        return [\n            'a' => $this->a instanceof \\JsonSerializable ? $this->a->jsonSerialize() : $this->a,\n            'b' => $this->b instanceof \\JsonSerializable ? $this->b->jsonSerialize() : $this->b,\n        ];\n    }\n}\n\nclass Skipped implements \\JsonSerializable {\n    public function __construct(public int $_0) {}\n\n    public function jsonSerialize(): mixed {\n        return [$this->_0 instanceof \\JsonSerializable ? $this->_0->jsonSerialize() : $this->_0];\n    }\n}\n\n/** Transparent wrapper for Newtype. */\nclass Transparent implements \\JsonSerializable {\n    public function __construct(public Newtype $value) {}\n\n    public function jsonSerialize(): mixed {\n        return $this->value instanceof \\JsonSerializable ? $this->value->jsonSerialize() : $this->value;\n    }\n}\n\nclass Tuple implements \\JsonSerializable {\n    public function __construct(public int $_0, public Named $_1) {}\n\n    public function jsonSerialize(): mixed {\n        return [$this->_0 instanceof \\JsonSerializable ? $this->_0->jsonSerialize() : $this->_0, $this->_1 instanceof \\JsonSerializable ? $this->_1->jsonSerialize() : $this->_1];\n    }\n}\n\n/** Unit type, serializes as null. */\nclass Unit implements \\JsonSerializable {\n    public function jsonSerialize(): mixed { return null; }\n}\n\n"
                );
            }

            #[test]
            fn types_should_declare_the_enums_in_their_tagging() {
                let php = declared(
                    r#"/// E
#[derive(Serialize)]
pub enum External {
    /// a
    A,
    B(u8),
    C(u8, String),
    D { a: u8, #[serde(skip)] b: u8 },
    #[serde(skip)]
    E,
}
#[derive(Serialize)]
#[serde(tag = "t")]
pub enum Internal { A, B { a: u8 }, C(Inner) }
#[derive(Serialize)]
#[serde(tag = "t", content = "c")]
pub enum Adjacent { A, B(u8), C { a: u8 } }
#[derive(Serialize)]
#[serde(untagged)]
pub enum Untagged { A, B(u8), C { a: u8 } }
#[derive(Serialize)]
pub enum Empty {}
#[derive(Serialize)]
pub enum Skipped { #[serde(skip)] A }
#[derive(Serialize)]
pub struct Inner { pub b: u8 }"#,
                    &[
                        "External", "Internal", "Adjacent", "Untagged", "Empty", "Skipped",
                    ],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nclass Adjacent implements \\JsonSerializable {\n    private function __construct(private string $tag, private mixed $value) {}\n\n    public static function A(): self { return new self('A', null); }\n    public static function B(int $value): self { return new self('B', $value); }\n    public static function C(int $a): self { return new self('C', ['a' => $a]); }\n\n    public function getTag(): string { return $this->tag; }\n\n    public function getValue(): mixed { return $this->value; }\n\n    public function jsonSerialize(): mixed {\n        if ($this->value === null) {\n            return ['t' => $this->tag];\n        }\n        return ['t' => $this->tag, 'c' => $this->value instanceof \\JsonSerializable ? $this->value->jsonSerialize() : $this->value];\n    }\n}\n\n/** E */\nclass External implements \\JsonSerializable {\n    private function __construct(private string $tag, private mixed $value) {}\n\n    /** a */\n    public static function A(): self { return new self('A', null); }\n    public static function B(int $value): self { return new self('B', $value); }\n    public static function C(array $value): self { return new self('C', $value); }\n    public static function D(int $a): self { return new self('D', ['a' => $a]); }\n\n    public function getTag(): string { return $this->tag; }\n\n    public function getValue(): mixed { return $this->value; }\n\n    public function jsonSerialize(): mixed {\n        if ($this->value === null) {\n            return $this->tag;\n        }\n        return [$this->tag => $this->value instanceof \\JsonSerializable ? $this->value->jsonSerialize() : $this->value];\n    }\n}\n\nclass Inner implements \\JsonSerializable {\n    public function __construct(public int $b) {}\n\n    public function jsonSerialize(): mixed {\n        return [\n            'b' => $this->b,\n        ];\n    }\n}\n\nclass Internal implements \\JsonSerializable {\n    private function __construct(private string $tag, private mixed $value) {}\n\n    public static function A(): self { return new self('A', null); }\n    public static function B(int $a): self { return new self('B', ['a' => $a]); }\n    public static function C(Inner $value): self { return new self('C', $value); }\n\n    public function getTag(): string { return $this->tag; }\n\n    public function getValue(): mixed { return $this->value; }\n\n    public function jsonSerialize(): mixed {\n        if (is_array($this->value)) {\n            return array_merge(['t' => $this->tag], $this->value);\n        }\n        return ['t' => $this->tag];\n    }\n}\n\nclass Untagged implements \\JsonSerializable {\n    private function __construct(private string $tag, private mixed $value) {}\n\n    public static function A(): self { return new self('A', null); }\n    public static function B(int $value): self { return new self('B', $value); }\n    public static function C(int $a): self { return new self('C', ['a' => $a]); }\n\n    public function getTag(): string { return $this->tag; }\n\n    public function getValue(): mixed { return $this->value; }\n\n    public function jsonSerialize(): mixed {\n        if ($this->value === null) {\n            return null;\n        }\n        return $this->value instanceof \\JsonSerializable ? $this->value->jsonSerialize() : $this->value;\n    }\n}\n\n"
                );
            }

            #[test]
            fn types_should_declare_the_unit_enums_as_backed_enums_and_the_scalars_and_options() {
                let php = declared(
                    r#"#[derive(Serialize)]
pub enum Kind {
    /// a
    A,
    #[serde(rename = "bee")]
    B,
}
#[derive(Serialize)]
pub struct Scalars<T> { pub a: bool, pub b: f64, pub c: Option<T> }
#[derive(Serialize)]
pub struct Options<T>(pub Option<T>, pub Option<u8>);"#,
                    &["Kind", "Scalars", "Options"],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nenum Kind: string {\n    /** a */\n    case A = 'A';\n    case B = 'bee';\n}\n\nclass Options implements \\JsonSerializable {\n    public function __construct(public mixed /* T */ $_0, public int|null $_1) {}\n\n    public function jsonSerialize(): mixed {\n        return [$this->_0 instanceof \\JsonSerializable ? $this->_0->jsonSerialize() : $this->_0, $this->_1 instanceof \\JsonSerializable ? $this->_1->jsonSerialize() : $this->_1];\n    }\n}\n\nclass Scalars implements \\JsonSerializable {\n    public function __construct(\n        public bool $a,\n        public float $b,\n        public mixed /* T */ $c = null\n    ) {}\n\n    public function jsonSerialize(): mixed {\n        return [\n            'a' => $this->a,\n            'b' => $this->b,\n            'c' => $this->c instanceof \\JsonSerializable ? $this->c->jsonSerialize() : $this->c,\n        ];\n    }\n}\n\n"
                );
                assert_eq!(php.unsupported, Vec::<String>::new());
            }

            #[test]
            fn types_should_report_the_types_serde_does_not_serialize() {
                let php = declared(
                    r#"#[derive(Serialize)]
#[serde(transparent)]
pub struct Two { pub a: u8, pub b: u8 }
#[derive(Serialize)]
#[serde(tag = "t")]
pub enum Tagged { A(u8, u8) }
#[derive(Serialize)]
pub struct Uses { pub a: Plain }
pub struct Plain { pub a: u8 }"#,
                    &["Two", "Tagged", "Uses"],
                );

                assert_eq!(
                    php.code,
                    "<?php\n// header\n\nnamespace Test;\n\nclass Tagged implements \\JsonSerializable {\n    private function __construct(private string $tag, private mixed $value) {}\n\n    public static function A(array $value): self { return new self('A', $value); }\n\n    public function getTag(): string { return $this->tag; }\n\n    public function getValue(): mixed { return $this->value; }\n\n    public function jsonSerialize(): mixed {\n        if (is_array($this->value)) {\n            return array_merge(['t' => $this->tag], $this->value);\n        }\n        return ['t' => $this->tag];\n    }\n}\n\nclass Uses implements \\JsonSerializable {\n    public function __construct(public unsupported() $a) {}\n\n    public function jsonSerialize(): mixed {\n        return [\n            'a' => $this->a instanceof \\JsonSerializable ? $this->a->jsonSerialize() : $this->a,\n        ];\n    }\n}\n\n"
                );
                assert_eq!(
                    php.unsupported,
                    vec![
                        "Two: `a transparent struct without one field`".to_owned(),
                        "Uses: `Plain is no serde type`".to_owned(),
                    ]
                );
            }
        }
    }
}
