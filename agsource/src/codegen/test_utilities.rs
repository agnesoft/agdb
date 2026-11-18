use super::lower::Roots;
use super::lowered::Module;
use crate::Cfg;
use crate::ItemId;
use crate::Namespace;
use crate::Resolved;
use crate::SourceCrate;
use crate::Target;
use crate::TargetKind;
use agplatform::Fs;
use agplatform::Platform;
use agplatform::test_platform::test_platform;

pub(crate) fn load(source: &str) -> SourceCrate {
    let platform = test_platform();
    platform.fs().create_dir_all("/c/src").unwrap();
    platform.fs().write("/c/src/lib.rs", source).unwrap();
    SourceCrate::load(
        &platform,
        &Target::new(TargetKind::Lib, "/c/src/lib.rs"),
        &Cfg::All,
    )
}

pub(crate) fn id(c: &SourceCrate, path: &str) -> ItemId {
    c.item_ids()
        .find(|id| c.resolve_str(path, Namespace::Type) == Resolved::Item(*id))
        .unwrap()
}

/// The module of `Root` and the errors, with `/` separators.
pub(crate) fn lower(source: &str, traits: &[&str], data: &[&str]) -> (Module, Vec<String>) {
    let c = load(source);
    let roots = Roots {
        root: id(&c, "Root"),
        traits: traits.iter().map(|t| id(&c, t)).collect(),
        data: data.iter().map(|d| id(&c, d)).collect(),
    };
    let (m, errors) = c.lower(&roots);
    (m, strings(&errors))
}

pub(crate) fn strings(errors: &[crate::Error]) -> Vec<String> {
    errors
        .iter()
        .map(|e| e.to_string().replace('\\', "/"))
        .collect()
}
