use crate::CIError;
use crate::config::Os;
use crate::config::Toolchain;
use crate::manifest::Lang;
use crate::project::Project;
use crate::project::is_within;
use agdb::DbError;
use agdb::DbMemory;
use agdb::QueryBuilder;
use serde::Serialize;
use std::collections::BTreeSet;

/// Changes to these affect every project of the language. Entries ending with `/` are directories.
const RUST_INPUTS: [&str; 5] = [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
    ".cargo/",
];
const NODE_INPUTS: [&str; 6] = [
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "turbo.json",
    "vitest.workspace.ts",
    ".npmrc",
];
const ALL_INPUTS: [&str; 1] = [".github/workflows/agdb.yaml"];

#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Job {
    /// The group or the directory of the only project.
    pub(crate) name: String,
    pub(crate) projects: Vec<String>,
    pub(crate) os: Os,
    #[serde(rename = "runs-on")]
    pub(crate) runs_on: &'static str,
    pub(crate) rust: bool,
    pub(crate) node: bool,
    pub(crate) php: bool,
    pub(crate) coverage: bool,
    /// Name for the uploaded artifacts, unique per job.
    pub(crate) artifact: String,
    /// Paths of the uploaded artifacts (coverage, test reports, logs), one per line.
    pub(crate) paths: String,
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

/// Projects affected by the changed files: the owners of the files, the projects declaring them as
/// inputs and everything that transitively depends on those (reverse search along the dependency edges).
pub(crate) fn affected<'a>(
    projects: &'a [Project],
    changed: &[String],
) -> Result<BTreeSet<&'a str>, CIError> {
    let mut seeds: BTreeSet<&str> = BTreeSet::new();

    for file in changed {
        let file = file.as_str();
        let all = ALL_INPUTS.iter().any(|i| matches(file, i));
        let rust = all || RUST_INPUTS.iter().any(|i| matches(file, i));
        let node = all || NODE_INPUTS.iter().any(|i| matches(file, i));

        for project in projects {
            let global = match project.lang {
                Lang::Rust => rust,
                Lang::Node => node,
            };

            if global || project.inputs.iter().any(|i| is_within(file, i)) {
                seeds.insert(&project.dir);
            }
        }

        if let Some(owner) = projects
            .iter()
            .filter(|p| p.owns(file))
            .max_by_key(|p| p.dir.len())
        {
            seeds.insert(&owner.dir);
        }
    }

    let db = graph(projects)?;
    let mut affected = BTreeSet::new();

    for seed in seeds {
        if affected.contains(seed) {
            continue;
        }

        let dependents = db.exec(
            QueryBuilder::select()
                .aliases()
                .search()
                .to(seed)
                .where_()
                .node()
                .query(),
        )?;

        for element in &dependents.elements {
            let alias = element.values[0].value.string()?;
            let project = projects
                .iter()
                .find(|p| p.dir == *alias)
                .ok_or(format!("unknown project '{alias}' in the graph"))?;
            affected.insert(project.dir.as_str());
        }
    }

    Ok(affected)
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

/// A job per project and OS, projects of the same group share it.
pub(crate) fn jobs(projects: &[Project], affected: &BTreeSet<&str>) -> Vec<Job> {
    let mut jobs: Vec<Job> = Vec::new();

    for project in projects
        .iter()
        .filter(|p| affected.contains(p.dir.as_str()))
    {
        let name = project.config.group.as_ref().unwrap_or(&project.dir);
        let paths = ["coverage/", "**/playwright-report/", "*.log"]
            .map(|p| format!("{}/{p}", project.dir))
            .join("\n");

        for os in project.config.os() {
            if let Some(job) = jobs.iter_mut().find(|j| j.name == *name && j.os == os) {
                job.projects.push(project.dir.clone());
                job.rust |= project.needs(Toolchain::Rust);
                job.node |= project.needs(Toolchain::Node);
                job.php |= project.needs(Toolchain::Php);
                job.coverage |= project.config.coverage.is_some();
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
                rust: project.needs(Toolchain::Rust),
                node: project.needs(Toolchain::Node),
                php: project.needs(Toolchain::Php),
                coverage: project.config.coverage.is_some(),
                artifact: format!(
                    "{}-{}",
                    name.replace('/', "_"),
                    format!("{os:?}").to_lowercase()
                ),
                paths: paths.clone(),
            });
        }
    }

    jobs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Dep;
    use crate::project::projects;
    use crate::project::tests::config;
    use crate::project::tests::manifest;

    // db <- server <- client <- ui, server <- bench, ui has its own input file
    fn sample() -> Vec<Project> {
        projects(
            vec![
                manifest("db", Lang::Rust, vec![], Some(config(&["test"], &[]))),
                manifest(
                    "server",
                    Lang::Rust,
                    vec![Dep::Dir("db".to_string())],
                    Some(config(&["test"], &[])),
                ),
                manifest(
                    "bench",
                    Lang::Rust,
                    vec![],
                    Some(config(&["test"], &["server"])),
                ),
                manifest(
                    "client",
                    Lang::Node,
                    vec![],
                    Some(config(&["test"], &["server"])),
                ),
                manifest(
                    "ui",
                    Lang::Node,
                    vec![Dep::Name("client".to_string())],
                    Some(config(&["test"], &["ui.lock"])),
                ),
                manifest("ui/lib", Lang::Node, vec![], None),
            ],
            |p| p == "ui.lock",
        )
        .unwrap()
    }

    fn run(changed: &[&str]) -> Vec<String> {
        let projects = sample();
        let changed: Vec<String> = changed.iter().map(|c| c.to_string()).collect();
        affected(&projects, &changed)
            .unwrap()
            .into_iter()
            .map(|p| p.to_string())
            .collect()
    }

    #[test]
    fn leaf_change_affects_only_itself() {
        assert_eq!(run(&["ui/lib/src/a.ts"]), vec!["ui"]);
    }

    #[test]
    fn change_propagates_to_dependents_across_languages() {
        assert_eq!(
            run(&["server/src/main.rs"]),
            vec!["bench", "client", "server", "ui"]
        );
        assert_eq!(
            run(&["db/src/lib.rs"]),
            vec!["bench", "client", "db", "server", "ui"]
        );
    }

    #[test]
    fn input_file_affects_declaring_project() {
        assert_eq!(run(&["ui.lock"]), vec!["ui"]);
    }

    #[test]
    fn language_inputs_affect_all_projects_of_the_language() {
        assert_eq!(run(&["pnpm-lock.yaml"]), vec!["client", "ui"]);
        assert_eq!(
            run(&["Cargo.toml"]),
            vec!["bench", "client", "db", "server", "ui"]
        );
    }

    #[test]
    fn workflow_affects_everything() {
        assert_eq!(run(&[".github/workflows/agdb.yaml"]).len(), 5);
    }

    #[test]
    fn unowned_files_affect_nothing() {
        assert!(run(&["README.md", ".github/workflows/pr_title.yaml", "dbx/a.rs"]).is_empty());
    }

    #[test]
    fn cycles_terminate() {
        let projects = projects(
            vec![
                manifest("a", Lang::Rust, vec![], Some(config(&["test"], &["b"]))),
                manifest("b", Lang::Rust, vec![], Some(config(&["test"], &["a"]))),
            ],
            |_| false,
        )
        .unwrap();

        assert_eq!(affected(&projects, &["a/x".to_string()]).unwrap().len(), 2);
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
    fn jobs_per_os_with_toolchains() {
        let mut projects = sample();
        let (bench, client) = (0, 1);
        projects[bench].config.os = vec![Os::Linux, Os::Windows];
        projects[bench].config.coverage = Some(100);
        projects[client].config.toolchains = vec![Toolchain::Rust, Toolchain::Php];

        let jobs = jobs(&projects, &BTreeSet::from(["bench", "client"]));

        assert_eq!(
            serde_json::to_string(&jobs).unwrap(),
            r#"[{"name":"bench","projects":["bench"],"os":"linux","runs-on":"ubuntu-latest","rust":true,"node":false,"php":false,"coverage":true,"artifact":"bench-linux","paths":"bench/coverage/\nbench/**/playwright-report/\nbench/*.log"},{"name":"bench","projects":["bench"],"os":"windows","runs-on":"windows-latest","rust":true,"node":false,"php":false,"coverage":true,"artifact":"bench-windows","paths":"bench/coverage/\nbench/**/playwright-report/\nbench/*.log"},{"name":"client","projects":["client"],"os":"linux","runs-on":"ubuntu-latest","rust":true,"node":true,"php":true,"coverage":false,"artifact":"client-linux","paths":"client/coverage/\nclient/**/playwright-report/\nclient/*.log"}]"#
        );
    }

    #[test]
    fn grouped_projects_share_a_job_per_os() {
        let mut projects = sample();
        let (bench, client, db) = (0, 1, 2);
        projects[bench].config.os = vec![Os::Linux, Os::Windows];

        for i in [bench, client, db] {
            projects[i].config.group = Some("small".to_string());
        }

        let jobs = jobs(&projects, &BTreeSet::from(["bench", "client", "db", "ui"]));
        let summary: Vec<(&str, Os, Vec<&str>, bool, bool)> = jobs
            .iter()
            .map(|j| {
                (
                    j.name.as_str(),
                    j.os,
                    j.projects.iter().map(String::as_str).collect(),
                    j.rust,
                    j.node,
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
}
