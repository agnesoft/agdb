//! The [`Wire`] representation of a data type from its serde attributes.

use super::ir;
use super::lowered::Member;
use super::lowered::Shape;
use super::lowered::Tagging;
use super::lowered::Wire;
use std::collections::BTreeMap;

/// Whether the attributes derive serde's `Serialize` or `Deserialize`.
pub(super) fn is_serde(attrs: &[ir::Attr]) -> bool {
    attrs.iter().any(|a| {
        let Some(list) = list(a, "derive") else {
            return false;
        };
        let mut serde = false;
        let _ = list.parse_nested_meta(|meta| {
            serde |= meta
                .path
                .segments
                .last()
                .is_some_and(|s| s.ident == "Serialize" || s.ident == "Deserialize");
            Ok(())
        });
        serde
    })
}

/// The representation of a struct.
pub(super) fn of_struct(attrs: &[ir::Attr], s: &ir::Struct) -> Result<Wire, String> {
    let container = Container::of(attrs)?;

    if container.tagging != Tagging::External {
        return Err("serde tagging of a struct".into());
    }

    let shape = match s.fields {
        ir::Fields::Named(_) => Shape::Named,
        ir::Fields::Unnamed(_) => Shape::Tuple,
        ir::Fields::Unit => Shape::Unit,
    };

    Ok(Wire {
        doc: doc(attrs),
        shape,
        tagging: Tagging::External,
        transparent: container.transparent,
        members: fields(&s.fields, container.rename_all.as_deref())?,
    })
}

/// The representation of an enum.
pub(super) fn of_enum(attrs: &[ir::Attr], e: &ir::Enum) -> Result<Wire, String> {
    let container = Container::of(attrs)?;

    if container.transparent {
        return Err("serde `transparent` on an enum".into());
    }

    let mut members = BTreeMap::new();

    for v in &e.variants {
        let mut member = Member::of(&v.attrs, &v.name, |n| {
            rename_variant(n, container.rename_all.as_deref())
        })?;

        if member.optional || member.flatten {
            return Err(format!(
                "serde `default` or `flatten` on variant `{}`",
                v.name
            ));
        }

        member.fields = fields(&v.fields, None)?;
        members.insert(v.name.clone(), member);
    }

    Ok(Wire {
        doc: doc(attrs),
        shape: Shape::Named,
        tagging: container.tagging,
        transparent: false,
        members,
    })
}

/// The doc comment of the attributes, without the space after `///`.
pub(super) fn doc(attrs: &[ir::Attr]) -> Option<String> {
    let lines: Vec<&str> = attrs
        .iter()
        .filter_map(|a| match a {
            ir::Attr::Doc(d) => Some(d.strip_prefix(' ').unwrap_or(d)),
            ir::Attr::Meta(_) => None,
        })
        .collect();

    (!lines.is_empty()).then(|| lines.join("\n").trim_end().to_owned())
}

fn fields(
    fields: &ir::Fields,
    rename_all: Option<&str>,
) -> Result<BTreeMap<String, Member>, String> {
    let fields = match fields {
        ir::Fields::Named(fields) | ir::Fields::Unnamed(fields) => fields.as_slice(),
        ir::Fields::Unit => &[],
    };
    let mut members = BTreeMap::new();

    for (i, f) in fields.iter().enumerate() {
        let name = f.name.clone().unwrap_or_else(|| i.to_string());
        let member = Member::of(&f.attrs, &name, |n| rename_field(n, rename_all))?;
        members.insert(name, member);
    }

    Ok(members)
}

/// The container attributes of a struct or an enum.
#[derive(Default)]
struct Container {
    rename_all: Option<String>,
    tagging: Tagging,
    transparent: bool,
}

impl Container {
    fn of(attrs: &[ir::Attr]) -> Result<Container, String> {
        let mut c = Container::default();
        let mut tag = None;
        let mut content = None;
        let mut untagged = false;

        for list in attrs.iter().filter_map(|a| list(a, "serde")) {
            list.parse_nested_meta(|meta| {
                let key = key(&meta.path);

                match key.as_str() {
                    "rename_all" => c.rename_all = Some(string(&meta)?),
                    "tag" => tag = Some(string(&meta)?),
                    "content" => content = Some(string(&meta)?),
                    "untagged" => untagged = true,
                    "transparent" => c.transparent = true,
                    _ => return Err(meta.error(format!("serde `{key}`"))),
                }

                Ok(())
            })
            .map_err(|e| e.to_string())?;
        }

        c.tagging = match (tag, content, untagged) {
            (None, None, false) => Tagging::External,
            (Some(t), None, false) => Tagging::Internal(t),
            (Some(t), Some(c), false) => Tagging::Adjacent(t, c),
            (None, None, true) => Tagging::Untagged,
            _ => return Err("serde `content` without `tag`, or `untagged` with either".into()),
        };

        if let Some(rule) = &c.rename_all
            && !RULES.contains(&rule.as_str())
        {
            return Err(format!("serde `rename_all = {rule:?}`"));
        }

        Ok(c)
    }
}

impl Member {
    /// The attributes of a field or a variant `name`, whose serialized name
    /// is `renamed(name)` without a `rename`.
    fn of(
        attrs: &[ir::Attr],
        name: &str,
        renamed: impl Fn(&str) -> String,
    ) -> Result<Member, String> {
        let mut m = Member {
            name: renamed(name),
            doc: doc(attrs),
            ..Member::default()
        };

        for list in attrs.iter().filter_map(|a| list(a, "serde")) {
            list.parse_nested_meta(|meta| {
                let key = key(&meta.path);

                match key.as_str() {
                    "rename" => m.name = string(&meta)?,
                    "skip" | "skip_serializing" => m.skip = true,
                    "default" => {
                        if meta.input.peek(syn::Token![=]) {
                            string(&meta)?;
                        }

                        m.optional = true;
                    }
                    "skip_serializing_if" => {
                        string(&meta)?;
                        m.optional = true;
                    }
                    "flatten" => m.flatten = true,
                    _ => return Err(meta.error(format!("serde `{key}`"))),
                }

                Ok(())
            })
            .map_err(|e| e.to_string())?;
        }

        Ok(m)
    }
}

/// The `rename_all` rules of serde.
const RULES: [&str; 8] = [
    "lowercase",
    "UPPERCASE",
    "PascalCase",
    "camelCase",
    "snake_case",
    "SCREAMING_SNAKE_CASE",
    "kebab-case",
    "SCREAMING-KEBAB-CASE",
];

/// A variant name (PascalCase) under a `rename_all` rule, as serde does.
fn rename_variant(name: &str, rule: Option<&str>) -> String {
    let snake = || {
        let mut s = String::new();

        for (i, c) in name.char_indices() {
            if i > 0 && c.is_uppercase() {
                s.push('_');
            }

            s.push(c.to_ascii_lowercase());
        }

        s
    };

    match rule {
        Some("lowercase") => name.to_ascii_lowercase(),
        Some("UPPERCASE") => name.to_ascii_uppercase(),
        Some("camelCase") => name[..1].to_ascii_lowercase() + &name[1..],
        Some("snake_case") => snake(),
        Some("SCREAMING_SNAKE_CASE") => snake().to_ascii_uppercase(),
        Some("kebab-case") => snake().replace('_', "-"),
        Some("SCREAMING-KEBAB-CASE") => snake().to_ascii_uppercase().replace('_', "-"),
        _ => name.to_owned(),
    }
}

/// A field name (snake_case) under a `rename_all` rule, as serde does.
fn rename_field(name: &str, rule: Option<&str>) -> String {
    let pascal = || {
        let mut s = String::new();
        let mut upper = true;

        for c in name.chars() {
            if c == '_' {
                upper = true;
            } else if upper {
                s.push(c.to_ascii_uppercase());
                upper = false;
            } else {
                s.push(c);
            }
        }

        s
    };

    match rule {
        Some("UPPERCASE" | "SCREAMING_SNAKE_CASE") => name.to_ascii_uppercase(),
        Some("PascalCase") => pascal(),
        Some("camelCase") => {
            let p = pascal();
            p[..1].to_ascii_lowercase() + &p[1..]
        }
        Some("kebab-case") => name.replace('_', "-"),
        Some("SCREAMING-KEBAB-CASE") => name.to_ascii_uppercase().replace('_', "-"),
        _ => name.to_owned(),
    }
}

/// The attribute as the list `name(..)`.
fn list(attr: &ir::Attr, name: &str) -> Option<syn::MetaList> {
    let ir::Attr::Meta(text) = attr else {
        return None;
    };

    match syn::parse_str::<syn::Meta>(text) {
        Ok(syn::Meta::List(list)) if list.path.is_ident(name) => Some(list),
        _ => None,
    }
}

fn key(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn string(meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    Ok(meta.value()?.parse::<syn::LitStr>()?.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::test_utilities::lower;

    /// The wire of `T` declared after `Root`, and the lowering errors.
    fn wire(source: &str) -> (Option<Wire>, Vec<String>) {
        let (m, errors) = lower(&format!("pub struct Root;\n{source}"), &[], &["T"]);
        (m.wire.get("T").cloned(), errors)
    }

    fn member(name: &str) -> Member {
        Member {
            name: name.into(),
            ..Member::default()
        }
    }

    fn members<const N: usize>(members: [(&str, Member); N]) -> BTreeMap<String, Member> {
        members
            .into_iter()
            .map(|(name, m)| (name.to_owned(), m))
            .collect()
    }

    mod is_serde {
        use super::*;

        #[test]
        fn is_serde_should_be_whether_a_derive_lists_serialize_or_deserialize() {
            let serde = |attrs: &[&str]| {
                is_serde(
                    &attrs
                        .iter()
                        .map(|a| ir::Attr::Meta((*a).into()))
                        .collect::<Vec<_>>(),
                )
            };

            assert_eq!(
                [
                    serde(&["derive(Debug, serde::Serialize)"]),
                    serde(&["derive(Deserialize)"]),
                    serde(&["derive(Debug)", "serde(Serialize)", "derive"]),
                    is_serde(&[ir::Attr::Doc("derive(Serialize)".into())]),
                ],
                [true, true, false, false]
            );
        }
    }

    mod of_struct {
        use super::*;

        #[test]
        fn of_struct_should_be_the_fields_under_the_serde_attributes() {
            let (wire, errors) = wire(
                r#"/// T
///
/// doc
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T {
    /// a
    pub a_b: u8,
    #[serde(rename = "x")]
    pub c: u8,
    #[serde(skip)]
    pub d: u8,
    #[serde(skip_serializing)]
    pub e: u8,
    #[serde(default)]
    pub f: u8,
    #[serde(default = "zero")]
    pub g: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<u8>,
    #[serde(flatten)]
    pub i: u8,
}"#,
            );

            assert_eq!(errors, Vec::<String>::new());
            assert_eq!(
                wire,
                Some(Wire {
                    doc: Some("T\n\ndoc".into()),
                    shape: Shape::Named,
                    tagging: Tagging::External,
                    transparent: false,
                    members: members([
                        (
                            "a_b",
                            Member {
                                doc: Some("a".into()),
                                ..member("aB")
                            }
                        ),
                        ("c", member("x")),
                        (
                            "d",
                            Member {
                                skip: true,
                                ..member("d")
                            }
                        ),
                        (
                            "e",
                            Member {
                                skip: true,
                                ..member("e")
                            }
                        ),
                        (
                            "f",
                            Member {
                                optional: true,
                                ..member("f")
                            }
                        ),
                        (
                            "g",
                            Member {
                                optional: true,
                                ..member("g")
                            }
                        ),
                        (
                            "h",
                            Member {
                                optional: true,
                                ..member("h")
                            }
                        ),
                        (
                            "i",
                            Member {
                                flatten: true,
                                ..member("i")
                            }
                        ),
                    ]),
                })
            );
        }

        #[test]
        fn of_struct_should_be_the_shape_of_a_tuple_a_unit_or_a_transparent_struct() {
            let shape = |source: &str| {
                let (wire, errors) = wire(source);
                assert_eq!(errors, Vec::<String>::new());
                wire.map(|w| {
                    (
                        w.shape,
                        w.transparent,
                        w.members.into_keys().collect::<Vec<_>>(),
                    )
                })
            };

            assert_eq!(
                [
                    shape("#[derive(Serialize)] pub struct T(pub u8, pub u8);"),
                    shape("#[derive(Deserialize)] pub struct T;"),
                    shape("#[derive(Serialize)] #[serde(transparent)] pub struct T { pub a: u8 }"),
                ],
                [
                    Some((Shape::Tuple, false, vec!["0".into(), "1".into()])),
                    Some((Shape::Unit, false, vec![])),
                    Some((Shape::Named, true, vec!["a".into()])),
                ]
            );
        }

        #[test]
        fn of_struct_should_fail_on_a_tagging_or_an_attribute_it_does_not_know() {
            let error = |attrs: &str, field: &str| {
                let (wire, errors) = wire(&format!(
                    "#[derive(Serialize)] {attrs} pub struct T {{ {field} pub a: u8 }}"
                ));
                assert_eq!(wire, None);
                errors
            };

            assert_eq!(
                [
                    error(r#"#[serde(tag = "t")]"#, ""),
                    error("#[serde(deny_unknown_fields)]", ""),
                    error(r#"#[serde(content = "c")]"#, ""),
                    error(r#"#[serde(untagged, tag = "t")]"#, ""),
                    error(r#"#[serde(rename_all = "Title")]"#, ""),
                    error("#[serde(rename_all)]", ""),
                    error("#[serde(tag)]", ""),
                    error("#[serde(content)]", ""),
                    error("", "#[serde(rename)]"),
                    error("", r#"#[serde(with = "m")]"#),
                    error("", "#[serde(default = 1)]"),
                    error("", "#[serde(skip_serializing_if)]"),
                ],
                [
                    vec!["/c/src/lib.rs:2: `T`: serde tagging of a struct".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: serde `deny_unknown_fields`".to_owned()],
                    vec![
                        "/c/src/lib.rs:2: `T`: serde `content` without `tag`, or `untagged` with either"
                            .to_owned()
                    ],
                    vec![
                        "/c/src/lib.rs:2: `T`: serde `content` without `tag`, or `untagged` with either"
                            .to_owned()
                    ],
                    vec!["/c/src/lib.rs:2: `T`: serde `rename_all = \"Title\"`".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: expected `=`".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: expected `=`".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: expected `=`".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: expected `=`".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: serde `with`".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: expected string literal".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: expected `=`".to_owned()],
                ]
            );
        }
    }

    mod of_enum {
        use super::*;

        #[test]
        fn of_enum_should_be_the_variants_under_the_serde_attributes() {
            let (wire, errors) = wire(
                r#"#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum T {
    /// a
    AbCd,
    #[serde(rename = "x")]
    B(u8, u8),
    #[serde(skip)]
    C { #[serde(rename = "y")] d_e: u8 },
}"#,
            );

            assert_eq!(errors, Vec::<String>::new());
            assert_eq!(
                wire,
                Some(Wire {
                    doc: None,
                    shape: Shape::Named,
                    tagging: Tagging::External,
                    transparent: false,
                    members: members([
                        (
                            "AbCd",
                            Member {
                                doc: Some("a".into()),
                                ..member("ab_cd")
                            }
                        ),
                        (
                            "B",
                            Member {
                                fields: members([("0", member("0")), ("1", member("1"))]),
                                ..member("x")
                            }
                        ),
                        (
                            "C",
                            Member {
                                skip: true,
                                fields: members([("d_e", member("y"))]),
                                ..member("c")
                            }
                        ),
                    ]),
                })
            );
        }

        #[test]
        fn of_enum_should_be_the_tagging_of_the_container() {
            let tagging = |attrs: &str| {
                let (wire, errors) =
                    wire(&format!("#[derive(Serialize)] {attrs} pub enum T {{ A }}"));
                assert_eq!(errors, Vec::<String>::new());
                wire.map(|w| w.tagging)
            };

            assert_eq!(
                [
                    tagging(r#"#[serde(tag = "t")]"#),
                    tagging(r#"#[serde(tag = "t", content = "c")]"#),
                    tagging("#[serde(untagged)]"),
                ],
                [
                    Some(Tagging::Internal("t".into())),
                    Some(Tagging::Adjacent("t".into(), "c".into())),
                    Some(Tagging::Untagged),
                ]
            );
        }

        #[test]
        fn of_enum_should_fail_on_transparent_or_a_default_or_flattened_variant() {
            let error = |attrs: &str, variant: &str| {
                let (wire, errors) = wire(&format!(
                    "#[derive(Serialize)] {attrs} pub enum T {{ {variant} A(u8) }}"
                ));
                assert_eq!(wire, None);
                errors
            };

            assert_eq!(
                [
                    error("#[serde(transparent)]", ""),
                    error("#[serde(deny_unknown_fields)]", ""),
                    error("", "#[serde(default)]"),
                    error("", "#[serde(flatten)]"),
                    error("", "#[serde(other)]"),
                ],
                [
                    vec!["/c/src/lib.rs:2: `T`: serde `transparent` on an enum".to_owned()],
                    vec!["/c/src/lib.rs:2: `T`: serde `deny_unknown_fields`".to_owned()],
                    vec![
                        "/c/src/lib.rs:2: `T`: serde `default` or `flatten` on variant `A`"
                            .to_owned()
                    ],
                    vec![
                        "/c/src/lib.rs:2: `T`: serde `default` or `flatten` on variant `A`"
                            .to_owned()
                    ],
                    vec!["/c/src/lib.rs:2: `T`: serde `other`".to_owned()],
                ]
            );
        }

        #[test]
        fn of_enum_should_fail_on_an_attribute_of_a_field_it_does_not_know() {
            let (wire, errors) =
                wire("#[derive(Serialize)] pub enum T { A { #[serde(borrow)] a: u8 } }");

            assert_eq!(
                (wire, errors),
                (
                    None,
                    vec!["/c/src/lib.rs:2: `T`: serde `borrow`".to_owned()]
                )
            );
        }
    }

    mod rename_variant {
        use super::*;

        #[test]
        fn rename_variant_should_follow_the_rules_of_serde() {
            let renamed: Vec<String> = [None]
                .into_iter()
                .chain(RULES.map(Some))
                .map(|rule| rename_variant("AbCd", rule))
                .collect();

            assert_eq!(
                renamed,
                [
                    "AbCd", "abcd", "ABCD", "AbCd", "abCd", "ab_cd", "AB_CD", "ab-cd", "AB-CD"
                ]
            );
        }
    }

    mod rename_field {
        use super::*;

        #[test]
        fn rename_field_should_follow_the_rules_of_serde() {
            let renamed: Vec<String> = [None]
                .into_iter()
                .chain(RULES.map(Some))
                .map(|rule| rename_field("ab_cd", rule))
                .collect();

            assert_eq!(
                renamed,
                [
                    "ab_cd", "ab_cd", "AB_CD", "AbCd", "abCd", "ab_cd", "AB_CD", "ab-cd", "AB-CD"
                ]
            );
        }
    }
}
