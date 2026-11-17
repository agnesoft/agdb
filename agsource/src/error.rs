use std::path::PathBuf;

/// A problem with one source file. Lines and columns are 1-based.
#[derive(Debug, Clone)]
pub enum Error {
    /// The file or directory could not be read.
    Read {
        path: PathBuf,
        error: agplatform::Error,
    },
    /// The file is not valid Rust.
    Parse {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },
    /// No file exists for `mod name;` in `path`.
    ModuleNotFound {
        path: PathBuf,
        line: usize,
        name: String,
        candidates: Vec<PathBuf>,
    },
    /// Both `name.rs` and `name/mod.rs` exist for `mod name;` in `path`.
    ModuleAmbiguous {
        path: PathBuf,
        line: usize,
        name: String,
        candidates: Vec<PathBuf>,
    },
    /// `mod name;` in `path` loads `file`, which an enclosing module already
    /// comes from.
    CircularModule {
        path: PathBuf,
        line: usize,
        name: String,
        file: PathBuf,
    },
    /// The cfg `name` in `path` is not known to the
    /// [`CfgSet`](crate::CfgSet) and taken as false.
    UnknownCfg {
        path: PathBuf,
        line: usize,
        name: String,
    },
    /// The import `name` in `path` names nothing.
    UnresolvedImport {
        path: PathBuf,
        line: usize,
        name: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let join = |paths: &[PathBuf], separator: &str| {
            paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(separator)
        };

        match self {
            Error::Read { path, error } => write!(f, "{}: cannot read: {error}", path.display()),
            Error::Parse {
                path,
                line,
                column,
                message,
            } => write!(f, "{}:{line}:{column}: {message}", path.display()),
            Error::ModuleNotFound {
                path,
                line,
                name,
                candidates,
            } => write!(
                f,
                "{}:{line}: file not found for module `{name}`: {}",
                path.display(),
                join(candidates, " or ")
            ),
            Error::ModuleAmbiguous {
                path,
                line,
                name,
                candidates,
            } => write!(
                f,
                "{}:{line}: file for module `{name}` found at both {}",
                path.display(),
                join(candidates, " and ")
            ),
            Error::CircularModule {
                path,
                line,
                name,
                file,
            } => write!(
                f,
                "{}:{line}: circular module `{name}`: {} is already an enclosing module",
                path.display(),
                file.display()
            ),
            Error::UnknownCfg { path, line, name } => write!(
                f,
                "{}:{line}: unknown cfg `{name}`, taken as false",
                path.display()
            ),
            Error::UnresolvedImport { path, line, name } => {
                write!(f, "{}:{line}: unresolved import `{name}`", path.display())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod error {
        use super::*;

        mod std_fmt_display {
            use super::*;

            #[test]
            fn fmt_should_prefix_the_location() {
                let not_found = Error::ModuleNotFound {
                    path: "src/lib.rs".into(),
                    line: 3,
                    name: "a".into(),
                    candidates: vec!["src/a.rs".into(), "src/a/mod.rs".into()],
                };
                let ambiguous = Error::ModuleAmbiguous {
                    path: "src/lib.rs".into(),
                    line: 4,
                    name: "b".into(),
                    candidates: vec!["src/b.rs".into(), "src/b/mod.rs".into()],
                };
                let circular = Error::CircularModule {
                    path: "src/c.rs".into(),
                    line: 1,
                    name: "c".into(),
                    file: "src/c.rs".into(),
                };
                let parse = Error::Parse {
                    path: "src/d.rs".into(),
                    line: 2,
                    column: 5,
                    message: "expected `;`".into(),
                };
                let unknown = Error::UnknownCfg {
                    path: "src/f.rs".into(),
                    line: 7,
                    name: "unix".into(),
                };
                let unresolved = Error::UnresolvedImport {
                    path: "src/g.rs".into(),
                    line: 8,
                    name: "a::b".into(),
                };
                let read = Error::Read {
                    path: "src/e.rs".into(),
                    error: agplatform::Error::fs("no such file"),
                };

                assert_eq!(
                    not_found.to_string(),
                    "src/lib.rs:3: file not found for module `a`: src/a.rs or src/a/mod.rs"
                );
                assert_eq!(
                    ambiguous.to_string(),
                    "src/lib.rs:4: file for module `b` found at both src/b.rs and src/b/mod.rs"
                );
                assert_eq!(
                    circular.to_string(),
                    "src/c.rs:1: circular module `c`: src/c.rs is already an enclosing module"
                );
                assert_eq!(parse.to_string(), "src/d.rs:2:5: expected `;`");
                assert_eq!(read.to_string(), "src/e.rs: cannot read: [Fs] no such file");
                assert_eq!(
                    unknown.to_string(),
                    "src/f.rs:7: unknown cfg `unix`, taken as false"
                );
                assert_eq!(
                    unresolved.to_string(),
                    "src/g.rs:8: unresolved import `a::b`"
                );
            }
        }
    }
}
