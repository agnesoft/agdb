# Rust Test Rules

Unit tests mirror the code: inside `#[cfg(test)] mod tests`, nested modules name the real types, traits, methods and functions under test, and each test is named `<subject>[_<args>]_should_<behaviour>`. Rust has no built-in test grouping, and generated (LLM) tests in particular drift into inconsistent structure and noisy comments, so the structural rules are enforced deterministically by `agtestcheck` rather than left to judgement. Coverage must be 100 % (see [Coverage](#coverage)).

Checked by `agtestcheck`:

- Modules name real types, traits, methods and functions only: `type → method`, `type → trait → method` or a standalone function.
- Tests are named `<module>[_<args>]_should_<behaviour>`.
- Cross-method tests sit directly in the type module, before its method modules.
- Comments in test bodies are only `GIVEN` / `WHEN` / `THEN` / `AND` steps or `WHY` explanations.
- No production code after the tests of a module.
- Every implementer of a trait tests each of its [contract cases](#contract-cases).

Checked by `cargo llvm-cov`:

- 100 % function, line and region coverage.

Left to judgement:

- Split tests on behaviour, not input; use a matrix for inputs of the same behaviour.
- Each test is self-contained and does only what its name claims.

## Running agtestcheck

```bash
# Install if not present
cargo install agtestcheck
cargo test-check [<dir|file.rs>] # defaults to the current directory
```

Every `.rs` file is checked, including integration tests, except cache directories tagged with a `CACHEDIR.TAG` such as cargo's `target`. Each issue is printed as `file:line: message` and any issue makes the exit code 1.

A test is a `#[test]` function or, within `#[cfg(test)]` code, a function with any attribute ending in `test` (e.g. `#[test]`, `#[tokio::test]`). Modules without tests (helpers) are ignored.

Tests may live in a separate test file, which is checked as the body of the `tests` module. A file is a test file when it is:

- an integration test or its helper (under `tests/` next to a `Cargo.toml`),
- marked `#![cfg(test)]`, or
- holding tests outside `#[cfg(test)]` modules, with no `#[cfg(test)]` test module of its own (e.g. the target of `#[cfg(test)] mod tests;`). Here any attribute ending in `test` makes a test.

A file without its own types, traits, functions or macros (a test file, or one with only a `#[cfg(test)] mod tests`) names the items of its whole crate, i.e. everything under the nearest `Cargo.toml`, with a type's impls merged across files. Any other file names only its own items.

## Structure

```rust
struct Cache { /* ... */ }
impl Cache { fn get(..) {..} fn insert(..) {..} }
impl std::fmt::Display for Cache { fn fmt(..) {..} }
fn normalize_path(..) {..}

#[cfg(test)]
mod tests {
    use super::*;

    mod cache {                                   // type
        use super::*;

        #[test]                                   // cross-method test, before method modules
        fn cache_should_evict_oldest_when_full() { /* insert + get */ }

        mod get {                                 // method
            use super::*;

            #[test]
            fn get_should_return_inserted_value() { /* ... */ }

            #[test]
            fn get_missing_key_should_return_none() { /* <args> qualify the subject */ }
        }

        mod std_fmt_display {                     // trait implemented by the type
            use super::*;

            mod fmt {                             // trait method
                use super::*;

                #[test]
                fn fmt_should_list_entries() { /* ... */ }
            }
        }
    }

    mod normalize_path {                          // standalone function or macro
        use super::*;

        #[test]
        fn normalize_path_should_resolve_parent_references() { /* ... */ }
    }
}
```

- **Tests come last.** Production code (any item not gated by `#[cfg(test)]`, incl. `use` and `impl`) comes before the tests of its module (a `#[cfg(test)]` module with tests, a `#[test]` or a module of `#[test]`s). Test helpers may come before or after them. Only the first such item of each module is reported.
- **Modules** name only items of the same file (of the crate for [test files](#running-agtestcheck)), plus the default methods of traits the file's types implement: `type → method`, `type → trait → method`, or a standalone function / `macro_rules!` macro. Names are snake cased (`FsImpl` → `fs_impl`). Never invent concept modules (`lifecycle`, `edge_cases`). A method gets its own module even for a single test.
- **Tests** live in the module of their subject and are named after it: `<module>_should_…` or `<module>_<args>_should_…`. The name repeats the module so flat `cargo test` output stays readable.
- **Cross-method tests** of a type sit directly in the type module, before its method modules. Cross-type tests belong in integration tests (`tests/`), still under the module of the item they exercise.
- **Trait modules** follow the trait path as written, snake cased with `::` → `_` and generics dropped: `impl fmt::Write` → `fmt_write`, `impl From<A>` and `impl From<B>` share `from`. This keeps an inherent `fmt` apart from `Display::fmt`, and `Debug::fmt` apart from `Display::fmt`.
  - Trait methods are tested in the implementer, overridden or not: a default method the type does not override still goes in `type → trait → method`. The trait may be defined in another file.
  - A trait implementation with at most one method (`impl Drop`, `impl Error for E {}`) may hold its tests directly in the trait module, named after the trait or the method (`x::drop::drop_should_…`, `x::std_fmt_display::fmt_should_…`).
  - Blanket implementations (`impl<T: Bound> Trait for T`) and implementations for types not defined in the file (`impl MyTrait for Vec<u8>`) are tested under the trait at the top level: `trait → method`.

## Contract cases

Implementations of the same trait (e.g. a real and an in-memory one) drift apart when each has its own tests: a scenario tested for one is easily forgotten for the other, and coverage cannot tell, as the behaviour often lives in the OS or a dependency rather than in the code. A trait can therefore define its contract once, as contract cases that each implementer must run.

A **case** is a function of test code that is not a test but is named like one (`<method>[_args]_should_<behaviour>`). Cases live in the method modules of the trait's module at the root of the tests, in the file defining the trait (or its test file), and take whatever the trait needs as parameters:

```rust
pub trait Fs { fn copy(..); fn exists(..) -> bool { .. } }

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) mod fs {                           // trait
        use super::*;

        pub(crate) mod copy {                     // method
            use super::*;

            pub(crate) fn copy_should_overwrite_target(fs: &impl Fs, root: &Path) { /* ... */ }
        }
    }
}
```

Every implementer of the trait in the crate then has a test of the same name in its `type → trait → method` module (or the trait module for a trait with a single method) that calls the case, by path so the test does not call itself:

```rust
mod fs_impl {
    mod fs {
        mod copy {
            #[test]
            fn copy_should_overwrite_target() {
                let dir = TempDir::new();
                crate::fs::tests::fs::copy::copy_should_overwrite_target(&FsImpl, dir.path());
            }
        }
    }
}
```

- `cargo test fs::copy::copy_should_overwrite_target` runs the case for every implementer, and a failure names the implementer in its path.
- Cases apply to default methods too, as a default's behaviour depends on the methods the implementer provides.
- An implementer that cannot support a case keeps the test without code and with only a WHY comment explaining why:

  ```rust
  #[test]
  fn copy_should_keep_owner() {
      // WHY: TestFs has no owners.
  }
  ```

  It is a real test rather than an ignored one, so it runs and its body is covered. A test ignored on some platforms only (`#[cfg_attr(windows, ignore = "…")]`) runs elsewhere, so it still calls the case.

- Implementer-specific tests (e.g. fault injection of the in-memory one) are not affected.
- An implementer names the trait by its name or by a path from the crate or one of its modules (`Fs`, `crate::fs::Fs`, `fs::Fs`). Blanket impls (`impl<T: Bound> Trait for T`), impls for types defined in another file and test-only types (mocks) are not implementers, so keep an `impl` in the file of its type. Implementers in other crates are not checked.
- Where implementers legitimately differ (e.g. per platform), the case asserts only the contract: leave unspecified behaviour unasserted, or take the expectation from the parameters.

Each missing test is reported at the `impl` of the implementer, a test neither calling its case nor explaining why not in only a WHY comment at the test, and a case outside the method module of a trait at the case.

## Comments

Inside test bodies a comment must be either:

- a step: starts with `GIVEN`, `WHEN`, `THEN` or `AND` (`// GIVEN …`, `// GIVEN: …`). `AND` cannot come first and `GIVEN` cannot follow `WHEN` or `THEN`. A step may wrap over consecutive `//` lines.
- a `WHY` comment explaining what the code cannot show, e.g. a platform or spec constraint: `// WHY: POSIX rejects opening a file with both append and truncate`.

Any other comment is reported: delete it, or rewrite it as `WHY` if it carries meaning not present in the code.

Use steps when the setup is non-trivial (mocks, multi-step wiring, filesystem scaffolding) so it is clear where setup ends and the action begins:

```rust
#[tokio::test]
async fn connect_should_retry_after_transient_failure() {
    // GIVEN a server that fails once then succeeds
    let server = MockServer::start().await;
    server.enqueue_failure(503);
    server.enqueue_success();

    // WHEN connecting with retry enabled
    let result = Client::new(server.url(), RetryPolicy::Once).connect().await;

    // THEN the connection succeeds on the second attempt
    assert!(result.is_ok());
    assert_eq!(server.request_count(), 2);
}
```

## Left to judgement

- **Split on behaviour, not input.** A new test only when the expected behaviour or code path differs (resolving relative paths vs rejecting invalid ones). Different inputs to the same behaviour go into one test as a matrix, with the input in the assertion message:

  ```rust
  #[test]
  fn parse_bool_should_accept_truthy_representations() {
      for input in ["1", "true", "TRUE", "yes", "on"] {
          assert_eq!(parse_bool(input), Ok(true), "input: {input}");
      }
  }
  ```

  Use a local `fn check(input, expected)` when per-case setup is more than a one-liner.

- **Self-contained.** Each test builds its own state; no shared fixtures or ordering dependencies. Local helper functions are fine.
- **Do what the name claims.** The body does only what its name claims, using the simplest verification path. Losing coverage when splitting a monolithic test is expected: it exposes methods that were only covered incidentally and need their own tests.
- **Same claims, different setup.** Multiple implementations of a trait (real and in-memory) may have tests with identical names; the module path tells them apart.

## Coverage

100 % function, line, and region coverage is enforced.

```bash
# Summary
cargo llvm-cov --all-features

# HTML with per-instantiation breakdown
cargo llvm-cov --all-features --html --open --show-instantiations

# CI gate — exits 1 on any uncovered line, function, or region
cargo llvm-cov --all-features \
  --fail-uncovered-lines 0 \
  --fail-uncovered-functions 0 \
  --fail-uncovered-regions 0
```

A function is covered if any of its instantiations is called. Lines and regions are **not** merged: per source function, `llvm-cov` counts only the instantiation covering the most regions (and, separately, the one covering the most lines), so a region covered by one instantiation still counts as missed when the best covered one misses it. The HTML merges instantiations when rendering, so the source can look green while the summary reports missed lines and regions.

### Monomorphization and coverage

When tests call the same generic function with different concrete types, the compiler emits a separate copy for each, with its own regions. If different tests exercise different code paths through different instantiations, no single copy is fully covered and the gate fails even though every path ran.

```rust
// Bad — creates two instantiations, each partially covered:
Error::fs("literal");           // instantiates Error::fs::<&str>
Error::fs(format!("dynamic"));  // instantiates Error::fs::<String>

// Good — one instantiation, must be fully covered:
Error::fs(format!("literal"));  // always String
Error::fs(format!("dynamic"));  // always String
```

**Fix:** use a single concrete type per generic parameter across all tests, so only one instantiation exists and it must be fully covered. If multiple instantiations are intentional, exercise every code path for one of them.

### Diagnosing missed regions with the JSON export

The JSON `data[0].functions` array has one entry per **instantiation** (not per source function) and, unlike `data[0].files`, is not filtered by `--ignore-filename-regex` or the default exclusion of `tests/` and `examples/`. Each has a mangled `name`, `filenames`, `count` (times entered), and `regions`, which are tuples of `[line_start, col_start, line_end, col_end, execution_count, file_id, expanded_file_id, kind]`. Instantiations of one source function share the location of their first region, and a region with `execution_count == 0` and `kind == 0` (Code) is uncovered.

This script prints exactly what the gate counts: per source function the missed regions of its best covered instantiation, each marked `uncovered` (a real gap, write a test) or `covered only by` another instantiation (a monomorphization split, unify the types). Names are demangled when `llvm-cxxfilt` or `rustfilt` is on the `PATH`. Clean first, as stale binaries of older sources stay in the target directory and their records, at outdated locations and without counts, are reported too:

```bash
cargo llvm-cov clean --workspace
cargo llvm-cov -p <crate> --all-features --no-report
cargo llvm-cov report -p <crate> --json --output-path coverage.json
python3 - coverage.json <<'EOF'
import json, os, re, shutil, subprocess, sys
from collections import defaultdict

data = json.load(open(sys.argv[1]))['data'][0]
cwd = os.getcwd()
# WHY: the ignored files are filtered out of `files` only.
reported = {f['filename'] for f in data['files']}

# Instantiations of one source function share its start location.
groups = defaultdict(list)
for f in data['functions']:
    if f['filenames'][0] not in reported:
        continue
    start = next(r for r in f['regions'] if r[5] == 0)
    groups[(f['filenames'][0], start[0], start[1])].append(f)

demangler = shutil.which('llvm-cxxfilt') or shutil.which('rustfilt')
names = [f['name'] for fs in groups.values() for f in fs]
demangled = {n: n for n in names}
if demangler:
    out = subprocess.run([demangler], input='\n'.join(names),
                         capture_output=True, text=True).stdout.split('\n')
    demangled.update(zip(names, out))

def code(f):
    return [r for r in f['regions'] if r[7] == 0]

def loc(f, r):
    path = os.path.relpath(f['filenames'][r[5]], cwd)
    return f'{path}:{r[0]}:{r[1]}-{r[2]}:{r[3]}'

real = split = never = 0
for (file, line, _), instances in sorted(groups.items()):
    # WHY: llvm-cov counts the regions (and lines) of the instantiation
    # covering the most, not their union.
    best = max(instances, key=lambda f: sum(r[4] > 0 for r in code(f)))
    missed = [r for r in code(best) if r[4] == 0]
    if not missed:
        continue
    print(f'{os.path.relpath(file, cwd)}:{line} {demangled[best["name"]]}'
          f' ({len(instances)} instantiations)')
    if best['count'] == 0:
        never += len(missed)
        print(f'  never called ({len(missed)} regions)')
        continue
    for r in missed:
        covering = [f for f in instances
                    if any(o[:4] == r[:4] and o[4] > 0 for o in code(f))]
        if covering:
            split += 1
            print(f'  {loc(best, r)}  covered only by {demangled[covering[0]["name"]]}')
        else:
            real += 1
            print(f'  {loc(best, r)}  uncovered')

print(f'{real + split + never} missed regions: {real} uncovered, {split} covered '
      f'only by another instantiation, {never} in functions never called')

# WHY: builds of the same crate differ only in the crate hash, so one
# name at several locations means a build of older sources.
starts = defaultdict(set)
for key, instances in groups.items():
    for f in instances:
        starts[re.sub(r'Cs[0-9A-Za-z]*_|17h[0-9a-f]{16}E', '', f['name'])].add(key)
if any(len(keys) > 1 for keys in starts.values()):
    print('warning: stale builds found, run `cargo llvm-cov clean --workspace`')
EOF
```

Typical output:

```
src/error.rs:61 agplatform::error::Error::fs::<alloc::string::String> (2 instantiations)
  src/error.rs:63:19-63:32  covered only by agplatform::error::Error::fs::<&str>
src/fs.rs:2315 <agplatform::fs::FsImpl as agplatform::fs::Fs>::copy (1 instantiations)
  src/fs.rs:2321:47-2321:48  uncovered
src/lib.rs:72 agplatform::platform (1 instantiations)
  never called (3 regions)
5 missed regions: 1 uncovered, 1 covered only by another instantiation, 3 in functions never called
```
