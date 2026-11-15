use std::cell::RefCell;
use std::hint::black_box;
use std::io::Write;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use crate::config::Config;
use crate::stats::TargetSummary;
use crate::stats::summarize;

/// A runner of benchmarks, each comparing targets (e.g. a wrapper and the
/// `std` function it wraps) performing the same work on a shared fixture.
/// Create benchmarks with [`Self::benchmark`].
///
/// Benchmarks and targets are selected with `--filter` and
/// `--filter-target`, the first selected target being the baseline. With
/// `--list` the selected benchmarks and their targets are printed instead.
///
/// A benchmark runs in phases:
///
/// 1. setup: the fixture is created, unless no target is selected
///    (and an input for each iteration, see [`Benchmark::iteration_setup`]),
/// 2. calibration: the iterations per sample are doubled until the first
///    target (the baseline) takes at least `--sample-ms`,
/// 3. measurement: `--samples` samples each run every target in turn,
///    rotating which one goes first,
/// 4. cleanup: the fixture is dropped,
/// 5. report: the median time per iteration of each target and the median
///    and interquartile range of its per-sample ratios to the baseline.
///
/// With `--check` a target whose lower quartile ratio exceeds the
/// [`Benchmark::fail_factor`] of its benchmark is marked as `FAIL` and the
/// run returns failure (1).
pub struct Bench<'a> {
    config: Config,
    clock: &'a mut dyn FnMut() -> Duration,
    out: &'a mut dyn Write,
    header: bool,
    failures: usize,
}

/// A benchmark being defined by [`Bench::benchmark`], run with
/// [`Self::run`].
#[must_use = "a benchmark only runs when `run` is called"]
pub struct Benchmark<'b, 'a, F> {
    bench: &'b mut Bench<'a>,
    name: String,
    setup: Box<dyn FnOnce() -> F + 'b>,
    targets: Vec<(String, TargetFn<'b, F>)>,
    fail_factor: Option<f64>,
}

/// Runs the given number of iterations of a target on the fixture and
/// returns their duration measured with the clock.
type TargetFn<'b, F> = Box<dyn FnMut(&mut F, u64, &mut dyn FnMut() -> Duration) -> Duration + 'b>;

/// A benchmark with an input set up for each iteration, see
/// [`Benchmark::iteration_setup`].
#[must_use = "a benchmark only runs when `run` is called"]
pub struct IteratedBenchmark<'b, 'a, F, I> {
    benchmark: Benchmark<'b, 'a, F>,
    setup: Rc<RefCell<IterationSetup<'b, F, I>>>,
}

type IterationSetup<'b, F, I> = Box<dyn FnMut(&mut F) -> I + 'b>;

/// A [`TargetFn`] with the fixture type erased.
type Run<'t> = dyn FnMut(u64, &mut dyn FnMut() -> Duration) -> Duration + 't;

struct Target<'t> {
    name: String,
    run: Box<Run<'t>>,
}

/// The maximum number of inputs of [`Benchmark::iteration_setup`] prepared
/// at once.
const BATCH: u64 = 64;

impl<'a> Bench<'a> {
    pub(crate) fn new(
        config: Config,
        clock: &'a mut dyn FnMut() -> Duration,
        out: &'a mut dyn Write,
    ) -> Self {
        Self {
            config,
            clock,
            out,
            header: false,
            failures: 0,
        }
    }

    /// Prints the number of targets that failed their fail factor, if any,
    /// and returns the exit code of the run.
    pub(crate) fn finish(mut self) -> ExitCode {
        if self.failures == 0 {
            ExitCode::SUCCESS
        } else {
            self.print(&format!(
                "FAILED: {} target(s) slower than their fail factor",
                self.failures
            ));
            ExitCode::FAILURE
        }
    }

    /// Defines a benchmark of `name` whose fixture is created by `setup`.
    /// Add targets with [`Benchmark::target`] and run it with
    /// [`Benchmark::run`].
    pub fn benchmark<'b, F>(
        &'b mut self,
        name: &str,
        setup: impl FnOnce() -> F + 'b,
    ) -> Benchmark<'b, 'a, F> {
        Benchmark {
            bench: self,
            name: name.to_string(),
            setup: Box::new(setup),
            targets: Vec::new(),
            fail_factor: None,
        }
    }

    fn run(
        &mut self,
        name: &str,
        fail_factor: Option<f64>,
        setup: &mut dyn FnMut(),
        cleanup: &mut dyn FnMut(),
        targets: &mut [Target],
    ) {
        if !self.config.matches_benchmark(name) {
            return;
        }

        assert!(!targets.is_empty(), "Benchmark {name} has no targets");
        let mut targets: Vec<&mut Target> = targets
            .iter_mut()
            .filter(|t| self.config.matches_target(&t.name))
            .collect();

        if targets.is_empty() {
            return;
        }

        if self.config.list {
            let targets: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();
            self.print(&format!("{name}: {}", targets.join(", ")));
            return;
        }

        setup();
        let iters = self.iters(&mut *targets[0].run);

        for target in &mut targets[1..] {
            (target.run)(iters, &mut *self.clock);
        }

        let samples: Vec<Vec<f64>> = (0..self.config.samples)
            .map(|sample| {
                let mut times = vec![0.0; targets.len()];

                for offset in 0..targets.len() {
                    let target = (sample + offset) % targets.len();
                    times[target] = self.per_iter(&mut *targets[target].run, iters);
                }

                times
            })
            .collect();

        cleanup();
        self.report(name, fail_factor, iters, &targets, &summarize(&samples));
    }

    fn iters(&mut self, baseline: &mut Run) -> u64 {
        if let Some(iters) = self.config.iters {
            return iters;
        }

        let mut iters = 1;

        while self.elapsed(baseline, iters) < self.config.sample_time {
            iters *= 2;
        }

        iters
    }

    fn elapsed(&mut self, f: &mut Run, iters: u64) -> Duration {
        f(iters, &mut *self.clock)
    }

    fn per_iter(&mut self, f: &mut Run, iters: u64) -> f64 {
        self.elapsed(f, iters).as_nanos() as f64 / iters as f64
    }

    fn report(
        &mut self,
        name: &str,
        fail_factor: Option<f64>,
        iters: u64,
        targets: &[&mut Target],
        summaries: &[TargetSummary],
    ) {
        if !self.header {
            self.header = true;

            self.print(&format!(
                "{:<32} {:<16} {:>10} {:>10} {:>9}  {}",
                "benchmark", "target", "iters", "time", "overhead", "IQR"
            ));
        }

        for (index, (target, summary)) in targets.iter().zip(summaries).enumerate() {
            let (name, iters) = if index == 0 {
                (name, iters.to_string())
            } else {
                ("", String::new())
            };
            let failed = summary
                .ratio
                .as_ref()
                .is_some_and(|ratio| self.fails(ratio.low, fail_factor));
            let ratio = summary.ratio.as_ref().map_or_else(String::new, |ratio| {
                format!(
                    "{:>9}  [{} .. {}]{}",
                    format_overhead(ratio.median),
                    format_overhead(ratio.low),
                    format_overhead(ratio.high),
                    if failed {
                        format!("  FAIL > {}x", fail_factor.unwrap_or_default())
                    } else {
                        String::new()
                    }
                )
            });
            self.failures += usize::from(failed);

            self.print(
                format!(
                    "{name:<32} {:<16} {iters:>10} {:>10} {ratio}",
                    target.name,
                    format_time(summary.time)
                )
                .trim_end(),
            );
        }
    }

    /// Returns whether a target whose lower quartile ratio to the baseline
    /// is `low` exceeds the `fail_factor` of its benchmark, if checked by
    /// `--check`.
    fn fails(&self, low: f64, fail_factor: Option<f64>) -> bool {
        self.config.check && fail_factor.is_some_and(|factor| low > factor)
    }

    fn print(&mut self, line: &str) {
        writeln!(self.out, "{line}").expect("output should be writable");
    }
}

impl<'b, 'a, F> Benchmark<'b, 'a, F> {
    /// Adds a target of `name` performing the measured work on the fixture.
    /// The first target added is the baseline the others are compared
    /// against. The result is passed through [`black_box`] so that the work
    /// is not optimized away.
    pub fn target<R>(mut self, name: &str, mut f: impl FnMut(&mut F) -> R + 'b) -> Self {
        self.targets.push((
            name.to_string(),
            Box::new(
                move |fixture: &mut F, iters: u64, clock: &mut dyn FnMut() -> Duration| {
                    let start = clock();

                    for _ in 0..iters {
                        black_box(f(fixture));
                    }

                    clock() - start
                },
            ),
        ));
        self
    }

    /// Sets the maximum ratio of each target to the baseline, e.g. `1.5` for
    /// at most 50 % slower. Checked only with `--check`, see
    /// [`Bench`].
    ///
    /// # Panics
    ///
    /// Panics if `factor` is not a positive finite number.
    pub fn fail_factor(mut self, factor: f64) -> Self {
        self.fail_factor = Some(checked_factor(&self.name, factor));
        self
    }

    /// Sets up an input for each iteration of the targets added
    /// afterwards, e.g. a file to remove, passed to them along with the
    /// fixture. The `setup` is not measured: up to 64 inputs are created at
    /// a time before their iterations are measured.
    pub fn iteration_setup<I>(
        self,
        setup: impl FnMut(&mut F) -> I + 'b,
    ) -> IteratedBenchmark<'b, 'a, F, I> {
        IteratedBenchmark {
            benchmark: self,
            setup: Rc::new(RefCell::new(Box::new(setup))),
        }
    }

    /// Runs the benchmark, see [`Bench`] for the phases.
    ///
    /// # Panics
    ///
    /// Panics if the benchmark has no targets and is not filtered out.
    pub fn run(self) {
        let Self {
            bench,
            name,
            setup,
            targets,
            fail_factor,
        } = self;
        let fixture = &RefCell::new(None);
        let mut setup = Some(setup);
        let mut targets: Vec<Target> = targets
            .into_iter()
            .map(|(name, mut f)| Target {
                name,
                run: Box::new(move |iters, clock| {
                    f(
                        fixture
                            .borrow_mut()
                            .as_mut()
                            .expect("fixture should be set up"),
                        iters,
                        clock,
                    )
                }),
            })
            .collect();

        bench.run(
            &name,
            fail_factor,
            &mut || *fixture.borrow_mut() = setup.take().map(|setup| setup()),
            &mut || drop(fixture.take()),
            &mut targets,
        );
    }
}

impl<'b, F: 'b, I: 'b> IteratedBenchmark<'b, '_, F, I> {
    /// Same as [`Benchmark::fail_factor`].
    pub fn fail_factor(mut self, factor: f64) -> Self {
        self.benchmark = self.benchmark.fail_factor(factor);
        self
    }

    /// Adds a target of `name` like [`Benchmark::target`], performing the
    /// measured work on the fixture and the input of the iteration.
    pub fn target<R>(mut self, name: &str, mut f: impl FnMut(&mut F, I) -> R + 'b) -> Self {
        let setup = self.setup.clone();
        self.benchmark.targets.push((
            name.to_string(),
            Box::new(
                move |fixture: &mut F, iters: u64, clock: &mut dyn FnMut() -> Duration| {
                    batched(iters, clock, &mut |chunk, clock| {
                        let mut setup = setup.borrow_mut();
                        let inputs: Vec<I> = (0..chunk).map(|_| setup(fixture)).collect();
                        let start = clock();

                        for input in inputs {
                            black_box(f(fixture, input));
                        }

                        clock() - start
                    })
                },
            ),
        ));
        self
    }

    /// Runs the benchmark, see [`Bench`] for the phases.
    ///
    /// # Panics
    ///
    /// Panics if the benchmark has no targets and is not filtered out.
    pub fn run(self) {
        self.benchmark.run();
    }
}

/// Returns `factor` if it is a positive finite number, panics otherwise.
fn checked_factor(benchmark: &str, factor: f64) -> f64 {
    assert!(
        factor.is_finite() && factor > 0.0,
        "Fail factor of {benchmark} must be a positive number: {factor}"
    );
    factor
}

/// Runs `iters` iterations in chunks of at most [`BATCH`] with `chunk` and
/// returns the sum of the durations it measured.
fn batched(
    iters: u64,
    clock: &mut dyn FnMut() -> Duration,
    chunk: &mut dyn FnMut(u64, &mut dyn FnMut() -> Duration) -> Duration,
) -> Duration {
    let mut elapsed = Duration::ZERO;
    let mut remaining = iters;

    while remaining > 0 {
        let iters = remaining.min(BATCH);
        elapsed += chunk(iters, clock);
        remaining -= iters;
    }

    elapsed
}

/// Formats nanoseconds with a unit keeping three significant digits.
fn format_time(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("{ns:.1} ns")
    } else if ns < 1_000_000.0 {
        format!("{:.2} µs", ns / 1_000.0)
    } else if ns < 1_000_000_000.0 {
        format!("{:.2} ms", ns / 1_000_000.0)
    } else {
        format!("{:.2} s", ns / 1_000_000_000.0)
    }
}

/// Formats a ratio to the baseline as a signed percentage.
fn format_overhead(ratio: f64) -> String {
    format!("{:+.1}%", (ratio - 1.0) * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::*;

    mod benchmark {
        use super::*;

        mod fail_factor {
            use super::*;

            #[test]
            fn fail_factor_not_positive_should_panic() {
                fn invalid(bench: &mut Bench) {
                    let _ = bench.benchmark("invalid", || ()).fail_factor(0.0);
                }

                let result = std::panic::catch_unwind(|| run_with(&[], &[invalid]));

                assert!(result.is_err());
            }
        }

        mod run {
            use super::*;

            #[test]
            fn run_should_set_up_measure_and_clean_up_in_order() {
                let _ = run_with(
                    &["--iters", "1", "--samples", "1", "--filter", "read"],
                    &[read_and_write],
                );

                assert_eq!(
                    LOG.take(),
                    [
                        "setup fs/read",
                        "agplatform",
                        "std",
                        "agplatform",
                        "cleanup"
                    ]
                );
            }

            #[test]
            fn run_should_skip_filtered_out_benchmarks() {
                let (_, out) = run_with(
                    &["--iters", "1", "--samples", "1", "--filter", "write"],
                    &[read_and_write],
                );

                assert_eq!(
                    LOG.take(),
                    [
                        "setup fs/write",
                        "agplatform",
                        "std",
                        "agplatform",
                        "cleanup"
                    ]
                );
                assert!(out.contains("\nfs/write "), "{out}");
                assert!(!out.contains("fs/read"), "{out}");
            }

            #[test]
            fn run_list_should_print_matching_benchmarks_without_running_them() {
                let (code, out) = run_with(
                    &[
                        "--list",
                        "--filter",
                        "read",
                        "--filter-target",
                        "agplatform",
                    ],
                    &[read, read_and_write],
                );

                assert_eq!(code, ExitCode::SUCCESS);
                assert_eq!(out, "fs/read: agplatform\nfs/read: agplatform\n");
                assert!(LOG.take().is_empty());
            }

            #[test]
            fn run_should_measure_only_selected_targets_against_first_selected() {
                let (_, out) = run_with(
                    &[
                        "--iters",
                        "1",
                        "--samples",
                        "1",
                        "--filter-target",
                        "agplatform",
                        "--filter-target",
                        "memory",
                    ],
                    &[read],
                );

                assert!(
                    out.ends_with(&format!(
                        "{:<32} {:<16} {:>10} {:>10}\n{:<32} {:<16} {:>10} {:>10} {:>9}  [-66.7% .. -66.7%]\n",
                        "fs/read", "agplatform", 1, "150.0 ns", "", "in_memory", "", "50.0 ns", "-66.7%"
                    )),
                    "{out}"
                );
                assert!(!out.contains("std"), "{out}");
            }

            #[test]
            fn run_without_selected_targets_should_skip_benchmark() {
                let (_, out) = run_with(&["--filter-target", "missing"], &[read_and_write]);

                assert!(out.is_empty(), "{out}");
                assert!(LOG.take().is_empty());
            }

            #[test]
            fn run_should_rotate_which_target_goes_first() {
                fn rotated(bench: &mut Bench) {
                    bench
                        .benchmark("rotated", || ())
                        .target("a", |_| log("a"))
                        .target("b", |_| log("b"))
                        .target("c", |_| log("c"))
                        .run();
                }

                let _ = run_with(&["--iters", "1", "--samples", "3"], &[rotated]);

                assert_eq!(
                    LOG.take(),
                    ["b", "c", "a", "b", "c", "b", "c", "a", "c", "a", "b"]
                );
            }

            #[test]
            fn run_should_calibrate_iterations_to_sample_time() {
                let (_, out) = run_with(&["--samples", "3", "--sample-ms", "1"], &[read]);

                assert!(
                    out.contains(
                        "\nfs/read                          std                   16384   100.0 ns\n"
                    ),
                    "{out}"
                );
            }

            #[test]
            fn run_without_targets_should_panic() {
                fn empty(bench: &mut Bench) {
                    bench.benchmark("empty", || ()).run();
                }

                let result = std::panic::catch_unwind(|| run_with(&[], &[empty]));

                assert!(result.is_err());
            }
        }
    }

    mod iterated_benchmark {
        use super::*;

        mod target {
            use super::*;

            #[test]
            fn target_should_measure_only_work_set_up_in_batches() {
                // GIVEN a setup counting the inputs it creates
                fn counted(bench: &mut Bench) {
                    bench
                        .benchmark("counted", || 0)
                        .iteration_setup(|count| {
                            *count += 1;
                            log("setup");
                            takes(1000);
                            *count
                        })
                        .target("a", |_, input| {
                            log(&format!("work {input}"));
                            takes(100)
                        })
                        .run();
                }

                // WHEN running 100 iterations
                let (_, out) = run_with(&["--iters", "100", "--samples", "1"], &[counted]);

                // THEN inputs are set up in batches of 64 and only the work is measured
                let mut expected = Vec::new();

                for batch in [1..=64, 65..=100] {
                    expected.extend(batch.clone().map(|_| "setup".to_string()));
                    expected.extend(batch.map(|input| format!("work {input}")));
                }

                assert_eq!(LOG.take(), expected);
                assert!(
                    out.ends_with(&format!(
                        "{:<32} {:<16} {:>10} {:>10}\n",
                        "counted", "a", 100, "100.0 ns"
                    )),
                    "{out}"
                );
            }

            #[test]
            fn target_should_share_iteration_setup_with_other_targets() {
                fn shared(bench: &mut Bench) {
                    bench
                        .benchmark("shared", || 0)
                        .iteration_setup(|count| {
                            *count += 1;
                            *count
                        })
                        .fail_factor(2.0)
                        .target("a", |_, input| log(&format!("a {input}")))
                        .target("b", |_, input| log(&format!("b {input}")))
                        .run();
                }

                let _ = run_with(&["--iters", "1", "--samples", "1"], &[shared]);

                assert_eq!(LOG.take(), ["b 1", "a 2", "b 3"]);
            }
        }
    }

    mod format_time {
        use super::*;

        #[test]
        fn format_time_should_pick_unit_by_magnitude() {
            for (ns, expected) in [
                (12.34, "12.3 ns"),
                (1_234.0, "1.23 µs"),
                (1_234_000.0, "1.23 ms"),
                (1_234_000_000.0, "1.23 s"),
            ] {
                assert_eq!(format_time(ns), expected, "ns: {ns}");
            }
        }
    }

    mod format_overhead {
        use super::*;

        #[test]
        fn format_overhead_should_return_signed_percentage() {
            for (ratio, expected) in [(1.5, "+50.0%"), (1.0, "+0.0%"), (0.75, "-25.0%")] {
                assert_eq!(format_overhead(ratio), expected, "ratio: {ratio}");
            }
        }
    }
}
