use crate::CIError;
use crate::config::FILE;
use crate::config::File;
use crate::config::Os;
use crate::config::Toolchain;
use crate::git;
use crate::project::Project;
use crate::project::is_within;
use crate::utilities;
use agdb::DbError;
use agdb::DbMemory;
use agdb::QueryBuilder;
use agplatform::Fs;
use agplatform::Platform;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Prefix of the evidence (actions cache keys) of passed check jobs.
pub(crate) const EVIDENCE: &str = "agci-done-";

/// Changes to these affect every project, those of a project type its `inputs`. Entries ending with `/`
/// are directories.
const ALL_INPUTS: [&str; 1] = [".github/workflows/agdb.yaml"];

#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Job {
    /// The group or the directory of the only project.
    pub(crate) name: String,
    pub(crate) projects: Vec<String>,
    pub(crate) os: Os,
    #[serde(rename = "runs-on")]
    pub(crate) runs_on: &'static str,
    /// Whether the job installs the toolchain.
    #[serde(flatten)]
    pub(crate) toolchains: BTreeMap<Toolchain, bool>,
    /// Name for the uploaded artifacts, unique per job.
    pub(crate) artifact: String,
    /// Paths of the uploaded artifacts (coverage, test reports, logs), one per line.
    pub(crate) paths: String,
    /// The actions cache key recording that the job passed: its OS, name and the fingerprints of its
    /// projects.
    pub(crate) evidence: String,
}

fn matches(path: &str, input: &str) -> bool {
    match input.strip_suffix('/') {
        Some(dir) => is_within(path, dir),
        None => path == input,
    }
}

/// The project dependency graph: a node per project aliased by its directory and an edge from each
/// project to every project it depends on.
pub(crate) fn graph(projects: &[Project]) -> Result<DbMemory, CIError> {
    // DbMemory loads an existing file of the given name, an empty name cannot be one.
    let mut db = DbMemory::new("")?;
    db.transaction_mut(|t| -> Result<(), DbError> {
        t.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(projects.iter().map(|p| p.dir.as_str()).collect::<Vec<_>>())
                .query(),
        )?;

        for project in projects {
            for dep in &project.depends {
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(project.dir.as_str())
                        .to(dep.as_str())
                        .query(),
                )?;
            }
        }

        Ok(())
    })?;
    Ok(db)
}

/// The files (path -> git object id) and `agci.toml` of a commit or the working tree.
#[derive(Debug, Default)]
pub(crate) struct Snapshot {
    pub(crate) files: BTreeMap<String, String>,
    pub(crate) config: File,
}

impl Snapshot {
    /// The commit or, without one, the working tree.
    pub(crate) fn of(platform: &impl Platform, rev: Option<&str>) -> Result<Self, CIError> {
        let files = match rev {
            Some(rev) => git::files_at(platform, rev)?,
            None => git::worktree_files(platform)?,
        };
        let config = match rev {
            _ if !files.contains_key(FILE) => File::default(),
            Some(rev) => File::parse(&git::file_at(platform, rev, FILE)?)?,
            None => File::parse(&platform.fs().read_to_string(FILE)?)?,
        };
        Ok(Self { files, config })
    }
}

/// What a project consists of in the snapshot: its effective `agci.toml` entry (with the defaults),
/// its files (not those of projects nested in it), its `dependencies` paths and the files
/// affecting every project of its type.
fn contents<'a>(
    projects: &'a [Project],
    snapshot: &Snapshot,
) -> Result<BTreeMap<&'a str, String>, CIError> {
    let mut contents = BTreeMap::new();

    for project in projects {
        let entry = serde_json::to_string(&snapshot.config.effective(&project.dir, project.kind))?;
        contents.insert(project.dir.as_str(), format!("{entry}\n"));
    }

    for (path, id) in &snapshot.files {
        let all = ALL_INPUTS.iter().any(|i| matches(path, i));
        let owner = projects
            .iter()
            .filter(|p| p.owns(path))
            .max_by_key(|p| p.dir.len())
            .map(|p| p.dir.as_str());

        for project in projects {
            let global = all
                || project
                    .project_type
                    .inputs()
                    .iter()
                    .any(|i| matches(path, i));

            if global
                || owner == Some(project.dir.as_str())
                || project.inputs.iter().any(|i| is_within(path, i))
            {
                contents
                    .entry(project.dir.as_str())
                    .or_default()
                    .push_str(&format!("{id} {path}\n"));
            }
        }
    }

    Ok(contents)
}

/// A hash per project of its contents and those of every project it transitively depends on, so it
/// changes with any of them. Stable across commits (git object ids) and runs.
pub(crate) fn fingerprints(
    projects: &[Project],
    snapshot: &Snapshot,
) -> Result<BTreeMap<String, String>, CIError> {
    let contents = contents(projects, snapshot)?;
    let db = graph(projects)?;
    let mut fingerprints = BTreeMap::new();

    for project in projects {
        let closure = db.exec(
            QueryBuilder::select()
                .aliases()
                .search()
                .from(project.dir.as_str())
                .where_()
                .node()
                .query(),
        )?;
        let mut members = Vec::new();

        for element in &closure.elements {
            members.push(element.values[0].value.string()?.as_str());
        }

        members.sort();
        let mut text = String::new();

        for member in members {
            text.push_str(&format!("{member}\n"));
            text.push_str(contents.get(member).map_or("", String::as_str));
        }

        fingerprints.insert(project.dir.clone(), utilities::hash(text.as_bytes()));
    }

    Ok(fingerprints)
}

/// Projects whose fingerprint differs between the snapshots: their own contents or those of a
/// (transitive) dependency changed. A new project or one without an entry in `base` is affected.
pub(crate) fn affected<'a>(
    projects: &'a [Project],
    base: &Snapshot,
    head: &Snapshot,
) -> Result<BTreeSet<&'a str>, CIError> {
    let base = fingerprints(projects, base)?;
    let head = fingerprints(projects, head)?;
    Ok(projects
        .iter()
        .filter(|p| base.get(&p.dir) != head.get(&p.dir))
        .map(|p| p.dir.as_str())
        .collect())
}

/// Projects ordered so that every project comes after all of its (transitive) dependencies: in an
/// acyclic graph a dependency always has fewer transitive dependencies than its dependent.
pub(crate) fn dependency_order(projects: &[Project]) -> Result<Vec<&Project>, CIError> {
    let db = graph(projects)?;
    let mut ordered = Vec::new();

    for project in projects {
        let dependencies = db
            .exec(
                QueryBuilder::search()
                    .from(project.dir.as_str())
                    .where_()
                    .node()
                    .query(),
            )?
            .result;
        ordered.push((dependencies, project));
    }

    ordered.sort_by_key(|(dependencies, _)| *dependencies);
    Ok(ordered.into_iter().map(|(_, p)| p).collect())
}

/// The affected projects that run on `os` and those that do not.
pub(crate) fn on_os<'a>(
    projects: &'a [Project],
    affected: &BTreeSet<&str>,
    os: Os,
) -> (Vec<&'a Project>, Vec<&'a Project>) {
    projects
        .iter()
        .filter(|p| affected.contains(p.dir.as_str()))
        .partition(|p| p.settings.os().contains(&os))
}

/// A job per project and OS, projects of the same group share it.
pub(crate) fn jobs(
    projects: &[Project],
    affected: &BTreeSet<&str>,
    fingerprints: &BTreeMap<String, String>,
) -> Vec<Job> {
    let mut jobs: Vec<Job> = Vec::new();

    for project in projects
        .iter()
        .filter(|p| affected.contains(p.dir.as_str()))
    {
        let name = project.settings.group.as_ref().unwrap_or(&project.dir);
        let paths = project
            .project_type
            .artifacts()
            .iter()
            .chain(&["*.log"])
            .map(|p| format!("{}/{p}", project.dir))
            .collect::<Vec<_>>()
            .join("\n");
        let toolchains = || Toolchain::ALL.map(|t| (t, project.needs(t)));

        for os in project.settings.os() {
            if let Some(job) = jobs.iter_mut().find(|j| j.name == *name && j.os == os) {
                job.projects.push(project.dir.clone());

                for (toolchain, needs) in toolchains() {
                    *job.toolchains.entry(toolchain).or_default() |= needs;
                }

                job.paths = format!("{}\n{paths}", job.paths);
                continue;
            }

            jobs.push(Job {
                name: name.clone(),
                projects: vec![project.dir.clone()],
                os,
                runs_on: match os {
                    Os::Linux => "ubuntu-latest",
                    Os::Windows => "windows-latest",
                },
                toolchains: toolchains().into(),
                artifact: format!(
                    "{}-{}",
                    name.replace('/', "_"),
                    format!("{os:?}").to_lowercase()
                ),
                paths: paths.clone(),
                evidence: String::new(),
            });
        }
    }

    for job in &mut jobs {
        let projects: String = job
            .projects
            .iter()
            .map(|p| format!("{p} {}\n", fingerprints.get(p).map_or("", String::as_str)))
            .collect();
        job.evidence = format!(
            "{EVIDENCE}{}-{}",
            job.artifact,
            utilities::hash(projects.as_bytes())
        );
    }

    jobs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Dep;
    use crate::project::tests::build;
    use crate::project::tests::config;
    use crate::project::tests::manifest;
    use crate::project_type::Kind;

    // db <- server <- client <- ui, server <- bench, ui has its own input file, server/image is a
    // Shell project nested in server
    fn sample() -> Vec<Project> {
        build(
            vec![
                manifest("db", Kind::Cargo, vec![], Some(config(&["coverage"], &[]))),
                manifest(
                    "server",
                    Kind::Cargo,
                    vec![Dep::Dir("db".to_string())],
                    Some(config(&["coverage"], &[])),
                ),
                manifest(
                    "bench",
                    Kind::Cargo,
                    vec![],
                    Some(config(&["coverage"], &["server"])),
                ),
                manifest(
                    "client",
                    Kind::Pnpm,
                    vec![],
                    Some(config(&["test"], &["server"])),
                ),
                manifest(
                    "ui",
                    Kind::Pnpm,
                    vec![Dep::Name("client".to_string())],
                    Some(config(&["test"], &["ui.lock"])),
                ),
                manifest("ui/lib", Kind::Pnpm, vec![], None),
                manifest(
                    "server/image",
                    Kind::Shell,
                    vec![],
                    Some(config(&["test"], &["server"])),
                ),
            ],
            |p| ["ui.lock", "server/image", "server/image/test.sh"].contains(&p),
        )
        .unwrap()
    }

    fn snapshot(files: &[(&str, &str)], config: &str) -> Snapshot {
        Snapshot {
            files: files
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            config: File::parse(config).unwrap(),
        }
    }

    const FILES: [(&str, &str); 4] = [
        ("db/src/lib.rs", "1"),
        ("server/src/main.rs", "1"),
        ("server/image/test.sh", "1"),
        ("ui/lib/src/a.ts", "1"),
    ];

    fn between(projects: &[Project], base: &Snapshot, head: &Snapshot) -> Vec<String> {
        affected(projects, base, head)
            .unwrap()
            .into_iter()
            .map(|p| p.to_string())
            .collect()
    }

    /// The projects affected by changing (or adding) the files.
    fn run(changed: &[&str]) -> Vec<String> {
        let mut head = snapshot(&FILES, "");

        for file in changed {
            head.files.insert(file.to_string(), "2".to_string());
        }

        between(&sample(), &snapshot(&FILES, ""), &head)
    }

    #[test]
    fn leaf_change_affects_only_itself() {
        assert_eq!(run(&["ui/lib/src/a.ts"]), vec!["ui"]);
    }

    #[test]
    fn nothing_changed_affects_nothing() {
        assert!(run(&[]).is_empty());
    }

    #[test]
    fn change_propagates_to_dependents_across_types() {
        assert_eq!(
            run(&["server/src/main.rs"]),
            vec!["bench", "client", "server", "server/image", "ui"]
        );
        assert_eq!(
            run(&["db/src/lib.rs"]),
            vec!["bench", "client", "db", "server", "server/image", "ui"]
        );
    }

    #[test]
    fn nested_project_files_do_not_affect_the_enclosing_one() {
        assert_eq!(run(&["server/image/test.sh"]), vec!["server/image"]);
    }

    #[test]
    fn deleted_file_affects_its_project() {
        let mut head = snapshot(&FILES, "");
        head.files.remove("ui/lib/src/a.ts");

        assert_eq!(between(&sample(), &snapshot(&FILES, ""), &head), vec!["ui"]);
    }

    #[test]
    fn config_entry_change_affects_the_project_and_its_dependents_only() {
        let base = snapshot(
            &FILES,
            "[projects.client]\ngroup = \"a\"\n[projects.db]\ncoverage = false",
        );
        let head = snapshot(
            &FILES,
            "[projects.client]\ngroup = \"b\"\n[projects.db]\ncoverage = false",
        );

        assert_eq!(between(&sample(), &base, &head), vec!["client", "ui"]);
    }

    #[test]
    fn new_entry_affects_the_project() {
        let head = snapshot(&FILES, "[projects.client]\ngroup = \"a\"");

        assert_eq!(
            between(&sample(), &snapshot(&FILES, ""), &head),
            vec!["client", "ui"]
        );
    }

    #[test]
    fn defaults_affect_the_projects_of_the_type_not_setting_the_field() {
        let base = snapshot(&FILES, "[projects.db]\ncoverage = true");
        let head = snapshot(
            &FILES,
            "[defaults.cargo]\ncoverage = false\n[projects.db]\ncoverage = true",
        );

        // bench and server use the default, client and ui depend on server
        assert_eq!(
            between(&sample(), &base, &head),
            vec!["bench", "client", "server", "server/image", "ui"]
        );
    }

    #[test]
    fn input_file_affects_declaring_project() {
        assert_eq!(run(&["ui.lock"]), vec!["ui"]);
    }

    #[test]
    fn type_inputs_affect_all_projects_of_the_type() {
        assert_eq!(run(&["pnpm-lock.yaml"]), vec!["client", "ui"]);
        assert_eq!(
            run(&["Cargo.toml"]),
            vec!["bench", "client", "db", "server", "server/image", "ui"]
        );
    }

    #[test]
    fn workflow_affects_everything() {
        assert_eq!(run(&[".github/workflows/agdb.yaml"]).len(), 6);
    }

    #[test]
    fn unowned_files_affect_nothing() {
        assert!(run(&["README.md", ".github/workflows/pr_title.yaml", "dbx/a.rs"]).is_empty());
    }

    #[test]
    fn cycles_terminate() {
        let projects = build(
            vec![
                manifest(
                    "a",
                    Kind::Cargo,
                    vec![],
                    Some(config(&["coverage"], &["b"])),
                ),
                manifest(
                    "b",
                    Kind::Cargo,
                    vec![],
                    Some(config(&["coverage"], &["a"])),
                ),
            ],
            |_| false,
        )
        .unwrap();
        let head = snapshot(&[("a/x", "1")], "");

        assert_eq!(between(&projects, &Snapshot::default(), &head).len(), 2);
    }

    #[test]
    fn dependencies_come_first() {
        let projects = sample();
        let order: Vec<&str> = dependency_order(&projects)
            .unwrap()
            .into_iter()
            .map(|p| p.dir.as_str())
            .collect();
        let position = |dir: &str| order.iter().position(|d| *d == dir).unwrap();

        assert!(position("db") < position("server"));
        assert!(position("server") < position("client"));
        assert!(position("server") < position("bench"));
        assert!(position("client") < position("ui"));
    }

    #[test]
    fn on_os_splits_the_affected_by_their_os() {
        let mut projects = sample();
        let bench = 0;
        projects[bench].settings.os = vec![Os::Windows];
        let dirs = |projects: Vec<&Project>| -> Vec<String> {
            projects.into_iter().map(|p| p.dir.clone()).collect()
        };
        let affected = BTreeSet::from(["bench", "client", "db"]);

        let (linux, not_linux) = on_os(&projects, &affected, Os::Linux);
        let (windows, not_windows) = on_os(&projects, &affected, Os::Windows);

        assert_eq!(
            (dirs(linux), dirs(not_linux)),
            (vec!["client".into(), "db".into()], vec!["bench".into()])
        );
        assert_eq!(
            (dirs(windows), dirs(not_windows)),
            (vec!["bench".into()], vec!["client".into(), "db".into()])
        );
    }

    #[test]
    fn jobs_per_os_with_toolchains() {
        let mut projects = sample();
        let (bench, client) = (0, 1);
        projects[bench].settings.os = vec![Os::Linux, Os::Windows];
        projects[client].settings.add_toolchains = vec![Toolchain::Rust, Toolchain::Php];

        let mut jobs = jobs(
            &projects,
            &BTreeSet::from(["bench", "client"]),
            &BTreeMap::new(),
        );

        for job in &mut jobs {
            assert!(
                job.evidence
                    .starts_with(&format!("agci-done-{}-", job.artifact))
            );
            job.evidence.clear();
        }

        assert_eq!(
            serde_json::to_string(&jobs).unwrap(),
            r#"[{"name":"bench","projects":["bench"],"os":"linux","runs-on":"ubuntu-latest","rust":true,"node":false,"php":false,"coverage":true,"artifact":"bench-linux","paths":"bench/*.log","evidence":""},{"name":"bench","projects":["bench"],"os":"windows","runs-on":"windows-latest","rust":true,"node":false,"php":false,"coverage":true,"artifact":"bench-windows","paths":"bench/*.log","evidence":""},{"name":"client","projects":["client"],"os":"linux","runs-on":"ubuntu-latest","rust":true,"node":true,"php":true,"coverage":false,"artifact":"client-linux","paths":"client/coverage/\nclient/**/playwright-report/\nclient/*.log","evidence":""}]"#
        );
    }

    #[test]
    fn grouped_projects_share_a_job_per_os() {
        let mut projects = sample();
        let (bench, client, db) = (0, 1, 2);
        projects[bench].settings.os = vec![Os::Linux, Os::Windows];

        for i in [bench, client, db] {
            projects[i].settings.group = Some("small".to_string());
        }

        let jobs = jobs(
            &projects,
            &BTreeSet::from(["bench", "client", "db", "ui"]),
            &BTreeMap::new(),
        );
        let summary: Vec<(&str, Os, Vec<&str>, bool, bool)> = jobs
            .iter()
            .map(|j| {
                (
                    j.name.as_str(),
                    j.os,
                    j.projects.iter().map(String::as_str).collect(),
                    j.toolchains[&Toolchain::Rust],
                    j.toolchains[&Toolchain::Node],
                )
            })
            .collect();

        assert_eq!(
            summary,
            vec![
                (
                    "small",
                    Os::Linux,
                    vec!["bench", "client", "db"],
                    true,
                    true
                ),
                ("small", Os::Windows, vec!["bench"], true, false),
                ("ui", Os::Linux, vec!["ui"], false, true),
            ]
        );
        assert_eq!(jobs[0].artifact, "small-linux");
        assert!(jobs[0].paths.ends_with("\ndb/*.log"));
    }

    #[test]
    fn evidence_changes_with_the_fingerprint_of_any_project_of_the_job() {
        let mut projects = sample();
        let (bench, db) = (0, 2);
        projects[bench].settings.group = Some("small".to_string());
        projects[db].settings.group = Some("small".to_string());
        let affected = BTreeSet::from(["bench", "db"]);
        let evidence = |fingerprints: &[(&str, &str)]| {
            let fingerprints = fingerprints
                .iter()
                .map(|(p, f)| (p.to_string(), f.to_string()))
                .collect();
            jobs(&projects, &affected, &fingerprints)[0]
                .evidence
                .clone()
        };
        let base = evidence(&[("bench", "1"), ("db", "1"), ("ui", "1")]);

        assert_eq!(base, evidence(&[("bench", "1"), ("db", "1"), ("ui", "2")]));
        assert_ne!(base, evidence(&[("bench", "1"), ("db", "2"), ("ui", "1")]));
    }

    #[test]
    fn fingerprints_are_stable_and_include_dependencies() {
        let projects = sample();
        let head = snapshot(&FILES, "[projects.db]\ncoverage = false");
        let a = fingerprints(&projects, &head).unwrap();
        let b = fingerprints(&projects, &head).unwrap();
        let mut changed = snapshot(&FILES, "[projects.db]\ncoverage = false");
        changed
            .files
            .insert("db/src/lib.rs".to_string(), "2".to_string());
        let c = fingerprints(&projects, &changed).unwrap();

        assert_eq!(a, b);
        assert_ne!(a["server"], c["server"]);
        assert_eq!(a["ui"].len(), 16);
    }
}
