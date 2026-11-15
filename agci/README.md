# description

The `agci` utility drives the CI (see [ci](#ci)) and helps automate the release process. Without arguments it determines the current version from the latest `v*` git tag and the new one from the repository root (the root `Cargo.toml` `[workspace.package].version`), updates the versions in the manifests that have the current one (e.g. `package.json` files) and runs the `regenerate` hooks of the projects in dependency order (after `pnpm install --frozen-lockfile` if a pnpm project has one): `agdb_server/regenerate.sh` writes the OpenAPI spec and the test queries, the `regenerate` scripts of the API clients generate from them.

- agdb
- agdb_derive
- agdb_api
- @agnesoft/agdb_api

# usage

NOTE: Requires Rust toolchain and `pnpm` to be installed. It is recommended to use [`rustup`](https://www.rust-lang.org/tools/install) to get Rust and [NodeJS](https://nodejs.org/en) to get `npm` with which you can install the pnpm: `npm i -g pnpm`.

```
cargo run --release -p agci
```

# release

In order to release a new version of the packages in the repository:

1. Create an issue for the new release, e.g. `[ci] Release 1.2.3`.

2. Create a branch for the issue, e.g. `999-ci-release-1-2-3`.

3. Update the version in the root `Cargo.toml` (the `[workspace.package].version` value).

4. Run `agci` (NOTE: the pull request pipeline contains a validation that will fail if `Cargo.toml` changed but generated packages/artifacts do not match it, i.e. `agci` was not run.)

```
cargo run --release -p agci
```

5. Commit the result.

6. Open a pull request from the release branch to `main`.

7. After merge the release will happen automatically.

# ci

The `agdb` workflow checks only the projects affected by a pull request:

1. `agci affected --base origin/main` compares the fingerprint of each project at the merge base and at HEAD. A fingerprint is a hash of the project's `agci.toml` entry, the git objects of its files (the longest matching project directory owns a file), of its `dependencies` paths and of the files affecting all projects of its kind, together with the same of every project it transitively depends on. A project whose fingerprint differs is affected. The jobs of the affected projects are the matrix.
2. Each job runs `agci check <project>...` which runs the checks the projects declare. The checks fail if they modify tracked files or create new untracked ones (e.g. outdated `openapi.json` or generated clients). A job checks one project, or all affected projects of a `group` one after another (every project is checked, the failing ones are listed at the end). A project's checks stop at the first failure, the rest are skipped. The results (check, passed/failed/skipped, time) are printed at the end. On GitHub Actions each check is also a collapsible log group, a failure is an error annotation and the results are a table in the job summary.
3. The `ci` job aggregates the results, it is the only one to require in the branch protection.

The same commands work locally: `cargo run -p agci -- check` runs the checks of the projects affected by the uncommitted changes.

A job that passes saves its evidence to the actions cache, an empty entry keyed by the OS, the job and the fingerprints of its projects. `agci config` drops the jobs whose evidence the pull request already has, so re-runs and pushes that change only some projects check only those. The evidence is scoped to the pull request; when it is evicted the job runs again.

A push to main checks nothing (that happened in the pull request), it only releases via `agci release --projects <affected since before>` (`--base <before>` locally):

1. The `release:dev` hooks of the projects affected by the push, with `AGCI_VERSION=dev` (the studio package on the `dev` GitHub release and the `agnesoft/agdb:dev` image).
2. A release when there is no `v<version>` tag for the root `Cargo.toml` version, all projects in dependency order (`AGCI_VERSION=<version>`): their packages are published (crates with `cargo publish`, npm packages built if they have a `build` script, then `pnpm publish --provenance`), then the `release` hook runs. The GitHub release with the tag comes last, so re-running a failed release retries it; packages whose version is already in the registry are skipped, hooks should tolerate re-runs.

What is released follows from the manifests: a crate unless its `Cargo.toml` has `publish = false`, a `package.json` unless it is `"private": true`. Hooks are optional scripts of the project: pnpm `release`/`release:dev`/`regenerate` package.json scripts (turbo tasks), cargo and shell `release.sh`/`release_dev.sh`/`regenerate.sh` (or `.ps1`).

`--plan` prints the steps without running them.

The first job (`bootstrap`, Linux and Windows) provides the binary of each OS to the others as the `agci-<os>` artifact. The latest cached binary decides whether it is outdated (`agci bootstrap --check`): its fingerprint, the git objects of its sources (agci, the projects it depends on and the root Rust files), differs from the current one, or it fails to load the projects (e.g. configuration it does not know yet) or the check itself fails (e.g. a binary older than the command). Only then, or without any cached binary, agci is built from the sources (`cargo run -r -p agci -- bootstrap <dir>`) and cached. The `config` job is then `agci config`: it writes the job outputs for the GitHub event. Only the `config` job clones the full history (the base and the tags), the others are shallow and look up tags on `origin`.

## projects

Projects are detected from the manifests, their kind is the tool driving them: every Cargo workspace member (`cargo`) and `package.json` (`pnpm`, except the root one) that is not nested in a project of the same kind is a project. A nested one is folded into the nearest enclosing project of the same kind (e.g. all `agdb_studio/**` packages form the `agdb_studio` project). Projects are identified by their directory.

The configuration is one optional file, `agci.toml` in the repository root, holding only the deviations from the conventions. `[defaults.cargo]`, `[defaults.pnpm]` and `[defaults.shell]` apply to every project of the kind, `[projects."<dir>"]` overrides them field by field. Without them the default checks are:

- cargo: `format`, `lint`, `doc` (crates with a library only), `examples` (crates with example targets only, runs the tests in the examples), `bench` (crates with bench targets only, runs each once), `coverage`.
- pnpm: the tasks of the root `package.json` `check` script (`turbo run <task>...`, in that order), otherwise `format:check`, `build`, `lint:check`, `type-check`, `test`, `test:functional`.
- shell: `format`, `lint`, `build`, `test`.

A check that is not built in is a script: a pnpm package.json script, or for cargo and shell projects the `<name>.sh` (bash) or `<name>.ps1` (PowerShell, preferred on Windows) file in the project directory, `:` is `_` in the file name. A `run.sh`/`run.ps1` of a cargo or shell project is its last check (`run`) unless its `checks` place it. Default checks run only if the project can run them, its own `checks` must all exist. An entry makes a nested manifest its own project, and an entry for a directory without a manifest is a shell project. agci metadata left in a manifest is an error.

Dependencies come from the manifests (Cargo path dependencies, `workspace:`/`file:` package.json dependencies) and `dependencies` in `agci.toml`. Changes to the root `Cargo.toml`/`Cargo.lock` affect all cargo projects, changes to the root `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml` or `turbo.json` affect all pnpm projects and changes to `.github/workflows/agdb.yaml` affect everything. A change of `agci.toml` affects only the projects whose effective configuration (entry over defaults) changes, and their dependents; formatting and comments affect nothing. `agci projects` lists the projects with their dependencies and checks.

| field                  | applies to | description                                                                                                                                                                                                                                                                                           |
| ---------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `checks`               | all        | cargo: any of `format`, `lint`, `testcheck`, `doc`, `examples`, `bench`, `coverage` (always run in this order, all with `--all-features`), then scripts in the given order. pnpm: package.json scripts, run in the given order via turbo over all packages of the project. shell: scripts run in the given order. |
| `dependencies`         | all        | Additional dependencies: project directories (transitive) or repository paths.                                                                                                                                                                                                                        |
| `os`                   | all        | `linux` (default) and/or `windows`.                                                                                                                                                                                                                                                                   |
| `add_toolchains`       | all        | Toolchains installed besides the kind's own (cargo: `rust`, pnpm: `node`): `rust`, `node`, `php`, `coverage` (cargo llvm-cov).                                                                                                                                                                         |
| `group`                | all        | Check together with the other projects of the group in one job per OS (toolchains of all of them). Cheap projects only, where the job setup dominates; keep coverage projects separate.                                                                                                               |
| `coverage`             | cargo      | `false`: the `coverage` check only runs the tests (and is named `test`). By default it runs them under `cargo llvm-cov` requiring all lines, regions and functions covered, counting only the project's own sources.                                                                                  |
| `feature_combinations` | cargo      | Also lint the library and all targets with `--no-default-features --features <combination>` for each entry.                                                                                                                                                                                           |

```toml
[projects."libs/platform"]
os = ["linux", "windows"]
checks = ["format", "lint", "testcheck", "doc", "examples", "bench", "coverage"]
coverage = true
feature_combinations = ["", "fs", "fs,process"]
```
