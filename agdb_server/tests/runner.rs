//! The test runner of this target (`harness = false`): it starts what the tests
//! share once and then runs them like libtest does, each on its own thread named
//! after the test. Arguments: `[FILTER...] [--exact] [--skip FILTER]... [--list]
//! [--test-threads N] [--nocapture]`.

use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Instant;

const FAILURE: u8 = 101;

pub type TestFuture = Pin<Box<dyn Future<Output = Result<(), String>>>>;

pub struct Test {
    pub name: &'static str,
    pub run: fn() -> TestFuture,
}

/// `tests![a, b, #[cfg(feature = "x")] c]` registers the async test functions of
/// the module, they return a `Result` with a `Debug` error. Keep the functions
/// private: one missing in the list is then dead code that fails the lint.
macro_rules! tests {
    ($($(#[$attr:meta])* $test:ident),* $(,)?) => {
        &[$($(#[$attr])* $crate::runner::Test {
            name: concat!(module_path!(), "::", stringify!($test)),
            run: || -> $crate::runner::TestFuture {
                Box::pin(async { $test().await.map_err(|e| format!("{e:?}")) })
            },
        }),*]
    };
}

pub(crate) use tests;

struct Options {
    filters: Vec<String>,
    skip: Vec<String>,
    exact: bool,
    list: bool,
    threads: usize,
}

impl Options {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut options = Options {
            filters: Vec::new(),
            skip: Vec::new(),
            exact: false,
            list: false,
            threads: std::env::var("RUST_TEST_THREADS")
                .ok()
                .and_then(|threads| threads.parse().ok())
                .or_else(|| std::thread::available_parallelism().ok().map(usize::from))
                .unwrap_or(1),
        };

        while let Some(arg) = args.next() {
            let (arg, inline) = match arg.split_once('=') {
                Some((arg, value)) if arg.starts_with("--") => {
                    (arg.to_string(), Some(value.to_string()))
                }
                _ => (arg, None),
            };
            let mut value = || {
                inline
                    .clone()
                    .or_else(|| args.next())
                    .ok_or_else(|| format!("{arg} requires a value"))
            };

            match arg.as_str() {
                "--exact" => options.exact = true,
                "--list" => options.list = true,
                // The output of the tests is never captured.
                "--nocapture" => {}
                "--skip" => options.skip.push(value()?),
                "--test-threads" => {
                    options.threads = value()?
                        .parse()
                        .ok()
                        .filter(|threads| *threads > 0)
                        .ok_or("--test-threads requires a positive integer")?;
                }
                _ if arg.starts_with('-') => return Err(format!("unknown option {arg}")),
                _ => options.filters.push(arg),
            }
        }

        Ok(options)
    }

    fn matches(&self, name: &str, filter: &str) -> bool {
        if self.exact {
            name == filter
        } else {
            name.contains(filter)
        }
    }

    fn selects(&self, name: &str) -> bool {
        (self.filters.is_empty() || self.filters.iter().any(|f| self.matches(name, f)))
            && !self.skip.iter().any(|s| self.matches(name, s))
    }
}

/// Runs the selected `tests` while the result of `setup` lives. Without any
/// selected test there is no setup.
pub fn main<G, E: Display, F: Future<Output = Result<G, E>>>(
    tests: &[&'static Test],
    setup: impl FnOnce() -> F,
) -> ExitCode {
    let options = match Options::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(FAILURE);
        }
    };

    let selected: Vec<(&str, &Test)> = tests
        .iter()
        .map(|test| (name(test), *test))
        .filter(|(name, _)| options.selects(name))
        .collect();
    let filtered = tests.len() - selected.len();

    if options.list {
        for (name, _) in &selected {
            println!("{name}: test");
        }
        return ExitCode::SUCCESS;
    }

    println!("\nrunning {} tests", selected.len());
    let start = Instant::now();
    let failures = Mutex::new(Vec::new());

    if !selected.is_empty() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("the setup runtime should build");
        let shared = match runtime.block_on(setup()) {
            Ok(shared) => shared,
            Err(e) => {
                eprintln!("error: setup failed: {e}");
                return ExitCode::from(FAILURE);
            }
        };
        let next = AtomicUsize::new(0);

        std::thread::scope(|scope| {
            for _ in 0..options.threads.min(selected.len()) {
                scope.spawn(|| {
                    while let Some((name, test)) =
                        selected.get(next.fetch_add(1, Ordering::Relaxed))
                    {
                        if let Err(e) = run(name, test) {
                            println!("test {name} ... FAILED");
                            failures.lock().unwrap().push((*name, e));
                        } else {
                            println!("test {name} ... ok");
                        }
                    }
                });
            }
        });

        drop(shared);
    }

    let mut failures = failures.into_inner().unwrap();
    failures.sort();

    if !failures.is_empty() {
        println!("\nfailures:\n");
        for (name, e) in &failures {
            println!("---- {name} ----\n{e}\n");
        }
        println!("failures:");
        for (name, _) in &failures {
            println!("    {name}");
        }
    }

    println!(
        "\ntest result: {}. {} passed; {} failed; 0 ignored; 0 measured; {filtered} filtered out; finished in {:.2}s\n",
        if failures.is_empty() { "ok" } else { "FAILED" },
        selected.len() - failures.len(),
        failures.len(),
        start.elapsed().as_secs_f64()
    );

    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(FAILURE)
    }
}

/// The name without the crate, as libtest names it.
fn name(test: &Test) -> &'static str {
    test.name
        .split_once("::")
        .map_or(test.name, |(_, name)| name)
}

fn run(name: &str, test: &Test) -> Result<(), String> {
    let run = test.run;

    std::thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?
                .block_on(run())
        })
        .map_err(|e| e.to_string())?
        .join()
        .unwrap_or_else(|panic| {
            Err(panic
                .downcast_ref::<&str>()
                .map(ToString::to_string)
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panicked".to_string()))
        })
}
