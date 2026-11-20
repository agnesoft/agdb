use crate::CIError;
use crate::roundtrip::host_cfg;
use agplatform::Fs;
use agplatform::Platform;

use agsource::Cfg;
use agsource::CfgSet;
use agsource::Namespace;
use agsource::Resolved;
use agsource::SourceCrate;
use agsource::TargetKind;
use agsource::codegen::lower::Roots;
use agsource::codegen::lowered::Module;
use agsource::codegen::pass::EraseNewtypes;
use agsource::codegen::pass::EraseOwnership;
use agsource::codegen::pass::ExplicitReturns;
use agsource::codegen::pass::InferEmptyLists;
use agsource::codegen::pass::Inline;
use agsource::codegen::pass::LowerConversions;
use agsource::codegen::pass::Passes;
use agsource::codegen::pass::Prune;
use agsource::codegen::pass::Resolve;
use agsource::codegen::pass::TagResults;
use agsource::codegen::pass::TraitDicts;
use agsource::codegen::print;
use agsource::codegen::ts::AcceptElementSources;
use agsource::codegen::ts::Emitter;
use agsource::codegen::ts::EraseCasts;
use agsource::codegen::ts::EraseTraits;
use agsource::codegen::ts::OmitSkippedDefaults;
use agsource::codegen::ts::RemoveDeadHelpers;
use agsource::codegen::ts::TsConfig;
use agsource::codegen::ts::UnescapeIdentifiers;
use std::path::Path;

/// Where `agci generate` dumps the module after each pass.
pub(crate) const DUMP_DIR: &str = "target/agci/generate";

/// The TS query builder, relative to the repository root.
pub(crate) const TS: &str = "agdb_api/typescript/src/query_builder.ts";

/// The source crate of the clients' builder API.
const SOURCE: &str = "agdb";

/// The crate whose public API carries the types the builder API makes.
const API: &str = "agdb_api";

/// The test target of [`SOURCE`] with the tests of the builder API.
const TESTS: &str = "tests/query_builder_test.rs";

/// The name of the test target [`TESTS`].
const TESTS_NAME: &str = "query_builder_test";

/// The TS tests of the builder API, relative to the repository root.
const TS_TESTS: &str = "agdb_api/typescript/tests/query_builder.test.ts";

/// The import of [`TS`] in [`TS_TESTS`].
const TS_MODULE: &str = "../src/query_builder.ts";

/// The query builder of the clients from the agdb source: the lowered module is dumped to `dump`
/// after loading and after each pass as `NN_<pass>.rs.txt`.
pub(crate) fn generate(platform: &impl Platform, root: &str, dump: &Path) -> Result<(), CIError> {
    let packages = agsource::packages(platform, &Path::new(root).join("Cargo.toml"))?;
    let load = |name: &str, kind: TargetKind, root: Option<&str>, cfg: CfgSet| {
        let target = packages
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| {
                p.targets
                    .iter()
                    .find(|t| t.kind == kind && root.is_none_or(|r| t.root.ends_with(r)))
            })
            .ok_or_else(|| format!("generate: no {kind:?} {} in {name}", root.unwrap_or("")))?;
        Ok::<_, CIError>(SourceCrate::load(platform, target, &Cfg::Eval(cfg)))
    };
    let cfg = host_cfg(platform, false)?;
    let features = cfg.clone().feature("serde").feature("derive");
    let c = load(SOURCE, TargetKind::Lib, None, features.clone())?;
    let tests = load(SOURCE, TargetKind::Test, Some(TESTS), features.name("test"))?;
    let (types, _) = load(API, TargetKind::Lib, None, cfg)?.external_api();
    let data = types
        .iter()
        .filter(|p| p.first().is_some_and(|k| k == SOURCE))
        .map(
            |p| match c.resolve_str(&p[1..].join("::"), Namespace::Type) {
                Resolved::Item(id) => Ok(id),
                _ => Err(CIError::from(format!(
                    "generate: no type {} in {SOURCE}",
                    p.join("::")
                ))),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    let roots = Roots::of(&c, data)?;

    let (mut m, mut errors) = c.lower(&roots);
    errors.extend(tests.lower_tests(&c, SOURCE, &mut m));
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
        .with(ExplicitReturns)
        .with(Inline)
        .with(AcceptElementSources)
        .with(LowerConversions)
        .with(TraitDicts)
        .with(EraseOwnership)
        .with(InferEmptyLists)
        .with(TagResults)
        .with(EraseTraits)
        .with(EraseCasts)
        .with(EraseNewtypes)
        .with(UnescapeIdentifiers)
        .with(OmitSkippedDefaults)
        .with(RemoveDeadHelpers);
    let mut failed = None;
    errors.extend(passes.run(&mut m, &mut |title: &str, m: &Module| {
        if let Err(e) = write(title, m) {
            failed.get_or_insert(e);
        }
    }));

    if let Some(e) = failed {
        return Err(e.into());
    }

    let ts = Emitter::new(&m, &ts_config("./openapi.d.ts")).module();
    let tests = Emitter::new(&m, &ts_config("../src/openapi.d.ts")).tests(TESTS_NAME, TS_MODULE);
    fs.write(dump.join("ts_notes.txt"), ts.notes.join("\n") + "\n")?;
    fs.write(Path::new(root).join(TS), ts.code)?;
    fs.write(Path::new(root).join(TS_TESTS), tests.code)?;
    let unsupported: Vec<String> = ts
        .unsupported
        .into_iter()
        .chain(tests.unsupported)
        .collect();

    if !unsupported.is_empty() {
        return Err(format!(
            "generate: {} expressions without a TS form:\n{}",
            unsupported.len(),
            unsupported.join("\n")
        )
        .into());
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

/// The schema types are imported from `openapi`, relative to the emitted file.
fn ts_config(openapi: &str) -> TsConfig {
    TsConfig {
        header: format!("// GENERATED by `agci generate` from {SOURCE}. DO NOT EDIT."),
        import: format!("import type {{ Components }} from {openapi:?};"),
        schemas: "Components.Schemas".into(),
    }
}
