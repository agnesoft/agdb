use super::lower::Roots;
use super::lowered::Conversion;
use super::lowered::Expr;
use super::lowered::Module;
use super::lowered::Ty;
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

/// The module of `Root` of the crate `c` with the tests of a test crate of
/// it, and the errors of both, with `/` separators.
pub(crate) fn tested(source: &str, tests: &str, data: &[&str]) -> (Module, Vec<String>) {
    let platform = test_platform();
    platform.fs().create_dir_all("/c/tests").unwrap();
    platform.fs().write("/c/tests/t.rs", tests).unwrap();
    let mut target = Target::new(TargetKind::Test, "/c/tests/t.rs");
    target.externs.push("c".into());
    let t = SourceCrate::load(&platform, &target, &Cfg::All);
    let (mut m, mut errors) = lower(source, &[], data);
    errors.extend(strings(&t.lower_tests(&load(source), "c", &mut m)));
    (m, errors)
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

/// The module of `Root` after [`Resolve`](super::pass::Resolve) and
/// [`ExplicitReturns`](super::pass::ExplicitReturns), with neither the
/// lowering nor the functions having errors.
pub(crate) fn resolved(source: &str, traits: &[&str], data: &[&str]) -> Module {
    use super::pass::Pass;

    let (mut m, mut errors) = lower(source, traits, data);
    super::pass::Resolve.run(&mut m, &mut Vec::new());
    super::pass::ExplicitReturns.run(&mut m, &mut Vec::new());
    m.for_each_func(&mut |f, _| errors.extend(f.errors.iter().cloned()));
    assert_eq!(errors, Vec::<String>::new());
    m
}

/// A conversion with no generics, location or errors.
pub(crate) fn conversion(to: &str, from: Ty, body: Expr) -> Conversion {
    Conversion {
        to: to.into(),
        from,
        generics: Vec::new(),
        body,
        at: None,
        errors: Vec::new(),
    }
}
