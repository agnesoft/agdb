use crate::CIError;
use crate::affected::dependency_order;
use crate::check::Step;
use crate::check::step;
use crate::project::Project;
use crate::project_type::Hook;
use crate::project_type::Kind;
use crate::project_type::Registry;
use std::collections::BTreeSet;

pub(crate) const VERSION_ENV: &str = "AGCI_VERSION";
const DEV: &str = "dev";

/// What to release after a push to main.
pub(crate) struct Release<'a> {
    /// Projects affected by the push, their dev release hooks run.
    pub(crate) affected: BTreeSet<&'a str>,
    /// The version to release when its `v<version>` tag does not exist yet.
    pub(crate) version: Option<String>,
    /// The commit to tag.
    pub(crate) sha: String,
}

/// Dev release hooks of the affected projects followed by the release (if any): the packages of all
/// projects in dependency order (those already in their registry are skipped), each project's
/// release hook and finally the GitHub release with the tag. The tag comes last so that a failed
/// release is retried by re-running it.
pub(crate) fn plan(
    projects: &[Project],
    release: &Release,
    published: impl Fn(Registry, &str, &str) -> Result<bool, CIError>,
) -> Result<Vec<Step>, CIError> {
    let ordered = dependency_order(projects)?;
    let mut steps = Vec::new();

    for project in ordered
        .iter()
        .filter(|p| release.affected.contains(p.dir.as_str()))
    {
        steps.push((project.kind, hook(project, Hook::Dev, DEV)));
    }

    if let Some(version) = &release.version {
        for project in &ordered {
            let mut project_steps = Vec::new();

            for package in &project.published {
                if let Some(publish) = project.project_type.publish(package, version)
                    && !published(publish.registry, &package.name, &publish.version)?
                {
                    project_steps.extend(publish.steps);
                }
            }

            project_steps.extend(hook(project, Hook::Release, version));
            steps.push((project.kind, project_steps));
        }
    }

    let mut steps = with_setup(steps);

    if let Some(version) = &release.version {
        let tag = format!("v{version}");
        steps.push(step(
            "gh",
            &[
                "release",
                "create",
                &tag,
                "--target",
                &release.sha,
                "--title",
                &tag,
                "--generate-notes",
            ],
            "",
        ));
    }

    Ok(steps)
}

/// The steps of the hook with the version in the environment.
pub(crate) fn hook(project: &Project, hook: Hook, version: &str) -> Vec<Step> {
    with_version(project.project_type.hook(project, hook), version)
}

/// The steps of the projects preceded by the setup of the types that have any.
pub(crate) fn with_setup(steps: Vec<(Kind, Vec<Step>)>) -> Vec<Step> {
    let kinds: BTreeSet<Kind> = steps
        .iter()
        .filter(|(_, steps)| !steps.is_empty())
        .map(|(kind, _)| *kind)
        .collect();
    kinds
        .into_iter()
        .flat_map(Kind::setup)
        .chain(steps.into_iter().flat_map(|(_, steps)| steps))
        .collect()
}

pub(crate) fn with_version(steps: Vec<Step>, version: &str) -> Vec<Step> {
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
    use crate::manifest::Dep;
    use crate::project::tests::Entry;
    use crate::project::tests::build;
    use crate::project::tests::config;
    use crate::project::tests::manifest;
    use crate::project_type::Details;
    use crate::project_type::pnpm::PNPM;
    use crate::project_type::pnpm::tests::package_details;
    use crate::utilities::BASH;

    /// Published, a package.json with the version.
    fn with(mut entry: Entry, version: Option<&str>) -> Entry {
        let manifest = entry.0.as_mut().unwrap();
        manifest.publish = true;

        if let Details::Pnpm(_) = manifest.details {
            manifest.details = package_details(&manifest.name, &["build", "test"], version);
        }

        entry
    }

    fn sample() -> Vec<Project> {
        let mut ui = manifest(
            "ui",
            Kind::Pnpm,
            vec![Dep::Name("client".to_string())],
            None,
        );
        ui.0.as_mut().unwrap().details =
            package_details("ui", &["build", "test", Hook::Dev.name()], None);

        build(
            vec![
                ui,
                with(
                    manifest(
                        "client",
                        Kind::Pnpm,
                        vec![],
                        Some(config(&["test"], &["server"])),
                    ),
                    Some("0.1.0"),
                ),
                with(
                    manifest(
                        "server",
                        Kind::Cargo,
                        vec![Dep::Dir("db".to_string())],
                        None,
                    ),
                    None,
                ),
                with(manifest("db", Kind::Cargo, vec![], None), None),
                manifest("tools", Kind::Cargo, vec![], None),
                manifest(
                    "server/image",
                    Kind::Shell,
                    vec![],
                    Some(config(&["test"], &["server"])),
                ),
            ],
            |p| {
                [
                    "server/image",
                    "server/image/test.sh",
                    "server/image/release.sh",
                    "server/image/release_dev.sh",
                ]
                .contains(&p)
            },
        )
        .unwrap()
    }

    fn lines(affected: &[&str], release: Option<&str>, published: &[&str]) -> Vec<String> {
        let projects = sample();
        let release = Release {
            affected: affected.iter().copied().collect(),
            version: release.map(str::to_string),
            sha: "abc".to_string(),
        };

        plan(&projects, &release, |registry, name, version| {
            Ok(published.contains(&format!("{} {name}@{version}", registry.name()).as_str()))
        })
        .unwrap()
        .iter()
        .map(Step::to_string)
        .collect()
    }

    #[test]
    fn nothing_to_release() {
        assert!(lines(&["server", "client"], None, &[]).is_empty());
    }

    #[test]
    fn dev_runs_the_hooks_of_the_affected_projects_only() {
        assert_eq!(
            lines(&["ui", "client"], None, &[]),
            vec![
                format!("{PNPM} install --frozen-lockfile"),
                format!(
                    "AGCI_VERSION=dev {PNPM} exec turbo run release:dev --force --env-mode=loose --log-order=stream --filter=ui"
                ),
            ]
        );
    }

    #[test]
    fn shell_dev_release_needs_no_pnpm_install() {
        assert_eq!(
            lines(&["server/image"], None, &[]),
            vec![format!("AGCI_VERSION=dev {BASH} release_dev.sh")]
        );
    }

    #[test]
    fn release_publishes_in_dependency_order_with_tag_last() {
        assert_eq!(
            lines(&[], Some("1.2.3"), &["crates.io db@1.2.3"]),
            vec![
                format!("{PNPM} install --frozen-lockfile"),
                "cargo publish -p server".to_string(),
                format!(
                    "AGCI_VERSION=1.2.3 {PNPM} exec turbo run build --env-mode=loose --log-order=stream --filter=client"
                ),
                format!(
                    "AGCI_VERSION=1.2.3 {PNPM} publish --provenance --access public --no-git-checks"
                ),
                format!("AGCI_VERSION=1.2.3 {BASH} release.sh"),
                "gh release create v1.2.3 --target abc --title v1.2.3 --generate-notes".to_string(),
            ]
        );
    }

    #[test]
    fn published_packages_are_skipped() {
        assert_eq!(
            lines(
                &[],
                Some("1.2.3"),
                &[
                    "crates.io db@1.2.3",
                    "crates.io server@1.2.3",
                    "npm client@0.1.0"
                ]
            ),
            vec![
                format!("AGCI_VERSION=1.2.3 {BASH} release.sh"),
                "gh release create v1.2.3 --target abc --title v1.2.3 --generate-notes".to_string(),
            ]
        );
    }
}
