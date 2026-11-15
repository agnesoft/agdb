//! Helpers shared by the tests of the crate.

use crate::Bench;
use crate::run;
use std::cell::Cell;
use std::cell::RefCell;
use std::process::ExitCode;
use std::time::Duration;

thread_local! {
    /// The clock of [`run_with`], advanced only by the benchmarked code.
    pub(crate) static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    /// Events of the benchmarked code in order.
    pub(crate) static LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub(crate) const HEADER: &str =
    "benchmark                        target                iters       time  overhead  IQR";

/// Runs `benchmarks` with `args` against [`TIME`] and returns the exit
/// code and the output.
pub(crate) fn run_with(args: &[&str], benchmarks: &[fn(&mut Bench)]) -> (ExitCode, String) {
    let mut out = Vec::new();
    let code = run(
        args.iter().map(|a| a.to_string()).collect(),
        &mut || TIME.get(),
        &mut out,
        benchmarks,
    );

    (code, String::from_utf8(out).unwrap())
}

pub(crate) fn takes(ns: u64) {
    TIME.set(TIME.get() + Duration::from_nanos(ns));
}

pub(crate) fn log(event: &str) {
    LOG.with_borrow_mut(|log| log.push(event.to_string()));
}

/// A fixture logging its setup and cleanup.
pub(crate) struct Logged;

impl Logged {
    pub(crate) fn new(name: &str) -> Self {
        log(&format!("setup {name}"));
        Self
    }
}

impl Drop for Logged {
    fn drop(&mut self) {
        log("cleanup");
    }
}

pub(crate) fn read(bench: &mut Bench) {
    bench
        .benchmark("fs/read", || ())
        .target("std", |_| takes(100))
        .target("agplatform", |_| takes(150))
        .target("in_memory", |_| takes(50))
        .run();
}

pub(crate) fn limited(bench: &mut Bench, factor: f64) {
    bench
        .benchmark("fs/read", || ())
        .fail_factor(factor)
        .target("std", |_| takes(100))
        .target("agplatform", |_| takes(150))
        .target("in_memory", |_| takes(50))
        .run();
}

pub(crate) fn strict(bench: &mut Bench) {
    limited(bench, 1.4);
}

pub(crate) fn loose(bench: &mut Bench) {
    limited(bench, 1.5);
}

pub(crate) fn read_and_write(bench: &mut Bench) {
    for name in ["fs/read", "fs/write"] {
        bench
            .benchmark(name, || Logged::new(name))
            .target("std", |_| {
                log("std");
                takes(100)
            })
            .target("agplatform", |_| {
                log("agplatform");
                takes(100)
            })
            .run();
    }
}
