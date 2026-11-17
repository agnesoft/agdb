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

`SourceCrate::resolve` tells what a path names from a given module, in the type, value or macro
namespace: a module, an item, an enum variant, an associated item of a local type (`S::new`), a
primitive or a path into another crate. It follows the imports the way rustc does: renames, groups,
`self`, globs with the names visible to the importing module, and `pub use` chains in any order.

`packages` asks `cargo metadata` (`$CARGO` when set, as for a cargo subcommand) for the packages
of a workspace and the crates cargo builds from them: the library, binaries, tests, examples,
benches and the build script, each a `Target` that `SourceCrate::load` takes. A target's `externs`
are the crates its code can name, as cargo passes them with `--extern`: the dependencies (dev and
target-specific ones included, build dependencies for the build script), the package's own library
for the other crates, and `proc_macro` for a proc-macro library. `std`, `core`, `alloc` and
`extern crate`s are always there.
The std prelude is spelled out, `Vec` is `std::vec::Vec`.

A `macro_rules!` is in textual scope, in its module and the modules within, and from a
`#[macro_use]` module in its parent. It gets a path with `#[macro_export]` (at the crate root) or
`use`.

Names only the caller can know are not resolved: `Self`, generic parameters, and the items and
imports declared inside function bodies.

`SourceCrate::exports` lists the public names of the crate: the paths from its root through `pub`
modules and re-exports, `pub` globs included. `SourceCrate::reachable` collects what some of them
lead to, through the paths in the items:

- field, variant and signature types, bounds and the paths in bodies (`Self`, generic parameters
  and the bindings of the item are skipped);
- the impls of the types reached: inherent ones and those of traits that are reached, exported or
  of other crates;
- for a module root, its `pub` functions and those of its `pub` submodules.

The caller can name opaque items, which are reached but not followed: the impls of an opaque trait
are skipped too. Paths into other crates are listed, to continue in their exports, and so are the
paths that name nothing. Macro bodies and arguments are not followed.

A problem with one file does not stop loading. These are all collected as `Error`s with their
location:

- an unreadable file;
- invalid Rust;
- a module without a file or with two files;
- a circular `#[path]`;
- an invalid or unknown cfg;
- an unresolved import.

`packages` returns an `Error` when `cargo metadata` fails.

All file and process access goes through [agplatform](https://crates.io/crates/agplatform), so it
can be tested in memory.

Used by [agtestcheck](../agtestcheck) and the agdb API client generator.
