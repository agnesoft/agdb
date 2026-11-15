use crate::CIError;
use crate::config::File;
use crate::config::Settings;
use crate::config::Toolchain;
use crate::manifest::Dep;
use crate::manifest::Manifest;
use crate::project_type::Details;
use crate::project_type::Kind;
use crate::project_type::ProjectType;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// A package the project releases to the registry of its type.
#[derive(Debug)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) dir: String,
}

#[derive(Debug)]
pub(crate) struct Project {
    pub(crate) dir: String,
    pub(crate) kind: Kind,
    /// The names of the package in the directory followed by those folded into it, none without a
    /// manifest.
    pub(crate) packages: Vec<String>,
    pub(crate) settings: Settings,
    /// Directories of the projects this project depends on.
    pub(crate) depends: BTreeSet<String>,
    /// Repository paths (files or directories) outside of the project that affect it.
    pub(crate) inputs: Vec<String>,
    pub(crate) published: Vec<Package>,
    /// Its checks and release hooks.
    pub(crate) project_type: Box<dyn ProjectType>,
}

impl Project {
    pub(crate) fn owns(&self, path: &str) -> bool {
        is_within(path, &self.dir)
    }

    pub(crate) fn needs(&self, toolchain: Toolchain) -> bool {
        self.project_type.toolchains().contains(&toolchain)
            || self.settings.add_toolchains.contains(&toolchain)
    }
}

/// A project of the manifests before its configuration.
struct Draft {
    dir: String,
    kind: Kind,
    packages: Vec<String>,
    published: Vec<Package>,
    /// Of its manifests, its own first.
    details: Vec<Details>,
}

pub(crate) fn is_within(path: &str, dir: &str) -> bool {
    path == dir
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Builds the projects from the manifests and `agci.toml`. A manifest is a project unless it is nested
/// in a project of the same type (then it is folded into the nearest one) and has no entry of
/// its own. An entry without a manifest is of `Kind::WITHOUT_MANIFEST`. Each gets its effective configuration,
/// its type resolves the checks (inherited ones only if the project can run them).
pub(crate) fn projects(
    mut manifests: Vec<Manifest>,
    file: &File,
    path_exists: impl Fn(&str) -> bool,
) -> Result<Vec<Project>, CIError> {
    let mut drafts: Vec<Draft> = Vec::new();
    let mut deps: Vec<(String, Kind, Dep)> = Vec::new();
    let mut owners: BTreeMap<String, String> = BTreeMap::new();
    let new = |dir: &str, kind: Kind, packages: Vec<String>| Draft {
        dir: dir.to_string(),
        kind,
        packages,
        published: Vec::new(),
        details: Vec::new(),
    };

    if file.projects.keys().any(|dir| dir.is_empty() || dir == ".") {
        return Err("the repository root cannot be a project".into());
    }

    manifests.sort_by(|a, b| (a.dir.len(), &a.dir).cmp(&(b.dir.len(), &b.dir)));

    for manifest in manifests {
        let kind = manifest.kind();
        let parent = drafts
            .iter_mut()
            .filter(|p| p.kind == manifest.kind() && is_within(&manifest.dir, &p.dir))
            .max_by_key(|p| p.dir.len());
        let owner = match parent {
            Some(parent) if !file.projects.contains_key(&manifest.dir) => {
                parent.packages.push(manifest.name.clone());
                parent
            }
            _ => {
                if drafts.iter().any(|p| p.dir == manifest.dir) {
                    return Err(format!("'{}' has more than one manifest", manifest.dir).into());
                }

                drafts.push(new(
                    &manifest.dir,
                    manifest.kind(),
                    vec![manifest.name.clone()],
                ));
                drafts.last_mut().expect("just pushed")
            }
        };

        if manifest.publish {
            owner.published.push(Package {
                name: manifest.name,
                dir: manifest.dir.clone(),
            });
        }

        owner.details.push(manifest.details);
        let owner = owner.dir.clone();
        deps.extend(manifest.deps.into_iter().map(|d| (owner.clone(), kind, d)));
        owners.insert(manifest.dir, owner);
    }

    for dir in file.projects.keys() {
        if drafts.iter().any(|p| p.dir == *dir) {
            continue;
        }

        if !path_exists(dir) {
            return Err(format!(
                "'{dir}' is neither the directory of a manifest nor an existing path"
            )
            .into());
        }

        owners.insert(dir.clone(), dir.clone());
        drafts.push(new(dir, Kind::WITHOUT_MANIFEST, Vec::new()));
    }

    let mut projects = Vec::new();

    for draft in drafts {
        let (config, inherited) = file.config(&draft.dir, draft.kind)?;
        let (settings, config) = config.split(&draft.dir, inherited);
        let project_type = draft
            .kind
            .project(&draft.dir, config, draft.details, &path_exists)?;
        projects.push(Project {
            dir: draft.dir,
            kind: draft.kind,
            packages: draft.packages,
            settings,
            depends: BTreeSet::new(),
            inputs: Vec::new(),
            published: draft.published,
            project_type,
        });
    }

    let names: BTreeMap<(Kind, String), String> = projects
        .iter()
        .flat_map(|p| {
            p.packages
                .iter()
                .map(|n| ((p.kind, n.clone()), p.dir.clone()))
        })
        .collect();

    for (dir, kind, dep) in deps {
        let target = match &dep {
            Dep::Dir(d) => owners.get(d),
            Dep::Name(n) => names.get(&(kind, n.clone())),
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
        match project.settings.group.as_deref() {
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

        for depend in &project.settings.dependencies {
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::config::Config;
    use crate::project_type::Hook;
    use crate::project_type::cargo::tests::crate_details;
    use crate::project_type::pnpm::tests::package_details;

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
        let name = dir.replace('/', "_");
        let details = match kind {
            Kind::Cargo => Some(crate_details(true, false, false)),
            Kind::Pnpm => Some(package_details(&name, &["build", "test"], None)),
            Kind::Shell => None,
        };
        let manifest = details.map(|details| Manifest {
            dir: dir.to_string(),
            name,
            deps,
            publish: false,
            details,
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

    /// The steps of the checks of the project in `dir` as `<dir>> <program> <args>`.
    pub(crate) fn command_lines(projects: &[Project], dir: &str) -> Vec<String> {
        let project = projects.iter().find(|p| p.dir == dir).unwrap();
        project
            .project_type
            .checks(project, projects, "/repo")
            .into_iter()
            .flat_map(|c| c.steps)
            .map(|s| format!("{}> {} {}", s.dir, s.program, s.args.join(" ")))
            .collect()
    }

    fn checks(project: &Project) -> Vec<String> {
        project.project_type.check_names()
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
    fn resolves_cross_type_depends_and_inputs() {
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
            checks(&projects[0]),
            vec!["format", "lint", "doc", "coverage"]
        );
        // the default pnpm checks that are scripts of the project
        assert_eq!(checks(&projects[1]), vec!["build", "test"]);
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
    fn manifest_does_not_fold_into_other_type() {
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
        node.as_mut().unwrap().details = package_details("a", &[], None);

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
    fn fields_the_type_does_not_have_fail() {
        for (kind, field) in [
            (Kind::Pnpm, "feature_combinations"),
            (Kind::Shell, "coverage"),
            (Kind::Cargo, "check"),
        ] {
            let mut c = config(&["test"], &[]);
            c.fields
                .insert(field.to_string(), toml::Value::Array(Vec::new()));
            let error = build(vec![manifest("a", kind, vec![], Some(c))], |_| true)
                .unwrap_err()
                .description;

            assert!(
                error.starts_with("'agci.toml' project 'a': unknown field"),
                "{error}"
            );
            assert!(error.contains(&format!("`{field}`")), "{error}");
        }
    }

    #[test]
    fn group_must_not_be_a_project_directory() {
        let mut c = config(&["coverage"], &[]);
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
        // the default checks with a script and the hooks that have one
        assert_eq!(checks(&projects[1]), vec!["build", "test"]);
        assert_eq!(
            projects[1].project_type.hook(&projects[1], Hook::Dev)[0].args,
            vec!["release_dev.sh"]
        );
        assert!(
            projects[1]
                .project_type
                .hook(&projects[1], Hook::Release)
                .is_empty()
        );
    }

    #[test]
    fn default_rust_script_check_only_where_the_script_exists() {
        let manifests = ["a", "b"].map(|dir| manifest(dir, Kind::Cargo, vec![], None).0.unwrap());
        let file = File::parse("[defaults.cargo]\nchecks = [\"lint\", \"smoke\"]").unwrap();
        let projects = projects(manifests.into(), &file, |p| p == "b/smoke.ps1").unwrap();

        assert_eq!(checks(&projects[0]), vec!["lint"]);
        assert_eq!(checks(&projects[1]), vec!["lint", "smoke"]);
        assert_eq!(
            command_lines(&projects, "b").last().unwrap(),
            "b> pwsh -NoProfile -File smoke.ps1"
        );
    }

    #[test]
    fn doc_only_for_crates_with_a_library() {
        let bin = |config: Option<Config>| {
            let (mut manifest, config) = manifest("bin", Kind::Cargo, vec![], config);
            manifest.as_mut().unwrap().details = crate_details(false, false, false);
            (manifest, config)
        };
        let projects = build(vec![bin(None)], |_| false).unwrap();

        assert_eq!(checks(&projects[0]), vec!["format", "lint", "coverage"]);
        assert_eq!(
            error(vec![bin(Some(config(&["doc"], &[])))]),
            "'bin' has the 'doc' check but no library"
        );
    }

    #[test]
    fn examples_and_bench_only_for_crates_with_such_targets() {
        let crate_with = |dir: &str, examples: bool, benches: bool| {
            let (mut manifest, config) = manifest(dir, Kind::Cargo, vec![], None);
            let m = manifest.as_mut().unwrap();
            m.details = crate_details(true, examples, benches);
            (manifest, config)
        };
        let projects = build(
            vec![
                crate_with("both", true, true),
                crate_with("examples", true, false),
                crate_with("plain", false, false),
            ],
            |_| false,
        )
        .unwrap();

        assert_eq!(
            checks(&projects[0]),
            vec!["format", "lint", "doc", "examples", "bench", "coverage"]
        );
        assert_eq!(
            checks(&projects[1]),
            vec!["format", "lint", "doc", "examples", "coverage"]
        );
        assert_eq!(
            checks(&projects[2]),
            vec!["format", "lint", "doc", "coverage"]
        );

        for (check, error_message) in [
            (
                "examples",
                "'plain' has the 'examples' check but no example targets",
            ),
            (
                "bench",
                "'plain' has the 'bench' check but no bench targets",
            ),
        ] {
            assert_eq!(
                error(vec![manifest(
                    "plain",
                    Kind::Cargo,
                    vec![],
                    Some(config(&[check], &[]))
                )]),
                error_message
            );
        }
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

        // Cargo built-ins always run before the scripts
        assert_eq!(checks(&projects[0]), vec!["lint", "coverage", "run"]);
        assert_eq!(
            checks(&projects[1]),
            vec!["format", "lint", "doc", "coverage", "run"]
        );
        assert_eq!(projects[2].kind, Kind::Shell);
        assert_eq!(checks(&projects[2]), vec!["run"]);
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
        let package = |dir: &str, publish: bool, scripts: &[&str]| {
            let name = dir.replace('/', "_");
            Manifest {
                dir: dir.to_string(),
                details: package_details(&name, scripts, publish.then_some("1.0.0")),
                name,
                deps: Vec::new(),
                publish,
            }
        };
        let manifests = vec![
            package("ui", true, &["build", "test"]),
            package("ui/lib", true, &["test"]),
            package("ui/tool", false, &["build"]),
        ];
        let projects = projects(manifests, &File::default(), |_| false).unwrap();
        let published: Vec<(&str, &str, String, usize)> = projects[0]
            .published
            .iter()
            .map(|p| {
                let publish = projects[0].project_type.publish(p, "9.9.9").unwrap();
                (
                    p.name.as_str(),
                    p.dir.as_str(),
                    publish.version,
                    publish.steps.len(),
                )
            })
            .collect();

        // built before it is published if it has a build script, in its own version
        assert_eq!(
            published,
            vec![
                ("ui", "ui", "1.0.0".to_string(), 2),
                ("ui_lib", "ui/lib", "1.0.0".to_string(), 1)
            ]
        );
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
}
