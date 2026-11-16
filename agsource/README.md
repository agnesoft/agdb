# agsource

The source model of a Rust crate, read with [`syn`](https://github.com/dtolnay/syn) without
compiling anything.

`SourceCrate::load` starts at a crate root (`lib.rs`, `main.rs`, an integration test, ...) and
follows the `mod` declarations the way rustc does:

- `x.rs` or `x/mod.rs`;
- `a.rs` looking in `a/`;
- inline modules nesting the directory;
- `#[path]` relative to the declaring file, and naming the directory of an inline module.

It keeps the parsed files and the module tree over them. `rust_files` lists the `.rs` files of a
directory, skipping cache directories such as cargo's `target`, so files that no crate root reaches
can be told apart.

A `Cfg` selects the code:

- `Cfg::All` keeps all of it and applies every `cfg_attr`;
- `Cfg::Eval(CfgSet)` keeps what rustc compiles with the given options (`feature = "serde"`,
  `test`, `unix`, ...). It strips the elements whose `#[cfg]` does not hold, wherever rustc strips
  them, and applies a `cfg_attr` only when its predicate holds. A name the set does not know is
  false and reported.

A problem with one file does not stop loading. These are all collected as `Error`s with their
location:

- an unreadable file;
- invalid Rust;
- a module without a file or with two files;
- a circular `#[path]`;
- an invalid or unknown cfg.

All file access goes through [agplatform](https://crates.io/crates/agplatform), so it can be tested
in memory.

Used by [agtestcheck](../agtestcheck) and the agdb API client generator.
