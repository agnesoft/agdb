mod facts;

use crate::CIError;
use crate::roundtrip::host_cfg;
use agplatform::Fs;
use agplatform::Platform;

use agsource::Cfg;
use agsource::Namespace;
use agsource::Resolved;
use agsource::SourceCrate;
use agsource::TargetKind;
use agsource::codegen::lower::Roots;
use agsource::codegen::lowered::Module;
use agsource::codegen::pass::ExplicitReturns;
use agsource::codegen::pass::Passes;
use agsource::codegen::pass::Prune;
use agsource::codegen::pass::Resolve;
use agsource::codegen::print;
use std::path::Path;

/// Where `agci generate` dumps the module after each pass.
pub(crate) const DUMP_DIR: &str = "target/agci/generate";

/// The query builder of the clients from the agdb source: the lowered module of `QueryBuilder` is
/// dumped to `dump` after loading and after each pass as `NN_<pass>.rs.txt`.
pub(crate) fn generate(platform: &impl Platform, root: &str, dump: &Path) -> Result<(), CIError> {
    let agdb = agsource::packages(platform, &Path::new(root).join("Cargo.toml"))?
        .into_iter()
        .find(|p| p.name == "agdb")
        .ok_or("generate: no package agdb")?;
    let lib = agdb
        .targets
        .iter()
        .find(|t| t.kind == TargetKind::Lib)
        .ok_or("generate: agdb has no library")?;
    let cfg = host_cfg(platform, false)?
        .feature("serde")
        .feature("derive");
    let c = SourceCrate::load(platform, lib, &Cfg::Eval(cfg));
    let item = |path: &str| match c.resolve_str(path, Namespace::Type) {
        Resolved::Item(id) => Ok(id),
        _ => Err(CIError::from(format!("generate: no type {path} in agdb"))),
    };
    let roots = Roots {
        root: item("QueryBuilder")?,
        traits: vec![item("query_builder::search::SearchQueryBuilder")?],
        data: vec![
            item("QueryResult")?,
            item("QueryType")?,
            item("DbKeyValue")?,
        ],
    };

    let (mut m, mut errors) = c.lower(&roots);
    add_facts(&mut m);

    let fs = platform.fs();
    fs.create_dir_all(dump)?;
    let mut step = 0;
    let mut write = |title: &str, m: &Module| {
        let file = dump.join(format!("{step:02}_{title}.rs.txt"));
        step += 1;
        fs.write(&file, print::module(m, title))
    };
    write("load", &m)?;

    let passes = Passes::new()
        .with(Resolve)
        .with(Prune)
        .with(ExplicitReturns);
    let mut failed = None;
    errors.extend(passes.run(&mut m, &mut |title: &str, m: &Module| {
        if let Err(e) = write(title, m) {
            failed.get_or_insert(e);
        }
    }));

    if let Some(e) = failed {
        return Err(e.into());
    }

    if errors.is_empty() {
        return Ok(());
    }

    Err(format!(
        "generate: {} constructs the lowering does not handle:\n{}",
        errors.len(),
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    )
    .into())
}

fn add_facts(m: &mut Module) {
    m.traits.insert("DbType".into(), facts::db_type_trait());
    m.conversions = facts::conversions();
    m.defaults.extend(facts::defaults());
    m.skipped_defaults = facts::skipped_defaults();
}
