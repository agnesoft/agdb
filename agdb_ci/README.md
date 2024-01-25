# description

The `agdb_ci` utility helps automate the release process. It determines the current version by reading the version of the `agdb` crate. It then reads the new version from the `Version` file in the root of the repository. Finally it updates all current versions of to the new one in all packages - Rust, Typescript etc. It bumps both versions of the packages (if present) and of the following dependencies (if declared). For typescript projects it addittionally runs `npm install` to update the dependencies (also updates `package-lock.json`):

- agdb
- agdb_derive
- agdb_api
- @agnesoft/agdb_api

# usage

NOTE: Requires Rust toolchain and `npm` to be installed. It is recommended to use [`rustup`](https://www.rust-lang.org/tools/install) to get Rust and [NodeJS](https://nodejs.org/en) to get `npm`.

```
cargo run --release -p agdb_ci
```

# release

In order to release a new version of the packages in the repository:

1. Create an issue for the new release, e.g. `[ci] Release 1.2.3`.

2. Create a branch for the issue, e.g. `999-ci-release-1-2-3`.

3. Update the `Version` file to the new version, e.g. `1.2.3`.

4. Run the `agdb_ci` (NOTE: the pull request pipelines contain a validation that will fail if the `Version` file changed but the packages do not match it, i.e. the `agdb_ci` was not run.)

```
cargo run --release -p agdb_ci
```

5. Commit the result.

6. Open a pull request from the release branch to `main`.

7. After merge the release will happen automatically.
