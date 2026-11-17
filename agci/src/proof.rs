use crate::CIError;
use crate::check::Check;
use crate::config::Os;
use crate::git;
use crate::utilities;
use agplatform::Fs;
use agplatform::Platform;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

/// Evidence that a check of a project passed on this OS: an empty file in the git directory
/// (shared by the worktrees, not committed, in the pipeline restored from and saved to the actions
/// cache), so `check` skips the check until the project's fingerprint or the check's steps change.
pub(crate) struct Proofs {
    dir: PathBuf,
    /// Proofs are only saved, never used.
    force: bool,
}

impl Proofs {
    pub(crate) fn new(platform: &impl Platform, force: bool) -> Result<Self, CIError> {
        Ok(Self {
            dir: git::common_dir(platform)?.join("agci").join("proofs"),
            force,
        })
    }

    /// Removes all proofs.
    pub(crate) fn clean(&self, platform: &impl Platform) -> Result<(), CIError> {
        if platform.fs().exists(&self.dir)? {
            platform.fs().remove_dir_all(&self.dir)?;
        }

        Ok(())
    }

    /// Whether the check passed before with the fingerprint, which keeps the proof from pruning.
    pub(crate) fn proven(&self, platform: &impl Platform, key: &str) -> bool {
        !self.force
            && platform
                .fs()
                .set_modified(self.dir.join(key), SystemTime::now())
                .is_ok()
    }

    pub(crate) fn save(&self, platform: &impl Platform, key: &str) -> Result<(), CIError> {
        platform.fs().create_dir_all(&self.dir)?;
        platform.fs().write(self.dir.join(key), "")?;
        Ok(())
    }

    /// Removes the proofs not used for `max_age`.
    pub(crate) fn prune(
        &self,
        platform: &impl Platform,
        now: SystemTime,
        max_age: Duration,
    ) -> Result<(), CIError> {
        let Ok(entries) = platform.fs().read_dir(&self.dir) else {
            return Ok(());
        };

        for entry in entries {
            let path = entry?.path().to_path_buf();
            let modified = platform.fs().metadata(&path)?.modified;

            if now.duration_since(modified).unwrap_or_default() > max_age {
                platform.fs().remove_file(&path)?;
            }
        }

        Ok(())
    }
}

/// The proof of the check: the project, check and OS to read, and a hash of the project's
/// fingerprint and the check's steps (programs, arguments, environment and directories).
/// The installed toolchains are not part of it, they are kept evergreen.
pub(crate) fn key(project: &str, check: &Check, fingerprint: &str) -> String {
    key_on(project, check, fingerprint, Os::CURRENT)
}

fn key_on(project: &str, check: &Check, fingerprint: &str, os: Os) -> String {
    let mut text = format!("{fingerprint}\n");

    for step in &check.steps {
        text += &format!("{}> {step}\n", step.dir);
    }

    format!(
        "{}-{}-{os}-{}",
        project.replace('/', "_"),
        check.name.replace([':', '/'], "_"),
        utilities::hash(text.as_bytes())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::step;
    use agplatform::test_platform::test_platform;

    fn proofs(force: bool) -> Proofs {
        Proofs {
            dir: PathBuf::from("/repo/.git/agci/proofs"),
            force,
        }
    }

    fn lint(args: &[&str]) -> Check {
        Check {
            name: "lint".to_string(),
            steps: vec![step("cargo", args, "")],
        }
    }

    #[test]
    fn saved_proofs_are_proven_unless_forced() {
        let platform = test_platform();

        proofs(false).save(&platform, "a").unwrap();

        assert!(proofs(false).proven(&platform, "a"));
        assert!(!proofs(false).proven(&platform, "b"));
        assert!(!proofs(true).proven(&platform, "a"));
    }

    #[test]
    fn prune_removes_the_proofs_not_used_recently() {
        let platform = test_platform();
        let proofs = proofs(false);
        let now = SystemTime::now();
        proofs.save(&platform, "old").unwrap();
        proofs.save(&platform, "used").unwrap();
        proofs.save(&platform, "new").unwrap();
        let max_age = Duration::from_secs(60);
        let old = now - max_age - Duration::from_secs(1);
        platform
            .fs()
            .set_modified(proofs.dir.join("old"), old)
            .unwrap();
        platform
            .fs()
            .set_modified(proofs.dir.join("used"), old)
            .unwrap();
        assert!(proofs.proven(&platform, "used"));

        proofs.prune(&platform, now, max_age).unwrap();

        assert!(!proofs.proven(&platform, "old"));
        assert!(proofs.proven(&platform, "used"));
        assert!(proofs.proven(&platform, "new"));
    }

    #[test]
    fn prune_without_proofs_does_nothing() {
        proofs(false)
            .prune(&test_platform(), SystemTime::now(), Duration::ZERO)
            .unwrap();
    }

    #[test]
    fn clean_removes_all_proofs() {
        let platform = test_platform();
        let proofs = proofs(false);
        proofs.clean(&platform).unwrap();
        proofs.save(&platform, "a").unwrap();

        proofs.clean(&platform).unwrap();

        assert!(!proofs.proven(&platform, "a"));
        assert!(!platform.fs().exists(&proofs.dir).unwrap());
    }

    #[test]
    fn key_changes_with_the_fingerprint_and_the_steps() {
        let check = lint(&["clippy"]);
        let key = |check: &Check, fingerprint: &str| {
            key_on("agdb_api/php", check, fingerprint, Os::Windows)
        };
        let base = key(&check, "1a2b");

        assert!(base.starts_with("agdb_api_php-lint-windows-"));
        assert_eq!(base, key(&check, "1a2b"));
        assert_ne!(base, key(&check, "3c4d"));
        assert_ne!(base, key(&lint(&["clippy", "-D"]), "1a2b"));
    }
}
