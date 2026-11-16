pub(crate) mod cargo;
pub(crate) mod pnpm;
pub(crate) mod shell;

use crate::CIError;
use crate::check::Check;
use crate::check::Step;
use crate::config::Toolchain;
use crate::config::TypeConfig;
use crate::manifest::Manifest;
use crate::project::Package;
use crate::project::Project;
use agplatform::BlockingRequestBuilderExt;
use agplatform::Platform;
use agplatform::Request;
use agplatform::reqwest::header::USER_AGENT;
use serde::Deserialize;

/// How the projects of a type (the tool driving them) are found, checked and released. The static
/// functions are of the type, reached via its [`Kind`]. Each implementation holds the project's
/// checks and hooks resolved and validated when it is built.
pub(crate) trait ProjectType: std::fmt::Debug {
    /// Its manifests among the tracked files, what only it needs of them in their `details`.
    fn manifests(platform: &impl Platform, tracked: &[&str]) -> Result<Vec<Manifest>, CIError>
    where
        Self: Sized;

    /// When neither `agci.toml` nor the repository root set them, only those a project can run
    /// apply.
    fn default_checks() -> Vec<&'static str>
    where
        Self: Sized;

    /// The default checks the repository root sets for its projects, `read` takes repository
    /// paths (None if missing).
    fn root_checks(_read: &dyn Fn(&str) -> Result<Option<String>, CIError>) -> RootChecks
    where
        Self: Sized,
    {
        Ok(None)
    }

    /// The version of the repository's releases if the type defines it in the repository root.
    fn version(_platform: &impl Platform) -> Result<Option<String>, CIError>
    where
        Self: Sized,
    {
        Ok(None)
    }

    /// Run once before any steps of its projects outside of the checks (hooks, publishing).
    fn setup() -> Vec<Step>
    where
        Self: Sized,
    {
        Vec::new()
    }

    /// The project in `dir` of its own manifest and those folded into it (`details`, its own first),
    /// none for an entry without a manifest.
    fn new(
        dir: &str,
        config: TypeConfig,
        details: Vec<Details>,
        path_exists: &dyn Fn(&str) -> bool,
    ) -> Result<Self, CIError>
    where
        Self: Sized;

    /// Installed in the jobs of the project.
    fn toolchains(&self) -> Vec<Toolchain>;

    /// Repository paths affecting all of its projects. Entries ending with `/` are directories.
    fn inputs(&self) -> &'static [&'static str];

    /// Paths in the project directory its checks write reports to, uploaded from the jobs.
    fn artifacts(&self) -> &'static [&'static str] {
        &[]
    }

    /// The names of the checks in the order they run.
    fn check_names(&self) -> Vec<String>;

    /// The checks in the order they run.
    fn checks(&self, project: &Project, projects: &[Project], root: &str) -> Vec<Check>;

    /// None if the project does not have the hook.
    fn hook(&self, project: &Project, hook: Hook) -> Vec<Step>;

    /// None if the type has no registry.
    fn publish(&self, _package: &Package, _version: &str) -> Option<Publish> {
        None
    }

    /// Its manifests with the versions that are `from` updated to `to` (file, content), `read`
    /// takes repository paths.
    fn version_updates(
        &self,
        _from: &str,
        _to: &str,
        _read: &dyn Fn(&str) -> Result<String, CIError>,
    ) -> Result<Vec<(String, String)>, CIError> {
        Ok(Vec::new())
    }
}

pub(crate) type RootChecks = Result<Option<Vec<String>>, CIError>;

/// The project types, the only place listing them.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Kind {
    /// A Cargo workspace member.
    Cargo,
    /// A package.json package (pnpm workspace, turbo).
    Pnpm,
    /// A directory without a manifest, its checks are script files.
    Shell,
}

/// What only the project type knows about its manifest, handed back to it to build the project.
#[derive(Debug)]
pub(crate) enum Details {
    Cargo(cargo::CrateDetails),
    Pnpm(pnpm::PackageDetails),
}

impl Details {
    pub(crate) fn kind(&self) -> Kind {
        match self {
            Details::Cargo(_) => Kind::Cargo,
            Details::Pnpm(_) => Kind::Pnpm,
        }
    }
}

impl Kind {
    pub(crate) const ALL: [Kind; 3] = [Kind::Cargo, Kind::Pnpm, Kind::Shell];

    /// Of an `agci.toml` entry without a manifest.
    pub(crate) const WITHOUT_MANIFEST: Kind = Kind::Shell;

    pub(crate) fn manifests(
        self,
        platform: &impl Platform,
        tracked: &[&str],
    ) -> Result<Vec<Manifest>, CIError> {
        match self {
            Kind::Cargo => cargo::Cargo::manifests(platform, tracked),
            Kind::Pnpm => pnpm::Pnpm::manifests(platform, tracked),
            Kind::Shell => shell::Shell::manifests(platform, tracked),
        }
    }

    pub(crate) fn default_checks(self) -> Vec<&'static str> {
        match self {
            Kind::Cargo => cargo::Cargo::default_checks(),
            Kind::Pnpm => pnpm::Pnpm::default_checks(),
            Kind::Shell => shell::Shell::default_checks(),
        }
    }

    pub(crate) fn root_checks(
        self,
        read: &dyn Fn(&str) -> Result<Option<String>, CIError>,
    ) -> RootChecks {
        match self {
            Kind::Cargo => cargo::Cargo::root_checks(read),
            Kind::Pnpm => pnpm::Pnpm::root_checks(read),
            Kind::Shell => shell::Shell::root_checks(read),
        }
    }

    fn version(self, platform: &impl Platform) -> Result<Option<String>, CIError> {
        match self {
            Kind::Cargo => cargo::Cargo::version(platform),
            Kind::Pnpm => pnpm::Pnpm::version(platform),
            Kind::Shell => shell::Shell::version(platform),
        }
    }

    pub(crate) fn setup(self) -> Vec<Step> {
        match self {
            Kind::Cargo => cargo::Cargo::setup(),
            Kind::Pnpm => pnpm::Pnpm::setup(),
            Kind::Shell => shell::Shell::setup(),
        }
    }

    pub(crate) fn project(
        self,
        dir: &str,
        config: TypeConfig,
        details: Vec<Details>,
        path_exists: &dyn Fn(&str) -> bool,
    ) -> Result<Box<dyn ProjectType>, CIError> {
        Ok(match self {
            Kind::Cargo => Box::new(cargo::Cargo::new(dir, config, details, path_exists)?),
            Kind::Pnpm => Box::new(pnpm::Pnpm::new(dir, config, details, path_exists)?),
            Kind::Shell => Box::new(shell::Shell::new(dir, config, details, path_exists)?),
        })
    }
}

/// As in `[defaults.<kind>]`.
impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Kind::Cargo => "cargo",
            Kind::Pnpm => "pnpm",
            Kind::Shell => "shell",
        })
    }
}

/// The version of the repository's releases, defined by one of the project types in the root.
pub(crate) fn version(platform: &impl Platform) -> Result<String, CIError> {
    for kind in Kind::ALL {
        if let Some(version) = kind.version(platform)? {
            return Ok(version);
        }
    }

    Err("no project type defines the repository version in the root".into())
}

/// A script of the project run outside of the checks: a package.json script (pnpm) or a script
/// file (cargo, shell).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Hook {
    /// Run after the project's packages are published on a release.
    Release,
    /// Run on every push to main that affects the project (`AGCI_VERSION=dev`).
    Dev,
    /// Regenerates the generated sources (`agci` without arguments).
    Regenerate,
}

impl Hook {
    pub(crate) const ALL: [Hook; 3] = [Hook::Release, Hook::Dev, Hook::Regenerate];

    /// Of the script.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Hook::Release => "release",
            Hook::Dev => "release:dev",
            Hook::Regenerate => "regenerate",
        }
    }
}

/// Where the packages of a type are published.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Registry {
    CratesIo,
    Npm,
}

impl Registry {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Registry::CratesIo => "crates.io",
            Registry::Npm => "npm",
        }
    }

    /// Whether the version of the package is already in the registry.
    pub(crate) fn published(
        self,
        platform: &impl Platform,
        package: &str,
        version: &str,
    ) -> Result<bool, CIError> {
        let request = platform.request();
        let url = match self {
            Registry::CratesIo => format!("https://crates.io/api/v1/crates/{package}/{version}"),
            Registry::Npm => format!("https://registry.npmjs.org/{package}/{version}"),
        };

        match request
            .blocking_client()?
            .get(url)
            .header(USER_AGENT, "agci (https://github.com/agnesoft/agdb)")
            .send_blocking(request)
        {
            Ok(_) => Ok(true),
            Err(e) if e.http_status() == Some(404) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}

/// Publishing a package of a release.
pub(crate) struct Publish {
    pub(crate) registry: Registry,
    /// The version that ends up in the registry.
    pub(crate) version: String,
    pub(crate) steps: Vec<Step>,
}

pub(crate) fn no_checks(dir: &str) -> CIError {
    format!("'{dir}' has no checks, set them in agci.toml").into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::RequestOutput;
    use agplatform::test_platform::test_platform;

    #[test]
    fn published_versions_are_found_by_url() {
        let platform = test_platform();
        platform
            .request
            .set_response("", RequestOutput::status(404));
        platform.request.set_response(
            "GET https://crates.io/api/v1/crates/agdb/1.0.0",
            RequestOutput::status(200),
        );
        platform.request.set_response(
            "GET https://registry.npmjs.org/@agnesoft/agdb_api/1.0.0",
            RequestOutput::status(200),
        );

        assert!(
            Registry::CratesIo
                .published(&platform, "agdb", "1.0.0")
                .unwrap()
        );
        assert!(
            !Registry::CratesIo
                .published(&platform, "agdb", "2.0.0")
                .unwrap()
        );
        assert!(
            Registry::Npm
                .published(&platform, "@agnesoft/agdb_api", "1.0.0")
                .unwrap()
        );
        assert!(
            !Registry::Npm
                .published(&platform, "@agnesoft/agdb_api", "2.0.0")
                .unwrap()
        );
        assert_eq!(
            platform.request.call_details()[0].headers["user-agent"],
            "agci (https://github.com/agnesoft/agdb)"
        );
    }

    #[test]
    fn published_other_status_fails() {
        let platform = test_platform();
        platform
            .request
            .set_response("", RequestOutput::status(503));

        assert!(
            Registry::CratesIo
                .published(&platform, "agdb", "1.0.0")
                .is_err()
        );
    }
}
