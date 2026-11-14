use crate::CIError;
use crate::affected::dependency_order;
use crate::check::Step;
use crate::check::node_scripts;
use crate::check::pnpm_install;
use crate::check::step;
use crate::manifest::Lang;
use crate::project::Project;
use std::collections::BTreeSet;

pub(crate) const VERSION_ENV: &str = "AGCI_VERSION";
const DEV: &str = "dev";

/// What to publish after a push to main.
pub(crate) struct Publish<'a> {
    /// Projects affected by the push, their `publish_dev` scripts run with `AGCI_VERSION=dev`.
    pub(crate) affected: BTreeSet<&'a str>,
    /// The version to release when its `v<version>` tag does not exist yet.
    pub(crate) release: Option<String>,
    /// The commit to tag.
    pub(crate) sha: String,
}

/// Dev publishing of the affected projects followed by the release (if any): the `publish` targets of
/// all projects in dependency order and finally the GitHub release with the tag. The tag comes last
/// so that a failed release is retried by re-running it, crates already on crates.io are skipped.
pub(crate) fn plan(
    projects: &[Project],
    publish: &Publish,
    published_crate: impl Fn(&str, &str) -> Result<bool, CIError>,
) -> Result<Vec<Step>, CIError> {
    let ordered = dependency_order(projects)?;
    let mut steps = Vec::new();

    for project in ordered
        .iter()
        .filter(|p| publish.affected.contains(p.dir.as_str()))
    {
        steps.extend(with_version(
            node_scripts(project, &project.config.publish_dev),
            DEV,
        ));
    }

    if let Some(version) = &publish.release {
        for project in &ordered {
            match project.lang {
                Lang::Rust => {
                    let name = project.packages[0].as_str();

                    if project.config.publish.iter().any(|t| t == "crates")
                        && !published_crate(name, version)?
                    {
                        steps.push(step("cargo", &["publish", "-p", name], ""));
                    }
                }
                Lang::Node => steps.extend(with_version(
                    node_scripts(project, &project.config.publish),
                    version,
                )),
            }
        }

        let tag = format!("v{version}");
        steps.push(step(
            "gh",
            &[
                "release",
                "create",
                &tag,
                "--target",
                &publish.sha,
                "--title",
                &tag,
                "--generate-notes",
            ],
            "",
        ));
    }

    if steps
        .iter()
        .any(|s| s.env.iter().any(|(k, _)| k == VERSION_ENV))
    {
        steps.insert(0, pnpm_install());
    }

    Ok(steps)
}

fn with_version(steps: Vec<Step>, version: &str) -> Vec<Step> {
    steps
        .into_iter()
        .map(|mut s| {
            s.env.push((VERSION_ENV.to_string(), version.to_string()));
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::PNPM;
    use crate::manifest::Dep;
    use crate::project::projects;
    use crate::project::tests::config;
    use crate::project::tests::manifest;

    fn sample() -> Vec<Project> {
        let mut db = config(&["test"], &[]);
        db.publish = vec!["crates".to_string()];
        let mut server = db.clone();
        server.depends = vec!["db".to_string()];
        let mut client = config(&["test"], &["server"]);
        client.publish = vec!["build".to_string()];
        let mut ui = config(&["test"], &[]);
        ui.publish = vec!["build".to_string()];
        ui.publish_dev = vec!["test".to_string()];

        projects(
            vec![
                manifest(
                    "ui",
                    Lang::Node,
                    vec![Dep::Name("client".to_string())],
                    Some(ui),
                ),
                manifest("client", Lang::Node, vec![], Some(client)),
                manifest("server", Lang::Rust, vec![], Some(server)),
                manifest("db", Lang::Rust, vec![], Some(db)),
            ],
            |_| false,
        )
        .unwrap()
    }

    fn lines(affected: &[&str], release: Option<&str>, published: &[&str]) -> Vec<String> {
        let projects = sample();
        let publish = Publish {
            affected: affected.iter().copied().collect(),
            release: release.map(str::to_string),
            sha: "abc".to_string(),
        };

        plan(&projects, &publish, |name, _| Ok(published.contains(&name)))
            .unwrap()
            .iter()
            .map(Step::to_string)
            .collect()
    }

    #[test]
    fn nothing_to_publish() {
        assert!(lines(&["server", "client"], None, &[]).is_empty());
    }

    #[test]
    fn dev_runs_for_affected_projects_only() {
        assert_eq!(
            lines(&["ui"], None, &[]),
            vec![
                format!("{PNPM} install --frozen-lockfile"),
                format!(
                    "AGCI_VERSION=dev {PNPM} exec turbo run test --force --env-mode=loose --filter=ui"
                ),
            ]
        );
    }

    #[test]
    fn release_in_dependency_order_with_tag_last() {
        assert_eq!(
            lines(&[], Some("1.2.3"), &["db"]),
            vec![
                format!("{PNPM} install --frozen-lockfile"),
                "cargo publish -p server".to_string(),
                format!(
                    "AGCI_VERSION=1.2.3 {PNPM} exec turbo run build --force --env-mode=loose --filter=client"
                ),
                format!(
                    "AGCI_VERSION=1.2.3 {PNPM} exec turbo run build --force --env-mode=loose --filter=ui"
                ),
                "gh release create v1.2.3 --target abc --title v1.2.3 --generate-notes".to_string(),
            ]
        );
    }
}
