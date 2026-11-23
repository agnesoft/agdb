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
use agsource::codegen::pass::DERIVED;
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
use agsource::codegen::pass::ThrowErrors;
use agsource::codegen::pass::TraitDicts;
use agsource::codegen::print;
use agsource::codegen::routes;
use agsource::codegen::ts::AcceptElementSources;
use agsource::codegen::ts::Emitter;
use agsource::codegen::ts::EraseCasts;
use agsource::codegen::ts::EraseTraits;
use agsource::codegen::ts::OmitSkippedDefaults;
use agsource::codegen::ts::RemoveDeadHelpers;
use agsource::codegen::ts::RenameShadowed;
use agsource::codegen::ts::TsConfig;
use agsource::codegen::ts::UnescapeIdentifiers;
use std::path::Path;

/// Where `agci generate` dumps the module after each pass.
pub(crate) const DUMP_DIR: &str = "target/agci/generate";

/// The TS query builder, relative to the repository root.
pub(crate) const TS: &str = "agdb_api/typescript/src/query_builder.ts";

/// The TS declarations of the serde types, relative to the repository root.
pub(crate) const TS_TYPES: &str = "agdb_api/typescript/src/types.ts";

/// The TS client generated from [`API`], relative to the repository root.
pub(crate) const TS_API: &str = "agdb_api/typescript/src/api.ts";

/// Where `agci generate` dumps the client module after each pass, in [`DUMP_DIR`].
const CLIENT_DUMP: &str = "client";

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

/// The handwritten TS of the trait impls the types of [`TESTS`] derive, relative to [`TS_TESTS`].
const TS_DERIVED: &str = "./query_builder_derived.ts";

/// The package with the tests of the routes the client calls.
const SERVER: &str = "agdb_server";

/// The test target of [`SERVER`] with the [`SUITE`].
const SERVER_TESTS: &str = "tests/tests.rs";

/// The module of [`SERVER_TESTS`] with the tests of the routes.
const SUITE: &str = "api";

/// The module of [`SERVER_TESTS`] whose items the TS tests import from [`TS_GLUE`].
const GLUE: &str = "test_server";

/// The handwritten TS of [`GLUE`], relative to [`TS_ROUTES`].
const TS_GLUE: &str = "./test_server.ts";

/// The TS tests of the routes, relative to the repository root.
pub(crate) const TS_ROUTES: &str = "agdb_api/typescript/tests/routes.test.ts";

/// The vitest suite of [`TS_ROUTES`].
const ROUTES: &str = "routes";

/// Where `agci generate` dumps the route tests module after each pass, in [`DUMP_DIR`].
const ROUTES_DUMP: &str = "routes";

/// The root of the binary of [`SERVER`] whose routes the client calls.
const SERVER_MAIN: &str = "src/main.rs";

/// Where `agci generate` writes the routes each client method calls, in [`DUMP_DIR`].
const ROUTE_CHECK_DUMP: &str = "route_check.txt";

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
    let api = load(API, TargetKind::Lib, None, cfg)?;
    let (types, _) = api.external_api();
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
    errors.extend(api.lower_exported(&c, SOURCE, &mut m));
    let declarations = Emitter::new(&m, &ts_config("")).types();
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
    run(platform, &mut m, &passes, dump, &mut errors)?;

    let server_tests = load(
        SERVER,
        TargetKind::Test,
        Some(SERVER_TESTS),
        host_cfg(platform, false)?.name("test"),
    )?;
    let (client_m, lowering) = api.lower_client(&c, SOURCE);
    errors.extend(lowering);
    // The builder's lowering errors again, already reported.
    let (mut routes, _) = c.lower(&roots);
    api.lower_exported(&c, SOURCE, &mut routes);
    routes.provide(client_m.clone(), API);
    errors.extend(server_tests.lower_suite(&[(&c, SOURCE), (&api, API)], SUITE, GLUE, &mut routes));
    let passes = Passes::new()
        .with(Resolve)
        .with(Prune)
        .with(ExplicitReturns)
        .with(Inline)
        .with(AcceptElementSources)
        .with(LowerConversions)
        .with(TraitDicts)
        .with(EraseOwnership)
        .with(ThrowErrors)
        .with(InferEmptyLists)
        .with(TagResults)
        .with(EraseTraits)
        .with(EraseCasts)
        .with(EraseNewtypes)
        .with(UnescapeIdentifiers)
        .with(OmitSkippedDefaults)
        .with(RemoveDeadHelpers)
        .with(RenameShadowed);
    run(
        platform,
        &mut routes,
        &passes,
        &dump.join(ROUTES_DUMP),
        &mut errors,
    )?;
    let (client, checked_m) = client(platform, client_m, &dump.join(CLIENT_DUMP), &mut errors)?;
    let server = load(
        SERVER,
        TargetKind::Bin,
        Some(SERVER_MAIN),
        host_cfg(platform, false)?,
    )?;
    errors.extend(server.errors().iter().cloned());
    let (mut defs, _) = c.lower(&roots);
    api.lower_exported(&c, SOURCE, &mut defs);
    let (transport, lowering) = api.transport();
    errors.extend(lowering);
    let (served, lowering) = server.lower_routes(&[(&c, SOURCE), (&api, API)]);
    errors.extend(lowering);
    let (checked, lowering) = routes::check(&checked_m, &[&defs], &transport, &served);
    errors.extend(lowering);
    let ts = Emitter::new(&m, &ts_config("./types.ts")).module();
    let fs = platform.fs();
    let tests = Emitter::new(&m, &ts_config("../src/types.ts")).tests(
        TESTS_NAME,
        TS_MODULE,
        &[(DERIVED, TS_DERIVED)],
    );
    let routes = Emitter::new(&routes, &ts_config("../src/types.ts")).tests(
        ROUTES,
        TS_MODULE,
        &[(API, "../src/api.ts"), (GLUE, TS_GLUE)],
    );
    fs.write(dump.join("ts_notes.txt"), ts.notes.join("\n") + "\n")?;
    fs.write(dump.join(ROUTE_CHECK_DUMP), checked.dump())?;
    fs.write(Path::new(root).join(TS_TYPES), declarations.code)?;
    fs.write(Path::new(root).join(TS), ts.code)?;
    fs.write(Path::new(root).join(TS_TESTS), tests.code)?;
    fs.write(Path::new(root).join(TS_API), client.code)?;
    fs.write(Path::new(root).join(TS_ROUTES), routes.code)?;
    let unsupported: Vec<String> = declarations
        .unsupported
        .into_iter()
        .chain(ts.unsupported)
        .chain(client.unsupported)
        .chain(tests.unsupported)
        .chain(routes.unsupported)
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

/// `m` after `passes`, dumped to `dump` after loading and after each pass as
/// `NN_<pass>.rs.txt`.
fn run(
    platform: &impl Platform,
    m: &mut Module,
    passes: &Passes,
    dump: &Path,
    errors: &mut Vec<agsource::Error>,
) -> Result<(), CIError> {
    run_from(platform, m, passes, dump, 0, errors)
}

/// [`run`] numbering the dumps from `first`, the module after loading only
/// dumped for `0`.
fn run_from(
    platform: &impl Platform,
    m: &mut Module,
    passes: &Passes,
    dump: &Path,
    first: usize,
    errors: &mut Vec<agsource::Error>,
) -> Result<(), CIError> {
    let fs = platform.fs();
    fs.create_dir_all(dump)?;
    let mut step = first;
    let mut write = |title: &str, m: &Module| {
        let file = dump.join(format!("{step:02}_{title}.rs.txt"));
        step += 1;
        fs.write(&file, print::module(m, title))
    };
    if first == 0 {
        write("load", m)?;
    }

    let mut failed = None;
    errors.extend(passes.run(m, &mut |title: &str, m: &Module| {
        if let Err(e) = write(title, m) {
            failed.get_or_insert(e);
        }
    }));

    match failed {
        Some(e) => Err(e.into()),
        None => Ok(()),
    }
}

/// The client module `m` after its passes, dumped to `dump`, and the module
/// after [`Inline`], with the URLs and the types of its requests.
fn client(
    platform: &impl Platform,
    mut m: Module,
    dump: &Path,
    errors: &mut Vec<agsource::Error>,
) -> Result<(agsource::codegen::ts::Ts, Module), CIError> {
    let passes = Passes::new()
        .with(Resolve)
        .with(Prune)
        .with(ExplicitReturns)
        .with(Inline);
    run(platform, &mut m, &passes, dump, errors)?;
    let inlined = m.clone();
    let passes = Passes::new()
        .with(EraseOwnership)
        .with(ThrowErrors)
        .with(EraseCasts)
        .with(RemoveDeadHelpers);
    run_from(platform, &mut m, &passes, dump, 5, errors)?;
    Ok((Emitter::new(&m, &ts_config("./types.ts")).client(), inlined))
}

/// The serde types are imported from [`TS_TYPES`] at `types`, relative to the emitted file.
fn ts_config(types: &str) -> TsConfig {
    TsConfig {
        header: format!("// GENERATED by `agci generate` from {SOURCE} and {API}. DO NOT EDIT."),
        import: format!("import type * as types from {types:?};"),
        schemas: "types".into(),
    }
}
