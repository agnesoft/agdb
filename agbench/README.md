<p align="center">
    <a href="https://github.com/agnesoft/agdb/tree/main/agbench"><img width="300" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo"></a>
</p>
<h1 align="center">
    agbench
</h1>
<h3 align="center">
    Benchmarks against a baseline.
</h3>

<p align="center">
    <a href="https://github.com/agnesoft/agdb/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="license"></a>
    <a href="https://crates.io/crates/agbench"><img src="https://img.shields.io/crates/v/agbench" alt="Crates.io"></a>
    <a href="https://docs.rs/agbench/latest/agbench/"><img src="https://docs.rs/agbench/badge.svg" alt="docs.rs"></a>
    <a href="https://github.com/agnesoft/agdb/actions/workflows/agdb.yaml"><img src="https://github.com/agnesoft/agdb/actions/workflows/agdb.yaml/badge.svg" alt="ci"></a>
</p>

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;Agnesoft Bench

<p align="center">
    <a href="#at-a-glance">Quickstart</a> | <a href="#options">OPTIONS</a> | <a href="#how-it-measures">How it measures</a> | <a href="https://docs.rs/agbench/latest/agbench/">API</a>
</p>

The Agnesoft Bench (aka _agbench_) is a microbenchmark harness for `cargo bench` that measures how several targets doing the same work compare to a baseline, for example a wrapper against the `std` function it wraps or a new implementation against the old one. It reports each target's time and its overhead against the baseline, and it can fail the run when a target is slower than allowed. Because the result is a ratio measured within each sample, a slower or busier machine does not change it. That makes the benchmarks usable as a CI gate on shared runners.

- [Key Features](#key-features)
- [Why agbench](#why-agbench)
- [At a glance](#at-a-glance)
- [Options](#options)
- [How it measures](#how-it-measures)
- [Crate Features](#crate-features)
- [Reference](#reference)

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;Key Features

- Compares targets doing the same work: the first target is the baseline and the others are reported as its overhead.
- Relative results that hold on any machine, with no stored baselines or earlier runs to compare against.
- A CI gate: with `--check`, a target slower than its benchmark's fail factor fails the run.
- Simple to use: a plain `main` in a `harness = false` bench target, one builder chain per benchmark.
- _No dependencies_.

Technical features:

- Targets run in turn within every sample, rotating which goes first, so drifts in clock speed, caches and load hit all of them alike.
- The overhead is the median of the per-sample ratios to the baseline, reported with their interquartile range.
- Iterations are calibrated per benchmark to a target sample duration, or fixed with `--iters`.
- A shared fixture per benchmark, set up once and not measured.
- An unmeasured input per iteration (`iteration_setup`) for work that consumes it, such as removing a file. Inputs are prepared up to 64 at a time.
- Results are passed through `black_box` so the measured work is not optimized away.
- Filtering of benchmarks and targets, and listing them without running.

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;Why agbench

Most benchmark harnesses measure absolute time and compare it to earlier runs. That works on a dedicated, quiet machine with the history kept around. It does not work on CI runners, which are shared, vary from run to run and keep nothing. There, the same code measures noticeably faster or slower depending on its neighbours, so absolute numbers cannot gate anything.

The question worth gating is usually relative anyway: is the wrapper still close to `std`, and is the new implementation not slower than the old one? agbench measures exactly that. Each sample runs every target in turn under the same conditions, and the overhead is their ratio, so machine noise largely cancels out. The ratio is stable enough to fail a build on, with a fail factor set per benchmark.

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;At a glance

```
cargo add agbench --dev
```

Add a bench target without the default harness to `Cargo.toml`:

```toml
[[bench]]
name = "collections"
harness = false
```

Then define the benchmarks in `benches/collections.rs`. Each one has a fixture shared by its targets, and optionally an input set up for each iteration:

```rs
use agbench::Bench;
use std::process::ExitCode;

fn main() -> ExitCode {
    agbench::main(&[concat, remove_first])
}

fn concat(bench: &mut Bench) {
    bench
        .benchmark("string/concat", || vec!["word"; 64])
        .fail_factor(1.5)
        .target("concat", |words| words.concat())
        .target("join", |words| words.join(""))
        .target("collect", |words| words.iter().copied().collect::<String>())
        .run();
}

fn remove_first(bench: &mut Bench) {
    bench
        .benchmark("vec/remove_first", || ())
        .iteration_setup(|_| (0..4096).collect::<Vec<u32>>())
        .target("remove", |_, mut v| v.remove(0))
        .target("swap_remove", |_, mut v| v.swap_remove(0))
        .run();
}
```

Run it with `cargo bench`:

```
cargo bench --bench collections
```

```
benchmark                        target                iters       time  overhead  IQR
string/concat                    concat                65536   148.5 ns
                                 join                          152.8 ns     -1.2%  [-2.9% .. +2.9%]
                                 collect                       173.1 ns    +11.4%  [+7.4% .. +17.6%]
vec/remove_first                 remove                16384   675.0 ns
                                 swap_remove                   412.7 ns    -38.3%  [-39.1% .. -35.4%]
```

`iters` is the number of iterations per sample, `time` the median time of one iteration, `overhead` the median ratio to the baseline and `IQR` the interquartile range of the ratios. In CI, add `--check` to fail the run (exit code 1) if `join` or `collect` is more than 1.5 times slower than `concat`:

```
cargo bench --bench collections -- --check
```

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;Options

Passed after `--` of `cargo bench [-p <crate>] [--bench <name>]`:

| Option                    | Default | Description                                                                                                       |
| ------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `--filter PATTERN`        |         | Runs only the benchmarks whose name contains `PATTERN`. Repeatable to run those matching any of them.             |
| `--filter-target PATTERN` |         | Runs only the targets whose name contains `PATTERN`. Repeatable. The first selected target is the baseline.       |
| `--samples N`             | 30      | Number of samples, each running every target.                                                                     |
| `--sample-ms MS`          | 10      | Duration of one sample of the baseline that the calibration aims for.                                             |
| `--iters N`               |         | Fixed number of iterations per sample instead of calibrating.                                                     |
| `--list`                  |         | Lists the benchmarks and their targets without running them.                                                      |
| `--check`                 |         | Fails (exit code 1) if a target is slower than its fail factor times the baseline in at least 3/4 of the samples. |
| `--help`                  |         | Prints the usage. Invalid options also print it and fail with exit code 2 without running anything.               |

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;How it measures

Each benchmark runs in phases:

1. **Setup**: the fixture is created, unless no target is selected.
2. **Calibration**: the iterations per sample are doubled until the baseline takes at least `--sample-ms`.
3. **Measurement**: each of the `--samples` samples runs every target in turn, rotating which one goes first.
4. **Cleanup**: the fixture is dropped.
5. **Report**: the median time per iteration of each target, and the median and interquartile range of its per-sample ratios to the baseline.

With `--check`, a target whose lower quartile ratio exceeds the fail factor of its benchmark is marked `FAIL` and the run fails. The lower quartile is used rather than the median so that the gate fires only when the target is slower in most samples, not when a few are noisy. Benchmarks without a fail factor are reported but never fail.

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;Crate Features

agbench has no features and no dependencies.

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agnesoft logo">&nbsp;&nbsp;Reference

- [API](https://docs.rs/agbench/latest/agbench/)

- [agplatform benchmarks](https://github.com/agnesoft/agdb/tree/main/agplatform/benches) (real-world usage against `std`)

- [agplatform](https://crates.io/crates/agplatform)

- [agtestcheck](https://crates.io/crates/agtestcheck)
