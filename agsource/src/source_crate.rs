use crate::Cfg;
use crate::Error;
use crate::Target;
use crate::cfg::configure;
use crate::resolve::Scopes;
use agplatform::Fs;
use agplatform::Platform;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use syn::Item;
use syn::ext::IdentExt;

/// Index of a [`SourceFile`] in its [`SourceCrate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(usize);

/// Index of a [`Module`] in its [`SourceCrate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(pub(crate) usize);

/// An item of a [`Module`]: the module and the index among its
/// [`SourceCrate::items`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId {
    pub(crate) module: ModuleId,
    pub(crate) index: usize,
}

/// A loaded source file. A file that could not be read or parsed has no
/// syntax tree, the reason is among the [`SourceCrate::errors`].
pub struct SourceFile {
    path: PathBuf,
    text: String,
    ast: Option<syn::File>,
}

/// A module of the crate: the root, an out-of-line `mod x;` with its own file
/// or an inline `mod x { .. }` within the file of its parent.
#[derive(Debug)]
pub struct Module {
    path: Vec<String>,
    parent: Option<ModuleId>,
    children: Vec<ModuleId>,
    file: FileId,
    item: Option<ItemId>,
    // Item indices of the inline `mod` blocks leading from the file's items
    // to the module's.
    inline: Vec<usize>,
    // The directory of the module's submodules is `dir`, plus `relative` for
    // a non-mod-rs file (`a.rs` looks for `a/x.rs`), as in rustc.
    dir: PathBuf,
    relative: Option<String>,
}

/// The module tree of a crate, from its root file over the parsed files.
#[derive(Default)]
pub struct SourceCrate {
    cfg: Cfg,
    files: Vec<SourceFile>,
    modules: Vec<Module>,
    pub(crate) scopes: Scopes,
    errors: Vec<Error>,
}

struct Declaration {
    index: usize,
    name: String,
    line: usize,
    path: Option<String>,
    inline: bool,
}

impl SourceFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn ast(&self) -> Option<&syn::File> {
        self.ast.as_ref()
    }
}

impl ItemId {
    pub fn module(self) -> ModuleId {
        self.module
    }
}

impl Module {
    /// The segments after `crate`, empty for the root.
    pub fn path(&self) -> &[String] {
        &self.path
    }

    /// The module's name, `crate` for the root.
    pub fn name(&self) -> &str {
        self.path.last().map_or("crate", String::as_str)
    }

    pub fn parent(&self) -> Option<ModuleId> {
        self.parent
    }

    pub fn children(&self) -> &[ModuleId] {
        &self.children
    }

    pub fn file(&self) -> FileId {
        self.file
    }

    /// The `mod` item declaring the module, none for the root.
    pub fn item(&self) -> Option<ItemId> {
        self.item
    }

    fn submodule_dir(&self) -> PathBuf {
        match &self.relative {
            Some(relative) => self.dir.join(relative),
            None => self.dir.clone(),
        }
    }
}

impl SourceCrate {
    /// Loads the crate from the root file of `target` (`lib.rs`, `main.rs`,
    /// an integration test, ...), following its `mod` declarations, `#[path]`
    /// included, with the code selected by `cfg`. A module whose `#[cfg]`
    /// or own `#![cfg]` does not hold is not loaded. The code can name the
    /// crates of the target's `externs`.
    pub fn load(p: &impl Platform, target: &Target, cfg: &Cfg) -> Self {
        let root = normalize(&target.root);
        let mut source_crate = Self {
            cfg: cfg.clone(),
            ..Self::default()
        };
        let (file, _) = source_crate.read(p, &root);
        source_crate.modules.push(Module {
            path: Vec::new(),
            parent: None,
            children: Vec::new(),
            file,
            item: None,
            inline: Vec::new(),
            dir: parent_dir(&root),
            relative: None,
        });

        let mut next = 0;

        while next < source_crate.modules.len() {
            let id = ModuleId(next);

            for declaration in source_crate.declarations(id) {
                if let Some(child) = source_crate.declare(p, id, declaration) {
                    source_crate.modules[id.0].children.push(child);
                }
            }

            next += 1;
        }

        let (scopes, errors) = Scopes::build(&source_crate, target.externs.clone());
        source_crate.scopes = scopes;
        source_crate.errors.extend(errors);
        source_crate
    }

    pub fn root(&self) -> ModuleId {
        ModuleId(0)
    }

    pub fn module(&self, id: ModuleId) -> &Module {
        &self.modules[id.0]
    }

    /// All modules, parents before their children.
    pub fn modules(&self) -> impl Iterator<Item = (ModuleId, &Module)> {
        self.modules
            .iter()
            .enumerate()
            .map(|(i, module)| (ModuleId(i), module))
    }

    pub fn file(&self, id: FileId) -> &SourceFile {
        &self.files[id.0]
    }

    pub fn files(&self) -> impl Iterator<Item = (FileId, &SourceFile)> {
        self.files
            .iter()
            .enumerate()
            .map(|(i, file)| (FileId(i), file))
    }

    /// The items of the module, empty when its file has no syntax tree.
    pub fn items(&self, id: ModuleId) -> &[Item] {
        let module = &self.modules[id.0];
        let mut items: &[Item] = self.files[module.file.0]
            .ast
            .as_ref()
            .map_or(&[], |ast| &ast.items);

        for &index in &module.inline {
            items = items
                .get(index)
                .and_then(as_mod)
                .and_then(|m| m.content.as_ref())
                .map_or(&[], |(_, inner)| inner.as_slice());
        }

        items
    }

    pub fn item(&self, id: ItemId) -> &Item {
        &self.items(id.module)[id.index]
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    fn declarations(&self, id: ModuleId) -> Vec<Declaration> {
        self.items(id)
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                as_mod(item).map(|m| Declaration {
                    index,
                    name: m.ident.unraw().to_string(),
                    line: m.ident.span().start().line,
                    path: path_attribute(&m.attrs),
                    inline: m.content.is_some(),
                })
            })
            .collect()
    }

    fn declare(
        &mut self,
        p: &impl Platform,
        parent_id: ModuleId,
        declaration: Declaration,
    ) -> Option<ModuleId> {
        let parent = &self.modules[parent_id.0];
        let mut path = parent.path.clone();
        path.push(declaration.name.clone());
        let item = Some(ItemId {
            module: parent_id,
            index: declaration.index,
        });

        if declaration.inline {
            // `#[path]` on an inline module names its directory.
            let dir = match &declaration.path {
                Some(dir) => normalize(&parent.dir.join(dir)),
                None => normalize(&parent.submodule_dir().join(&declaration.name)),
            };
            let mut inline = parent.inline.clone();
            inline.push(declaration.index);
            let module = Module {
                path,
                parent: Some(parent_id),
                children: Vec::new(),
                file: parent.file,
                item,
                inline,
                dir,
                relative: None,
            };
            return Some(self.push(module));
        }

        let declared_in = self.files[parent.file.0].path.clone();
        let (file_path, relative) = self.module_file(p, parent_id, &declared_in, &declaration)?;

        if let Some(file) = self.enclosing_file(parent_id, &file_path) {
            self.errors.push(Error::CircularModule {
                path: declared_in,
                line: declaration.line,
                name: declaration.name,
                file,
            });
            return None;
        }

        let (file, holds) = self.read(p, &file_path);

        if !holds {
            return None;
        }

        let module = Module {
            path,
            parent: Some(parent_id),
            children: Vec::new(),
            file,
            item,
            inline: Vec::new(),
            dir: parent_dir(&file_path),
            relative,
        };
        Some(self.push(module))
    }

    /// The file of `mod name;` and its non-mod-rs name. A `#[path]` file
    /// is treated as a mod-rs file, as in rustc.
    fn module_file(
        &mut self,
        p: &impl Platform,
        parent_id: ModuleId,
        declared_in: &Path,
        declaration: &Declaration,
    ) -> Option<(PathBuf, Option<String>)> {
        let parent = &self.modules[parent_id.0];
        let candidates = match &declaration.path {
            Some(file) => vec![(normalize(&parent.dir.join(file)), None)],
            None => {
                let dir = parent.submodule_dir();
                vec![
                    (
                        normalize(&dir.join(format!("{}.rs", declaration.name))),
                        Some(declaration.name.clone()),
                    ),
                    (normalize(&dir.join(&declaration.name).join("mod.rs")), None),
                ]
            }
        };
        let found: Vec<_> = candidates
            .iter()
            .filter(|(file, _)| p.fs().exists(file).unwrap_or(false))
            .cloned()
            .collect();
        let candidates = candidates.into_iter().map(|(file, _)| file).collect();

        match <[_; 1]>::try_from(found) {
            Ok([found]) => Some(found),
            Err(found) => {
                let path = declared_in.to_path_buf();
                let line = declaration.line;
                let name = declaration.name.clone();
                self.errors.push(if found.is_empty() {
                    Error::ModuleNotFound {
                        path,
                        line,
                        name,
                        candidates,
                    }
                } else {
                    Error::ModuleAmbiguous {
                        path,
                        line,
                        name,
                        candidates,
                    }
                });
                None
            }
        }
    }

    fn enclosing_file(&self, mut id: ModuleId, file: &Path) -> Option<PathBuf> {
        loop {
            let module = &self.modules[id.0];
            let path = &self.files[module.file.0].path;

            if path == file {
                return Some(path.clone());
            }

            id = module.parent?;
        }
    }

    fn push(&mut self, module: Module) -> ModuleId {
        self.modules.push(module);
        ModuleId(self.modules.len() - 1)
    }

    /// Reads and configures the file, returns whether its `#![cfg]` holds.
    fn read(&mut self, p: &impl Platform, path: &Path) -> (FileId, bool) {
        let mut holds = true;
        let (text, ast) = match p.fs().read_to_string(path) {
            Ok(text) => match syn::parse_file(&text) {
                Ok(mut ast) => {
                    holds = configure(&self.cfg, path, &mut ast, &mut self.errors);
                    (text, Some(ast))
                }
                Err(e) => {
                    self.errors.push(parse_error(path, &text, &e));
                    (text, None)
                }
            },
            Err(error) => {
                self.errors.push(Error::Read {
                    path: path.to_path_buf(),
                    error,
                });
                (String::new(), None)
            }
        };

        self.files.push(SourceFile {
            path: path.to_path_buf(),
            text,
            ast,
        });
        (FileId(self.files.len() - 1), holds)
    }
}

fn as_mod(item: &Item) -> Option<&syn::ItemMod> {
    match item {
        Item::Mod(m) => Some(m),
        _ => None,
    }
}

fn path_attribute(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        syn::Meta::NameValue(syn::MetaNameValue {
            path,
            value:
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                }),
            ..
        }) if path.is_ident("path") => Some(value.value()),
        _ => None,
    })
}

fn parse_error(path: &Path, text: &str, e: &syn::Error) -> Error {
    // Spans are relative to the content without the BOM.
    let content = text.strip_prefix('\u{feff}').unwrap_or(text);
    let span = e.span();

    // Errors at the end of input carry an empty call site span.
    let (line, column) = if span.byte_range().is_empty() {
        let last_line = content.rsplit('\n').next().unwrap_or_default();
        (content.matches('\n').count() + 1, last_line.chars().count())
    } else {
        (span.start().line, span.start().column)
    };

    Error::Parse {
        path: path.to_path_buf(),
        line,
        column: column + 1,
        message: e.to_string(),
    }
}

fn parent_dir(path: &Path) -> PathBuf {
    path.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// Resolves `.` and `..` without touching the file system, so the same file
/// reached on different paths compares equal.
pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) =>
            {
                normalized.pop();
            }
            Component::ParentDir
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::RootDir)
                ) => {}
            _ => normalized.push(component),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CfgSet;
    use crate::TargetKind;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    fn platform(files: &[(&str, &str)]) -> TestPlatform {
        let platform = test_platform();

        for (path, content) in files {
            platform
                .fs()
                .create_dir_all(Path::new(path).parent().unwrap())
                .unwrap();
            platform.fs().write(path, content).unwrap();
        }

        platform
    }

    fn lib(root: &str) -> Target {
        Target::new(TargetKind::Lib, root)
    }

    fn module_files(source_crate: &SourceCrate) -> Vec<(String, PathBuf)> {
        source_crate
            .modules()
            .map(|(_, module)| {
                (
                    module.path().join("::"),
                    source_crate.file(module.file()).path().to_path_buf(),
                )
            })
            .collect()
    }

    // WHY: joined paths display with `\` on Windows.
    fn errors(source_crate: &SourceCrate) -> Vec<String> {
        source_crate
            .errors()
            .iter()
            .map(|e| e.to_string().replace('\\', "/"))
            .collect()
    }

    fn entry(path: &str, file: &str) -> (String, PathBuf) {
        (path.to_string(), PathBuf::from(file))
    }

    mod module {
        use super::*;

        mod name {
            use super::*;

            #[test]
            fn name_should_be_crate_for_the_root_and_the_last_segment_otherwise() {
                let platform = platform(&[("/c/src/lib.rs", "mod a { mod b {} }")]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                let names: Vec<_> = source_crate.modules().map(|(_, m)| m.name()).collect();
                assert_eq!(names, vec!["crate", "a", "b"]);
            }
        }
    }

    mod item_id {
        use super::*;

        mod module {
            use super::*;

            #[test]
            fn module_should_be_the_module_of_the_item() {
                let platform = platform(&[("/c/src/lib.rs", "mod a { struct A; }")]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                let a = source_crate.module(source_crate.root()).children()[0];
                let declaration = source_crate.module(a).item().unwrap();
                assert_eq!(declaration.module(), source_crate.root());
                assert!(matches!(source_crate.item(declaration), Item::Mod(m) if m.ident == "a"));
            }
        }
    }

    mod source_crate {
        use super::*;

        mod load {
            use super::*;

            #[test]
            fn load_should_follow_out_of_line_modules() {
                let platform = platform(&[
                    ("/c/src/lib.rs", "mod a;\nmod b;"),
                    ("/c/src/a.rs", "mod x;"),
                    ("/c/src/a/x.rs", ""),
                    ("/c/src/b/mod.rs", "mod y;"),
                    ("/c/src/b/y.rs", ""),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(
                    module_files(&source_crate),
                    vec![
                        entry("", "/c/src/lib.rs"),
                        entry("a", "/c/src/a.rs"),
                        entry("b", "/c/src/b/mod.rs"),
                        entry("a::x", "/c/src/a/x.rs"),
                        entry("b::y", "/c/src/b/y.rs"),
                    ]
                );
                assert!(source_crate.errors().is_empty());
            }

            #[test]
            fn load_cfg_gated_modules_should_be_followed() {
                let platform = platform(&[
                    (
                        "/c/src/lib.rs",
                        "#[cfg(test)]\nmod tests;\n#[cfg(feature = \"x\")]\nmod x;",
                    ),
                    ("/c/src/tests.rs", ""),
                    ("/c/src/x.rs", ""),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(
                    module_files(&source_crate),
                    vec![
                        entry("", "/c/src/lib.rs"),
                        entry("tests", "/c/src/tests.rs"),
                        entry("x", "/c/src/x.rs"),
                    ]
                );
            }

            #[test]
            fn load_eval_should_skip_modules_whose_cfg_does_not_hold() {
                let platform = platform(&[
                    (
                        "/c/src/lib.rs",
                        "#[cfg(feature = \"off\")]\nmod gone;\nmod x;\n#[cfg_attr(feature = \"on\", path = \"other.rs\")]\nmod y;",
                    ),
                    ("/c/src/x.rs", "#![cfg(feature = \"off\")]"),
                    ("/c/src/other.rs", ""),
                ]);
                let cfg = Cfg::Eval(CfgSet::new().feature("on"));

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &cfg);

                assert_eq!(
                    module_files(&source_crate),
                    vec![entry("", "/c/src/lib.rs"), entry("y", "/c/src/other.rs")]
                );
                assert!(source_crate.errors().is_empty());
                assert_eq!(source_crate.files().count(), 3);
            }

            #[test]
            fn load_inline_module_should_nest_the_directory_of_its_submodules() {
                let platform = platform(&[
                    ("/c/src/lib.rs", "mod a;\nmod i { mod x; }"),
                    ("/c/src/a.rs", "mod j { mod y; }"),
                    ("/c/src/i/x.rs", ""),
                    ("/c/src/a/j/y.rs", ""),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(
                    module_files(&source_crate),
                    vec![
                        entry("", "/c/src/lib.rs"),
                        entry("a", "/c/src/a.rs"),
                        entry("i", "/c/src/lib.rs"),
                        entry("a::j", "/c/src/a.rs"),
                        entry("i::x", "/c/src/i/x.rs"),
                        entry("a::j::y", "/c/src/a/j/y.rs"),
                    ]
                );
                assert!(source_crate.errors().is_empty());
            }

            #[test]
            fn load_path_attribute_should_be_relative_to_the_declaring_file() {
                let platform = platform(&[
                    ("/c/src/lib.rs", "mod a;"),
                    ("/c/src/a.rs", "#[path = \"../other/o.rs\"]\nmod x;"),
                    ("/c/other/o.rs", "mod y;"),
                    ("/c/other/y.rs", ""),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(
                    module_files(&source_crate),
                    vec![
                        entry("", "/c/src/lib.rs"),
                        entry("a", "/c/src/a.rs"),
                        entry("a::x", "/c/other/o.rs"),
                        entry("a::x::y", "/c/other/y.rs"),
                    ]
                );
            }

            #[test]
            fn load_inline_path_attribute_should_set_the_directory() {
                let platform = platform(&[
                    ("/c/src/lib.rs", "#[path = \"platform\"]\nmod i { mod x; }"),
                    ("/c/src/platform/x.rs", ""),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(
                    module_files(&source_crate),
                    vec![
                        entry("", "/c/src/lib.rs"),
                        entry("i", "/c/src/lib.rs"),
                        entry("i::x", "/c/src/platform/x.rs"),
                    ]
                );
            }

            #[test]
            fn load_raw_identifier_should_look_for_the_unraw_name() {
                let platform =
                    platform(&[("/c/src/lib.rs", "mod r#type;"), ("/c/src/type.rs", "")]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(
                    module_files(&source_crate)[1],
                    entry("type", "/c/src/type.rs")
                );
            }

            #[test]
            fn load_relative_root_should_resolve_beside_it() {
                let platform = platform(&[("/lib.rs", "mod a;"), ("/a.rs", "")]);

                let source_crate = SourceCrate::load(&platform, &lib("./lib.rs"), &Cfg::All);

                assert_eq!(
                    module_files(&source_crate),
                    vec![entry("", "lib.rs"), entry("a", "a.rs")]
                );
            }

            #[test]
            fn load_missing_module_should_report_the_candidates() {
                let platform =
                    platform(&[("/c/src/lib.rs", "mod a;\n#[path = \"gone.rs\"]\nmod b;")]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(source_crate.modules().count(), 1);
                assert_eq!(
                    errors(&source_crate),
                    vec![
                        "/c/src/lib.rs:1: file not found for module `a`: /c/src/a.rs or /c/src/a/mod.rs",
                        "/c/src/lib.rs:3: file not found for module `b`: /c/src/gone.rs",
                    ]
                );
            }

            #[test]
            fn load_ambiguous_module_should_report_both_files() {
                let platform = platform(&[
                    ("/c/src/lib.rs", "mod a;"),
                    ("/c/src/a.rs", ""),
                    ("/c/src/a/mod.rs", ""),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(source_crate.modules().count(), 1);
                assert_eq!(
                    errors(&source_crate),
                    vec![
                        "/c/src/lib.rs:1: file for module `a` found at both /c/src/a.rs and /c/src/a/mod.rs"
                    ]
                );
            }

            #[test]
            fn load_circular_module_should_report_and_stop() {
                let platform = platform(&[
                    ("/c/src/lib.rs", "mod a;"),
                    ("/c/src/a.rs", "#[path = \"a.rs\"]\nmod again;"),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(source_crate.modules().count(), 2);
                assert_eq!(
                    errors(&source_crate),
                    vec![
                        "/c/src/a.rs:2: circular module `again`: /c/src/a.rs is already an enclosing module"
                    ]
                );
            }

            #[test]
            fn load_invalid_file_should_report_the_location_and_keep_the_module() {
                let platform = platform(&[
                    ("/c/src/lib.rs", "mod a;\nmod b;"),
                    ("/c/src/a.rs", "\u{feff}fn f() {\n    let x = ;\n}"),
                    ("/c/src/b.rs", "fn g() {\n    let y = 1;"),
                ]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(source_crate.modules().count(), 3);
                assert!(source_crate.items(ModuleId(1)).is_empty());
                assert_eq!(
                    errors(&source_crate),
                    vec![
                        "/c/src/a.rs:2:13: expected an expression",
                        "/c/src/b.rs:2:15: cannot parse string into token stream",
                    ]
                );
            }

            #[test]
            fn load_unreadable_file_should_report_it() {
                let platform = platform(&[("/c/src/lib.rs", "mod a;"), ("/c/src/a.rs", "")]);
                platform
                    .fs
                    .set_io_error(FsOp::Open, "/c/src/a.rs", ErrorKind::PermissionDenied);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                assert_eq!(source_crate.modules().count(), 2);
                assert_eq!(source_crate.file(FileId(1)).text(), "");
                assert!(source_crate.file(FileId(1)).ast().is_none());
                assert!(matches!(
                    source_crate.errors(),
                    [Error::Read { path, .. }] if path == Path::new("/c/src/a.rs")
                ));
            }
        }

        mod items {
            use super::*;

            #[test]
            fn items_should_return_the_content_of_inline_modules() {
                let platform = platform(&[(
                    "/c/src/lib.rs",
                    "struct A;\nmod i { struct B; mod j { struct C; } }",
                )]);

                let source_crate = SourceCrate::load(&platform, &lib("/c/src/lib.rs"), &Cfg::All);

                let structs = |id: ModuleId| {
                    source_crate
                        .items(id)
                        .iter()
                        .filter_map(|item| match item {
                            Item::Struct(s) => Some(s.ident.to_string()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                };
                let ids: Vec<_> = source_crate.modules().map(|(id, _)| id).collect();
                assert_eq!(
                    ids.iter().map(|id| structs(*id)).collect::<Vec<_>>(),
                    vec![vec!["A"], vec!["B"], vec!["C"]]
                );
                assert_eq!(source_crate.root(), ids[0]);
                assert_eq!(source_crate.module(ids[2]).parent(), Some(ids[1]));
                assert_eq!(source_crate.module(ids[1]).children(), &[ids[2]]);
                assert_eq!(source_crate.files().count(), 1);
            }
        }
    }

    mod normalize {
        use super::*;

        #[test]
        fn normalize_should_resolve_dots_lexically() {
            assert_eq!(
                normalize(Path::new("/a/./b/../c.rs")),
                PathBuf::from("/a/c.rs")
            );
            assert_eq!(
                normalize(Path::new("../a/../../b")),
                PathBuf::from("../../b")
            );
        }

        #[test]
        fn normalize_should_handle_root_dir() {
            assert_eq!(normalize(Path::new("/../a/b")), PathBuf::from("/a/b"));
        }
    }
}
