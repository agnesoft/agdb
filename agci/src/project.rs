use crate::CIError;
use crate::config::CARGO_CHECKS;
use crate::config::Config;
use crate::config::File;
use crate::config::Toolchain;
use crate::manifest::Dep;
use crate::manifest::Kind;
use crate::manifest::Manifest;
use crate::release::HOOKS;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// A package the project releases to its registry (crates.io, npm).
#[derive(Debug)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) dir: String,
    /// package.json only, crates have the release version.
    pub(crate) version: Option<String>,
    /// Has a `build` script, built before it is published.
    pub(crate) build: bool,
}

#[derive(Debug)]
pub(crate) struct Project {
    pub(crate) dir: String,
    pub(crate) kind: Kind,
    /// Cargo: the crate. pnpm: the package in the directory followed by the packages folded into it.
    /// Shell: none.
    pub(crate) packages: Vec<String>,
    pub(crate) config: Config,
    /// Directories of the projects this project depends on.
    pub(crate) depends: BTreeSet<String>,
    /// Repository paths (files or directories) outside of the project that affect it.
    pub(crate) inputs: Vec<String>,
    /// pnpm: the package.json scripts of its packages. Cargo and Shell: its `<name>.sh`/`<name>.ps1`
    /// files of the checks and release hooks.
    pub(crate) scripts: BTreeSet<String>,
    pub(crate) published: Vec<Package>,
    /// Cargo: the crate has a library target.
    pub(crate) library: bool,
}

impl Project {
    pub(crate) fn owns(&self, path: &str) -> bool {
        is_within(path, &self.dir)
    }

    pub(crate) fn needs(&self, toolchain: Toolchain) -> bool {
        let own = match self.kind {
            Kind::Pnpm => Some(Toolchain::Node),
            Kind::Cargo => Some(Toolchain::Rust),
            Kind::Shell => None,
        };
        own == Some(toolchain) || self.config.add_toolchains.contains(&toolchain)
    }

    /// pnpm: a script of any of its packages. Cargo and Shell: a script file of the name.
    pub(crate) fn has_script(&self, name: &str) -> bool {
        match self.kind {
            Kind::Pnpm => self.scripts.contains(name),
            Kind::Cargo | Kind::Shell => self.shell_script(name).is_some(),
        }
    }

    /// The name the check runs as: Cargo `coverage` without coverage is `test`.
    pub(crate) fn check_name<'a>(&self, check: &'a str) -> &'a str {
        if self.kind == Kind::Cargo && check == "coverage" && !self.config.coverage() {
            "test"
        } else {
            check
        }
    }

    /// Runs the tests under `cargo llvm-cov`.
    pub(crate) fn measures_coverage(&self) -> bool {
        self.kind == Kind::Cargo && self.config.has_check("coverage") && self.config.coverage()
    }

    /// Cargo: a built-in check (`doc` needs a library) or a script. pnpm and Shell: a script.
    pub(crate) fn can_run(&self, check: &str) -> bool {
        match (self.kind, check) {
            (Kind::Cargo, "doc") => self.library,
            (Kind::Cargo, _) if CARGO_CHECKS.contains(&check) => true,
            _ => self.has_script(check),
        }
    }

    /// The file of the script `name` (`:` is `_` in file names), the PowerShell one is
    /// preferred on Windows, the bash one elsewhere.
    pub(crate) fn shell_script(&self, name: &str) -> Option<&str> {
        let [sh, ps1] = script_files(name);
        let order = if cfg!(windows) { [ps1, sh] } else { [sh, ps1] };
        order
            .into_iter()
            .find_map(|file| self.scripts.get(&file).map(String::as_str))
    }
}

/// The script of Cargo and Shell projects that runs as their last check if it exists.
const RUN: &str = "run";

fn script_files(name: &str) -> [String; 2] {
    let base = name.replace(':', "_");
    [format!("{base}.sh"), format!("{base}.ps1")]
}

pub(crate) fn is_within(path: &str, dir: &str) -> bool {
    path == dir
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Builds the projects from the manifests and `agci.toml`. A manifest is a project unless it is nested
/// in a project of the same language (then it is folded into the nearest one) and has no entry of
/// its own. An entry without a manifest is a Shell project. Each gets its effective configuration,
/// inherited pnpm and Shell checks only if the project has the scripts.
pub(crate) fn projects(
    mut manifests: Vec<Manifest>,
    file: &File,
    path_exists: impl Fn(&str) -> bool,
) -> Result<Vec<Project>, CIError> {
    let mut projects: Vec<Project> = Vec::new();
    let mut deps: Vec<(String, Dep)> = Vec::new();
    let mut owners: BTreeMap<String, String> = BTreeMap::new();
    let new = |dir: &str, kind: Kind, packages: Vec<String>| Project {
        dir: dir.to_string(),
        kind,
        packages,
        config: Config::default(),
        depends: BTreeSet::new(),
        inputs: Vec::new(),
        scripts: BTreeSet::new(),
        published: Vec::new(),
        library: false,
    };

    if file.projects.keys().any(|dir| dir.is_empty() || dir == ".") {
        return Err("the repository root cannot be a project".into());
    }

    manifests.sort_by(|a, b| (a.dir.len(), &a.dir).cmp(&(b.dir.len(), &b.dir)));

    for manifest in manifests {
        let parent = projects
            .iter_mut()
            .filter(|p| p.kind == manifest.kind && is_within(&manifest.dir, &p.dir))
            .max_by_key(|p| p.dir.len());
        let owner = match parent {
            Some(parent) if !file.projects.contains_key(&manifest.dir) => {
                parent.packages.push(manifest.name.clone());
                parent
            }
            _ => {
                if projects.iter().any(|p| p.dir == manifest.dir) {
                    return Err(format!("'{}' has more than one manifest", manifest.dir).into());
                }

                let mut project = new(&manifest.dir, manifest.kind, vec![manifest.name.clone()]);
                project.library = manifest.library;
                projects.push(project);
                projects.last_mut().expect("just pushed")
            }
        };

        if manifest.publish {
            owner.published.push(Package {
                name: manifest.name,
                dir: manifest.dir.clone(),
                version: manifest.version,
                build: manifest.scripts.contains("build"),
            });
        }

        owner.scripts.extend(manifest.scripts);
        let owner = owner.dir.clone();
        deps.extend(manifest.deps.into_iter().map(|d| (owner.clone(), d)));
        owners.insert(manifest.dir, owner);
    }

    for dir in file.projects.keys() {
        if projects.iter().any(|p| p.dir == *dir) {
            continue;
        }

        if !path_exists(dir) {
            return Err(format!(
                "'{dir}' is neither the directory of a manifest nor an existing path"
            )
            .into());
        }

        owners.insert(dir.clone(), dir.clone());
        projects.push(new(dir, Kind::Shell, Vec::new()));
    }

    for project in &mut projects {
        let (mut config, inherited) = file.config(&project.dir, project.kind)?;

        if project.kind != Kind::Pnpm {
            for name in config
                .checks
                .iter()
                .map(String::as_str)
                .chain(HOOKS)
                .chain([RUN])
            {
                for script in script_files(name) {
                    if path_exists(&format!("{}/{script}", project.dir)) {
                        project.scripts.insert(script);
                    }
                }
            }
        }

        if inherited {
            config.checks.retain(|check| project.can_run(check));
        }

        if project.kind != Kind::Pnpm && project.has_script(RUN) && !config.has_check(RUN) {
            config.checks.push(RUN.to_string());
        }

        project.config = config;
    }

    let names: BTreeMap<String, String> = projects
        .iter()
        .filter(|p| p.kind == Kind::Pnpm)
        .flat_map(|p| p.packages.iter().map(|n| (n.clone(), p.dir.clone())))
        .collect();

    for (dir, dep) in deps {
        let target = match &dep {
            Dep::Dir(d) => owners.get(d),
            Dep::Name(n) => names.get(n),
        }
        .ok_or(format!(
            "'{dir}' depends on {dep:?} which is not part of any project"
        ))?;

        if *target != dir {
            let target = target.clone();
            project_mut(&mut projects, &dir).depends.insert(target);
        }
    }

    let dirs: BTreeSet<String> = projects.iter().map(|p| p.dir.clone()).collect();

    for project in &mut projects {
        match project.config.group.as_deref() {
            Some("") => return Err(format!("'{}' has an empty group", project.dir).into()),
            Some(group) if dirs.contains(group) => {
                return Err(format!(
                    "'{}' group '{group}' is the directory of a project",
                    project.dir
                )
                .into());
            }
            _ => {}
        }

        for depend in &project.config.dependencies {
            if dirs.contains(depend) {
                project.depends.insert(depend.clone());
            } else if path_exists(depend) {
                project.inputs.push(depend.clone());
            } else {
                return Err(format!(
                    "'{}' depends on '{depend}' which is neither a project nor an existing path",
                    project.dir
                )
                .into());
            }
        }

        project.depends.remove(&project.dir);
        validate(project)?;
    }

    projects.sort_by(|a, b| a.dir.cmp(&b.dir));
    Ok(projects)
}

fn project_mut<'a>(projects: &'a mut [Project], dir: &str) -> &'a mut Project {
    projects
        .iter_mut()
        .find(|p| p.dir == dir)
        .expect("dependency source is a project")
}

fn validate(project: &Project) -> Result<(), CIError> {
    let dir = &project.dir;
    let config = &project.config;

    if config.checks.is_empty() {
        return Err(format!("'{dir}' has no checks, set them in agci.toml").into());
    }

    match project.kind {
        Kind::Cargo => {
            if config.has_check("doc") && !project.library {
                return Err(format!("'{dir}' has the 'doc' check but no library").into());
            }

            if let Some(check) = config.checks.iter().find(|c| !project.can_run(c)) {
                let [sh, ps1] = script_files(check);
                return Err(format!(
                    "'{dir}' check '{check}' is neither one of {CARGO_CHECKS:?} nor has '{sh}' or '{ps1}'"
                )
                .into());
            }

            if !config.feature_combinations.is_empty() && !config.checks.iter().any(|c| c == "lint")
            {
                return Err(
                    format!("'{dir}' sets feature_combinations without the 'lint' check").into(),
                );
            }
        }
        Kind::Pnpm | Kind::Shell => {
            if config.has_rust_only_fields() {
                return Err(format!(
                    "'{dir}' sets cargo only fields (coverage, feature_combinations)"
                )
                .into());
            }

            if let Some(check) = config.checks.iter().find(|c| !project.has_script(c)) {
                return Err(match project.kind {
                    Kind::Pnpm => {
                        format!("'{dir}' check '{check}' is not a script of any of its packages")
                    }
                    _ => {
                        let [sh, ps1] = script_files(check);
                        format!("'{dir}' check '{check}' has neither '{sh}' nor '{ps1}'")
                    }
                }
                .into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn config(checks: &[&str], depends: &[&str]) -> Config {
        Config {
            checks: checks.iter().map(|c| c.to_string()).collect(),
            dependencies: depends.iter().map(|d| d.to_string()).collect(),
            ..Default::default()
        }
    }

    /// A manifest (none for Shell) and its project configuration (none if it is folded).
    pub(crate) type Entry = (Option<Manifest>, Option<(String, Config)>);

    pub(crate) fn manifest(dir: &str, kind: Kind, deps: Vec<Dep>, config: Option<Config>) -> Entry {
        let manifest = (kind != Kind::Shell).then(|| Manifest {
            dir: dir.to_string(),
            kind,
            name: dir.replace('/', "_"),
            deps,
            scripts: ["build", "test"].iter().map(|s| s.to_string()).collect(),
            publish: false,
            version: None,
            library: true,
        });
        (manifest, config.map(|c| (dir.to_string(), c)))
    }

    pub(crate) fn build(
        entries: Vec<Entry>,
        path_exists: impl Fn(&str) -> bool,
    ) -> Result<Vec<Project>, CIError> {
        let (manifests, configs): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
        projects(
            manifests.into_iter().flatten().collect(),
            &file(configs.into_iter().flatten()),
            path_exists,
        )
    }

    /// `agci.toml` with the fields of the configurations that are set.
    pub(crate) fn file(configs: impl IntoIterator<Item = (String, Config)>) -> File {
        let table = |config: Config| {
            let serde_json::Value::Object(fields) = serde_json::to_value(config).unwrap() else {
                unreachable!()
            };
            let fields = fields.into_iter().filter(|(_, v)| !v.is_null()).collect();
            serde_json::from_value(serde_json::Value::Object(fields)).unwrap()
        };
        File {
            projects: configs
                .into_iter()
                .map(|(dir, config)| (dir, table(config)))
                .collect(),
            ..Default::default()
        }
    }

    fn dir(d: &str) -> Dep {
        Dep::Dir(d.to_string())
    }

    fn name(n: &str) -> Dep {
        Dep::Name(n.to_string())
    }

    fn error(entries: Vec<Entry>) -> String {
        build(entries, |_| false).unwrap_err().description
    }

    #[test]
    fn folds_nested_manifests_into_enclosing_project() {
        let projects = build(
            vec![
                manifest(
                    "ui",
                    Kind::Pnpm,
                    vec![name("ui_libs_b")],
                    Some(config(&["build"], &[])),
                ),
                manifest("ui/libs/a", Kind::Pnpm, vec![name("client")], None),
                manifest("ui/libs/b", Kind::Pnpm, vec![], None),
                manifest("client", Kind::Pnpm, vec![], Some(config(&["test"], &[]))),
            ],
            |_| false,
        )
        .unwrap();

        assert_eq!(projects.len(), 2);
        assert_eq!(projects[1].dir, "ui");
        assert_eq!(projects[1].packages, vec!["ui", "ui_libs_a", "ui_libs_b"]);
        assert_eq!(projects[1].depends, BTreeSet::from(["client".to_string()]));
    }

    #[test]
    fn resolves_cross_language_depends_and_inputs() {
        let projects = build(
            vec![
                manifest(
                    "server",
                    Kind::Cargo,
                    vec![dir("db")],
                    Some(config(&["coverage"], &[])),
                ),
                manifest("db", Kind::Cargo, vec![], Some(config(&["coverage"], &[]))),
                manifest(
                    "client",
                    Kind::Pnpm,
                    vec![],
                    Some(config(&["test"], &["server", "composer.json"])),
                ),
            ],
            |p| p == "composer.json",
        )
        .unwrap();

        assert_eq!(projects[0].dir, "client");
        assert_eq!(projects[0].depends, BTreeSet::from(["server".to_string()]));
        assert_eq!(projects[0].inputs, vec!["composer.json"]);
        assert_eq!(projects[2].depends, BTreeSet::from(["db".to_string()]));
    }

    #[test]
    fn top_level_manifests_are_detected_with_the_defaults() {
        let projects = build(
            vec![
                manifest("tools/x", Kind::Cargo, vec![], None),
                manifest("web", Kind::Pnpm, vec![], None),
            ],
            |_| false,
        )
        .unwrap();

        assert_eq!(projects[0].dir, "tools/x");
        assert_eq!(
            projects[0].config.checks,
            vec!["format", "lint", "doc", "coverage"]
        );
        // the default pnpm checks that are scripts of the project
        assert_eq!(projects[1].config.checks, vec!["build", "test"]);
    }

    #[test]
    fn nested_manifest_with_an_entry_is_its_own_project() {
        let projects = build(
            vec![
                manifest("ui", Kind::Pnpm, vec![], None),
                manifest("ui/tool", Kind::Pnpm, vec![], Some(config(&["test"], &[]))),
                manifest("ui/tool/lib", Kind::Pnpm, vec![], None),
                manifest("ui/lib", Kind::Pnpm, vec![], None),
            ],
            |_| false,
        )
        .unwrap();

        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].packages, vec!["ui", "ui_lib"]);
        assert_eq!(projects[1].packages, vec!["ui_tool", "ui_tool_lib"]);
    }

    #[test]
    fn manifest_does_not_fold_into_other_language() {
        let projects = build(
            vec![
                manifest("api", Kind::Cargo, vec![], Some(config(&["coverage"], &[]))),
                manifest("api/ts", Kind::Pnpm, vec![], None),
            ],
            |_| false,
        )
        .unwrap();

        assert_eq!(projects[1].dir, "api/ts");
        assert_eq!(projects[1].kind, Kind::Pnpm);
    }

    #[test]
    fn node_without_any_default_script_fails() {
        let (mut node, _) = manifest("a", Kind::Pnpm, vec![], None);
        node.as_mut().unwrap().scripts.clear();

        assert_eq!(
            error(vec![(node, None)]),
            "'a' has no checks, set them in agci.toml"
        );
    }

    #[test]
    fn unknown_depends_fails() {
        assert_eq!(
            error(vec![manifest(
                "a",
                Kind::Cargo,
                vec![],
                Some(config(&["coverage"], &["nope"]))
            )]),
            "'a' depends on 'nope' which is neither a project nor an existing path"
        );
    }

    #[test]
    fn dependency_outside_projects_fails() {
        assert_eq!(
            error(vec![manifest(
                "a",
                Kind::Pnpm,
                vec![name("ghost")],
                Some(config(&["test"], &[]))
            )]),
            "'a' depends on Name(\"ghost\") which is not part of any project"
        );
    }

    #[test]
    fn root_project_fails() {
        assert_eq!(
            error(vec![manifest(
                "",
                Kind::Pnpm,
                vec![],
                Some(config(&["test"], &[]))
            )]),
            "the repository root cannot be a project"
        );
    }

    #[test]
    fn no_checks_fails() {
        assert_eq!(
            error(vec![manifest(
                "a",
                Kind::Cargo,
                vec![],
                Some(config(&[], &[]))
            )]),
            "'a' has no checks, set them in agci.toml"
        );
    }

    #[test]
    fn unknown_rust_check_fails() {
        assert!(
            error(vec![manifest(
                "a",
                Kind::Cargo,
                vec![],
                Some(config(&["fmt"], &[]))
            )])
            .starts_with("'a' check 'fmt' is neither one of [\"format\"")
        );
    }

    #[test]
    fn node_check_must_be_a_script() {
        assert_eq!(
            error(vec![manifest(
                "a",
                Kind::Pnpm,
                vec![],
                Some(config(&["lint:check"], &[]))
            )]),
            "'a' check 'lint:check' is not a script of any of its packages"
        );
    }

    #[test]
    fn node_rejects_rust_only_fields() {
        let mut c = config(&["test"], &[]);
        c.feature_combinations = vec![String::new()];

        assert!(
            error(vec![manifest("a", Kind::Pnpm, vec![], Some(c))])
                .starts_with("'a' sets cargo only fields")
        );
    }

    #[test]
    fn group_must_not_be_a_project_directory() {
        let mut c = config(&["test"], &[]);
        c.group = Some("b".to_string());

        assert_eq!(
            error(vec![
                manifest("a", Kind::Cargo, vec![], Some(c)),
                manifest("b", Kind::Cargo, vec![], Some(config(&["coverage"], &[]))),
            ]),
            "'a' group 'b' is the directory of a project"
        );
    }

    #[test]
    fn owns_matches_whole_path_segments() {
        let project = build(
            vec![manifest(
                "agdb",
                Kind::Cargo,
                vec![],
                Some(config(&["coverage"], &[])),
            )],
            |_| false,
        )
        .unwrap()
        .remove(0);

        assert!(project.owns("agdb"));
        assert!(project.owns("agdb/src/lib.rs"));
        assert!(!project.owns("agdb_server/src/main.rs"));
    }

    #[test]
    fn directory_without_manifest_is_a_shell_project() {
        let manifests = [
            manifest("server", Kind::Cargo, vec![], None),
            manifest("ui", Kind::Pnpm, vec![], None),
        ];
        let file = File::parse("[projects.\"server/image\"]\ndependencies = [\"server\", \"ui\"]")
            .unwrap();
        let projects = projects(
            manifests.into_iter().filter_map(|(m, _)| m).collect(),
            &file,
            |p| {
                [
                    "server/image",
                    "server/image/build.ps1",
                    "server/image/test.sh",
                    "server/image/release_dev.sh",
                    "server/image/other.sh",
                ]
                .contains(&p)
            },
        )
        .unwrap();

        assert_eq!(projects[1].dir, "server/image");
        assert_eq!(projects[1].kind, Kind::Shell);
        assert!(projects[1].packages.is_empty());
        assert_eq!(
            projects[1].depends,
            BTreeSet::from(["server".to_string(), "ui".to_string()])
        );
        assert!(!projects[1].needs(Toolchain::Rust));
        assert!(!projects[1].needs(Toolchain::Node));
        // the default checks with a script, the scripts of the checks and hooks only
        assert_eq!(projects[1].config.checks, vec!["build", "test"]);
        assert_eq!(
            projects[1].scripts,
            BTreeSet::from(["build.ps1", "release_dev.sh", "test.sh"].map(str::to_string))
        );
        assert!(projects[1].has_script("release:dev"));
        assert!(!projects[1].has_script("release"));
    }

    #[test]
    fn default_rust_script_check_only_where_the_script_exists() {
        let manifests = ["a", "b"].map(|dir| manifest(dir, Kind::Cargo, vec![], None).0.unwrap());
        let file = File::parse("[defaults.cargo]\nchecks = [\"lint\", \"smoke\"]").unwrap();
        let projects = projects(manifests.into(), &file, |p| p == "b/smoke.ps1").unwrap();

        assert_eq!(projects[0].config.checks, vec!["lint"]);
        assert_eq!(projects[1].config.checks, vec!["lint", "smoke"]);
        assert_eq!(projects[1].shell_script("smoke"), Some("smoke.ps1"));
    }

    #[test]
    fn doc_only_for_crates_with_a_library() {
        let bin = |config: Option<Config>| {
            let (mut manifest, config) = manifest("bin", Kind::Cargo, vec![], config);
            manifest.as_mut().unwrap().library = false;
            (manifest, config)
        };
        let projects = build(vec![bin(None)], |_| false).unwrap();

        assert_eq!(
            projects[0].config.checks,
            vec!["format", "lint", "coverage"]
        );
        assert_eq!(
            error(vec![bin(Some(config(&["doc"], &[])))]),
            "'bin' has the 'doc' check but no library"
        );
    }

    #[test]
    fn run_script_is_the_last_check() {
        let manifests = ["a", "b"].map(|dir| manifest(dir, Kind::Cargo, vec![], None).0.unwrap());
        let file = File::parse(
            "[projects.a]\nchecks = [\"lint\", \"run\", \"coverage\"]\n[projects.c]\nchecks = []",
        )
        .unwrap();
        let projects = projects(manifests.into(), &file, |p| {
            ["a/run.sh", "b/run.ps1", "c", "c/run.sh"].contains(&p)
        })
        .unwrap();

        assert_eq!(projects[0].config.checks, vec!["lint", "run", "coverage"]);
        assert_eq!(
            projects[1].config.checks,
            vec!["format", "lint", "doc", "coverage", "run"]
        );
        assert_eq!(projects[2].kind, Kind::Shell);
        assert_eq!(projects[2].config.checks, vec!["run"]);
    }

    #[test]
    fn shell_check_without_a_script_fails() {
        assert_eq!(
            build(
                vec![manifest(
                    "a",
                    Kind::Shell,
                    vec![],
                    Some(config(&["test:image"], &[]))
                )],
                |p| p == "a" || p == "a/test.sh"
            )
            .unwrap_err()
            .description,
            "'a' check 'test:image' has neither 'test_image.sh' nor 'test_image.ps1'"
        );
    }

    #[test]
    fn published_packages_of_folded_manifests() {
        let package = |dir: &str, publish: bool, scripts: &[&str]| Manifest {
            dir: dir.to_string(),
            kind: Kind::Pnpm,
            name: dir.replace('/', "_"),
            deps: Vec::new(),
            scripts: scripts.iter().map(|s| s.to_string()).collect(),
            publish,
            version: publish.then(|| "1.0.0".to_string()),
            library: false,
        };
        let manifests = vec![
            package("ui", true, &["build", "test"]),
            package("ui/lib", true, &["test"]),
            package("ui/tool", false, &["build"]),
        ];
        let projects = projects(manifests, &File::default(), |_| false).unwrap();
        let published: Vec<(&str, &str, bool)> = projects[0]
            .published
            .iter()
            .map(|p| (p.name.as_str(), p.dir.as_str(), p.build))
            .collect();

        assert_eq!(
            published,
            vec![("ui", "ui", true), ("ui_lib", "ui/lib", false)]
        );
        assert_eq!(projects[0].published[0].version.as_deref(), Some("1.0.0"));
    }
    #[test]
    fn project_without_manifest_or_directory_fails() {
        assert_eq!(
            error(vec![manifest(
                "gone",
                Kind::Shell,
                vec![],
                Some(config(&["test"], &[]))
            )]),
            "'gone' is neither the directory of a manifest nor an existing path"
        );
    }

    #[test]
    fn directory_with_two_manifests_fails() {
        let (rust, config) = manifest("a", Kind::Cargo, vec![], Some(config(&["coverage"], &[])));
        let (node, _) = manifest("a", Kind::Pnpm, vec![], None);

        assert_eq!(
            error(vec![(rust, config), (node, None)]),
            "'a' has more than one manifest"
        );
    }

    #[test]
    fn shell_rejects_rust_only_fields() {
        let mut c = config(&["test"], &[]);
        c.coverage = Some(false);

        assert!(
            build(vec![manifest("a", Kind::Shell, vec![], Some(c))], |_| true)
                .unwrap_err()
                .description
                .starts_with("'a' sets cargo only fields")
        );
    }
}
