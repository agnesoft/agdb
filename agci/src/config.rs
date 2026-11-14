use serde::Deserialize;
use serde::Serialize;

/// `[package.metadata.agci]` in `Cargo.toml` or `"agci"` in `package.json`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Config {
    /// Rust: built-in checks (see `RUST_CHECKS`). Node: package.json scripts, run in the given order.
    pub(crate) checks: Vec<String>,
    /// Project directories or repository paths that affect this project in addition to the manifest dependencies.
    pub(crate) depends: Vec<String>,
    /// Defaults to `["linux"]`.
    pub(crate) os: Vec<Os>,
    /// Toolchains needed in addition to the project's own language.
    pub(crate) toolchains: Vec<Toolchain>,
    /// Rust only: run tests under `cargo llvm-cov` with this minimum line coverage, 100 also requires all regions and functions.
    pub(crate) coverage: Option<u8>,
    /// Rust only: features used instead of `--all-features`.
    pub(crate) features: Option<String>,
    /// Rust only: additionally lint each combination with `--no-default-features`.
    pub(crate) feature_combinations: Vec<String>,
    /// Rust only: extra shell commands run in the project directory after the checks.
    pub(crate) run: Vec<String>,
    /// Run on a release (no tag for the version yet). Rust: `crates`. Node: package.json scripts.
    pub(crate) publish: Vec<String>,
    /// Node only: package.json scripts run on every push to main that affects the project.
    pub(crate) publish_dev: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Os {
    Linux,
    Windows,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Toolchain {
    Node,
    Php,
    Rust,
}

pub(crate) const RUST_CHECKS: [&str; 6] = ["format", "lint", "testcheck", "doc", "bench", "test"];
pub(crate) const RUST_PUBLISH: [&str; 1] = ["crates"];

impl Config {
    pub(crate) fn os(&self) -> Vec<Os> {
        if self.os.is_empty() {
            vec![Os::Linux]
        } else {
            self.os.clone()
        }
    }

    pub(crate) fn has_rust_only_fields(&self) -> bool {
        self.coverage.is_some()
            || self.features.is_some()
            || !self.feature_combinations.is_empty()
            || !self.run.is_empty()
    }
}
