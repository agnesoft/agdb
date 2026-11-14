use crate::CIError;
use crate::config::Config;
use crate::config::RUST_CHECKS;
use crate::config::RUST_RELEASE;
use crate::config::Toolchain;
use crate::manifest::Dep;
use crate::manifest::Lang;
use crate::manifest::Manifest;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug)]
pub(crate) struct Project {
    pub(crate) dir: String,
    pub(crate) lang: Lang,
    /// Rust: the crate. Node: the package with the metadata followed by the packages folded into it.
    pub(crate) packages: Vec<String>,
    pub(crate) config: Config,
    /// Directories of the projects this project depends on.
    pub(crate) depends: BTreeSet<String>,
    /// Repository paths (files or directories) outside of the project that affect it.
    pub(crate) inputs: Vec<String>,
}

impl Project {
    pub(crate) fn owns(&self, path: &str) -> bool {
        is_within(path, &self.dir)
    }

    pub(crate) fn needs(&self, toolchain: Toolchain) -> bool {
        let own = match self.lang {
            Lang::Node => Toolchain::Node,
            Lang::Rust => Toolchain::Rust,
        };
        own == toolchain || self.config.toolchains.contains(&toolchain)
    }
}

pub(crate) fn is_within(path: &str, dir: &str) -> bool {
    path == dir
        || path
            .strip_prefix(dir)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Builds the projects from the manifests. A manifest with the `agci` metadata is a project.
/// A manifest without it is folded into the nearest enclosing project of the same language.
pub(crate) fn projects(
    manifests: Vec<Manifest>,
    path_exists: impl Fn(&str) -> bool,
) -> Result<Vec<Project>, CIError> {
    let mut projects: Vec<Project> = Vec::new();
    let mut scripts: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut deps: Vec<(String, Dep)> = Vec::new();
    let mut folded = Vec::new();

    for manifest in manifests {
        let Some(config) = manifest.config else {
            folded.push(Manifest {
                config: None,
                ..manifest
            });
            continue;
        };

        if manifest.dir.is_empty() {
            return Err("the repository root cannot be a project".into());
        }

        if projects.iter().any(|p| p.dir == manifest.dir) {
            return Err(format!(
                "'{}' has more than one manifest with agci metadata",
                manifest.dir
            )
            .into());
        }

        scripts.insert(manifest.dir.clone(), manifest.scripts);
        deps.extend(manifest.deps.into_iter().map(|d| (manifest.dir.clone(), d)));
        projects.push(Project {
            dir: manifest.dir,
            lang: manifest.lang,
            packages: vec![manifest.name],
            config,
            depends: BTreeSet::new(),
            inputs: Vec::new(),
        });
    }

    let mut owners: BTreeMap<String, String> = projects
        .iter()
        .map(|p| (p.dir.clone(), p.dir.clone()))
        .collect();

    for manifest in folded {
        let parent = projects
            .iter_mut()
            .filter(|p| p.lang == manifest.lang && is_within(&manifest.dir, &p.dir))
            .max_by_key(|p| p.dir.len())
            .ok_or(format!(
                "'{}' ({}) has no agci metadata and no enclosing {:?} project",
                manifest.dir, manifest.name, manifest.lang
            ))?;
        parent.packages.push(manifest.name);
        scripts
            .entry(parent.dir.clone())
            .or_default()
            .extend(manifest.scripts);
        deps.extend(manifest.deps.into_iter().map(|d| (parent.dir.clone(), d)));
        owners.insert(manifest.dir, parent.dir.clone());
    }

    let names: BTreeMap<String, String> = projects
        .iter()
        .filter(|p| p.lang == Lang::Node)
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

        for depend in &project.config.depends {
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
        validate(project, &scripts[&project.dir])?;
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

fn validate(project: &Project, scripts: &BTreeSet<String>) -> Result<(), CIError> {
    let dir = &project.dir;
    let config = &project.config;

    if config.checks.is_empty() {
        return Err(format!("'{dir}' declares no checks").into());
    }

    match project.lang {
        Lang::Rust => {
            for check in &config.checks {
                if !RUST_CHECKS.contains(&check.as_str()) {
                    return Err(format!(
                        "'{dir}' has unknown check '{check}', expected one of {RUST_CHECKS:?}"
                    )
                    .into());
                }
            }

            if config.coverage.is_some_and(|c| c > 100) {
                return Err(format!("'{dir}' coverage must be at most 100").into());
            }

            if config.coverage.is_some() && !config.checks.iter().any(|c| c == "test") {
                return Err(format!("'{dir}' sets coverage without the 'test' check").into());
            }

            if !config.feature_combinations.is_empty() && !config.checks.iter().any(|c| c == "lint")
            {
                return Err(
                    format!("'{dir}' sets feature_combinations without the 'lint' check").into(),
                );
            }

            if let Some(target) = config
                .release
                .iter()
                .find(|t| !RUST_RELEASE.contains(&t.as_str()))
            {
                return Err(format!(
                    "'{dir}' has unknown release target '{target}', expected one of {RUST_RELEASE:?}"
                )
                .into());
            }

            if !config.release_dev.is_empty() {
                return Err(format!("'{dir}' sets release_dev which is Node only").into());
            }
        }
        Lang::Node => {
            if config.has_rust_only_fields() {
                return Err(format!("'{dir}' sets Rust only fields (coverage, features, feature_combinations, run), use package.json scripts instead").into());
            }

            for (field, script) in config
                .checks
                .iter()
                .map(|s| ("check", s))
                .chain(config.release.iter().map(|s| ("release", s)))
                .chain(config.release_dev.iter().map(|s| ("release_dev", s)))
            {
                if !scripts.contains(script) {
                    return Err(format!(
                        "'{dir}' {field} '{script}' is not a script of any of its packages"
                    )
                    .into());
                }
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
            depends: depends.iter().map(|d| d.to_string()).collect(),
            ..Default::default()
        }
    }

    pub(crate) fn manifest(
        dir: &str,
        lang: Lang,
        deps: Vec<Dep>,
        config: Option<Config>,
    ) -> Manifest {
        Manifest {
            dir: dir.to_string(),
            lang,
            name: dir.replace('/', "_"),
            deps,
            scripts: ["build", "test"].iter().map(|s| s.to_string()).collect(),
            config,
        }
    }

    fn dir(d: &str) -> Dep {
        Dep::Dir(d.to_string())
    }

    fn name(n: &str) -> Dep {
        Dep::Name(n.to_string())
    }

    fn error(manifests: Vec<Manifest>) -> String {
        projects(manifests, |_| false).unwrap_err().description
    }

    #[test]
    fn folds_nested_manifests_into_enclosing_project() {
        let projects = projects(
            vec![
                manifest(
                    "ui",
                    Lang::Node,
                    vec![name("ui_libs_b")],
                    Some(config(&["build"], &[])),
                ),
                manifest("ui/libs/a", Lang::Node, vec![name("client")], None),
                manifest("ui/libs/b", Lang::Node, vec![], None),
                manifest("client", Lang::Node, vec![], Some(config(&["test"], &[]))),
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
        let projects = projects(
            vec![
                manifest(
                    "server",
                    Lang::Rust,
                    vec![dir("db")],
                    Some(config(&["test"], &[])),
                ),
                manifest("db", Lang::Rust, vec![], Some(config(&["test"], &[]))),
                manifest(
                    "client",
                    Lang::Node,
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
    fn unattributed_manifest_fails() {
        assert_eq!(
            error(vec![manifest("tools/x", Lang::Rust, vec![], None)]),
            "'tools/x' (tools_x) has no agci metadata and no enclosing Rust project"
        );
    }

    #[test]
    fn manifest_does_not_fold_into_other_language() {
        assert_eq!(
            error(vec![
                manifest("api", Lang::Rust, vec![], Some(config(&["test"], &[]))),
                manifest("api/ts", Lang::Node, vec![], None),
            ]),
            "'api/ts' (api_ts) has no agci metadata and no enclosing Node project"
        );
    }

    #[test]
    fn unknown_depends_fails() {
        assert_eq!(
            error(vec![manifest(
                "a",
                Lang::Rust,
                vec![],
                Some(config(&["test"], &["nope"]))
            )]),
            "'a' depends on 'nope' which is neither a project nor an existing path"
        );
    }

    #[test]
    fn dependency_outside_projects_fails() {
        assert_eq!(
            error(vec![manifest(
                "a",
                Lang::Node,
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
                Lang::Node,
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
                Lang::Rust,
                vec![],
                Some(config(&[], &[]))
            )]),
            "'a' declares no checks"
        );
    }

    #[test]
    fn unknown_rust_check_fails() {
        assert!(
            error(vec![manifest(
                "a",
                Lang::Rust,
                vec![],
                Some(config(&["fmt"], &[]))
            )])
            .starts_with("'a' has unknown check 'fmt'")
        );
    }

    #[test]
    fn coverage_requires_test() {
        let mut c = config(&["lint"], &[]);
        c.coverage = Some(100);

        assert_eq!(
            error(vec![manifest("a", Lang::Rust, vec![], Some(c))]),
            "'a' sets coverage without the 'test' check"
        );
    }

    #[test]
    fn node_check_must_be_a_script() {
        assert_eq!(
            error(vec![manifest(
                "a",
                Lang::Node,
                vec![],
                Some(config(&["lint:check"], &[]))
            )]),
            "'a' check 'lint:check' is not a script of any of its packages"
        );
    }

    #[test]
    fn node_rejects_rust_only_fields() {
        let mut c = config(&["test"], &[]);
        c.run = vec!["echo".to_string()];

        assert!(
            error(vec![manifest("a", Lang::Node, vec![], Some(c))])
                .starts_with("'a' sets Rust only fields")
        );
    }

    #[test]
    fn group_must_not_be_a_project_directory() {
        let mut c = config(&["test"], &[]);
        c.group = Some("b".to_string());

        assert_eq!(
            error(vec![
                manifest("a", Lang::Rust, vec![], Some(c)),
                manifest("b", Lang::Rust, vec![], Some(config(&["test"], &[]))),
            ]),
            "'a' group 'b' is the directory of a project"
        );
    }

    #[test]
    fn owns_matches_whole_path_segments() {
        let project = projects(
            vec![manifest(
                "agdb",
                Lang::Rust,
                vec![],
                Some(config(&["test"], &[])),
            )],
            |_| false,
        )
        .unwrap()
        .remove(0);

        assert!(project.owns("agdb"));
        assert!(project.owns("agdb/src/lib.rs"));
        assert!(!project.owns("agdb_server/src/main.rs"));
    }
}
