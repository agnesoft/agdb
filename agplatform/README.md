# agplatform

## Crates

### agbench

[![Crates.io](https://img.shields.io/crates/v/agbench)](https://crates.io/crates/agbench)
[![docs.rs](https://docs.rs/agbench/badge.svg)](https://docs.rs/agbench/latest/agbench/)

<details>

<summary>Microbenchmark harness comparing targets doing the same work, e.g. agplatform wrappers against the std functions they wrap. Each benchmark has a setup creating its fixture, its targets are measured in turn on the fixture and reported relative to the first one (the baseline), then the fixture is dropped.</summary>

```rust
// <crate>/benches/fs.rs, with `[[bench]] name = "fs" harness = false`
use agbench::Bench;
use agplatform::{Fs, Platform};

fn main() -> std::process::ExitCode {
    agbench::main(&[read])
}

fn read(bench: &mut Bench) {
    bench
        .benchmark("fs/read", agplatform::platform)
        .target("std", |_| std::fs::read("Cargo.toml").unwrap())
        .target("agplatform", |p| p.fs().read("Cargo.toml").unwrap())
        .run();
}
```

Run

```
cargo bench [-p <crate>] -- [--filter PATTERN]... [--filter-target PATTERN]... [--samples N] [--sample-ms MS] [--iters N] [--list] [--check]
```

Possible output:

```
benchmark                        target                iters       time  overhead  IQR
fs/read/64B                      std                    8192   927.0 ns
                                 agplatform                     1.87 µs   +100.4%  [+67.6% .. +106.5%]
```

</details>

### agplatform

[![Crates.io](https://img.shields.io/crates/v/agplatform)](https://crates.io/crates/agplatform)
[![docs.rs](https://docs.rs/agplatform/badge.svg)](https://docs.rs/agplatform/latest/agplatform/)

<details>

<summary>Platform providing env, fs, request and process infrastructure for production and testing. Abstracted over a Platform trait it allows easy swapping production and test implementations. Refer to the <a href="agplatform/cheatsheet.md">cheatsheet</a> for quick overview and details.</summary>

```rust
fn read_file(p: impl agplatform::Platform) {
    let content = p.fs().read_to_string()?;
}
read_file(agplatform::platform()); // production implementation using std, tokio, reqwest
read_file(agplatform::test_platfrom()); // test in-memory implementation with abilities to simulate failures, manipulate time etc.

let platfrom = agplatform::platform();
let out = Command::new("git").arg("diff").run_blocking(&platform.process()).unwrap();
# OR
let out = platform.process().run_blocking(Command::new("git").arg("diff")).unwrap();
```

</details>

### agtestcheck

[![Crates.io](https://img.shields.io/crates/v/agtestcheck)](https://crates.io/crates/agtestcheck)

<details>

<summary>Cargo subcommand checking the structure of Rust tests binding the tests code in the production structure and enforcing the rules. See the [test rules](agtestcheck/rust-test-rules.md) for details.</summary>

```rust
// source.rs
fn foo() -> i32 { 1 }
struct S;
impl S { fn bar() -> Self { S } }

#[cfg(test)]
mod tests {
    use super::*;

    mod foo {
        #[test]
        fn return_1() { assert_eq(foo(), 1); }
    }

    mod s {
        mod bar {
            #[test]
            fn bar_should_crate_s() { /* testing stuff */}
        }
    }

    #[test]
    fn ranom_arbitrary_test() {}
}
```

Run

```
# cargo install agtestcheck
cargo test-check [<dir|file.rs>]
```

Possible output:

```
source.rs:12: test `return_1` should be named `foo_should_<behaviour>` or `foo_<args>_should_<behaviour>`
source.rs:18: comment "testing stuff" should be deleted, start with WHY or GIVEN, WHEN, THEN, AND
source.rs:23: test `ranom_arbitrary_test` should be in the module of its type, trait, function or method
```

</details>
