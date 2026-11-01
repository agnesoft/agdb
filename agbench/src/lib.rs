mod bench;
mod config;
mod stats;
#[cfg(test)]
mod test_utilities;

use std::io::Write;
use std::process::ExitCode;
use std::time::Duration;
use std::time::Instant;

pub use crate::bench::Bench;
pub use crate::bench::Benchmark;
pub use crate::bench::IteratedBenchmark;
use crate::config::Config;

/// Runs the `benchmarks` with the options parsed from the command line and
/// prints the results to stdout. Intended as the body of `main` of a bench
/// target with `harness = false`, run by `cargo bench`. See [`Bench`].
///
/// Returns failure (2) without running the benchmarks if the options are
/// invalid or `--help` is given, and failure (1) if `--check` is given
/// and a target is slower than its [`Benchmark::fail_factor`] allows.
///
/// # Examples
///
/// ```rust,no_run
/// // benches/path.rs
/// use agbench::Bench;
/// use std::path::Path;
///
/// fn main() -> std::process::ExitCode {
///     agbench::main(&[exists])
/// }
///
/// fn exists(bench: &mut Bench) {
///     bench
///         .benchmark("path/exists", || Path::new("Cargo.toml"))
///         .target("std::fs", |path| std::fs::exists(path).unwrap())
///         .target("Path", |path| path.exists())
///         .run();
/// }
/// ```
pub fn main(benchmarks: &[fn(&mut Bench)]) -> ExitCode {
    let origin = Instant::now();

    run(
        std::env::args().skip(1).collect(),
        &mut || origin.elapsed(),
        &mut std::io::stdout().lock(),
        benchmarks,
    )
}

fn run(
    args: Vec<String>,
    clock: &mut dyn FnMut() -> Duration,
    out: &mut dyn Write,
    benchmarks: &[fn(&mut Bench)],
) -> ExitCode {
    match Config::parse(args) {
        Ok(config) => {
            let mut bench = Bench::new(config, clock, out);

            for benchmark in benchmarks {
                benchmark(&mut bench);
            }

            bench.finish()
        }
        Err(error) => {
            writeln!(out, "{error}").expect("output should be writable");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::*;

    mod main {
        use super::*;

        #[test]
        fn main_should_run_benchmarks_with_command_line_args() {
            // WHY: with the default calibration a 10 ms iteration is
            // measured once per sample, keeping the run short.
            fn sleeps(bench: &mut Bench) {
                bench
                    .benchmark("sleeps", || ())
                    .target("a", |_| std::thread::sleep(Duration::from_millis(10)))
                    .run();
            }

            let _ = main(&[sleeps]);
        }
    }

    mod run {
        use super::*;

        #[test]
        fn run_should_print_results_of_benchmarks() {
            let (code, out) = run_with(&["--iters", "1", "--samples", "1"], &[read]);

            assert_eq!(code, ExitCode::SUCCESS);
            assert_eq!(
                out,
                format!(
                    "{HEADER}\n\
                     fs/read                          std                       1   100.0 ns\n\
                     \x20                                agplatform                    150.0 ns    +50.0%  [+50.0% .. +50.0%]\n\
                     \x20                                in_memory                      50.0 ns    -50.0%  [-50.0% .. -50.0%]\n"
                )
            );
        }

        #[test]
        fn run_invalid_args_should_fail_without_running_benchmarks() {
            let (code, out) = run_with(&["--samples"], &[read_and_write]);

            assert_eq!(code, ExitCode::from(2));
            assert!(
                out.starts_with("Option --samples requires a positive integer\n"),
                "{out}"
            );
            assert!(LOG.take().is_empty());
        }

        #[test]
        fn run_slower_than_fail_factor_should_fail() {
            let (code, out) = run_with(&["--iters", "1", "--samples", "1", "--check"], &[strict]);

            assert_eq!(code, ExitCode::FAILURE);
            assert!(
                out.ends_with(&format!(
                    "{:<32} {:<16} {:>10} {:>10} {:>9}  [+50.0% .. +50.0%]  FAIL > 1.4x\n\
                     {:<32} {:<16} {:>10} {:>10} {:>9}  [-50.0% .. -50.0%]\n\
                     FAILED: 1 target(s) slower than their fail factor\n",
                    "",
                    "agplatform",
                    "",
                    "150.0 ns",
                    "+50.0%",
                    "",
                    "in_memory",
                    "",
                    "50.0 ns",
                    "-50.0%"
                )),
                "{out}"
            );
        }

        #[test]
        fn run_within_fail_factor_or_unchecked_should_succeed() {
            let fixed = ["--iters", "1", "--samples", "1"];
            let checked = [&fixed[..], &["--check"]].concat();

            for (args, benchmark) in [
                (&checked, loose as fn(&mut Bench)),
                (&checked, read),
                (&fixed.to_vec(), strict),
            ] {
                let (code, out) = run_with(args, &[benchmark]);

                assert_eq!(code, ExitCode::SUCCESS, "args: {args:?}");
                assert!(!out.contains("FAIL"), "{out}");
            }
        }

        #[test]
        fn run_should_print_header_once() {
            let (_, out) = run_with(&["--iters", "1", "--samples", "1"], &[read, read_and_write]);

            assert_eq!(out.matches(HEADER).count(), 1, "{out}");
        }
    }
}
