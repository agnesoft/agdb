use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

#[cfg(any(feature = "process", feature = "request"))]
use crate::Result;
#[cfg(all(feature = "tokio", any(feature = "process", feature = "request")))]
use crate::interceptor::AsyncExecutor;
#[cfg(any(feature = "process", feature = "request"))]
use crate::interceptor::{
    BlockingExecutor, Interceptor, Interceptors, Outcome, Run, Sink, Subject,
};

/// Returns `path` resolved against `current_dir` with the `.` and `..`
/// components removed lexically, never going above the root. The leading
/// `..` of a relative result are kept, like by the fs.
pub(crate) fn resolve_path(current_dir: &Path, path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();

    for component in current_dir.join(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(
                    resolved.components().next_back(),
                    None | Some(Component::ParentDir)
                ) {
                    resolved.push(component);
                } else {
                    resolved.pop();
                }
            }
            component => resolved.push(component),
        }
    }

    resolved
}

/// [internal]
///
/// The interceptors of the runs of a process or request, changed through a
/// shared reference for the runs starting afterwards, and running the runs
/// through them, or through their own, see [`OwnInterceptors`].
#[cfg(any(feature = "process", feature = "request"))]
pub(crate) struct Runs<S: Subject>(std::sync::RwLock<Interceptors<S>>);

/// [internal]
///
/// The interceptors of a run instead of those of its process or request,
/// set on the run as a value no interceptor can name, as they are taken
/// once before the run starts.
#[cfg(any(feature = "process", feature = "request"))]
pub(crate) struct OwnInterceptors<S: Subject>(pub(crate) Interceptors<S>);

#[cfg(any(feature = "process", feature = "request"))]
impl<S: Subject> Runs<S> {
    pub(crate) fn new(interceptors: Interceptors<S>) -> Self {
        Self(std::sync::RwLock::new(interceptors))
    }

    /// Returns the interceptors, shared with the runs started before.
    pub(crate) fn interceptors(&self) -> Interceptors<S> {
        self.0
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn clear_interceptors(&self) {
        *self.write() = Interceptors::new();
    }

    pub(crate) fn add_interceptor(&self, interceptor: impl Interceptor<S>) {
        self.write().push(interceptor);
    }

    /// Runs `run` by `executor` through its interceptors, passing its output
    /// to `sink`, if any.
    pub(crate) fn run_blocking<E: BlockingExecutor<Subject = S>>(
        &self,
        executor: &E,
        run: &mut Run<S>,
        sink: Option<&mut dyn Sink<S>>,
    ) -> Result<S::Output> {
        self.interceptors_of(run).run_blocking(executor, run, sink)
    }

    /// Runs `run` like [`Self::run_blocking`] asynchronously.
    #[cfg(feature = "tokio")]
    pub(crate) async fn run<E: AsyncExecutor<Subject = S>>(
        &self,
        executor: &E,
        run: &mut Run<S>,
        sink: Option<&mut dyn Sink<S>>,
    ) -> Result<S::Output> {
        self.interceptors_of(run).run(executor, run, sink).await
    }

    /// Returns the own interceptors of `run`, if any, else these.
    fn interceptors_of(&self, run: &Run<S>) -> Interceptors<S> {
        run.get::<OwnInterceptors<S>>()
            .map_or_else(|| self.interceptors(), |own| own.0.clone())
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, Interceptors<S>> {
        self.0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// [internal]
///
/// Returns the `result` of a run with an output that is not a success as
/// the error of `failure`, at the boundary of a process or request.
#[cfg(any(feature = "process", feature = "request"))]
pub(crate) fn conclude<O: Outcome>(
    result: Result<O>,
    failure: impl FnOnce(O) -> crate::Error,
) -> Result<O> {
    match result {
        Ok(output) if !output.success() => Err(failure(output)),
        result => result,
    }
}

/// [internal]
///
/// Returns the 64-bit FNV-1a hash of `values`, each written with whether
/// it is set and its length so that different lists hash differently, the
/// same on every platform and Rust version, unlike
/// [`std::hash::DefaultHasher`].
#[cfg(feature = "process")]
pub(crate) fn fnv1a<'a>(values: impl IntoIterator<Item = Option<&'a [u8]>>) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;

    let hash = |hash: u64, bytes: &[u8]| {
        bytes.iter().fold(hash, |hash, &byte| {
            (hash ^ u64::from(byte)).wrapping_mul(PRIME)
        })
    };

    values.into_iter().fold(OFFSET, |state, value| match value {
        Some(value) => hash(
            hash(hash(state, &[1]), &(value.len() as u64).to_le_bytes()),
            value,
        ),
        None => hash(state, &[0]),
    })
}

/// Removes surrounding single or double quotes from a string slice, if present.
/// Leading and trailing whitespace is trimmed before checking for quotes.
pub(crate) fn unquote<T: AsRef<str> + ?Sized>(str: &T) -> &str {
    let str_ref = str.as_ref().trim();
    let mut chars = str_ref.chars();

    if let Some(first) = chars.next()
        && let Some(last) = chars.last()
        && ((first == '\'' && last == '\'') || (first == '"' && last == '"'))
    {
        return &str_ref[1..str_ref.len() - 1];
    }

    str_ref
}

/// Displays the last `lines` of `text` lossily, at most `lines` KiB of
/// them, preceded by the number of the lines or the bytes omitted if any.
#[cfg(any(feature = "process", feature = "request"))]
pub(crate) struct Tail<'a> {
    pub(crate) text: &'a [u8],
    pub(crate) lines: usize,
}

#[cfg(any(feature = "process", feature = "request"))]
impl std::fmt::Display for Tail<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const LINE_BYTES: usize = 1024;

        // WHY: found from the end so that only the shown part is scanned
        // for lines, the omitted ones are only counted.
        let text = self.text;
        let lines_start = match self.lines.checked_sub(1) {
            Some(last) => text
                .strip_suffix(b"\n")
                .unwrap_or(text)
                .iter()
                .enumerate()
                .rev()
                .filter(|&(_, &byte)| byte == b'\n')
                .nth(last)
                .map_or(0, |(newline, _)| newline + 1),
            None => text.len(),
        };
        let max_bytes = self.lines.saturating_mul(LINE_BYTES);

        let start = if text.len() - lines_start > max_bytes {
            let start = text.len() - max_bytes;
            // WHY: a UTF-8 sequence cut at the start would be replaced. It
            // has at most 3 continuation bytes, more are invalid anyway.
            let start = start
                + text[start..]
                    .iter()
                    .take(3)
                    .take_while(|&&byte| byte & 0xC0 == 0x80)
                    .count();

            writeln!(f, "({start} bytes omitted)")?;
            start
        } else {
            let omitted = text[..lines_start]
                .iter()
                .filter(|&&byte| byte == b'\n')
                .count()
                + usize::from(
                    lines_start == text.len() && !text.is_empty() && !text.ends_with(b"\n"),
                );

            if omitted > 0 {
                writeln!(f, "({omitted} lines omitted)")?;
            }

            lines_start
        };

        text[start..].utf8_chunks().try_for_each(|chunk| {
            f.write_str(chunk.valid())?;

            if chunk.invalid().is_empty() {
                Ok(())
            } else {
                f.write_str("\u{FFFD}")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod resolve_path {
        use super::*;

        #[cfg(unix)]
        const ROOT: &str = "/";
        #[cfg(windows)]
        const ROOT: &str = "C:\\";

        #[test]
        fn resolve_path_should_resolve_relative_path_against_current_dir() {
            let current_dir = Path::new(ROOT).join("cwd");

            for (path, expected) in [
                ("a", "cwd/a"),
                ("a/./b", "cwd/a/b"),
                ("./a/../b/", "cwd/b"),
                ("", "cwd"),
                ("..", ""),
                ("../../..", ""),
                ("../a", "a"),
            ] {
                assert_eq!(
                    resolve_path(&current_dir, Path::new(path)),
                    Path::new(ROOT).join(expected),
                    "path: {path}"
                );
            }
        }

        #[test]
        fn resolve_path_should_normalize_absolute_path() {
            let current_dir = Path::new(ROOT).join("cwd");
            let path = Path::new(ROOT).join("a/./b/../c");

            assert_eq!(
                resolve_path(&current_dir, &path),
                Path::new(ROOT).join("a/c")
            );
        }

        #[test]
        fn resolve_path_empty_current_dir_should_keep_path_relative() {
            for (path, expected) in [
                ("./a", "a"),
                ("a/./b/../c", "a/c"),
                ("../a", "../a"),
                ("../../a/..", "../.."),
            ] {
                assert_eq!(
                    resolve_path(Path::new(""), Path::new(path)),
                    Path::new(expected),
                    "path: {path}"
                );
            }
        }

        #[cfg(windows)]
        #[test]
        fn resolve_path_should_resolve_windows_prefixes() {
            assert_eq!(
                resolve_path(Path::new("C:\\cwd"), Path::new("\\a\\..\\b")),
                Path::new("C:\\b")
            );
            assert_eq!(
                resolve_path(Path::new("C:\\cwd"), Path::new("D:\\a")),
                Path::new("D:\\a")
            );
        }
    }

    #[cfg(feature = "process")]
    mod fnv1a {
        use super::*;

        #[test]
        fn fnv1a_should_hash_values_with_presence_and_length() {
            let hashes = [
                fnv1a([]),
                fnv1a([None]),
                fnv1a([Some(b"".as_slice())]),
                fnv1a([Some(b"ab".as_slice())]),
                fnv1a([Some(b"a".as_slice()), Some(b"b")]),
            ];

            assert_eq!(fnv1a([]), 0xcbf2_9ce4_8422_2325);
            assert_eq!(
                hashes
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                hashes.len(),
                "{hashes:x?}"
            );
        }
    }

    #[cfg(any(feature = "process", feature = "request"))]
    mod runs {
        use super::*;
        use crate::interceptor::tests::*;
        use std::sync::Arc;

        mod add_interceptor {
            use super::*;

            #[test]
            fn add_interceptor_should_not_change_interceptors_taken_before() {
                let runs = Runs::new(Interceptors::new());
                let before = runs.interceptors();

                runs.add_interceptor(Passing);

                assert!(before.is_empty());
                assert_eq!(runs.interceptors().len(), 1);
            }
        }

        mod clear_interceptors {
            use super::*;

            #[test]
            fn clear_interceptors_should_remove_all_interceptors() {
                let runs = Runs::new(chain(vec![Arc::new(Passing), Arc::new(Passing)]));

                runs.clear_interceptors();

                assert!(runs.interceptors().is_empty());
            }
        }

        mod run_blocking {
            use super::*;

            #[test]
            fn run_blocking_should_run_through_interceptors() {
                let events = Events::default();
                let runs = Runs::new(chain(vec![Arc::new(log("a", &events))]));

                runs.run_blocking(&Script::default(), &mut task("x"), None)
                    .unwrap();

                assert_eq!(recorded(&events).last().unwrap(), "a handled 1 Some(0)");
            }

            #[test]
            fn run_blocking_with_own_interceptors_should_use_them_instead() {
                // GIVEN runs with an interceptor and a run with its own one
                let events = Events::default();
                let runs = Runs::new(chain(vec![Arc::new(log("a", &events))]));
                let mut run =
                    task("x").with(OwnInterceptors(chain(vec![Arc::new(log("b", &events))])));

                // WHEN running it
                runs.run_blocking(&Script::default(), &mut run, None)
                    .unwrap();

                // THEN only its own interceptor sees it
                assert_eq!(
                    recorded(&events),
                    ["b chunk 1 ", "b chunk 2 ", "b handled 1 Some(0)"]
                );
            }
        }
    }

    #[cfg(any(feature = "process", feature = "request"))]
    mod conclude {
        use super::*;
        use crate::interceptor::tests::*;

        fn failed(output: Done) -> crate::Error {
            crate::Error::process(format!("failed with {}", output.status))
        }

        #[test]
        fn conclude_should_keep_success_and_error() {
            let success = Done {
                success: true,
                ..done(1)
            };

            assert_eq!(conclude(Ok(success.clone()), failed).unwrap(), success);
            assert_eq!(
                conclude(Err(crate::Error::process("io")), failed)
                    .unwrap_err()
                    .description(),
                "io"
            );
        }

        #[test]
        fn conclude_failure_should_return_error_of_failure() {
            let error = conclude(Ok(done(3)), failed).unwrap_err();

            assert_eq!(error.description(), "failed with 3");
        }
    }

    mod unquote {
        use super::*;

        #[test]
        fn unquote_should_strip_matching_quotes() {
            for (input, expected) in [
                ("'test'", "test"),
                ("\"test\"", "test"),
                ("'\"test\"'", "\"test\""),
                ("\"'test'\"", "'test'"),
                ("''", ""),
                ("' a '", " a "),
            ] {
                assert_eq!(unquote(input), expected, "input: {input}");
            }
        }

        #[test]
        fn unquote_should_return_unquoted_input_unchanged() {
            for input in ["test", "'test", "test\"", "'", "\"", "'test\"", "\"test'"] {
                assert_eq!(unquote(input), input, "input: {input}");
            }
        }

        #[test]
        fn unquote_should_trim_whitespace_before_checking() {
            assert_eq!(unquote(" 'test' "), "test");
        }
    }

    #[cfg(any(feature = "process", feature = "request"))]
    mod tail {
        use super::*;

        fn tail(text: &str, lines: usize) -> String {
            Tail {
                text: text.as_bytes(),
                lines,
            }
            .to_string()
        }

        #[test]
        fn tail_should_keep_up_to_lines() {
            for text in ["", "a", "a\nb\n", &"x\n".repeat(10)] {
                assert_eq!(tail(text, 10), text, "text: {text:?}");
            }
        }

        #[test]
        fn tail_more_than_lines_should_keep_last_lines() {
            let text = (1..=11).map(|line| format!("{line}\n")).collect::<String>();

            assert_eq!(
                tail(&text, 10),
                "(1 lines omitted)\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n"
            );
            assert_eq!(tail(&text, 2), "(9 lines omitted)\n10\n11\n");
        }

        #[test]
        fn tail_should_count_lines_as_str_lines() {
            for (text, lines, expected) in [
                ("a\nb", 1, "(1 lines omitted)\nb"),
                ("a\n\nb\n", 2, "(1 lines omitted)\n\nb\n"),
                ("\n\n", 1, "(1 lines omitted)\n\n"),
                ("a\nb", 0, "(2 lines omitted)\n"),
                ("a\nb\n", 0, "(2 lines omitted)\n"),
                ("", 0, ""),
            ] {
                assert_eq!(
                    tail(text, lines),
                    expected,
                    "text: {text:?}, lines: {lines}"
                );
            }
        }

        #[test]
        fn tail_longer_than_lines_kib_should_keep_last_bytes() {
            let text = format!("a\n{}", "b".repeat(2048));

            assert_eq!(
                tail(&text, 1),
                format!("(1026 bytes omitted)\n{}", "b".repeat(1024))
            );
        }

        #[test]
        fn tail_cut_in_utf8_sequence_should_start_at_next_char() {
            let text = format!("{}a", "é".repeat(512));

            assert_eq!(
                tail(&text, 1),
                format!("(2 bytes omitted)\n{}a", "é".repeat(511))
            );
        }

        #[test]
        fn tail_failing_formatter_should_fail() {
            struct Writer {
                writes: usize,
                written: usize,
            }

            impl std::fmt::Write for Writer {
                fn write_str(&mut self, _: &str) -> std::fmt::Result {
                    self.written += 1;
                    (self.written <= self.writes)
                        .then_some(())
                        .ok_or(std::fmt::Error)
                }
            }

            for text in [
                [b"a\n".as_slice(), &b"b".repeat(2048), b"\xFF"].concat(),
                b"a\nb\xFF\n".to_vec(),
            ] {
                let tail = Tail {
                    text: &text,
                    lines: 1,
                };
                let mut counter = Writer {
                    writes: usize::MAX,
                    written: 0,
                };
                std::fmt::write(&mut counter, format_args!("{tail}")).unwrap();

                for writes in 0..counter.written {
                    let mut writer = Writer { writes, written: 0 };

                    assert!(
                        std::fmt::write(&mut writer, format_args!("{tail}")).is_err(),
                        "text: {text:?}, writes: {writes}"
                    );
                }
            }
        }

        #[test]
        fn tail_cut_in_invalid_utf8_should_keep_last_bytes_lossily() {
            // GIVEN a binary output of continuation bytes only, no UTF-8 sequence
            let text = [0x80].repeat(2048);

            // WHEN keeping its last line, longer than 1 KiB
            let tail = Tail {
                text: &text,
                lines: 1,
            }
            .to_string();

            // THEN the last KiB is shown lossily rather than dropped
            // WHY: a UTF-8 sequence has at most 3 continuation bytes, so no more
            // can belong to a sequence cut at the start.
            let (omitted, shown) = tail.split_once('\n').unwrap();
            assert!(
                omitted == "(1024 bytes omitted)" || omitted == "(1027 bytes omitted)",
                "{omitted}"
            );
            assert!(shown.chars().count() >= 1021, "{tail:?}");
            assert!(shown.chars().all(|c| c == '\u{FFFD}'), "{tail:?}");
        }

        #[test]
        fn tail_should_replace_invalid_utf8() {
            assert_eq!(
                Tail {
                    text: b"a\xFFb\xC3",
                    lines: 1,
                }
                .to_string(),
                "a\u{FFFD}b\u{FFFD}"
            );
        }
    }
}
