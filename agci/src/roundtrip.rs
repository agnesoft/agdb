use crate::CIError;
use crate::utilities;
use agplatform::Fs;
use agplatform::Platform;
use agsource::Cfg;
use agsource::CfgSet;
use agsource::Export;
use agsource::Package;
use agsource::SourceCrate;
use agsource::TargetKind;
use std::path::Path;
use std::process::Command;

/// The copy of the repository, and the target directory of its build (kept between the runs).
const DIR: &str = "target/agci/roundtrip";

/// The Rust round trip of the agsource IR: in a copy of the repository, the items that the exports
/// of the libraries of the packages reach are replaced by the Rust printed from their IR, then the
/// tests of the packages run there (`cargo test --all-features`). Fails if an item has a construct
/// the IR does not have.
pub(crate) fn roundtrip(
    platform: &impl Platform,
    root: &str,
    packages: &[String],
) -> Result<(), CIError> {
    if packages.is_empty() {
        return Err("roundtrip: no packages given".into());
    }

    let dir = Path::new(root).join(DIR);
    let src = dir.join("src");
    copy(platform, root, &src)?;
    let cfg = host_cfg(platform)?;
    let all = agsource::packages(platform, &src.join("Cargo.toml"))?;

    for name in packages {
        let package = all
            .iter()
            .find(|p| &p.name == name)
            .ok_or_else(|| format!("roundtrip: unknown package '{name}'"))?;
        rewrite(platform, package, &cfg)?;
    }

    let mut cargo = Command::new("cargo");
    cargo.arg("test");

    for name in packages {
        cargo.args(["-p", name]);
    }

    cargo
        .arg("--all-features")
        .current_dir(&src)
        .env("CARGO_TARGET_DIR", dir.join("target"));
    utilities::stream_command(platform, &cargo, &mut std::io::stdout())
}

/// The files git does not ignore, the uncommitted ones included, into the empty `to`.
fn copy(platform: &impl Platform, root: &str, to: &Path) -> Result<(), CIError> {
    let fs = platform.fs();

    if fs.exists(to)? {
        fs.remove_dir_all(to)?;
    }

    let files = utilities::run_quiet(
        platform,
        Command::new("git")
            .args([
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
            ])
            .current_dir(root),
    )?;

    for file in files.split('\0').filter(|f| !f.is_empty()) {
        let from = Path::new(root).join(file);

        // WHY: a file deleted in the worktree is still in the index.
        if fs.exists(&from)? {
            let to = to.join(file);
            fs.create_dir_all(to.parent().unwrap_or(&to))?;
            fs.copy(&from, &to)?;
        }
    }

    Ok(())
}

/// The cfg of `cargo test` on this host: `rustc --print cfg` with `test` and `debug_assertions`.
fn host_cfg(platform: &impl Platform) -> Result<CfgSet, CIError> {
    let output = utilities::run_quiet(platform, Command::new("rustc").args(["--print", "cfg"]))?;
    let cfg = ["windows", "unix", "docsrs", "miri", "feature"]
        .into_iter()
        .fold(CfgSet::new(), CfgSet::unset)
        .name("test")
        .name("debug_assertions");

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

/// Replaces the reached items of the package's library in its files.
fn rewrite(platform: &impl Platform, package: &Package, cfg: &CfgSet) -> Result<(), CIError> {
    let lib = package
        .targets
        .iter()
        .find(|t| t.kind == TargetKind::Lib)
        .ok_or_else(|| format!("roundtrip: {} has no library", package.name))?;
    let cfg = package
        .features
        .iter()
        .fold(cfg.clone(), |cfg, feature| cfg.feature(feature));
    let c = SourceCrate::load(platform, lib, &Cfg::Eval(cfg));

    // WHY: the build tells whether these matter, e.g. imports of names a derive generates.
    for e in c.errors() {
        println!("[roundtrip] {}: {e}", package.name);
    }

    let exports = c.exports();
    let roots: Vec<&Export> = exports.iter().collect();
    let reachable = c.reachable(&roots, &[]);
    let (items, errors) = c.ir(reachable.items.iter().copied());

    if !errors.is_empty() {
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        return Err(format!(
            "roundtrip: {} has constructs the IR does not have:\n{}",
            package.name,
            errors.join("\n")
        )
        .into());
    }

    for (path, text) in agsource::codegen::rust::rewrite(&c, &items) {
        platform.fs().write(path, text)?;
    }

    println!(
        "[roundtrip] {}: {} of {} reached items rewritten",
        package.name,
        items.len(),
        reachable.items.len()
    );
    Ok(())
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
