//! [`Emitter::types`]: the TS declarations of the serde types.

use super::Emitter;
use super::Ts;
use crate::codegen::lowered::*;
use crate::codegen::print;

impl Emitter<'_> {
    /// The declarations of the [wire](Module::wire) types of the module as
    /// serde represents them: a struct is an object of its fields, a
    /// newtype or a transparent struct its field, a tuple struct a tuple
    /// and a unit struct `null`; an enum is the union of its variants,
    /// tagged as the serde attributes say. A field is optional when it is
    /// an `Option` or a value may leave it out, an `Option` is `T | null`
    /// and the fields of a flattened field are those of the parent.
    pub fn types(mut self) -> Ts {
        let m = self.m;

        for (name, wire) in &m.wire {
            self.current = name.clone();

            match m.classes.get(name) {
                Some(c) => self.struct_declaration(c, wire),
                None => self.enum_declaration(&m.enums[name], wire),
            }
        }

        Ts {
            code: format!("{}\n\n{}", self.config.header, self.out),
            notes: self.notes,
            unsupported: self.unsupported,
            values: Vec::new(),
        }
    }

    fn struct_declaration(&mut self, c: &Class, wire: &Wire) {
        self.doc(0, wire.doc.as_deref());
        let generics = generics(&c.generics);
        let fields: Vec<(&String, &Ty, &Member)> = c
            .fields
            .iter()
            .filter_map(|(f, ty)| wire.members.get(f).filter(|m| !m.skip).map(|m| (f, ty, m)))
            .collect();
        let ty = match (wire.shape, fields.as_slice()) {
            (_, [(_, ty, _)]) if wire.transparent => self.wire_ty(ty),
            (_, _) if wire.transparent => {
                self.unsupported("a transparent struct without one field")
            }
            (Shape::Unit, _) => "null".into(),
            (Shape::Tuple, [(_, ty, _)]) if c.fields.len() == 1 => self.wire_ty(ty),
            (Shape::Tuple, fields) => {
                let items: Vec<String> = fields.iter().map(|(_, ty, _)| self.wire_ty(ty)).collect();
                format!("[{}]", items.join(", "))
            }
            (Shape::Named, fields)
                if fields.iter().all(|(_, _, m)| !m.flatten) && !fields.is_empty() =>
            {
                self.line(0, &format!("export interface {}{generics} {{", c.name));

                for (_, ty, m) in fields {
                    self.doc(1, m.doc.as_deref());
                    let property = self.property(ty, m);
                    self.line(1, &format!("{property};"));
                }

                self.line(0, "}\n");
                return;
            }
            (Shape::Named, fields) => self.object(fields, &[]),
        };

        self.line(0, &format!("export type {}{generics} = {ty};\n", c.name));
    }

    fn enum_declaration(&mut self, e: &Enum, wire: &Wire) {
        self.doc(0, wire.doc.as_deref());
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
            self.line(0, &format!("export type {} = never;\n", e.name));
            return;
        }

        self.line(0, &format!("export type {} =", e.name));
        let last = variants.len() - 1;

        for (i, (v, m)) in variants.into_iter().enumerate() {
            self.doc(1, m.doc.as_deref());
            let ty = self.variant(v, m, &wire.tagging);
            let end = if i == last { ";\n" } else { "" };
            self.line(1, &format!("| {ty}{end}"));
        }
    }

    /// A variant as its tagging represents it.
    fn variant(&mut self, v: &Variant, m: &Member, tagging: &Tagging) -> String {
        let name = format!("{:?}", m.name);

        match (tagging, v) {
            (Tagging::External, Variant::Unit(_)) => name,
            (Tagging::External, _) => {
                format!("{{ {}: {} }}", key(&m.name), self.payload(v, m))
            }
            (Tagging::Internal(tag) | Tagging::Adjacent(tag, _), Variant::Unit(_)) => {
                format!("{{ {}: {name} }}", key(tag))
            }
            (Tagging::Internal(tag), Variant::Struct(_, fields)) => {
                self.variant_object(fields, m, &[format!("{}: {name}", key(tag))])
            }
            (Tagging::Internal(_), Variant::Tuple(_, Ty::Tuple(_))) => {
                self.unsupported("an internally tagged tuple variant")
            }
            (Tagging::Internal(tag), Variant::Tuple(_, ty)) => {
                format!("{{ {}: {name} }} & {}", key(tag), self.wire_ty(ty))
            }
            (Tagging::Adjacent(tag, content), _) => {
                let payload = self.payload(v, m);
                format!("{{ {}: {name}; {}: {payload} }}", key(tag), key(content))
            }
            (Tagging::Untagged, _) => self.payload(v, m),
        }
    }

    /// The variant without its tag: `null` for a unit variant.
    fn payload(&mut self, v: &Variant, m: &Member) -> String {
        match v {
            Variant::Unit(_) => "null".into(),
            Variant::Tuple(_, ty) => self.wire_ty(ty),
            Variant::Struct(_, fields) => self.variant_object(fields, m, &[]),
        }
    }

    /// The object of the fields of a struct variant on one line, after the
    /// properties `first`.
    fn variant_object(&mut self, fields: &[(String, Ty)], v: &Member, first: &[String]) -> String {
        let fields: Vec<(&String, &Ty, &Member)> = fields
            .iter()
            .filter_map(|(f, ty)| v.fields.get(f).filter(|m| !m.skip).map(|m| (f, ty, m)))
            .collect();
        self.object(&fields, first)
    }

    /// The object of the fields on one line, after the properties `first`,
    /// and the intersection with the flattened ones.
    fn object(&mut self, fields: &[(&String, &Ty, &Member)], first: &[String]) -> String {
        let mut properties: Vec<String> = first.to_vec();
        let mut flattened = Vec::new();

        for (_, ty, m) in fields {
            if m.flatten {
                flattened.push(self.wire_ty(ty));
            } else {
                properties.push(self.property(ty, m));
            }
        }

        let object = match (properties.is_empty(), flattened.is_empty()) {
            (true, true) => Some("Record<string, never>".to_owned()),
            (true, false) => None,
            (false, _) => Some(format!("{{ {} }}", properties.join("; "))),
        };

        object
            .into_iter()
            .chain(flattened)
            .collect::<Vec<_>>()
            .join(" & ")
    }

    fn property(&mut self, ty: &Ty, m: &Member) -> String {
        let optional = if m.optional || matches!(ty, Ty::Option(_)) {
            "?"
        } else {
            ""
        };
        format!("{}{optional}: {}", key(&m.name), self.wire_ty(ty))
    }

    /// The serialized type: a named type is one of the wire types.
    fn wire_ty(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "null".into(),
            Ty::Bool => "boolean".into(),
            Ty::Int(_) | Ty::Float => "number".into(),
            Ty::Str => "string".into(),
            Ty::List(inner) => {
                let inner = self.wire_ty(inner);

                if inner.contains(' ') {
                    format!("({inner})[]")
                } else {
                    format!("{inner}[]")
                }
            }
            Ty::Option(inner) => format!("{} | null", self.wire_ty(inner)),
            Ty::Tuple(items) => {
                let items: Vec<String> = items.iter().map(|t| self.wire_ty(t)).collect();
                format!("[{}]", items.join(", "))
            }
            Ty::Named(name, args) if self.m.wire.contains_key(name) => {
                if args.is_empty() {
                    return name.clone();
                }

                let args: Vec<String> = args.iter().map(|t| self.wire_ty(t)).collect();
                format!("{name}<{}>", args.join(", "))
            }
            Ty::Param(p) => p.clone(),
            ty => self.unsupported(&format!("{} is no serde type", print::ty(ty))),
        }
    }

    fn doc(&mut self, indent: usize, doc: Option<&str>) {
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

fn generics(generics: &[Generic]) -> String {
    if generics.is_empty() {
        return String::new();
    }

    let names: Vec<&str> = generics.iter().map(|g| g.name.as_str()).collect();
    format!("<{}>", names.join(", "))
}

/// The property key of a name, quoted unless it is an identifier.
fn key(name: &str) -> String {
    let mut chars = name.chars();
    let identifier = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');

    if identifier {
        name.to_owned()
    } else {
        format!("{name:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::test_utilities::lower;
    use crate::codegen::ts::TsConfig;

    /// The declarations of the wire types `data` and what they reach,
    /// declared after `Root`.
    fn declared(source: &str, data: &[&str]) -> Ts {
        let (m, errors) = lower(&format!("pub struct Root;\n{source}"), &[], data);
        assert_eq!(errors, Vec::<String>::new());
        let config = TsConfig {
            header: "// header".into(),
            import: String::new(),
            schemas: String::new(),
        };
        Emitter::new(&m, &config).types()
    }

    mod emitter {
        use super::*;

        mod types {
            use super::*;

            #[test]
            fn types_should_declare_the_structs_as_serde_serializes_them() {
                let ts = declared(
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
                    (ts.code.as_str(), ts.unsupported),
                    (
                        r#"// header

export type Empty = Record<string, never>;

export type Flattened = { a: number } & Empty;

/**
 * A
 *
 * *\/ b
 */
export interface Named<T> {
    /** a */
    a: number;
    b?: string | null;
    c?: (boolean | null)[];
    "e-f": [number, number];
    g: T;
    h: Unit;
    i: null;
    _j: number[];
    $k: number;
}

export type Newtype = number;

export type OnlyFlattened = Newtype & Tuple;

export type Skipped = [number];

export type Transparent = Newtype;

export type Tuple = [number, Named<number>];

export type Unit = null;

"#,
                        Vec::<String>::new()
                    )
                );
            }

            #[test]
            fn types_should_declare_the_enums_in_their_tagging() {
                let ts = declared(
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
                    (ts.code.as_str(), ts.unsupported),
                    (
                        r#"// header

export type Adjacent =
    | { t: "A" }
    | { t: "B"; c: number }
    | { t: "C"; c: { a: number } };

export type Empty = never;

/** E */
export type External =
    /** a */
    | "A"
    | { B: number }
    | { C: [number, string] }
    | { D: { a: number } };

export interface Inner {
    b: number;
}

export type Internal =
    | { t: "A" }
    | { t: "B"; a: number }
    | { t: "C" } & Inner;

export type Skipped = never;

export type Untagged =
    | null
    | number
    | { a: number };

"#,
                        Vec::<String>::new()
                    )
                );
            }

            #[test]
            fn types_should_report_the_types_serde_does_not_serialize() {
                let ts = declared(
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
                    (ts.code.as_str(), ts.unsupported),
                    (
                        r#"// header

export type Tagged =
    | unsupported();

export type Two = unsupported();

export interface Uses {
    a: unsupported();
}

"#,
                        vec![
                            "Tagged: `an internally tagged tuple variant`".to_owned(),
                            "Two: `a transparent struct without one field`".to_owned(),
                            "Uses: `Plain is no serde type`".to_owned(),
                        ]
                    )
                );
            }
        }
    }
}
