use crate::Error;
use crate::ItemId;
use crate::ModuleId;
use crate::SourceCrate;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use syn::Item;
use syn::UseTree;
use syn::ext::IdentExt;

/// The namespace of a name: a type, a function and a macro can share one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Namespace {
    /// Modules, types, traits and enum variants.
    Type,
    /// Functions, constants, statics and the constructors of unit and tuple
    /// structs.
    Value,
    Macro,
}

/// What a path names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    Module(ModuleId),
    Item(ItemId),
    /// The variant at the index of a local enum.
    Variant(ItemId, usize),
    /// `base::name` of a local type or trait or of a primitive: an
    /// associated item, which is found in the impls of the base.
    Associated(Box<Resolved>, String),
    /// A primitive type such as `u64` or `str`.
    Primitive(String),
    /// A path into another crate. A name of the std prelude is spelled out,
    /// `Vec` is `std::vec::Vec`.
    External(Vec<String>),
    Unresolved,
}

/// The names in scope of every module: its items, its imports and its glob
/// imports, plus the crates of the extern prelude.
#[derive(Default)]
pub(crate) struct Scopes {
    scopes: Vec<Scope>,
    externs: HashMap<String, Resolved>,
}

#[derive(Default)]
struct Scope {
    names: [HashMap<String, Binding>; 3],
    globs: Vec<Glob>,
    /// The `macro_rules!` of the module, which are in textual scope rather
    /// than names of the module.
    macro_rules: HashMap<String, ItemId>,
}

#[derive(Clone)]
struct Binding {
    target: Resolved,
    visibility: Visibility,
    /// A name of another crate imported in every namespace, as its own is
    /// unknown. A binding of the name in the namespace replaces it.
    guessed: bool,
}

struct Glob {
    source: Resolved,
    visibility: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Public,
    /// Visible within the module and its descendants.
    Restricted(ModuleId),
}

struct Import {
    module: ModuleId,
    line: usize,
    leading_colon: bool,
    path: Vec<String>,
    kind: ImportKind,
    visibility: Visibility,
}

enum ImportKind {
    /// The last segment of the path, in every namespace it exists in.
    Name(String),
    /// The path itself, `self` in a group, in the type namespace.
    Module(String),
    Glob,
}

const NAMESPACES: [Namespace; 3] = [Namespace::Type, Namespace::Value, Namespace::Macro];

const PRIMITIVES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64",
];

const PRELUDE: &[(Namespace, &str, &str)] = &[
    (Namespace::Type, "std::marker", "Copy"),
    (Namespace::Type, "std::marker", "Send"),
    (Namespace::Type, "std::marker", "Sized"),
    (Namespace::Type, "std::marker", "Sync"),
    (Namespace::Type, "std::marker", "Unpin"),
    (Namespace::Type, "std::ops", "Drop"),
    (Namespace::Type, "std::ops", "Fn"),
    (Namespace::Type, "std::ops", "FnMut"),
    (Namespace::Type, "std::ops", "FnOnce"),
    (Namespace::Type, "std::ops", "AsyncFn"),
    (Namespace::Type, "std::ops", "AsyncFnMut"),
    (Namespace::Type, "std::ops", "AsyncFnOnce"),
    (Namespace::Type, "std::boxed", "Box"),
    (Namespace::Type, "std::borrow", "ToOwned"),
    (Namespace::Type, "std::clone", "Clone"),
    (Namespace::Type, "std::cmp", "PartialEq"),
    (Namespace::Type, "std::cmp", "PartialOrd"),
    (Namespace::Type, "std::cmp", "Eq"),
    (Namespace::Type, "std::cmp", "Ord"),
    (Namespace::Type, "std::convert", "AsRef"),
    (Namespace::Type, "std::convert", "AsMut"),
    (Namespace::Type, "std::convert", "Into"),
    (Namespace::Type, "std::convert", "From"),
    (Namespace::Type, "std::convert", "TryFrom"),
    (Namespace::Type, "std::convert", "TryInto"),
    (Namespace::Type, "std::default", "Default"),
    (Namespace::Type, "std::future", "Future"),
    (Namespace::Type, "std::future", "IntoFuture"),
    (Namespace::Type, "std::iter", "Iterator"),
    (Namespace::Type, "std::iter", "Extend"),
    (Namespace::Type, "std::iter", "IntoIterator"),
    (Namespace::Type, "std::iter", "DoubleEndedIterator"),
    (Namespace::Type, "std::iter", "ExactSizeIterator"),
    (Namespace::Type, "std::iter", "FromIterator"),
    (Namespace::Type, "std::option", "Option"),
    (Namespace::Type, "std::result", "Result"),
    (Namespace::Type, "std::string", "String"),
    (Namespace::Type, "std::string", "ToString"),
    (Namespace::Type, "std::vec", "Vec"),
    (Namespace::Value, "std::mem", "drop"),
    (Namespace::Value, "std::mem", "size_of"),
    (Namespace::Value, "std::mem", "size_of_val"),
    (Namespace::Value, "std::mem", "align_of"),
    (Namespace::Value, "std::mem", "align_of_val"),
    (Namespace::Value, "std::option::Option", "Some"),
    (Namespace::Value, "std::option::Option", "None"),
    (Namespace::Value, "std::result::Result", "Ok"),
    (Namespace::Value, "std::result::Result", "Err"),
    (Namespace::Macro, "std", "assert"),
    (Namespace::Macro, "std", "assert_eq"),
    (Namespace::Macro, "std", "assert_ne"),
    (Namespace::Macro, "std", "cfg"),
    (Namespace::Macro, "std", "column"),
    (Namespace::Macro, "std", "compile_error"),
    (Namespace::Macro, "std", "concat"),
    (Namespace::Macro, "std", "dbg"),
    (Namespace::Macro, "std", "debug_assert"),
    (Namespace::Macro, "std", "debug_assert_eq"),
    (Namespace::Macro, "std", "debug_assert_ne"),
    (Namespace::Macro, "std", "env"),
    (Namespace::Macro, "std", "eprint"),
    (Namespace::Macro, "std", "eprintln"),
    (Namespace::Macro, "std", "file"),
    (Namespace::Macro, "std", "format"),
    (Namespace::Macro, "std", "format_args"),
    (Namespace::Macro, "std", "include"),
    (Namespace::Macro, "std", "include_bytes"),
    (Namespace::Macro, "std", "include_str"),
    (Namespace::Macro, "std", "line"),
    (Namespace::Macro, "std", "matches"),
    (Namespace::Macro, "std", "module_path"),
    (Namespace::Macro, "std", "option_env"),
    (Namespace::Macro, "std", "panic"),
    (Namespace::Macro, "std", "print"),
    (Namespace::Macro, "std", "println"),
    (Namespace::Macro, "std", "stringify"),
    (Namespace::Macro, "std", "thread_local"),
    (Namespace::Macro, "std", "todo"),
    (Namespace::Macro, "std", "unimplemented"),
    (Namespace::Macro, "std", "unreachable"),
    (Namespace::Macro, "std", "vec"),
    (Namespace::Macro, "std", "write"),
    (Namespace::Macro, "std", "writeln"),
    (Namespace::Macro, "std::clone", "Clone"),
    (Namespace::Macro, "std::marker", "Copy"),
    (Namespace::Macro, "std::fmt", "Debug"),
    (Namespace::Macro, "std::default", "Default"),
    (Namespace::Macro, "std::cmp", "Eq"),
    (Namespace::Macro, "std::cmp", "Ord"),
    (Namespace::Macro, "std::cmp", "PartialEq"),
    (Namespace::Macro, "std::cmp", "PartialOrd"),
    (Namespace::Macro, "std::hash", "Hash"),
];

impl SourceCrate {
    /// Resolves the path as written in the module `from`, its last segment
    /// in `namespace`. Generic arguments are ignored, and so are the
    /// names only the caller knows: `Self`, generic parameters and the items
    /// and imports of blocks.
    pub fn resolve(&self, from: ModuleId, path: &syn::Path, namespace: Namespace) -> Resolved {
        let segments: Vec<String> = path.segments.iter().map(|s| unraw(&s.ident)).collect();
        self.scopes.resolve(
            self,
            from,
            &segments,
            path.leading_colon.is_some(),
            namespace,
        )
    }
}

impl Scopes {
    /// Collects the names of every module, then resolves the imports until
    /// none resolves anymore. The ones left are reported.
    pub(crate) fn build(c: &SourceCrate, externs: Vec<String>) -> (Self, Vec<Error>) {
        let mut scopes = Self {
            scopes: c.modules().map(|_| Scope::default()).collect(),
            externs: ["std", "core", "alloc"]
                .map(String::from)
                .into_iter()
                .chain(externs)
                .map(|name| {
                    let target = external(std::slice::from_ref(&name));
                    (name, target)
                })
                .collect(),
        };
        let children: HashMap<ItemId, ModuleId> = c
            .modules()
            .filter_map(|(id, module)| module.item().map(|item| (item, id)))
            .collect();
        let mut imports = Vec::new();

        for (module, _) in c.modules() {
            for (index, item) in c.items(module).iter().enumerate() {
                let id = ItemId { module, index };
                scopes.define(c, &children, id, item, &mut imports);
            }
        }

        let errors = scopes.import(c, imports);
        (scopes, errors)
    }

    fn define(
        &mut self,
        c: &SourceCrate,
        children: &HashMap<ItemId, ModuleId>,
        id: ItemId,
        item: &Item,
        imports: &mut Vec<Import>,
    ) {
        let module = id.module;
        let target = Resolved::Item(id);
        let mut bind = |namespace, ident: &syn::Ident, target, vis: &syn::Visibility| {
            let visibility = visibility(c, module, vis);
            self.bind(module, namespace, unraw(ident), target, visibility);
        };

        match item {
            Item::Mod(m) => {
                if let Some(child) = children.get(&id) {
                    bind(Namespace::Type, &m.ident, Resolved::Module(*child), &m.vis);
                }
            }
            Item::Struct(s) => {
                if !matches!(s.fields, syn::Fields::Named(_)) {
                    bind(Namespace::Value, &s.ident, target.clone(), &s.vis);
                }
                bind(Namespace::Type, &s.ident, target, &s.vis);
            }
            Item::Enum(e) => bind(Namespace::Type, &e.ident, target, &e.vis),
            Item::Union(u) => bind(Namespace::Type, &u.ident, target, &u.vis),
            Item::Trait(t) => bind(Namespace::Type, &t.ident, target, &t.vis),
            Item::TraitAlias(t) => bind(Namespace::Type, &t.ident, target, &t.vis),
            Item::Type(t) => bind(Namespace::Type, &t.ident, target, &t.vis),
            Item::Fn(f) => bind(Namespace::Value, &f.sig.ident, target, &f.vis),
            Item::Const(k) if k.ident != "_" => bind(Namespace::Value, &k.ident, target, &k.vis),
            Item::Static(s) => bind(Namespace::Value, &s.ident, target, &s.vis),
            Item::Macro(m) => {
                if let Some(ident) = &m.ident {
                    if m.attrs.iter().any(|a| a.path().is_ident("macro_export")) {
                        let root = c.root();
                        self.bind(
                            root,
                            Namespace::Macro,
                            unraw(ident),
                            target,
                            Visibility::Public,
                        );
                    }

                    self.scopes[module.0]
                        .macro_rules
                        .entry(unraw(ident))
                        .or_insert(id);
                }
            }
            Item::ExternCrate(e) => {
                let target = if e.ident == "self" {
                    Resolved::Module(c.root())
                } else {
                    external(&[unraw(&e.ident)])
                };
                let name = e.rename.as_ref().map_or(&e.ident, |(_, rename)| rename);
                bind(Namespace::Type, name, target.clone(), &e.vis);

                if module == c.root() {
                    self.externs.insert(unraw(name), target);
                }
            }
            Item::Use(u) => {
                let visibility = visibility(c, module, &u.vis);
                let leading_colon = u.leading_colon.is_some();
                // A path without segments, as in `use {self};`, is invalid.
                flatten(&u.tree, &mut Vec::new(), &mut |path, kind, line| {
                    if path.is_empty() {
                        return;
                    }

                    imports.push(Import {
                        module,
                        line,
                        leading_colon,
                        path,
                        kind,
                        visibility,
                    });
                });
            }
            _ => {}
        }
    }

    fn bind(
        &mut self,
        module: ModuleId,
        namespace: Namespace,
        name: String,
        target: Resolved,
        visibility: Visibility,
    ) {
        self.bind_as(module, namespace, name, target, visibility, false);
    }

    fn bind_as(
        &mut self,
        module: ModuleId,
        namespace: Namespace,
        name: String,
        target: Resolved,
        visibility: Visibility,
        guessed: bool,
    ) {
        let names = &mut self.scopes[module.0].names[namespace as usize];

        if names.get(&name).is_none_or(|b| b.guessed && !guessed) {
            names.insert(
                name,
                Binding {
                    target,
                    visibility,
                    guessed,
                },
            );
        }
    }

    /// Resolves the imports in rounds until a round resolves none. A name
    /// not in scope of a module is then taken from its glob imports of
    /// other crates, for one round, as no other import can bring it in.
    fn import(&mut self, c: &SourceCrate, mut pending: Vec<Import>) -> Vec<Error> {
        let mut globs = false;

        loop {
            let count = pending.len();
            pending.retain(|import| !self.apply(c, import, globs));

            if pending.len() < count {
                globs = false;
            } else if globs || pending.is_empty() {
                break;
            } else {
                globs = true;
            }
        }

        pending
            .into_iter()
            .map(|import| {
                let mut name = import.path.join("::");

                if import.leading_colon {
                    name.insert_str(0, "::");
                }

                if let ImportKind::Glob = import.kind {
                    name.push_str("::*");
                }

                Error::UnresolvedImport {
                    path: c.file(c.module(import.module).file()).path().to_path_buf(),
                    line: import.line,
                    name,
                }
            })
            .collect()
    }

    fn apply(&mut self, c: &SourceCrate, import: &Import, globs: bool) -> bool {
        if let ImportKind::Name(name) = &import.kind
            && import.path.len() == 1
            && !import.leading_colon
        {
            return self.apply_single(c, import, name);
        }

        let Some(start) = self.import_start(c, import) else {
            return false;
        };
        let module = import.module;
        let rest = &import.path[1..];

        match &import.kind {
            ImportKind::Glob => {
                let Some(source) = self.walk(c, start, rest, globs) else {
                    return false;
                };
                self.scopes[module.0].globs.push(Glob {
                    source,
                    visibility: import.visibility,
                });
            }
            ImportKind::Module(name) => {
                let Some(target) = self.walk(c, start, rest, globs) else {
                    return false;
                };
                self.bind(
                    module,
                    Namespace::Type,
                    name.clone(),
                    target,
                    import.visibility,
                );
            }
            ImportKind::Name(name) => {
                let Some((last, inner)) = rest.split_last() else {
                    self.bind(
                        module,
                        Namespace::Type,
                        name.clone(),
                        start,
                        import.visibility,
                    );
                    return true;
                };
                let Some(container) = self.walk(c, start, inner, globs) else {
                    return false;
                };
                let found: Vec<_> = NAMESPACES
                    .iter()
                    .filter_map(|&ns| {
                        self.member(c, &container, last, ns, globs)
                            .map(|target| (ns, target))
                    })
                    .collect();

                if found.is_empty() {
                    return false;
                }

                let guessed = matches!(container, Resolved::External(_));

                for (ns, target) in found {
                    self.bind_as(module, ns, name.clone(), target, import.visibility, guessed);
                }
            }
        }

        true
    }

    /// `use name;`: a crate, or a name in scope of the module such as a
    /// `macro_rules!` it re-exports, in every namespace it exists in.
    fn apply_single(&mut self, c: &SourceCrate, import: &Import, name: &str) -> bool {
        let module = import.module;
        let first = &import.path[0];
        let found: Vec<_> = NAMESPACES
            .iter()
            .filter_map(|&ns| {
                self.lookup(c, module, first, ns)
                    .map(|binding| binding.target)
                    .or_else(|| match ns {
                        Namespace::Type => self.externs.get(first).cloned(),
                        Namespace::Value => None,
                        Namespace::Macro => self.textual_macro(c, module, first),
                    })
                    .map(|target| (ns, target))
            })
            .collect();

        if found.is_empty() {
            return false;
        }

        for (ns, target) in found {
            self.bind(module, ns, name.to_string(), target, import.visibility);
        }

        true
    }

    /// The first segment of an import: a module keyword, a name in scope of
    /// the importing module or a crate of the extern prelude.
    fn import_start(&self, c: &SourceCrate, import: &Import) -> Option<Resolved> {
        let module = import.module;
        let first = &import.path[0];

        if import.leading_colon {
            return self.externs.get(first).cloned();
        }

        match first.as_str() {
            "crate" => Some(Resolved::Module(c.root())),
            "self" => Some(Resolved::Module(module)),
            "super" => c.module(module).parent().map(Resolved::Module),
            _ => self
                .lookup(c, module, first, Namespace::Type)
                .map(|binding| binding.target)
                .or_else(|| self.externs.get(first).cloned()),
        }
    }

    fn resolve(
        &self,
        c: &SourceCrate,
        from: ModuleId,
        segments: &[String],
        leading_colon: bool,
        namespace: Namespace,
    ) -> Resolved {
        let Some((first, rest)) = segments.split_first() else {
            return Resolved::Unresolved;
        };
        let start = if leading_colon {
            self.externs.get(first).cloned()
        } else {
            match first.as_str() {
                "crate" => Some(Resolved::Module(c.root())),
                "self" => Some(Resolved::Module(from)),
                "super" => c.module(from).parent().map(Resolved::Module),
                _ => {
                    let ns = if rest.is_empty() {
                        namespace
                    } else {
                        Namespace::Type
                    };
                    self.resolve_first(c, from, first, ns)
                }
            }
        };
        let mut target = start.unwrap_or(Resolved::Unresolved);

        for (i, name) in rest.iter().enumerate() {
            let ns = if i + 1 == rest.len() {
                namespace
            } else {
                Namespace::Type
            };
            target = self
                .member(c, &target, name, ns, true)
                .unwrap_or(Resolved::Unresolved);
        }

        target
    }

    /// The first segment of a path: a name in scope of the module, a crate
    /// of the extern prelude, a name of the std prelude, a primitive or a
    /// name of a glob import from another crate.
    fn resolve_first(
        &self,
        c: &SourceCrate,
        from: ModuleId,
        name: &str,
        ns: Namespace,
    ) -> Option<Resolved> {
        self.lookup(c, from, name, ns)
            .map(|binding| binding.target)
            .or_else(|| {
                (ns == Namespace::Type)
                    .then(|| self.externs.get(name).cloned())
                    .flatten()
            })
            .or_else(|| {
                (ns == Namespace::Macro)
                    .then(|| self.textual_macro(c, from, name))
                    .flatten()
            })
            .or_else(|| prelude(name, ns))
            .or_else(|| self.external_glob(from, name))
    }

    /// A `macro_rules!` in textual scope: of an enclosing module, or of a
    /// `#[macro_use]` module declared in one.
    fn textual_macro(&self, c: &SourceCrate, from: ModuleId, name: &str) -> Option<Resolved> {
        let mut module = Some(from);

        while let Some(m) = module {
            if let Some(target) = self.macro_rules(c, m, name) {
                return Some(target);
            }

            module = c.module(m).parent();
        }

        None
    }

    fn macro_rules(&self, c: &SourceCrate, module: ModuleId, name: &str) -> Option<Resolved> {
        if let Some(id) = self.scopes[module.0].macro_rules.get(name) {
            return Some(Resolved::Item(*id));
        }

        c.module(module)
            .children()
            .iter()
            .filter(|child| {
                c.module(**child)
                    .item()
                    .is_some_and(|item| macro_use(c.item(item)))
            })
            .find_map(|&child| self.macro_rules(c, child, name))
    }

    fn walk(
        &self,
        c: &SourceCrate,
        start: Resolved,
        segments: &[String],
        globs: bool,
    ) -> Option<Resolved> {
        segments.iter().try_fold(start, |target, name| {
            self.member(c, &target, name, Namespace::Type, globs)
        })
    }

    /// `target::name`. Within a module, a name not in scope is taken from a
    /// glob import of another crate when `globs` is set.
    fn member(
        &self,
        c: &SourceCrate,
        target: &Resolved,
        name: &str,
        ns: Namespace,
        globs: bool,
    ) -> Option<Resolved> {
        match target {
            Resolved::Module(module) if name == "super" => {
                c.module(*module).parent().map(Resolved::Module)
            }
            Resolved::Module(module) => self
                .lookup(c, *module, name, ns)
                .map(|binding| binding.target)
                .or_else(|| globs.then(|| self.external_glob(*module, name)).flatten()),
            Resolved::Item(id) => match c.item(*id) {
                Item::Enum(e) => {
                    Some(variant(e, *id, name).unwrap_or_else(|| associated(target, name)))
                }
                // Only types and traits are items in the type namespace.
                _ => Some(associated(target, name)),
            },
            Resolved::Primitive(_) | Resolved::Associated(..) => Some(associated(target, name)),
            Resolved::External(path) => {
                let mut path = path.clone();
                path.push(name.to_string());
                Some(Resolved::External(path))
            }
            Resolved::Variant(..) | Resolved::Unresolved => None,
        }
    }

    /// The name in scope of the module: its items and imports first, then
    /// its globs, which bring in the names visible to the module.
    fn lookup(
        &self,
        c: &SourceCrate,
        module: ModuleId,
        name: &str,
        ns: Namespace,
    ) -> Option<Binding> {
        self.lookup_in(c, module, name, ns, &mut HashSet::new())
    }

    fn lookup_in(
        &self,
        c: &SourceCrate,
        module: ModuleId,
        name: &str,
        ns: Namespace,
        visited: &mut HashSet<ModuleId>,
    ) -> Option<Binding> {
        let scope = &self.scopes[module.0];

        if let Some(binding) = scope.names[ns as usize].get(name) {
            return Some(binding.clone());
        }

        visited.insert(module);

        scope.globs.iter().find_map(|glob| {
            let binding = match &glob.source {
                Resolved::Module(source) if !visited.contains(source) => self
                    .lookup_in(c, *source, name, ns, visited)
                    .filter(|binding| visible(c, binding.visibility, module))?,
                Resolved::Item(id) => match c.item(*id) {
                    Item::Enum(e) => Binding {
                        target: variant(e, *id, name)?,
                        visibility: Visibility::Public,
                        guessed: false,
                    },
                    _ => return None,
                },
                _ => return None,
            };
            Some(Binding {
                target: binding.target,
                visibility: narrower(c, binding.visibility, glob.visibility),
                guessed: binding.guessed,
            })
        })
    }

    /// The `pub` names of the module: its items and imports, then the names
    /// of its `pub` globs of local modules and enums. The names of globs of
    /// other crates are unknown.
    pub(crate) fn public_names(
        &self,
        c: &SourceCrate,
        module: ModuleId,
    ) -> BTreeMap<(Namespace, String), Resolved> {
        let mut names = BTreeMap::new();
        self.public_names_in(c, module, &mut names, &mut HashSet::new());
        names
    }

    fn public_names_in(
        &self,
        c: &SourceCrate,
        module: ModuleId,
        names: &mut BTreeMap<(Namespace, String), Resolved>,
        visited: &mut HashSet<ModuleId>,
    ) {
        let scope = &self.scopes[module.0];
        visited.insert(module);

        for ns in NAMESPACES {
            for (name, binding) in &scope.names[ns as usize] {
                if binding.visibility == Visibility::Public {
                    names
                        .entry((ns, name.clone()))
                        .or_insert_with(|| binding.target.clone());
                }
            }
        }

        for glob in &scope.globs {
            if glob.visibility != Visibility::Public {
                continue;
            }

            match &glob.source {
                Resolved::Module(source) if !visited.contains(source) => {
                    self.public_names_in(c, *source, names, visited);
                }
                Resolved::Item(id) => {
                    if let Item::Enum(e) = c.item(*id) {
                        for (index, v) in e.variants.iter().enumerate() {
                            names
                                .entry((Namespace::Type, unraw(&v.ident)))
                                .or_insert(Resolved::Variant(*id, index));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn external_glob(&self, module: ModuleId, name: &str) -> Option<Resolved> {
        self.scopes[module.0]
            .globs
            .iter()
            .find_map(|glob| match &glob.source {
                Resolved::External(path) => {
                    let mut path = path.clone();
                    path.push(name.to_string());
                    Some(Resolved::External(path))
                }
                _ => None,
            })
    }
}

/// Calls `import` with the path, kind and line of each import of the tree.
fn flatten(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    import: &mut impl FnMut(Vec<String>, ImportKind, usize),
) {
    let mut single = |ident: &syn::Ident, name: &syn::Ident, prefix: &[String]| {
        let line = ident.span().start().line;

        if ident == "self" {
            let name = if name == "self" {
                prefix.last().cloned().unwrap_or(unraw(name))
            } else {
                unraw(name)
            };
            import(prefix.to_vec(), ImportKind::Module(name), line);
        } else {
            let mut path = prefix.to_vec();
            path.push(unraw(ident));
            import(path, ImportKind::Name(unraw(name)), line);
        }
    };

    match tree {
        UseTree::Path(p) => {
            prefix.push(unraw(&p.ident));
            flatten(&p.tree, prefix, import);
            prefix.pop();
        }
        UseTree::Name(n) => single(&n.ident, &n.ident, prefix),
        UseTree::Rename(r) => {
            if r.rename != "_" {
                single(&r.ident, &r.rename, prefix);
            }
        }
        UseTree::Glob(g) => import(
            prefix.clone(),
            ImportKind::Glob,
            g.star_token.span.start().line,
        ),
        UseTree::Group(g) => {
            for tree in &g.items {
                flatten(tree, prefix, import);
            }
        }
    }
}

fn visibility(c: &SourceCrate, module: ModuleId, vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Inherited => Visibility::Restricted(module),
        syn::Visibility::Restricted(r) => {
            let restricted = r.path.segments.iter().fold(module, |m, segment| {
                match unraw(&segment.ident).as_str() {
                    "crate" => c.root(),
                    "self" => module,
                    "super" => c.module(m).parent().unwrap_or(m),
                    name => c
                        .module(m)
                        .children()
                        .iter()
                        .copied()
                        .find(|&child| c.module(child).name() == name)
                        .unwrap_or(m),
                }
            });
            Visibility::Restricted(restricted)
        }
    }
}

fn visible(c: &SourceCrate, visibility: Visibility, from: ModuleId) -> bool {
    let Visibility::Restricted(within) = visibility else {
        return true;
    };
    let mut module = Some(from);

    while let Some(m) = module {
        if m == within {
            return true;
        }

        module = c.module(m).parent();
    }

    false
}

fn narrower(c: &SourceCrate, a: Visibility, b: Visibility) -> Visibility {
    match (a, b) {
        (Visibility::Restricted(x), Visibility::Restricted(y)) if !visible(c, a, y) => {
            Visibility::Restricted(x)
        }
        (_, Visibility::Restricted(_)) => b,
        _ => a,
    }
}

fn macro_use(item: &Item) -> bool {
    matches!(item, Item::Mod(m) if m.attrs.iter().any(|a| a.path().is_ident("macro_use")))
}

fn variant(e: &syn::ItemEnum, id: ItemId, name: &str) -> Option<Resolved> {
    e.variants
        .iter()
        .position(|v| v.ident == name)
        .map(|index| Resolved::Variant(id, index))
}

fn associated(base: &Resolved, name: &str) -> Resolved {
    Resolved::Associated(Box::new(base.clone()), name.to_string())
}

fn prelude(name: &str, ns: Namespace) -> Option<Resolved> {
    if ns == Namespace::Type && PRIMITIVES.contains(&name) {
        return Some(Resolved::Primitive(name.to_string()));
    }

    PRELUDE
        .iter()
        .find(|(n, _, item)| *n == ns && *item == name)
        .map(|(_, module, item)| {
            Resolved::External(
                module
                    .split("::")
                    .chain([*item])
                    .map(String::from)
                    .collect(),
            )
        })
}

fn external(path: &[String]) -> Resolved {
    Resolved::External(path.to_vec())
}

fn unraw(ident: &syn::Ident) -> String {
    ident.unraw().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cfg;
    use crate::Target;
    use crate::TargetKind;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::test_platform;
    use std::path::Path;

    fn load(files: &[(&str, &str)], externs: &[&str]) -> SourceCrate {
        let platform = test_platform();

        for (path, content) in files {
            platform
                .fs()
                .create_dir_all(Path::new(path).parent().unwrap())
                .unwrap();
            platform.fs().write(path, content).unwrap();
        }

        let target = Target {
            externs: externs.iter().map(ToString::to_string).collect(),
            ..Target::new(TargetKind::Lib, "/c/src/lib.rs")
        };
        SourceCrate::load(&platform, &target, &Cfg::All)
    }

    fn lib(source: &str) -> SourceCrate {
        load(&[("/c/src/lib.rs", source)], &[])
    }

    fn lib_with_crates(source: &str) -> SourceCrate {
        load(
            &[("/c/src/lib.rs", source)],
            &["serde", "serde_json", "dep"],
        )
    }

    fn module(c: &SourceCrate, path: &str) -> ModuleId {
        c.modules()
            .find(|(_, m)| m.path().join("::") == path)
            .unwrap()
            .0
    }

    fn item(c: &SourceCrate, module_path: &str, index: usize) -> Resolved {
        Resolved::Item(ItemId {
            module: module(c, module_path),
            index,
        })
    }

    fn resolve(c: &SourceCrate, from: &str, path: &str, ns: Namespace) -> Resolved {
        c.resolve(module(c, from), &syn::parse_str(path).unwrap(), ns)
    }

    fn ty(c: &SourceCrate, from: &str, path: &str) -> Resolved {
        resolve(c, from, path, Namespace::Type)
    }

    fn ext(path: &str) -> Resolved {
        Resolved::External(path.split("::").map(String::from).collect())
    }

    // WHY: paths display with `\` on Windows.
    fn errors(c: &SourceCrate) -> Vec<String> {
        c.errors()
            .iter()
            .map(|e| e.to_string().replace('\\', "/"))
            .collect()
    }

    mod source_crate {
        use super::*;

        mod resolve {
            use super::*;

            #[test]
            fn resolve_should_find_items_in_their_namespaces() {
                let c = lib("struct U;
struct T(u8);
struct N {}
enum E {}
union Un { a: u8 }
trait Tr {}
trait Ta = Tr;
type Ty = u8;
fn f() {}
const C: u8 = 0;
const _: () = ();
static S: u8 = 0;
mod m {}
x!();");

                let types: Vec<_> = ["U", "T", "N", "E", "Un", "Tr", "Ta", "Ty", "m"]
                    .map(|name| ty(&c, "", name))
                    .into();
                let values: Vec<_> = ["U", "T", "N", "f", "C", "S", "m"]
                    .map(|name| resolve(&c, "", name, Namespace::Value))
                    .into();

                assert_eq!(
                    types,
                    vec![
                        item(&c, "", 0),
                        item(&c, "", 1),
                        item(&c, "", 2),
                        item(&c, "", 3),
                        item(&c, "", 4),
                        item(&c, "", 5),
                        item(&c, "", 6),
                        item(&c, "", 7),
                        Resolved::Module(module(&c, "m")),
                    ]
                );
                assert_eq!(
                    values,
                    vec![
                        item(&c, "", 0),
                        item(&c, "", 1),
                        Resolved::Unresolved,
                        item(&c, "", 8),
                        item(&c, "", 9),
                        item(&c, "", 11),
                        Resolved::Unresolved,
                    ]
                );
            }

            #[test]
            fn resolve_should_start_at_crate_self_super_and_crates() {
                let c = lib("extern crate self as me;
struct X;
mod a { struct Y; mod b {} }");

                assert_eq!(ty(&c, "a::b", "crate::X"), item(&c, "", 1));
                assert_eq!(ty(&c, "a", "self::Y"), item(&c, "a", 0));
                assert_eq!(ty(&c, "a", "super::X"), item(&c, "", 1));
                assert_eq!(ty(&c, "a::b", "super::super::X"), item(&c, "", 1));
                assert_eq!(ty(&c, "", "super::X"), Resolved::Unresolved);
                assert_eq!(ty(&c, "a::b", "::std::vec::Vec"), ext("std::vec::Vec"));
                assert_eq!(ty(&c, "a::b", "::me::X"), item(&c, "", 1));
                assert_eq!(ty(&c, "a::b", "::anyhow::Error"), Resolved::Unresolved);
                assert_eq!(ty(&c, "a::b", "me::a::Y"), item(&c, "a", 0));
            }

            #[test]
            fn resolve_should_follow_imports() {
                let c = lib("pub use a::reexport::Z;
use a::{X, Y as W, b::{self, Inner}, b::{self as c}};
use a::X as _;
use ::std::fmt;
use self::a::X as SelfX;
mod a {
    pub struct X;
    pub struct Y;
    pub mod b { pub struct Inner; }
    pub mod reexport { pub use super::b::Inner as Z; }
}");

                assert_eq!(ty(&c, "", "X"), item(&c, "a", 0));
                assert_eq!(ty(&c, "", "W"), item(&c, "a", 1));
                assert_eq!(ty(&c, "", "Y"), Resolved::Unresolved);
                assert_eq!(ty(&c, "", "b"), Resolved::Module(module(&c, "a::b")));
                assert_eq!(ty(&c, "", "c::Inner"), item(&c, "a::b", 0));
                assert_eq!(ty(&c, "", "Inner"), item(&c, "a::b", 0));
                assert_eq!(ty(&c, "", "Z"), item(&c, "a::b", 0));
                assert_eq!(ty(&c, "", "fmt::Debug"), ext("std::fmt::Debug"));
                assert_eq!(ty(&c, "", "SelfX"), item(&c, "a", 0));
                assert!(c.errors().is_empty());
            }

            #[test]
            fn resolve_glob_should_import_the_names_visible_to_the_module() {
                let c = lib("mod a {
    pub struct P;
    struct Q;
    pub(crate) struct R;
    use crate::e::Z;
    mod inner { use super::*; }
}
mod b { use crate::a::*; struct P; }
mod e { pub struct Z; }
mod x { pub use crate::y::*; }
mod y { pub use crate::x::*; }
mod d { use crate::c::*; }
mod c { use crate::e::*; }");

                assert_eq!(ty(&c, "a::inner", "Q"), item(&c, "a", 1));
                assert_eq!(ty(&c, "a::inner", "Z"), item(&c, "e", 0));
                assert_eq!(ty(&c, "b", "R"), item(&c, "a", 2));
                assert_eq!(ty(&c, "b", "Q"), Resolved::Unresolved);
                assert_eq!(ty(&c, "b", "P"), item(&c, "b", 1));
                assert_eq!(ty(&c, "b", "Z"), Resolved::Unresolved);
                assert_eq!(ty(&c, "x", "Gone"), Resolved::Unresolved);
                assert_eq!(ty(&c, "c", "Z"), item(&c, "e", 0));
                assert_eq!(ty(&c, "d", "Z"), Resolved::Unresolved);
            }

            #[test]
            fn resolve_glob_of_an_enum_should_import_its_variants() {
                let c = lib("enum E { A, B }
struct S;
mod m { use crate::E::*; use crate::S::*; use crate::E::B as Second; }");

                assert_eq!(
                    resolve(&c, "m", "B", Namespace::Value),
                    Resolved::Variant(
                        ItemId {
                            module: c.root(),
                            index: 0
                        },
                        1
                    )
                );
                assert_eq!(
                    ty(&c, "m", "Second"),
                    Resolved::Variant(
                        ItemId {
                            module: c.root(),
                            index: 0
                        },
                        1
                    )
                );
                assert_eq!(ty(&c, "m", "C"), Resolved::Unresolved);
            }

            #[test]
            fn resolve_visibility_should_restrict_glob_imports() {
                let c = lib("pub(super) struct Root;
pub(in crate::missing) struct Missing;
mod a {
    mod b {
        pub(in crate::a) struct X;
        pub(super) struct Y;
        pub(self) struct Z;
        pub(crate) struct W;
    }
    mod d { pub(crate) use super::b::*; }
}
mod c { use crate::a::b::*; mod inner { use super::super::*; } }
mod f { use crate::a::d::*; }
mod g { pub use crate::a::b::*; }
mod h { use crate::g::*; }");

                let from_d: Vec<_> = ["X", "Y", "Z", "W"].map(|name| ty(&c, "a::d", name)).into();
                let from_c: Vec<_> = ["X", "Y", "Z", "W"].map(|name| ty(&c, "c", name)).into();

                assert_eq!(
                    from_d,
                    vec![
                        item(&c, "a::b", 0),
                        item(&c, "a::b", 1),
                        Resolved::Unresolved,
                        item(&c, "a::b", 3),
                    ]
                );
                assert_eq!(
                    from_c,
                    vec![
                        Resolved::Unresolved,
                        Resolved::Unresolved,
                        Resolved::Unresolved,
                        item(&c, "a::b", 3),
                    ]
                );
                assert_eq!(ty(&c, "c::inner", "Root"), item(&c, "", 0));
                assert_eq!(ty(&c, "c::inner", "Missing"), item(&c, "", 1));
                assert_eq!(ty(&c, "f", "X"), Resolved::Unresolved);
                assert_eq!(ty(&c, "f", "W"), item(&c, "a::b", 3));
                assert_eq!(ty(&c, "h", "W"), item(&c, "a::b", 3));
            }

            #[test]
            fn resolve_imports_should_start_at_the_crates_of_the_manifest() {
                let c = lib_with_crates(
                    "use serde::Serialize;
use dep::a;
use a::b::C;
use serde_json;
use ::serde::Deserialize;
use other::X;
mod m {
    use std::collections::*;
    use super::Map;
}
use m::HashMap as Map;",
                );

                assert_eq!(ty(&c, "", "Serialize"), ext("serde::Serialize"));
                assert_eq!(ty(&c, "", "Deserialize"), ext("serde::Deserialize"));
                assert_eq!(ty(&c, "m", "serde::de::Error"), ext("serde::de::Error"));
                assert_eq!(ty(&c, "", "C"), ext("dep::a::b::C"));
                assert_eq!(ty(&c, "", "serde_json::Value"), ext("serde_json::Value"));
                assert_eq!(ty(&c, "m", "Map"), ext("std::collections::HashMap"));
                assert_eq!(ty(&c, "", "other::Y"), Resolved::Unresolved);
                assert_eq!(
                    errors(&c),
                    vec!["/c/src/lib.rs:6: unresolved import `other::X`"]
                );
            }

            #[test]
            fn resolve_unresolved_imports_should_be_reported() {
                let c = lib("use crate::Missing;
use crate::gone::X;
use crate::gone::*;
use crate::gone::{self};
use super::Y;
use ::me::Missing as M;
extern crate self as me;");

                assert_eq!(
                    errors(&c),
                    vec![
                        "/c/src/lib.rs:1: unresolved import `crate::Missing`",
                        "/c/src/lib.rs:2: unresolved import `crate::gone::X`",
                        "/c/src/lib.rs:3: unresolved import `crate::gone::*`",
                        "/c/src/lib.rs:4: unresolved import `crate::gone`",
                        "/c/src/lib.rs:5: unresolved import `super::Y`",
                        "/c/src/lib.rs:6: unresolved import `::me::Missing`",
                    ]
                );
                assert_eq!(ty(&c, "", "Missing"), Resolved::Unresolved);
            }

            #[test]
            fn resolve_should_fall_back_to_the_preludes() {
                let c = lib("mod m { struct Vec; }
mod g { use std::collections::*; }");

                assert_eq!(ty(&c, "", "Vec"), ext("std::vec::Vec"));
                assert_eq!(ty(&c, "m", "Vec"), item(&c, "m", 0));
                assert_eq!(
                    resolve(&c, "", "Some", Namespace::Value),
                    ext("std::option::Option::Some")
                );
                assert_eq!(resolve(&c, "", "vec", Namespace::Macro), ext("std::vec"));
                assert_eq!(ty(&c, "", "u64"), Resolved::Primitive("u64".into()));
                assert_eq!(
                    resolve(&c, "", "u64", Namespace::Value),
                    Resolved::Unresolved
                );
                assert_eq!(
                    resolve(&c, "", "std", Namespace::Value),
                    Resolved::Unresolved
                );
                assert_eq!(ty(&c, "", "core::mem"), ext("core::mem"));
                assert_eq!(ty(&c, "g", "HashMap"), ext("std::collections::HashMap"));
                assert_eq!(ty(&c, "", "g::HashSet"), ext("std::collections::HashSet"));
                assert_eq!(ty(&c, "", "Unknown"), Resolved::Unresolved);
            }

            #[test]
            fn resolve_should_find_variants_and_associated_items() {
                let c = lib("enum E { A, B }
struct S;
fn f() {}");
                let e = Resolved::Item(ItemId {
                    module: c.root(),
                    index: 0,
                });
                let s = Resolved::Item(ItemId {
                    module: c.root(),
                    index: 1,
                });
                let value = |path| resolve(&c, "", path, Namespace::Value);

                assert_eq!(
                    value("E::B"),
                    Resolved::Variant(
                        ItemId {
                            module: c.root(),
                            index: 0
                        },
                        1
                    )
                );
                assert_eq!(value("E::new"), associated(&e, "new"));
                assert_eq!(value("S::new"), associated(&s, "new"));
                assert_eq!(
                    value("S::Item::new"),
                    associated(&associated(&s, "Item"), "new")
                );
                assert_eq!(
                    value("u64::MAX"),
                    associated(&Resolved::Primitive("u64".into()), "MAX")
                );
                assert_eq!(value("E::A::x"), Resolved::Unresolved);
                assert_eq!(value("f::x"), Resolved::Unresolved);
                assert_eq!(value("Gone::x"), Resolved::Unresolved);
            }

            #[test]
            fn resolve_macros_should_follow_export_and_textual_scope() {
                let c = lib_with_crates(
                    "#[macro_use]
mod a { macro_rules! from_a { () => {} } }
mod b {
    macro_rules! local { () => {} }
    #[macro_export]
    macro_rules! exported { () => {} }
    use serde_json::json;
    mod inner {}
}
mod c { macro_rules! hidden { () => {} } }",
                );
                let mac = |from, path| resolve(&c, from, path, Namespace::Macro);

                assert_eq!(mac("b", "local"), item(&c, "b", 0));
                assert_eq!(mac("b::inner", "local"), item(&c, "b", 0));
                assert_eq!(mac("c", "crate::exported"), item(&c, "b", 1));
                assert_eq!(mac("c", "exported"), Resolved::Unresolved);
                assert_eq!(mac("c", "from_a"), item(&c, "a", 0));
                assert_eq!(mac("b::inner", "json"), Resolved::Unresolved);
                assert_eq!(mac("b", "hidden"), Resolved::Unresolved);
                assert_eq!(mac("", "b::local"), Resolved::Unresolved);
            }

            #[test]
            fn resolve_single_segment_imports_should_reexport_names_in_scope() {
                let c = lib_with_crates(
                    "mod m {
    macro_rules! bail { () => {} }
    pub(crate) use bail;
    pub(crate) use serde;
    use ::dep;
    use nothing;
}
mod n { use crate::m::*; use bail as again; }",
                );

                assert_eq!(resolve(&c, "n", "bail", Namespace::Macro), item(&c, "m", 0));
                assert_eq!(
                    resolve(&c, "", "m::bail", Namespace::Macro),
                    item(&c, "m", 0)
                );
                assert_eq!(
                    resolve(&c, "n", "again", Namespace::Macro),
                    item(&c, "m", 0)
                );
                assert_eq!(ty(&c, "n", "serde::Serialize"), ext("serde::Serialize"));
                assert_eq!(ty(&c, "m", "dep::X"), ext("dep::X"));
                assert_eq!(
                    errors(&c),
                    vec!["/c/src/lib.rs:6: unresolved import `nothing`"]
                );
            }

            #[test]
            fn resolve_extern_crate_should_bind_the_crate() {
                let c = lib("extern crate alloc as a2;
mod m { extern crate core as k; }");

                assert_eq!(ty(&c, "m", "a2::vec::Vec"), ext("alloc::vec::Vec"));
                assert_eq!(ty(&c, "m", "k::mem"), ext("core::mem"));
                assert_eq!(ty(&c, "", "k::mem"), Resolved::Unresolved);
            }

            #[test]
            fn resolve_imports_without_a_path_should_be_ignored() {
                let c = lib("use {self};\nuse *;");

                assert!(c.errors().is_empty());
            }

            #[test]
            fn resolve_cfg_all_should_keep_the_first_of_duplicate_names() {
                let c = lib("#[cfg(unix)]
use a::X;
#[cfg(windows)]
use b::X;
mod a { pub struct X; }
mod b { pub struct X; }");

                assert_eq!(ty(&c, "", "X"), item(&c, "a", 0));
            }

            #[test]
            fn resolve_local_name_should_replace_the_same_name_of_another_crate() {
                let c = lib_with_crates(
                    "pub use dep::Name;
pub use local::Name;
pub use local::Other;
pub use dep::Other;
mod local { pub trait Name {} pub trait Other {} }",
                );

                assert_eq!(ty(&c, "", "Name"), item(&c, "local", 0));
                assert_eq!(ty(&c, "", "Other"), item(&c, "local", 1));
                assert_eq!(
                    resolve(&c, "", "Name", Namespace::Macro),
                    Resolved::External(vec!["dep".into(), "Name".into()])
                );
            }

            #[test]
            fn resolve_empty_path_should_be_unresolved() {
                let c = lib("");
                let path = syn::Path {
                    leading_colon: None,
                    segments: syn::punctuated::Punctuated::new(),
                };

                assert_eq!(
                    c.resolve(c.root(), &path, Namespace::Type),
                    Resolved::Unresolved
                );
            }
        }
    }
}
