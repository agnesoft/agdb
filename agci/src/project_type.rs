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
use serde::Deserialize;

/// How the projects of a type (the tool driving them) are found, checked and released. The static
/// functions are of the type, reached via its [`Kind`]. Each implementation holds the project's
/// checks and hooks resolved and validated when it is built.
pub(crate) trait ProjectType: std::fmt::Debug {
    /// Its manifests among the tracked files, what only it needs of them in their `details`.
    fn manifests(tracked: &[&str]) -> Result<Vec<Manifest>, CIError>
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
    fn version() -> Result<Option<String>, CIError>
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

    /// Updates the versions in its manifests that are `from` to `to`.
    fn set_version(&self, _from: &str, _to: &str) -> Result<(), CIError> {
        Ok(())
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

    pub(crate) fn manifests(self, tracked: &[&str]) -> Result<Vec<Manifest>, CIError> {
        match self {
            Kind::Cargo => cargo::Cargo::manifests(tracked),
            Kind::Pnpm => pnpm::Pnpm::manifests(tracked),
            Kind::Shell => shell::Shell::manifests(tracked),
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

    fn version(self) -> Result<Option<String>, CIError> {
        match self {
            Kind::Cargo => cargo::Cargo::version(),
            Kind::Pnpm => pnpm::Pnpm::version(),
            Kind::Shell => shell::Shell::version(),
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
pub(crate) fn version() -> Result<String, CIError> {
    for kind in Kind::ALL {
        if let Some(version) = kind.version()? {
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
pub(crate) trait Registry {
    fn name(&self) -> &'static str;

    /// Whether the version of the package is already in the registry.
    fn published(&self, package: &str, version: &str) -> Result<bool, CIError>;
}

/// Publishing a package of a release.
pub(crate) struct Publish {
    pub(crate) registry: &'static dyn Registry,
    /// The version that ends up in the registry.
    pub(crate) version: String,
    pub(crate) steps: Vec<Step>,
}

pub(crate) fn no_checks(dir: &str) -> CIError {
    format!("'{dir}' has no checks, set them in agci.toml").into()
}
