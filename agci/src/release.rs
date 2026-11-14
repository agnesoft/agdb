use crate::CIError;
use crate::affected::dependency_order;
use crate::check::PNPM;
use crate::check::Step;
use crate::check::pnpm_install;
use crate::check::pnpm_scripts;
use crate::check::shell_scripts;
use crate::check::step;
use crate::manifest::Kind;
use crate::project::Project;
use std::collections::BTreeSet;

pub(crate) const VERSION_ENV: &str = "AGCI_VERSION";
const DEV: &str = "dev";
/// Run after the project's packages are published on a release.
const RELEASE_HOOK: &str = "release";
/// Run on every push to main that affects the project (`AGCI_VERSION=dev`).
const DEV_HOOK: &str = "release:dev";
pub(crate) const HOOKS: [&str; 2] = [RELEASE_HOOK, DEV_HOOK];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Registry {
    Crates,
    Npm,
}

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
    let hook = |project: &Project, hook: &str, version: &str| {
        let steps = match project.kind {
            Kind::Pnpm if project.has_script(hook) => pnpm_scripts(project, &[hook.to_string()]),
            Kind::Cargo | Kind::Shell if project.has_script(hook) => {
                shell_scripts(project, &[hook.to_string()])
            }
            _ => Vec::new(),
        };
        with_version(steps, version)
    };

    for project in ordered
        .iter()
        .filter(|p| release.affected.contains(p.dir.as_str()))
    {
        steps.extend(hook(project, DEV_HOOK, DEV));
    }

    if let Some(version) = &release.version {
        for project in &ordered {
            for package in &project.published {
                let name = package.name.as_str();

                if project.kind == Kind::Cargo {
                    if !published(Registry::Crates, name, version)? {
                        steps.push(step("cargo", &["publish", "-p", name], ""));
                    }
                } else {
                    let package_version = package.version.as_deref().unwrap_or(version);

                    if !published(Registry::Npm, name, package_version)? {
                        let mut publish = Vec::new();

                        if package.build {
                            publish.push(build(name));
                        }

                        publish.push(step(
                            PNPM,
                            &[
                                "publish",
                                "--provenance",
                                "--access",
                                "public",
                                "--no-git-checks",
                            ],
                            &package.dir,
                        ));
                        steps.extend(with_version(publish, version));
                    }
                }
            }

            steps.extend(hook(project, RELEASE_HOOK, version));
        }

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

    if steps.iter().any(|s| s.program == PNPM) {
        steps.insert(0, pnpm_install());
    }

    Ok(steps)
}

fn build(package: &str) -> Step {
    step(
        PNPM,
        &[
            "exec",
            "turbo",
            "run",
            "build",
            "--env-mode=loose",
            "--log-order=stream",
            &format!("--filter={package}"),
        ],
        "",
    )
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
    use crate::manifest::Dep;
    use crate::project::tests::Entry;
    use crate::project::tests::build;
    use crate::project::tests::config;
    use crate::project::tests::manifest;
    use crate::utilities::BASH;

    fn with(mut entry: Entry, publish: Option<&str>, scripts: &[&str]) -> Entry {
        let manifest = entry.0.as_mut().unwrap();
        manifest.publish = true;
        manifest.version = publish.map(str::to_string);
        manifest
            .scripts
            .extend(scripts.iter().map(|s| s.to_string()));
        entry
    }

    fn sample() -> Vec<Project> {
        let mut ui = manifest(
            "ui",
            Kind::Pnpm,
            vec![Dep::Name("client".to_string())],
            None,
        );
        ui.0.as_mut().unwrap().scripts.insert(DEV_HOOK.to_string());

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
                    &[],
                ),
                with(
                    manifest(
                        "server",
                        Kind::Cargo,
                        vec![Dep::Dir("db".to_string())],
                        None,
                    ),
                    None,
                    &[],
                ),
                with(manifest("db", Kind::Cargo, vec![], None), None, &[]),
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
            Ok(published.contains(&format!("{registry:?} {name}@{version}").as_str()))
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
            lines(&[], Some("1.2.3"), &["Crates db@1.2.3"]),
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
                &["Crates db@1.2.3", "Crates server@1.2.3", "Npm client@0.1.0"]
            ),
            vec![
                format!("AGCI_VERSION=1.2.3 {BASH} release.sh"),
                "gh release create v1.2.3 --target abc --title v1.2.3 --generate-notes".to_string(),
            ]
        );
    }
}
