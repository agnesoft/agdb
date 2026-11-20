use crate::CIError;
use crate::utilities;
use agplatform::Platform;
use agsource::Cfg;
use agsource::CfgSet;
use agsource::Package;
use agsource::SourceCrate;
use agsource::TargetKind;
use agsource::codegen::compare::compare;
use std::path::Path;
use std::process::Command;

/// The Rust round trip of the agsource IR: every item of the packages' libraries and integration
/// tests, the unit tests included, printed from its IR is the item as written (see
/// `agsource::codegen::compare`). Fails if an item has a construct the IR does not have or prints
/// differently.
pub(crate) fn roundtrip(
    platform: &impl Platform,
    root: &str,
    packages: &[String],
) -> Result<(), CIError> {
    if packages.is_empty() {
        return Err("roundtrip: no packages given".into());
    }

    let cfg = host_cfg(platform, true)?;
    let all = agsource::packages(platform, &Path::new(root).join("Cargo.toml"))?;
    let mut failures = Vec::new();

    for name in packages {
        let package = all
            .iter()
            .find(|p| &p.name == name)
            .ok_or_else(|| format!("roundtrip: unknown package '{name}'"))?;
        failures.extend(check(platform, package, &cfg));
    }

    if failures.is_empty() {
        return Ok(());
    }

    Err(format!(
        "roundtrip: {} items differ or have constructs the IR does not have:\n{}",
        failures.len(),
        failures.join("\n")
    )
    .into())
}

/// The cfg of a debug build on this host, of `cargo test` with `test`: `rustc --print cfg` with
/// `debug_assertions`.
pub(crate) fn host_cfg(platform: &impl Platform, test: bool) -> Result<CfgSet, CIError> {
    let output = utilities::run_quiet(platform, Command::new("rustc").args(["--print", "cfg"]))?;
    let cfg = ["windows", "unix", "docsrs", "miri", "feature", "test"]
        .into_iter()
        .fold(CfgSet::new(), CfgSet::unset)
        .name("debug_assertions");
    let cfg = if test { cfg.name("test") } else { cfg };

    Ok(parse_cfg(&output)
        .into_iter()
        .fold(cfg, |cfg, (name, value)| match value {
            Some(value) => cfg.value(name, value),
            None => cfg.name(name),
        }))
}

/// The lines of `rustc --print cfg`: `name` or `key="value"`.
fn parse_cfg(output: &str) -> Vec<(&str, Option<&str>)> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| match line.split_once('=') {
            Some((key, value)) => (key, Some(value.trim_matches('"'))),
            None => (line, None),
        })
        .collect()
}

/// The failures of the items of the package's library and integration tests.
fn check(platform: &impl Platform, package: &Package, cfg: &CfgSet) -> Vec<String> {
    let cfg = package
        .features
        .iter()
        .fold(cfg.clone(), |cfg, feature| cfg.feature(feature));
    let mut failures = Vec::new();

    for target in package
        .targets
        .iter()
        .filter(|t| matches!(t.kind, TargetKind::Lib | TargetKind::Test))
    {
        let c = SourceCrate::load(platform, target, &Cfg::Eval(cfg.clone()));
        let name = format!("{} {}", package.name, target.root.display());

        // WHY: these do not fail the check, e.g. imports of names a derive generates.
        for e in c.errors() {
            println!("[roundtrip] {name}: {e}");
        }

        let (items, unsupported) = c.ir(c.item_ids());
        let differ: Vec<_> = items
            .iter()
            .filter_map(|(id, item)| compare(&c, *id, item).err())
            .collect();
        println!(
            "[roundtrip] {name}: {} items equal, {} differ, {} unsupported constructs",
            items.len() - differ.len(),
            differ.len(),
            unsupported.len()
        );
        failures.extend(unsupported.iter().chain(&differ).map(ToString::to_string));
    }

    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cfg_names_and_values() {
        assert_eq!(
            parse_cfg("debug_assertions\ntarget_os=\"linux\"\n\nunix\n"),
            vec![
                ("debug_assertions", None),
                ("target_os", Some("linux")),
                ("unix", None)
            ]
        );
    }
}
