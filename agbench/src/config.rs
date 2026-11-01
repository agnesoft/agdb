use std::time::Duration;

pub(crate) const USAGE: &str = "\
Usage: cargo bench [-p <crate>] [--bench <name>] -- [OPTIONS]

  --filter PATTERN         run only benchmarks whose name contains PATTERN,
                           repeatable to run those matching any of them
  --filter-target PATTERN  run only targets whose name contains PATTERN,
                           repeatable, the first one selected is the baseline
  --samples N              number of samples, each running every target
                           (default 30)
  --sample-ms MS           target duration of one sample when calibrating
                           (default 10)
  --iters N                fixed iterations per sample instead of calibrating
  --list                   list the benchmarks and their targets without
                           running them
  --check                  exit with failure (1) if a target is slower than
                           its benchmark's fail factor times the baseline in
                           at least three quarters of the samples

The `--bench` option passed by `cargo bench` is ignored.";

/// Benchmark run configuration parsed from the command line arguments.
#[derive(Debug, PartialEq)]
pub(crate) struct Config {
    pub(crate) benchmark_filters: Vec<String>,
    pub(crate) target_filters: Vec<String>,
    pub(crate) samples: usize,
    pub(crate) sample_time: Duration,
    pub(crate) iters: Option<u64>,
    pub(crate) list: bool,
    pub(crate) check: bool,
}

impl Config {
    /// Parses the arguments (without the program name). Returns the error
    /// followed by [`USAGE`] for unknown options or arguments, missing or
    /// invalid values and `--help`.
    pub(crate) fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut config = Config {
            benchmark_filters: Vec::new(),
            target_filters: Vec::new(),
            samples: 30,
            sample_time: Duration::from_millis(10),
            iters: None,
            list: false,
            check: false,
        };
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bench" => {}
                "--list" => config.list = true,
                "--filter" => config.benchmark_filters.push(pattern(&arg, args.next())?),
                "--filter-target" => config.target_filters.push(pattern(&arg, args.next())?),
                "--samples" => config.samples = positive(&arg, args.next())?,
                "--sample-ms" => {
                    config.sample_time = Duration::from_millis(positive(&arg, args.next())?)
                }
                "--iters" => config.iters = Some(positive(&arg, args.next())?),
                "--check" => config.check = true,
                "--help" | "-h" => return Err(USAGE.to_string()),
                _ if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {arg}\n\n{USAGE}"));
                }
                _ => return Err(format!("Unexpected argument: {arg}\n\n{USAGE}")),
            }
        }

        Ok(config)
    }

    /// Returns whether a benchmark of the given name should run.
    pub(crate) fn matches_benchmark(&self, name: &str) -> bool {
        matches(&self.benchmark_filters, name)
    }

    /// Returns whether a target of the given name should run.
    pub(crate) fn matches_target(&self, name: &str) -> bool {
        matches(&self.target_filters, name)
    }
}

/// Returns whether `name` contains any of the `filters` or there are none.
fn matches(filters: &[String], name: &str) -> bool {
    filters.is_empty() || filters.iter().any(|f| name.contains(f.as_str()))
}

/// Returns the value of `option`, which must be present.
fn pattern(option: &str, value: Option<String>) -> Result<String, String> {
    value.ok_or_else(|| format!("Option {option} requires a pattern\n\n{USAGE}"))
}

/// Parses the value of `option`, which must be a positive integer.
fn positive<T: std::str::FromStr + Default + PartialEq>(
    option: &str,
    value: Option<String>,
) -> Result<T, String> {
    value
        .and_then(|v| v.parse().ok())
        .filter(|v| *v != T::default())
        .ok_or_else(|| format!("Option {option} requires a positive integer\n\n{USAGE}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Config, String> {
        Config::parse(args.iter().map(|a| a.to_string()).collect())
    }

    mod config {
        use super::*;

        mod parse {
            use super::*;

            #[test]
            fn parse_without_args_should_return_defaults() {
                assert_eq!(
                    parse(&[]).unwrap(),
                    Config {
                        benchmark_filters: Vec::new(),
                        target_filters: Vec::new(),
                        samples: 30,
                        sample_time: Duration::from_millis(10),
                        iters: None,
                        list: false,
                        check: false,
                    }
                );
            }

            #[test]
            fn parse_should_read_options_and_filters() {
                assert_eq!(
                    parse(&[
                        "--filter",
                        "read",
                        "--samples",
                        "5",
                        "--filter-target",
                        "std",
                        "--sample-ms",
                        "20",
                        "--iters",
                        "100",
                        "--filter",
                        "write",
                        "--bench",
                        "--list",
                        "--check"
                    ])
                    .unwrap(),
                    Config {
                        benchmark_filters: vec!["read".to_string(), "write".to_string()],
                        target_filters: vec!["std".to_string()],
                        samples: 5,
                        sample_time: Duration::from_millis(20),
                        iters: Some(100),
                        list: true,
                        check: true,
                    }
                );
            }

            #[test]
            fn parse_help_should_return_usage() {
                for arg in ["--help", "-h"] {
                    assert_eq!(parse(&[arg]).unwrap_err(), USAGE, "arg: {arg}");
                }
            }

            #[test]
            fn parse_unknown_option_should_fail() {
                assert_eq!(
                    parse(&["--nocapture"]).unwrap_err(),
                    format!("Unknown option: --nocapture\n\n{USAGE}")
                );
            }

            #[test]
            fn parse_positional_argument_should_fail() {
                assert_eq!(
                    parse(&["read"]).unwrap_err(),
                    format!("Unexpected argument: read\n\n{USAGE}")
                );
            }

            #[test]
            fn parse_missing_pattern_should_fail() {
                for option in ["--filter", "--filter-target"] {
                    assert_eq!(
                        parse(&[option]).unwrap_err(),
                        format!("Option {option} requires a pattern\n\n{USAGE}"),
                        "option: {option}"
                    );
                }
            }

            #[test]
            fn parse_invalid_value_should_fail() {
                for args in [
                    &["--samples"][..],
                    &["--samples", "x"],
                    &["--samples", "0"],
                    &["--sample-ms", "-1"],
                    &["--iters", "0"],
                ] {
                    assert_eq!(
                        parse(args).unwrap_err(),
                        format!("Option {} requires a positive integer\n\n{USAGE}", args[0]),
                        "args: {args:?}"
                    );
                }
            }
        }

        mod matches_benchmark {
            use super::*;

            #[test]
            fn matches_benchmark_should_match_names_containing_any_filter() {
                let config = parse(&["--filter", "read", "--filter", "exists"]).unwrap();

                for (name, expected) in [
                    ("fs/read/64B", true),
                    ("fs/exists", true),
                    ("fs/write/64B", false),
                ] {
                    assert_eq!(config.matches_benchmark(name), expected, "name: {name}");
                }
            }
        }

        mod matches_target {
            use super::*;

            #[test]
            fn matches_target_should_match_names_containing_any_target_filter() {
                let config = parse(&["--filter-target", "plat", "--filter", "std"]).unwrap();

                for (name, expected) in [("agplatform", true), ("std", false)] {
                    assert_eq!(config.matches_target(name), expected, "name: {name}");
                }
            }
        }
    }

    mod matches {
        use super::*;

        #[test]
        fn matches_without_filters_should_match_everything() {
            assert!(matches(&[], "fs/read"));
        }
    }
}
