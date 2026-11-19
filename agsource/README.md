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
of a workspace, with their features, and the crates cargo builds from them: the library, binaries, tests, examples,
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
paths that name nothing.

`SourceCrate::macro_call` parses a macro call:

- the std macros taking a format string (`format!`, `println!`, `write!`, `panic!`, `assert_eq!`,
  ...) into the expressions before it and the format arguments, the names the string captures
  (`{name}`, `{:width$}`) included;
- `vec!`, `matches!` and `thread_local!` by their forms;
- a `macro_rules!` of the crate by expanding it with the first rule that matches, into statements.
  All fragment specifiers but `stmt` are supported, and an `expr` fragment stays one operand.

`reachable` follows the parsed calls, expansions from the module of the call, and lists those it
cannot parse: the macros of other crates and the calls that do not fit.

With the `codegen` feature, `SourceCrate::ir` reads items (e.g. the reachable ones) into an
intermediate representation of them as written, and `codegen::rust::item` prints one back as Rust
(`codegen::rust::rewrite` replaces the items in their files):

- structs, enums, traits, impls, functions, consts, statics, type aliases, `use` trees and
  `extern crate`;
- attributes after `cfg_attr`, docs, visibility, generics with bounds and `where` clauses;
- types and paths, each path with what it names (an item, a variant, a generic parameter, `Self`,
  a local binding or an item of a block, ...);
- bodies as statements, expressions and patterns, `async`/`.await`, `move` closures and
  `let`-`else` included. A macro call is one of the forms `agsource` parses (`format!` and the
  like, `vec!`, `matches!`, `thread_local!`, `file!()` and the like) with its arguments loaded.

A single name in a pattern is a constant when it names a const, a unit struct or a variant, or a
name of another crate starting upper case (`None`), and a binding otherwise.

An item using a construct the representation does not have is left out, and the construct is
reported as an `Error`: e.g. labels, `async` closures, `..base` in struct expressions, byte
literals, slice patterns, glob imports in blocks, other macro calls (`macro_rules!` of the crate
too) and `macro_rules!` or `mod` in bodies. `macro_rules!` and `mod` items are left out silently,
the items of a module are items of their own.

`codegen::compare::compare` checks an item's IR against the source: the item printed from it,
parsed again, has the syntax tree of the item as loaded (after cfg). Layout and comments do not
count, nor do the spellings the IR does not keep: literals are their values, trailing commas and
the delimiters of macro calls do not count. A mismatch is an `Error` showing the lines of the two
trees around the first difference.

For generators of other languages, `SourceCrate::lower` lowers the IR of a builder API into
`codegen::lowered`: classes, enums and functions with typed expressions. The roots are the root
builder, the traits whose impls to load and data types. Builders are the root and the structs
the `pub` methods of builders return (also as type arguments) that have a `pub` method with a
receiver. Data types are the types reached through fields, the signatures of builders and trait
impls, and the data roots. All their inherent impls are loaded. A construct the lowered IR does
not have (e.g. `for`, `?`, ranges, struct variants, destructuring `let`) is kept as
`K::Unsupported` and noted on its function.

`SourceCrate::lower_tests` lowers the `#[test]` functions of a test crate of that crate into
`Module::tests`, so a generator can translate them too: a path of the tested crate
(`agdb::QueryBuilder`) names its item in the module, `assert_eq!(a, b)` is the `AssertEq`
intrinsic, and the structs and enums the test crate declares are data types listed in
`Module::test_types`. The passes treat the tests as callers of the API.

`codegen::pass` holds the passes over the lowered IR. A generator lists the ones its target
needs in `Passes`, built-in or its own `Pass`, and can dump the module after each with
`codegen::print`:

- `Resolve` types the expressions and resolves what lowering left by name: method calls into
  member, associated, trait and std (intrinsic) calls, paths into variants and constants,
  `.into()`/`From` into conversions. The elements of an array argument get the type the
  non-generic `From` impls of the parameter's `Into` target agree on, as rustc infers it;
- `Prune` keeps what the `pub` methods of the builders, the trait impls and the tests call and
  reports the unsupported constructs of what it keeps, and the tests' calls of private methods,
  as `Error::Lowering`;
- `ExplicitReturns` turns tail expressions into `return`s;
- `Inline` replaces the calls of single-expression constructors and private helpers with their
  bodies and records them in `Module::inlined`, leaving the tests as they are;
- `LowerConversions` turns `G: Into<X>` generics into `Into(X)` parameters, dispatches `.into()`
  statically where the type is known and generates the `into_X` normalizers from
  `Module::conversions` otherwise, those only the tests need last;
- `TraitDicts` passes trait implementations as dictionaries: a field for class generics, a
  parameter for function generics and a global per named type (recorded in `Module::dicts`).
  Its `reflective` traits get the one dictionary the target runtime provides;
- `EraseOwnership` removes references, `mem::take`/`mem::swap` and assignments through pattern
  bindings, for targets that share objects by reference;
- `InferEmptyLists` types `vec![]` by what is pushed to it;
- `EraseNewtypes` replaces the single-field tuple data structs with their field, as serde
  serializes them.

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
