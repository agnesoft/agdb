# description

The `agci` utility drives the CI (see [ci](#ci)) and helps automate the release process. It determines the current version from the latest `v*` git tag and reads the new version from the root `Cargo.toml` (`[workspace.package].version`). It then updates version strings in workspace packages (e.g. TypeScript `package.json` files) and regenerates OpenAPI/client artifacts and test queries. For TypeScript projects it also runs `pnpm install`, which updates `pnpm-lock.yaml`.

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

1. `agci affected --base origin/main` maps the changed files to projects (the longest matching project directory owns a file), adds the projects that declare a changed path in `depends` and then, transitively, every project depending on any of those. The result is the job matrix.
2. Each job runs `agci check <project>...` which runs the checks the projects declare. The checks fail if they modify tracked files or create new untracked ones (e.g. outdated `openapi.json` or generated clients). A job checks one project, or all affected projects of a `group` one after another (every project is checked, the failing ones are listed at the end). A project's checks stop at the first failure, the rest are skipped. The results (check, passed/failed/skipped, time) are printed at the end. On GitHub Actions each check is also a collapsible log group, a failure is an error annotation and the results are a table in the job summary.
3. The `ci` job aggregates the results, it is the only one to require in the branch protection.

The same commands work locally: `cargo run -p agci -- check` runs the checks of the projects affected by the uncommitted changes.

A push to main checks nothing (that happened in the pull request), it only releases via `agci release --projects <affected since before>` (`--base <before>` locally):

1. The `release_dev` scripts of the projects affected by the push, with `AGCI_VERSION=dev` (the studio package on the `dev` GitHub release and the `agnesoft/agdb:dev` image).
2. A release when there is no `v<version>` tag for the root `Cargo.toml` version: the `release` targets of all projects in dependency order (`AGCI_VERSION=<version>`), then the GitHub release with the tag. The tag comes last, so re-running a failed release retries it; crates already on crates.io are skipped, scripts should tolerate re-runs.

`--plan` prints the steps without running them.

The first job (`bootstrap`) provides the binary to the others as an artifact. It is cached under a hash of its sources (agci, its path dependencies and the root `Cargo.toml`), so it is only rebuilt when those change. Windows jobs build it themselves. Only the `config` job clones the full history (diffs against the base and the tags), the others are shallow and look up tags on `origin`.

## projects

A project is a `Cargo.toml` with `[package.metadata.agci]` or a `package.json` with an `"agci"` object. A manifest without it is folded into the nearest enclosing project of the same language (e.g. all `agdb_studio/**` packages form the `agdb_studio` project). A manifest that cannot be attributed to a project is an error. Projects are identified by their directory.

Dependencies come from the manifests (Cargo path dependencies, `workspace:`/`file:` package.json dependencies). Changes to the root `Cargo.toml`/`Cargo.lock` affect all Rust projects, changes to the root `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml` or `turbo.json` affect all Node projects and changes to `.github/workflows/agdb.yaml` affect everything.

| field                  | applies to | description                                                                                                                                                                                   |
| ---------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `checks`               | all        | Rust: any of `format`, `lint`, `testcheck`, `doc`, `bench`, `test` (always run in this order). Node: package.json scripts, run in the given order via turbo over all packages of the project. |
| `depends`              | all        | Additional dependencies: project directories (transitive) or repository paths.                                                                                                                |
| `os`                   | all        | `linux` (default) and/or `windows`.                                                                                                                                                           |
| `toolchains`           | all        | Extra toolchains: `rust`, `node`, `php`.                                                                                                                                                      |
| `group`                | all        | Check together with the other projects of the group in one job per OS (toolchains of all of them). Cheap projects only, where the job setup dominates; keep coverage projects separate.       |
| `features`             | Rust       | Features to use instead of `--all-features`.                                                                                                                                                  |
| `coverage`             | Rust       | Run `test` under `cargo llvm-cov` counting only the project's own sources. `100` requires all lines, regions and functions covered, `0` only reports.                                         |
| `feature_combinations` | Rust       | Also lint the library and all targets with `--no-default-features --features <combination>` for each entry.                                                                                   |
| `run`                  | Rust       | Extra shell commands run in the project directory after the checks.                                                                                                                           |
| `release`              | all        | Run on a release. Rust: `crates` (`cargo publish`). Node: package.json scripts.                                                                                                               |
| `release_dev`          | Node       | package.json scripts run on every push to main that affects the project.                                                                                                                      |

```toml
[package.metadata.agci]
os = ["linux", "windows"]
checks = ["format", "lint", "testcheck", "doc", "bench", "test"]
coverage = 100
feature_combinations = ["", "fs", "fs,process"]
```
