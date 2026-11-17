use crate::CIError;
use crate::affected::dependency_order;
use crate::check::Step;
use crate::check::run_steps;
use crate::git;
use crate::project::Project;
use crate::project_type;
use crate::project_type::Hook;
use crate::release::hook;
use crate::release::with_setup;
use agplatform::Fs;
use agplatform::Platform;

/// Updates the versions of the latest release in the manifests to the repository version and runs
/// the `regenerate` hooks of the projects in dependency order.
pub(crate) fn regenerate(
    platform: &impl Platform,
    projects: &[Project],
    root: &str,
) -> Result<(), CIError> {
    let current_version = git::current_version(platform)?;
    let new_version = project_type::version(platform)?;
    println!("Current version: {current_version}");
    println!("New version: {new_version}");

    let fs = platform.fs();
    let read = |file: &str| -> Result<String, CIError> { Ok(fs.read_to_string(file)?) };

    for project in projects {
        for (file, content) in
            project
                .project_type
                .version_updates(&current_version, &new_version, &read)?
        {
            println!("Updating '{file}'");
            fs.write(&file, content)?;
        }
    }

    run_steps(
        platform,
        "regenerate",
        &plan(projects, &new_version)?,
        root,
        &mut std::io::stdout(),
    )
}

fn plan(projects: &[Project], version: &str) -> Result<Vec<Step>, CIError> {
    Ok(with_setup(
        dependency_order(projects)?
            .into_iter()
            .map(|p| (p.kind, hook(p, Hook::Regenerate, version)))
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::tests::build;
    use crate::project::tests::config;
    use crate::project::tests::manifest;
    use crate::project_type::Kind;
    use crate::project_type::pnpm::PNPM;
    use crate::project_type::pnpm::tests::package_details;
    use crate::utilities::BASH;

    #[test]
    fn hooks_in_dependency_order_after_the_setup() {
        let mut client = manifest(
            "client",
            Kind::Pnpm,
            vec![],
            Some(config(&["test"], &["server"])),
        );
        client.0.as_mut().unwrap().details =
            package_details("client", &["test", Hook::Regenerate.name()], None);
        let projects = build(
            vec![
                client,
                manifest("server", Kind::Cargo, vec![], None),
                manifest("db", Kind::Cargo, vec![], None),
            ],
            |p| p == "server/regenerate.sh",
        )
        .unwrap();

        assert_eq!(
            plan(&projects, "1.2.3")
                .unwrap()
                .iter()
                .map(Step::to_string)
                .collect::<Vec<_>>(),
            vec![
                format!("{PNPM} install --frozen-lockfile"),
                format!("AGCI_VERSION=1.2.3 {BASH} regenerate.sh"),
                format!(
                    "AGCI_VERSION=1.2.3 {PNPM} exec turbo run regenerate --force --env-mode=loose --log-order=stream --filter=client"
                ),
            ]
        );
    }
}
